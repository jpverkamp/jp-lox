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
    Assignment(Span, String, Box<AstNode>),
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
            AstNode::Assignment(_, name, value) => {
                write!(f, "(var {} {})", name, value)?;

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
            | AstNode::Assignment(span, _, _)
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

            let node = self.parse_statement()?;
            span = span.merge(&node.span());
            nodes.push(node);

            // Semi colon delimited
            if let Some(Token::Keyword(_, Keyword::Semicolon)) = self.tokenizer.peek() {
                self.tokenizer.next();
            }
        }

        Ok(AstNode::Program(span, nodes))
    }

    fn parse_statement(&mut self) -> Result<AstNode> {
        log::debug!("parse_statement");

        match self.tokenizer.peek() {
            Some(Token::Keyword(_, Keyword::Print)) => self.parse_print(),
            Some(Token::Keyword(_, Keyword::Var)) => self.parse_var(),
            _ => self.parse_expression(),
        }
    }

    fn parse_print(&mut self) -> Result<AstNode> {
        let keyword = self.tokenizer.next().unwrap();
        let span = keyword.span();
        log::debug!("parse_print @ {span:?}");

        let expression = self.parse_expression()?;
        let span = span.merge(&expression.span());

        Ok(AstNode::Application(
            span,
            Box::new(AstNode::Symbol(span, "print".to_string())),
            vec![expression],
        ))
    }

    fn parse_var(&mut self) -> Result<AstNode> {
        let var_keyword = self.tokenizer.next().unwrap();
        let span = var_keyword.span();
        log::debug!("parse_var @ {span:?}");

        let name = if let Some(Token::Identifier(span, name)) = self.tokenizer.next() {
            span.merge(&span);
            name
        } else {
            let line = self.line_number(span.start);
            return Err(anyhow!("[line {}] Error at '{}': Expect identifier", line, var_keyword));
        };

        if let Some(Token::Keyword(_, Keyword::Equal)) = self.tokenizer.next() {
        } else {
            let line = self.line_number(span.start);
            return Err(anyhow!("[line {}] Error at '{}': Expect '='", line, var_keyword));
        }

        let expression = self.parse_expression()?;
        let span = span.merge(&expression.span());

        Ok(AstNode::Assignment(span, name, Box::new(expression)))
    }


    fn parse_expression(&mut self) -> Result<AstNode> {
        log::debug!("parse_expression");
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<AstNode> {
        let mut lhs = self.parse_comparison()?;

        while let Some((&op_span, op)) = matches_keyword!(
            self.tokenizer.peek() => BangEqual, EqualEqual,
        ) {
            log::debug!("parse_equality @ op_span: {:?}", op_span);

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
            log::debug!("parse_comparison @ op_span: {:?}", op_span);

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
            log::debug!("parse_term @ op_span: {:?}", op_span);

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
            log::debug!("parse_factor @ op_span: {:?}", op_span);

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
            log::debug!("parse_unary @ op_span: {:?}", op_span);

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
            log::debug!("parse_primary @ {:?}", token.span());

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
                Token::Identifier(span, id) => {
                    Ok(AstNode::Symbol(span, id))
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
