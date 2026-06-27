use crate::eval_env::{EnvPtr, EvalEnv};
use crate::error::LispError;
use crate::value::*;

pub type SpecialFormFunc = fn(&[ValuePtr], &EnvPtr) -> Result<ValuePtr, LispError>;

pub fn lookup_special_form(name: &str) -> Option<SpecialFormFunc> {
    match name {
        "begin" => Some(begin_form),
        "cond" => Some(cond_form),
        "define" => Some(define_form),
        "lambda" => Some(lambda_form),
        "let" => Some(let_form),
        "quasiquote" => Some(quasiquote_form),
        "quote" => Some(quote_form),
        "if" => Some(if_form),
        "and" => Some(and_form),
        "or" => Some(or_form),
        _ => None,
    }
}

fn eval_sequence(exprs: &[ValuePtr], env: &EnvPtr) -> Result<ValuePtr, LispError> {
    let mut last = ValuePtr::new(Value::Nil);
    for expr in exprs {
        last = EvalEnv::eval(env, expr.clone())?;
    }
    Ok(last)
}

fn make_list(values: Vec<ValuePtr>) -> ValuePtr {
    values
        .into_iter()
        .rev()
        .fold(ValuePtr::new(Value::Nil), |cdr, car| ValuePtr::new(Value::Pair(car, cdr)))
}

fn quasiquote_expand(expr: &ValuePtr, env: &EnvPtr) -> Result<ValuePtr, LispError> {
    match expr.as_ref() {
        Value::Pair(car, cdr) => {
            if let Some("unquote") = car.as_symbol() {
                let items = expr.to_vec()?;
                if items.len() != 2 {
                    return Err(LispError::RuntimeError("unquote requires 1 argument".into()));
                }
                return EvalEnv::eval(env, items[1].clone());
            }

            let new_car = quasiquote_expand(car, env)?;
            let new_cdr = quasiquote_expand(cdr, env)?;
            Ok(ValuePtr::new(Value::Pair(new_car, new_cdr)))
        }
        _ => Ok(expr.clone()),
    }
}

pub fn begin_form(args: &[ValuePtr], env: &EnvPtr) -> Result<ValuePtr, LispError> {
    eval_sequence(args, env)
}

pub fn cond_form(args: &[ValuePtr], env: &EnvPtr) -> Result<ValuePtr, LispError> {
    for (index, clause) in args.iter().enumerate() {
        let items = clause.to_vec().map_err(|_| {
            LispError::RuntimeError("cond clause must be a proper list".into())
        })?;
        if items.is_empty() {
            return Err(LispError::RuntimeError("cond clause cannot be empty".into()));
        }

        if let Some("else") = items[0].as_symbol() {
            if index != args.len() - 1 {
                return Err(LispError::RuntimeError("else must be the last cond clause".into()));
            }
            return eval_sequence(&items[1..], env);
        }

        let cond_value = EvalEnv::eval(env, items[0].clone())?;
        if cond_value.is_false() {
            continue;
        }

        if items.len() == 1 {
            return Ok(cond_value);
        }

        return eval_sequence(&items[1..], env);
    }

    Ok(ValuePtr::new(Value::Nil))
}

