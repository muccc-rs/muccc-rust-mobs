use std::collections::HashMap;
// decisions:
// our lua starts at 0
use std::fs::read_to_string;

use crate::parser::{LobsterParser, Stmt};

mod parser;
mod tokenizer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Nil,
    Number(i64),
    String(String),
    Bool(bool),
    Closure {
        params: Vec<String>,
        body: Vec<Stmt>,
    }
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
            (Value::Bool(l), Value::Bool(r)) => Ok(Self::Bool(l && r)),
            _ => Err("PANIK"),
        }
    }
    fn or(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Bool(l), Value::Bool(r)) => Ok(Self::Bool(l || r)),
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

    fn lt(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Self::Bool(l < r)),
            _ => Err("PANIK"),
        }
    }

    fn geq(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Self::Bool(l >= r)),
            _ => Err("PANIK"),
        }
    }

    fn leq(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Self::Bool(l <= r)),
            _ => Err("PANIK"),
        }
    }

    fn bitor(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Self::Number(l | r)),
            _ => Err("PANIK"),
        }
    }

    fn bitand(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Self::Number(l & r)),
            _ => Err("PANIK"),
        }
    }

    fn bitxor(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Self::Number(l ^ r)),
            _ => Err("PANIK"),
        }
    }
}

pub struct Context {
    globals: HashMap<String, Value>,
    locals: Vec<HashMap<String, Value>>,
}

impl Context {
    pub fn get(&self, name: &str) -> Option<Value> {
        for scope in self.locals.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone())
            }
        }
        self.globals.get(name).cloned()
    }

    pub fn insert_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, value);
    }

    pub fn insert_local(&mut self, name: String, value: Value) {
        self.locals.last_mut().unwrap().insert(name, value);
    }

    pub fn enter_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    pub fn leave_scope(&mut self) {
        self.locals.pop();
    }
}

pub struct StackFrame {
    locals: HashMap<String, Value>,
}

fn main() {
    let source = read_to_string("sample.lua").expect("todo");
    let parser = LobsterParser::new(source);
    let ast = parser.parse();

    let globals: HashMap<String, Value> = Default::default();
    let mut context: Context = Context { globals, locals: vec![HashMap::new()] };

    run_block(&ast, &mut context); 

    if let Value::String(s) = context.get("out").unwrap() {
        println!("{s}");
    }
}

fn run_block(stmts: &[parser::Stmt], context: &mut Context) {
    for stmt in stmts {
        // dbg!(stmt);
        match stmt {
            parser::Stmt::Assignment { variable, value } => {
                let res = eval(value, context);
                context.insert_global(variable.clone(), res);
            }
            parser::Stmt::If {
                cond,
                then,
                r#else,
            } => {
                if eval(cond, context) == Value::Bool(true) {
                    run_block(then, context);
                } else {
                    run_block(r#else, context);
                }
            }
            parser::Stmt::While { cond, body } => {
                while eval(cond, context) == Value::Bool(true) {
                    run_block(body, context)
                }
            },
            parser::Stmt::Break => todo!(),
            parser::Stmt::Return(exprs) => todo!(),
            parser::Stmt::DoEnd { body } => todo!(),
            parser::Stmt::FunctionCall {
                function_name,
                args,
            } => {
                let evaluated_args: Vec<_> = args.into_iter().map(|arg| eval(arg, context)).collect();
                if function_name == "print" {
                    println!("{:?}", evaluated_args);
                } else {
                    let function = context.get(function_name).expect("TODO");
                    match function {
                        Value::Closure { params, body } => {
                            context.enter_scope();
                            assert_eq!(params.len(), args.len(), "calling with wrong number of parameters");
                            for (param,arg) in params.iter().zip(evaluated_args) {
                                context.insert_local(param.clone(),arg);
                            }
                            run_block(&body.clone() /* TODO: get rid of clone */, context);
                            context.leave_scope();
                        }
                        x => panic!("{x:?} is not callable")
                    }
                }
            },
        }
    }

}

fn eval(expr: &parser::Expr, context: &mut Context) -> Value {
    match expr {
        parser::Expr::Nil => Value::Nil,
        parser::Expr::Numeral(i) => Value::Number(*i),
        parser::Expr::Boolean(b) => Value::Bool(*b),
        parser::Expr::String(s) => Value::String(s.clone()),
        parser::Expr::BinOp { op, lhs, rhs } => {
            let lhs = eval(lhs, context);
            if let (parser::BinOp::And, Value::Bool(false)) = (op, &lhs) {
                return Value::Bool(false);
            }
            if let (parser::BinOp::Or, Value::Bool(true)) = (op, &lhs) {
                return Value::Bool(true);
            }
            let rhs = eval(rhs, context);
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
                parser::BinOp::LT => lhs.lt(rhs),
                parser::BinOp::GEQ => lhs.geq(rhs),
                parser::BinOp::LEQ => lhs.leq(rhs),
                parser::BinOp::BitOR => lhs.bitor(rhs),
                parser::BinOp::BitAnd => lhs.bitand(rhs),
                parser::BinOp::BitXor => lhs.bitxor(rhs),
                parser::BinOp::Equals => Ok(Value::Bool(lhs.eq(&rhs))),
                parser::BinOp::NotEquals => Ok(Value::Bool(!lhs.eq(&rhs))),
                parser::BinOp::Concat => lhs.concat(rhs),
            }
            .expect("TODO")
        }
        parser::Expr::Var(ident) => context.get(ident).expect("TODO").clone(),
        parser::Expr::FunctionCall {
            function_name,
            args,
        } => todo!(),
        parser::Expr::FunctionDef { arguments, body } => 
            Value::Closure {
                params: arguments.clone(),
                body: body.clone(),
            }
    }
}
