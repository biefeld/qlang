use crate::engine::{
    qubit_register::QubitRegister,
    register_error::RegisterError

};
use num_complex::Complex64;
use rand::RngExt;


impl QubitRegister {
/// Given a subset of indexes in the interval `0..n`,
/// Apply a partial measure on the qubits found at those
/// indices. The remaining wave function will be kept intact,
/// And the returned vector will contain the bitstring representing the
/// collapsed/ measured qubits.
/// 
/// Pre:
/// 
/// - `{∀k ∈ tgts | 0 <= k < n}`
/// 
/// - every element of `tgts` is unique.
pub fn partial_measure(&mut self, tgts: &Vec<usize>) -> Result<Vec<usize>, RegisterError> {
    // --- Ensure all elements in `tgts < n` ---
    let filter: Vec<&usize> = tgts.into_iter()
    .filter(|x| **x >= self.get_n()).collect();
    if filter.len() > 0 {
        let fst: usize = **filter.get(0).unwrap();
        return Err(RegisterError::OutOfRange(fst, self.get_n()));
    }

    // --- Build measurement mask ---
    let mut mask: usize = 0;
    for &q in tgts {
        mask |= 1 << ((self.get_n() - 1) - q);
    }

    // --- Bucket probabilities ---
    let mut probs: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();

    for (i, amp) in self.state.iter().enumerate() {
        let key = i & mask;
        *probs.entry(key).or_insert(0.0) += amp.norm_sqr();
    }

    // --- Sample outcome ---
    let r: f64 = rand::rng().random();

    let mut cumulative = 0.0;
    let mut chosen_key = 0;

    for (key, p) in probs.iter() {
        cumulative += p;
        if r <= cumulative {
            chosen_key = *key;
            break;
        }
    }

    // --- Collapse state ---
    let mut norm = 0.0;

    for i in 0..self.state.len() {
        if (i & mask) == chosen_key {
            norm += self.state[i].norm_sqr();
        } else {
            self.state[i] = Complex64::new(0.0, 0.0);
        }
    }

    let norm_factor = norm.sqrt();

    for amp in self.state.iter_mut() {
        if *amp != Complex64::new(0.0, 0.0) {
            *amp /= norm_factor;
        }
    }

    // --- Extract measured bits in tgts order ---
    let mut result = Vec::new();

    for &q in tgts {
        let bit = (chosen_key >> ((self.get_n() - 1) - q)) & 1;
        result.push(bit);
    }

    Ok(result)
}
}