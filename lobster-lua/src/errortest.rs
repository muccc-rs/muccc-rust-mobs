use crate::parser::LobsterParser;
use expect_test::{Expect, expect};

fn check_parse_error(code: &str, expected_err: &Expect) {
    let parser = LobsterParser::new(code.to_owned());
    let result = parser.parse();

    let mut result = result.err().unwrap().render("test.lua", code);
    result.push('\n');
    expected_err.assert_eq(&result);
}

#[test]
fn error00001() {
    let code = "++";
    let expected_err = expect![[r#"
        test.lua:1 weird statement start: Plus
         | ++
    "#]];
    check_parse_error(code, &expected_err);
}

#[test]
fn error00002() {
    let code = "list = { 3 = 3 }";
    let expected_err = expect![[r#"
        test.lua:1 list key must be an identifier
         | list = { 3 = 3 }
    "#]];
    check_parse_error(code, &expected_err);
}

#[test]
fn error00003() {
    let code = "function IV () end";
    let expected_err = expect![[r#"
        test.lua:1 Function name must be an identifier!
         | function IV () end
    "#]];
    check_parse_error(code, &expected_err);
}

#[test]
fn error00004() {
    let code = "function foo end";
    let expected_err = expect![[r#"
        test.lua:1 Expected ParOpen, got Keyword(End)
         | function foo end
    "#]];
    check_parse_error(code, &expected_err);
}

#[test]
fn error00005() {
    let code = "function foo (a b) end";
    let expected_err = expect![[r#"
        test.lua:1 Expected Comma, got Ident("b")
         | function foo (a b) end
    "#]];
    check_parse_error(code, &expected_err);
}

#[test]
fn error000051() {
    let code = "list = { a b }";
    let expected_err = expect![[r#"
        test.lua:1 Expected Comma or BraceClose, got Ident("b")
         | list = { a b }
    "#]];
    check_parse_error(code, &expected_err);
}

#[test]
fn error00006() {
    let code = "print(list[[[5]]])";
    let expected_err = expect![[r#"
        test.lua:1 Expected ParClose or Comma, got StringLiteral("[5")
         | print(list[[[5]]])
    "#]];
    check_parse_error(code, &expected_err);
}
