use std::any::Any;
use std::collections::HashMap;
use std::ops::DerefMut;

use crate::parser::LobsterParser;
use crate::{Context, run_block};

fn run(source: &str) -> String {
    let ast = LobsterParser::parse(source.to_owned()).unwrap();

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
    expect_test::expect!([r#"
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
    "#]).assert_eq(&out);
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
    expect_test::expect![[r#"
        1
        12

        1
        2
        3
        nil
    "#]].assert_eq(&out);
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
    expect_test::expect![[r#"
        1
    "#]].assert_eq(&out);
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
    expect_test::expect![[r#"
        2
    "#]].assert_eq(&out);
}

#[test]
fn zzz() {
    let out = run(r#"
out = [[]]
size = IIIIIIIIIIIIIIIIIIIIII
y = 0-size
while y ~= size do
	x = 0 - size
	while x ~= size do
		xabs = x
		if 0 > x then
			xabs = 0 - x
		end
		y_ = y + xabs
		f = x * x + y_ * y_
		if f > 200 then
			out = out .. [[XX]]
		else
			out = out .. [[  ]]
		end
		x = x + 1
	end
	out = out .. [[
]]
y = y + 1
end
print(out)

"#);
    expect_test::expect![[r#"
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXX  XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX  XXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXX            XXXXXXXXXXXXXXXXXXXXXXXXXX            XXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXX                  XXXXXXXXXXXXXXXXXX                  XXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXX                    XXXXXXXXXXXXXX                    XXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXX                          XXXXXX                          XXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXX                            XX                            XXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXX                                                          XXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXX                                                          XXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXX                                                          XXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXX                                                      XXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXX                                                      XXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXX                                                      XXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXX                                                      XXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXX                                                  XXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXX                                                  XXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXX                                                  XXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXX                                              XXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXX                                              XXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXX                                          XXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXX                                          XXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXX                                          XXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXX                                      XXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXX                                  XXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXX                                  XXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX                              XXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX                              XXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX                          XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX                      XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX                      XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX                  XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX              XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX          XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX          XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX      XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX  XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

    "#]].assert_eq(&out);
}