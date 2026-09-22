use densleaf_diagnostic::Diagnostic;
use densleaf_lexer::lex;
use densleaf_parser::parse;
use densleaf_semantic::analyze;

fn diagnostics(source: &str) -> Vec<Diagnostic> {
    let lexed = lex(source, "test.dl");
    assert!(lexed.diagnostics.is_empty());
    let parsed = parse(lexed.tokens);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    analyze(&parsed.program)
}

#[test]
fn catches_duplicate_models_and_fields() {
    let errors = diagnostics("model User { name: string name: string } model User { id: id }");
    assert!(errors.iter().any(|d| d.message.contains("duplicate model")));
    assert!(errors.iter().any(|d| d.message.contains("duplicate field")));
}

#[test]
fn catches_duplicate_controllers_methods_and_parameters() {
    let errors = diagnostics(
        "controller Api { show(id, id) { return true } show() { return false } } controller Api {}",
    );
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("duplicate controller `Api`"))
    );
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("duplicate controller method"))
    );
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("duplicate parameter"))
    );
}

#[test]
fn catches_unknown_types() {
    let errors = diagnostics("model User { age: number }");
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("unknown type `number`"));
}
