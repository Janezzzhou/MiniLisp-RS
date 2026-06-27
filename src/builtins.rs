use crate::error::*;
use crate::eval_env::EnvPtr;
use crate::eval_env::EvalEnv;
use crate::output;
use crate::value::*;
use std::collections::HashMap;

pub fn builtin_map() -> HashMap<String, ValuePtr> {
    let mut map = HashMap::new();

    map.insert("append".into(), ValuePtr::new(Value::BuiltinProc(append_proc)));
    map.insert("abs".into(), ValuePtr::new(Value::BuiltinProc(abs_proc)));
    map.insert("apply".into(), ValuePtr::new(Value::BuiltinProc(apply_proc)));
    map.insert("atom?".into(), ValuePtr::new(Value::BuiltinProc(atom_pred)));
    map.insert("boolean?".into(), ValuePtr::new(Value::BuiltinProc(boolean_pred)));
    map.insert("car".into(), ValuePtr::new(Value::BuiltinProc(car_proc)));
    map.insert("cdr".into(), ValuePtr::new(Value::BuiltinProc(cdr)));
    map.insert("cons".into(), ValuePtr::new(Value::BuiltinProc(cons_proc)));
    map.insert("display".into(), ValuePtr::new(Value::BuiltinProc(display_proc)));
    map.insert("displayln".into(), ValuePtr::new(Value::BuiltinProc(displayln_proc)));
    map.insert("eq?".into(), ValuePtr::new(Value::BuiltinProc(eq_pred)));
    map.insert("equal?".into(), ValuePtr::new(Value::BuiltinProc(equal_pred)));
    map.insert("error".into(), ValuePtr::new(Value::BuiltinProc(error_proc)));
    map.insert("even?".into(), ValuePtr::new(Value::BuiltinProc(even_pred)));
    map.insert("eval".into(), ValuePtr::new(Value::BuiltinProc(eval_proc)));
    map.insert("expt".into(), ValuePtr::new(Value::BuiltinProc(expt_proc)));
    map.insert("exit".into(), ValuePtr::new(Value::BuiltinProc(exit_proc)));
    map.insert("filter".into(), ValuePtr::new(Value::BuiltinProc(filter_proc)));
    map.insert("integer?".into(), ValuePtr::new(Value::BuiltinProc(integer_pred)));
    map.insert("length".into(), ValuePtr::new(Value::BuiltinProc(length)));
    map.insert("list".into(), ValuePtr::new(Value::BuiltinProc(list_proc)));
    map.insert("list?".into(), ValuePtr::new(Value::BuiltinProc(list_pred)));
    map.insert("map".into(), ValuePtr::new(Value::BuiltinProc(map_proc)));
    map.insert("newline".into(), ValuePtr::new(Value::BuiltinProc(newline_proc)));
    map.insert("not".into(), ValuePtr::new(Value::BuiltinProc(not_pred)));
    map.insert("null?".into(), ValuePtr::new(Value::BuiltinProc(null_pred)));
    map.insert("number?".into(), ValuePtr::new(Value::BuiltinProc(number_pred)));
    map.insert("odd?".into(), ValuePtr::new(Value::BuiltinProc(odd_pred)));
    map.insert("pair?".into(), ValuePtr::new(Value::BuiltinProc(pair_pred)));
    map.insert("print".into(), ValuePtr::new(Value::BuiltinProc(print_proc)));
    map.insert("procedure?".into(), ValuePtr::new(Value::BuiltinProc(procedure_pred)));
    map.insert("quotient".into(), ValuePtr::new(Value::BuiltinProc(quotient_proc)));
    map.insert("reduce".into(), ValuePtr::new(Value::BuiltinProc(reduce_proc)));
    map.insert("remainder".into(), ValuePtr::new(Value::BuiltinProc(remainder_proc)));
    map.insert("string?".into(), ValuePtr::new(Value::BuiltinProc(string_pred)));
    map.insert("symbol?".into(), ValuePtr::new(Value::BuiltinProc(symbol_pred)));
    map.insert("zero?".into(), ValuePtr::new(Value::BuiltinProc(zero_pred)));
    map.insert("+".into(), ValuePtr::new(Value::BuiltinProc(add)));
    map.insert("-".into(), ValuePtr::new(Value::BuiltinProc(sub)));
    map.insert("*".into(), ValuePtr::new(Value::BuiltinProc(mul)));
    map.insert("/".into(), ValuePtr::new(Value::BuiltinProc(div_proc)));
    map.insert("=".into(), ValuePtr::new(Value::BuiltinProc(num_eq)));
    map.insert("<".into(), ValuePtr::new(Value::BuiltinProc(lt)));
    map.insert("<=".into(), ValuePtr::new(Value::BuiltinProc(le)));
    map.insert("modulo".into(), ValuePtr::new(Value::BuiltinProc(modulo_proc)));
    map.insert(">".into(), ValuePtr::new(Value::BuiltinProc(gt)));
    map.insert(">=".into(), ValuePtr::new(Value::BuiltinProc(ge)));

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

fn list_from_vec(values: Vec<ValuePtr>) -> ValuePtr {
    values.into_iter().rev().fold(ValuePtr::new(Value::Nil), |cdr, car| {
        ValuePtr::new(Value::Pair(car, cdr))
    })
}

fn is_list_value(value: &Value) -> bool {
    match value {
        Value::Nil => true,
        Value::Pair(_, cdr) => is_list_value(cdr.as_ref()),
        _ => false,
    }
}

fn expect_two_args<'a>(
    name: &str,
    args: &'a [ValuePtr],
) -> Result<(&'a ValuePtr, &'a ValuePtr), LispError> {
    match args {
        [left, right] => Ok((left, right)),
        _ => Err(LispError::RuntimeError(format!("{} requires 2 arguments", name))),
    }
}