pub fn define_form(args: &[ValuePtr], env: &EnvPtr) -> Result<ValuePtr, LispError> {
    if args.len() < 2 {
        return Err(LispError::RuntimeError(
            "define requires at least 2 arguments".into(),
        ));
    }

    if let Some(name) = args[0].as_symbol() {
        if args.len() != 2 {
            return Err(LispError::RuntimeError(
                "define variable form requires 2 arguments".into(),
            ));
        }

        let value = EvalEnv::eval(env, args[1].clone())?;
        env.borrow_mut().define_binding(name.to_string(), value);
        return Ok(ValuePtr::new(Value::Nil));
    }

    let signature = args[0].to_vec().map_err(|_| {
        LispError::RuntimeError("define function signature must be a proper list".into())
    })?;

    if signature.is_empty() {
        return Err(LispError::RuntimeError(
            "define function signature cannot be empty".into(),
        ));
    }

    let name = signature[0]
        .as_symbol()
        .ok_or_else(|| LispError::RuntimeError("define function name must be symbol".into()))?;

    let params = signature[1..]
        .iter()
        .map(|param| {
            param
                .as_symbol()
                .map(str::to_string)
                .ok_or_else(|| LispError::RuntimeError("lambda parameters must be symbols".into()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let body = args[1..].to_vec();
    if body.is_empty() {
        return Err(LispError::RuntimeError(
            "define function body cannot be empty".into(),
        ));
    }

    env.borrow_mut().define_binding(
        name.to_string(),
        ValuePtr::new(Value::LambdaProc {
            params,
            body,
            env: env.clone(),
        }),
    );
    Ok(ValuePtr::new(Value::Nil))
}

pub fn lambda_form(args: &[ValuePtr], env: &EnvPtr) -> Result<ValuePtr, LispError> {
    if args.len() < 2 {
        return Err(LispError::RuntimeError(
            "lambda requires parameters and body".into(),
        ));
    }

    let params = args[0]
        .to_vec()
        .map_err(|_| LispError::RuntimeError("lambda parameter list must be a proper list".into()))?
        .into_iter()
        .map(|param| {
            param
                .as_symbol()
                .map(str::to_string)
                .ok_or_else(|| LispError::RuntimeError("lambda parameters must be symbols".into()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let body = args[1..].to_vec();
    Ok(ValuePtr::new(Value::LambdaProc {
        params,
        body,
        env: env.clone(),
    }))
}

pub fn let_form(args: &[ValuePtr], env: &EnvPtr) -> Result<ValuePtr, LispError> {
    if args.len() < 2 {
        return Err(LispError::RuntimeError("let requires bindings and body".into()));
    }

    let bindings = args[0]
        .to_vec()
        .map_err(|_| LispError::RuntimeError("let bindings must be a proper list".into()))?;

    let mut params = Vec::new();
    let mut values = Vec::new();
    for binding in bindings {
        let pair = binding
            .to_vec()
            .map_err(|_| LispError::RuntimeError("let binding must be a proper list".into()))?;
        if pair.len() != 2 {
            return Err(LispError::RuntimeError("let binding requires name and value".into()));
        }
        let name = pair[0]
            .as_symbol()
            .ok_or_else(|| LispError::RuntimeError("let binding name must be symbol".into()))?;
        params.push(ValuePtr::new(Value::Symbol(name.to_string())));
        values.push(pair[1].clone());
    }

    let lambda_expr = ValuePtr::new(Value::Pair(
        ValuePtr::new(Value::Symbol("lambda".into())),
        ValuePtr::new(Value::Pair(
            make_list(params),
            make_list(args[1..].to_vec()),
        )),
    ));

    let call_expr = ValuePtr::new(Value::Pair(lambda_expr, make_list(values)));
    EvalEnv::eval(env, call_expr)
}

pub fn quasiquote_form(args: &[ValuePtr], env: &EnvPtr) -> Result<ValuePtr, LispError> {
    if args.len() != 1 {
        return Err(LispError::RuntimeError("quasiquote requires 1 argument".into()));
    }
    quasiquote_expand(&args[0], env)
}

pub fn quote_form(args: &[ValuePtr], _: &EnvPtr) -> Result<ValuePtr, LispError> {
    if args.len() != 1 {
        return Err(LispError::RuntimeError("quote requires 1 argument".into()));
    }
    Ok(args[0].clone())
}

pub fn if_form(args: &[ValuePtr], env: &EnvPtr) -> Result<ValuePtr, LispError> {
    if args.len() != 3 {
        return Err(LispError::RuntimeError("if requires 3 arguments".into()));
    }

    let cond = EvalEnv::eval(env, args[0].clone())?;
    if cond.is_false() {
        EvalEnv::eval(env, args[2].clone())
    } else {
        EvalEnv::eval(env, args[1].clone())
    }
}

pub fn and_form(args: &[ValuePtr], env: &EnvPtr) -> Result<ValuePtr, LispError> {
    if args.is_empty() {
        return Ok(ValuePtr::new(Value::Boolean(true)));
    }

    let mut last = ValuePtr::new(Value::Boolean(true));
    for expr in args {
        last = EvalEnv::eval(env, expr.clone())?;
        if last.is_false() {
            return Ok(last);
        }
    }
    Ok(last)
}

pub fn or_form(args: &[ValuePtr], env: &EnvPtr) -> Result<ValuePtr, LispError> {
    for expr in args {
        let value = EvalEnv::eval(env, expr.clone())?;
        if value.is_true() {
            return Ok(value);
        }
    }

    Ok(ValuePtr::new(Value::Boolean(false)))
}
