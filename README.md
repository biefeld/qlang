# `qlang` - A DSL for evaluating simple quantum circuits.

`qlang` is an interpreted domain-specific language (DSL) for building and evaluating simple quantum circuits, implemented in Rust.

I started this project because I wanted to consolidate my understanding of the quantum circuit model and canonical quantum algorithms. Although practical quantum computing is still an emerging field, I find the underlying theory and computational model extremely interesting - I built `qlang` to better understand and appreciate this model.

The language currently supports:
- classical bitstring operations such as AND (`&`), OR (`|`), XOR (`^`) and dot-products modulo 2 (`*`).
- Bitstring assignment and classical function definitions.
- Unitary oracles determined by a classical function.
- Circuit construction (with up to 16 qubits), execution, and measurement.

Circuits are also capable of utilising the following gates:
- Unary Operators: Hadamard (`H`), The Pauli Operators (`X`, `Y`, and `Z`), and Phase Operators (`S`, `T`).
- Binary Operators: Controlled Not (`CNOT`).

Although the project is yet to be fully realised, the current implementation is capable of executing:
- [Deutsch's Algorithm](https://quantum.cloud.ibm.com/learning/en/courses/fundamentals-of-quantum-algorithms/quantum-query-algorithms/deutsch-algorithm).
- [The Deutsch-Jozsa Algorithm](https://quantum.cloud.ibm.com/learning/en/courses/fundamentals-of-quantum-algorithms/quantum-query-algorithms/deutsch-jozsa-algorithm).
- [The Bernstein-Vazirani problem](https://quantum.cloud.ibm.com/learning/en/courses/fundamentals-of-quantum-algorithms/quantum-query-algorithms/deutsch-jozsa-algorithm#the-bernstein-vazirani-problem).
- Bell State Preparation.


## Architecture
`qlang` is composed of:
- a lexer for tokenisation,
- a recursive-descent parser for AST construction,
- a static type-checker for ensuring programming correctness,
- an evaluator/runtime for semantic execution,
- and a quantum register engine for state evolution and measurement.


## Example: `qlang` executing the Bernstein-Vazirani algorithm
```txt
# This program executes the Bernstein-Vazirani  #
# algorithm. Given a function that encodes a    #
# secret bitstring `s`, this algorithm recovers #
# `s` in O(1) queries.                          #

bits secret = 0b0110;

# dot-products modulo-2 #
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

# PRINTS TO STDOUT: #
# Measuring circuit 'bernstein_vazirani' 3 time/s. #
# SHOT 1 OF 3: Got measurement [0, 1, 1, 0]        #
# SHOT 2 OF 3: Got measurement [0, 1, 1, 0]        #
# SHOT 3 OF 3: Got measurement [0, 1, 1, 0]        #
```

## Usage:
Building is as simple as executing:
```
cargo build
```

Running `qlang` can either be performed through `cargo`:
```
cargo run [file]
```

Or otherwise:
```
./path_to_file/qlang.exe [file]
```


## Future Work:
The current release is functional, but still experimental, and several improvements and features have been planned before the language can be considered stable.

Architectural improvements include:
- Refactoring evaluation architecture:
  - The language's evaluation of expressions and circuits can be tightened, and require a more modular separation of concerns.
  - With the introduction of static type checking, the evaluator can be adjusted to focus on a more narrow concern.
- Improving Error messaging.
- Implementing an automated and extensible testing suite (that is compatible with `cargo test`)
- Updating unitary oracles → oracles are currently constrained, in that they must be applied to an entire `QubitRegister`'s domain.

Planned features include:
- Implementing the `SWAP`, `CZ`, `CSWAP` and Toffoli (`CCNOT`) gates.
- Implementing a `printCircuit` mechanism for pretty-printing circuit diagrams.


## Notes:
This project was created as a learning exercise in programming language construction, and quantum computing. Although `qlang` is capable of correctly evaluating small circuits, It should not be considered as a substitute for more mature languages, libraries, or frameworks.

This work was heavily inspired by Robert Nystrom's [Crafting Interpreters](https://craftinginterpreters.com/), which was an excellent reference while I was building the language's lexer and parser. I would also like to credit Jack Hidary's [Quantum Computing: An Applied Approach](https://link.springer.com/book/10.1007/978-3-030-83274-2), for his accessible introduction to quantum computing.