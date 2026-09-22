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
fn catches_cross_kind_top_level_name_collisions() {
    let errors = diagnostics("model User { id: id } controller User {}");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("duplicate top-level declaration"))
    );
}

#[test]
fn catches_unknown_types() {
    let errors = diagnostics("model User { age: number }");
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("unknown type `number`"));
}

#[test]
fn accepts_declared_models_as_types_even_when_declared_later() {
    let errors = diagnostics(
        "model Post { author: User } controller Api { show(user: User) { return user } } model User { id: id }",
    );
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn resolves_parameters_locals_and_top_level_symbols() {
    let errors = diagnostics(
        r#"controller Api {
            show(id) {
                let selected = id
                let found = User.find(selected)
                return found
            }
        }
        model User { id: id }"#,
    );
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn catches_unknown_names_duplicate_locals_and_object_keys() {
    let errors = diagnostics(
        r#"controller Api {
            show(id) {
                let selected = id
                let selected = missing
                return { value: selected, value: null }
            }
        }"#,
    );

    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("unknown name `missing`"))
    );
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("duplicate local binding"))
    );
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("duplicate object key"))
    );
}

#[test]
fn validates_middleware_migration_and_policy_contracts() {
    let valid = diagnostics(
        r#"
        model User { id: id }
        middleware Auth { handle(request) { return request } }
        migration CreateUsers { up() {} }
        policy UserPolicy for User { view(actor: User) { return true } }
        "#,
    );
    assert!(valid.is_empty(), "{valid:?}");

    let invalid = diagnostics(
        r#"
        middleware EmptyMiddleware {}
        migration EmptyMigration {}
        policy GhostPolicy for Ghost {}
        "#,
    );
    assert!(
        invalid
            .iter()
            .any(|d| d.message.contains("must define `handle`"))
    );
    assert!(
        invalid
            .iter()
            .any(|d| d.message.contains("must define `up`"))
    );
    assert!(
        invalid
            .iter()
            .any(|d| d.message.contains("targets unknown model `Ghost`"))
    );
}

#[test]
fn application_declarations_share_the_top_level_namespace() {
    let errors =
        diagnostics("model User { id: id } middleware User { handle(request) { return request } }");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("duplicate top-level declaration"))
    );
}

#[test]
fn validates_event_listener_notification_and_mail_contracts() {
    let valid = diagnostics(
        r#"
        model User { id: id }

        event UserCreated { user: User }

        listener SendWelcomeMail listens UserCreated {
            handle(event: UserCreated) { return event }
        }

        notification WelcomeNotification {
            channels() { return ["mail"] }
            message(user: User) { return "Welcome" }
        }

        mail WelcomeMail {
            subject() { return "Welcome" }
            body(user: User) { return "Hello" }
        }
        "#,
    );
    assert!(valid.is_empty(), "{valid:?}");

    let invalid = diagnostics(
        r#"
        event ExistingEvent { value: string }

        listener MissingEventListener listens MissingEvent {
            handle(event) { return event }
        }

        listener MissingHandle listens ExistingEvent {}

        listener WrongHandle listens ExistingEvent {
            handle(event: ExistingEvent) { return event }
        }

        notification EmptyNotification {}

        mail EmptyMail {}
        "#,
    );

    assert!(invalid.iter().any(|d| {
        d.message
            .contains("listens to unknown event `MissingEvent`")
    }));
    assert!(
        invalid
            .iter()
            .any(|d| d.message.contains("must define `handle`"))
    );
    assert!(
        invalid
            .iter()
            .any(|d| d.message.contains("must define `channels`"))
    );
    assert!(
        invalid
            .iter()
            .any(|d| d.message.contains("must define `message`"))
    );
    assert!(
        invalid
            .iter()
            .any(|d| d.message.contains("must define `subject`"))
    );
    assert!(
        invalid
            .iter()
            .any(|d| d.message.contains("must define `body`"))
    );
}

#[test]
fn listener_rejects_wrong_typed_event_parameter() {
    let errors = diagnostics(
        r#"
        event UserCreated { id: id }
        event UserDeleted { id: id }

        listener AuditUserCreated listens UserCreated {
            handle(event: UserDeleted) { return event }
        }
        "#,
    );

    assert!(errors.iter().any(|d| {
        d.message
            .contains("handles `UserCreated`, not `UserDeleted`")
    }));
}

#[test]
fn events_are_valid_user_defined_types() {
    let errors = diagnostics(
        r#"
        event UserCreated { id: id }

        controller Api {
            show(event: UserCreated) { return event }
        }
        "#,
    );
    assert!(errors.is_empty(), "{errors:?}");
}
