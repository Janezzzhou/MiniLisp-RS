use std::fmt;

#[derive(Debug)]
pub enum LispError {
    InvalidNumber(String),
    SyntaxError(String),
    RuntimeError(String),
    Exit(i32),
    Other(String),
}

impl fmt::Display for LispError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LispError::InvalidNumber(s) => write!(f, "Invalid number literal: {}", s),
            LispError::SyntaxError(s) => write!(f, "Syntax error: {}", s),
            LispError::RuntimeError(s) => write!(f, "Runtime error: {}", s),
            LispError::Exit(code) => write!(f, "Exit with code {}", code),
            LispError::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for LispError {}
