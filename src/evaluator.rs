use crate::{parser::AstNode, tokenizer::Keyword};
use crate::values::Value;
use crate::environment::Environment;

use anyhow::{anyhow, Result};

pub trait Evaluate {
    fn evaluate(&self, env: &mut impl Environment) -> Result<Value>;
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
            _ => return Err(anyhow!("Operands must be numbers."))
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
    ($op:tt) => {
        |args: Vec<Value>| {
            assert_arity!(args => 2);

            let a = as_number!(args[0]);
            let b = as_number!(args[1]);
            Ok(Value::Number(a $op b))
        }
    };
}

macro_rules! comparison_binop {
    ($op:tt) => {
        |args: Vec<Value>| {
            assert_arity!(args => 2);

            let a = as_number!(args[0]);
            let b = as_number!(args[1]);
            Ok(Value::Bool(a $op b))
        }
    };
}

impl Evaluate for AstNode {
    fn evaluate(&self, env: &mut impl Environment) -> Result<Value> {
        match self {
            AstNode::Literal(_, value) => Ok(value.clone()),
            AstNode::Symbol(_, name) => {
                // Keywords become builtins; fall back to env; then error
                if Keyword::try_from(name.as_str()).is_ok() {
                    return Ok(Value::Builtin(name.clone()));
                }

                match env.get(name) {
                    Some(value) => Ok(value),
                    None => Err(anyhow!("Undefined variable '{}'", name)),
                }
            },

            AstNode::Program(_, nodes) | AstNode::Group(_, nodes) => {
                let mut last = Value::Nil;
                for node in nodes {
                    last = node.evaluate(env)?;
                }

                Ok(last)
            }

            AstNode::Application(span, func, args) => {
                let func = match func.evaluate(env)? {
                    Value::Builtin(name) => {
                        match name.as_str() {
                            // Overloaded operator, both addition and string concatenation
                            // TODO: This is ugly :)
                            "+" => |args: Vec<Value>| {
                                assert_arity!(args => 2);

                                let a = args[0].clone();
                                let b = args[1].clone();

                                match (a, b) {
                                    (Value::Number(a), Value::Number(b)) => {
                                        Ok(Value::Number(a + b))
                                    }
                                    (Value::String(a), Value::String(b)) => {
                                        let mut result = String::new();
                                        result.push_str(&a);
                                        result.push_str(&b);
                                        Ok(Value::String(result))
                                    }
                                    _ => {
                                        return Err(anyhow!(
                                            "Expected number or string, found {} and {}",
                                            args[0],
                                            args[1]
                                        ))
                                    }
                                }
                            },
                            "-" => |args: Vec<Value>| {
                                assert_arity!(args => 1, 2);

                                if args.len() == 1 {
                                    if let Value::Number(v) = args[0] {
                                        Ok(Value::Number(-v))
                                    } else {
                                        Err(anyhow!{"Operand must be a number."})
                                    }
                                } else {
                                    let a = as_number!(args[0]);
                                    let b = as_number!(args[1]);
                                    Ok(Value::Number(a - b))
                                }
                            },
                            "*" => numeric_binop!(*),
                            "/" => numeric_binop!(/),

                            "!" => |args: Vec<Value>| {
                                assert_arity!(args => 1);

                                let v = as_boolean!(args[0]);
                                Ok(Value::Bool(!v))
                            },

                            "<" => comparison_binop!(<),
                            "<=" => comparison_binop!(<=),
                            ">=" => comparison_binop!(>=),
                            ">" => comparison_binop!(>),

                            // Equality can apply to any types as long as they're the same
                            "==" => |args: Vec<Value>| {
                                assert_arity!(args => 2);

                                Ok(Value::Bool(args[0] == args[1]))
                            },
                            "!=" => |args: Vec<Value>| {
                                assert_arity!(args => 2);

                                Ok(Value::Bool(args[0] != args[1]))
                            },

                            "print" => |args: Vec<Value>| {
                                assert_arity!(args => 1);

                                match args[0] {
                                    Value::Number(n) => println!("{}", n),
                                    _ => println!("{}", args[0]),
                                }

                                Ok(Value::Nil)
                            },

                            _ => unimplemented!("Only built ins are implemented"),
                        }
                    }
                    _ => unimplemented!("Only built ins are implemented"),
                };

                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(arg.evaluate(env)?);
                }

                match func(arg_values) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(anyhow!("[line {}] {e}", span.line))
                }
            }

            AstNode::Assignment(_, name, body) => {
                let value = body.evaluate(env)?;
                env.set(name, value.clone());
                Ok(value)
            }
        }
    }
}
