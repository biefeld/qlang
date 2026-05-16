mod lang_types;
mod type_error;
mod resolve_expr;

use std::collections::HashMap;
use qlang::engine::gate::Gate;
use crate::interpreter::type_checker::type_error::TypeError;
use crate::interpreter::type_checker::lang_types::{FunctionType, LanguageType};
use crate::interpreter::parser::ast_types::{CircuitRef, Program, Statement};
use crate::interpreter::parser::ast_types as ast; 


pub struct TypeChecker {
    identifier_types: HashMap<String, LanguageType>
}


/// ----- TYPE VERIFIERS -----
impl TypeChecker {
    fn ensure_stmt(&mut self, stmt: &Statement) -> Result<(), TypeError> {
        match stmt {
            Statement::Assignment(assign) => self.ensure_assignment(assign),
            Statement::Expr(expr) => {
                if let Err(e) = self.resolve_expr(expr) { return Err(e); }
                return Ok(());
            }, // even if Statement::Expr is vacuous, type-checking is necessary
            Statement::Function(f) => self.ensure_function_decl(f),
            Statement::Oracle(o) => self.ensure_oracle_decl(o),
            Statement::Circuit(c) => self.ensure_circuit_decl(c),

            Statement::MethodCall(m) => {
                // ensure that the callee exists as a circuit
                let name: String = m.name.to_string();
                let lookup = self.identifier_types.get(&name);
                match lookup {
                    Some(LanguageType::Circuit) => { return Ok(()); },
                    Some(l) => {
                        return Err(TypeError::Expected("circuit", l.label()));
                    },
                    None => {
                        return Err(TypeError::UnresolvedIdentifier(name))
                    }
                }
            }
        }
    }


    fn ensure_assignment(&mut self, assignment: &ast::Assignment) -> Result<(), TypeError> {
        let bitstring_name: &String = &assignment.name;
        let bitstring_value: LanguageType = self.resolve_expr(&assignment.value)?;
        match bitstring_value {
            LanguageType::Bits(_) => {
                self.identifier_types.insert(bitstring_name.clone(), bitstring_value);
            },
            _ => { return Err(TypeError::Expected("bits", bitstring_value.label())); }
        };
        return  Ok(());
    }


    // We can "cheat" here by working with lexical scoping - recall that function arguments override
    // existing values in the environment. To that end, we can clone the environment, and
    // insert the function arguments in as symbols. Then, we can simply resolve the expression, with a
    // new type checker, and ensure that the resulting bitstring is equal to `func.return_type`s cardinality.
    fn ensure_function_decl(&mut self, func: &ast::FunctionDecl) -> Result<(), TypeError> {
        let func_name: String = func.name.to_string();

        // parse parameters into a type-friendly structure
        let mut indices: Vec<usize> = Vec::new();
        let mut table: Vec<(String, LanguageType)> = Vec::new();
        for param in &func.params {
            let var_name: String = param.name.to_string();
            let var_value: LanguageType = match param.ty {
                ast::Type::Bits(n) => {
                    indices.push(n);
                    LanguageType::Bits(n)
                },
                _ => { return Err(TypeError::FunctionArgWrongType(func_name)); }
            };
            table.push((var_name, var_value));
        }

        // ensure that the output has a `bits` format
        let output_bits: usize;
        let output: LanguageType = match func.return_type {
            ast::Type::Bits(n) => {
                output_bits = n;
                LanguageType::Bits(n)
            },
            _ => { return Err(TypeError::FunctionReturnWrongType(func_name)); }
        };

        // override the environment with new values, and yield the actual return type
        let mut tmp_env: HashMap<String, LanguageType> = self.identifier_types.clone();
        for (name, value) in table {
            tmp_env.insert(name, value);
        }
        let new_checker: TypeChecker = TypeChecker { identifier_types: tmp_env };
        let return_type: LanguageType = new_checker.resolve_expr(&func.body)?;
        
        if return_type != output {
            return Err(TypeError::FunctionIncorrectlyTyped(func_name));
        }

        // function is well typed, so it can be inserted into the classical environment
        let func: FunctionType = FunctionType { input: indices, output: output_bits };
        self.identifier_types.insert(func_name, LanguageType::Function(func));
        return Ok(());
    }


    fn ensure_oracle_decl(&mut self, oracle: &ast::OracleDecl) -> Result<(), TypeError> {
        // fetch the attached function name in the environment, and ensure that its a function
        let loaded: &LanguageType = match self.identifier_types.get(&oracle.loads) {
            Some(f) => f,
            None => {
                return Err(TypeError::UnresolvedIdentifier(oracle.loads.to_string()));
            }
        };
        let f_type: &FunctionType = match loaded {
            LanguageType::Function(f) => f,
            _ => { return Err(TypeError::UnresolvedIdentifier(loaded.label())); }
        };

        let sanitised_oracle: LanguageType = sanitise_oracle(oracle, f_type)?;
        self.identifier_types.insert(oracle.name.to_string(), sanitised_oracle);
        return Ok(());
    }


