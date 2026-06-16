use std::any::Any;
use std::collections::HashMap;
use std::ops::DerefMut;

use crate::parser::LobsterParser;
use crate::{Context, run_block};

fn run(source: &str) -> String {
    let parser = LobsterParser::new(source.to_owned());
    let ast = parser.parse().unwrap();

    let globals = crate::default_globals();

    let mut context: Context = Context {
        stdout: Box::new(Vec::<u8>::new()),
        globals,
        locals: vec![HashMap::new()],
    };

    run_block(&ast, &mut context).expect("TODO");

    let stdout: Box<dyn Any> = context.stdout;
    let mut stdout: Box<Vec<u8>> = stdout.downcast::<Vec<u8>>().unwrap();
    let stdout: Vec<u8> = std::mem::take(stdout.deref_mut()); // FIXME: replace with Box::into_inner once stable

    String::from_utf8(stdout).unwrap()
}

#[test]
fn hello_world() {
    let out = run(r#"
        print([[hello world]])
    "#);
    assert_eq!(out, "hello world\n");
}

#[test]
fn hello_romans() {
    let out = run(r#"
print([[All the fractions]])
print(·)
print(:)
print(∴)
print(∷)
print(⁙)
print(S)
print(S·)
print(S:)
print(S∴)
print(S∷)
    "#);
    assert_eq!(
        out,
        "\
All the fractions
1/12
1/6
1/4
1/3
5/12
1/2
7/12
2/3
3/4
5/6
"
    );
}

#[test]
fn tables() {
    let out = run(r#"
        empty = {}
        v = 12
        t = { x = 1, [v] = 12 }
        print(t[ [[x]] ])
        print(t[v])

        print()

        l = { 1, 2, 3 }
        print(l[0])
        print(l[1])
        print(l[2])
        print(l[3])
    "#);
    assert_eq!(
        out,
        "\
1
12

1
2
3
nil
"
    );
}

#[test]
fn test_break() {
    let out = run(r#"
    while true do
        print(1)
        break
        print(2)
    end
    "#);
    assert_eq!(out, "1\n");
}

#[test]
fn test_continue() {
    let out = run(r#"
    count = 0
    while true do
        count = count + 1
        if 2 > count then
            continue
        end
        break
    end
    print(count)
    "#);
    assert_eq!(out, "2\n");
}
