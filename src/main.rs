use std::error::Error;

use lox_rs::{chunk::{Chunk, OpCode}, vm::Vm};
use rustyline::{Editor, error::ReadlineError};

const _HISTORY: &'static str = ".lox_history.txt";

fn _repl() -> Result<(), Box<dyn Error>> {
    let mut rl = Editor::<()>::new();
    rl.load_history(_HISTORY).unwrap_or(());

    loop {
        let readline = rl.readline("lox> ");
        match readline {
            Ok(line) => {
                println!("{line}");
                rl.add_history_entry(line.as_str());
            },
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {err:?}");
                break;
            }
        }
    }

    rl.save_history(_HISTORY)?;
    Ok(())
}

fn main() {
    let mut vm = Vm::new();
    let mut chunk = Chunk::new();

    chunk.write_constant(5.0, 123);
    chunk.write_constant(7.0, 123);
    chunk.write_constant(6.0, 124);
    chunk.write_chunk(OpCode::Return, 124);

    // chunk.disassemble_chunk("test chunk");
    vm.interpret(chunk).unwrap();
}
