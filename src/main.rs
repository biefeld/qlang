mod interpreter;

use std::env;
use crate::interpreter::Interpreter;

fn main() {
    let argv: Vec<String> = env::args().collect();
    if argv.len() != 2 {
        println!("Usage: qlang.exe [file]");
        std::process::exit(0);
    }

    Interpreter::interpret(&argv[1]);
}