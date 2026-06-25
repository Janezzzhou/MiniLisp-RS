use crate::token::Token;
use crate::error::LispError;

pub fn tokenize(input: &str) -> Result<Vec<Token>, LispError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            '(' => {
                chars.next();
                tokens.push(Token::LeftParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RightParen);
            }
            c if c.is_whitespace() => {
                chars.next();
            }
            c if c.is_ascii_digit() || (c == '-' && chars.clone().nth(1).map_or(false, |n| n.is_ascii_digit())) => {
                // parse number (simple)
                let mut s = String::new();
                if c == '-' {
                    s.push('-');
                    chars.next();
                }
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() || d == '.' {
                        s.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                match s.parse::<f64>() {
                    Ok(n) => tokens.push(Token::NumericLiteral(n)),
                    Err(_) => return Err(LispError::InvalidNumber(s)),
                }
            }
            _ => {
                // identifier: run of non-space, non-paren chars
                let mut s = String::new();
                while let Some(&c2) = chars.peek() {
                    if c2.is_whitespace() || c2 == '(' || c2 == ')' {
                        break;
                    }
                    s.push(c2);
                    chars.next();
                }
                tokens.push(Token::Identifier(s));
            }
        }
    }

    Ok(tokens)
}
