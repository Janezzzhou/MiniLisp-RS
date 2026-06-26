use std::fmt;

pub type ValuePtr = std::rc::Rc<Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    Numeric(f64),
    String(String),
    Nil,
    Symbol(String),
    Pair(ValuePtr, ValuePtr),
}

impl Value {
    fn fmt_pair(car: &ValuePtr, cdr: &ValuePtr, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        write!(f, "{}", car)?;

        let mut tail = cdr.clone();
        while let Value::Pair(next_car, next_cdr) = tail.as_ref() {
            write!(f, " ")?;
            write!(f, "{}", next_car)?;
            tail = next_cdr.clone();
        }

        match tail.as_ref() {
            Value::Nil => write!(f, ")"),
            other => write!(f, " . {})", other),
        }
    }
}

impl Value {
    pub fn is_self_evaluating(&self) -> bool {
        matches!(self, Value::Boolean(_) | Value::Numeric(_) | Value::String(_))
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Boolean(true) => write!(f, "#t"),
            Value::Boolean(false) => write!(f, "#f"),
            Value::Numeric(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{:?}", s),
            Value::Nil => write!(f, "()"),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::Pair(car, cdr) => Value::fmt_pair(car, cdr, f),
        }
    }
}

use crate::error::LispError;

impl Value {
    /// 如果是 Symbol，返回名字
    pub fn as_symbol(&self) -> Option<&str> {
        match self {
            Value::Symbol(s) => Some(s),
            _ => None,
        }
    }

    /// 将 Proper List 转换为 Vec<ValuePtr>
    pub fn to_vec(&self) -> Result<Vec<ValuePtr>, LispError> {
        let mut result = Vec::new();
        let mut current = self;

        loop {
            match current {
                Value::Nil => return Ok(result),

                Value::Pair(car, cdr) => {
                    result.push(car.clone());
                    current = cdr.as_ref();
                }

                _ => {
                    return Err(LispError::RuntimeError(
                        "Malformed list.".into(),
                    ));
                }
            }
        }
    }
}