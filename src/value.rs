use std::fmt::Display;

#[derive(Debug)]
pub struct Value(f64);

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value(f)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
