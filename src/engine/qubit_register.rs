use std::collections::HashSet;
use num_complex::Complex64;
use crate::engine::{operator::Operator, register_error::RegisterError};


/* ----- CONSTANTS ----- */
const MAX_REGISTER_SIZE: usize = 16;


// STRUCT INVARIANTS:
// `state.len() == 2**n`
// `state` is subject to the born rule.
// `n <= MAX_REGISTER_SIZE` (system constraint)
pub struct QubitRegister {
    n: usize, // Number of qubits in a given register
    pub state: Vec<Complex64>
}


/* ----- CONSTRUCTORS ----- */
impl QubitRegister {
    /// Initialises an n-state qubit register
    /// to the `|0...0>` state
    /// 
    /// Pre: `n <= MAX_REGISTER_SIZE`
    pub fn new(n: usize) -> Result<Self, RegisterError> {
        // GUARDS
        if n > MAX_REGISTER_SIZE {
            return Err(RegisterError::TooManyQubits(n, MAX_REGISTER_SIZE));
        }
        if n == 0 { return Err(RegisterError::EmptyRegister); }
        // GUARDS

        let mut state: Vec<Complex64> = Vec::new();
        let range: usize = (2 as usize).pow(n as u32);
        for _ in 0..range {
            state.push(Complex64 {re: 0.0, im: 0.0});
        }

        let qubits: QubitRegister = QubitRegister { n, state };
        return Ok(qubits);
    }

    /// Initialises a qubit register to a specified pattern.
    /// a pattern is a string that only contains letters from the
    /// alphabet `{'0', '1', '+', '-'}`.
    /// 
    /// Pre:
    /// 
    /// - `pattern.len() <= MAX_REGISTER_SIZE`
    /// 
    /// - all characters in `pattern` belong to the aforementioned alphabet. 
    pub fn new_from_pattern(pattern: &String) -> Result<Self, RegisterError> {
        // GUARDS
        if pattern.len() == 0 { return Err(RegisterError::EmptyRegister); }
        if pattern.len() > MAX_REGISTER_SIZE {
            return Err(RegisterError::TooManyQubits(pattern.len(), MAX_REGISTER_SIZE));
        }

        let alphabet: HashSet<char> = HashSet::from(['0', '1', '+', '-']);
        for c in pattern.chars() {
            if !alphabet.contains(&c) {
                return Err(RegisterError::InvalidQubit(c, alphabet))
            }
        }
        // GUARDS

        let n: usize = pattern.len();
        let state: Vec<Complex64> = build_state_from_qubit_string(pattern);
        return Ok(QubitRegister { n, state });
    }
}


/* ----- PROJECTIONS ----- */
impl QubitRegister {
    pub fn get_n(&self) -> usize { return self.n; }
    pub fn get_qubit(&self, i: usize) -> Option<&Complex64> {
        return self.state.get(i);
    }
}


/* ----- API-CALLABLE METHODS ----- */
impl QubitRegister {
    pub fn apply(&mut self, op: &Operator) -> Result<Vec<usize>, RegisterError> {
        return op.apply_to_state(self);
    }

    /// Displays the contents of the state vector
    pub fn to_string(&self) -> String {
        let simplified: Vec<String> = self.state.iter()
            .map(|c| format!("({}+{}i)", c.re, c.im))
            .collect();
        let joined = simplified.join(", ") + &"]";
        return String::from("[") + &joined;
    }
}



/* ----- HELPERS ----- */
fn build_state_from_qubit_string(pattern: &String) -> Vec<Complex64> {
    let n = pattern.len();
    let dim = 1 << n;

    let mut fixed_mask = 0usize;
    let mut fixed_values = 0usize;
    let mut minus_mask = 0usize;
    let mut k = 0usize;

    // Build masks
    for (i, c) in pattern.chars().enumerate() {
        let bit = 1 << (n - 1 - i); // MSB = leftmost

        match c {
            '0' => { fixed_mask |= bit; }
            '+' => { k += 1; }
            '1' => {
                fixed_mask |= bit;
                fixed_values |= bit;
            }
            '-' => {
                k += 1;
                minus_mask |= bit;
            }
            _ => unreachable!(),
        }
    }

    let norm = 1.0 / (2.0f64).powf(k as f64 / 2.0);
    let mut state = vec![Complex64::new(0.0, 0.0); dim];

    for i in 0..dim {
        // Check fixed bits
        if (i & fixed_mask) != fixed_values { continue; }

        // Compute sign
        let parity = (i & minus_mask).count_ones() % 2;
        let sign = if parity == 0 { 1.0 } else { -1.0 };
        state[i] = Complex64::new(sign * norm, 0.0);
    }
    return state;
}