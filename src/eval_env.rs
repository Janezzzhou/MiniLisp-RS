use crate::builtins::builtin_map;
use crate::error::LispError;
use crate::form::lookup_special_form;
use crate::value::{Value, ValuePtr};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type EnvPtr = Rc<RefCell<EvalEnv>>;

#[derive(Debug)]
pub struct EvalEnv {
    symbols: HashMap<String, ValuePtr>,
    parent: Option<EnvPtr>,
}

impl EvalEnv {
    pub fn new() -> EnvPtr {
        Rc::new(RefCell::new(Self {
            symbols: builtin_map(),
            parent: None,
        }))
    }

    pub fn with_parent(parent: EnvPtr) -> EnvPtr {
        Rc::new(RefCell::new(Self {
            symbols: HashMap::new(),
            parent: Some(parent),
        }))
    }

    pub fn lookup_binding(&self, name: &str) -> Option<ValuePtr> {
        if let Some(value) = self.symbols.get(name) {
            return Some(value.clone());
        }

        self.parent
            .as_ref()
            .and_then(|parent| parent.borrow().lookup_binding(name))
    }

    pub fn define_binding(&mut self, name: String, value: ValuePtr) {
        self.symbols.insert(name, value);
    }

    pub fn eval(env: &EnvPtr, expr: ValuePtr) -> Result<ValuePtr, LispError> {
        if expr.is_self_evaluating() {
            return Ok(expr);
        }

        if expr.is_nil() {
            return Err(LispError::RuntimeError(
                "Evaluating nil is prohibited.".into(),
            ));
        }

        if let Some(name) = expr.as_symbol() {
            if let Some(value) = env.borrow().lookup_binding(name) {
                return Ok(value);
            }
            return Err(LispError::RuntimeError(format!(
                "Variable {} not defined.",
                name
            )));
        }

        let v = expr.to_vec()?;
        if v.is_empty() {
            return Err(LispError::RuntimeError("Empty list".into()));
        }

        if let Some(name) = v[0].as_symbol() {
            if let Some(form) = lookup_special_form(name) {
                return form(&v[1..], env);
            }
        }

        let proc = Self::eval(env, v[0].clone())?;
        let args = Self::eval_list(env, &v[1..])?;
        Self::apply_procedure(env, proc, args)
    }

    fn eval_list(env: &EnvPtr, list: &[ValuePtr]) -> Result<Vec<ValuePtr>, LispError> {
        let mut result = Vec::new();
        for expr in list {
            result.push(Self::eval(env, expr.clone())?);
        }
        Ok(result)
    }

    pub fn apply_procedure(
        env: &EnvPtr,
        proc: ValuePtr,
        args: Vec<ValuePtr>,
    ) -> Result<ValuePtr, LispError> {
        if let Some(f) = proc.as_builtin_proc() {
            return f(args, env);
        }

        if let Some((params, body, defining_env)) = proc.as_lambda_proc() {
            if params.len() != args.len() {
                return Err(LispError::RuntimeError(format!(
                    "Expected {} arguments, got {}",
                    params.len(),
                    args.len()
                )));
            }

            let local_env = EvalEnv::with_parent(defining_env);
            {
                let mut local = local_env.borrow_mut();
                for (param, arg) in params.iter().zip(args.into_iter()) {
                    local.define_binding(param.clone(), arg);
                }
            }

            let mut last = ValuePtr::new(Value::Nil);
            for expr in body {
                last = Self::eval(&local_env, expr.clone())?;
            }
            return Ok(last);
        }

        Err(LispError::RuntimeError(
            "Attempted to call a non-procedure value".into(),
        ))
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
        let env = EvalEnv::new();
        let result = EvalEnv::eval(&env, expr)?;
        Ok(result.to_string())
    }

