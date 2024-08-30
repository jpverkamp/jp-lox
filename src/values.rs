use derive_more::Display;

#[derive(Debug, Display, Clone, PartialEq)]
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