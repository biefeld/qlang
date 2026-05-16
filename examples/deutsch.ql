bits one = 0b1;
function f_not(x: bits[1]) -> bits[1] { x ^ one }
function f_zero(x: bits[1]) -> bits[1] { 0b0 }
oracle U_f(x: qubits[1], y: qubits[1]) loads f_zero;

circuit Deutsch {
    register:
        qubits x = "+";
        qubits y = "-";
    
    apply:
        U_f(x, y);
        H(x);
        measure(x);
}

Deutsch.distribution();
Deutsch.measure();