use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
}

impl Value {
    pub const CONSTANT_VALUES: [(&'static str, Value); 3] = [
        ("nil", Value::Nil),
        ("true", Value::Bool(true)),
        ("false", Value::Bool(false)),
    ];
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => {
                // I don't make the rules
                // TODO: Make the rules
                if n.fract() == 0.0 {
                    write!(f, "{:.1}", n)
                } else {
                    write!(f, "{}", n)
                }
            },
            Value::String(s) => write!(f, "{}", s),
        }
    }
}