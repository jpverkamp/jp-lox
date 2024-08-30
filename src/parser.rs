use std::fmt::{self, Display};

use anyhow::Result;
use crate::{tokenizer::{Token, Tokenizer}, values::Value};

#[derive(Debug)]
pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>, 
}

#[derive(Debug)]
pub enum AstNode {
    Literal(Value),
    Program(Vec<AstNode>),
}

impl<'a> From<Tokenizer<'a>> for Parser<'a> {
    fn from(value: Tokenizer<'a>) -> Self {
        Parser {
            tokenizer: value,
        }
    }
}

impl Display for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstNode::Literal(value) => write!(f, "{}", value),
            AstNode::Program(nodes) => {
                for node in nodes {
                    write!(f, "{}\n", node)?;
                }

                Ok(())
            },
        }
    }
}

impl Parser<'_> {
    pub fn parse(&mut self) -> Result<AstNode> {
        let mut program = vec![];

        while let Some(token) = self.tokenizer.next() {
            match token {
                Token::Literal(_, value) => program.push(AstNode::Literal(value)),
                Token::EOF => break,

                _ => panic!("Unexpected token: {token:?}"),
            }
        }

        Ok(AstNode::Program(program))
    }
}