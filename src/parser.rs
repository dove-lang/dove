use crate::ast::{Expr, Stmt};
use crate::token::{Token, TokenType, Literals};

pub struct ParseError {
    pub message: String,
}

pub type Result<T> = std::result::Result<T, ParseError>;

// Precondition: tokens.len() > 0
pub struct Parser {
    current: usize,
    tokens: Vec<Token>,
    /// If this is true, automatically skips newline after advance.
    ignore_newline: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            current: 0,
            tokens,
            ignore_newline: false,
        }
    }

    pub fn program(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = vec![];

        while !self.is_at_end() {
            statements.push(self.declaration()?);
            self.skip_newlines();
        }

        self.advance();

        Ok(statements)
    }
}

// TODO: figure out where to skip newlines

// Declarations / Statements
impl Parser {
    fn declaration(&mut self) -> Result<Stmt> {
        self.skip_newlines();

        match self.peek().token_type {
            TokenType::CLASS => self.class_decl(),
            TokenType::FUN => self.fun_decl(),
            TokenType::LET => self.var_decl(),
            _ => self.statement(),
        }
    }

    fn class_decl(&mut self) -> Result<Stmt> {
        self.consume(TokenType::CLASS)?;
        let identifier = self.consume(TokenType::IDENTIFIER)?;
        let superclass = if self.consume(TokenType::FROM).is_ok() {
            Some(self.consume(TokenType::IDENTIFIER)?)
        } else {
            None
        };

        self.consume(TokenType::LEFT_BRACE)?;
        self.skip_newlines();

        let mut functions = vec![];
        while !self.check(TokenType::RIGHT_BRACE) {
            functions.push(self.fun_decl()?);
            self.skip_newlines();
        }

        self.consume(TokenType::RIGHT_BRACE)?;

        Ok(Stmt::Class(identifier, superclass, functions))
    }

    fn fun_decl(&mut self) -> Result<Stmt> {
        self.consume(TokenType::FUN)?;
        let identifier = self.consume(TokenType::IDENTIFIER)?;
        self.consume(TokenType::LEFT_PAREN)?;

        // Allow newlines in arguments
        let prev = self.set_ignore_newline(true);
        let parameters = self.parameters()?;
        self.set_ignore_newline(prev);

        self.consume(TokenType::RIGHT_PAREN)?;
        let block = self.block()?;

        Ok(Stmt::Function(identifier, parameters, Box::new(block)))
    }

    fn var_decl(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LET)?;
        let variable = self.consume(TokenType::IDENTIFIER)?;
        let expr = if self.consume(TokenType::EQUAL).is_ok() {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume_newline()?;
        Ok(Stmt::Variable(variable, expr))
    }

    fn statement(&mut self) -> Result<Stmt> {
        match self.peek().token_type {
            TokenType::LEFT_BRACE => self.block(),
            TokenType::FOR => self.for_stmt(),
            TokenType::PRINT => self.print_stmt(),
            TokenType::RETURN => self.return_stmt(),
            TokenType::WHILE => self.while_stmt(),
            TokenType::BREAK => self.break_stmt(),
            _ => self.expr_stmt(),
        }
    }

    fn block(&mut self) -> Result<Stmt> {
        self.skip_newlines();

        self.consume(TokenType::LEFT_BRACE)?;
        self.skip_newlines();
        let prev = self.set_ignore_newline(false);

        let mut statements = vec![];
        while !self.check(TokenType::RIGHT_BRACE) {
            statements.push(self.declaration()?);
            self.skip_newlines();
        }

        self.set_ignore_newline(prev);
        self.consume(TokenType::RIGHT_BRACE)?;
        Ok(Stmt::Block(statements))
    }

    fn for_stmt(&mut self) -> Result<Stmt> {
        self.consume(TokenType::FOR)?;
        let variable = self.consume(TokenType::IDENTIFIER)?;
        self.consume(TokenType::IN)?;
        let collection = self.consume(TokenType::IDENTIFIER)?;
        let block = self.block()?;
        Ok(Stmt::For(variable, collection, Box::new(block)))
    }

