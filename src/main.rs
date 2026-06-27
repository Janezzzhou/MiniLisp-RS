use std::io::{self, Write};

use mini_lisp_rs::{tokenize, EvalEnv, LispError, Parser};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let env = EvalEnv::new();

    loop {
        // Prompt
        print!(">>> ");
        stdout.flush()?;

        let mut input = String::new();
        let bytes = stdin.read_line(&mut input)?;
        if bytes == 0 {
            // 输入为空
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        match tokenize(input) {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens);
                match parser.parse(){
                    Ok(expr) => match EvalEnv::eval(&env, expr) {
                        Ok(value) => println!("{}", value),
                        Err(LispError::Exit(code)) => std::process::exit(code),
                        Err(e) => eprintln!("Error: {}", e),
                    },
                    Err(e) => eprintln!("Parse error: {}", e),
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}
