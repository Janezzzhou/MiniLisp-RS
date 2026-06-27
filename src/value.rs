use std::fmt;
use crate::error::LispError;

pub type ValuePtr = std::rc::Rc<Value>;
pub type BuiltinFunc = fn(Vec<ValuePtr>) -> Result<ValuePtr, LispError>;

#[derive(Debug, Clone)]
pub enum Value {
    Boolean(bool),
    Numeric(f64),
    String(String),
    Nil,
    Symbol(String),
    Pair(ValuePtr, ValuePtr),
    BuiltinProc(BuiltinFunc),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Numeric(a), Value::Numeric(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Pair(a_car, a_cdr), Value::Pair(b_car, b_cdr)) => {
                a_car == b_car && a_cdr == b_cdr
            }
            (Value::BuiltinProc(a), Value::BuiltinProc(b)) => std::ptr::fn_addr_eq(*a, *b),
            _ => false,
        }
    }
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
            Value::BuiltinProc(_) => {write!(f, "#<procedure>")}
        }
    }
}

impl Value {
    // 类型判断
    pub fn is_self_evaluating(&self) -> bool {
        matches!(self, Value::Boolean(_) | Value::Numeric(_) | Value::String(_) | Value::BuiltinProc(_)) 
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    pub fn is_builtin_proc(&self) -> bool {
        matches!(self, Value::BuiltinProc(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Value::Numeric(_))
    }

    pub fn is_false(&self) -> bool {
        matches!(self, Value::Boolean(false))
    }

    pub fn is_true(&self) -> bool {
        !self.is_false()
    }
}

impl Value {
    // 类型转换
    pub fn as_symbol(&self) -> Option<&str> {
        match self {
            Value::Symbol(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_builtin_proc(&self) -> Option<BuiltinFunc> {
        match self {
            Value::BuiltinProc(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Numeric(n) => Some(*n),
            _ => None,
        }
    }

    // 列表操作：将 Proper List 转换为 Vec<ValuePtr>
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
                    return Err(LispError::RuntimeError("Malformed list.".into(),));
                }
            }
        }
    }
}
