use crate::engine::{black_box::BlackBox, register_error::RegisterError};
use std::f64::consts::SQRT_2;
use num_complex::Complex64;

/// Applies the Pauli X operator to a quantum state. `n` denotes the
/// Number of qubits in `state`, and `tgt` denotes the given qubit to be
/// operated on (such that `0` is the left-most qubit, and `n-1` is the right-most).
/// 
/// Pre:
/// 
/// - `0 <= tgt < n`.
/// 
/// - `state.len() == 2**n`.
pub fn apply_pauli_x(state: &mut Vec<Complex64>, n: usize, tgt: &Vec<usize>) -> bool {
    let tgt: Option<&usize> = tgt.get(0);
    if tgt.is_none() { return false; }
    let tgt: usize = *tgt.unwrap();
    if tgt >= n { return false ;}

    let mask: usize = 1 << ((n-1) - tgt);
    for i in 0..state.len() {
        let new: usize = i ^ mask;
        if new > i { state.swap(i, new); }
    }
    return true;
}

/// Applies the Pauli Y operator to a quantum state. `n` denotes the
/// Number of qubits in `state`, and `tgt` denotes the given qubit to be
/// operated on (such that `0` is the left-most qubit, and `n-1` is the right-most).
/// 
/// Pre:
/// 
/// - `0 <= tgt < n`.
/// 
/// - `state.len() == 2**n`.
pub fn apply_pauli_y(state: &mut Vec<Complex64>, n: usize, tgt: &Vec<usize>) -> bool {
    let tgt: Option<&usize> = tgt.get(0);
    if tgt.is_none() { return false; }
    let tgt: usize = *tgt.unwrap();
    if tgt >= n { return false ;}
        
    let mask: usize = 1 << ((n-1) - tgt);
    for i in 0..state.len() {
        let new: usize = i ^ mask;
        if new > i {
            let a = state[i];
            let b = state[new];
            if (i & mask) == 0 {
                // i corresponds to |0>, new to |1>
                state[i] = Complex64::new(0.0, -1.0) * b;
                state[new] = Complex64::new(0.0, 1.0) * a;
            } else {
                // i corresponds to |1>, new to |0>
                state[i] = Complex64::new(0.0, 1.0) * b;
                state[new] = Complex64::new(0.0, -1.0) * a;
            }
        }
    }
    return true;
}

/// Applies the Pauli Z operator to a quantum state. `n` denotes the
/// Number of qubits in `state`, and `tgt` denotes the given qubit to be
/// operated on (such that `0` is the left-most qubit, and `n-1` is the right-most).
/// 
/// Pre:
/// 
/// - `0 <= tgt < n`.
/// 
/// - `state.len() == 2**n`.
pub fn apply_pauli_z(state: &mut Vec<Complex64>, n: usize, tgt: &Vec<usize>) -> bool {
    let tgt: Option<&usize> = tgt.get(0);
    if tgt.is_none() { return false; }
    let tgt: usize = *tgt.unwrap();
    if tgt >= n { return false ;}

    let mask: usize = 1 << ((n-1) - tgt);

    for i in 0..state.len() {
        // essentially a no-op on zeroes
        if (i & mask) != 0 { state[i] = -state[i]; }
    }
    return true;
}

/// Applies the Hadamard operator to a quantum state. `n` denotes the
/// Number of qubits in `state`, and `tgt` denotes the given qubit to be
/// operated on (such that `0` is the left-most qubit, and `n-1` is the right-most).
/// 
/// Pre:
/// 
/// - `0 <= tgt < n`.
/// 
/// - `state.len() == 2**n`.
pub fn apply_hadamard(state: &mut Vec<Complex64>, n: usize, tgt: &Vec<usize>) -> bool {
    let tgt: Option<&usize> = tgt.get(0);
    if tgt.is_none() { return false; }
    let tgt: usize = *tgt.unwrap();
    if tgt >= n { return false ;}

    let mask: usize = 1 << ((n-1) - tgt);
    let inv_sqrt2: Complex64 = 1.0 / Complex64::from(SQRT_2);

    for i in 0..state.len() {
        let new: usize = i ^ mask;
        if new > i {
            let a = state[i];
            let b = state[new];


            state[i] = (a + b) * inv_sqrt2;
            state[new] = (a - b) * inv_sqrt2;
        }
    }
    return true;
}

