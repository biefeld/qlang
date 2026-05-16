use std::fmt;

pub enum TypeError {
    Expected(&'static str, String),
    FailedBitsParse(String),
    BinaryOpFailed(usize, usize),
    UnresolvedIdentifier(String),
    InvalidArgs,
    FunctionArgWrongType(String),
    FunctionReturnWrongType(String),
    FunctionIncorrectlyTyped(String),
    InvalidGateAppl(String, usize, usize),
    Todo
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expected(t, got) => {
                write!(f, "Expected type '{}', but value resolved to {}", t, got)
            },

            Self::FailedBitsParse(b) => {
                write!(f, "Failed to parse string '{}' into a bitstring", b)
            },

            Self::BinaryOpFailed(a, b) => {
                let comment: String = format!(
                    "LHS was a {}-bitstring, RHS was a {}-bitstring",
                    a, b
                );
                write!(f,"Failed to compose binary operation ({})", comment)
            },

            Self::UnresolvedIdentifier(name) => {
                write!(f, "Variable '{}' could not be resolved.", name)
            },

            Self::InvalidArgs => {
                write!(f, "Arguments which were passed to function did not align with signature")
            },

            Self::FunctionArgWrongType(func_name) => {
                write!(f, "function '{}' was defined with a non-bits parameter.", func_name)
            },

            Self::FunctionReturnWrongType(func_name) => {
                write!(f, "function '{}' was defined with a non-bits return value.", func_name)
            },

            Self::FunctionIncorrectlyTyped(func_name) => {
                write!(f, "Function '{}' was defined to have a different return type than what was analysed", func_name)
            },

            Self::Todo => write!(f, "This error has yet to be properly documented"),

            Self::InvalidGateAppl(name, requires, actual) => {
                write!(f, "Gate '{}' expected {} qubits, but got {}", name, requires, actual)
            }
        }
    }
}