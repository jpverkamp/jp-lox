use crate::{parser::AstNode, values::Value};

use anyhow::{anyhow, Result};

pub trait Evaluate {
    fn evaluate(&self) -> Result<Value>;
}

macro_rules! assert_arity {
    ($args:expr => $($n:literal),+ $(,)?) => {
        let arities = [$($n),+];

        if !arities.contains(&$args.len()) {
            return Err(anyhow!("Expected {:?} arguments, found {}", [$($n),+], $args.len()));
        }
    };
}

macro_rules! as_number {
    ($expr:expr) => {
        match $expr {
            Value::Number(n) => n,
            _ => return Err(anyhow!("Expected number, found {}", $expr)),
        }
    };
}

macro_rules! as_boolean {
    ($expr:expr) => {
        match $expr {
            Value::Bool(false) | Value::Nil => false,
            _ => true,
        }
    };
}

macro_rules! numeric_binop {
    ($name:ident, $op:tt) => {
        |args: Vec<Value> | {
            assert_arity!(args => 2);

            let a = as_number!(args[0]);
            let b = as_number!(args[1]);

            Ok(Value::Number(a $op b))
        }
    };
}

impl Evaluate for AstNode {
    fn evaluate(&self) -> Result<Value> {
        match self {
            AstNode::Literal(value) => Ok(value.clone()),
            AstNode::Symbol(name) => Ok(Value::Symbol(name.clone())),
            
            AstNode::Program(nodes) 
            | AstNode::Group(nodes) => {
                let mut last = Value::Nil;
                for node in nodes {
                    last = node.evaluate()?;
                }

                Ok(last)
            },

            AstNode::Application(func, args) => {
                let func = match func.evaluate()? {
                    Value::Symbol(name) => {
                        match name.as_str() {
                            "-" => |args: Vec<Value>| {
                                assert_arity!(args => 1, 2);

                                if args.len() == 1 {
                                    let v = as_number!(args[0]);
                                    Ok(Value::Number(-v))
                                } else {
                                    let a = as_number!(args[0]);
                                    let b = as_number!(args[1]);
                                    Ok(Value::Number(a - b))
                                }
                            },
                            "!" => |args: Vec<Value>| {
                                assert_arity!(args => 1);

                                let v = as_boolean!(args[0]);
                                Ok(Value::Bool(!v))
                            },

                            "+" => numeric_binop!(add, +),
                            "*" => numeric_binop!(mul, *),
                            "/" => numeric_binop!(div, /),

                            _ => unimplemented!("Only built ins are implemented"),
                        }
                    }
                    _ => unimplemented!("Only built ins are implemented"),
                };

                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(arg.evaluate()?);
                }

                let result = func(arg_values)?;
                Ok(result)
            },
        }
    }
}