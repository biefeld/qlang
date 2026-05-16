bits a = 0b1101;

function f_balanced(x: bits[4]) -> bits[1] { x * a }
function f_constant(x: bits[4]) -> bits[1] { 0b1 }
oracle U_f(x: qubits[4], y: qubits[1]) loads f_balanced;

circuit deutsch_jozsa {
    register:
        qubits x = "0000";
        qubits y = "1";
    
    apply:
        H(x);
        H(y);
        U_f(x, y);
        H(x);
        measure(x);
}

deutsch_jozsa.measure(shots=3);