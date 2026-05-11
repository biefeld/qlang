pub mod token;
pub mod lexer_error;


use crate::interpreter::lexer::lexer_error::LexerError;
use crate::interpreter::lexer::token::{Token, TokenType};
use std::fs;

pub struct Lexer {
    source: Vec<char>,
    pub tokens: Vec<Token>,
    start_lexeme: usize, // 
    ptr: usize,  // index for `self.source`
    line: usize, // label for creating tokens
    col: usize   // label for creating tokens
}







/// ----- CONSTRUCTORS -----
impl Lexer {
    pub fn new() -> Self {
        Lexer {
            source: vec![],
            tokens: vec![],
            start_lexeme: 0,
            ptr: 0, line: 1, col: 1
        }        
    }

    /// Pre: the file located at `f_n` exists, and is encoded in ASCII
    pub fn load_file(&mut self, f_n: &String) -> Result<(), LexerError> {
        let result = fs::read(f_n);
        if let Err(e) = result {
            let msg: String = e.to_string();
            return Err(LexerError::FileError(f_n.clone(), msg));
        }
        self.source = result.unwrap()
            .iter()
            .map(|&b| b as char)
            .collect();
        return Ok(());
    }
}





/// ----- HELPERS -----
impl Lexer {
    /// checks if EOF has been reached.
    fn is_at_end(&self) -> bool { return self.ptr >= self.source.len(); }

    /// Given a TokenType and lexeme, push a new token to the `source`
    /// vector. This function assumes that token_type and lexeme are sanitised.
    fn add_token(&mut self, token_type: TokenType) {
        let lexeme: String = self.source[self.start_lexeme..self.ptr]
            .iter()
            .collect();
        let token: Token = Token::new(
            token_type,
            lexeme,
            self.line,
            self.col
        );
        self.tokens.push(token);
    }

    /// fetch the current character and increment `self.ptr`.
    /// If `self.is_at_end()` is true, None is returned.
    fn advance(&mut self) -> Option<char> {
        if self.is_at_end() { return None; }
        let curr = self.source[self.ptr];
        self.ptr += 1;

        // update logging vars
        if curr == '\n' {
            self.col = 0;
            self.line += 1;
        }
        else { self.col += 1;}

        return Some(curr);
    }

    /// Look at the character found at index `self.ptr + lookahead`
    fn peek(&self, lookahead: usize) -> Option<char> {
        if (self.ptr + lookahead) >= self.source.len() {
            return None;
        }
        return Some(self.source[self.ptr + lookahead]);
    }
}





/// ----- PRIVATE SCANNING METHODS -----
impl Lexer {
    fn scan_token(&mut self) -> Result<(), LexerError> {
        let c: Option<char> = self.advance();
        if c.is_none() { return Err(LexerError::UnexpectedEOF); }
        let c: char = c.unwrap();

        // handle whitespace
        if vec![' ', '\r', '\t', '\n'].contains(&c) { return Ok(()); }

        // handle one-character tokens
        let single_char = self.scan_single_char(c)?;
        match single_char {
            Some(tt) => {
                self.add_token(tt);
                return Ok(());
            },
            None => { }
        }

        // will always be a keyword/ identifier, so handle it here
        if c.is_alphabetic() { return self.scan_identifier_keyword(); }

        // Assume that (for whatever reason), we're
        // allowed to prefix a number with zero
        if c.is_numeric() {
            if (c == '0') && self.peek(0) == Some('b') {
                self.advance(); // consume 'b'
                return self.scan_bitstring();
            }
            return Ok(self.scan_number());
        }

        // scan for comments in # these blocks #
        if c == '#' { return self.scan_comment(); }

        // lastly, scan for strings
        if c == '"' { return self.scan_string(); }
        return Err(LexerError::UnexpectedChar(c));
    }

    fn scan_comment(&mut self) -> Result<(), LexerError> {
        let mut comment_closed: bool = false;
        while let Some(c) = self.peek(0) {
            if c != '#' { self.advance(); }
            else {
                comment_closed = true;
                break;
            }
        }
        if !comment_closed { return Err(LexerError::UnclosedComment); }

        self.advance(); // close the comment
        return Ok(());
    }

