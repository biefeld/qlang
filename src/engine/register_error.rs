use std::collections::HashSet;
use std::fmt;

#[derive(Debug)]
pub enum RegisterError {
    EmptyRegister,
    TooManyQubits(usize, usize),
    InvalidQubit(char, HashSet<char>),
    OutOfRange(usize, usize),
    CompositionFailed(String),
    BlackBoxMisalign,
    RunTimeFailure(Option<String>)
}

impl fmt::Display for RegisterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRegister => write!(f, "Cannot have an empty register."),

            Self::TooManyQubits(n, max) => {
                write!(f, "Passed '{}' qubits to register, but maximum is {}.", n, max)
            },

            Self::InvalidQubit(c, alphabet) => {
                write!(f, "Character '{}' does not exist in alphabet {{'{:?}'}}.", c, alphabet)
            },

            Self::OutOfRange(val, max) => {
                write!(f, "Tried to operate on qubit '{}', but only {} are in the register.", val, max)
            },

            Self::CompositionFailed(op) => {
                write!(f, "Failed to apply Operation '{}' to register.", op)
            },

            Self::BlackBoxMisalign => {
                write!(f, "Black Box Misaligned with provided inputs")
            }

            Self::RunTimeFailure(s) => {
                match s {
                    Some(s) => { write!(f, "Unexpected failure at runtime: {}", s) },
                    None => {  write!(f, "Unexpected failure at runtime") }
                }
            }
        }
    }
}