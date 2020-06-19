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
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            current: 0,
            tokens,
        }
    }
}

// Statements
impl Parser {
    pub fn statement(&mut self) -> Result<Stmt> {
        panic!()
    }

    fn block(&mut self) -> Result<Stmt> {
        // TODO
        self.consume(TokenType::LEFT_BRACE)?;
        self.consume(TokenType::RIGHT_BRACE)?;
        Ok(Stmt::Block(vec![]))
    }
}

// Expressions
impl Parser {
    pub fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        // TODO: do call later
        self.if_expr()
    }

    fn if_expr(&mut self) -> Result<Expr> {
        if self.consume(TokenType::IF).is_ok() {
            let condition = self.logic_or()?;

            // Then branch must be a block
            let then_stmt = self.block()?;

            // Optional else branch
            let else_stmt = match self.consume(TokenType::ELSE) {
                Ok(()) => self.block()?,
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

        } else if let Some(token) = self.match_token(&[TokenType::IDENTIFIER]) {
            Ok(Expr::Variable(token))

        } else if self.consume(TokenType::LEFT_PAREN).is_ok() {
            let expr = self.expression()?;
            self.consume(TokenType::RIGHT_PAREN)?;
            Ok(expr)

        } else {
            // TODO: add exprs like super, self, etc.
            Err(ParseError {
                message: format!("unexpected token")
            })
        }
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

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn peek_next(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
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

    /// Consume the currect token by advancing if the token type matches. Otherwise returns an error.
    fn consume(&mut self, token_type: TokenType) -> Result<()> {
        if self.check(token_type) {
            self.advance();
            Ok(())
        } else {
            let token = self.peek();

            Err(ParseError {
                message: format!("expected type {:?} for token {}, at line {}", token_type, token.lexeme, token.line),
            })
        }
    }
}