/// helper for apply_s and apply_t.
/// when we extend the frontend to support R gates, then
/// we can make this public.
fn apply_r_theta(state: &mut Vec<Complex64>, n: usize, tgt: &Vec<usize>, theta: f64) -> bool {
    let tgt: Option<&usize> = tgt.get(0);
    if tgt.is_none() { return false; }
    let tgt: usize = *tgt.unwrap();
    if tgt >= n { return false ;}

    let mask: usize = 1 << ((n - 1) - tgt);
    let phase = Complex64::from_polar(1.0, theta); // e^{iθ}

    for i in 0..state.len() {
        // no-op on |0>
        if (i & mask) != 0 { state[i] *= phase; }
    }
    return true;
}

pub fn apply_s(state: &mut Vec<Complex64>, n: usize, tgt: &Vec<usize>) -> bool {
    apply_r_theta(state, n, tgt, std::f64::consts::FRAC_PI_2)
}

pub fn apply_t(state: &mut Vec<Complex64>, n: usize, tgt: &Vec<usize>) -> bool {
    apply_r_theta(state, n, tgt, std::f64::consts::FRAC_PI_4)
}


/// Applies the CNOT operator to a quantum state. `n` denotes the
/// Number of qubits in `state`, and `tgt` denotes the given qubits to be read and
/// operated on (such that `0` is the left-most qubit, and `n-1` is the right-most).
/// 
/// Pre:
/// 
/// - `tgts[0]` is the `control` qubit, and `tgts[1]` is the `tgt` qubit.
/// 
/// - `0 <= tgt < n`.
/// 
/// - `0 <= control < n`.
/// 
/// - `state.len() == 2**n`.
pub fn apply_cnot(state: &mut Vec<Complex64>, n: usize, tgts: &Vec<usize>) -> bool {
    let (control, tgt) = (tgts.get(0), tgts.get(1));
    if control.is_none() || tgt.is_none() { return false; }
    let (tgt, control) = (*tgt.unwrap(), *control.unwrap());
    if (tgt >= n) || (control >= n) { return false; }
    if tgt == control { return false; }

    let tgt_mask: usize = 1 << ((n-1) - tgt);
    let control_mask: usize = 1 << ((n-1) - control);

    for i in 0..state.len() {
        if (i & control_mask) == 0 { continue; }
        let new: usize = i ^ tgt_mask;
        if new > i { state.swap(i, new); }
    }
    return true;
}

/// Given some black-box `f`, apply the unitary `U_f` to the entire `state`.
/// Specifically, if we say that `x` denotes the first `n-1` qubits in state, and `y`
/// denotes the last qubit in state, then this function will apply `|x,y> -> |x, y ^ f(x)>`
/// to all elements in `state`.
/// 
/// Pre:
/// 
/// - `f` must have the type `{0,1}^n -> {0,1}^m`.
pub fn apply_u_f(state: &mut Vec<Complex64>, total_qubits: usize, f: &BlackBox) -> Result<bool, RegisterError> {
    // TODO - This gate really should be more flexible, much in the same way that CNOT is
    let n = f.input_size();
    let m = f.output_size();

    if total_qubits != n + m { return Err(RegisterError::BlackBoxMisalign); }
    let y_mask: usize = (1 << m) - 1;

    for i in 0..state.len() {
        // predicated on the fact that y forms the right-most
        // bits of the bitstring.
        let x = i >> m;
        let y = i & y_mask;

        // safety: ensure f(x) fits in m bits
        let fx = f.eval(x)?;
        if fx >= (1 << m) { return Err(RegisterError::BlackBoxMisalign); }

        let new_y = y ^ fx;
        let new_index = (x << m) | new_y;
        if i < new_index {
            state.swap(i, new_index);
        }
    }

    return Ok(true);
}