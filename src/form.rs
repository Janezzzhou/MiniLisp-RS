use crate::eval_env::EvalEnv;
use crate::error::LispError;
use crate::value::*;

pub type SpecialFormFunc = fn(&[ValuePtr], &mut EvalEnv) -> Result<ValuePtr, LispError>;

pub fn lookup_special_form(name: &str,) -> Option<SpecialFormFunc> {
    match name {
        "define" => Some(define_form),
        "quote" => Some(quote_form),
        "if" => Some(if_form),
        "and" => Some(and_form),
        "or" => Some(or_form),
        _ => None,
    }
}

pub fn define_form(args: &[ValuePtr], env: &mut EvalEnv,) -> Result<ValuePtr, LispError> {
    if args.len() != 2 {
        return Err(LispError::RuntimeError("define requires 2 arguments".into()));
    }

    let name = args[0]
        .as_symbol()
        .ok_or_else(|| {
            LispError::RuntimeError("define first arg must be symbol".into())
        })?;

    let value = env.eval(args[1].clone())?;
    env.define(name.to_string(), value);
    Ok(ValuePtr::new(Value::Nil))
}

pub fn quote_form(args: &[ValuePtr], _: &mut EvalEnv,) -> Result<ValuePtr, LispError> {
    if args.len() != 1 {
        return Err(LispError::RuntimeError("quote requires 1 argument".into()));
    }
    Ok(args[0].clone())
}

pub fn if_form(args: &[ValuePtr], env: &mut EvalEnv,) -> Result<ValuePtr, LispError> {
    if args.len() != 3 {
        return Err(LispError::RuntimeError("if requires 3 arguments".into()));
    }

    let cond = env.eval(args[0].clone())?;
    if cond.is_false() {
        env.eval(args[2].clone())
    } 
    else {
        env.eval(args[1].clone())
    }
}

pub fn and_form(args: &[ValuePtr], env: &mut EvalEnv,) -> Result<ValuePtr, LispError> {
    if args.is_empty() {
        return Ok(ValuePtr::new(Value::Boolean(true)));
    }

    let mut last =ValuePtr::new(Value::Boolean(true));
    for expr in args {
        last = env.eval(expr.clone())?;
        if last.is_false() {
            return Ok(last);
        }
    }
    Ok(last)
}

pub fn or_form(args: &[ValuePtr], env: &mut EvalEnv,) -> Result<ValuePtr, LispError> {
    for expr in args {
        let value = env.eval(expr.clone())?;
        if value.is_true() {
            return Ok(value);
        }
    }

    Ok(ValuePtr::new(Value::Boolean(false)))
}