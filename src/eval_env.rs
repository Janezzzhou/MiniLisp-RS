use crate::error::LispError;
use crate::value::ValuePtr;

pub struct EvalEnv;

impl EvalEnv {
    pub fn new() -> Self {
        EvalEnv
    }

    pub fn eval(&mut self, expr: ValuePtr) -> Result<ValuePtr, LispError> {
        // 1. 自求值表达式：布尔、数值、字符串
        if expr.is_self_evaluating() {
            return Ok(expr);
        }

        // 2. 空表求值报错
        if expr.is_nil() {
            return Err(LispError::RuntimeError(
                "Evaluating nil is prohibited.".to_string(),
            ));
        }

        // 3. 其它情况暂未实现
        Err(LispError::RuntimeError("Unimplemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::tokenizer::tokenize;

    fn eval_str(input: &str) -> Result<String, LispError> {
        let tokens = tokenize(input).map_err(|e| LispError::SyntaxError(e.to_string()))?;
        let mut parser = Parser::new(tokens);
        let expr = parser.parse()?;
        let mut env = EvalEnv::new();
        let result = env.eval(expr)?;
        Ok(result.to_string())
    }

    #[test]
    fn test_self_evaluating() {
        assert_eq!(eval_str("42").unwrap(), "42");
        assert_eq!(eval_str("#t").unwrap(), "#t");
        assert_eq!(eval_str("#f").unwrap(), "#f");
        assert_eq!(eval_str("\"hello\"").unwrap(), "\"hello\"");
    }

    #[test]
    fn test_nil_error() {
        let err = eval_str("()").unwrap_err();
        match err {
            LispError::RuntimeError(msg) => {
                assert_eq!(msg, "Evaluating nil is prohibited.");
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_unimplemented() {
        let err = eval_str("(+ 1 2)").unwrap_err();
        match err {
            LispError::RuntimeError(msg) => {
                assert_eq!(msg, "Unimplemented");
            }
            _ => panic!("Expected RuntimeError"),
        }
    }
}