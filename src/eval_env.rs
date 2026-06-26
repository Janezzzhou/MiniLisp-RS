use crate::error::LispError;
use crate::value::ValuePtr;
use crate::value::Value;
use std::collections::HashMap;

pub struct EvalEnv {
    symbols: HashMap<String, ValuePtr>,
}

impl EvalEnv {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    pub fn eval(&mut self, expr: ValuePtr) -> Result<ValuePtr, LispError> {
        if expr.is_self_evaluating() {
            return Ok(expr);
        }

        if expr.is_nil() {
            return Err(LispError::RuntimeError(
                "Evaluating nil is prohibited.".into(),
            ));
        }

        // Symbol
        if let Some(name) = expr.as_symbol() {
            if let Some(value) = self.symbols.get(name) {
                return Ok(value.clone());
            }
            return Err(LispError::RuntimeError(
                format!("Variable {} not defined.", name),
            ));
        }

        // List
        let v = expr.to_vec()?;
        if v.is_empty() {
            return Err(LispError::RuntimeError("Empty list".into()));
        }
        // 检查 define 特殊形式
        if let Some(op) = v[0].as_symbol() {
            if op == "define" {
                // 参数数量检查
                if v.len() != 3 {
                    return Err(LispError::RuntimeError(
                        "define requires exactly 2 arguments".into(),
                    ));
                }
                // 获取变量名（必须是符号）
                let name = v[1]
                    .as_symbol()
                    .ok_or_else(|| LispError::RuntimeError("define first arg must be symbol".into()))?;
                // 对值表达式求值
                let value = self.eval(v[2].clone())?;
                // 存入符号表
                self.symbols.insert(name.to_string(), value);
                return Ok(ValuePtr::new(Value::Nil));
            }
        }

        Err(LispError::RuntimeError(
            "Unimplemented".into(),
        ))
    }
}

// src/eval_env.rs 末尾添加

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::tokenizer::tokenize;

    // 每次新建环境，用于独立的求值测试
    fn eval_str(input: &str) -> Result<String, LispError> {
        let tokens = tokenize(input).map_err(|e| LispError::SyntaxError(e.to_string()))?;
        let mut parser = Parser::new(tokens);
        let expr = parser.parse()?;
        let mut env = EvalEnv::new();
        let result = env.eval(expr)?;
        Ok(result.to_string())
    }

    // 在指定环境中求值（用于需要共享状态的测试）
    fn eval_in_env(env: &mut EvalEnv, input: &str) -> Result<String, LispError> {
        let tokens = tokenize(input).map_err(|e| LispError::SyntaxError(e.to_string()))?;
        let mut parser = Parser::new(tokens);
        let expr = parser.parse()?;
        let result = env.eval(expr)?;
        Ok(result.to_string())
    }

    #[test]
    fn test_self_evaluating() {
        assert_eq!(eval_str("42").unwrap(), "42");
        assert_eq!(eval_str("#t").unwrap(), "#t");
        assert_eq!(eval_str("#f").unwrap(), "#f");
        assert_eq!(eval_str("\"Hello\"").unwrap(), "\"Hello\"");
    }

    #[test]
    fn test_nil_evaluation_error() {
        let err = eval_str("()").unwrap_err();
        match err {
            LispError::RuntimeError(msg) => assert_eq!(msg, "Evaluating nil is prohibited."),
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_define_and_lookup() {
        let mut env = EvalEnv::new();

        assert_eq!(eval_in_env(&mut env, "(define x 42)").unwrap(), "()");
        assert_eq!(eval_in_env(&mut env, "x").unwrap(), "42");

        assert_eq!(eval_in_env(&mut env, "(define y x)").unwrap(), "()");
        assert_eq!(eval_in_env(&mut env, "y").unwrap(), "42");

        let err = eval_in_env(&mut env, "z").unwrap_err();
        match err {
            LispError::RuntimeError(msg) => assert_eq!(msg, "Variable z not defined."),
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_malformed_define() {
        // 缺少参数
        let err = eval_str("(define x)").unwrap_err();
        assert!(matches!(err, LispError::RuntimeError(_)));
        // 变量名不是符号
        let err = eval_str("(define 123 42)").unwrap_err();
        assert!(matches!(err, LispError::RuntimeError(_)));
        // 非法列表（点对），to_vec 会报错
        let err = eval_str("(define x . 42)").unwrap_err();
        assert!(matches!(err, LispError::RuntimeError(_)));
    }

    #[test]
    fn test_unimplemented() {
        let err = eval_str("(+ 1 2)").unwrap_err();
        match err {
            LispError::RuntimeError(msg) => assert_eq!(msg, "Unimplemented"),
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_env_persistence() {
        let mut env = EvalEnv::new();
        // 构造 (define x 100)
        let expr1 = ValuePtr::new(Value::Pair(
            ValuePtr::new(Value::Symbol("define".to_string())),
            ValuePtr::new(Value::Pair(
                ValuePtr::new(Value::Symbol("x".to_string())),
                ValuePtr::new(Value::Pair(
                    ValuePtr::new(Value::Numeric(100.0)),
                    ValuePtr::new(Value::Nil),
                )),
            )),
        ));
        env.eval(expr1).unwrap();
        let expr2 = ValuePtr::new(Value::Symbol("x".to_string()));
        let result = env.eval(expr2).unwrap();
        assert_eq!(result.to_string(), "100");
    }
}