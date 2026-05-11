pub mod ast_types;
pub mod parser_error;

mod parse_exprs;
mod parse_functions;
mod parse_oracle;
mod parse_circuits;

use crate::interpreter::parser::{
    parser_error::ParserError,
    ast_types as ast
};
use crate::interpreter::lexer::token::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize, // index into `tokens`
}


/// CONSTRUCTORS
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        // Note that this will take ownership of `tokens`
        Self { tokens, current: 0 }
    }
}


/// HELPERS
impl Parser {
    fn is_at_end(&self) -> bool { self.current >= self.tokens.len() }

    /// Given some usize `lookahead`, return
    ///  `self.tokens[self.current + lookahead]`
    /// (provided it exists).
    fn peek(&self, lookahead: usize) -> Option<&Token> {
        let idx: usize = self.current + lookahead;
        return self.tokens.get(idx);
    }

    /// Pre: `self.current > 0`
    fn previous(&self) -> &Token { &self.tokens[self.current - 1] }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() { self.current += 1; }
        return self.previous();
    }

    /// Given the Token `t` found at `self.tokens[self.current]`,
    /// determine if `t.token_type == other`
    fn token_matches(&self, other: TokenType) -> bool {
        self.token_matches_lookahead(other, 0)
    }

    /// Given the Token `t` found at `self.tokens[self.current + lookahead]`,
    /// determine if `t.token_type == other`
    fn token_matches_lookahead(&self, other: TokenType, lookahead: usize) -> bool {
        match self.peek(lookahead) {
            Some(token) => token.token_type == other,
            None => false
        }
    }

    /// Pre: `!self.is_at_end()`
    /// 
    /// returns a cloned string of the current lexeme.
    fn curr_lexeme(&self) -> String {
        match self.is_at_end() {
            true  => String::from("UNEXPECTED EOF"),
            false => self.tokens[self.current].lexeme.clone()
        }
    }

    /// Checks to see if the current token is of type `expected`. If it is,
    /// the tokens lexeme is returned, and the token is consumed; If not,
    /// an error is thrown.
    fn expect(&mut self, expected: TokenType) -> Result<String, ParserError> {
        if !self.is_at_end() && self.token_matches(expected) {
            let lexeme: String = self.curr_lexeme();
            self.advance();
            return Ok(lexeme);
        }
        return Err(ParserError::UnexpectedToken(self.curr_lexeme()));
    }

    /// Attempt to convert a lexeme into a numeric literal. This function
    /// should be used primarily when parsing bit and qubit types.
    fn convert_to_usize(val: String) -> Result<usize, ParserError> {
        return match val.parse::<usize>() {
            Ok(n) => Ok(n),
            Err(_) => Err(ParserError::FailedNumConversion(val))
        };
    }
}


/// PARSING METHODS
impl Parser {
    fn parse_statement(&mut self) -> Result<ast::Statement, ParserError> {
        // Since this function is only ever called at the beginning of
        // each iteration, we should never enter this block.
        if self.is_at_end() { return Err(ParserError::UnexpectedEnd); }
        let curr_type: TokenType = self.peek(0).unwrap().token_type;
        match curr_type {
            TokenType::Bits => {
                let result = self.parse_assignment()?;
                return Ok(ast::Statement::Assignment(result));
            },
            TokenType::Function => {
                let result: ast::FunctionDecl = self.parse_function()?;
                return Ok(ast::Statement::Function(result));
            },
            TokenType::Oracle => {
                let result: ast::OracleDecl = self.parse_oracle()?;
                return Ok(ast::Statement::Oracle(result));
            },
            TokenType::Circuit => {
                let result: ast::CircuitDecl = self.parse_circuit()?;
                return Ok(ast::Statement::Circuit(result));
            },
            TokenType::Identifier => {
                let is_mtd = self.token_matches_lookahead(TokenType::Period, 1);
                let stmt: ast::Statement = match is_mtd {
                    true => ast::Statement::MethodCall(self.parse_method_call()?),
                    false => ast::Statement::Expr(self.parse_expr()?)
                };
                return Ok(stmt);
            },
            _ => {
                let expr: ast::Expr = self.parse_expr()?;
                return Ok(ast::Statement::Expr(expr));
            }
        }
    }

    fn parse_assignment(&mut self) -> Result<ast::Assignment, ParserError> {
        self.expect(TokenType::Bits)?;
        let name: String =self.expect(TokenType::Identifier)?;
        self.expect(TokenType::Equals)?;
        let value: ast::Expr = self.parse_expr()?;

        let assignment: ast::Assignment = ast::Assignment { name, value };
        self.expect(TokenType::Semicolon)?;
        return Ok(assignment);
    }

    fn parse_method_call(&mut self) -> Result<ast::MethodCall, ParserError> {
        let circuit_name: String = self.expect(TokenType::Identifier)?;
        self.expect(TokenType::Period)?; 
        let method_name: String = self.expect(TokenType::Identifier)?;
        self.expect(TokenType::LParen)?;

        let method_args: Vec<ast::MethodArg> = self.parse_method_args()?;

        self.expect(TokenType::RParen)?;
        self.expect(TokenType::Semicolon)?;
        let call: ast::MethodCall = ast::MethodCall {
            name: circuit_name,
            call: method_name,
            args: method_args
        };
        return Ok(call);
    }

    fn parse_method_args(&mut self) -> Result<Vec<ast::MethodArg>, ParserError> {
        let mut args: Vec<ast::MethodArg> = Vec::new();
        while self.token_matches(TokenType::Identifier) {
            let name: String = self.expect(TokenType::Identifier)?;
            self.expect(TokenType::Equals)?;
            let qty: String = self.expect(TokenType::IntLiteral)?;
            let qty: usize = Parser::convert_to_usize(qty)?;

            let arg: ast::MethodArg = ast::MethodArg { name, value: qty };
            args.push(arg);

            if self.token_matches(TokenType::RParen) { break; }
            self.expect(TokenType::Comma)?;
            if !self.token_matches(TokenType::Identifier) {
                return Err(ParserError::UnexpectedToken(self.curr_lexeme()));
            }
        }
        return Ok(args);
    }
}


/// API-CALLABLE METHODS
impl Parser {
    pub fn parse(&mut self) -> Result<ast::Program, ParserError> {
        let mut program: ast::Program = Vec::new();
        while !self.is_at_end() {
            // ensure Non-EOF is consumed
            let curr_type: &TokenType = &self.tokens[self.current].token_type;
            if *curr_type == TokenType::EOF { break; }

            let result = self.parse_statement()?;
            program.push(result);
        }
        Ok(program)
    }
}