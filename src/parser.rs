use crate::error::LispError;
use crate::token::Token;
use crate::value::Value;
use crate::value::ValuePtr;
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
            Token::LeftParen => self.parse_tails(),
            Token::Quote => {let expr = self.parse()?;
                Ok(Self::make_list(vec![
                    Value::Symbol("quote".into()),
                    expr,
                ]))
            }
            Token::Quasiquote => {
                let expr = self.parse()?;
                Ok(Self::make_list(vec![
                    Value::Symbol("quasiquote".into()),
                    expr,
                ]))
            }
            Token::Unquote => {
                let expr = self.parse()?;
                Ok(Self::make_list(vec![
                    Value::Symbol("unquote".into()),
                    expr,
                ]))
            }
            other => Err(LispError::SyntaxError(format!("Unexpected token: {}", other))),
        }
    }

    fn pop_token(&mut self) -> Result<Token, LispError> {
        self.tokens
            .pop_front()
            .ok_or_else(|| LispError::SyntaxError("Unexpected end of input".into()))
    }

    fn peek_token(&self) -> Option<&Token> {
        self.tokens.front()
    }

    fn make_list(values: Vec<Value>) -> Value {
        values
            .into_iter()
            .rev()
            .fold(Value::Nil, |cdr, car| {
                Value::Pair(ValuePtr::new(car), ValuePtr::new(cdr),)
            })
    }

    pub fn parse_tails(&mut self) -> Result<Value, LispError> {
        // ()
        if matches!(self.peek_token(), Some(Token::RightParen)) {
            self.pop_token()?; // 吃掉 )
            return Ok(Value::Nil);
        }

        // 解析 car
        let car = self.parse()?;

        // (a . b)
        if matches!(self.peek_token(), Some(Token::Dot)) {
            self.pop_token()?; // 吃掉 .

            let cdr = self.parse()?;

            match self.pop_token()? {
                Token::RightParen => {}
                other => {
                    return Err(LispError::SyntaxError(format!(
                        "Expected ')', got {}",
                        other
                    )))
                }
            }

            return Ok(Value::Pair(ValuePtr::new(car),ValuePtr::new(cdr),));
        }

        // (a b c)
        let cdr = self.parse_tails()?;

        Ok(Value::Pair(ValuePtr::new(car),ValuePtr::new(cdr),))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;

    fn eval(input: &str) -> String {
        let tokens = tokenize(input).unwrap();
        let mut parser = Parser::new(tokens);
        let value = parser.parse().unwrap();
        value.to_string()
    }

    #[test]
    fn parse_number() {
        assert_eq!(eval("42"), "42");
    }

    #[test]
    fn parse_quote() {
        assert_eq!(
            eval("'abc"),
            "(quote abc)"
        );
    }

    #[test]
    fn parse_list() {
        assert_eq!(
            eval("(a b c)"),
            "(a b c)"
        );
    }

    #[test]
    fn parse_dotted_pair() {
        assert_eq!(
            eval("(a . b)"),
            "(a . b)"
        );
    }

    #[test]
    fn parse_nested_pair() {
        assert_eq!(
            eval("(a . (b . c))"),
            "(a b . c)"
        );
    }
}