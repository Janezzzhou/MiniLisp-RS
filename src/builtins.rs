use crate::error::*;
use crate::value::*;
use std::collections::HashMap;

pub fn builtin_map() -> HashMap<String, ValuePtr> {
    let mut map = HashMap::new();

    map.insert(
        "+".into(),
        ValuePtr::new(Value::BuiltinProc(add)),
    );

    map.insert(
        "-".into(),
        ValuePtr::new(Value::BuiltinProc(sub)),
    );

    map.insert(
        "*".into(),
        ValuePtr::new(Value::BuiltinProc(mul)),
    );

    map.insert(
        ">".into(),
        ValuePtr::new(Value::BuiltinProc(gt)),
    );

    map.insert(
        "length".into(),
        ValuePtr::new(Value::BuiltinProc(length)),
    );

    map.insert(
        "cdr".into(),
        ValuePtr::new(Value::BuiltinProc(cdr)),
    );

    map.insert(
        "print".into(),
        ValuePtr::new(Value::BuiltinProc(print_proc)),
    );

    map
}

pub fn add(args: Vec<ValuePtr>,) -> Result<ValuePtr, LispError> {
    let mut sum = 0.0;
    for arg in args {
        sum += arg
            .as_number()
            .ok_or_else(|| {LispError::RuntimeError("Cannot add non-number".into())})?;
    }
    Ok(ValuePtr::new(Value::Numeric(sum),))
}

pub fn sub(args: Vec<ValuePtr>,) -> Result<ValuePtr, LispError> {
    if args.is_empty() {
        return Err(LispError::RuntimeError("- requires at least 1 argument".into()));
    }

    let first = args[0]
        .as_number()
        .ok_or_else(|| LispError::RuntimeError("Cannot sub non-number".into()))?;

    if args.len() == 1 {
        return Ok(ValuePtr::new(Value::Numeric(-first)));
    }

    let mut result = first;
    for arg in &args[1..] {
        result -= arg
            .as_number()
            .ok_or_else(|| LispError::RuntimeError("Cannot sub non-number".into()))?;
    }
    Ok(ValuePtr::new(Value::Numeric(result)))
}

pub fn mul(args: Vec<ValuePtr>,) -> Result<ValuePtr, LispError> {
    let mut product = 1.0;
    for arg in args {
        product *= arg
            .as_number()
            .ok_or_else(|| {LispError::RuntimeError("Cannot mul non-number".into())})?;
    }
    Ok(ValuePtr::new(Value::Numeric(product),))
}

pub fn gt(args: Vec<ValuePtr>,) -> Result<ValuePtr, LispError> {
    if args.len() != 2 {
        return Err(LispError::RuntimeError("> requires 2 arguments".into()));
    }

    let left = args[0]
        .as_number()
        .ok_or_else(|| LispError::RuntimeError("> expects numbers".into()))?;
    let right = args[1]
        .as_number()
        .ok_or_else(|| LispError::RuntimeError("> expects numbers".into()))?;

    Ok(ValuePtr::new(Value::Boolean(left > right)))
}

pub fn length(args: Vec<ValuePtr>,) -> Result<ValuePtr, LispError> {
    if args.len() != 1 {
        return Err(LispError::RuntimeError("length requires 1 argument".into()));
    }

    let count = args[0].to_vec()?.len() as f64;
    Ok(ValuePtr::new(Value::Numeric(count)))
}

pub fn cdr(args: Vec<ValuePtr>,) -> Result<ValuePtr, LispError> {
    if args.len() != 1 {
        return Err(LispError::RuntimeError("cdr requires 1 argument".into()));
    }

    match args[0].as_ref() {
        Value::Pair(_, cdr) => Ok(cdr.clone()),
        _ => Err(LispError::RuntimeError("cdr expects a pair".into())),
    }
}

pub fn print_proc(args: Vec<ValuePtr>,) -> Result<ValuePtr, LispError> {
    for v in args {
        println!("{}", v);
    }
    Ok(ValuePtr::new(Value::Nil))
}
