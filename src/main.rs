use std::error::Error;

use rustyline::{Editor, error::ReadlineError};

const HISTORY: &'static str = ".lox_history.txt";

fn repl() -> Result<(), Box<dyn Error>> {
    let mut rl = Editor::<()>::new();
    rl.load_history(HISTORY).unwrap_or(());

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

    rl.save_history(HISTORY)?;
    Ok(())
}

fn main() {
    repl().expect("Repl error")
}
