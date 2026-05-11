circuit bell_pair {
    register:
        qubits x = "0";
        qubits y = "0";

    apply:
        H(x);
        CNOT(x, y);
        measure(x,y);
}

bell_pair.measure(shots=10);