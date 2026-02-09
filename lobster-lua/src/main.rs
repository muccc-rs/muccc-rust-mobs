// decisions:
// our lua starts at 0
use std::fs::read_to_string;

use crate::parser::LobsterParser;
use crate::tokenizer::{Token, Tokenizer};

mod tokenizer;
mod parser;

fn main() {
    let source = read_to_string("sample.lua").expect("todo");

    let mut tokenizer = Tokenizer::new(source.clone());
    loop {
        let (token, _pos) = tokenizer.next_token().expect("TODO");
        dbg!(&token);
        if token == Token::EOF {
            break;
        }
    }

    let mut parser = LobsterParser::new(source);
    dbg!(parser.parse());
}
