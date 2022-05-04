use std::fmt::Display;

#[derive(Debug, Copy, Clone)]
pub enum Value {
    Num(f64)
}

impl Value {
    pub fn as_num(&self) -> Option<f64> {
        if let Self::Num(n) = self {
            Some(*n)
        } else {
            None
        }
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Num(f)
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
            Self::Num(n) => write!(f, "{n}")
        }
    }
}
