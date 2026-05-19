use crate::parser::LobsterParser;


fn check_parse_error(code: &str, expected_err: &str) {
    let mut parser = LobsterParser::new(code.to_owned());
    let result = parser.parse();

    assert_eq!(result.err().unwrap().render("", code), expected_err)
}

#[test]
fn test_we_get_an_error() {
    let code = "++";
    let expected_err = ":1 Expected EOF, found Plus\n | ++";
    check_parse_error(code, expected_err);
}
