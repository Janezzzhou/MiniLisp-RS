use crate::error::*;
use crate::eval_env::EnvPtr;
use crate::eval_env::EvalEnv;
use crate::value::*;
use std::collections::HashMap;

pub fn builtin_map() -> HashMap<String, ValuePtr> {
    let mut map = HashMap::new();

    map.insert(
        "atom?".into(),
        ValuePtr::new(Value::BuiltinProc(atom_pred)),
    );

    map.insert(
        "boolean?".into(),
        ValuePtr::new(Value::BuiltinProc(boolean_pred)),
    );

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
        "apply".into(),
        ValuePtr::new(Value::BuiltinProc(apply_proc)),
    );

    map.insert(
        "display".into(),
        ValuePtr::new(Value::BuiltinProc(display_proc)),
    );

    map.insert(
        "displayln".into(),
        ValuePtr::new(Value::BuiltinProc(displayln_proc)),
    );

    map.insert(
        "error".into(),
        ValuePtr::new(Value::BuiltinProc(error_proc)),
    );

    map.insert(
        "eval".into(),
        ValuePtr::new(Value::BuiltinProc(eval_proc)),
    );

    map.insert(
        "exit".into(),
        ValuePtr::new(Value::BuiltinProc(exit_proc)),
    );

    map.insert(
        "integer?".into(),
        ValuePtr::new(Value::BuiltinProc(integer_pred)),
    );

    map.insert(
        "list?".into(),
        ValuePtr::new(Value::BuiltinProc(list_pred)),
    );

    map.insert(
        "null?".into(),
        ValuePtr::new(Value::BuiltinProc(null_pred)),
    );

    map.insert(
        "newline".into(),
        ValuePtr::new(Value::BuiltinProc(newline_proc)),
    );

    map.insert(
        "number?".into(),
        ValuePtr::new(Value::BuiltinProc(number_pred)),
    );

    map.insert(
        "pair?".into(),
        ValuePtr::new(Value::BuiltinProc(pair_pred)),
    );

    map.insert(
        "print".into(),
        ValuePtr::new(Value::BuiltinProc(print_proc)),
    );

    map.insert(
        "procedure?".into(),
        ValuePtr::new(Value::BuiltinProc(procedure_pred)),
    );

    map.insert(
        "string?".into(),
        ValuePtr::new(Value::BuiltinProc(string_pred)),
    );

    map.insert(
        "symbol?".into(),
        ValuePtr::new(Value::BuiltinProc(symbol_pred)),
    );

    map
}

fn expect_one_arg<'a>(name: &str, args: &'a [ValuePtr]) -> Result<&'a ValuePtr, LispError> {
    match args {
        [value] => Ok(value),
        _ => Err(LispError::RuntimeError(format!("{} requires 1 argument", name))),
    }
}

fn bool_value(value: bool) -> ValuePtr {
    ValuePtr::new(Value::Boolean(value))
}

fn is_list_value(value: &Value) -> bool {
    match value {
        Value::Nil => true,
        Value::Pair(_, cdr) => is_list_value(cdr.as_ref()),
        _ => false,
    }
}

pub fn atom_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("atom?", &args)?;
    Ok(bool_value(matches!(
        value.as_ref(),
        Value::Boolean(_) | Value::Numeric(_) | Value::String(_) | Value::Symbol(_) | Value::Nil
    )))
}

pub fn boolean_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("boolean?", &args)?;
    Ok(bool_value(matches!(value.as_ref(), Value::Boolean(_))))
}

pub fn add(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let mut sum = 0.0;
    for arg in args {
        sum += arg
            .as_number()
            .ok_or_else(|| {LispError::RuntimeError("Cannot add non-number".into())})?;
    }
    Ok(ValuePtr::new(Value::Numeric(sum),))
}

pub fn sub(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
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

pub fn mul(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let mut product = 1.0;
    for arg in args {
        product *= arg
            .as_number()
            .ok_or_else(|| {LispError::RuntimeError("Cannot mul non-number".into())})?;
    }
    Ok(ValuePtr::new(Value::Numeric(product),))
}

pub fn gt(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
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

pub fn integer_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("integer?", &args)?;
    let is_integer = match value.as_ref() {
        Value::Numeric(n) => n.fract() == 0.0,
        _ => false,
    };
    Ok(bool_value(is_integer))
}

pub fn list_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("list?", &args)?;
    Ok(bool_value(is_list_value(value.as_ref())))
}

