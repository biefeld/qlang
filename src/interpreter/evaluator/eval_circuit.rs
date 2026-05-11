use qlang::engine::black_box::BlackBox;
use qlang::engine::gate::Gate;
use qlang::engine::operator::Operator;

use crate::interpreter::evaluator::Evaluator;
use crate::interpreter::evaluator::environment::{Circuit, EvaluatorType};
use crate::interpreter::evaluator::runtime_error::RuntimeError;
use crate::interpreter::parser::ast_types as ast;

// Although there is a case to use a HashMap over a Vec here,
// Vec's make it easy to preserve the ordering of variables. The
// tradeoff of O(n) searching through the vector is also negligible,
// seeing that the engine permits only 16 unique qubits in a given circuit.
type CircuitLookup = Vec<(String, usize)>; // (identifier, number_of_qubits)

impl Evaluator {
    pub fn eval_circuit_decl(&mut self, decl: &ast::CircuitDecl) -> Result<(), RuntimeError> {
        let mut register_str: String = String::new();
        let mut lookup: CircuitLookup = Vec::new();

        // Builds the entire register to be fed into the engine's QubitRegister,
        // along with the means to index into the register.
        //
        // For example, given the assignments `x := 00, y := ++`,
        // we would want to construct the register "00++" (we assume order);
        // but we also want to know that `x` begins at qubit 0, and `y` begins at 2.
        for reg in decl.registers.iter() {
            register_str += &reg.init;
            lookup.push((reg.name.clone(), reg.init.len()));
        }

        // build engine ops
        let mut ops: Vec<Operator> = Vec::new();
        for instruction in decl.instructions.iter() {
            let mut tgts: Vec<usize> = Vec::new(); 
            for arg in &instruction.args {
                let arg_name = arg.name.clone();
                match arg.applies {
                    ast::Applies::One(n) => {
                        let idx: usize = get_index(&lookup, &arg_name, n)?;
                        tgts.push(idx);
                    }
                    ast::Applies::All => {
                        let idx: usize = get_index(&lookup, &arg_name, 0)?;
                        let length: Option<usize> = lookup.iter()
                            .find(|(a, _)| &arg_name == a)
                            .map(|(_, n)| *n);
                        let length: usize = match length {
                            Some(n) => n,
                            None => { return Err(RuntimeError::Fatal); } // this should be unreachable, in theory
                        };
                        for i in 0..length { tgts.push(idx + i); }
                    }
                }
            }
            let gate: Gate = self.resolve_gate(&instruction.name, tgts.len())?;

            // If Gate arity is 1, then we want to apply the gate point-wise
            // to each target.
            if gate.arity() == 1 {
                let tmp_ops: Option<Vec<Operator>> = tgts.iter()
                    .map(|idx| Operator::new(gate.clone(), vec![*idx]))
                    .collect();
                if tmp_ops.is_none() { return Err(RuntimeError::Fatal); }
                let tmp_ops = tmp_ops.unwrap();
                for op in tmp_ops { ops.push(op); }
            }
            else { // TODO - there's probably a smarter way of representing this if/ else
                let op: Operator = match Operator::new(gate, tgts) {
                    Some(op) => op,
                    _ => { return Err(RuntimeError::OracleConstructionFailed); }
                };
                ops.push(op);
            }
        }

        let cirucit_name: String = decl.name.clone();
        let circuit: Circuit = Circuit { init: register_str, ops };

        self.environment.insert(cirucit_name, EvaluatorType::Circuit(circuit));

        return Ok(());
    }


    fn resolve_gate(&self, gate_name: &String, m_arity: usize) -> Result<Gate, RuntimeError> {
        // check to see that the gate exists as an oracle in the environment
        // - Note that this means that we're allowing users to override existing
        // gates ("H", "CNOT", etc...) with their own oracles.
        match self.environment.get(gate_name) {
            Some(EvaluatorType::Oracle(o)) => {
                let (x, y): (usize, usize) = (o.input.0, o.input.1);
                let func = &o.loads.func;
                let bb: BlackBox = BlackBox::new(func.clone(), x, y);
                return Ok(Gate::BlackBox(bb));
            },
            Some(_) => { return Err(RuntimeError::TypeMismatch); }
            None => { } // check default gate set
        };

        match gate_name.as_str() {
            "H" => Ok(Gate::Hadamard),
            "X" => Ok(Gate::RotateX),
            "Y" => Ok(Gate::RotateY),
            "Z" => Ok(Gate::RotateZ),
            "T" => Ok(Gate::ShiftT),
            "S" => Ok(Gate::ShiftS),
            "CNOT" => Ok(Gate::CNot),
            "SWAP" => Ok(Gate::Swap),
            "CZ" => Ok(Gate::CZ),
            "CCNOT" => Ok(Gate::Toffoli),
            "CSWAP" => Ok(Gate::CSwap),
            // we pass `m_arity` to this function because
            // measure's arity is not pre-determined.
            "measure" => Ok(Gate::Measure(m_arity)),
            _ => Err(RuntimeError::VarNotFound(gate_name.clone()))
        }
    }
}


// ----- HELPERS -----

/// given a `table` of identifiers, an identifier, and a (zero-indexed) 
/// pivot, return the index of `identifer[plus]`  in the circuits 
/// qubits register.
/// 
/// for example, given a `table` [("x", 2), ("y", 3)], and we wanted to find
/// `get_index(table, "y", 1)`, `3` should be returned, since `y[1]` is found at
/// index three. returns `None` if the identifier could not be resolved, or if
/// pivot exceeds the length of the resolved idenitifer.
fn get_index(table: &CircuitLookup, identifer: &String, plus: usize) -> Result<usize, RuntimeError> {
    let mut curr_idx: usize = 0;
    for (name, length) in table.iter() {
        if name != identifer {
            curr_idx += length;
            continue;
        }
        // match made, ensure that plus is valid
        return match plus < *length {
            true  => Ok(curr_idx + plus),
            false => Err(RuntimeError::OutOfBounds(identifer.clone(), plus))
        }
    }
    return Err(RuntimeError::VarNotFound(identifer.clone()));
}