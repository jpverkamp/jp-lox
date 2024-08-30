use convert_case::{Case, Casing};
use derive_more::Display;

use crate::const_enum;

#[derive(Debug, Display, Clone, PartialEq)]
pub enum Token {
    EOF,
    CharToken(CharToken),
    Keyword(Keyword),
    String(String),

    #[display("{}", _1)]
    Number(String, f64),

    Identifier(String),
}

// Code crafters requires a very specific output format, implement it here
impl Token {
    pub fn code_crafters_format(&self) -> String {
        match self {
            Token::EOF => "EOF  null".to_string(),
            Token::CharToken(char_token) => {
                let name = char_token.to_string().to_case(Case::ScreamingSnake);
                let lexeme = char_token.to_value();

                format!("{name} {lexeme} null")
            },
            Token::Keyword(keyword) => {
                let name = keyword.to_string().to_case(Case::ScreamingSnake);
                let lexeme = keyword.to_value();

                format!("{name} {lexeme} null")
            },
            Token::String(value) => {
                format!("STRING \"{value}\" {value}")
            },
            Token::Number(lexeme, value) => {
                // Integers always print with .0 for reasons
                if value.fract() == 0.0 {
                    format!("NUMBER {lexeme} {value}.0")
                } else {
                    format!("NUMBER {lexeme} {value}")
                }
            },
            Token::Identifier(name) => {
                format!("IDENTIFIER {name} null")
            },
        }
    }
}

// Use a marco (in const_enum.rs) to define an enum with to/from char values
const_enum!{
    pub CharToken as char {
        LeftParen => '(',
        RightParen => ')',
        LeftBrace => '{',
        RightBrace => '}',
        Comma => ',',
        Dot => '.',
        Semicolon => ';',
        Plus => '+',
        Minus => '-',
        Star => '*',
        Slash => '/',
        Equal => '=',
        Bang => '!',
        Less => '<',
        Greater => '>',
    }
}

// Define keywords which are based on strings
const_enum!{
    pub Keyword as &str {
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
    }
}

// The current state of the tokenizer, use it as an iterator (in general)
pub struct Tokenizer<'a> {
    // Internal state stored as raw bytes 
    // TODO: Do we actually need to keep this? I thought it might be handy for error messages
    #[allow(dead_code)]
    source: &'a str,
    byte_pos: usize,

    // Internal state stored as utf8 characters, processed once
    chars: Vec<char>,
    char_pos: usize,

    // The current position of the iterator in the source code
    line: usize,

    // Flag that the iterator has already emitted EOF, so should not iterate any more
    emitted_eof: bool,

    // Flag that we encountered and emitted at least one error
    encountered_error: bool,
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
            encountered_error: false,
        }
    }
}

impl Tokenizer<'_> {
    pub fn encountered_error(&self) -> bool {
        self.encountered_error
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // We've already consumed the iterator
        if self.emitted_eof {
            return None;
        }

        // We've reached the end of the source
        if self.char_pos >= self.chars.len() {
            self.emitted_eof = true;
            return Some(Token::EOF);
        }
        
        // Try to match comments, from // to EOL
        if self.source[self.byte_pos..].starts_with("//") {
            while self.char_pos < self.chars.len() && self.chars[self.char_pos] != '\n' {
                self.char_pos += 1;
                self.byte_pos += 1;
            }

            return self.next();
        }

        // Read strings, currently there is no escaping, so read until a matching " or EOL
        // If we reach EOL, report an error and continue on the next line
        if self.chars[self.char_pos] == '"' {
            let mut value = String::new();
            self.char_pos += 1;
            self.byte_pos += 1;

            loop {
                if self.char_pos >= self.chars.len() || self.chars[self.char_pos] == '\n' {
                    self.encountered_error = true;
                    eprintln!("[line {}] Error: Unterminated string.", self.line);
                    return self.next();
                }

                if self.chars[self.char_pos] == '"' {
                    break;
                }

                value.push(self.chars[self.char_pos]);
                self.char_pos += 1;
                self.byte_pos += 1;
            }

            // Consume closing "
            self.char_pos += 1;
            self.byte_pos += 1;

            return Some(Token::String(value));
        }

        // Read numbers
        // Numbers must start with a digit (cannot do .1)
        // Numbers can contain a single . (cannot do 1.2.3)
        // Numbers must have a digit after the . (cannot do 1. That's two tokens)
        if self.chars[self.char_pos].is_digit(10) {
            let mut lexeme = String::new();
            let mut has_dot = false;
            let mut last_dot = false;

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
            return Some(Token::Number(lexeme, value));
        }

        // Match identifiers
        // Identifiers start with a letter or _
        // Identifiers can contain letters, numbers, and _
        if self.chars[self.char_pos].is_alphabetic() || self.chars[self.char_pos] == '_' {
            let mut value = String::new();

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

            // Check if it's actually a keyword
            // This is called 'maximal munch', so superduper doesn't get parsed as <super><duper>
            if let Ok(keyword) = Keyword::try_from(value.as_str()) {
                return Some(Token::Keyword(keyword));
            } else {
                return Some(Token::Identifier(value));
            }
        }

        // Try to match keywords first
        // See, I knew there was a reason I was keeping around the raw bytes
        // TODO: This seems *really* weird
        for keyword in Keyword::values() {
            let pattern = keyword.to_value();

            if self.source[self.byte_pos..].starts_with(pattern) {
                self.byte_pos += pattern.len();
                self.char_pos += pattern.chars().count();

                return Some(Token::Keyword(keyword));
            }
        }

        // Consume the next character
        let c = self.chars[self.char_pos];
        self.char_pos += 1;
        self.byte_pos += c.len_utf8();

        // Match the character to a token
        if let Ok(token) = CharToken::try_from(c) {
            return Some(Token::CharToken(token));
        }

        // Newlines don't emit a token, but '\n' does increment the line number
        if c.is_whitespace() {
            if c == '\n' {
                self.line += 1;
            }
            return self.next();
        }

        // Anything else should emit an error and continue as best we can
        self.encountered_error = true;
        eprintln!("[line {}] Error: Unexpected character: {}", self.line, c);
        self.next()
    }
}