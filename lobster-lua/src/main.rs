#![expect(clippy::mutable_key_type)] // Lua tables keyed by Value, and Value has a variant with interior mutability

use core::panic;
use std::cell::RefCell;
use std::io::{Read, Write};
use std::rc::Rc;
use std::{any::Any, collections::HashMap};
// decisions:
// our lua starts at 0
use std::fs::{self, read_to_string};

use crate::parser::{LobsterParser, Stmt};

#[cfg(test)]
mod e2e;
#[cfg(test)]
mod errortest;
mod fraction;
mod parser;
mod tokenizer;

use fraction::Fraction;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Builtin {
    Print,
    Execute,
    FileOpen,
    FileRead,
    FileWrite,
    Fork,
    Bind,
    Accept,
    TcpStreamRead,
    TcpStreamWrite,
    TcpStreamClose,
    StringGmatch,
    StringLen,
}

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Number(i64),
    Fraction(Fraction),
    String(String),
    Bool(bool),
    Closure {
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Builtin(Builtin),
    Table(Rc<RefCell<HashMap<Value, Value>>>),
    FsFile(Rc<RefCell<fs::File>>),
    TcpStream(Rc<RefCell<std::net::TcpStream>>),
    TcpListener(Rc<RefCell<std::net::TcpListener>>),
}

pub struct TableMut<'a>(std::cell::RefMut<'a, HashMap<Value, Value>>);
// pub struct TableRef<'a>(std::cell::Ref<'a, HashMap<Value, Value>>);

impl TableMut<'_> {
    fn get_elem(&self, name: &str) -> Option<Value> {
        self.0.get(&Value::new_string(name)).cloned()
    }

    fn set_elem(&mut self, name: &str, value: Value) {
        self.0.insert(Value::new_string(name), value);
    }
}

impl Value {
    pub fn new_string(s: impl Into<String>) -> Self {
        Value::String(s.into())
    }

    pub fn new_table() -> Self {
        Value::Table(Default::default())
    }
}

impl Value {
    pub fn is_table(&self) -> bool {
        matches!(self, Value::Table(_))
    }

    pub fn try_as_table<'a>(&'a self) -> Option<TableMut<'a>> {
        match self {
            Value::Table(t) => Some(TableMut(t.borrow_mut())),
            _ => None,
        }
    }
}

impl Value {
    fn try_as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Nil, _) => false,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Number(_), _) => false,
            (Self::Fraction(l0), Self::Fraction(r0)) => l0 == r0,
            (Self::Fraction(_), _) => false,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::String(_), _) => false,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Bool(_), _) => false,
            (Self::Builtin(l0), Self::Builtin(r0)) => l0 == r0,
            (Self::Builtin(_), _) => false,
            (
                Self::Closure {
                    params: l_params,
                    body: l_body,
                },
                Self::Closure {
                    params: r_params,
                    body: r_body,
                },
            ) => l_params == r_params && l_body == r_body, // TODO: compare by pointer
            (Self::Closure { .. }, _) => false,
            (Self::Table(_), _) => todo!("No time, sorry"),
            (Self::FsFile(_), _) => todo!("dont go comparing your files kids"),
            (Self::TcpListener(_), _) => todo!("listen to your Mom, don't do TCP"),
            (Self::TcpStream(_), _) => todo!("TcpStream not yet ready"),
        }
    }
}
impl Eq for Value {}

impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Nil => {}
            Value::Number(n) => n.hash(state),
            Value::Fraction(_fraction) => todo!(),
            Value::String(s) => s.hash(state),
            Value::Bool(_) => todo!(),
            Value::Builtin(b) => b.hash(state),
            Value::Closure { params: _, body: _ } => todo!(),
            Value::Table(_hash_map) => todo!(),
            Value::FsFile(_file) => todo!(),
            Value::TcpListener(_listener) => todo!(),
            Value::TcpStream(_listener) => todo!(),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Fraction(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Builtin(b) => write!(f, "function {b:?}"),
            Value::Closure { params, body: _ } => write!(f, "function {params:#?}"),
            Value::Table(hash_map) => write!(f, "{hash_map:?}"),
            Value::FsFile(file) => write!(f, "fiel {file:?}"),
            Value::TcpListener(listener) => write!(f, "listener, your mom: {listener:?}"),
            Value::TcpStream(stream) => write!(f, "strom {stream:?}"),
        }
    }
}

