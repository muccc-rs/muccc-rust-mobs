use crate::tokenizer::{Token, Tokenizer, Keyword};

#[derive(Debug)]
pub enum Stmt {
    Break,
    Return(Vec<Expr>),
    While {
        cond: Expr,
        body: Vec<Stmt>,
    }
}

#[derive(Debug)]
pub enum Expr {
    Nil,
}

#[derive(Debug)]
pub struct LobsterParser {
    tokenizer: Tokenizer,
    current_tok: Token,
    current_pos: usize,
}

impl LobsterParser {
    pub fn new(source: String) -> Self {
        let mut tokenizer = Tokenizer::new(source);
        let (current_tok, current_pos) = tokenizer.next_token().expect("TODO");
        Self {
            current_tok,
            current_pos,
            tokenizer,
        }
    }

    pub fn parse(mut self) -> Vec<Stmt> {
        let res = self.parse_block();
        assert_eq!(self.current_tok, Token::EOF, "TODO");
        res
    }

    fn advance(&mut self) {
        let (current_tok, current_pos) = self.tokenizer.next_token().expect("TODO");
        self.current_pos = current_pos;
        self.current_tok = current_tok;
    }

    fn parse_block(&mut self) -> Vec<Stmt> {
        let mut stmt_list = vec![];
        while let Some(next_stmt) = self.parse_stmt() {
            stmt_list.push(next_stmt);
        }
        stmt_list
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match &self.current_tok {
            Token::Keyword(Keyword::Break) => {
                self.advance();
                Some(Stmt::Break)
            },
            Token::Keyword(Keyword::While) => {
                self.advance();
                let cond = self.parse_expr();
                self.expect(&Token::Keyword(Keyword::Do));
                let body = self.parse_block();
                self.expect(&Token::Keyword(Keyword::End));
                Some(Stmt::While { cond, body })
            }
            Token::Keyword(Keyword::Return) => {
                self.advance();
                let mut values = vec![self.parse_expr()];
                while self.current_tok == Token::Comma {
                    self.advance();
                    values.push(self.parse_expr());
                }
                if self.current_tok == Token::Semicolon {
                    self.advance();
                }
                Some(Stmt::Return(values))
            }
            _ => None,
        }
    }

    fn expect(&mut self, tok: &Token) {
        if &self.current_tok == tok {
            self.advance();
        } else {
            panic!("Expected {tok:?}, got {:?}", self.current_tok);
        }
    }

    fn parse_expr(&mut self) -> Expr {
        match &self.current_tok {
            Token::Keyword(Keyword::Nil) => {
                self.advance();
                Expr::Nil
            }
            _ => todo!(),
        }
    }
}