    fn ensure_circuit_decl(&mut self, circuit: &ast::CircuitDecl) -> Result<(), TypeError> {
        let mut table: HashMap<String, usize> = HashMap::new();
        for qbit in &circuit.registers {
            table.insert(qbit.name.to_string(), qbit.init.len());
        }

        for instruction in &circuit.instructions {
            let name: String = instruction.name.to_string();
            let actual: usize = count_qubits(&instruction.args, &table)?;
            let requires: usize = self.get_gate_arity(
                &name,
                actual // in case we find `measure`
            )?;
            // Unary gates are always valid, because
            // each operation is applied per-qubit.
            if (requires != 1) && (requires != actual) {
                return Err(TypeError::InvalidGateAppl(name, requires, actual));
            }
        }

        self.identifier_types.insert(circuit.name.to_string(), LanguageType::Circuit);
        return Ok(());
    }
}


/// ----- API-CALLABLE METHODS -----
impl TypeChecker {
    pub fn new() -> Self {
        Self { identifier_types: HashMap::new() }
    }

    /// Given an array of statements, ensure
    /// that the program is type safe. i.e.,
    /// ensure that values compose nicely with
    /// respect to their types.
    pub fn ensures(&mut self, prg: &Program) -> Result<(), TypeError> {
        for stmt in prg {
            self.ensure_stmt(stmt)?;
        }
        Ok(())
    }
}


// ----- HELPERS -----
impl TypeChecker {
    // TODO: This function is redundant, considering that we perform the same
    // task in `eval_circuit.rs`...
    fn get_gate_arity(
        &self,
        gate_name: &String,
        m_arity: usize
    ) -> Result<usize, TypeError> {
        // check if `gate_name` is in the environment
        let lookup: Option<&LanguageType> = self.identifier_types.get(gate_name);
        match lookup {
            Some(LanguageType::Oracle(x, y)) => {
                return Ok(x + y);
            },
            Some(t) => {
                return Err(TypeError::Expected("oracle", t.label()))
            },
            None => { /* no-op */ }
        };
        
        // check if `gate_name` exists in the default gate set
        let name: String = gate_name.to_string();
        return match Gate::arity_from_string(gate_name, m_arity) {
            Some(n) => Ok(n),
            None => Err(TypeError::UnresolvedIdentifier(name))
        }
    }
}


/// Given a list of qubits, and a set of circuit instruction arguments,
/// resolve the number of qubits that are being passed
fn count_qubits(
    args: &Vec<CircuitRef>,
    env: &HashMap<String, usize>
) -> Result<usize, TypeError> {
    let mut acc: usize = 0;
    for arg in args {
        // attempt to resolve the number of qubits in arg.name
        let size = match env.get(&arg.name) {
            Some(val) => val,
            None => {
                let name = arg.name.to_string();
                return Err(TypeError::UnresolvedIdentifier(name))
            }
        };

        match arg.applies {
            ast::Applies::One(_) => { acc += 1; },
            ast::Applies::All => { acc += size; }
        }
    }

    return Ok(acc);
}


/// Given an oracle declaration and a function in the type
/// pre-environment, verify that oracle is valid, and return
/// a sanitised oracle.
fn sanitise_oracle(
    oracle: &ast::OracleDecl,
    loads: &FunctionType
) -> Result<LanguageType, TypeError> {
    // ensure that oracle and function have the correct lengths
    if oracle.params.len() != 2 || loads.input.len() != 1 {
        return Err(TypeError::Todo);
    }

    // ensure that loads.in[0], loads.out oracle[0],
    // and oracle[1] are (bits, bits, qbits, qbits)
    let input_bits: usize = loads.input[0];
    let output_bits: usize = loads.output;

    let oracle_input_qubits: usize = match oracle.params[0].ty {
        ast::Type::Qubits(n) => n,
        _ => { return Err(TypeError::Todo); }
    };
    let oracle_output_qubits: usize = match oracle.params[1].ty {
        ast::Type::Qubits(n) => n,
        _ => { return Err(TypeError::Todo); }
    };

    // ensure that loads.in[0] >= loads.out
    if input_bits < output_bits {
        return Err(TypeError::Todo);
    }

    // ensure that loads.in[0] == oracle[0],
    // and that loads.out == oracle[1]
    if (input_bits != oracle_input_qubits) || (output_bits != oracle_output_qubits) {
        return Err(TypeError::Todo);
    }

    // return prepared oracle
    Ok(LanguageType::Oracle(oracle_input_qubits, oracle_output_qubits))
}