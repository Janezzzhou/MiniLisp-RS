use std::env;
use std::fs;
use std::io::{self, Write};

use mini_lisp_rs::{execute_source, EvalEnv, LispError};

fn run_file(path: &str) -> io::Result<()> {
    let source = fs::read_to_string(path)?;
    let env = EvalEnv::new();

    match execute_source(&source, &env, false) {
        Ok(()) => Ok(()),
        Err(LispError::Exit(code)) => std::process::exit(code),
        Err(e) => {
            eprintln!("Error: {}", e);
            Ok(())
        }
    }
}

fn run_repl() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let env = EvalEnv::new();

    loop {
        print!(">>> ");
        stdout.flush()?;

        let mut input = String::new();
        let bytes = stdin.read_line(&mut input)?;
        if bytes == 0 {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        match execute_source(input, &env, true) {
            Ok(()) => {}
            Err(LispError::Exit(code)) => std::process::exit(code),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    if let Some(path) = args.next() {
        run_file(&path)
    } else {
        run_repl()
    }
}
