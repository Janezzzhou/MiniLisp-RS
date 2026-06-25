use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftParen,                  // 左括号 (
    RightParen,                 // 右括号 ) 
    Quote,                      // 单引号 '
    Quasiquote,                 // 反引号 `
    Unquote,                    // 逗号 ,
    Dot,                        // 点 .
    BooleanLiteral(bool),       // 布尔字面量 #f 或 #t
    NumericLiteral(f64),        // 数型字面量如 42
    StringLiteral(String),   // 字符串字面量如 "Hello"
    Identifier(String),         // 标识符（变量名）如 +
}

//Token的打印格式定义
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::LeftParen => write!(f, "(LEFT_PAREN)"),
            Token::RightParen => write!(f, "(RIGHT_PAREN)"),
            Token::Quote => write!(f, "(QUOTE)"),
            Token::Quasiquote => write!(f, "(QUASIQUOTE)"),
            Token::Unquote => write!(f, "(UNQUOTE)"),
            Token::Dot => write!(f, "(DOT)"),
            Token::BooleanLiteral(value) => write!(f, "(BOOLEAN_LITERAL {})", if *value { "true" } else { "false" }),
            Token::NumericLiteral(n) => write!(f, "(NUMERIC_LITERAL {:.6})", n),
            Token::StringLiteral(s) => write!(f, "(STRING_LITERAL {:?})", s),
            Token::Identifier(s) => write!(f, "(IDENTIFIER {})", s),
        }
    }
}
