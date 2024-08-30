use crate::{parser::AstNode, values::Value};

use anyhow::Result;

pub trait Evaluate {
    fn evaluate(&self) -> Result<Value>;
}

impl Evaluate for AstNode {
    fn evaluate(&self) -> Result<Value> {
        match self {
            AstNode::Literal(value) => Ok(value.clone()),
            AstNode::Program(nodes) => {
                let mut last = Value::Nil;
                for node in nodes {
                    last = node.evaluate()?;
                }

                Ok(last)
            },
            _ => unimplemented!()
        }
    }
}