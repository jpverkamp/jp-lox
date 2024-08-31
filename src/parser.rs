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
    
    Group(Span, Vec<AstNode>), // No new scope
    Block(Span, Vec<AstNode>), // New scope

    Application(Span, Box<AstNode>, Vec<AstNode>),
    
    Declaration(Span, String, Box<AstNode>), // Creates new variables
    Assignment(Span, String, Box<AstNode>),  // Sets values, error on undeclared
    
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
            AstNode::Declaration(_, name, value) => write!(f, "(var {} {})", name, value),
            AstNode::Assignment(_, name, value) => write!(f, "(= {} {})", name, value),

            AstNode::Group(_, nodes) => {
                write!(f, "(group")?;
                for node in nodes {
                    write!(f, " {}", node)?;
                }
                write!(f, ")")?;

                std::fmt::Result::Ok(())
            }

            AstNode::Block(_, nodes) => {
                write!(f, "{{")?;
                let mut first = true;
                for node in nodes {
                    if first {                    
                        write!(f, "{}", node)?;
                        first = false;
                    } else {
                        write!(f, " {}", node)?;
                    }
                }
                write!(f, "}}")?;

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
            | AstNode::Block(span, _)
            | AstNode::Application(span, _, _)
            | AstNode::Declaration(span, _, _)
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

            let node = self.parse_declaration()?;
            span = span.merge(&node.span());
            nodes.push(node);
        }

        Ok(AstNode::Program(span, nodes))
    }

    fn parse_declaration(&mut self) -> Result<AstNode> {
        log::debug!("parse_declaration");

        match self.tokenizer.peek() {
            Some(Token::Keyword(_, Keyword::Var)) => self.parse_var_statement(),
            _ => self.parse_statement(),
        }
    }

    fn parse_statement(&mut self) -> Result<AstNode> {
        log::debug!("parse_statement");

        match self.tokenizer.peek() {
            Some(Token::Keyword(_, Keyword::LeftBrace)) => self.parse_block(),
            Some(Token::Keyword(_, Keyword::Print)) => self.parse_print_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_block(&mut self) -> Result<AstNode> {
        let left_brace = self.tokenizer.next().unwrap();
        let span = left_brace.span();
        log::debug!("parse_block @ {span:?}");

        let mut nodes = vec![];
        while let Some(token) = self.tokenizer.peek() {
            if let Token::Keyword(_, Keyword::RightBrace) = token {
                break;
            }

            let node = self.parse_declaration()?;
            nodes.push(node);
        }

        let right_brace = self.tokenizer.next().unwrap();
        let span = span.merge(&right_brace.span());

        Ok(AstNode::Block(span, nodes))
    }

    fn parse_expression_statement(&mut self) -> Result<AstNode> {
        let expression = self.parse_expression()?;
        let mut span = expression.span();

        let semicolon = self.consume_semicolon_or_eof()?;
        span = span.merge(&semicolon.span());

        Ok(expression)
    }

    fn parse_print_statement(&mut self) -> Result<AstNode> {
        let keyword = self.tokenizer.next().unwrap();
        let span = keyword.span();
        log::debug!("parse_print @ {span:?}");

        let expression = self.parse_expression()?;
        let span = span.merge(&expression.span());

        let semicolon = self.consume_semicolon_or_eof()?;
        let span = span.merge(&semicolon.span());


        Ok(AstNode::Application(
            span,
            Box::new(AstNode::Symbol(span, "print".to_string())),
            vec![expression],
        ))
    }

    fn parse_var_statement(&mut self) -> Result<AstNode> {
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

        // We want to have '= expr ;' or ';'
        match self.tokenizer.next() {
            // End of expression, default to nil and return immediately
            Some(Token::Keyword(semispan, Keyword::Semicolon)) => {
                let span = span.merge(&semispan);
                Ok(AstNode::Declaration(span, name, Box::new(AstNode::Literal(span, Value::Nil))))
            }
            // Equal, parse expression
            Some(Token::Keyword(_, Keyword::Equal)) => {
                let expression = self.parse_expression()?;
                let span = span.merge(&expression.span());

                let semicolon = self.consume_semicolon_or_eof()?;
                let span = span.merge(&semicolon.span());

                Ok(AstNode::Declaration(span, name, Box::new(expression)))
            }
            // Anything else is an error, split for better reporting
            Some(token) => {
                let line = self.line_number(token.span().start);
                Err(anyhow!("[line {}] Error at '{}': Expect '=' or ';'", line, var_keyword))
            }
            None => {
                let line = self.line_number(span.start);
                Err(anyhow!("[line {}] Error at '{}': Expect '=' or ';'", line, var_keyword))
            }
        }
    }


    fn parse_expression(&mut self) -> Result<AstNode> {
        log::debug!("parse_expression");
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<AstNode> {
        let mut lhs = self.parse_equality()?;

        if let Some(Token::Keyword(_, Keyword::Equal)) = self.tokenizer.peek() {
            log::debug!("parse_assignment");

            // The lhs has to be a symbol to assign to
            // Evaluation will handle assignment to undefined variables
            let name = if let AstNode::Symbol(_, name) = &lhs {
                name.clone()
            } else {
                let line = lhs.span().line;
                return Err(anyhow!("[line {}] Error at '=': Invalid assignment target.", line));
            };

            self.tokenizer.next();
            let rhs = self.parse_assignment()?;
            let span = lhs.span().merge(&rhs.span());

            lhs = AstNode::Assignment(span, name, Box::new(rhs));
        }

        Ok(lhs)
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

    fn consume_semicolon_or_eof(&mut self) -> Result<Token> {
        match self.tokenizer.peek() {
            Some(Token::Keyword(_, Keyword::Semicolon)) => {
                Ok(self.tokenizer.next().unwrap())
            }
            Some(Token::EOF) => {
                Ok(Token::EOF)
            }
            
            Some(token) => {
                let line = token.span().line;
                Err(anyhow!("[line {}] Error: Expect ';'", line))
            }

            _ => unreachable!("EOF should be handled above"),
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
