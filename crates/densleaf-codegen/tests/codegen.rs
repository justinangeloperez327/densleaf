use densleaf_codegen::generate_rust;
use densleaf_lexer::lex;
use densleaf_parser::parse;

#[test]
fn generation_is_deterministic_and_contains_models_and_controllers() {
    let source = r#"
model User {
id: id
name: string
}
controller UserController {
index() { return "Users" }
}
"#;
    let lexed = lex(source, "app.dl");
    let parsed = parse(lexed.tokens);
    let first = generate_rust(&parsed.program);
    let second = generate_rust(&parsed.program);
    assert_eq!(first, second);
    assert!(first.contains("pub struct User"));
    assert!(first.contains("pub id: i64"));
    assert!(first.contains("pub struct UserController;"));
    assert!(first.contains("pub fn index() -> DensleafValue"));
    assert!(first.contains("DensleafValue::String(\"Users\".to_string())"));
}

#[test]
fn generates_variables_and_structured_literals() {
    let source = r#"
controller UserController {
index() {
    let users = [{ name: "Justin" }]
    let none = null
    return users
}
}
"#;
    let lexed = lex(source, "app.dl");
    let parsed = parse(lexed.tokens);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);

    let generated = generate_rust(&parsed.program);
    assert!(generated.contains("let users: DensleafValue = DensleafValue::Array"));
    assert!(generated.contains("DensleafValue::Object"));
    assert!(generated.contains("let none: DensleafValue = DensleafValue::Null;"));
    assert!(generated.contains("return users.clone().into();"));
}
