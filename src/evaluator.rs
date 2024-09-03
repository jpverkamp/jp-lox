use crate::environment::Environment;
use crate::values::Value;
use crate::{parser::AstNode, tokenizer::Keyword};
use crate::builtins::BuiltIn;

use anyhow::{anyhow, Result};

pub trait Evaluate {
    fn evaluate(&self, env: &mut impl Environment<Value>) -> Result<Value>;
}

impl Evaluate for AstNode {
    fn evaluate(&self, env: &mut impl Environment<Value>) -> Result<Value> {
        match self {
            AstNode::Literal(_, value) => Ok(value.clone()),
            AstNode::Symbol(span, name) => {
                // Keywords become builtins; fall back to env; then error
                if Keyword::try_from(name.as_str()).is_ok() {
                    return Ok(Value::Builtin(name.clone()));
                }

                match env.get(name) {
                    Some(value) => Ok(value),
                    None => {
                        let line = span.line;
                        Err(anyhow!("[line {line}] Undefined variable '{name}'"))
                    }
                }
            }

            AstNode::Program(_, nodes) | AstNode::Group(_, nodes) => {
                let mut last = Value::Nil;
                for node in nodes {
                    last = node.evaluate(env)?;
                }

                Ok(last)
            }

            AstNode::Block(_, nodes) => {
                env.enter();

                let mut last = Value::Nil;
                for node in nodes {
                    last = node.evaluate(env)?;
                }

                env.exit();

                Ok(last)
            }

            AstNode::Application(_span, func, args) => {
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(arg.evaluate(env)?);
                }
                
                match func.evaluate(env)? {
                    Value::Builtin(name) => {
                        let callable = BuiltIn::try_from(name.as_str())?;
                        callable.call(arg_values)
                    }
                    _ => unimplemented!("Only built ins are implemented"),
                }
            }

            AstNode::Declaration(_, name, body) => {
                let value = body.evaluate(env)?;
                env.set(name, value.clone());
                Ok(value)
            }

            AstNode::Assignment(span, name, body) => {
                if env.get(name).is_none() {
                    let line = span.line;
                    return Err(anyhow!("[line {line}] Undefined variable '{name}'"));
                }

                let value = body.evaluate(env)?;
                env.set(name, value.clone());
                Ok(value)
            }
        }
    }
}