impl Value {
    fn as_fraction(&self) -> Result<Fraction, &'static str> {
        match self {
            Self::Number(n) => Ok(Fraction::new(*n, 1)),
            Self::Fraction(f) => Ok(*f),
            _ => Err("NOT FRACTIONABLE"),
        }
    }

    fn add(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Self::Number(l + r)),
            (lv, rv) => Ok(Value::Fraction(lv.as_fraction()? + rv.as_fraction()?)),
        }
    }

    fn sub(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Self::Number(l - r)),
            (lv, rv) => Ok(Value::Fraction(lv.as_fraction()? - rv.as_fraction()?)),
        }
    }
    fn div(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (lv, rv) => Ok(Value::Fraction(lv.as_fraction()? / rv.as_fraction()?)),
        }
    }
    fn mul(self, rhs: Self) -> Result<Value, &'static str> {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Self::Number(l * r)),
            (lv, rv) => Ok(Value::Fraction(lv.as_fraction()? * rv.as_fraction()?)),
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
            (Value::String(l), Value::Number(r)) => Ok(Self::String(format!("{l}{r}"))),
            (Value::Number(l), Value::String(r)) => Ok(Self::String(format!("{l}{r}"))),
            _ => Err("PANIK?????"),
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

    fn get(&self, name: &Value) -> Result<Value, &'static str> {
        match self {
            Value::Table(ref_cell) => {
                let table = ref_cell.borrow();
                Ok(table.get(name).expect("todo").clone())
            }
            _ => Err("this is not a -bucket- table"),
        }
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

pub trait StdOut: Any + std::io::Write {}
impl<T> StdOut for T where T: Any + std::io::Write {}

pub struct Context {
    stdout: Box<dyn StdOut>,
    globals: HashMap<String, Value>,
    locals: Vec<HashMap<String, Value>>,
}

