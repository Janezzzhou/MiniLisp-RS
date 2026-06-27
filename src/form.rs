use crate::eval_env::{EnvPtr, EvalEnv};
use crate::error::LispError;
use crate::value::*;

pub type SpecialFormFunc = fn(&[ValuePtr], &EnvPtr) -> Result<ValuePtr, LispError>;

pub fn lookup_special_form(name: &str) -> Option<SpecialFormFunc> {
    match name {
        "define" => Some(define_form),
        "lambda" => Some(lambda_form),
        "quote" => Some(quote_form),
        "if" => Some(if_form),
        "and" => Some(and_form),
        "or" => Some(or_form),
        _ => None,
    }
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
