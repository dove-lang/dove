use std::rc::Rc;

use crate::ast::{Expr, Stmt};
use crate::token::{Token, TokenType, Literals};
use crate::error_handler::CompiletimeErrorHandler;
use crate::dove_output::DoveOutput;

#[derive(Debug)]
enum ParseError {
    Token(Token, String),
    Line(usize, String),
}

type Result<T> = std::result::Result<T, ParseError>;

// Precondition: tokens.len() > 0
pub struct Parser {
    current: usize,
    tokens: Vec<Token>,
    /// If this is true, automatically skips newline after advance.
    ignore_newline: bool,

    /// If `is_in_repl` is true, do not consider unterminated block as error,
    /// set `unfinished_blk` to true,
    /// when given source is parsed correctly, set `unfinished_blk` back to false.
    is_in_repl: bool,
    pub is_in_unfinished_blk: bool,

    error_handler: CompiletimeErrorHandler,

    /// Indicates how "deep" the parser currently is nested in (), [], and {}.
    /// Automatically updates when `Parser.advance` is called.
    /// Used in `Parser.sychronize` to determine when to stop synchronizing.
    nested_level: u32,
    /// The nested level of the parsing statement
    statement_nested_level: u32,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, is_in_repl: bool, output: Rc<dyn DoveOutput>) -> Parser {
        Parser {
            current: 0,
            tokens,
            ignore_newline: false,
            is_in_repl,
            is_in_unfinished_blk: false,
            error_handler: CompiletimeErrorHandler {
                had_error: false,
                output,
            },
            nested_level: 0,
            statement_nested_level: 0,
        }
    }

    pub fn program(&mut self) -> Vec<Stmt> {
        let mut statements = vec![];

        self.skip_newlines();

        while !self.is_at_end() {
            if let Some(statement) = self.declaration() {
                if self.consume_newline().is_ok() {
                    statements.push(statement);
                } else {
                    self.handle_newline_error();
                }
            }
        }

        self.advance();

        statements
    }

    fn handle_error(&mut self, error: ParseError) {
        self.synchronize();

        match error {
            ParseError::Token(token, message) => {
                if self.is_in_repl && message.contains("RIGHT_BRACE") {
                    self.is_in_unfinished_blk = true;
                    return;
                }

                self.error_handler.token_error(token, message)
            },
            ParseError::Line(line, message) => self.error_handler.line_error(line, message),
        }
    }

    fn handle_newline_error(&mut self) {
        self.handle_error(ParseError::Token(self.peek().clone(), "Expected newline after statement.".to_string()));
    }

    /// Synchronize an error, skip tokens until end of current statement and same nested level as statement.
    fn synchronize(&mut self) {
        while !self.is_at_end() {
            let token = self.advance();
            if self.nested_level <= self.statement_nested_level && token.token_type == TokenType::NEWLINE {
                break;
            }
        }

        self.set_ignore_newline(false);
    }
}

