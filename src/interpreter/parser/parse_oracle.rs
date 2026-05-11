use crate::interpreter::parser::{Parser, ParserError};
use crate::interpreter::parser::{ast_types as ast};
use crate::interpreter::lexer::token::TokenType;


// Note that this block uses a few methods from `parse_functions.rs`,
// particularly `parse_function_params`
impl Parser {
    pub fn parse_oracle(&mut self) -> Result<ast::OracleDecl, ParserError> {
        self.expect(TokenType::Oracle)?;
        let gate_name: String = self.expect(TokenType::Identifier)?;
        
        self.expect(TokenType::LParen)?;
        let params: Vec<ast::Param> = self.parse_function_params()?;
        self.expect(TokenType::RParen)?;
        
        self.expect(TokenType::Loads)?;
        let loads: String = self.expect(TokenType::Identifier)?;
        self.expect(TokenType::Semicolon)?;
        
        let oracle: ast::OracleDecl = ast::OracleDecl {
            name: gate_name,
            params,
            loads
        };
        return Ok(oracle);
    }
}