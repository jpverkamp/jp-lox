use convert_case::{Case, Casing};
use derive_more::Display;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    EOF,
    CharToken(CharToken),
}

impl Token {
    pub fn lox_format(&self) -> String {
        match self {
            Token::EOF => "EOF  null".to_string(),
            Token::CharToken(char_token) => {
                let name = char_token.to_string().to_case(Case::ScreamingSnake);
                let lexeme = char::from_u32(*char_token as u32).unwrap_or('\u{FFFD}');

                format!("{name} {lexeme} null")
            },
        }
    }
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum CharToken {
    LeftParen = '(' as u32,
    RightParen = ')' as u32,
}

pub struct Tokenizer<'a> {
    #[allow(dead_code)]
    source: &'a str,
    byte_pos: usize,

    chars: Vec<char>,
    char_pos: usize,

    eof: bool,
}



impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            byte_pos: 0,
            chars: source.chars().collect(),
            char_pos: 0,
            eof: false,
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // We've already consumed the iterator
        if self.eof {
            return None;
        }

        // We've reached the end of the source
        if self.char_pos >= self.chars.len() {
            self.eof = true;
            return Some(Token::EOF);
        }

        // Consume the next character
        let c = self.chars[self.char_pos];
        self.char_pos += 1;
        self.byte_pos += c.len_utf8();

        // Match the character to a token
        let token = match c {
            '(' => Token::CharToken(CharToken::LeftParen),
            ')' => Token::CharToken(CharToken::RightParen),
            _ => unimplemented!("Unknown token in input file"),
        };
        Some(token)
    }
}