// Declarations / Statements
impl Parser {
    fn declaration(&mut self) -> Option<Stmt> {
        self.skip_newlines();

        self.statement_nested_level = self.nested_level;

        let declaration = match self.peek().token_type {
            TokenType::CLASS => self.class_decl(),
            TokenType::FUN => self.fun_decl(),
            TokenType::LET => self.var_decl(),
            _ => self.statement(),
        };

        // Handle error in declaration
        match declaration {
            Ok(declaration) => Some(declaration),
            Err(error) => {
                self.handle_error(error);
                None
            },
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
        while !self.check(TokenType::RIGHT_BRACE) && !self.is_at_end() {
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

        Ok(Stmt::Variable(variable, expr))
    }

    fn statement(&mut self) -> Result<Stmt> {
        match self.peek().token_type {
            TokenType::LEFT_BRACE => {
                // Try to parse a dictionary. If it doesn't work, then parse block
                let current = self.current;
                let nested_level = self.nested_level;
                self.consume(TokenType::LEFT_BRACE)?;

                let prev = self.set_ignore_newline(true);
                let exprs = self.key_value_pairs();
                self.set_ignore_newline(prev);

                // Check if can parse key value pairs
                if let Ok(exprs) = exprs {
                    if self.consume(TokenType::RIGHT_BRACE).is_ok() {
                        return Ok(Stmt::Expression(Expr::Dictionary(exprs)));
                    }
                }

                // Backtrack and parse a block instead
                self.current = current;
                self.nested_level = nested_level;
                self.block()
            },
            TokenType::FOR => self.for_stmt(),
            TokenType::PRINT => self.print_stmt(),
            TokenType::RETURN => self.return_stmt(),
            TokenType::WHILE => self.while_stmt(),
            TokenType::BREAK => self.break_stmt(),
            TokenType::CONTINUE => self.continue_stmt(),
            _ => self.expr_stmt(),
        }
    }

    fn block(&mut self) -> Result<Stmt> {
        self.skip_newlines();

        self.consume(TokenType::LEFT_BRACE)?;
        self.skip_newlines();
        let prev = self.set_ignore_newline(false);

        let mut statements = vec![];
        while !self.check(TokenType::RIGHT_BRACE) && !self.is_at_end() {
            if let Some(statement) = self.declaration() {
                statements.push(statement);

                if self.consume_newline().is_err() {
                    // No newline as separator, cannot parse more statements
                    if self.check(TokenType::RIGHT_BRACE) {
                        break;
                    } else {
                        // Attempting to start another statement without newline - error
                        self.handle_newline_error();
                    }
                }
            }
        }

        self.set_ignore_newline(prev);
        self.consume(TokenType::RIGHT_BRACE)?;
        Ok(Stmt::Block(statements))
    }

    fn for_stmt(&mut self) -> Result<Stmt> {
        self.consume(TokenType::FOR)?;
        let variable = self.consume(TokenType::IDENTIFIER)?;
        self.consume(TokenType::IN)?;
        let expr = self.logic_or()?;
        let block = self.block()?;
        Ok(Stmt::For(variable, expr, Box::new(block)))
    }

    fn print_stmt(&mut self) -> Result<Stmt> {
        let token = self.consume(TokenType::PRINT)?;
        let expr = self.expression()?;
        Ok(Stmt::Print(token, expr))
    }

    fn return_stmt(&mut self) -> Result<Stmt> {
        let token = self.consume(TokenType::RETURN)?;

        if self.check(TokenType::NEWLINE) || self.check(TokenType::RIGHT_BRACE) {
            Ok(Stmt::Return(token, None))
        } else {
            let expr = self.expression()?;
            Ok(Stmt::Return(token, Some(expr)))
        }
    }

    fn while_stmt(&mut self) -> Result<Stmt> {
        self.consume(TokenType::WHILE)?;
        let condition = self.expression()?;
        let block = self.block()?;
        Ok(Stmt::While(condition, Box::new(block)))
    }

    fn break_stmt(&mut self) -> Result<Stmt> {
        let token = self.consume(TokenType::BREAK)?;
        Ok(Stmt::Break(token))
    }

    fn continue_stmt(&mut self) -> Result<Stmt> {
        let token = self.consume(TokenType::CONTINUE)?;
        Ok(Stmt::Continue(token))
    }

    fn expr_stmt(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        Ok(Stmt::Expression(expr))
    }
}

// Expressions
impl Parser {
    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.lambda()?;

        match self.peek().token_type {
            TokenType::EQUAL | TokenType::PLUS_EQUAL | TokenType::MINUS_EQUAL | TokenType::STAR_EQUAL | TokenType::SLASH_EQUAL |
            TokenType::PLUS_PLUS | TokenType::MINUS_MINUS => {
                let sign = self.advance();

                // If ++ or --, make value Number(1.0).
                let value = match (&sign).token_type {
                    TokenType::PLUS_PLUS | TokenType::MINUS_MINUS => Expr::Literal(Literals::Number(1.0)),
                    // If there is equal sign, parse assignment
                    // Parse expression here to allow assigning an assign expression
                    _ => self.expression()?,
                };

                // Check whether assign to variable or set object property
                return match expr {
                    Expr::Get(obj, name) => Ok(Expr::Set(obj, name, Box::new(value))),
                    Expr::IndexGet(expr, index) => Ok(Expr::IndexSet(expr, index, Box::new(value))),
                    Expr::Variable(variable) => Ok(Expr::Assign(variable, sign, Box::new(value))),
                    _ => Err(ParseError::Line(self.peek().line, "Cannot use assignment.".to_string())),
                };
            },
            _ => {
                return Ok(expr);
            }
        }
    }

    fn lambda(&mut self) -> Result<Expr> {
        if self.consume(TokenType::LAMBDA).is_ok() {
            let parameters = self.parameters()?;
            self.consume(TokenType::MINUS_GREATER)?;

            // Support both block statement (with braces) and
            // single-line statement (without braces)
            let stmt;
            if self.check(TokenType::LEFT_BRACE) {
                stmt = self.block()?;
            } else {
                let temp = self.statement()?;
                stmt = Stmt::Block(vec![temp]);
            }

            let res = Expr::Lambda(parameters, Box::new(stmt));
            // println!("{:?}", &res);
            Ok(res)
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
        let mut left = self.range()?;

        while let Some(op) = self.match_token(&[
            TokenType::LESS,
            TokenType::GREATER,
            TokenType::LESS_EQUAL,
            TokenType::GREATER_EQUAL,
        ]) {
            let right = self.range()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn range(&mut self) -> Result<Expr> {
        let left = self.addition()?;

        if let Some(token) = self.match_token(&[TokenType::DOT_DOT, TokenType::DOT_DOT_DOT]) {
            let right = self.addition()?;
            Ok(Expr::Binary(Box::new(left), token, Box::new(right)))
        } else {
            Ok(left)
        }
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

        while let Some(op) = self.match_token(&[TokenType::STAR,
                                                                   TokenType::SLASH, TokenType::SLASH_LESS, TokenType::SLASH_GREATER,
                                                                   TokenType::PERCENT]) {
            let right = self.unary()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr> {
        let mut unary_ops = vec![];

        while let Some(op) = self.match_token(&[TokenType::BANG, TokenType::MINUS, TokenType::NOT]) {
            unary_ops.push(op);
        }

        let mut expr = self.call()?;

        for op in unary_ops.into_iter().rev() {
            expr = Expr::Unary(op, Box::new(expr));
        }

        Ok(expr)
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            if let Ok(paren) = self.consume(TokenType::LEFT_PAREN) {
                let prev = self.set_ignore_newline(true);
                let args = self.arguments()?;
                self.set_ignore_newline(prev);
                self.consume(TokenType::RIGHT_PAREN)?;
                expr = Expr::Call(Box::new(expr), paren, args);

            } else if self.consume(TokenType::LEFT_BRACKET).is_ok() {
                let prev = self.set_ignore_newline(true);
                let index = self.expression()?;
                self.set_ignore_newline(prev);
                self.consume(TokenType::RIGHT_BRACKET)?;
                expr = Expr::IndexGet(Box::new(expr), Box::new(index));

            } else if self.consume(TokenType::DOT).is_ok() {
                let name = self.consume(TokenType::IDENTIFIER)?;
                expr = Expr::Get(Box::new(expr), name);

            } else if self.check(TokenType::NEWLINE) && self.peek_next_non_newline().token_type == TokenType::DOT {
                // Allows leading dot chain method call
                self.skip_newlines();
                self.consume(TokenType::DOT)?;
                let name = self.consume(TokenType::IDENTIFIER)?;
                expr = Expr::Get(Box::new(expr), name);

            } else {
                break;
            }
        }

        Ok(expr)
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

        } else if let Ok(token) = self.consume(TokenType::SELF) {
            Ok(Expr::SelfExpr(token))

        } else if let Ok(token) = self.consume(TokenType::SUPER) {
            self.consume(TokenType::DOT)?;
            let function = self.consume(TokenType::IDENTIFIER)?;
            Ok(Expr::SuperExpr(token, function))

        } else if self.consume(TokenType::LEFT_PAREN).is_ok() {
            // Ignore newlines when directly within a group
            let prev = self.set_ignore_newline(true);

            if self.consume(TokenType::RIGHT_PAREN).is_ok() {
                // Empty tuple
                return Ok(Expr::Tuple(vec![]));
            }

            let expr = self.expression()?;

            if self.consume(TokenType::COMMA).is_ok() {
                // Parse tuple
                let mut exprs = self.arguments()?;
                exprs.insert(0, expr);
                self.set_ignore_newline(prev);
                self.consume(TokenType::RIGHT_PAREN)?;

                Ok(Expr::Tuple(exprs))

            } else {
                // Grouped expression
                self.set_ignore_newline(prev);
                self.consume(TokenType::RIGHT_PAREN)?;

                Ok(expr)
            }

        } else if self.consume(TokenType::LEFT_BRACKET).is_ok() {
            // Parse array
            let prev = self.set_ignore_newline(true);
            let exprs = self.arguments()?;
            self.set_ignore_newline(prev);
            self.consume(TokenType::RIGHT_BRACKET)?;
            Ok(Expr::Array(exprs))

        } else if self.consume(TokenType::LEFT_BRACE).is_ok() {
            // Parse dictionary
            let prev = self.set_ignore_newline(true);
            let exprs = self.key_value_pairs()?;
            self.set_ignore_newline(prev);
            self.consume(TokenType::RIGHT_BRACE)?;
            Ok(Expr::Dictionary(exprs))

        } else {
            Err(ParseError::Token(self.peek().clone(), "Unexpected token.".to_string()))
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

    fn key_value_pairs(&mut self) -> Result<Vec<(Expr, Expr)>> {
        let mut pairs = vec![];

        loop {
            if let Ok(key) = self.expression() {
                self.consume(TokenType::COLON)?;
                let value = self.expression()?;
                pairs.push((key, value));

                if self.consume(TokenType::COMMA).is_ok() {
                    continue;
                }
            }
            break;
        }

        Ok(pairs)
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

    fn peek_next_non_newline(&self) -> &Token {
        let mut index = self.current + 1;
        while self.tokens[index].token_type == TokenType::NEWLINE && index < self.tokens.len() {
            index += 1;
        }

        &self.tokens[index]
    }

    fn advance(&mut self) -> Token {
        let token = self.peek().clone();

        if !self.is_at_end() {
            self.current += 1;

            if self.ignore_newline {
                self.skip_newlines();
            }
        }

        match token.token_type {
            TokenType::LEFT_PAREN | TokenType::LEFT_BRACKET | TokenType::LEFT_BRACE => self.nested_level += 1,
            TokenType::RIGHT_PAREN | TokenType::RIGHT_BRACKET | TokenType::RIGHT_BRACE => self.nested_level -= 1,
            _ => (),
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
            Err(ParseError::Token(self.peek().clone(), format!("Unexpected token, expected type {:?}.", token_type)))
        }
    }

    /// Consume at least one newline and skip the rest.
    /// Does not need to skip if at the end of file.
    fn consume_newline(&mut self) -> Result<()> {
        if !self.is_at_end() {
            self.consume(TokenType::NEWLINE)?;
            self.skip_newlines();
        }
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
