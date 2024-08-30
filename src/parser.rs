use std::{fmt::{self, Display}, iter::Peekable};

use anyhow::{anyhow, Result};
use crate::{tokenizer::{CharToken, Token, Tokenizer}, values::Value};

#[derive(Debug)]
pub struct Parser<'a> {
    tokenizer: Peekable<Tokenizer<'a>>, 
}

#[derive(Debug)]
pub enum AstNode {
    Literal(Value),
    Group(Vec<AstNode>),
    Program(Vec<AstNode>),
}

impl<'a> From<Tokenizer<'a>> for Parser<'a> {
    fn from(value: Tokenizer<'a>) -> Self {
        Parser {
            tokenizer: value.peekable(),
        }
    }
}

impl Display for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstNode::Literal(value) => write!(f, "{}", value),
            AstNode::Group(nodes) => {
                write!(f, "(group")?;
                for node in nodes {
                    write!(f, " {}", node)?;
                }
                write!(f, ")")?;

                Ok(())
            },
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
        log::debug!("parse()");

        let nodes = self.parse_until(Token::EOF)?;
        Ok(AstNode::Program(nodes))
    }

    fn parse_one(&mut self) -> Result<Option<AstNode>> {
        let token = self.tokenizer.next();
        log::debug!("parse_one(), token = {token:?}");
        
        match token {
            // Done parsing, nothing to return
            None | Some(Token::EOF) => Ok(None),
            
            // Literals
            Some(Token::Literal(_, value)) => Ok(Some(AstNode::Literal(value))),

            // Groups (...)
            Some(Token::CharToken(CharToken::LeftParen)) => {
                let group = self.parse_until(Token::CharToken(CharToken::RightParen))?;
                Ok(Some(AstNode::Group(group)))
            },
            
            t => Err(anyhow!("Haven't parsed {t:?} yet")),
        }
    }

    fn parse_until(&mut self, target: Token) -> Result<Vec<AstNode>> {
        log::debug!("parse_until({target:?})");
        let mut nodes = vec![];

        while let Some(token) = self.tokenizer.peek() {
            log::debug!("parse_until({target:?}), node = {token:?}");

            if token == &target {
                self.tokenizer.next(); // Consume target
                return Ok(nodes);
            } else if let Some(node) = self.parse_one()? {
                nodes.push(node);
            } else {
                return Err(anyhow!("Unexpected end of tokens; expected {target:?}"));
            }
        }

        Err(anyhow!("Unexpected end of tokens; expected {target:?}"))
    }
}