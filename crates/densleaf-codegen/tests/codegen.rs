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
