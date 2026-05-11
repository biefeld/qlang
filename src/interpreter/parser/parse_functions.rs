use crate::interpreter::parser::{Parser, ParserError};
use crate::interpreter::parser::{ast_types as ast};
use crate::interpreter::lexer::token::{Token, TokenType};

impl Parser {
    pub fn parse_function(&mut self) -> Result<ast::FunctionDecl, ParserError> {
        self.expect(TokenType::Function)?;
        let name: String = self.expect(TokenType::Identifier)?;
        self.expect(TokenType::LParen)?;
        let params: Vec<ast::Param> = self.parse_function_params()?;
        self.expect(TokenType::RParen)?;

        self.expect(TokenType::Arrow)?;
        let return_type: ast::Type = self.parse_function_type()?;
        
        self.expect(TokenType::LBrace)?;
        let body: ast::Expr = self.parse_expr()?;
        self.expect(TokenType::RBrace)?;

        let decl = ast::FunctionDecl { name, params, return_type, body };
        return Ok(decl);
    }

    pub fn parse_function_params(&mut self) -> Result<Vec<ast::Param>, ParserError> {
        let mut params: Vec<ast::Param> = Vec::new();
        while self.token_matches(TokenType::Identifier) {
            let var_name: String = self.expect(TokenType::Identifier)?;
            self.expect(TokenType::Colon)?;
            
            // In this version of the language, classical functions only take bits as input -
            // In a future release, I may consider accepting tuples of bitstrings.
            let ty: ast::Type = self.parse_function_type()?;
            let curr_param: ast::Param = ast::Param { name: var_name, ty: ty };
            params.push(curr_param);

            if self.token_matches(TokenType::RParen) { break; }
            self.expect(TokenType::Comma)?;
            // If we match on a comma, we need to ensure that an identifier follows
            if !self.token_matches(TokenType::Identifier) {
                return Err(ParserError::UnexpectedToken(self.curr_lexeme()));
            }
        }
        return Ok(params);
    }

    pub fn parse_function_type(&mut self) -> Result<ast::Type, ParserError> {
        let curr_token: Option<&Token> = self.peek(0);
        let curr_type: TokenType = match curr_token.map(|x| x.token_type) {
            Some(TokenType::Bits) => TokenType::Bits,
            Some(TokenType::Qubits) => TokenType::Qubits,
            Some(_) => return Err(ParserError::UnexpectedToken(self.curr_lexeme())),
            None => return Err(ParserError::UnexpectedEnd)
        };
        self.advance();

        self.expect(TokenType::LSqBracket)?;
        let qty: String = self.expect(TokenType::IntLiteral)?;
        let qty: usize = Parser::convert_to_usize(qty)?;
        self.expect(TokenType::RSqBracket)?;

        return match curr_type {
            TokenType::Bits => Ok(ast::Type::Bits(qty)),
            TokenType::Qubits => Ok(ast::Type::Qubits(qty)),
            _ => unreachable!()
        };
    }
}