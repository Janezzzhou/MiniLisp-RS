use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftParen,
    RightParen,
    Identifier(String),
    NumericLiteral(f64),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::LeftParen => write!(f, "(LEFT_PAREN)"),
            Token::RightParen => write!(f, "(RIGHT_PAREN)"),
            Token::Identifier(s) => write!(f, "(IDENTIFIER {})", s),
            Token::NumericLiteral(n) => write!(f, "(NUMERIC_LITERAL {:.6})", n),
        }
    }
}
