use qlang::engine::register_error::RegisterError;
use std::{fmt, num::ParseIntError};

pub enum RuntimeError {
    TypeMismatch,
    BinaryConversionFailed(String),
    VarNotFound(String),
    IncorrectArgs(usize, usize),
    OracleConstructionFailed,
    BackendError(RegisterError),
    OutOfBounds(String, usize),
    MethodUndefined(String),
    Fatal
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeMismatch => write!(f, "Type mismatched identified during runtime"),

            Self::BinaryConversionFailed(msg) => {
                write!(f, "Failed to cast binary to `usize`: {}", msg)
            },

            Self::VarNotFound(var_name) => write!(f, "variable '{}' was not found in the environment", var_name),

            Self::IncorrectArgs(expected, actual) => {
                write!(f, "expected '{}' args in function, but got '{}'", expected, actual)
            },

            Self::OracleConstructionFailed => write!(f, "ensure that `f :: x -> y` implies oracle has type |x, y>, such that x >= y."),

            Self::BackendError(err) => {
                write!(f, "Engine failed while performing circuit computation: {}", err)
            },
            
            Self::OutOfBounds(identifier, position) => {
                write!(f, "Identifier '{}' in circuit has no index {}", identifier, position)
            },

            Self::MethodUndefined(method) => {
                write!(f, "Method '{}' is not supported for circuits.", method)
            },

            Self::Fatal => write!(f, "an unexpected error occured.")
        }
    }
}

impl From<ParseIntError> for RuntimeError {
    fn from(err: ParseIntError) -> Self { Self::BinaryConversionFailed(err.to_string()) }
}

impl From<RegisterError> for RuntimeError {
    fn from(err: RegisterError) -> Self { Self::BackendError(err) }
}