fn expect_number_arg(name: &str, value: &ValuePtr) -> Result<f64, LispError> {
    value
        .as_number()
        .ok_or_else(|| LispError::RuntimeError(format!("{} expects a number", name)))
}

fn expect_nonzero_number_arg(name: &str, value: &ValuePtr) -> Result<f64, LispError> {
    let number = expect_number_arg(name, value)?;
    if number == 0.0 {
        return Err(LispError::RuntimeError(format!("{} division by zero", name)));
    }
    Ok(number)
}

pub fn add(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let mut sum = 0.0;
    for arg in args {
        sum += arg
            .as_number()
            .ok_or_else(|| LispError::RuntimeError("Cannot add non-number".into()))?;
    }
    Ok(ValuePtr::new(Value::Numeric(sum)))
}

pub fn abs_proc(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("abs", &args)?;
    Ok(ValuePtr::new(Value::Numeric(expect_number_arg("abs", value)?.abs())))
}

pub fn append_proc(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let mut result = Vec::new();
    for list in args {
        result.extend(list.to_vec()?);
    }
    Ok(list_from_vec(result))
}

pub fn apply_proc(args: Vec<ValuePtr>, env: &EnvPtr) -> Result<ValuePtr, LispError> {
    if args.len() != 2 {
        return Err(LispError::RuntimeError("apply requires 2 arguments".into()));
    }

    let proc = args[0].clone();
    let list_args = args[1].to_vec()?;
    EvalEnv::apply_procedure(env, proc, list_args)
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

pub fn car_proc(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("car", &args)?;
    match value.as_ref() {
        Value::Pair(car, _) => Ok(car.clone()),
        _ => Err(LispError::RuntimeError("car expects a pair".into())),
    }
}

pub fn cdr(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("cdr", &args)?;
    match value.as_ref() {
        Value::Pair(_, cdr) => Ok(cdr.clone()),
        _ => Err(LispError::RuntimeError("cdr expects a pair".into())),
    }
}

pub fn cons_proc(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    if args.len() != 2 {
        return Err(LispError::RuntimeError("cons requires 2 arguments".into()));
    }

    Ok(ValuePtr::new(Value::Pair(args[0].clone(), args[1].clone())))
}

pub fn display_proc(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("display", &args)?;
    match value.as_ref() {
        Value::String(s) => output::write(s),
        other => output::write(&other.to_string()),
    }
    Ok(ValuePtr::new(Value::Nil))
}

pub fn displayln_proc(args: Vec<ValuePtr>, env: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("displayln", &args)?.clone();
    display_proc(vec![value], env)?;
    newline_proc(Vec::new(), env)
}

pub fn div_proc(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    match args.as_slice() {
        [] => Err(LispError::RuntimeError("/ requires at least 1 argument".into())),
        [value] => {
            let denominator = expect_nonzero_number_arg("/", value)?;
            Ok(ValuePtr::new(Value::Numeric(1.0 / denominator)))
        }
        [left, right] => {
            let numerator = expect_number_arg("/", left)?;
            let denominator = expect_nonzero_number_arg("/", right)?;
            Ok(ValuePtr::new(Value::Numeric(numerator / denominator)))
        }
        _ => Err(LispError::RuntimeError("/ supports 1 or 2 arguments".into())),
    }
}

pub fn eq_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let (left, right) = expect_two_args("eq?", &args)?;
    let is_eq = match (left.as_ref(), right.as_ref()) {
        (Value::Nil, Value::Nil) => true,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::Numeric(a), Value::Numeric(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Symbol(a), Value::Symbol(b)) => a == b,
        (Value::BuiltinProc(a), Value::BuiltinProc(b)) => std::ptr::fn_addr_eq(*a, *b),
        (Value::Pair(_, _), Value::Pair(_, _)) => std::rc::Rc::ptr_eq(left, right),
        (Value::LambdaProc { .. }, Value::LambdaProc { .. }) => std::rc::Rc::ptr_eq(left, right),
        _ => false,
    };
    Ok(bool_value(is_eq))
}

