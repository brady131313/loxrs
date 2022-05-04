use std::path::Path;

use lox_rs::vm::{InterpretError, Vm};
use rustyline::{error::ReadlineError, Editor};

const HISTORY: &'static str = ".lox_history.txt";

fn repl() {
    let mut rl = Editor::<()>::new();
    rl.load_history(HISTORY).unwrap_or(());

    let mut vm = Vm::new();
    loop {
        let readline = rl.readline("lox> ");
        match readline {
            Ok(line) => {
                if let Ok(_) = vm.interpret(line.as_str()) {
                    rl.add_history_entry(line.as_str());
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {err:?}");
                break;
            }
        }
    }

    if let Err(e) = rl.save_history(HISTORY) {
        eprintln!("Failed to save history, {e}")
    }
}

fn run_file<P: AsRef<Path>>(path: P) {
    let src = match std::fs::read_to_string(path) {
        Ok(src) => src,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(74)
        }
    };

    let mut vm = Vm::new();
    match vm.interpret(&src) {
        Ok(_) => {}
        Err(InterpretError::Compile) => std::process::exit(65),
        Err(InterpretError::Runtime) => std::process::exit(70),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => repl(),
        2 => run_file(&args[1]),
        _ => {
            eprintln!("Usage: lox_rs [path]");
            std::process::exit(64)
        }
    }
}
