use convert_case::{Case, Casing};
use derive_more::Display;
use thiserror::Error;

use crate::const_enum;
use crate::span::Span;
use crate::values::Value;

#[derive(Debug, Display, Clone, PartialEq)]
pub enum Token {
    EOF,

    #[display("{}", _1)]
    Keyword(Span, Keyword),

    #[display("{}", _2)]
    Literal(Span, String, Value),

    #[display("{}", _1)]
    Identifier(Span, String),
}

#[derive(Clone, Debug, Error)]
pub enum TokenizerError {
    #[error("[line {}] Unexpected character: {}", .0.line, .1)]
    UnexpectedCharacter(Span, char),

    #[error("[line {}] Unterminated string", .0.line)]
    UnterminatedString(Span),
}

// Code crafters requires a very specific output format, implement it here
impl Token {
    pub fn code_crafters_format(&self) -> String {
        match self {
            Token::EOF => "EOF  null".to_string(),
            Token::Keyword(_, keyword) => {
                let name = keyword.to_string().to_case(Case::ScreamingSnake);
                let lexeme = keyword.to_value();

                format!("{name} {lexeme} null")
            }
            Token::Literal(_, lexeme, value) => {
                let name = match value {
                    Value::Nil => {
                        return "NIL nil null".to_string();
                    }
                    Value::Bool(_) => {
                        let name = value.to_string().to_case(Case::ScreamingSnake);
                        return format!("{name} {value} null");
                    }
                    Value::Number(_) => "NUMBER",
                    Value::String(_) => "STRING",
                    Value::Builtin(_) => "BUILTIN",
                };
                format!("{name} {lexeme} {value}")
            }
            Token::Identifier(_, name) => {
                format!("IDENTIFIER {name} null")
            }
        }
    }
}

impl Token {
    #[allow(dead_code)]
    pub fn span(&self) -> &Span {
        match self {
            Token::EOF => &Span::ZERO,

            Token::Keyword(span, _) | Token::Literal(span, _, _) | Token::Identifier(span, _) => {
                span
            }
        }
    }
}

// Define keywords which are based on strings
const_enum! {
    pub Keyword as &str {
        // Match these first to avoid partial matches (ex == vs =)
        EqualEqual => "==",
        BangEqual => "!=",
        LessEqual => "<=",
        GreaterEqual => ">=",

        And => "and",
        Class => "class",
        Else => "else",
        False => "false",
        For => "for",
        Fun => "fun",
        If => "if",
        Nil => "nil",
        Or => "or",
        Print => "print",
        Return => "return",
        Super => "super",
        This => "this",
        True => "true",
        Var => "var",
        While => "while",

        LeftParen => "(",
        RightParen => ")",
        LeftBrace => "{",
        RightBrace => "}",
        Comma => ",",
        Dot => ".",
        Semicolon => ";",
        Plus => "+",
        Minus => "-",
        Star => "*",
        Slash => "/",
        Equal => "=",
        Bang => "!",
        Less => "<",
        Greater => ">",
    }
}

// The current state of the tokenizer, use it as an iterator (in general)
#[derive(Debug)]
pub struct Tokenizer<'a> {
    // Internal state stored as raw bytes
    pub(crate) source: &'a str,
    byte_pos: usize,

    // Internal state stored as utf8 characters, processed once
    chars: Vec<char>,
    char_pos: usize,

    // The current position of the iterator in the source code
    line: usize,

    // Flag that the iterator has already emitted EOF, so should not iterate any more
    emitted_eof: bool,

    // Collect tokenizer errors this tokenizer has encountered.
    errors: Vec<TokenizerError>,

    // The currently peeked token
    peeked: Option<Token>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            byte_pos: 0,

            chars: source.chars().collect(),
            char_pos: 0,

            line: 1,

            emitted_eof: false,
            errors: Vec::new(),

            peeked: None,
        }
    }
}

