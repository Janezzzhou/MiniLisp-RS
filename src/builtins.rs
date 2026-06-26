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

pub fn print_proc(args: Vec<ValuePtr>,) -> Result<ValuePtr, LispError> {
    for v in args {
        println!("{}", v);
    }
    Ok(ValuePtr::new(Value::Nil))
}