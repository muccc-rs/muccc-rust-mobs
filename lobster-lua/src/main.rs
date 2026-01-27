// decisions:
// our lua starts at 0
use std::fs::{read, read_to_string};
fn main() {
    let source = read_to_string("sample.lua").expect("todo");
    let mut tokenizer = Tokenizer::new(source);
    while let token = tokenizer.next_token() {
        dbg!(token);
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    EOF,
    Keyword(Keyword),
    StringLiteral(String),
    NumberLiteral(Number),
    Ident(String),
    ParOpen,
    ParClose,
    SqParOpen,
    SqParClose,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Hash,
    Ampersand,
    Tilde,
    Pipe,
    LShift,
    RShift,
    DoubleSlash,
    DoubleEqualsSign,
    TildeEqualsSign,
    LtEqual,
    GtEqual,
    Lt,
    Gt,
    Equals,
    BraceOpen,
    BraceClose,
    DoubleColon,
    Semicolon,
    Colon,
    Comma,
    Dot,
    DoubleDot,
    TripleDot,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Float(f32),
    Integer(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    // Variables
    Local,

    // Control Flow
    Break,
    Do,
    If,
    ElseIf,
    Else,
    End,
    For,
    In,
    Return,
    While,
    Until,
    Then,
    Repeat,
    Function,
    Goto,

    // Literals
    False,
    True,
    Nil,

    // Operators
    Not,
    Or,
    And,
}

struct Tokenizer {
    source: String,
    pos: usize,
}

impl Tokenizer {
    fn new(source: String) -> Self {
        Self { source, pos: 0 }
    }

    fn remaining(&self) -> &str {
        &self.source[self.pos..]
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.remaining().chars().next() {
            if c.is_whitespace() {
                self.pos += c.len_utf8();
            } else if self.remaining().starts_with("--") {
                self.pos += 2;
                if self.multiline_string().is_none() {
                    if let Some(idx) = self.remaining().find("\n") {
                        self.pos += idx;
                    } else {
                        self.pos += self.remaining().len()
                    }
                }
            } else {
                break;
            }
        }
    }

    /*
    [[
    this is a string
    ]]
    [====[
    this is a string
    ]====]
     */
    fn multiline_string(&mut self) -> Option<String> {
        let mut chars = self.remaining().chars();
        if chars.next() != Some('[') {
            return None;
        };

        chars.skip_while(|x| *x == '=');
        if chars.next() != Some('[') {
            return None;
        };

        chars.skip_while(|x| *x != ']');

        let mut endmarker = String::new();
        endmarker.push(']');
        endmarker.push_str(&eqs);
        endmarker.push(']');

        let end = self.remaining().find(&endmarker).expect("should have an end");

        self.pos += (end + endmarker.len());
        Some("dummy!!!!!".to_owned())
    }

    fn check_for_identifier(&mut self) -> Option<String> {
        if let Some(c) = self.remaining().chars().next() {
            if !c.is_alphabetic() {
                return None;
            }
            let last_idx = self
                .remaining()
                .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                .expect("todo");
            let result = Some(self.remaining()[..last_idx].to_string());
            self.pos += last_idx;
            result
        } else {
            None
        }
    }

    fn next_token(&mut self) -> Result<(Token, usize), String> {
        self.skip_ws();

        if self.pos == self.source.len() {
            return Ok((Token::EOF, self.pos));
        }

        let start_pos = self.pos;
        if let Some(identifier) = self.check_for_identifier() {
            return Ok((Token::Ident(identifier), start_pos));
        }

        let mapping = [("(", Token::ParOpen), (")", Token::ParClose)];

        for (tok, tok2) in mapping.iter() {
            if self.remaining().starts_with(">>") {}
        }

        let token = match self.remaining().chars().next().unwrap() {
            '(' => Some(Token::ParOpen),
            ')' => Some(Token::ParClose),
            _ => None,
        };
        if let Some(token) = token {
            return Ok((token, start_pos));
        }

        Err("Unhappy?".to_owned())
    }
}
