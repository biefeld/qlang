use std::fmt;

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken(String),
    UnexpectedEnd,
    NoRegistersParsed,
    FailedNumConversion(String)
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken(lexeme) => write!(f, "Unexpected Token encountered while parsing: {}.", lexeme),
            Self::UnexpectedEnd => write!(f, "Encountered an unexpected end of Token stream while parsing."),
            Self::NoRegistersParsed => write!(f, "Circuit expected at least one register, but none were provided."),
            Self::FailedNumConversion(n) => write!(f, "Attempted to convert '{}' to a number, but conversion failed.", n)
        }
    }
}