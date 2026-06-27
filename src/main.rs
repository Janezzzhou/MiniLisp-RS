use std::io::{self, Write};

use mini_lisp_rs::{tokenize, EvalEnv, Parser};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut env = EvalEnv::new();

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
                    Ok(expr) => match env.eval(expr) {
                        Ok(value) => println!("{}", value),
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
