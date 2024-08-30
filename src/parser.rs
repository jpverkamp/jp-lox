use std::{fmt::{self, Display}, iter::Peekable};

use anyhow::{anyhow, Ok, Result};
use crate::{tokenizer::{Keyword, Token, Tokenizer}, values::Value};

#[derive(Debug)]
pub struct Parser<'a> {
    source: &'a str,
    tokenizer: Peekable<Tokenizer<'a>>, 
}

#[derive(Debug)]
pub enum AstNode {
    Literal(Value),
    Symbol(String),
    Group(Vec<AstNode>),
    Application(Box<AstNode>, Vec<AstNode>),
    Program(Vec<AstNode>),
}

impl<'a> From<Tokenizer<'a>> for Parser<'a> {
    fn from(value: Tokenizer<'a>) -> Self {
        Parser {
            source: value.source,
            tokenizer: value.peekable(),
        }
    }
}

impl Display for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstNode::Literal(value) => write!(f, "{}", value),
            AstNode::Symbol(name) => write!(f, "{}", name),
            AstNode::Group(nodes) => {
                write!(f, "(group")?;
                for node in nodes {
                    write!(f, " {}", node)?;
                }
                write!(f, ")")?;

                std::fmt::Result::Ok(())
            },
            AstNode::Application(func, args) => {
                write!(f, "({}", func)?;
                for arg in args {
                    write!(f, " {}", arg)?;
                }
                write!(f, ")")?;

                std::fmt::Result::Ok(())
            },
            AstNode::Program(nodes) => {
                for node in nodes {
                    write!(f, "{}\n", node)?;
                }

                std::fmt::Result::Ok(())
            },
        }
    }
}

macro_rules! matches_keyword {
    (
        $token:expr => 
        $($which:ident),*
        $(,)?
    ) => {
        match $token {
            $(
                Some(Token::Keyword(_, which @ Keyword::$which)) => Some(which.to_value().to_string()),
            )*
            _ => None,
        }
    }
}

impl Parser<'_> {
    pub fn parse(&mut self) -> Result<AstNode> {
        let mut nodes = vec![];

        while let Some(token) = self.tokenizer.peek() {
            if token == &Token::EOF {
                break;
            }

            nodes.push(self.parse_expression()?);
        }

        Ok(AstNode::Program(nodes))
    }

    fn parse_expression(&mut self) -> Result<AstNode> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<AstNode> {
        let mut lhs = self.parse_comparison()?;

        while let Some(op) = matches_keyword!(
            self.tokenizer.peek() => BangEqual, EqualEqual,
        ) {
            self.tokenizer.next();
            let rhs = self.parse_comparison()?;

            lhs = AstNode::Application(
                Box::new(AstNode::Symbol(op)),
                vec![lhs, rhs],
            );
        }

        Ok(lhs)
    }

    fn parse_comparison(&mut self) -> Result<AstNode> {
        let mut lhs = self.parse_term()?;

        while let Some(op) = matches_keyword!(
            self.tokenizer.peek() => Greater, GreaterEqual, Less, LessEqual,
        ) {
            self.tokenizer.next();
            let rhs = self.parse_term()?;

            lhs = AstNode::Application(
                Box::new(AstNode::Symbol(op)),
                vec![lhs, rhs],
            );
        }

        Ok(lhs)
    }

    fn parse_term(&mut self) -> Result<AstNode> {
        let mut lhs = self.parse_factor()?;

        while let Some(op) = matches_keyword!(
            self.tokenizer.peek() => Minus, Plus,
        ) {
            self.tokenizer.next();
            let rhs = self.parse_factor()?;

            lhs = AstNode::Application(
                Box::new(AstNode::Symbol(op)),
                vec![lhs, rhs],
            );
        }

        Ok(lhs)
    }

    fn parse_factor(&mut self) -> Result<AstNode> {
        let mut lhs = self.parse_unary()?;

        while let Some(op) = matches_keyword!(
            self.tokenizer.peek() => Slash, Star,
        ) {
            self.tokenizer.next();
            let rhs = self.parse_unary()?;

            lhs = AstNode::Application(
                Box::new(AstNode::Symbol(op)),
                vec![lhs, rhs],
            );
        }

        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<AstNode> {
        if let Some(op) = matches_keyword!(
            self.tokenizer.peek() => Bang, Minus,
        ) {
            self.tokenizer.next();
            let rhs = self.parse_unary()?;

            Ok(AstNode::Application(
                Box::new(AstNode::Symbol(op)),
                vec![rhs],
            ))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<AstNode> {
        if let Some(token) = self.tokenizer.next() {
            match token {
                Token::Literal(_, _, v) => Ok(AstNode::Literal(v)),
                Token::Keyword(span, Keyword::LeftParen) => {
                    let group = self.parse_expression()?;
                    if let Some(Token::Keyword(_, Keyword::RightParen)) = self.tokenizer.next() {
                        Ok(AstNode::Group(vec![group]))
                    } else {
                        let line = self.line_number(span.start);
                        Err(anyhow!("[line {}] Error at '{}': Expect expression", line, token)) 
                    }
                },
                Token::EOF => Err(anyhow!("Error at EOF: Expect expression")),
                Token::Identifier(span, _) => {
                    let line = self.line_number(span.start);
                    Err(anyhow!("[line {}] Error at '{}': Expect expression", line, token)) 
                },
                Token::Keyword(span, keyword) => {
                    let line = self.line_number(span.start);
                    Err(anyhow!("[line {}] Error at '{}': Expect expression", line, keyword.to_value())) 
                }
            }
        } else {
            unreachable!("EOF should be handled in parse_expression")
        }
    }
}

impl Parser<'_> {
    fn line_number(&self, byte_pos: usize) -> usize {
        self.source[..byte_pos].lines().count()
    }
}