use core::panic;
use std::collections::HashMap;
// decisions:
// our lua starts at 0
use std::fs::read_to_string;
use std::ops;

use crate::parser::LobsterParser;
use crate::tokenizer::{Token, Tokenizer};

mod parser;
mod tokenizer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Nil,
    Number(i64),
    String(String),
    Bool(bool),
}

impl Value {
    fn add(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Self::Number(l + r)),
            _ => Err("PANIK"),
        }
    }

    fn sub(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Self::Number(l - r)),
            _ => Err("PANIK"),
        }
    }
    fn div(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Self::Number(l / r)),
            _ => Err("PANIK"),
        }
    }
    fn mul(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Self::Number(l * r)),
            _ => Err("PANIK"),
        }
    }
    fn exp(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => {
                Ok(Self::Number(l.pow(r.try_into().expect("TODO"))))
            }
            _ => Err("PANIK"),
        }
    }
    fn r#mod(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Self::Number(l % r)),
            _ => Err("PANIK"),
        }
    }
    fn and(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Bool(l), Value::Bool(r)) => Ok((Self::Bool(l && r))),
            _ => Err("PANIK"),
        }
    }
    fn or(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Bool(l), Value::Bool(r)) => Ok((Self::Bool(l || r))),
            _ => Err("PANIK"),
        }
    }
    fn lshift(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Self::Number(l << r)),
            _ => Err("PANIK"),
        }
    }
    fn rshift(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Self::Number(l >> r)),
            _ => Err("PANIK"),
        }
    }

    fn concat(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::String(l), Value::String(r)) => Ok(Self::String(l + &r)),
            _ => Err("PANIK"),
        }
    }

    fn gt(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Self::Bool(l > r)),
            _ => Err("PANIK"),
        }
    }

}

pub struct StackFrame {
    locals: HashMap<String, Value>,
}

fn main() {
    let source = read_to_string("sample.lua").expect("todo");
    let parser = LobsterParser::new(source);
    let ast = parser.parse();

    let mut locals: HashMap<String, Value> = Default::default();

    run_block(&ast, &mut locals); 

    dbg!(&locals);

    if let Value::String(s) = locals.get("out").unwrap() {
        println!("{s}");
    }
}

fn run_block(stmts: &[parser::Stmt], locals: &mut HashMap<String, Value>) {
    for stmt in stmts {
        match stmt {
            parser::Stmt::Assignment { variable, value } => {
                let res = eval(value, locals);
                locals.insert(variable.clone(), res);
            }
            parser::Stmt::If {
                cond,
                then,
                r#else,
            } => {
                if eval(cond, locals) == Value::Bool(true) {
                    run_block(then, locals);
                } else {
                    run_block(r#else, locals);
                }
            }
            parser::Stmt::While { cond, body } => {
                while (eval(cond, locals) == Value::Bool(true)) {
                    run_block(body, locals)
                }
            },
            _ => (),
            parser::Stmt::Break => todo!(),
            parser::Stmt::Return(exprs) => todo!(),
            parser::Stmt::DoEnd { body } => todo!(),
            parser::Stmt::FunctionCall {
                function_name,
                args,
            } => todo!(),
        }
    }

}

fn eval(expr: &parser::Expr, locals: &mut HashMap<String, Value>) -> Value {
    match expr {
        parser::Expr::Nil => Value::Nil,
        parser::Expr::Numeral(i) => Value::Number(*i),
        parser::Expr::Boolean(b) => Value::Bool(*b),
        parser::Expr::String(s) => Value::String(s.clone()),
        parser::Expr::BinOp { op, lhs, rhs } => {
            let lhs = eval(lhs, locals);
            if let (parser::BinOp::And, Value::Bool(false)) = (op, &lhs) {
                return Value::Bool(false);
            }
            if let (parser::BinOp::Or, Value::Bool(true)) = (op, &lhs) {
                return Value::Bool(true);
            }
            let rhs = eval(rhs, locals);
            match op {
                parser::BinOp::Plus => lhs.add(rhs),
                parser::BinOp::Minus => lhs.sub(rhs),
                parser::BinOp::Mul => lhs.mul(rhs),
                parser::BinOp::Div => lhs.div(rhs),
                parser::BinOp::IDiv => lhs.div(rhs),
                parser::BinOp::Exp => lhs.exp(rhs),
                parser::BinOp::Mod => lhs.r#mod(rhs),
                parser::BinOp::And => lhs.and(rhs),
                parser::BinOp::Or => lhs.or(rhs),
                parser::BinOp::LShift => lhs.lshift(rhs),
                parser::BinOp::RShift => lhs.rshift(rhs),
                parser::BinOp::GT => lhs.gt(rhs),
                parser::BinOp::LT => todo!(),
                parser::BinOp::GEQ => todo!(),
                parser::BinOp::LEQ => todo!(),
                parser::BinOp::BitOR => todo!(),
                parser::BinOp::BitAnd => todo!(),
                parser::BinOp::BitXor => todo!(),
                parser::BinOp::Equals => Ok(Value::Bool(lhs.eq(&rhs))),
                parser::BinOp::NotEquals => Ok(Value::Bool(!lhs.eq(&rhs))),
                parser::BinOp::Concat => lhs.concat(rhs),
            }
            .expect("TODO")
        }
        parser::Expr::Var(ident) => locals.get(ident).expect("TODO").clone(),
        parser::Expr::FunctionCall {
            function_name,
            args,
        } => todo!(),
        parser::Expr::FunctionDef { body } => todo!(),
    }
}
