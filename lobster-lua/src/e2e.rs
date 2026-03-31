use std::collections::HashMap;

use crate::parser::LobsterParser;
use crate::{Context, Value, run_block};

fn run(source: &str) -> String {
    let parser = LobsterParser::new(source.to_owned());
    let ast = parser.parse();

    let globals: HashMap<String, Value> = Default::default();

    let mut context: Context = Context {
        test_stdout: Some(String::new()),
        globals,
        locals: vec![HashMap::new()],
    };

    run_block(&ast, &mut context);

    context.test_stdout.unwrap()
}

#[test]
fn hello_world() {
    let out = run(r#"
        print([[hello world]])
    "#);
    assert_eq!(out, "hello world\t\n");
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
print()
print([[zzz]], S∷ / 2)
    "#);
    assert_eq!(
        out,
        r#"
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
"#
    );
}
