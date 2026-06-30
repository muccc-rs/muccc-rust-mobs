use crate::parser::LobsterParser;
use expect_test::{Expect, expect};

fn check_parse_error(code: &str, expected_err: &Expect) {
    let result = LobsterParser::parse(code.to_owned());

    let mut result = result
        .err()
        .expect("we want an error, we did not get it")
        .render("test.lua", code);
    result.push('\n');
    expected_err.assert_eq(&result);
}

#[test]
fn error00001() {
    let code = "++";
    let expected_err = expect![[r#"
        test.lua:1:1: weird statement start: Plus
         | ++
           ^
    "#]];
    check_parse_error(code, &expected_err);
}

#[test]
fn error00002() {
    let code = "list = { 3 = 3 }";
    let expected_err = expect![[r#"
        test.lua:1:10: list key must be an identifier
         | list = { 3 = 3 }
                    ^
    "#]];
    check_parse_error(code, &expected_err);
}

#[test]
fn error00003() {
    let code = "function IV () end";
    let expected_err = expect![[r#"
        test.lua:1:10: Function name must be an identifier!
         | function IV () end
                    ^
    "#]];
    check_parse_error(code, &expected_err);
}

#[test]
fn error00004() {
    let code = "function foo end";
    let expected_err = expect![[r#"
        test.lua:1:14: Expected ParOpen, got Keyword(End)
         | function foo end
                        ^
    "#]];
    check_parse_error(code, &expected_err);
}

#[test]
fn error00005() {
    let code = "function foo (a b) end";
    let expected_err = expect![[r#"
        test.lua:1:17: Expected Comma, got Ident("b")
         | function foo (a b) end
                           ^
    "#]];
    check_parse_error(code, &expected_err);
}

#[test]
fn error000051() {
    let code = "list = { a b }";
    let expected_err = expect![[r#"
        test.lua:1:12: Expected Comma or BraceClose, got Ident("b")
         | list = { a b }
                      ^
    "#]];
    check_parse_error(code, &expected_err);
}

#[test]
fn error00006() {
    let code = "print(list[[[5]]])";
    let expected_err = expect![[r#"
        test.lua:1:11: Expected ParClose or Comma, got StringLiteral("[5")
         | print(list[[[5]]])
                     ^
    "#]];
    check_parse_error(code, &expected_err);
}

#[test]
fn error00007() {
    let code = "{ foo, bar, }";
    let expected_err = expect![[r#"
        test.lua:1:1: weird statement start: BraceOpen
         | { foo, bar, }
           ^
    "#]];
    check_parse_error(code, &expected_err);
}

#[test]
fn error00008() {
    let code = "function end";
    let expected_err = expect![[r#"
        test.lua:1:10: Function name must be an identifier!
         | function end
                    ^
    "#]];
    check_parse_error(code, &expected_err);
}

#[test]
fn error00009() {
    let code = "[[";
    let expected_err = expect![[r#"
        test.lua:1:1: Did not find matching string end marker ]]
         | [[
           ^
    "#]];
    check_parse_error(code, &expected_err);
}

#[test]
fn error00010() {
    let code = "x = 9223372036854775808";
    let expected_err = expect![[r#"
        test.lua:1:5: number too large to fit in target type
         | x = 9223372036854775808
               ^
    "#]];
    check_parse_error(code, &expected_err);
}

#[test]
fn error00011() {
    let code = "--[====[ \n ]===]";
    let expected_err = expect![[r#"
        test.lua:1:1: Multiline comment is missing ]====] end marker
         | --[====[ 
           ^
    "#]];
    check_parse_error(code, &expected_err);
}
