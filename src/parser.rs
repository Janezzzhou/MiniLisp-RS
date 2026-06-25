use crate::error::LispError;
use crate::token::Token;
use crate::value::Value;
use std::collections::VecDeque;

pub struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    pub fn new(tokens: VecDeque<Token>) -> Self {
        Self { tokens }
    }

    // 入口方法，解析表达式
    pub fn parse(&mut self) -> Result<Value, LispError> {
        let token = self.pop_token()?;

        match token {
            Token::BooleanLiteral(value) => Ok(Value::Boolean(value)),
            Token::NumericLiteral(value) => Ok(Value::Numeric(value)),
            Token::StringLiteral(value) => Ok(Value::String(value)),
            Token::Identifier(symbol) => Ok(Value::Symbol(symbol)),
            other => Err(LispError::SyntaxError(format!("Unexpected token: {}", other))),
        }
    }

    fn pop_token(&mut self) -> Result<Token, LispError> {
        self.tokens
            .pop_front()
            .ok_or_else(|| LispError::SyntaxError("Unexpected end of input".into()))
    }
}
