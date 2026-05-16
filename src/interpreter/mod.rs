mod lexer;
mod parser;
mod type_checker;
mod evaluator;

use crate::interpreter::lexer::Lexer;
use crate::interpreter::parser::Parser;
use crate::interpreter::type_checker::TypeChecker;
use crate::interpreter::evaluator::Evaluator;

pub struct Interpreter { }
impl Interpreter {
    pub fn interpret(file: &String) -> () {
        let mut lexer: Lexer = Lexer::new();
        if let Err(e) = lexer.load_file(file) {
            panic!("Error: {}", e);
        }
        let result = lexer.scan_tokens();
        if let Err(e) = result { lexer.show_error(e); }

        let mut parser: Parser = Parser::new(lexer.tokens);
        let result = parser.parse();
        if let Err(e) = result { panic!("Parsing Error: {}", e); }
        let result = result.unwrap();

        let mut tc: TypeChecker = TypeChecker::new();
        match tc.ensures(&result) {
            Ok(_) => { }, //static check passed, safe to evaluate
            Err(e) => { panic!("Typing Error: {}", e); }
        }

        let mut evaluator: Evaluator = Evaluator::new(result);
        let result = evaluator.eval();
        if let Err(e) = result { panic!("Runtime Error: {}", e); }
    }
}