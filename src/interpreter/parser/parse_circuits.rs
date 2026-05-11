use crate::interpreter::parser::{Parser, ParserError};
use crate::interpreter::parser::{ast_types as ast};
use crate::interpreter::lexer::token::TokenType;


impl Parser {
    pub fn parse_circuit(&mut self) -> Result<ast::CircuitDecl, ParserError> {
        self.expect(TokenType::Circuit)?;
        
        let circuit_name: String = self.expect(TokenType::Identifier)?;
        self.expect(TokenType::LBrace)?;

        let registers: Vec<ast::QubitDecl> = self.parse_circuit_registers()?;
        let instructions: Vec<ast::CircuitInstr> = self.parse_circuit_instructions()?;

        self.expect(TokenType::RBrace)?;
        let decl: ast::CircuitDecl = ast::CircuitDecl {
            name: circuit_name,
            registers: registers,
            instructions: instructions
        };
        return Ok(decl);
    }


    fn parse_circuit_registers(&mut self) -> Result<Vec<ast::QubitDecl>, ParserError> {
        self.expect(TokenType::Register)?;
        self.expect(TokenType::Colon)?;
        let mut registers: Vec<ast::QubitDecl> = Vec::new();
        while self.token_matches(TokenType::Qubits) {
            let curr: ast::QubitDecl = self.parse_qubit_decl()?;
            registers.push(curr);
        }

        match registers.len() > 0 {
            true => Ok(registers),
            false => Err(ParserError::NoRegistersParsed)
        }
    }


    fn parse_qubit_decl(&mut self) -> Result<ast::QubitDecl, ParserError> {
        self.expect(TokenType::Qubits)?;
        let var_name: String = self.expect(TokenType::Identifier)?;
        self.expect(TokenType::Equals)?;
        let value: String = self.expect(TokenType::StringLiteral)?;
        self.expect(TokenType::Semicolon)?;

        // strip `value` of quotations
        let value = &value[1..value.len() - 1];
        return Ok(ast::QubitDecl { name: var_name, init: value.to_string() });
    }


    fn parse_circuit_instructions(&mut self) -> Result<Vec<ast::CircuitInstr>, ParserError> {
        self.expect(TokenType::Apply)?;
        self.expect(TokenType::Colon)?;

        let mut instructions: Vec<ast::CircuitInstr> = Vec::new();
        while self.token_matches(TokenType::Identifier) {
            let curr: ast::CircuitInstr = self.parse_circuit_instruction()?;
            instructions.push(curr);
        }
        // unlike registers, its ok to have an "empty"/ identity circuit.
        return Ok(instructions);
    }

    
    fn parse_circuit_instruction(&mut self) -> Result<ast::CircuitInstr, ParserError> {
        let gate_name: String = self.expect(TokenType::Identifier)?;
        self.expect(TokenType::LParen)?;

        let mut args: Vec<ast::CircuitRef> = Vec::new();

        // parse the registers that the gate above applies to
        while self.token_matches(TokenType::Identifier) {
            let curr = self.parse_circuit_arg()?;
            args.push(curr);

            if self.token_matches(TokenType::RParen) { break; }
            self.expect(TokenType::Comma)?;
            // If we match on a comma, we need to ensure that an identifier follows
            if !self.token_matches(TokenType::Identifier) {
                return Err(ParserError::UnexpectedToken(self.curr_lexeme()));
            }
        }
        self.expect(TokenType::RParen)?;
        self.expect(TokenType::Semicolon)?;
        
        if args.len() == 0 { return Err(ParserError::NoRegistersParsed); }
        return Ok(ast::CircuitInstr { name: gate_name, args: args });
    }


    fn parse_circuit_arg(&mut self) -> Result<ast::CircuitRef, ParserError> {
        let curr_identifier: String = self.expect(TokenType::Identifier)?;
        // syntactic - sugar `x` is functionally equivalent to `x[_]`
        if self.token_matches(TokenType::Comma) || self.token_matches(TokenType::RParen) {
            return Ok(ast::CircuitRef {name: curr_identifier, applies: ast::Applies::All} );
        }
        self.expect(TokenType::LSqBracket)?;

        let curr_type = match self.peek(0) {
            Some(t) => &t.token_type,
            None => return Err(ParserError::UnexpectedEnd),
        };
        let curr_applies: ast::Applies = match curr_type {
            TokenType::Underscore => ast::Applies::All,
            TokenType::IntLiteral => {
                let val: usize = Parser::convert_to_usize(self.curr_lexeme())?;
                ast::Applies::One(val)
            },
            _ => { return Err(ParserError::UnexpectedToken(self.curr_lexeme())); }
        };
        self.advance();

        self.expect(TokenType::RSqBracket)?;
        let curr_ref: ast::CircuitRef = ast::CircuitRef {
            name: curr_identifier,
            applies: curr_applies
        };
        return Ok(curr_ref);
    }
}