    fn eval_in_env(env: &EnvPtr, input: &str) -> Result<String, LispError> {
        let tokens = tokenize(input).map_err(|e| LispError::SyntaxError(e.to_string()))?;
        let mut parser = Parser::new(tokens);
        let expr = parser.parse()?;
        let result = EvalEnv::eval(env, expr)?;
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
        let env = EvalEnv::new();

        assert_eq!(eval_in_env(&env, "(define x 42)").unwrap(), "()");
        assert_eq!(eval_in_env(&env, "x").unwrap(), "42");

        assert_eq!(eval_in_env(&env, "(define y x)").unwrap(), "()");
        assert_eq!(eval_in_env(&env, "y").unwrap(), "42");

        let err = eval_in_env(&env, "z").unwrap_err();
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

    #[test]
    fn test_builtin_add() {
        assert_eq!(eval_str("(+ 1 2)").unwrap(), "3");
        assert_eq!(eval_str("(+)").unwrap(), "0");
        assert_eq!(eval_str("(+ 1 2 3 4 5)").unwrap(), "15");
        assert_eq!(eval_str("(+ 1.5 2.5)").unwrap(), "4");
    }

    #[test]
    fn test_builtin_sub_and_negative_literal() {
        assert_eq!(eval_str("-2").unwrap(), "-2");
        assert_eq!(eval_str("(- 5 2)").unwrap(), "3");
        assert_eq!(eval_str("(- 2)").unwrap(), "-2");
    }

    #[test]
    fn test_define_with_add() {
        let env = EvalEnv::new();

        assert_eq!(eval_in_env(&env, "(define x (+ 1 2))").unwrap(), "()");
        assert_eq!(eval_in_env(&env, "x").unwrap(), "3");
        assert_eq!(eval_in_env(&env, "(+ x 4)").unwrap(), "7");
    }

    #[test]
    fn test_define_alias() {
        let env = EvalEnv::new();

        assert_eq!(eval_in_env(&env, "(define add +)").unwrap(), "()");
        assert_eq!(eval_in_env(&env, "(add 1 2 3)").unwrap(), "6");
        assert_eq!(eval_in_env(&env, "(+ 10 20)").unwrap(), "30");
    }

    #[test]
    fn test_lambda_literal_application() {
        assert_eq!(eval_str("((lambda (x y) (+ x y)) 3 4)").unwrap(), "7");
    }

    #[test]
    fn test_define_function_sugar() {
        let env = EvalEnv::new();

        assert_eq!(eval_in_env(&env, "(define (add2 x y) (+ x y))").unwrap(), "()");
        assert_eq!(eval_in_env(&env, "(add2 10 32)").unwrap(), "42");
    }

    #[test]
    fn test_define_function_with_multiple_body_expressions() {
        let env = EvalEnv::new();

        assert_eq!(
            eval_in_env(&env, "(define (show-and-return x) (print x) (+ x 1))").unwrap(),
            "()"
        );
        assert_eq!(eval_in_env(&env, "(show-and-return 4)").unwrap(), "5");
    }

    #[test]
    fn test_false_binding_with_if_and_short_circuit() {
        let env = EvalEnv::new();

        assert_eq!(eval_in_env(&env, "(define false #f)").unwrap(), "()");
        assert_eq!(eval_in_env(&env, "(if false \"OK\" \"Emm\")").unwrap(), "\"Emm\"");
        assert_eq!(
            eval_in_env(&env, "(and false (print \"Don't print\"))").unwrap(),
            "#f"
        );
    }

    #[test]
    fn test_empty_or_returns_false() {
        assert_eq!(eval_str("(or)").unwrap(), "#f");
    }

    #[test]
    fn test_lambda_evaluates_to_procedure() {
        assert_eq!(eval_str("(lambda (x) (+ x x))").unwrap(), "#<procedure>");
    }

    #[test]
    fn test_defined_function_evaluates_to_procedure_and_applies() {
        let env = EvalEnv::new();

        assert_eq!(eval_in_env(&env, "(define (double x) (+ x x))").unwrap(), "()");
        assert_eq!(eval_in_env(&env, "double").unwrap(), "#<procedure>");
        assert_eq!(eval_in_env(&env, "(double 3.14)").unwrap(), "6.28");
    }

    #[test]
    fn test_builtin_gt() {
        assert_eq!(eval_str("(if (> 3 2) \"Correct\" \"Bad\")").unwrap(), "\"Correct\"");
        assert_eq!(eval_str("(> 2 3)").unwrap(), "#f");
    }

    #[test]
    fn test_builtin_length() {
        assert_eq!(eval_str("(length '(1 2 3 4))").unwrap(), "4");
        assert_eq!(eval_str("(length '())").unwrap(), "0");
    }

    #[test]
    fn test_builtin_cdr() {
        assert_eq!(eval_str("(cdr '(1 . 2))").unwrap(), "2");
        assert_eq!(eval_str("(cdr '(1 2 3))").unwrap(), "(2 3)");
    }

    #[test]
    fn test_list_construction_builtins() {
        assert_eq!(
            eval_str("(append '(1 2 3) '(a b c) '(foo bar baz))").unwrap(),
            "(1 2 3 a b c foo bar baz)"
        );
        assert_eq!(eval_str("(append)").unwrap(), "()");
        assert_eq!(eval_str("(car '(1 . 2))").unwrap(), "1");
        assert_eq!(eval_str("(cons 1 '(2 3))").unwrap(), "(1 2 3)");
        assert_eq!(eval_str("(list 1 2 3)").unwrap(), "(1 2 3)");
    }

    #[test]
    fn test_map_filter_reduce() {
        assert_eq!(eval_str("(map - '(1 2 3))").unwrap(), "(-1 -2 -3)");
        assert_eq!(
            eval_str("(filter (lambda (x) (> x 2)) '(1 2 3 4))").unwrap(),
            "(3 4)"
        );
        assert_eq!(eval_str("(reduce * '(1 2 3 4))").unwrap(), "24");
    }

    #[test]
    fn test_comparison_and_numeric_predicates() {
        let env = EvalEnv::new();

        assert_eq!(eval_str("(not #f)").unwrap(), "#t");
        assert_eq!(eval_str("(not 1)").unwrap(), "#f");

        assert_eq!(eval_str("(= 2 2)").unwrap(), "#t");
        assert_eq!(eval_str("(< 1 2)").unwrap(), "#t");
        assert_eq!(eval_str("(<= 2 2)").unwrap(), "#t");
        assert_eq!(eval_str("(> 3 2)").unwrap(), "#t");
        assert_eq!(eval_str("(>= 3 3)").unwrap(), "#t");

        assert_eq!(eval_str("(even? 4)").unwrap(), "#t");
        assert_eq!(eval_str("(even? 3)").unwrap(), "#f");
        assert_eq!(eval_str("(odd? 3)").unwrap(), "#t");
        assert_eq!(eval_str("(odd? 4)").unwrap(), "#f");
        assert_eq!(eval_str("(zero? 0)").unwrap(), "#t");
        assert_eq!(eval_str("(zero? -0)").unwrap(), "#t");

        assert_eq!(eval_str("(eq? '(1 2 3) '(1 2 3))").unwrap(), "#f");
        assert_eq!(eval_in_env(&env, "(define x '(1 2 3))").unwrap(), "()");
        assert_eq!(eval_in_env(&env, "(eq? x x)").unwrap(), "#t");
        assert_eq!(eval_str("(equal? '(1 2 3) '(1 2 3))").unwrap(), "#t");
    }

    #[test]
    fn test_numeric_library() {
        assert_eq!(eval_str("(/ 4)").unwrap(), "0.25");
        assert_eq!(eval_str("(/ 7 2)").unwrap(), "3.5");
        assert_eq!(eval_str("(abs -3)").unwrap(), "3");
        assert_eq!(eval_str("(expt 2 3)").unwrap(), "8");
        assert_eq!(eval_str("(quotient 7 2)").unwrap(), "3");
        assert_eq!(eval_str("(quotient -7 2)").unwrap(), "-3");

        assert_eq!(eval_str("(modulo 10 3)").unwrap(), "1");
        assert_eq!(eval_str("(modulo -10 3)").unwrap(), "2");
        assert_eq!(eval_str("(modulo 10 -3)").unwrap(), "-2");
        assert_eq!(eval_str("(modulo -10 -3)").unwrap(), "-1");

        assert_eq!(eval_str("(remainder 10 3)").unwrap(), "1");
        assert_eq!(eval_str("(remainder -10 3)").unwrap(), "-1");
        assert_eq!(eval_str("(remainder 10 -3)").unwrap(), "1");
        assert_eq!(eval_str("(remainder -10 -3)").unwrap(), "-1");
    }

    #[test]
    fn test_type_predicates() {
        assert_eq!(eval_str("(atom? #t)").unwrap(), "#t");
        assert_eq!(eval_str("(atom? 'abc)").unwrap(), "#t");
        assert_eq!(eval_str("(atom? '())").unwrap(), "#t");
        assert_eq!(eval_str("(atom? '(1 . 2))").unwrap(), "#f");

        assert_eq!(eval_str("(boolean? #f)").unwrap(), "#t");
        assert_eq!(eval_str("(boolean? 0)").unwrap(), "#f");

        assert_eq!(eval_str("(integer? 3)").unwrap(), "#t");
        assert_eq!(eval_str("(integer? 3.5)").unwrap(), "#f");

        assert_eq!(eval_str("(list? '())").unwrap(), "#t");
        assert_eq!(eval_str("(list? '(1 2 3))").unwrap(), "#t");
        assert_eq!(eval_str("(list? '(1 . 2))").unwrap(), "#f");

        assert_eq!(eval_str("(number? 3.14)").unwrap(), "#t");
        assert_eq!(eval_str("(number? 'abc)").unwrap(), "#f");

        assert_eq!(eval_str("(null? '())").unwrap(), "#t");
        assert_eq!(eval_str("(null? '(1))").unwrap(), "#f");

        assert_eq!(eval_str("(pair? '(1 . 2))").unwrap(), "#t");
        assert_eq!(eval_str("(pair? '())").unwrap(), "#f");

        assert_eq!(eval_str("(procedure? +)").unwrap(), "#t");
        assert_eq!(eval_str("(procedure? (lambda (x) x))").unwrap(), "#t");
        assert_eq!(eval_str("(procedure? 1)").unwrap(), "#f");

        assert_eq!(eval_str("(string? \"hello\")").unwrap(), "#t");
        assert_eq!(eval_str("(string? 'hello)").unwrap(), "#f");

        assert_eq!(eval_str("(symbol? 'hello)").unwrap(), "#t");
        assert_eq!(eval_str("(symbol? \"hello\")").unwrap(), "#f");
    }

    #[test]
    fn test_builtin_apply() {
        assert_eq!(eval_str("(apply + '(1 2 3))").unwrap(), "6");
        assert_eq!(eval_str("(apply - '(10 3 2))").unwrap(), "5");
    }

    #[test]
    fn test_builtin_eval() {
        assert_eq!(eval_str("(eval '(+ 1 2 3))").unwrap(), "6");
    }

    #[test]
    fn test_error_builtin_signals_runtime_error() {
        let err = eval_str("(error \"boom\")").unwrap_err();
        match err {
            LispError::RuntimeError(msg) => assert_eq!(msg, "boom"),
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_exit_builtin_signals_exit() {
        let err = eval_str("(exit 7)").unwrap_err();
        match err {
            LispError::Exit(code) => assert_eq!(code, 7),
            _ => panic!("Expected Exit"),
        }
    }

    #[test]
    fn test_display_family_returns_nil() {
        assert_eq!(eval_str("(display \"hi\")").unwrap(), "()");
        assert_eq!(eval_str("(displayln \"hi\")").unwrap(), "()");
        assert_eq!(eval_str("(newline)").unwrap(), "()");
    }

    #[test]
    fn test_nested_add() {
        let env = EvalEnv::new();

        assert_eq!(eval_in_env(&env, "(define add +)").unwrap(), "()");
        assert_eq!(eval_in_env(&env, "(+ 1 (add 2))").unwrap(), "3");
        assert_eq!(eval_in_env(&env, "(+ (add 1 2) (add 3 4))").unwrap(), "10");
    }

    #[test]
    fn test_builtin_print() {
        assert_eq!(eval_str("(print 42)").unwrap(), "()");
        assert_eq!(eval_str("(print 1 2 3)").unwrap(), "()");
        assert_eq!(eval_str("(print (+ 1 2))").unwrap(), "()");
    }

    #[test]
    fn test_if_with_quoted_nil_is_truthy() {
        assert_eq!(eval_str("(if '() (print \"Yea\") (print \"Nay\"))").unwrap(), "()");
    }

    #[test]
    fn test_if_false_branch() {
        assert_eq!(eval_str("(if #f (print \"Yea\") (print \"Nay\"))").unwrap(), "()");
    }

    #[test]
    fn test_and_short_circuit() {
        assert_eq!(eval_str("(and (print 1) (print 2) #f (print 3))").unwrap(), "#f");
    }

    #[test]
    fn test_or_short_circuit() {
        assert_eq!(eval_str("(or #f #f (print 1) (print 3))").unwrap(), "()");
    }

    #[test]
    fn test_and_skips_later_erroring_expression() {
        assert_eq!(eval_str("(and #f unknown-symbol)").unwrap(), "#f");
    }

    #[test]
    fn test_or_skips_later_erroring_expression() {
        assert_eq!(eval_str("(or 1 unknown-symbol)").unwrap(), "1");
    }

    #[test]
    fn test_unknown_procedure() {
        let err = eval_str("(unknown 1 2)").unwrap_err();
        match err {
            LispError::RuntimeError(msg) => {
                assert!(
                    msg.contains("not defined")
                        || msg.contains("Variable")
                        || msg.contains("Unimplemented")
                );
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_env_persistence() {
        let env = EvalEnv::new();
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
        EvalEnv::eval(&env, expr1).unwrap();
        let expr2 = ValuePtr::new(Value::Symbol("x".to_string()));
        let result = EvalEnv::eval(&env, expr2).unwrap();
        assert_eq!(result.to_string(), "100");
    }

    #[test]
    fn test_lookup_binding_walks_parent_chain() {
        let parent = EvalEnv::new();
        parent
            .borrow_mut()
            .define_binding("x".to_string(), ValuePtr::new(Value::Numeric(7.0)));

        let child = EvalEnv::with_parent(parent);
        let value = child.borrow().lookup_binding("x").unwrap();
        assert_eq!(value.to_string(), "7");
    }

    #[test]
    fn test_lambda_captures_defining_environment() {
        let env = EvalEnv::new();

        assert_eq!(
            eval_in_env(&env, "(define make-adder (lambda (x) (lambda (y) (+ x y))))").unwrap(),
            "()"
        );
        assert_eq!(eval_in_env(&env, "(define add-ten (make-adder 10))").unwrap(), "()");
        assert_eq!(eval_in_env(&env, "(add-ten 5)").unwrap(), "15");
    }

    #[test]
    fn test_a_plus_abs_b_uses_sub_builtin() {
        let env = EvalEnv::new();

        assert_eq!(
            eval_in_env(
                &env,
                "(define (a-plus-abs-b a b) ((if (> b 0) + -) a b))"
            )
            .unwrap(),
            "()"
        );
        assert_eq!(eval_in_env(&env, "(a-plus-abs-b 3 -2)").unwrap(), "5");
    }

    #[test]
    fn test_begin_cond_let_and_quasiquote() {
        let env = EvalEnv::new();

        assert_eq!(eval_str("(begin 1 2 3)").unwrap(), "3");

        assert_eq!(eval_in_env(&env, "(define n -5)").unwrap(), "()");
        assert_eq!(
            eval_in_env(&env, "(cond ((< n 0) -1) ((> n 0) 1) (else 0))").unwrap(),
            "-1"
        );

        assert_eq!(
            eval_str("(let ((x 5) (y 10)) (print x) (print y) (+ x y))").unwrap(),
            "15"
        );

        assert_eq!(eval_str("`(11 45 ,(* 2 7))").unwrap(), "(11 45 14)");
    }
}
