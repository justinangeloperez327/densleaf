use densleaf_ast::{Declaration, Expression, Statement};
use densleaf_lexer::lex;
use densleaf_parser::{ParseOutput, parse};

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
    let output =
        parse_source("controller UserController { show(id: id, verbose: bool) { return true } }");
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
fn parses_variables_arrays_objects_null_and_grouping() {
    let output = parse_source(
        r#"controller Api {
            index() {
                let users = [{ name: "Justin" }, { name: "John" }]
                let empty = []
                let options = {}
                let nothing = null
                return (users)
            }
        }"#,
    );
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);

    let Declaration::Controller(controller) = &output.program.declarations[0] else {
        panic!("expected controller")
    };
    let body = &controller.methods[0].body;
    assert_eq!(body.len(), 5);

    let Statement::Let(users) = &body[0] else {
        panic!("expected let statement")
    };
    let Expression::ArrayLiteral { elements, .. } = &users.initializer else {
        panic!("expected array literal")
    };
    assert_eq!(elements.len(), 2);
    assert!(matches!(elements[0], Expression::ObjectLiteral { .. }));

    let Statement::Let(nothing) = &body[3] else {
        panic!("expected let statement")
    };
    assert!(matches!(
        nothing.initializer,
        Expression::NullLiteral { .. }
    ));

    let Statement::Return(return_statement) = &body[4] else {
        panic!("expected return statement")
    };
    assert!(matches!(
        return_statement.expression,
        Expression::Grouped { .. }
    ));
}

#[test]
fn accepts_trailing_commas_in_arrays_and_objects() {
    let output = parse_source(
        r#"controller Api {
            index() {
                let values = [1, 2, 3,]
                return { values: values, empty: [], }
            }
        }"#,
    );
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn reports_malformed_model_fields() {
    let output = parse_source("model User { email: }");
    assert!(!output.diagnostics.is_empty());
    assert!(output.diagnostics[0].message.contains("field type"));
}
