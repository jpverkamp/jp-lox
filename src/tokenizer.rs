use convert_case::{Case, Casing};
use derive_more::Display;

use crate::char_enum;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    EOF,
    CharToken(CharToken),
}

// Code crafters requires a very specific output format, implement it here
impl Token {
    pub fn code_crafters_format(&self) -> String {
        match self {
            Token::EOF => "EOF  null".to_string(),
            Token::CharToken(char_token) => {
                let name = char_token.to_string().to_case(Case::ScreamingSnake);
                let lexeme = char_token.to_char();

                format!("{name} {lexeme} null")
            },
        }
    }
}

// Use a marco (in char_enum.rs) to define an enum with to/from char values
char_enum!{
    pub CharToken {
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

    // Flag that the iterator has already emitted EOF, so should not iterate any more
    emitted_eof: bool,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            byte_pos: 0,
            chars: source.chars().collect(),
            char_pos: 0,
            emitted_eof: false,
        }
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

        // Consume the next character
        let c = self.chars[self.char_pos];
        self.char_pos += 1;
        self.byte_pos += c.len_utf8();

        // Match the character to a token
        if let Ok(token) = CharToken::try_from(c) {
            return Some(Token::CharToken(token));
        }
        
        unimplemented!("Unexpected character: {:?}", c);
    }
}