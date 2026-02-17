use crate::tokenizer::{Keyword, Token, Tokenizer};

#[derive(Debug, serde::Serialize)]
pub enum Stmt {
    Break,
    Return(Vec<Expr>),
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    DoEnd {
        body: Vec<Stmt>,
    },
    Assignment {
        variable: String,
        value: Expr,
    },
    FunctionCall {
        function_name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, serde::Serialize)]
pub enum Expr {
    Nil,
    Numeral(i64),
    Boolean(bool),
    String(String),
    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Var(String),
    FunctionCall{function_name: String, args: Vec<Expr>},
}

impl Expr {
    fn to_s_expr(&self) -> String {
        match self {
            Expr::Nil => "nil".to_string(),
            Expr::Numeral(x) => x.to_string(),
            Expr::Boolean(b) => b.to_string(),
            Expr::String(s) => format!("{s:?}"),
            Expr::BinOp { op, lhs, rhs } => format!(
                "({} {} {})",
                op.to_s_expr(),
                lhs.to_s_expr(),
                rhs.to_s_expr()
            ),
            Expr::Var(name) => name.to_string(),
            Expr::FunctionCall { function_name, args } => format!("({} {})", function_name, args.into_iter().map(|e| e.to_s_expr()).collect::<Vec<_>>().join(" "))
        }
    }
}

#[derive(Debug)]
pub struct LobsterParser {
    tokenizer: Tokenizer,
    current_tok: Token,
    current_pos: usize,
}

#[derive(Debug, serde::Serialize)]
pub enum BinOp {
    Plus,
    Minus,
    Mul,
    Div,
    IDiv,
    Exp,
    Mod,
    And,
    Or,
    LShift,
    RShift,
    GT,
    LT,
    GEQ,
    LEQ,
    BitOR,
    BitAnd,
    BitXor,
    Equals,
    NotEquals,
    Concat,
}

impl BinOp {
    fn get_precedence(&self) -> (u16, u16) {
        match self {
            Self::Or => (10, 11),
            Self::And => (20, 21),
            Self::GT => (30, 31),
            Self::LT => (30, 31),
            Self::GEQ => (30, 31),
            Self::LEQ => (30, 31),
            Self::NotEquals => (30, 31),
            Self::Equals => (30, 31),
            Self::BitOR => (40, 41),
            Self::BitXor => (50, 51),
            Self::BitAnd => (60, 61),
            Self::LShift => (70, 71),
            Self::RShift => (70, 71),
            Self::Concat => (80, 79), // right-associative
            Self::Plus => (90, 91),
            Self::Minus => (90, 91),
            Self::Mul => (100, 101),
            Self::Div => (100, 101),
            Self::IDiv => (100, 101),
            Self::Mod => (100, 101),
            Self::Exp => (120, 119), // right-associative
        }
    }

