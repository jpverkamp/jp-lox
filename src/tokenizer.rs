use convert_case::{Case, Casing};
use derive_more::Display;

use crate::const_enum;

#[derive(Debug, Display, Clone, PartialEq, Eq)]
pub enum Token {
    EOF,
    CharToken(CharToken),
    Keyword(Keyword),
    String(String),
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
                let name = "STRING";

                format!("{name} {value:?} {value}")
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
    pub Keyword as &'static str {
        EqualEqual => "==",
        BangEqual => "!=",
        LessEqual => "<=",
        GreaterEqual => ">=",
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