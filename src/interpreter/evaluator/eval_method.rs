use crate::interpreter::evaluator::Evaluator;
use crate::interpreter::evaluator::environment::{Circuit, EvaluatorType};
use crate::interpreter::evaluator::runtime_error::RuntimeError;
use crate::interpreter::parser::ast_types as ast;

impl Evaluator {
    pub fn eval_method_call(&mut self, call: &ast::MethodCall) -> Result<(), RuntimeError> {
        let circuit_name = &call.name;
        let c: Circuit = match self.environment.get(circuit_name) {
            Some(EvaluatorType::Circuit(c)) => c.clone(),
            Some(_) => { return Err(RuntimeError::TypeMismatch); }
            None => { return Err(RuntimeError::VarNotFound(circuit_name.clone())); }
        };

        match call.call.as_str() {
            "distribution" => {
                let shots: usize = get_arg(&call.args, "shots").unwrap_or(1);
                println!("Executing circuit '{}' {} time/s.", circuit_name, shots);
                for i in 0..shots {
                    print!("SHOT {} OF {}: ", i+1, shots);
                    c.execute(true)?;
                }
                println!("");
            },

            "measure" => {
                let shots: usize = get_arg(&call.args, "shots").unwrap_or(1);
                println!("Measuring circuit '{}' {} time/s.", circuit_name, shots);
                for i in 0..shots {
                    print!("SHOT {} OF {}: ", i+1, shots);
                    let result: Vec<usize> = c.execute(false)?;
                    println!("Got measurement {:?}", result);
                }
                println!("");
            },

            "printCircuit" => todo!(),

            _ => {
                let name: String = call.call.to_string();
                return Err(RuntimeError::MethodUndefined(name));
            }
        }

        return  Ok(());
    }
}

// ----- HELPERS -----
fn get_arg(args: &Vec<ast::MethodArg>, arg_name: &str) -> Option<usize> {
    for arg in args {
        if arg.name.as_str() == arg_name { return Some(arg.value); }
    }
    return None;
}