    fn scan_single_char(&mut self, c: char) -> Result<Option<TokenType>, LexerError> {
        match c {
            // symbols
            '=' => Ok(Some(TokenType::Equals)),
            ';' => Ok(Some(TokenType::Semicolon)),
            ':' => Ok(Some(TokenType::Colon)),
            ',' => Ok(Some(TokenType::Comma)),
            '_' => Ok(Some(TokenType::Underscore)),
            '.' => Ok(Some(TokenType::Period)),
            '(' => Ok(Some(TokenType::LParen)),
            ')' => Ok(Some(TokenType::RParen)),
            '{' => Ok(Some(TokenType::LBrace)),
            '}' => Ok(Some(TokenType::RBrace)),
            '[' => Ok(Some(TokenType::LSqBracket)),
            ']' => Ok(Some(TokenType::RSqBracket)),
            // arithmetic symbols
            '^' => Ok(Some(TokenType::Caret)),
            '&' => Ok(Some(TokenType::Ampersand)),
            '|' => Ok(Some(TokenType::Pipe)),
            '*' => Ok(Some(TokenType::Star)),
            // Arrows require unique control flow
            '-' => {
                if let Some('>') = self.peek(0) {
                    self.advance();
                    return Ok(Some(TokenType::Arrow));
                }
                return Err(LexerError::UnexpectedChar(c));
            },
            _   => Ok(None)
        }
    }

    
    fn scan_identifier_keyword(&mut self) -> Result<(), LexerError> {
        while let Some(c) = self.peek(0) {
            // we know that on the first iteration of this loop,
            // the character is strictly alphabetic.
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            }
            // every other character can be considered a terminal.
            else { break; }
        }
        
        // we can't develop `pattern` at the beginning of this function, as the first
        // character has already been consumed (self.advance() at the beginning of `scan_token`)
        let pattern: String = self.source[self.start_lexeme..self.ptr].iter().collect();
        let token_type = match pattern.as_str() {
            "bits" => TokenType::Bits,
            "qubits" => TokenType::Qubits,
            "function" => TokenType::Function,
            "oracle" => TokenType::Oracle,
            "loads" => TokenType::Loads,
            "circuit" => TokenType::Circuit,
            "register" => TokenType::Register,
            "apply" => TokenType::Apply,
            _ => TokenType::Identifier
        };
        self.add_token(token_type);
        Ok(())
    }


    /// Pre: '0b' has already been scanned
    fn scan_bitstring(&mut self) -> Result<(), LexerError> {
        let mut is_valid: bool = false;
        while let Some(c) = self.peek(0) {
            if (c == '0') || (c == '1') {
                is_valid = true; 
                self.advance();
            }
            else { break; }
        }
        if !is_valid { // failed to scan anything beyond `0b`
            let unexpected_char: char = match self.peek(0) {
                Some(c) => c,
                None => '\0'
            };
            return Err(LexerError::UnexpectedChar(unexpected_char));
        }
        self.add_token(TokenType::BitsLiteral);
        return Ok(());
    }

    fn scan_number(&mut self) -> () {
        while let Some(c) = self.peek(0) {
            if c.is_numeric() { self.advance(); }
            else { break; }
        }
        self.add_token(TokenType::IntLiteral);
        return ();
    }


    fn scan_string(&mut self) -> Result<(), LexerError> {
        let mut string_closed: bool = false;
        while let Some(c) = self.peek(0) {
            if c != '"' { self.advance(); }
            else {
                string_closed = true;
                break;
            }
        }
        if !string_closed {
            return Err(LexerError::UnclosedString);
        }

        self.advance(); // close the string
        self.add_token(TokenType::StringLiteral);
        return Ok(());
    }
}





/// ----- API-CALLABLE METHODS -----
impl Lexer {
    pub fn scan_tokens(&mut self) -> Result<(), LexerError> {
        while !self.is_at_end() {
            self.start_lexeme = self.ptr;
            self.scan_token()?;
        }
        
        self.add_token(TokenType::EOF);
        return Ok(());
    }

    /// panics, logging the LexerError `e`, with
    /// details of where the error occured.
    pub fn show_error(&self, e: LexerError) {
        let msg: String = format!("Error on line {}, col {}:", self.line, self.col);
        panic!("{}\nDetails: {}", msg, e);
    }
}