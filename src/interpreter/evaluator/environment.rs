use std::collections::HashMap;
use qlang::engine::{black_box::Lambda, operator::Operator, qubit_register::QubitRegister};

use crate::interpreter::evaluator::runtime_error::RuntimeError;

#[derive(Clone)]
pub enum EvaluatorType {
    Bits(Bits),
    Function(Function),
    Oracle(Oracle),
    Circuit(Circuit)
}
pub type Environment = HashMap<String, EvaluatorType>;

// ----- BITS DEFINITION -----
#[derive(Debug, Clone, Copy)]
pub struct Bits {
    pub literal: usize,
    pub length: usize
}

// ----- FUNCTION DEFINITION -----
pub struct Function {
    /// the cardinality of each `bits` argument.
    /// 
    /// For example, if a function was defined with the
    /// signature `f(x: bits[2], y: bits[4])`, then `input`
    /// will be `vec![2, 4]`.
    /// 
    /// PRE `{∀x ∈ self.input | x > 0}`
    pub input: Vec<usize>,

    /// The number of bits returned by the function.
    /// 
    /// PRE: `output > 0`
    pub output: usize,

    pub func: Lambda
}

impl Clone for Function {
    fn clone(&self) -> Self {
        Function {
            input: self.input.clone(),
            output: self.output,
            func: self.func.clone(),
        }
    }
}

// ----- ORACLE DEFINITION -----
pub struct Oracle {
    /// the cardinality of each `qubits` argument.
    /// 
    /// PRE: `(input.fst > 0) && (input.snd > 0)`
    pub input: (usize, usize),

    /// PRE:
    /// 
    /// - `loads.input.len() == 1`,
    /// 
    /// - `loads.input[0] == self.input.fst`,
    /// 
    /// - `loads.output == self.input.snd`
    pub loads: Function
}
impl Clone for Oracle {
    fn clone(&self) -> Self {
        Oracle {
            input: self.input,
            loads: self.loads.clone(),
        }
    }
}

// ----- CIRCUIT DEFINITION -----
#[derive(Debug, Clone)]
pub struct Circuit {
    pub init: String,
    pub ops: Vec<Operator>
}

impl Circuit {
    /// executes the circuit implemented by `self`. A return value
    /// of `Ok(v)` implies safe execution, and `v` stores the last measurement
    /// captured in the system. If `v` is empty, it means that the circuit did not
    /// end with a measurement.
    /// 
    /// enabling `verbose` will print the final distribution to `stdout`.
    pub fn execute(&self, verbose: bool) -> Result<Vec<usize>, RuntimeError> {
        let mut register: QubitRegister = QubitRegister::new_from_pattern(&self.init)?;
        let mut measurement: Vec<usize> = Vec::new();
        for op in &self.ops {
            measurement = register.apply(op)?;
        }
        if verbose {
            println!("Yielded distribution of {}", register.to_string());
        }
        Ok(measurement)
    }
}