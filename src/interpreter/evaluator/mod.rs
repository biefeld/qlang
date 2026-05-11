pub mod runtime_error;
mod eval_expr;
mod eval_circuit;
mod eval_method;
mod environment;

use std::collections::HashMap;
use qlang::engine::black_box::Lambda;
use crate::interpreter::evaluator::runtime_error::RuntimeError;
use crate::interpreter::evaluator::environment::{Environment, EvaluatorType,};
use crate::interpreter::evaluator::environment::{Bits, Function, Oracle}; 
use crate::interpreter::parser::ast_types as ast;


pub struct Evaluator {
    program: ast::Program,
    environment: Environment
}


/// EVALUATION ROUTINES 
impl Evaluator {
    /// consumes a `bits` assignment, and loads the respective variable into
    /// `self.environment`.
    fn eval_assignment(&mut self, bits: &ast::Assignment) -> Result<(), RuntimeError> {
        let var_name: String = bits.name.clone();
        let literal: Bits = match self.eval_expr(&bits.value, None, None)? {
            EvaluatorType::Bits(b) => b,
            _ => { return Err(RuntimeError::TypeMismatch); }
        };
        self.environment.insert(var_name, EvaluatorType::Bits(literal));
        return Ok(()); 
    }


    fn eval_function_decl(&mut self, decl: &ast::FunctionDecl) -> Result<(), RuntimeError> {
        let func_name: String = decl.name.clone();
        let output: usize = match decl.return_type {
            ast::Type::Bits(n) => n,
            _ => { return Err(RuntimeError::TypeMismatch); }
        };

        // while evaluating the function body expression, if we ever encounter an
        // identifier that cannot be resolved, (i.e., we catch RuntimeError::VarNotFound),
        // then we consult the symbol table to ensure that its a function parameter.
        // If we can't find the identifier in the symbol table, then we can be confident that
        // the variable is not found.
        let mut symbol_table: Vec<(String, usize)> = Vec::new();
        for symbol in &decl.params {
            let size: usize = match symbol.ty {
                ast::Type::Bits(n) => n,
                _ => { return Err(RuntimeError::TypeMismatch); }
            };
            let name: String = symbol.name.clone();
            symbol_table.push((name, size));
        }
        let input: Vec<usize> = symbol_table.iter()
            .map(|x| x.1)
            .collect();

        let func: Lambda = self.build_closure_from_expr(&decl.body, &symbol_table);   

        let func_literal: Function = Function { input, output, func };
        self.environment.insert(func_name, EvaluatorType::Function(func_literal));
        return Ok(());
    }


    fn eval_oracle_decl(&mut self, decl: &ast::OracleDecl) -> Result<(), RuntimeError> {
        let oracle_name:  String = decl.name.clone();
        let oracle_loads: String = decl.loads.clone();

        // ensure that `oracle_loads` exists in the environment
        let f = match self.environment.get(&oracle_loads) {
            None => { return Err(RuntimeError::VarNotFound(oracle_loads)) },
            Some(EvaluatorType::Function(f)) => f,
            Some(_) => { return Err(RuntimeError::TypeMismatch) }
        };

        // ensure that `f :: bits[x] -> bits[y]` matches the
        // cardinality for `|x, y>`
        let (in_size, out_size) = sanitize_oracle(decl)?;
        if f.input[0] != in_size { return Err(RuntimeError::OracleConstructionFailed); }
        if f.output != out_size { return Err(RuntimeError::OracleConstructionFailed); }

        // oracle is safe to construct
        let oracle: Oracle = Oracle { input: (in_size, out_size), loads: f.clone() };
        self.environment.insert(oracle_name, EvaluatorType::Oracle(oracle));
        return Ok(());
    }


    fn eval_statement(&mut self, stmt: ast::Statement) -> Result<(), RuntimeError> {
        match stmt {
            ast::Statement::Expr(_) => { Ok(()) }, // vacuous expressions can be skipped.
            ast::Statement::Assignment(bits) => self.eval_assignment(&bits),
            ast::Statement::Function(decl) => self.eval_function_decl(&decl),
            ast::Statement::Oracle(decl) => self.eval_oracle_decl(&decl),
            ast::Statement::Circuit(decl) => self.eval_circuit_decl(&decl),
            ast::Statement::MethodCall(call) => self.eval_method_call(&call)
        }
    }
}



/// PUBLIC API METHODS
impl Evaluator {
    pub fn new(program: ast::Program) -> Self {
        let environment: Environment = HashMap::new();
        Self { program, environment }
    }

    pub fn eval(&mut self) -> Result<(), RuntimeError> {
        for _ in 0..self.program.len() {
            let stmt: ast::Statement = self.program.remove(0);
            self.eval_statement(stmt)?;
        }
        return Ok(());
    }
}


// ----- HELPERS -----
/// Given an oracle declaration, ensure that it is correctly typed.
/// 
/// This includes ensuring that it contains exactly two params (both
/// of which must be typed as `qubits`), and ensuring that the first
/// param is greater-than or equal to the second.
fn sanitize_oracle(decl: &ast::OracleDecl) -> Result<(usize, usize), RuntimeError> {
    let length: usize = decl.params.len();
    if length != 2 {
        return Err(RuntimeError::IncorrectArgs(2, length));
    }

    let in_size: usize = match decl.params[0].ty {
        ast::Type::Qubits(n) => n,
        _ => { return Err(RuntimeError::TypeMismatch); }
    };
    let out_size: usize = match decl.params[1].ty {
        ast::Type::Qubits(n) => n,
        _ => { return Err(RuntimeError::TypeMismatch); }
    };

    match in_size >= out_size {
        true => Ok((in_size, out_size)),
        false => Err(RuntimeError::OracleConstructionFailed)
    }
}