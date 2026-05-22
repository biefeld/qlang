
use std::vec;

use crate::engine::black_box::{BlackBox, Lambda};
use crate::engine::gate::{Gate};
use crate::engine::qubit_register::QubitRegister;
use crate::engine::register_error::RegisterError;
use crate::engine::operator_helpers;

// INVARIANT:
// `gate_arity(&self.gate_kind) == self.targets.len()`
#[derive(Debug, Clone)]
pub struct Operator {
    gate_kind: Gate,
    targets: Vec<usize>
}


/* ----- CONSTRUCTORS ----- */
impl Operator {
    /// Creates a new operation to be performed by a circuit. If a circuit
    /// is found to be "invalid", then a no-op is returned.
    ///
    /// Pre:
    /// 
    /// - `gate_arity(&g) == targets.len()`
    /// 
    /// - all of the elements of `targets` must be valid indices of a circuit.
    pub fn new(g: Gate, targets: Vec<usize>) -> Option<Self> {
        if g.arity() != targets.len() { return None; }
        let op: Operator = Operator { gate_kind: g, targets };
        return Some(op);
    }

    /// defines a new gate U_f, based off some specified function 
    /// `f :: {0,1}^n -> {0,1}^m`.
    /// 
    /// Pre: 1 <= m <= n
    pub fn new_u_f(f: Lambda, n: usize, m: usize) -> Option<Self> {
        let bb: BlackBox = BlackBox::new(f,n,m);
        let g: Gate = Gate::BlackBox(bb);
        
        // `targets` is redundant for U_f, since it implicitly targets the entire input space
        let op: Operator = Operator { gate_kind: g, targets: vec![] };
        return Some(op);
    }
}


/* ----- API CALLABLE METHODS ----- */
impl Operator {
    /// Given a register with n-qubits, apply the operator
    /// determined by `&self` to the state, and return the new state.
    /// 
    /// Pre: every value `k ∈ self.targets` is in the intveral `0..(state.n)`
    pub fn apply_to_state(&self, state: &mut QubitRegister) -> Result<Vec<usize>, RegisterError> {
        let n: usize = state.get_n();
        // GUARDS
        for val in &self.targets {
            if *val >= n {
                let err: RegisterError = RegisterError::OutOfRange(*val, n);
                return Err(err);
            }
        }
        // GUARDS

        if let Gate::Measure(_) = &self.gate_kind {
            let result = state.partial_measure(&self.targets)?;
            return Ok(result);
        }

        let result: bool = match &self.gate_kind {
            // unary operations
            Gate::RotateX   => operator_helpers::apply_pauli_x(&mut state.state, n, &self.targets),
            Gate::RotateY   => operator_helpers::apply_pauli_y(&mut state.state, n, &self.targets),
            Gate::RotateZ   => operator_helpers::apply_pauli_z(&mut state.state, n, &self.targets),
            Gate::Hadamard => operator_helpers::apply_hadamard(&mut state.state, n, &self.targets),
            Gate::ShiftS => operator_helpers::apply_s(&mut state.state, n, &self.targets),
            Gate::ShiftT => operator_helpers::apply_t(&mut state.state, n, &self.targets),
            // binary operations
            Gate::CNot => operator_helpers::apply_cnot(&mut state.state, n, &self.targets),
            Gate::Swap => todo!(),
            Gate::CZ => operator_helpers::apply_cz(&mut state.state, n, &self.targets),
            // ternary operators
            Gate::Toffoli => operator_helpers::apply_toffoli(&mut state.state, n, &self.targets),
            Gate::CSwap => todo!(),
            // custom operations
            Gate::BlackBox(bb) => operator_helpers::apply_u_f(&mut state.state, n, bb)?,
            _ => unreachable!() // since we handled `Measure` above
        };

        return match result {
            true => Ok(vec![]),
            false => {
                let msg: String = self.to_string();
                return Err(RegisterError::CompositionFailed(msg));
            }
        };
    }

    pub fn to_string(&self) -> String {
        return format!("{}{:?}", self.gate_kind, self.targets);
    }
}