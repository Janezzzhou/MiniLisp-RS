use std::io::{self, Write};

use mini_lisp_rs::tokenize;

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

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
                for t in tokens {
                    println!("{}", t);
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}