    fn print_stmt(&mut self) -> Result<Stmt> {
        self.consume(TokenType::PRINT)?;
        let expr = self.expression()?;
        self.consume_newline()?;
        Ok(Stmt::Print(expr))
    }

    fn return_stmt(&mut self) -> Result<Stmt> {
        self.consume(TokenType::RETURN)?;
        let expr = self.expression()?;
        self.consume_newline()?;
        Ok(Stmt::Return(expr))
    }

    fn while_stmt(&mut self) -> Result<Stmt> {
        self.consume(TokenType::WHILE)?;
        let condition = self.expression()?;
        let block = self.block()?;
        Ok(Stmt::While(condition, Box::new(block)))
    }

    fn break_stmt(&mut self) -> Result<Stmt> {
        self.consume(TokenType::BREAK)?;
        self.consume_newline()?;
        Ok(Stmt::Break)
    }

    fn expr_stmt(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::NEWLINE)?;
        Ok(Stmt::Expression(expr))
    }
}

// Expressions
impl Parser {
    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        // TODO: finish after call
        if self.check(TokenType::IDENTIFIER) && self.peek_next().token_type == TokenType::EQUAL {
            let variable = self.consume(TokenType::IDENTIFIER)?;
            self.consume(TokenType::EQUAL)?;
            let expr = self.if_expr()?;

            Ok(Expr::Assign(variable, Box::new(expr)))
        } else {
            self.if_expr()
        }
    }

    fn if_expr(&mut self) -> Result<Expr> {
        if self.consume(TokenType::IF).is_ok() {
            let condition = self.logic_or()?;

            // Then branch must be a block
            let then_stmt = self.block()?;

            // Optional else/else if branch
            let else_stmt = match self.consume(TokenType::ELSE) {
                Ok(_) => {
                    // Continue with else if branch
                    if self.peek().token_type == TokenType::IF {
                        Stmt::Expression(self.if_expr()?)
                    } else {
                        // End with else branch
                        self.block()?
                    }
                },
                Err(_) => Stmt::Block(vec![]),
            };

            Ok(Expr::IfExpr(Box::new(condition), Box::new(then_stmt), Box::new(else_stmt)))

        } else {
            self.logic_or()
        }
    }

    fn logic_or(&mut self) -> Result<Expr> {
        let mut left = self.logic_and()?;

        while let Some(op) = self.match_token(&[TokenType::OR]) {
            let right = self.logic_and()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn logic_and(&mut self) -> Result<Expr> {
        let mut left = self.equality()?;

        while let Some(op) = self.match_token(&[TokenType::PLUS, TokenType::AND]) {
            let right = self.equality()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut left = self.comparison()?;

        while let Some(op) = self.match_token(&[TokenType::EQUAL_EQUAL, TokenType::BANG_EQUAL]) {
            let right = self.comparison()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut left = self.addition()?;

        while let Some(op) = self.match_token(&[
            TokenType::LESS,
            TokenType::GREATER,
            TokenType::LESS_EQUAL,
            TokenType::GREATER_EQUAL,
        ]) {
            let right = self.addition()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn addition(&mut self) -> Result<Expr> {
        let mut left = self.multiplication()?;

        while let Some(op) = self.match_token(&[TokenType::PLUS, TokenType::MINUS]) {
            let right = self.multiplication()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn multiplication(&mut self) -> Result<Expr> {
        let mut left = self.unary()?;

        while let Some(op) = self.match_token(&[TokenType::STAR, TokenType::SLASH]) {
            let right = self.unary()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr> {
        let mut unary_ops = vec![];

        while let Some(op) = self.match_token(&[TokenType::BANG, TokenType::MINUS]) {
            unary_ops.push(op);
        }

        let mut expr = self.call()?;

        for op in unary_ops.into_iter().rev() {
            expr = Expr::Unary(op, Box::new(expr));
        }

        Ok(expr)
    }

    fn call(&mut self) -> Result<Expr> {
        // TODO: add call
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
        if let Some(token) = self.match_token(&[
            TokenType::STRING,
            TokenType::NUMBER,
        ]) {
            Ok(Expr::Literal(token.literal.unwrap()))

        } else if self.consume(TokenType::TRUE).is_ok() {
            Ok(Expr::Literal(Literals::Boolean(true)))

        } else if self.consume(TokenType::FALSE).is_ok() {
            Ok(Expr::Literal(Literals::Boolean(false)))

        } else if self.consume(TokenType::NIL).is_ok() {
            Ok(Expr::Literal(Literals::Nil))

        } else if let Ok(token) = self.consume(TokenType::IDENTIFIER) {
            Ok(Expr::Variable(token))

        } else if self.consume(TokenType::LEFT_PAREN).is_ok() {
            // Ignore newlines when directly within a group
            let prev = self.set_ignore_newline(true);
            let expr = self.expression()?;
            self.set_ignore_newline(prev);

            self.consume(TokenType::RIGHT_PAREN)?;

            Ok(expr)

        } else {
            // TODO: add exprs like super, self, etc.
            Err(ParseError {
                message: format!("unexpected token {:?}", self.peek()),
            })
        }
    }
}

// Other parsing methods
impl Parser {
    fn parameters(&mut self) -> Result<Vec<Token>> {
        let mut parameters = vec![];

        loop {
            if let Ok(token) = self.consume(TokenType::IDENTIFIER) {
                parameters.push(token);

                if self.consume(TokenType::COMMA).is_ok() {
                    continue;
                }
            }
            break;
        }

        Ok(parameters)
    }

    fn arguments(&mut self) -> Result<Vec<Expr>> {
        let mut arguments = vec![];

        loop {
            if let Ok(expr) = self.expression() {
                arguments.push(expr);

                if self.consume(TokenType::COMMA).is_ok() {
                    continue;
                }
            }
            break;
        }

        Ok(arguments)
    }
}

// Helper functions
impl Parser {
    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn check(&self, token_type: TokenType) -> bool {
        !self.is_at_end() && self.peek().token_type == token_type
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_next(&self) -> &Token {
        if self.ignore_newline {
            // Need to skip past newlines if ignore newline is true
            let mut index = self.current + 1;
            while self.tokens[index].token_type == TokenType::NEWLINE && index < self.tokens.len() {
                index += 1;
            }

            &self.tokens[index]
        } else {
            &self.tokens[self.current + 1]
        }
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> Token {
        let token = self.peek().clone();

        if !self.is_at_end() {
            self.current += 1;

            if self.ignore_newline {
                self.skip_newlines();
            }
        }

        token
    }

    /// Return the current token and advance if it is one of the given types. Otherwise return None.
    fn match_token(&mut self, token_types: &[TokenType]) -> Option<Token> {
        for &token_type in token_types {
            if self.check(token_type) {
                return Some(self.advance());
            }
        }
        None
    }

    /// Consume the currect token and return it if the token type matches. Otherwise returns an error.
    fn consume(&mut self, token_type: TokenType) -> Result<Token> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            let token = self.peek();

            Err(ParseError {
                message: format!("expected type {:?} for token {:?}", token_type, token),
            })
        }
    }

    /// Consume at least one newline and skip the rest
    fn consume_newline(&mut self) -> Result<()> {
        self.consume(TokenType::NEWLINE)?;
        self.skip_newlines();
        Ok(())
    }

    fn skip_newlines(&mut self) {
        while self.consume(TokenType::NEWLINE).is_ok() {}
    }

    /// Set a new value for ignore_newline, skips newline if it is true, and return the previous value.
    fn set_ignore_newline(&mut self, value: bool) -> bool {
        let prev = self.ignore_newline;
        self.ignore_newline = value;

        if self.ignore_newline {
            self.skip_newlines();
        }

        prev
    }
}
