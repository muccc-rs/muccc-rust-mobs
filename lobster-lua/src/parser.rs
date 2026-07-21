use crate::{
    fraction::Fraction,
    tokenizer::{Keyword, Token, Tokenizer},
};

#[derive(Debug, serde::Serialize, Clone, PartialEq, Eq)]
pub enum Stmt {
    Break,
    Continue,
    Return(Vec<Expr>),
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    DoEnd {
        body: Vec<Stmt>,
    },
    Assignment {
        lhs: LeftExpr,
        rhs: Expr,
        local: bool,
    },
    Expr {
        expr: Expr,
    },
    If {
        cond: Expr,
        then: Vec<Stmt>,
        r#else: Vec<Stmt>,
    },
}

#[derive(Debug, serde::Serialize, Clone, PartialEq, Eq)]
pub enum LeftExpr {
    Var(String),
    TableIndex { table: Box<Expr>, index: Box<Expr> },
}

impl TryFrom<Expr> for LeftExpr {
    type Error = ();

    fn try_from(value: Expr) -> Result<Self, Self::Error> {
        match value {
            Expr::Var(v) => Ok(LeftExpr::Var(v)),
            Expr::TableIndex { table, index } => Ok(LeftExpr::TableIndex { table, index }),
            _ => Err(()),
        }
    }
}

