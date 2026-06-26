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