    fn to_s_expr(&self) -> &'static str {
        match self {
            BinOp::Plus => "+",
            BinOp::Minus => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::IDiv => "//",
            BinOp::Exp => "^",
            BinOp::Mod => "%",
            BinOp::And => "and",
            BinOp::Or => "or",
            BinOp::LShift => "<<",
            BinOp::RShift => ">>",
            BinOp::GT => ">",
            BinOp::LT => "<",
            BinOp::GEQ => ">=",
            BinOp::LEQ => "<=",
            BinOp::BitOR => "|",
            BinOp::BitAnd => "&",
            BinOp::BitXor => "~",
            BinOp::Equals => "==",
            BinOp::NotEquals => "~=",
            BinOp::Concat => "..",
        }
    }
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
        println!("advance");
        let (current_tok, current_pos) = self.tokenizer.next_token().expect("TODO");
        println!("current_tok: {:?}", current_tok);
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

    fn peak_binop(&mut self) -> Option<BinOp> {
        match &self.current_tok {
            Token::Plus => Some(BinOp::Plus),
            Token::Minus => Some(BinOp::Minus),
            Token::Star => Some(BinOp::Mul),
            Token::Slash => Some(BinOp::Div),
            Token::DoubleSlash => Some(BinOp::IDiv),
            Token::Caret => Some(BinOp::Exp),
            Token::Percent => Some(BinOp::Mod),
            Token::Keyword(Keyword::And) => Some(BinOp::And),
            Token::Keyword(Keyword::Or) => Some(BinOp::Or),
            Token::LShift => Some(BinOp::LShift),
            Token::RShift => Some(BinOp::RShift),
            Token::Gt => Some(BinOp::GT),
            Token::Lt => Some(BinOp::LT),
            Token::GtEqual => Some(BinOp::GEQ),
            Token::LtEqual => Some(BinOp::LEQ),
            Token::Pipe => Some(BinOp::BitOR),
            Token::Ampersand => Some(BinOp::BitAnd),
            Token::Tilde => Some(BinOp::BitXor),
            Token::DoubleEqualsSign => Some(BinOp::Equals),
            Token::TildeEqualsSign => Some(BinOp::NotEquals),
            Token::DoubleDot => Some(BinOp::Concat),
            _ => None,
        }
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match &self.current_tok {
            Token::Keyword(Keyword::Break) => {
                self.advance();
                Some(Stmt::Break)
            }
            Token::Keyword(Keyword::While) => {
                self.advance();
                let cond = self.parse_expr().expect("todo");
                self.expect(&Token::Keyword(Keyword::Do));
                let body = self.parse_block();
                self.expect(&Token::Keyword(Keyword::End));
                Some(Stmt::While { cond, body })
            }
            //Do End
            Token::Keyword(Keyword::Do) => {
                self.advance();
                let block = self.parse_block();
                self.expect(&Token::Keyword(Keyword::End));
                Some(Stmt::DoEnd { body: block })
            }
            Token::Keyword(Keyword::Return) => {
                self.advance();
                let mut values = vec![self.parse_expr().expect("todo")];
                while self.current_tok == Token::Comma {
                    self.advance();
                    values.push(self.parse_expr().expect("todo"));
                }
                if self.current_tok == Token::Semicolon {
                    self.advance();
                }
                Some(Stmt::Return(values))
            }
            Token::Ident(ident) => {
                let ident = ident.clone();
                self.advance();
                match self.current_tok {
                    Token::Equals => {
                        self.advance();
                        let variable = ident;
                        let value = self.parse_expr().expect("todo");
                        Some(Stmt::Assignment { variable, value })
                        // Assignment
                    }
                    Token::ParOpen => {
                        self.advance();
                        let mut args = vec![];
                        while let Some(arg) = self.parse_expr() {
                            args.push(arg);
                            if self.current_tok == Token::Comma {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        self.expect(&Token::ParClose);
                        Some(Stmt::FunctionCall {
                            function_name: ident,
                            args,
                        })

                        // Function call
                    }
                    _ => panic!("unexpected token"),
                }
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

    fn parse_atomic_expr(&mut self) -> Option<Expr> {
        match &self.current_tok {
            Token::Keyword(Keyword::Nil) => {
                self.advance();
                Some(Expr::Nil)
            }
            &Token::NumberLiteral(num) => {
                self.advance();
                Some(Expr::Numeral(num))
            }
            &Token::Keyword(Keyword::True) => {
                self.advance();
                Some(Expr::Boolean(true))
            }
            &Token::Keyword(Keyword::False) => {
                self.advance();
                Some(Expr::Boolean(false))
            }
            Token::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Some(Expr::String(s))
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                if self.current_tok == Token::ParOpen {
                    self.advance();
                    // function call
                    let mut args = vec![];
                    while let Some(arg) = self.parse_expr() {
                        args.push(arg);
                        if self.current_tok == Token::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect(&Token::ParClose);
                    Some(Expr::FunctionCall{function_name: name, args})
                } else {
                    // variable
                    Some(Expr::Var(name))
                }
            }
            _ => None,
        }
    }

    fn parse_expr_inner(&mut self, minimum_binding_power: u16) -> Option<Expr> {
        let mut lhs = self.parse_atomic_expr()?;
        while let Some(op) = self.peak_binop() {
            let (l_prec, r_prec) = op.get_precedence();
            assert_ne!(minimum_binding_power, l_prec);
            if l_prec < minimum_binding_power {
                break;
            }
            self.advance();
            let rhs = self.parse_expr_inner(r_prec).expect("TODO");
            lhs = Expr::BinOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        }
        Some(lhs)
    }

    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_expr_inner(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_expr(s: &str, expected: &str) {
        let mut parser = LobsterParser::new(s.to_owned());
        let expr = parser.parse_expr().unwrap();
        assert_eq!(expr.to_s_expr(), expected, "failed when parsing {s:?}");
    }

    macro_rules! test_expr {
        // sex = s-expression
        ($name:ident, $source:expr, $sex:expr) => {
            #[test]
            fn $name() {
                check_expr($source, $sex)
            }
        };
    }

    test_expr!(test_expr_numeral, "1", "1");
    test_expr!(test_expr_variable, "x", "x");
    // test_expr!(test_expr_parens, "1 + (2 + 3)", "(+ 1 (+ 2 3))");
    test_expr!(test_expr_precedence, "1 + 2 * 3", "(+ 1 (* 2 3))");
    test_expr!(
        test_expr_right_assoc_exp,
        "123^456^789",
        "(^ 123 (^ 456 789))"
    );

    macro_rules! parse_test {
        ($name:ident, $source:expr) => {
            #[test]
            fn $name() {
                let parser = LobsterParser::new($source.to_string());
                let result = parser.parse();
                insta::assert_yaml_snapshot!(result);
            }
        };
    }

    parse_test!(test_parse_assigment, "foobar = 1");
    parse_test!(test_parse_lobster_emoji_identifier, "🦞 = 1");
    parse_test!(test_parse_func_call, "frobnicate(VIVIVIVIVI, [[Foo Bar]])");
    // Plus binds tighter than `and`: 1 + 2 and 3 => (1 + 2) and 3
    parse_test!(test_precedence_plus_over_and, "x = 1 + 2 and 3");
    // Both sides: 1 + 2 and 3 + 4 => (1 + 2) and (3 + 4)
    parse_test!(
        test_precedence_plus_both_sides_of_and,
        "x = 1 + 2 and 3 + 4"
    );
    // Left associativity of +: 1 + 2 + 3 => (1 + 2) + 3
    parse_test!(test_precedence_plus_left_associative, "x = 1 + 2 + 3");
    // Left associativity of `and`: 1 and 2 and 3 => (1 and 2) and 3
    parse_test!(test_precedence_and_left_associative, "x = 1 and 2 and 3");
}
