bits one = 0b1;
function f_not(x: bits[1]) -> bits[1] {
    x ^ one
}
oracle U_f(x: qubits[1], y: qubits[1]) loads f_not;

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