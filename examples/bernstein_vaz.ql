bits secret = 0b0110;
function f(x: bits[4]) -> bits[1] {
    x * secret
}
oracle U_f(x: qubits[4], y: qubits[1]) loads f;

circuit bernstein_vazirani {
    register:
        qubits x = "++++";
        qubits y = "-";

    apply:
        U_f(x, y);
        H(x);
        measure(x);
}

bernstein_vazirani.measure(shots=3);