pub fn equal_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let (left, right) = expect_two_args("equal?", &args)?;
    Ok(bool_value(left.as_ref() == right.as_ref()))
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

pub fn expt_proc(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let (left, right) = expect_two_args("expt", &args)?;
    let base = expect_number_arg("expt", left)?;
    let exponent = expect_number_arg("expt", right)?;
    if base == 0.0 && exponent == 0.0 {
        return Err(LispError::RuntimeError("expt is undefined for 0^0".into()));
    }
    Ok(ValuePtr::new(Value::Numeric(base.powf(exponent))))
}

pub fn even_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("even?", &args)?;
    let number = expect_number_arg("even?", value)?;
    Ok(bool_value(number.fract() == 0.0 && (number as i64) % 2 == 0))
}

pub fn eval_proc(args: Vec<ValuePtr>, env: &EnvPtr) -> Result<ValuePtr, LispError> {
    let expr = expect_one_arg("eval", &args)?;
    EvalEnv::eval(env, expr.clone())
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

pub fn filter_proc(args: Vec<ValuePtr>, env: &EnvPtr) -> Result<ValuePtr, LispError> {
    if args.len() != 2 {
        return Err(LispError::RuntimeError("filter requires 2 arguments".into()));
    }

    let proc = args[0].clone();
    let items = args[1].to_vec()?;
    let mut kept = Vec::new();

    for item in items {
        let result = EvalEnv::apply_procedure(env, proc.clone(), vec![item.clone()])?;
        if result.is_true() {
            kept.push(item);
        }
    }

    Ok(list_from_vec(kept))
}

pub fn gt(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let (left, right) = expect_two_args(">", &args)?;
    let left = expect_number_arg(">", left)?;
    let right = expect_number_arg(">", right)?;

    Ok(ValuePtr::new(Value::Boolean(left > right)))
}

pub fn ge(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let (left, right) = expect_two_args(">=", &args)?;
    let left = expect_number_arg(">=", left)?;
    let right = expect_number_arg(">=", right)?;
    Ok(bool_value(left >= right))
}

pub fn integer_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("integer?", &args)?;
    let is_integer = match value.as_ref() {
        Value::Numeric(n) => n.fract() == 0.0,
        _ => false,
    };
    Ok(bool_value(is_integer))
}

pub fn length(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let list = expect_one_arg("length", &args)?;
    let count = list.to_vec()?.len() as f64;
    Ok(ValuePtr::new(Value::Numeric(count)))
}

pub fn list_proc(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    Ok(list_from_vec(args))
}

pub fn list_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("list?", &args)?;
    Ok(bool_value(is_list_value(value.as_ref())))
}

pub fn map_proc(args: Vec<ValuePtr>, env: &EnvPtr) -> Result<ValuePtr, LispError> {
    if args.len() != 2 {
        return Err(LispError::RuntimeError("map requires 2 arguments".into()));
    }

    let proc = args[0].clone();
    let items = args[1].to_vec()?;
    let mut mapped = Vec::new();

    for item in items {
        mapped.push(EvalEnv::apply_procedure(env, proc.clone(), vec![item])?);
    }

    Ok(list_from_vec(mapped))
}

pub fn not_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("not", &args)?;
    Ok(bool_value(value.is_false()))
}

pub fn mul(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let mut product = 1.0;
    for arg in args {
        product *= arg
            .as_number()
            .ok_or_else(|| LispError::RuntimeError("Cannot mul non-number".into()))?;
    }
    Ok(ValuePtr::new(Value::Numeric(product)))
}