pub fn length(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    if args.len() != 1 {
        return Err(LispError::RuntimeError("length requires 1 argument".into()));
    }

    let count = args[0].to_vec()?.len() as f64;
    Ok(ValuePtr::new(Value::Numeric(count)))
}

pub fn cdr(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    if args.len() != 1 {
        return Err(LispError::RuntimeError("cdr requires 1 argument".into()));
    }

    match args[0].as_ref() {
        Value::Pair(_, cdr) => Ok(cdr.clone()),
        _ => Err(LispError::RuntimeError("cdr expects a pair".into())),
    }
}

pub fn null_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("null?", &args)?;
    Ok(bool_value(matches!(value.as_ref(), Value::Nil)))
}

pub fn number_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("number?", &args)?;
    Ok(bool_value(matches!(value.as_ref(), Value::Numeric(_))))
}

pub fn pair_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("pair?", &args)?;
    Ok(bool_value(matches!(value.as_ref(), Value::Pair(_, _))))
}

pub fn apply_proc(args: Vec<ValuePtr>, env: &EnvPtr) -> Result<ValuePtr, LispError> {
    if args.len() != 2 {
        return Err(LispError::RuntimeError("apply requires 2 arguments".into()));
    }

    let proc = args[0].clone();
    let list_args = args[1].to_vec()?;
    EvalEnv::apply_procedure(env, proc, list_args)
}

pub fn display_proc(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    if args.len() != 1 {
        return Err(LispError::RuntimeError("display requires 1 argument".into()));
    }

    match args[0].as_ref() {
        Value::String(s) => print!("{}", s),
        other => print!("{}", other),
    }
    Ok(ValuePtr::new(Value::Nil))
}

pub fn displayln_proc(args: Vec<ValuePtr>, env: &EnvPtr) -> Result<ValuePtr, LispError> {
    if args.len() != 1 {
        return Err(LispError::RuntimeError("displayln requires 1 argument".into()));
    }

    display_proc(args, env)?;
    newline_proc(Vec::new(), env)
}

pub fn error_proc(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let message = match args.as_slice() {
        [] => "error".to_string(),
        [value] => match value.as_ref() {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        },
        _ => return Err(LispError::RuntimeError("error accepts at most 1 argument".into())),
    };

    Err(LispError::RuntimeError(message))
}

pub fn eval_proc(args: Vec<ValuePtr>, env: &EnvPtr) -> Result<ValuePtr, LispError> {
    if args.len() != 1 {
        return Err(LispError::RuntimeError("eval requires 1 argument".into()));
    }

    EvalEnv::eval(env, args[0].clone())
}

pub fn exit_proc(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let code = match args.as_slice() {
        [] => 0,
        [value] => {
            let number = value
                .as_number()
                .ok_or_else(|| LispError::RuntimeError("exit expects a number".into()))?;
            number as i32
        }
        _ => return Err(LispError::RuntimeError("exit accepts at most 1 argument".into())),
    };

    Err(LispError::Exit(code))
}

pub fn newline_proc(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    if !args.is_empty() {
        return Err(LispError::RuntimeError("newline requires 0 arguments".into()));
    }

    println!();
    Ok(ValuePtr::new(Value::Nil))
}

pub fn procedure_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("procedure?", &args)?;
    Ok(bool_value(matches!(
        value.as_ref(),
        Value::BuiltinProc(_) | Value::LambdaProc { .. }
    )))
}

pub fn print_proc(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    for v in args {
        println!("{}", v);
    }
    Ok(ValuePtr::new(Value::Nil))
}

pub fn string_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("string?", &args)?;
    Ok(bool_value(matches!(value.as_ref(), Value::String(_))))
}

pub fn symbol_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("symbol?", &args)?;
    Ok(bool_value(matches!(value.as_ref(), Value::Symbol(_))))
}
