use crate::error::LispError;
use crate::value::ValuePtr;
use crate::value::Value;
use std::collections::HashMap;
use crate::builtins::builtin_map;

pub struct EvalEnv {
    symbols: HashMap<String, ValuePtr>,
}

impl EvalEnv {
    pub fn new() -> Self {
        Self {
            symbols: builtin_map(),
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
        let proc = self.eval(v[0].clone())?;
        let args = self.eval_list(&v[1..])?;
        self.apply(proc, args)
    }

    fn eval_list(&mut self,list: &[ValuePtr],) -> Result<Vec<ValuePtr>, LispError> {
        let mut result = Vec::new();
        for expr in list {
            result.push(self.eval(expr.clone())?);
        }
        Ok(result)
    }
    fn apply(&mut self, proc: ValuePtr, args: Vec<ValuePtr>,) -> Result<ValuePtr, LispError> {
        if let Some(f) = proc.as_builtin_proc() {
            return f(args);
        }
        Err(LispError::RuntimeError("Unimplemented".into()))
    }
}

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
        let err = eval_str("(define x)").unwrap_err();
        assert!(matches!(err, LispError::RuntimeError(_)));

        let err = eval_str("(define 123 42)").unwrap_err();
        assert!(matches!(err, LispError::RuntimeError(_)));

        let err = eval_str("(define x . 42)").unwrap_err();
        assert!(matches!(err, LispError::RuntimeError(_)));
    }

    // --- 新增测试：内置过程 + 和 print ---

    #[test]
    fn test_builtin_add() {
        // 单个参数
        assert_eq!(eval_str("(+ 1 2)").unwrap(), "3");
        // 零参数
        assert_eq!(eval_str("(+)").unwrap(), "0");
        // 多个参数
        assert_eq!(eval_str("(+ 1 2 3 4 5)").unwrap(), "15");
        // 浮点数
        assert_eq!(eval_str("(+ 1.5 2.5)").unwrap(), "4");
    }

    #[test]
    fn test_define_with_add() {
        let mut env = EvalEnv::new();

        assert_eq!(eval_in_env(&mut env, "(define x (+ 1 2))").unwrap(), "()");
        assert_eq!(eval_in_env(&mut env, "x").unwrap(), "3");

        assert_eq!(eval_in_env(&mut env, "(+ x 4)").unwrap(), "7");
    }

    #[test]
    fn test_define_alias() {
        let mut env = EvalEnv::new();

        assert_eq!(eval_in_env(&mut env, "(define add +)").unwrap(), "()");
        assert_eq!(eval_in_env(&mut env, "(add 1 2 3)").unwrap(), "6");
        // 继续使用原 + 也正常
        assert_eq!(eval_in_env(&mut env, "(+ 10 20)").unwrap(), "30");
    }

    #[test]
    fn test_nested_add() {
        let mut env = EvalEnv::new();

        assert_eq!(eval_in_env(&mut env, "(define add +)").unwrap(), "()");
        assert_eq!(eval_in_env(&mut env, "(+ 1 (add 2))").unwrap(), "3");
        assert_eq!(eval_in_env(&mut env, "(+ (add 1 2) (add 3 4))").unwrap(), "10");
    }

    #[test]
    fn test_builtin_print() {
        // print 返回空表
        assert_eq!(eval_str("(print 42)").unwrap(), "()");
        // print 可以接受多个参数（通常实现为逐个打印，但返回值是空表）
        // 这里只测试返回值，输出无法在单元测试中轻易捕获，故忽略
        assert_eq!(eval_str("(print 1 2 3)").unwrap(), "()");
        // print 表达式本身是自求值的？不，它是过程调用
        // 嵌套使用
        assert_eq!(eval_str("(print (+ 1 2))").unwrap(), "()");
    }

    // 修改原来的 test_unimplemented，针对未定义的过程
    #[test]
    fn test_unknown_procedure() {
        let err = eval_str("(unknown 1 2)").unwrap_err();
        match err {
            LispError::RuntimeError(msg) => {
                // 期望错误信息包含 "not defined" 或类似
                assert!(msg.contains("not defined") || msg.contains("Variable") || msg.contains("Unimplemented"),
                    "Unexpected error message: {}", msg);
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_env_persistence() {
        let mut env = EvalEnv::new();
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