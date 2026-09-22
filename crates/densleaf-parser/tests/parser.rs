use densleaf_ast::{Declaration, Statement};
use densleaf_lexer::lex;
use densleaf_parser::{parse, ParseOutput};


fn parse_source(source: &str) -> ParseOutput {
    let lexed = lex(source, "test.dl");
    assert!(lexed.diagnostics.is_empty(), "{:?}", lexed.diagnostics);
    parse(lexed.tokens)
}

#[test]
fn parses_model_declarations() {
    let output = parse_source("model User { id: id name: string }");
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let Declaration::Model(model) = &output.program.declarations[0] else {
        panic!("expected model")
    };
    assert_eq!(model.name, "User");
    assert_eq!(model.fields.len(), 2);
    assert_eq!(model.fields[1].type_reference.name, "string");
}

#[test]
fn parses_controllers_methods_parameters_and_returns() {
    let output = parse_source(
        "controller UserController { show(id: id, verbose: bool) { return true } }",
    );
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let Declaration::Controller(controller) = &output.program.declarations[0] else {
        panic!("expected controller")
    };
    assert_eq!(controller.methods[0].parameters.len(), 2);
    assert!(matches!(
        controller.methods[0].body[0],
        Statement::Return(_)
    ));
}

#[test]
fn reports_malformed_model_fields() {
    let output = parse_source("model User { email: }");
    assert!(!output.diagnostics.is_empty());
    assert!(output.diagnostics[0].message.contains("field type"));
}
