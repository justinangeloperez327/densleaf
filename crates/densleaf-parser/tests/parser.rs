use densleaf_ast::{BinaryOperator, Declaration, Expression, Statement, UnaryOperator};
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
fn parses_operator_precedence() {
    let output =
        parse_source("controller Api { index() { return 1 + 2 * 3 >= 7 && !false || false } }");
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);

    let Declaration::Controller(controller) = &output.program.declarations[0] else {
        panic!("expected controller")
    };
    let Statement::Return(return_statement) = &controller.methods[0].body[0] else {
        panic!("expected return")
    };

    let Expression::Binary {
        operator: BinaryOperator::Or,
        left,
        ..
    } = &return_statement.expression
    else {
        panic!("expected logical OR at the root")
    };

    let Expression::Binary {
        operator: BinaryOperator::And,
        left: comparison,
        right,
        ..
    } = left.as_ref()
    else {
        panic!("expected logical AND before OR")
    };
    assert!(matches!(
        right.as_ref(),
        Expression::Unary {
            operator: UnaryOperator::Not,
            ..
        }
    ));

    let Expression::Binary {
        operator: BinaryOperator::GreaterEqual,
        left: additive,
        ..
    } = comparison.as_ref()
    else {
        panic!("expected comparison")
    };

    let Expression::Binary {
        operator: BinaryOperator::Add,
        right: multiplied,
        ..
    } = additive.as_ref()
    else {
        panic!("expected addition")
    };
    assert!(matches!(
        multiplied.as_ref(),
        Expression::Binary {
            operator: BinaryOperator::Multiply,
            ..
        }
    ));
}

#[test]
fn parses_member_calls_indexes_and_chains() {
    let output = parse_source(
        r#"controller Api {
            show(id) {
                let result = User.find(id).items[0].name
                return result
            }
        }"#,
    );
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);

    let Declaration::Controller(controller) = &output.program.declarations[0] else {
        panic!("expected controller")
    };
    let Statement::Let(result) = &controller.methods[0].body[0] else {
        panic!("expected let statement")
    };

    let Expression::MemberAccess { member, object, .. } = &result.initializer else {
        panic!("expected final member access")
    };
    assert_eq!(member, "name");

    let Expression::Index { object, .. } = object.as_ref() else {
        panic!("expected index before final member")
    };
    assert!(matches!(
        object.as_ref(),
        Expression::MemberAccess { member, .. } if member == "items"
    ));
}

#[test]
fn accepts_trailing_commas_in_arrays_objects_and_calls() {
    let output = parse_source(
        r#"controller Api {
            index() {
                let values = [1, 2, 3,]
                let result = User.find(1, true,)
                return { values: values, result: result, }
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

#[test]
fn parses_middleware_migration_and_policy_declarations() {
    let output = parse_source(
        r#"
        model User { id: id }

        middleware AuthMiddleware {
            handle(request) { return request }
        }

        migration CreateUsers {
            up() {}
            down() {}
        }

        policy UserPolicy for User {
            view(actor: User, target: User) { return true }
        }
        "#,
    );
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert!(matches!(
        output.program.declarations[1],
        Declaration::Middleware(_)
    ));
    assert!(matches!(
        output.program.declarations[2],
        Declaration::Migration(_)
    ));

    let Declaration::Policy(policy) = &output.program.declarations[3] else {
        panic!("expected policy")
    };
    assert_eq!(policy.target.name, "User");
    assert_eq!(policy.methods[0].name, "view");
}
