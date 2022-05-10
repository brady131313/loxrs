use std::fmt::Display;

use crate::object::IString;

const FLOAT_TOL: f64 = 1e-9;

#[derive(Debug, Copy, Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Num(f64),
    String(IString),
}

impl Value {
    pub fn as_num(&self) -> Option<f64> {
        if let Self::Num(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<IString> {
        if let Self::String(s) = self {
            Some(*s)
        } else {
            None
        }
    }

    pub fn is_falsey(&self) -> bool {
        matches!(self, Self::Nil | Self::Bool(false))
    }

    pub fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Nil, _) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Num(a), Value::Num(b)) => (a - b) < FLOAT_TOL,
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Num(f)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Num(f64::default())
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Num(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s:?}"),
        }
    }
}
