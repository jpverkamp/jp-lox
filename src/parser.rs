use std::fmt::{self, Display};

use crate::{
    span::Span,
    tokenizer::{Keyword, Token, Tokenizer},
    values::Value,
};
use anyhow::{anyhow, Ok, Result};

#[derive(Debug)]
pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
}

#[derive(Debug)]
pub enum AstNode {
    Literal(Span, Value),
    Symbol(Span, String),
    Group(Span, Vec<AstNode>),
    Application(Span, Box<AstNode>, Vec<AstNode>),
    Program(Span, Vec<AstNode>),
}

impl<'a> From<Tokenizer<'a>> for Parser<'a> {
    fn from(value: Tokenizer<'a>) -> Self {
        Parser { tokenizer: value }
    }
}

impl Display for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstNode::Literal(_, value) => write!(f, "{}", value),
            AstNode::Symbol(_, name) => write!(f, "{}", name),
            AstNode::Group(_, nodes) => {
                write!(f, "(group")?;
                for node in nodes {
                    write!(f, " {}", node)?;
                }
                write!(f, ")")?;

                std::fmt::Result::Ok(())
            }
            AstNode::Application(_, func, args) => {
                write!(f, "({}", func)?;
                for arg in args {
                    write!(f, " {}", arg)?;
                }
                write!(f, ")")?;

                std::fmt::Result::Ok(())
            }
            AstNode::Program(_, nodes) => {
                for node in nodes {
                    write!(f, "{}\n", node)?;
                }

                std::fmt::Result::Ok(())
            }
        }
    }
}

impl AstNode {
    pub fn span(&self) -> Span {
        match self {
            AstNode::Literal(span, _)
            | AstNode::Symbol(span, _)
            | AstNode::Group(span, _)
            | AstNode::Application(span, _, _)
            | AstNode::Program(span, _) => *span,
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
                Some(Token::Keyword(span, which @ Keyword::$which)) => Some((span, which.to_value().to_string())),
            )*
            _ => None,
        }
    }
}

impl Parser<'_> {
    pub fn parse(&mut self) -> Result<AstNode> {
        let mut nodes = vec![];
        let mut span = Span::ZERO;

        while let Some(token) = self.tokenizer.peek() {
            if token == &Token::EOF {
                break;
            }

            let node = self.parse_expression()?;
            span = span.merge(&node.span());
            nodes.push(node);
        }

        Ok(AstNode::Program(span, nodes))
    }

    fn parse_expression(&mut self) -> Result<AstNode> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<AstNode> {
        let mut lhs = self.parse_comparison()?;

        while let Some((&op_span, op)) = matches_keyword!(
            self.tokenizer.peek() => BangEqual, EqualEqual,
        ) {
            self.tokenizer.next();
            let rhs = self.parse_comparison()?;
            let span = lhs.span().merge(&op_span.merge(&rhs.span()));

            lhs =
                AstNode::Application(span, Box::new(AstNode::Symbol(op_span, op)), vec![lhs, rhs]);
        }

        Ok(lhs)
    }

    fn parse_comparison(&mut self) -> Result<AstNode> {
        let mut lhs = self.parse_term()?;

        while let Some((&op_span, op)) = matches_keyword!(
            self.tokenizer.peek() => Greater, GreaterEqual, Less, LessEqual,
        ) {
            self.tokenizer.next();
            let rhs = self.parse_term()?;
            let span = lhs.span().merge(&op_span.merge(&rhs.span()));

            lhs =
                AstNode::Application(span, Box::new(AstNode::Symbol(op_span, op)), vec![lhs, rhs]);
        }

        Ok(lhs)
    }

    fn parse_term(&mut self) -> Result<AstNode> {
        let mut lhs = self.parse_factor()?;

        while let Some((&op_span, op)) = matches_keyword!(
            self.tokenizer.peek() => Minus, Plus,
        ) {
            self.tokenizer.next();
            let rhs = self.parse_factor()?;
            let span = lhs.span().merge(&op_span.merge(&rhs.span()));

            lhs =
                AstNode::Application(span, Box::new(AstNode::Symbol(op_span, op)), vec![lhs, rhs]);
        }

        Ok(lhs)
    }

    fn parse_factor(&mut self) -> Result<AstNode> {
        let mut lhs = self.parse_unary()?;

        while let Some((&op_span, op)) = matches_keyword!(
            self.tokenizer.peek() => Slash, Star,
        ) {
            self.tokenizer.next();
            let rhs = self.parse_unary()?;
            let span = lhs.span().merge(&op_span.merge(&rhs.span()));

            lhs =
                AstNode::Application(span, Box::new(AstNode::Symbol(op_span, op)), vec![lhs, rhs]);
        }

        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<AstNode> {
        if let Some((&op_span, op)) = matches_keyword!(
            self.tokenizer.peek() => Bang, Minus,
        ) {
            self.tokenizer.next();
            let rhs = self.parse_unary()?;
            let span = op_span.merge(&rhs.span());

            Ok(AstNode::Application(
                span,
                Box::new(AstNode::Symbol(op_span, op)),
                vec![rhs],
            ))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<AstNode> {
        if let Some(token) = self.tokenizer.next() {
            match token {
                Token::Literal(span, _, v) => Ok(AstNode::Literal(span, v)),
                Token::Keyword(left_span, Keyword::LeftParen) => {
                    let group = self.parse_expression()?;
                    if let Some(Token::Keyword(right_span, Keyword::RightParen)) =
                        self.tokenizer.next()
                    {
                        let span = left_span.merge(&right_span);
                        Ok(AstNode::Group(span, vec![group]))
                    } else {
                        let line = self.line_number(left_span.start);
                        Err(anyhow!(
                            "[line {}] Error at '{}': Expect expression",
                            line,
                            token
                        ))
                    }
                }
                Token::EOF => Err(anyhow!("Error at EOF: Expect expression")),
                Token::Identifier(span, _) => {
                    let line = self.line_number(span.start);
                    Err(anyhow!(
                        "[line {}] Error at '{}': Expect expression",
                        line,
                        token
                    ))
                }
                Token::Keyword(span, keyword) => {
                    let line = self.line_number(span.start);
                    Err(anyhow!(
                        "[line {}] Error at '{}': Expect expression",
                        line,
                        keyword.to_value()
                    ))
                }
            }
        } else {
            unreachable!("EOF should be handled in parse_expression")
        }
    }
}

impl Parser<'_> {
    fn line_number(&self, byte_pos: usize) -> usize {
        self.tokenizer.source[..byte_pos].lines().count()
    }

    pub fn encountered_tokenizer_error(&self) -> bool {
        self.tokenizer.encountered_error()
    }
}