pub fn newline_proc(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    if !args.is_empty() {
        return Err(LispError::RuntimeError("newline requires 0 arguments".into()));
    }

    output::newline();
    Ok(ValuePtr::new(Value::Nil))
}

pub fn null_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("null?", &args)?;
    Ok(bool_value(matches!(value.as_ref(), Value::Nil)))
}

pub fn number_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("number?", &args)?;
    Ok(bool_value(matches!(value.as_ref(), Value::Numeric(_))))
}

pub fn quotient_proc(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let (left, right) = expect_two_args("quotient", &args)?;
    let dividend = expect_number_arg("quotient", left)?;
    let divisor = expect_nonzero_number_arg("quotient", right)?;
    Ok(ValuePtr::new(Value::Numeric((dividend / divisor).trunc())))
}

pub fn pair_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("pair?", &args)?;
    Ok(bool_value(matches!(value.as_ref(), Value::Pair(_, _))))
}

pub fn odd_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("odd?", &args)?;
    let number = expect_number_arg("odd?", value)?;
    Ok(bool_value(number.fract() == 0.0 && (number as i64) % 2 != 0))
}

pub fn print_proc(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    for v in args {
        output::writeln(&v.to_string());
    }
    Ok(ValuePtr::new(Value::Nil))
}

pub fn procedure_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("procedure?", &args)?;
    Ok(bool_value(matches!(
        value.as_ref(),
        Value::BuiltinProc(_) | Value::LambdaProc { .. }
    )))
}

pub fn num_eq(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let (left, right) = expect_two_args("=", &args)?;
    let left = expect_number_arg("=", left)?;
    let right = expect_number_arg("=", right)?;
    Ok(bool_value(left == right))
}

pub fn reduce_proc(args: Vec<ValuePtr>, env: &EnvPtr) -> Result<ValuePtr, LispError> {
    if args.len() != 2 {
        return Err(LispError::RuntimeError("reduce requires 2 arguments".into()));
    }

    let proc = args[0].clone();
    let items = args[1].to_vec()?;
    if items.is_empty() {
        return Err(LispError::RuntimeError("reduce requires a non-empty list".into()));
    }
    if items.len() == 1 {
        return Ok(items[0].clone());
    }

    let first = items[0].clone();
    let rest = list_from_vec(items[1..].to_vec());
    let reduced_rest = reduce_proc(vec![proc.clone(), rest], env)?;
    EvalEnv::apply_procedure(env, proc, vec![first, reduced_rest])
}

pub fn remainder_proc(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let (left, right) = expect_two_args("remainder", &args)?;
    let dividend = expect_number_arg("remainder", left)?;
    let divisor = expect_nonzero_number_arg("remainder", right)?;
    let quotient = (dividend / divisor).trunc();
    Ok(ValuePtr::new(Value::Numeric(dividend - divisor * quotient)))
}

pub fn string_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("string?", &args)?;
    Ok(bool_value(matches!(value.as_ref(), Value::String(_))))
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

pub fn symbol_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("symbol?", &args)?;
    Ok(bool_value(matches!(value.as_ref(), Value::Symbol(_))))
}

pub fn modulo_proc(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let (left, right) = expect_two_args("modulo", &args)?;
    let dividend = expect_number_arg("modulo", left)?;
    let divisor = expect_nonzero_number_arg("modulo", right)?;
    let quotient = (dividend / divisor).trunc();
    let mut remainder = dividend - divisor * quotient;
    if remainder != 0.0 && remainder.signum() != divisor.signum() {
        remainder += divisor;
    }
    Ok(ValuePtr::new(Value::Numeric(remainder)))
}

pub fn lt(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let (left, right) = expect_two_args("<", &args)?;
    let left = expect_number_arg("<", left)?;
    let right = expect_number_arg("<", right)?;
    Ok(bool_value(left < right))
}

pub fn le(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let (left, right) = expect_two_args("<=", &args)?;
    let left = expect_number_arg("<=", left)?;
    let right = expect_number_arg("<=", right)?;
    Ok(bool_value(left <= right))
}

pub fn zero_pred(args: Vec<ValuePtr>, _: &EnvPtr) -> Result<ValuePtr, LispError> {
    let value = expect_one_arg("zero?", &args)?;
    let number = expect_number_arg("zero?", value)?;
    Ok(bool_value(number == 0.0))
}
