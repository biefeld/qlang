use std::fmt;

pub enum LexerError {
    FileError(String, String),
    UnexpectedChar(char),
    UnexpectedEOF,
    UnclosedString,
    UnclosedComment
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileError(f_n, e) => {
                write!(f, "Cannot read file '{}': {}", f_n, e)
            },
            Self::UnexpectedChar(c) => {
                write!(f, "Unexpected Character '{}' read.", c)
            },
            Self::UnexpectedEOF => {
                write!(f, "Unexpected End-of-File")
            },
            Self::UnclosedString => {
                write!(f, "String not closed before EOF.")
            },
            Self::UnclosedComment => {
                write!(f, "Comment not closed before EOF.")
            }
        }
    }
}