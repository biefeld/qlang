use crate::interpreter::evaluator::Evaluator;
use crate::interpreter::evaluator::runtime_error::RuntimeError;
use crate::interpreter::evaluator::environment::{Bits, EvaluatorType, Function, Environment}; 
use crate::interpreter::parser::ast_types as ast;
use std::collections::HashMap;

// ----- NORMAL EXPRESSIONS -----
impl Evaluator {

    pub fn eval_expr(
        &mut self,
        expr: &ast::Expr,
    ) -> Result<EvaluatorType, RuntimeError> {
        match expr {
            ast::Expr::BitsLiteral(bitstring) => {
                let trimmed_str: String = bitstring[2..].to_string();
                let value_of: usize = usize::from_str_radix(&trimmed_str, 2)?;
                let bits: Bits = Bits { literal: value_of, length: trimmed_str.len() };
                return Ok(EvaluatorType::Bits(bits));
            },


            ast::Expr::Grouping(boxed_expr) => {
                return self.eval_expr(boxed_expr);
            }


            ast::Expr::Identifier(var_name) => {
                return match self.environment.resolve(var_name) {
                    Some(t) => Ok(t.clone()),
                    None => Err(RuntimeError::VarNotFound(var_name.to_string()))
                };
            },


            ast::Expr::Binary { op, left, right } => {
                // Extract bitstrings from expressions
                let pair: (EvaluatorType, EvaluatorType) = (
                    self.eval_expr(left)?,
                    self.eval_expr(right)?
                );
                let (left_res, right_res) = match pair {
                    (EvaluatorType::Bits(l), EvaluatorType::Bits(r)) => (l, r),
                    (_, _) => { return Err(RuntimeError::TypeMismatch); }
                };

                // should be covered by type checking
                if left_res.length != right_res.length {
                    return Err(RuntimeError::TypeMismatch);
                }
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

            
            // Pre: call_args is well-ordered
            ast::Expr::Call { callee, args: call_args } => {
                // evaluate args to their literal values
                let call_args: Result<Vec<EvaluatorType>, RuntimeError> = call_args.iter()
                    .map(|expr| self.eval_expr(expr))
                    .collect();
                let call_args: Vec<EvaluatorType> = call_args?;

                // ensure that func_name points to an actual function
                let func_name = match &**callee { // ?
                    ast::Expr::Identifier(f) => f.clone(),
                    _ => { return Err(RuntimeError::TypeMismatch); }
                };
                let func: Function = match self.environment.resolve(&func_name) {
                    Some(EvaluatorType::Function(func)) => func.clone(),
                    _ => { return Err(RuntimeError::TypeMismatch); }
                };

                // create a virtual environment and interpreter, and eval
                // the expression in the virtualised context
                let mut new_env: Environment = Environment {
                    working_env: HashMap::new(),
                    parent: Some(Box::from(self.environment.clone()))
                };
                // insert reduced args into the new environment.
                for i in 0..func.input.len() {
                    new_env.working_env.insert(
                        func.input[i].to_string(), 
                        call_args[i].clone()
                    );
                }
                let mut new_eval: Evaluator = Evaluator::new();
                new_eval.environment = new_env;
                return new_eval.eval_expr(&func.func);
            }
        }
    }
}