#[derive(Debug, serde::Serialize, Clone, PartialEq, Eq)]
pub enum Expr {
    Nil,
    Numeral(i64),
    Fraction(Fraction),
    Boolean(bool),
    String(String),
    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Var(String),
    FunctionCall {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    // special AST node instead of desugaring to function call to avoid duplicating callee
    MethodCall {
        callee: Box<Expr>,
        method_name: String,
        args: Vec<Expr>,
    },
    FunctionDef {
        arguments: Vec<String>,
        body: Vec<Stmt>,
    },
    Table {
        values: Vec<(Expr, Expr)>,
    },
    TableIndex {
        table: Box<Expr>,
        index: Box<Expr>,
    },
}

impl Expr {
    #[cfg(test)]
    fn to_s_expr(&self) -> String {
        match self {
            Expr::Nil => "nil".to_string(),
            Expr::Numeral(x) => x.to_string(),
            Expr::Fraction(f) => f.to_string(),
            Expr::Boolean(b) => b.to_string(),
            Expr::String(s) => format!("{s:?}"),
            Expr::BinOp { op, lhs, rhs } => format!(
                "({} {} {})",
                op.to_s_expr(),
                lhs.to_s_expr(),
                rhs.to_s_expr()
            ),
            Expr::Var(name) => name.to_string(),
            Expr::FunctionCall {
                callee: function_name,
                args,
            } => format!(
                "(call {} {})",
                function_name.to_s_expr(),
                args.into_iter()
                    .map(|e| e.to_s_expr())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            Expr::MethodCall {
                callee,
                method_name,
                args,
            } => format!(
                "(meth call {} {} {})",
                callee.to_s_expr(),
                method_name,
                args.into_iter()
                    .map(|e| e.to_s_expr())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            Expr::FunctionDef {
                arguments: _,
                body: _,
            } => "(fn () <TODO: body>)".to_string(),
            Expr::Table { values: _ } => "table".to_string(),
            Expr::TableIndex { table, index } => {
                format!("{}[{}]", table.to_s_expr(), index.to_s_expr())
            }
        }
    }
}

#[derive(Debug)]
pub struct LobsterParser {
    tokenizer: Tokenizer,
    current_tok: Token,
    current_pos: usize,
}

#[derive(Debug, serde::Serialize, Copy, Clone, PartialEq, Eq)]
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

    #[cfg(test)]
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

#[derive(Debug)]
pub struct ParserError {
    message: String,
    pos: usize,
}

impl ParserError {
    pub fn render(&self, fname: &str, code: &str) -> String {
        let line_number = code[..self.pos].chars().filter(|&c| c == '\n').count() + 1;

        let line_start = code[..self.pos].rfind('\n').map_or(0, |p| p + 1);
        let line = code[(line_start)..].split('\n').next().unwrap();
        let col = self.pos - line_start + 1; // TODO: assumes ASCII

        format!(
            "{fname}:{line_number}:{col}: {}\n | {line}\n   {:pad$}^",
            self.message,
            "",
            pad = col - 1
        )
    }
}

pub type ParseResult<T> = Result<T, ParserError>;

fn is_block_terminator(tok: &Token) -> bool {
    matches!(
        tok,
        Token::Keyword(Keyword::End)
            | Token::Keyword(Keyword::Else)
            | Token::Keyword(Keyword::ElseIf)
            | Token::Keyword(Keyword::Until)
            | Token::EOF
    )
}

impl LobsterParser {
    pub fn parse(source: String) -> ParseResult<Vec<Stmt>> {
        let mut tokenizer = Tokenizer::new(source);
        let (current_tok, current_pos) = tokenizer.next_token().map_err(|e| ParserError {
            message: e.message,
            pos: e.pos,
        })?; // anchor: yagEGIQ2
        let mut this = Self {
            current_tok,
            current_pos,
            tokenizer,
        };

        let res = this.parse_block()?;
        if this.current_tok == Token::EOF {
            Ok(res)
        } else {
            Err(ParserError {
                message: format!("Expected EOF, found {:?}", this.current_tok),
                pos: this.current_pos,
            })
        }
    }

    fn advance(&mut self) -> ParseResult<()> {
        println!("advance");
        let (current_tok, current_pos) = self.tokenizer.next_token().map_err(|e| ParserError {
            message: e.message,
            pos: e.pos,
        })?; // anchor: yagEGIQ2
        println!("current_tok: {:?}", current_tok);
        self.current_pos = current_pos;
        self.current_tok = current_tok;
        Ok(())
    }

    fn parse_block(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut stmt_list = vec![];
        loop {
            if let Token::Keyword(Keyword::Return) = self.current_tok {
                self.advance()?;
                let e = match self.parse_expr()? {
                    None => {
                        stmt_list.push(Stmt::Return(vec![]));
                        break;
                    }
                    Some(e) => e,
                };
                let mut values = vec![e];
                while self.current_tok == Token::Comma {
                    self.advance()?;
                    values.push(self.parse_expr()?.expect("todo"));
                }
                if self.current_tok == Token::Semicolon {
                    self.advance()?;
                }
                stmt_list.push(Stmt::Return(values));
                break;
            }
            if is_block_terminator(&self.current_tok) {
                break;
            }
            stmt_list.push(self.parse_stmt()?);
        }
        Ok(stmt_list)
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

    fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        match &self.current_tok {
            Token::Keyword(Keyword::Function) => {
                self.advance()?;
                let Token::Ident(function_name) = self.current_tok.clone() else {
                    return Err(ParserError {
                        message: "Function name must be an identifier!".to_string(),
                        pos: self.current_pos,
                    });
                };
                self.advance()?;
                self.expect(&Token::ParOpen)?;
                let mut arguments = vec![];
                if self.current_tok != Token::ParClose {
                    arguments.push(self.parse_argument()?);
                    loop {
                        if self.current_tok == Token::ParClose {
                            break;
                        }
                        self.expect(&Token::Comma)?;
                        arguments.push(self.parse_argument()?);
                    }
                }
                self.expect(&Token::ParClose)?;
                let body = self.parse_block().expect("TODO");
                self.expect(&Token::Keyword(Keyword::End))?;
                Ok(Stmt::Assignment {
                    lhs: LeftExpr::Var(function_name),
                    rhs: Expr::FunctionDef { arguments, body },
                    local: false,
                })
            }
            Token::Keyword(Keyword::Break) => {
                self.advance()?;
                Ok(Stmt::Break)
            }
            Token::Keyword(Keyword::Continue) => {
                self.advance()?;
                Ok(Stmt::Continue)
            }
            Token::Keyword(Keyword::While) => {
                self.advance()?;
                let cond = self.parse_expr()?.expect("todo");
                self.expect(&Token::Keyword(Keyword::Do))?;
                let body = self.parse_block().expect("TODO");
                self.expect(&Token::Keyword(Keyword::End))?;
                Ok(Stmt::While { cond, body })
            }
            //Do End
            Token::Keyword(Keyword::Do) => {
                self.advance()?;
                let block = self.parse_block().expect("TODO");
                self.expect(&Token::Keyword(Keyword::End))?;
                Ok(Stmt::DoEnd { body: block })
            }
            Token::Keyword(Keyword::If) => {
                self.advance()?;
                let cond = self.parse_expr()?.expect("todo");
                self.expect(&Token::Keyword(Keyword::Then))?;
                let then = self.parse_block().expect("TODO");

                let mut whole = Stmt::If {
                    cond,
                    then,
                    r#else: vec![],
                };
                let Stmt::If {
                    r#else: else_placeholder,
                    ..
                } = &mut whole
                else {
                    unreachable!()
                };
                let mut else_placeholder = else_placeholder;

                while self.current_tok == Token::Keyword(Keyword::ElseIf) {
                    self.advance()?;
                    let cond = self.parse_expr()?.expect("todo");
                    self.expect(&Token::Keyword(Keyword::Then))?;
                    let then = self.parse_block().expect("TODO");

                    *else_placeholder = vec![Stmt::If {
                        cond,
                        then,
                        r#else: vec![],
                    }];
                    let [
                        Stmt::If {
                            r#else: else_placeholder2,
                            ..
                        },
                    ] = &mut else_placeholder[..]
                    else {
                        unreachable!()
                    };
                    else_placeholder = else_placeholder2;
                }

                match self.current_tok {
                    Token::Keyword(Keyword::End) => {
                        self.advance()?;
                    }
                    Token::Keyword(Keyword::Else) => {
                        self.advance()?;
                        let else_block = self.parse_block().expect("TODO");
                        self.expect(&Token::Keyword(Keyword::End))?;
                        *else_placeholder = else_block;
                    }
                    _ => todo!(),
                }
                Ok(whole)
            }
            Token::Keyword(Keyword::Local) => {
                self.advance()?;
                let Token::Ident(ident) = self.current_tok.clone() else {
                    return Err(ParserError {
                        message: "expected variable name".to_string(),
                        pos: self.current_pos,
                    });
                };
                self.advance()?;
                self.expect(&Token::Equals)?;
                let variable = ident;
                let value = self.parse_expr()?.expect("todo");
                Ok(Stmt::Assignment {
                    lhs: LeftExpr::Var(variable),
                    rhs: value,
                    local: true,
                })
            }
            Token::Ident(_ident) => {
                let e = self.parse_expr()?.expect("TODO");
                match self.current_tok {
                    Token::Equals => {
                        self.advance()?;
                        let value = self.parse_expr()?.expect("todo");
                        Ok(Stmt::Assignment {
                            lhs: e.try_into().expect(
                                "left-hand side of assignment must be a variable or table index",
                            ),
                            rhs: value,
                            local: false,
                        })
                    }
                    Token::Comma => {
                        todo!("a,b = c,d");
                    }
                    _ => match e {
                        Expr::FunctionCall { .. } => Ok(Stmt::Expr { expr: e }),
                        Expr::MethodCall { .. } => Ok(Stmt::Expr { expr: e }),
                        _ => todo!("only function calls can be statement-level expressions"),
                    },
                }
                /*let ident = ident.clone();
                self.advance()?;
                match self.current_tok {
                    Token::Equals => {
                        self.advance()?;
                        let variable = ident;
                        let value = self.parse_expr()?.expect("todo");
                        Some(Stmt::Assignment {
                            variable,
                            value,
                            local: false,
                        })
                        // Assignment
                    }
                    Token::ParOpen => {
                        self.advance()?;
                        let mut args = vec![];
                        while let Some(arg) = self.parse_expr() {
                            args.push(arg);
                            if self.current_tok == Token::Comma {
                                self.advance()?;
                            } else {
                                break;
                            }
                        }
                        self.expect(&Token::ParClose)?;
                        Some(Stmt::FunctionCall {
                            function_name: ident,
                            args,
                        })

                        // Function call
                    }
                    _ => panic!("unexpected token"),
                }*/
            }
            tok => Err(ParserError {
                message: format!("weird statement start: {tok:?}"),
                pos: self.current_pos,
            }),
        }
    }

    #[track_caller]
    fn expect(&mut self, tok: &Token) -> ParseResult<()> {
        if &self.current_tok == tok {
            self.advance()?;
            Ok(())
        } else {
            Err(ParserError {
                message: format!("Expected {tok:?}, got {:?}", self.current_tok),
                pos: self.current_pos,
            })
        }
    }

    #[track_caller]
    fn expect_with_message(&mut self, tok: &Token, message: &str) -> ParseResult<()> {
        if &self.current_tok == tok {
            self.advance()?;
            Ok(())
        } else {
            Err(ParserError {
                message: format!("Expected {message}, got {:?}", self.current_tok),
                pos: self.current_pos,
            })
        }
    }

    fn parse_atomic_expr(&mut self) -> ParseResult<Option<Expr>> {
        let mut expr: Expr = match &self.current_tok {
            Token::ParOpen => {
                self.advance()?;
                let res = self.parse_expr()?.expect("TODO");
                self.expect(&Token::ParClose)?;
                res
            }
            Token::Keyword(Keyword::Nil) => {
                self.advance()?;
                Expr::Nil
            }
            &Token::NumberLiteral(num) => {
                self.advance()?;
                Expr::Numeral(num)
            }
            &Token::FractionLiteral(num) => {
                self.advance()?;
                Expr::Fraction(num)
            }
            &Token::Keyword(Keyword::True) => {
                self.advance()?;
                Expr::Boolean(true)
            }
            &Token::Keyword(Keyword::False) => {
                self.advance()?;
                Expr::Boolean(false)
            }
            Token::StringLiteral(s) => {
                let s = s.clone();
                self.advance()?;
                Expr::String(s)
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.advance()?;
                Expr::Var(name)
            }
            Token::Keyword(Keyword::Function) => {
                self.advance()?;
                self.expect(&Token::ParOpen)?;

                let mut arguments = vec![];
                if self.current_tok != Token::ParClose {
                    arguments.push(self.parse_argument()?);
                    loop {
                        if self.current_tok == Token::ParClose {
                            break;
                        }
                        self.expect(&Token::Comma)?;
                        arguments.push(self.parse_argument()?);
                    }
                }
                self.expect(&Token::ParClose)?;

                let body = match self.parse_block() {
                    Ok(val) => val,
                    Err(err) => return Err(err.into()),
                };
                self.expect(&Token::Keyword(Keyword::End))?;

                Expr::FunctionDef { arguments, body }
            }
            Token::BraceOpen => {
                self.advance()?;
                let mut kvs = vec![];
                let mut cur_idx = 0;
                loop {
                    if self.current_tok == Token::BraceClose {
                        break;
                    }
                    match self.current_tok {
                        Token::SqParOpen => {
                            self.advance()?;
                            let key = self.parse_expr()?.expect("TODO");
                            self.expect(&Token::SqParClose)?;
                            self.expect(&Token::Equals)?;
                            let value = self.parse_expr()?.expect("TODO");
                            kvs.push((key, value));
                        }
                        _ => {
                            let expr_pos = self.current_pos;
                            let expr = self.parse_expr()?.expect("TODO");
                            if self.current_tok == Token::Equals {
                                let Expr::Var(key) = expr else {
                                    return Err(ParserError {
                                        message: "list key must be an identifier".to_string(),
                                        pos: expr_pos,
                                    });
                                };
                                self.advance()?;
                                let value = self.parse_expr()?.expect("TODO");
                                kvs.push((Expr::String(key), value));
                            } else {
                                let key = Expr::Numeral(cur_idx);
                                cur_idx += 1;
                                kvs.push((key, expr));
                            }
                        }
                    }
                    match self.current_tok {
                        Token::Comma => self.advance()?,
                        Token::BraceClose => break,
                        _ => {
                            return Err(ParserError {
                                message: format!(
                                    "Expected Comma or BraceClose, got {:?}",
                                    self.current_tok
                                ),
                                pos: self.current_pos,
                            });
                        }
                    }
                }
                self.expect(&Token::BraceClose)?;
                Expr::Table { values: kvs }
            }
            _ => return Ok(None),
        };

        eprintln!("expr {expr:?} — {:?}", self.current_tok);
        loop {
            match self.current_tok {
                Token::Dot => {
                    self.advance()?;
                    let table = expr;
                    let Token::Ident(index) = self.current_tok.clone() else {
                        panic!("PANIKK");
                    };
                    self.advance()?;
                    expr = Expr::TableIndex {
                        table: Box::new(table),
                        index: Box::new(Expr::String(index)),
                    };
                }
                Token::FractionLiteral(f) if f == Fraction::new(1, 6) => {
                    // colon
                    self.advance()?;
                    let Token::Ident(method_name) = self.current_tok.clone() else {
                        todo!("Expected method name after colon");
                    };
                    self.advance()?;
                    self.expect(&Token::ParOpen)?;

                    // TODO: dedup anchor: cbEnCpYf
                    let mut args = vec![];
                    while let Some(arg) = self.parse_expr()? {
                        args.push(arg);
                        if self.current_tok == Token::Comma {
                            self.advance()?;
                        } else {
                            break;
                        }
                    }
                    self.expect_with_message(&Token::ParClose, "ParClose or Comma")?;

                    expr = Expr::MethodCall {
                        callee: Box::new(expr),
                        method_name,
                        args,
                    };
                }
                Token::ParOpen => {
                    self.advance()?;
                    // function call
                    // TODO: dedup anchor: cbEnCpYf
                    let mut args = vec![];
                    while let Some(arg) = self.parse_expr()? {
                        args.push(arg);
                        if self.current_tok == Token::Comma {
                            self.advance()?;
                        } else {
                            break;
                        }
                    }
                    self.expect_with_message(&Token::ParClose, "ParClose or Comma")?;
                    expr = Expr::FunctionCall {
                        callee: Box::new(expr),
                        args,
                    };
                }
                Token::SqParOpen => {
                    self.advance()?;
                    let table = expr;
                    let index = self.parse_expr()?.expect("TODO");
                    self.expect(&Token::SqParClose)?;
                    expr = Expr::TableIndex {
                        table: Box::new(table),
                        index: Box::new(index),
                    };
                }

                _ => {
                    return Ok(Some(expr));
                }
            }
        }
    }

    fn parse_expr_inner(&mut self, minimum_binding_power: u16) -> ParseResult<Option<Expr>> {
        let Some(mut lhs) = self.parse_atomic_expr()? else {
            return Ok(None);
        };
        while let Some(op) = self.peak_binop() {
            let (l_prec, r_prec) = op.get_precedence();
            assert_ne!(minimum_binding_power, l_prec);
            if l_prec < minimum_binding_power {
                break;
            }
            self.advance()?;
            let rhs = self.parse_expr_inner(r_prec)?.expect("TODO");
            lhs = Expr::BinOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        }
        Ok(Some(lhs))
    }

    fn parse_expr(&mut self) -> ParseResult<Option<Expr>> {
        self.parse_expr_inner(0)
    }

    fn parse_argument(&mut self) -> ParseResult<String> {
        let arg = match &self.current_tok {
            Token::Ident(name) => name.clone(),
            _ => panic!("Expected identifier, got {:?}", self.current_tok),
        };
        self.advance()?;
        Ok(arg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_expr(s: &str, expected: &str) {
        let blocks = LobsterParser::parse(format!("x = {s}")).unwrap();
        let crate::parser::Stmt::Assignment { rhs, .. } = &blocks[0] else {
            panic!("Could not extract expression from assignment");
        };
        assert_eq!(rhs.to_s_expr(), expected, "failed when parsing {s:?}");
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
                LobsterParser::parse($source.to_string()).unwrap();
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

    parse_test!(return_something, "return 42");
    parse_test!(return_nothing, "return");
    parse_test!(return_inside_block, "do break return [[]] end");

    parse_test!(
        break_break_mic_check_do_you_read,
        "if nil then break elseif nil then break break else break break break end"
    );
}
