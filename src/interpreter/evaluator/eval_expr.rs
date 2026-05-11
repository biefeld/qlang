use crate::interpreter::evaluator::Evaluator;
use qlang::engine::black_box::Lambda;
use qlang::engine::register_error::RegisterError;

use crate::interpreter::evaluator::runtime_error::RuntimeError;
use crate::interpreter::evaluator::environment::{Bits, EvaluatorType, Function, Environment}; 
use crate::interpreter::parser::ast_types as ast;
use std::collections::HashMap;

// ----- NORMAL EXPRESSIONS -----
impl Evaluator {

    pub fn eval_expr(
        &mut self,
        expr: &ast::Expr,
        args: Option<&Vec<usize>>, 
        table: Option<&Vec<(String, usize)>>
    ) -> Result<EvaluatorType, RuntimeError> {
        match expr {
            ast::Expr::BitsLiteral(bitstring) => {
                let trimmed_str: String = bitstring[2..].to_string();
                let value_of: usize = usize::from_str_radix(&trimmed_str, 2)?;
                let bits: Bits = Bits { literal: value_of, length: trimmed_str.len() };
                return Ok(EvaluatorType::Bits(bits));
            },

            ast::Expr::Grouping(boxed_expr) => {
                return self.eval_expr(boxed_expr, args, table)
            }

            ast::Expr::Identifier(var_name) => {
                // Ensure that var_name can be found in the function args, if any exist
                if let Some(b) = get_var_from_local_scope(var_name, args, table) {
                    return Ok(b);
                };

                // Not found in local scope, so consult the environment
                match self.environment.get(var_name) {
                    Some(res) => Ok(res.clone()),
                    None => Err(RuntimeError::VarNotFound(var_name.to_string()))
                }
            },

            ast::Expr::Binary { op, left, right } => {
                // Extract bitstrings from expressions
                let pair: (EvaluatorType, EvaluatorType) = (
                    self.eval_expr(left, args, table)?,
                    self.eval_expr(right, args, table)?
                );
                let (left_res, right_res) = match pair {
                    (EvaluatorType::Bits(l), EvaluatorType::Bits(r)) => (l, r),
                    (_, _) => { return Err(RuntimeError::TypeMismatch); }
                };

                // ensure that left_res and right_res are compatible
                if left_res.length != right_res.length { return Err(RuntimeError::TypeMismatch); }
                let mut length: usize = left_res.length;
                let combine: usize = match op {
                    ast::BinOp::And => left_res.literal & right_res.literal,
                    ast::BinOp::Or =>  left_res.literal | right_res.literal,
                    ast::BinOp::Xor => left_res.literal ^ right_res.literal,
                    ast::BinOp::DotProduct => {
                        length = 1;
                        let mut result: usize = 0;
                        for i in 0..left_res.length {
                            let l: usize = (left_res.literal >> i) & 1;
                            let r: usize = (right_res.literal >> i) & 1;
                            result ^= l & r; // xorring is functionally equivalent to mod2 addition.
                        }
                        result
                    }
                };
                let bits: Bits = Bits { length: length, literal: combine };
                return Ok(EvaluatorType::Bits(bits));
            },

            ast::Expr::Call { callee, args: call_args } => {
                // evaluate args to their literal values
                let reduced_args: Result<Vec<EvaluatorType>, RuntimeError> = call_args.iter()
                    .map(|expr| self.eval_expr(expr, args, table)).collect();
                let reduced_args = reduced_args?;
                let mut base10_args: Vec<usize> = Vec::new();
                for e in &reduced_args {
                    match e {
                        EvaluatorType::Bits(b) => { base10_args.push(b.literal); },
                        _ => { return Err(RuntimeError::TypeMismatch); }
                    }
                }

                // ensure that func_name points to an actual function
                let func_name = match &**callee { // ?
                    ast::Expr::Identifier(f) => f.clone(),
                    _ => { return Err(RuntimeError::TypeMismatch); }
                };
                
                let func: Function = match self.environment.clone().get(&func_name) {
                    Some(EvaluatorType::Function(func)) => func.clone(),
                    _ => { return Err(RuntimeError::TypeMismatch); }
                };

                // check that arity matches between the two
                let expected: usize = func.input.len();
                let actual: usize = call_args.len();
                if expected != actual {
                    return Err(RuntimeError::IncorrectArgs(expected, actual));
                }

                // execute
                let result = (func.func)(base10_args)?;
                let bits: Bits = Bits { length: func.output, literal: result };
                return Ok(EvaluatorType::Bits(bits));
            }
        }
    }
}



// ----- FUNCTION DEFINITIONS FROM EXPRS -----
impl Evaluator {
    /// Pre: The first argument is a symbol table, `[(x, sizeof(x)), (y, sizeof(y))]`
    /// 
    /// Given an expression from a function body and a symbol table, build
    ///  a closure derived from the function body. If an identifier is found
    /// in the function, it is binded either to the environment, or to the symbol table.
    pub fn build_closure_from_expr(&mut self, expr: &ast::Expr, table: &Vec<(String, usize)>) -> Lambda {
        let expr_owned = expr.clone();
        let table_owned = table.clone();
        let captured_env: HashMap<String, EvaluatorType> = self.environment.clone();
        
        std::sync::Arc::new(move |args: Vec<usize>| {
            let temp_env: Environment = captured_env.clone();
            let mut temp_evaluator: Evaluator = Evaluator {
                program: Vec::new(),
                environment: temp_env
            };
            let result = temp_evaluator.eval_expr(&expr_owned, Some(&args), Some(&table_owned));
            match result {
                Ok(EvaluatorType::Bits(b)) => Ok(b.literal),
                _ => Err(RegisterError::RunTimeFailure)
            }
        })
    }
}



// ----- HELPERS -----
/// Given a variable name `var_name`, attempt to
/// resovle `var_name` in a set of function args
/// `args` and `table`. Returns None if the variable
/// could not be resolved.
fn get_var_from_local_scope(
    var_name: &String,
    args: Option<&Vec<usize>>,
    table: Option<&Vec<(String, usize)>>
) -> Option<EvaluatorType> {
    if table.is_none() || args.is_none() { return None; }
    let table = table.unwrap();
    let args = args.unwrap();

    for i in 0..table.len() {
        if table[i].0 != *var_name { continue; }
        // match made
        let (size, value) = (table[i].1, args[i]);
        let bits: EvaluatorType = EvaluatorType::Bits(Bits { literal: value, length: size });
        return Some(bits);
    }
    return None;
}