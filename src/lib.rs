pub mod token;
pub mod tokenizer;
pub mod error;
pub mod value;
pub mod parser;
pub mod eval_env;
pub mod builtins;
pub mod form;

pub use token::Token;
pub use tokenizer::tokenize;
pub use error::LispError;
pub use value::{Value, ValuePtr};
pub use parser::Parser;
pub use eval_env::{EnvPtr, EvalEnv};

pub fn execute_source(source: &str, env: &EnvPtr, print_results: bool) -> Result<(), LispError> {
    let tokens = tokenize(source)?;
    let mut parser = Parser::new(tokens);

    while parser.has_tokens() {
        let expr = parser.parse()?;
        let value = EvalEnv::eval(env, expr)?;
        if print_results {
            println!("{}", value);
        }
    }

    Ok(())
}
