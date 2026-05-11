use crate::interpreter::parser::{Parser, ParserError};
use crate::interpreter::parser::{ast_types as ast};
use crate::interpreter::lexer::token::TokenType;


/* Due to the relative complexity of parsing expressions,
   Parsing expressions has been localised within this file */

impl Parser {
    pub fn parse_expr(&mut self) -> Result<ast::Expr, ParserError> { self.parse_dot_expr() }

    /* BINARY EXPRESSIONS */
    fn parse_dot_expr(&mut self) -> Result<ast::Expr, ParserError> {
        let mut left: ast::Expr = self.parse_or_expr()?;
        while self.token_matches(TokenType::Star) {
            self.advance();
            let right: ast::Expr = self.parse_or_expr()?;
            left = ast::Expr::Binary {
                left: Box::new(left),
                op: ast::BinOp::DotProduct,
                right: Box::new(right)
            };
        }
        return Ok(left);
    }

    fn parse_or_expr(&mut self) -> Result<ast::Expr, ParserError> {
        let mut left: ast::Expr = self.parse_xor_expr()?;
        while self.token_matches(TokenType::Pipe) {
            self.advance();
            let right: ast::Expr = self.parse_xor_expr()?;
            left = ast::Expr::Binary {
                left: Box::new(left),
                op: ast::BinOp::Or,
                right: Box::new(right)
            };
        }
        return Ok(left);
    }

    fn parse_xor_expr(&mut self) -> Result<ast::Expr, ParserError> {
        let mut left: ast::Expr = self.parse_and_expr()?;   
        while self.token_matches(TokenType::Caret) {
            self.advance();
            let right: ast::Expr = self.parse_and_expr()?;
            left = ast::Expr::Binary {
                left: Box::new(left),
                op: ast::BinOp::Xor,
                right: Box::new(right)
            };
        }
        return Ok(left);
    }

    fn parse_and_expr(&mut self) -> Result<ast::Expr, ParserError> {
        let mut left: ast::Expr = self.parse_postfix()?;
        while self.token_matches(TokenType::Ampersand) {
            self.advance();
            let right: ast::Expr = self.parse_postfix()?;
            left = ast::Expr::Binary {
                left: Box::new(left),
                op: ast::BinOp::And,
                right: Box::new(right)
            };
        }
        return Ok(left);
    }
    /* END BINARY EXPRESSIONS */


    /* POSTFIX EXPRESSIONS */
    fn parse_postfix(&mut self) -> Result<ast::Expr, ParserError> {
        let mut primary: ast::Expr = self.parse_primary()?;
        while self.token_matches(TokenType::LParen) {
            self.advance();
            let postfix: Vec<ast::Expr> = self.parse_call_args()?;
            self.expect(TokenType::RParen)?;
            primary = ast::Expr::Call { callee: Box::new(primary), args: postfix };
        }
        Ok(primary)
    }

    fn parse_call_args(&mut self) -> Result<Vec<ast::Expr>, ParserError> {
        let mut contents: Vec<ast::Expr> = vec![];
        // empty args
        if self.token_matches(TokenType::RParen) { return Ok(contents); }
        let arg: ast::Expr = self.parse_expr()?;
        contents.push(arg);
        while self.token_matches(TokenType::Comma) {
            self.advance();
            let new_arg: ast::Expr = self.parse_expr()?;
            contents.push(new_arg);
        }
        return Ok(contents);
    }

    fn parse_primary(&mut self) -> Result<ast::Expr, ParserError> {
        let curr_lexeme: String = self.curr_lexeme();
        match self.tokens[self.current].token_type {
            TokenType::Identifier => {
                let curr: String = self.curr_lexeme();
                self.advance();
                Ok(ast::Expr::Identifier(curr))
            },
            TokenType::BitsLiteral => {
                let curr: String = self.curr_lexeme();
                self.advance();
                Ok(ast::Expr::BitsLiteral(curr))
            },
            TokenType::LParen => {
                self.advance();
                return self.parse_grouping();
            },
            _ => Err(ParserError::UnexpectedToken(curr_lexeme))
        }
    }

    fn parse_grouping(&mut self) -> Result<ast::Expr, ParserError> {
        let expr: ast::Expr = self.parse_expr()?;
        self.expect(TokenType::RParen)?;
        return Ok(ast::Expr::Grouping(Box::new(expr)));
    }
    /* END POSTFIX EXPRESSIONS */
}