impl Context {
    pub fn get(&self, name: &str) -> Option<Value> {
        for scope in self.locals.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
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

fn os_module() -> Value {
    let mut m = HashMap::default();
    m.insert("execute".into(), Value::Builtin(Builtin::Execute));
    m.insert("fork".into(), Value::Builtin(Builtin::Fork));
    Value::Table(Rc::new(RefCell::new(m)))
}

fn io_module() -> Value {
    let mut m = HashMap::default();
    m.insert("open".into(), Value::Builtin(Builtin::FileOpen));
    m.insert("bind".into(), Value::Builtin(Builtin::Bind));
    Value::Table(Rc::new(RefCell::new(m)))
}

fn string_module() -> Value {
    let mut m = HashMap::default();
    m.insert("gmatch".into(), Value::Builtin(Builtin::StringGmatch));
    m.insert("len".into(), Value::Builtin(Builtin::StringLen));
    Value::Table(Rc::new(RefCell::new(m)))
}

fn default_globals() -> HashMap<String, Value> {
    let mut globals: HashMap<String, Value> = Default::default();
    globals.insert("print".to_string(), Value::Builtin(Builtin::Print));
    globals.insert("os".to_string(), os_module());
    globals.insert("io".to_string(), io_module());
    globals.insert("string".to_string(), string_module());
    globals
}

fn main() {
    let source = read_to_string("sample.lua").expect("todo");
    let ast = match LobsterParser::parse(source.clone()) {
        Ok(ast) => ast,
        Err(e) => {
            panic!("{}", e.render("sample.lua", &source));
        }
    };

    let globals = default_globals();
    let mut context: Context = Context {
        stdout: Box::new(std::io::stdout()),
        globals,
        locals: vec![HashMap::new()],
    };

    run_block(&ast, &mut context).expect("TODO");
}

#[derive(Debug)]
enum Ret {
    Continue,
    Break,
    Return(Value),
}

fn run_block(stmts: &[parser::Stmt], context: &mut Context) -> Result<(), Ret> {
    for stmt in stmts {
        // dbg!(stmt);
        match stmt {
            parser::Stmt::Assignment {
                lhs,
                rhs: value,
                local,
            } => {
                let res = eval(value, context);
                match lhs {
                    parser::LeftExpr::Var(variable) => {
                        if *local {
                            context.insert_local(variable.clone(), res);
                        } else {
                            context.insert_global(variable.clone(), res);
                        }
                    }
                    parser::LeftExpr::TableIndex { table, index } => {
                        let Value::Table(table) = eval(table, context) else {
                            panic!("Indexing into not a table");
                        };
                        let index = eval(index, context);
                        table.borrow_mut().insert(index, res);
                    }
                }
            }
            parser::Stmt::If { cond, then, r#else } => {
                // TODO: also accept '1'
                if eval(cond, context) == Value::Bool(true) {
                    run_block(then, context)?;
                } else {
                    run_block(r#else, context)?;
                }
            }
            parser::Stmt::While { cond, body } => {
                while eval(cond, context) == Value::Bool(true) {
                    match run_block(body, context) {
                        Ok(()) => {}
                        Err(Ret::Break) => break,
                        Err(Ret::Continue) => {}
                        x @ Err(Ret::Return(_)) => return x,
                    }
                }
            }
            parser::Stmt::For { name, expr, body } => {
                let iterator = eval(expr, context);
                loop {
                    let elem = evaluate_function(context, Vec::new(), iterator.clone());
                    if elem == Value::Nil {
                        break
                    }
                    context.insert_local(name.clone(), elem);
                    run_block(body.as_ref(), context)?;
                }
            }
            parser::Stmt::Break => return Err(Ret::Break),
            parser::Stmt::Continue => return Err(Ret::Continue),
            parser::Stmt::Return(exprs) => {
                let mut values: Vec<_> = exprs.iter().map(|expr| eval(expr, context)).collect();
                return Err(Ret::Return(values.remove(0)));
            }
            parser::Stmt::DoEnd { body } => run_block(body, context)?,
            parser::Stmt::Expr { expr } => {
                eval(expr, context); // should be a function call
            }
        }
    }
    Ok(())
}

fn eval(expr: &parser::Expr, context: &mut Context) -> Value {
    match expr {
        parser::Expr::Nil => Value::Nil,
        parser::Expr::Numeral(i) => Value::Number(*i),
        parser::Expr::Fraction(f) => Value::Fraction(*f),
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
        parser::Expr::Var(ident) => context
            .get(ident)
            .unwrap_or_else(|| panic!("TODO, variable {ident:?} not found"))
            .clone(),
        parser::Expr::FunctionCall { callee, args } => {
            let function = eval(callee, context);

            let evaluated_args: Vec<_> = args.into_iter().map(|arg| eval(arg, context)).collect();

            evaluate_function(context, evaluated_args, function)
        }
        parser::Expr::MethodCall {
            callee,
            method_name,
            args,
        } => {
            let object = eval(callee, context);

            let mut evaluated_args: Vec<_> = args.iter().map(|arg| eval(arg, context)).collect();
            let function = object
                .get(&Value::String(method_name.clone()))
                .expect("todo");

            evaluated_args.insert(0, object);
            evaluate_function(context, evaluated_args, function)
        }
        parser::Expr::FunctionDef { arguments, body } => Value::Closure {
            params: arguments.clone(),
            body: body.clone(),
        },
        parser::Expr::Table { values } => {
            let mut map = HashMap::new();
            for (key, value) in values {
                map.insert(eval(key, context), eval(value, context));
            }
            Value::Table(Rc::new(RefCell::new(map)))
        }
        parser::Expr::TableIndex { table, index } => {
            let t = eval(table, context);
            match t {
                Value::Table(map) => map
                    .borrow()
                    .get(&eval(index, context))
                    .cloned()
                    .unwrap_or(Value::Nil),
                _ => panic!("not a table"),
            }
        }
    }
}

fn evaluate_function(context: &mut Context, evaluated_args: Vec<Value>, function: Value) -> Value {
    match function {
        Value::Closure { params, body } => {
            context.enter_scope();
            assert_eq!(
                params.len(),
                evaluated_args.len(),
                "calling with wrong number of parameters"
            );
            for (param, arg) in params.iter().zip(evaluated_args) {
                context.insert_local(param.clone(), arg);
            }
            let return_val = run_block(&body.clone() /* TODO: get rid of clone */, context);
            context.leave_scope();
            match return_val {
                Ok(()) => Value::Nil,
                Err(Ret::Return(x)) => x,
                Err(_) => panic!("break or continue outside of loop {return_val:?}"),
            }
        }
        Value::Builtin(Builtin::Print) => {
            let mut line = evaluated_args
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            line.push('\n');

            write!(context.stdout, "{line}").expect("write failed!");
            Value::Nil
        }
        Value::Builtin(Builtin::Execute) => {
            assert_eq!(evaluated_args.len(), 1);
            let Value::String(command) = &evaluated_args[0] else {
                panic!("cannot run this shit");
            };
            cfg_select! {
                target_family = "unix" => {
                    std::process::Command::new("/bin/sh")
                        .arg("-c")
                        .arg(command)
                        .status()
                        .unwrap();
                }
                target_os = "windows" => {
                    std::process::Command::new("cmd.exe")
                        .arg(command)
                        .status()
                        .unwrap();
                }
            }
            Value::Nil
        }
        Value::Builtin(Builtin::FileOpen) => {
            assert_eq!(evaluated_args.len(), 2);
            let Value::String(filename) = &evaluated_args[0] else {
                panic!("this does not smell like a file");
            };
            let Value::String(mode) = &evaluated_args[1] else {
                panic!("mode should be string");
            };

            let file = match mode.as_str() {
                "r" => fs::File::open(filename),
                "w" => fs::File::create(filename),
                "a" => fs::OpenOptions::new().append(true).open(filename),
                _ => panic!("Not an option, bro"),
            };

            let file = Value::FsFile(Rc::new(RefCell::new(file.expect("I want to be a file"))));

            let h = Value::new_table();
            {
                let mut t = h.try_as_table().unwrap();
                t.set_elem("file", file);
                t.set_elem("read", Value::Builtin(Builtin::FileRead));
                t.set_elem("write", Value::Builtin(Builtin::FileWrite));
            }
            h
        }
        Value::Builtin(Builtin::FileWrite) => {
            assert_eq!(evaluated_args.len(), 2);
            let Value::Table(_file_handle) = &evaluated_args[0] else {
                panic!("TODO: expected a file table thingy");
            };
            let _write = evaluated_args[1]
                .try_as_str()
                .expect("Provide a String to write!");
            todo!("Writing not yet supported")
        }
        Value::Builtin(Builtin::FileRead) => {
            assert_eq!(evaluated_args.len(), 2);
            let h = &evaluated_args[0];
            let t = h
                .try_as_table()
                .expect("TODO: expected a file table thingy");

            let Value::Number(len) = &evaluated_args[1] else {
                panic!("TODO: not a number");
            };
            let len: usize = (*len).try_into().expect("they didn't let me use as(s)");

            let fs_file = t.get_elem("file").expect("TODO");
            let Value::FsFile(file_handle) = fs_file else {
                panic!("impossible! :o");
            };

            let mut file_handle = file_handle.borrow_mut();

            let mut buf = vec![0u8; len];
            let bytes_read = file_handle.read(&mut buf[0..len]).expect("no read no good");

            let data_as_string = String::from_utf8_lossy(&buf[0..bytes_read]).to_string();
            Value::String(data_as_string)
        }
        Value::Builtin(Builtin::Fork) => {
            unsafe extern "C" {
                unsafe fn fork() -> isize;
            }
            let r = unsafe { fork() };
            Value::Number(r.try_into().expect("TODO"))
        }
        Value::Builtin(Builtin::Bind) => {
            assert_eq!(evaluated_args.len(), 1);
            let address = &evaluated_args[0]
                .try_as_str()
                .expect("this does not smell like a address");

            let listener = std::net::TcpListener::bind(address).expect("TODO");

            let file = Value::TcpListener(Rc::new(RefCell::new(listener)));

            let h = Value::new_table();
            {
                let mut t = h.try_as_table().unwrap();
                t.set_elem("listener", file);
                t.set_elem("accept", Value::Builtin(Builtin::Accept));
            }
            h
        }
        Value::Builtin(Builtin::Accept) => {
            assert_eq!(evaluated_args.len(), 1);
            let h = &evaluated_args[0];
            let t = h
                .try_as_table()
                .expect("TODO: expected a file table thingy");
            let listener = t.get_elem("listener").expect("TODO");
            let Value::TcpListener(listener) = listener else {
                panic!("impossible! :o");
            };
            let listener = listener.borrow_mut();
            let (stream, _address) = listener.accept().expect("todont");

            let h = Value::new_table();
            {
                let mut t = h.try_as_table().unwrap();
                t.set_elem("stream", Value::TcpStream(Rc::new(RefCell::new(stream))));
                t.set_elem("read", Value::Builtin(Builtin::TcpStreamRead));
                t.set_elem("write", Value::Builtin(Builtin::TcpStreamWrite));
                t.set_elem("close", Value::Builtin(Builtin::TcpStreamClose));
            }
            h
        }
        Value::Builtin(Builtin::TcpStreamWrite) => {
            assert_eq!(evaluated_args.len(), 2);
            let h = &evaluated_args[0];
            let t = h
                .try_as_table()
                .expect("TODO: expected a file table thingy");

            let Value::String(buf) = &evaluated_args[1] else {
                panic!("TODO: not a number");
            };
            let buf = buf.as_bytes();

            let stream = t.get_elem("stream").expect("TODO");
            let Value::TcpStream(stream) = stream else {
                panic!("impossible! :o");
            };

            let mut stream = stream.borrow_mut();
            let written = stream.write(&buf).expect("bingle my bongle");
            Value::Number(written as i64)
        }
        Value::Builtin(Builtin::TcpStreamRead) => {
            assert_eq!(evaluated_args.len(), 2);
            let h = &evaluated_args[0];
            let t = h
                .try_as_table()
                .expect("TODO: expected a file table thingy");

            let Value::Number(len) = &evaluated_args[1] else {
                panic!("TODO: not a number");
            };
            let len: usize = (*len).try_into().expect("they didn't let me use as(s)");

            let stream = t.get_elem("stream").expect("TODO");
            let Value::TcpStream(stream) = stream else {
                panic!("impossible! :o");
            };

            let mut stream = stream.borrow_mut();

            let mut buf = vec![0u8; len];
            let bytes_read = stream.read(&mut buf[0..len]).expect("no read no good");

            let data_as_string = String::from_utf8_lossy(&buf[0..bytes_read]).to_string();
            Value::String(data_as_string)
        }
        Value::Builtin(Builtin::TcpStreamClose) => {
            assert_eq!(evaluated_args.len(), 1);
            let h = &evaluated_args[0];
            let mut t = h
                .try_as_table()
                .expect("TODO: expected a file table thingy");

            // DROP IT!
            t.set_elem("stream", Value::Nil);
            Value::Nil
        }
        Value::Builtin(Builtin::StringGmatch) => {
            assert_eq!(evaluated_args.len(), 2);
            let haystack = evaluated_args[0].try_as_str().expect("bingle");
            let pattern = evaluated_args[1].try_as_str().expect("bingle");

            // We only implement splitting by space for the purposes of
            // implementing string.split in terms of string.gsub (attempting to
            // be Like A Real Lua).
            let Some(rest) = pattern.strip_prefix("([^") else {
                panic!("you wish");
            };
            let Some(rest) = rest.strip_suffix("])") else {
                panic!("you wish");
            };
            assert_eq!(rest.len(), 1);

            let mut m = HashMap::default();
            for (i, v) in haystack.split(rest).enumerate() {
                m.insert(Value::Number(i as i64), Value::String(v.into()));
            }
            Value::Table(Rc::new(RefCell::new(m)))
        }
        Value::Builtin(Builtin::StringLen) => {
            assert_eq!(evaluated_args.len(), 1);
            let haystack = evaluated_args[0].try_as_str().expect("bingle");
            Value::Number(haystack.len() as i64)
        }

        x => panic!("{x:?} is not callable"),
    }
}