impl Tokenizer<'_> {
    pub fn had_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn iter_errors(&self) -> impl Iterator<Item = &TokenizerError> {
        self.errors.iter()
    }

    pub fn peek(&mut self) -> Option<&Token> {
        if self.peeked.is_none() {
            self.peeked = self.next();
        }

        self.peeked.as_ref()
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // We've already consumed the iterator
        if self.emitted_eof {
            return None;
        }

        // If we have a peeked token, clear and return it
        if let Some(token) = self.peeked.take() {
            log::debug!("Clearing peeked token: {}", token);
            self.peeked = None;
            return Some(token);
        }

        // We've reached the end of the source
        if self.char_pos >= self.chars.len() {
            log::debug!("Reached EOF");

            self.emitted_eof = true;
            return Some(Token::EOF);
        }

        // Try to match comments, from // to EOL
        if self.char_pos < self.chars.len() - 1
            && self.chars[self.char_pos] == '/'
            && self.chars[self.char_pos + 1] == '/'
        {
            log::debug!("Matching comment");

            while self.char_pos < self.chars.len() && self.chars[self.char_pos] != '\n' {
                self.char_pos += 1;
                self.byte_pos += 1;
            }

            return self.next();
        }

        // Read strings, currently there is no escaping, so read until a matching " or EOL
        // If we reach EOL, report an error and continue on the next line
        if self.chars[self.char_pos] == '"' {
            log::debug!("Matching string");

            let mut value = String::new();
            let start = self.char_pos;
            self.char_pos += 1;
            self.byte_pos += 1;

            loop {
                if self.char_pos >= self.chars.len() {
                    let error_span = Span {
                        line: self.line,
                        start,
                        end: self.char_pos,
                    };
                    self.errors
                        .push(TokenizerError::UnterminatedString(error_span));
                    return self.next();
                }

                if self.chars[self.char_pos] == '"' {
                    break;
                }

                if self.chars[self.char_pos] == '\n' {
                    self.line += 1
                }

                let c = self.chars[self.char_pos];
                value.push(c);

                self.byte_pos += c.len_utf8();
                self.char_pos += 1;
            }

            // Consume closing "
            self.char_pos += 1;
            self.byte_pos += 1;
            let end = self.char_pos;

            return Some(Token::Literal(
                Span {
                    line: self.line,
                    start,
                    end,
                },
                format!("\"{value}\""),
                Value::String(value),
            ));
        }

        // Read numbers
        // Numbers must start with a digit (cannot do .1)
        // Numbers can contain a single . (cannot do 1.2.3)
        // Numbers must have a digit after the . (cannot do 1. That's two tokens)
        if self.chars[self.char_pos].is_digit(10) {
            log::debug!("Matching number");

            let mut lexeme = String::new();
            let mut has_dot = false;
            let mut last_dot = false;
            let start = self.char_pos;

            while self.char_pos < self.chars.len() {
                let c = self.chars[self.char_pos];

                if c.is_digit(10) {
                    lexeme.push(c);
                    last_dot = false;
                } else if c == '.' && !has_dot {
                    lexeme.push(c);
                    has_dot = true;
                    last_dot = true;
                } else {
                    break;
                }

                self.char_pos += 1;
                self.byte_pos += 1;
            }

            // If the last character was a dot, we need to back up
            if last_dot {
                lexeme.pop();
                self.char_pos -= 1;
                self.byte_pos -= 1;
            }

            let value: f64 = lexeme.parse().unwrap();
            let end = self.char_pos;

            return Some(Token::Literal(
                Span {
                    line: self.line,
                    start,
                    end,
                },
                lexeme,
                Value::Number(value),
            ));
        }

        // Read constant values
        for (lexeme, value) in Value::CONSTANT_VALUES.iter() {
            let lexeme_chars = lexeme.chars().collect::<Vec<_>>();
            if self.chars[self.char_pos..].starts_with(&lexeme_chars) {
                log::debug!("Matching constant: {}", lexeme);

                let start = self.char_pos;
                self.char_pos += lexeme.len();
                self.byte_pos += lexeme.len();
                let end = self.char_pos;
                return Some(Token::Literal(
                    Span {
                        line: self.line,
                        start,
                        end,
                    },
                    lexeme.to_string(),
                    value.clone(),
                ));
            }
        }

        // Match identifiers
        // Identifiers start with a letter or _
        // Identifiers can contain letters, numbers, and _
        if self.chars[self.char_pos].is_alphabetic() || self.chars[self.char_pos] == '_' {
            log::debug!("Matching identifier");

            let mut value = String::new();
            let start = self.char_pos;

            while self.char_pos < self.chars.len() {
                let c = self.chars[self.char_pos];

                if c.is_alphanumeric() || c == '_' {
                    value.push(c);
                } else {
                    break;
                }

                self.char_pos += 1;
                self.byte_pos += 1;
            }

            let end = self.char_pos;

            // Check if it's actually a keyword
            // This is called 'maximal munch', so superduper doesn't get parsed as <super><duper>
            if let Ok(keyword) = Keyword::try_from(value.as_str()) {
                return Some(Token::Keyword(
                    Span {
                        line: self.line,
                        start,
                        end,
                    },
                    keyword,
                ));
            } else {
                return Some(Token::Identifier(
                    Span {
                        line: self.line,
                        start,
                        end,
                    },
                    value,
                ));
            }
        }

        // Match remaining keywords, this will include ones that are symbolic
        for keyword in Keyword::values() {
            let pattern = keyword.to_value();
            let pattern_chars = pattern.chars().collect::<Vec<_>>();

            if self.chars[self.char_pos..].starts_with(&pattern_chars) {
                log::debug!("Matching keyword: {}", keyword);

                let start = self.char_pos;
                self.byte_pos += pattern.len();
                self.char_pos += pattern_chars.len();
                let end = self.char_pos;

                return Some(Token::Keyword(
                    Span {
                        line: self.line,
                        start,
                        end,
                    },
                    keyword,
                ));
            }
        }

        // The only things that should be left are whitespace
        // Anything else is an error
        let c = self.chars[self.char_pos];
        self.char_pos += 1;
        self.byte_pos += c.len_utf8();

        // Newlines don't emit a token, but '\n' does increment the line number
        if c.is_whitespace() {
            if c == '\n' {
                self.line += 1;
            }
            return self.next();
        }

        // Anything else should emit an error and continue as best we can
        self.errors.push(TokenizerError::UnexpectedCharacter(
            Span {
                line: self.line,
                start: self.char_pos,
                end: self.char_pos,
            },
            c,
        ));
        self.next()
    }
}
