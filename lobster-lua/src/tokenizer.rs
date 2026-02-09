
#[expect(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    EOF,
    Keyword(Keyword),
    StringLiteral(String),
    NumberLiteral(i64),
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

#[expect(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
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

#[derive(Debug)]
pub struct Tokenizer {
    source: String,
    pos: usize,
}

impl Tokenizer {
    pub fn new(source: String) -> Self {
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
        let num_eqs = chars.clone().take_while(|&c| c == '=').count();
        if chars.nth(num_eqs) != Some('[') {
            return None;
        };
        let start = self.pos + 1 + num_eqs + 1;

        let mut endmarker = String::new();
        endmarker.push(']');
        endmarker.push_str(&"=".repeat(num_eqs));
        endmarker.push(']');

        let end = self
            .remaining()
            .find(&endmarker)
            .expect("TODO should have an end");

        let content_end = self.pos + end;
        self.pos += end + endmarker.len();
        Some(self.source[start..content_end].to_owned())
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

    fn check_for_number(&mut self) -> Option<i64> {
        let non_numeric_idx = self
            .remaining()
            .find(|c: char| !c.is_numeric())
            .expect("TODO");
        if non_numeric_idx == 0 {
            return None;
        }
        let res = self.remaining()[0..non_numeric_idx].parse().expect("TODO");
        self.pos += non_numeric_idx;
        Some(res)
    }

    pub fn next_token(&mut self) -> Result<(Token, usize), String> {
        self.skip_ws();

        if self.pos == self.source.len() {
            return Ok((Token::EOF, self.pos));
        }

        let start_pos = self.pos;
        if let Some(identifier) = self.check_for_identifier() {
            if let Some(roman) = roman_number(&identifier) {
                return Ok((Token::NumberLiteral(roman), start_pos));
            }
            if let Some((_, kw)) = KEYWORDS.iter().find(|(name, _)| identifier == *name) {
                return Ok((Token::Keyword(*kw), start_pos));
            }
            return Ok((Token::Ident(identifier), start_pos));
        }
        if let Some(s) = self.multiline_string() {
            return Ok((Token::StringLiteral(s), start_pos));
        }

        if let Some(n) = self.check_for_number() {
            return Ok((Token::NumberLiteral(n), start_pos));
        }

        for (s, tok) in MAPPING {
            if self.remaining().starts_with(s) {
                self.pos += s.len();
                return Ok((tok.clone(), self.pos - s.len()));
            }
        }

        // let token = match self.remaining().chars().next().unwrap() {
        //     '(' => Some(Token::ParOpen),
        //     ')' => Some(Token::ParClose),
        //     _ => None,
        // };
        // if let Some(token) = token {
        //     return Ok((token, start_pos));
        // }

        Err("Unhappy?".to_owned())
    }
}

fn roman_number(s: &str) -> Option<i64> {
    let mut res = 0;
    let mut last = None;
    for c in s.chars() {
        if last.is_none() {
            last = Some(c);
            continue;
        }
        let last_c = last.unwrap();
        let n = match last_c {
            'I' => 1,
            'V' => 5,
            'X' => 10,
            'L' => 50,
            'C' => 100,
            'D' => 500,
            'M' => 1000,
            _ => return None,
        };
        let m = match c {
            'I' => 1,
            'V' => 5,
            'X' => 10,
            'L' => 50,
            'C' => 100,
            'D' => 500,
            'M' => 1000,
            _ => return None,
        };
        if n < m {
            res -= n;
        } else {
            res += n
        }
        last = Some(c);
    }
    let c = last.unwrap();
    Some(
        res + match c {
            'I' => 1,
            'V' => 5,
            'X' => 10,
            'L' => 50,
            'C' => 100,
            'D' => 500,
            'M' => 1000,
            _ => return None,
        },
    )
}

// sorted from long to short for greedy tokenizing
const MAPPING: &[(&str, Token)] = &[
    ("..", Token::DoubleDot),
    ("(", Token::ParOpen),
    (")", Token::ParClose),
    ("=", Token::Equals),
    ("*", Token::Star),
    ("-", Token::Minus),
    ("+", Token::Plus),
    ("/", Token::Slash),
    (",", Token::Comma),
    (";", Token::Semicolon),
];

const KEYWORDS: &[(&str, Keyword)] = &[
    ("and", Keyword::And),
    ("break", Keyword::Break),
    ("do", Keyword::Do),
    ("else", Keyword::Else),
    ("elseif", Keyword::ElseIf),
    ("end", Keyword::End),
    ("false", Keyword::False),
    ("for", Keyword::For),
    ("function", Keyword::Function),
    ("goto", Keyword::Goto),
    ("if", Keyword::If),
    ("in", Keyword::In),
    ("local", Keyword::Local),
    ("nil", Keyword::Nil),
    ("not", Keyword::Not),
    ("or", Keyword::Or),
    ("repeat", Keyword::Repeat),
    ("return", Keyword::Return),
    ("then", Keyword::Then),
    ("true", Keyword::True),
    ("until", Keyword::Until),
    ("while", Keyword::While),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_romans() {
        let test_data = vec![
            (1, "I"),
            (2, "II"),
            (3, "III"),
            (4, "IV"),
            (5, "V"),
            (6, "VI"),
            (7, "VII"),
            (8, "VIII"),
            (9, "IX"),
            (10, "X"),
            (11, "XI"),
            (12, "XII"),
            (13, "XIII"),
            (14, "XIV"),
            (15, "XV"),
            (16, "XVI"),
            (17, "XVII"),
            (18, "XVIII"),
            (19, "XIX"),
            (20, "XX"),
            (21, "XXI"),
            (22, "XXII"),

            // v- real shit
            (4, "IVX"),
            (19, "IXX"),
            (10, "IXI"),
            (6, "IIIIII"),

            // v- no, _this_ is the reeeeeeal shit
            (4997, "MIMIMIMIMI"),
        ];

        for (i, s) in test_data {
            let res = roman_number(s);
            assert_eq!(Some(i), res);
        }
    }
}
