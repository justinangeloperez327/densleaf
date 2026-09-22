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

#[test]
fn lowers_operators_member_calls_and_indexes() {
    let source = r#"
model User { id: id }
controller UserController {
show(id: id) {
    let score = (10 + 5) * 2
    let allowed = score >= 20 && !false
    let first = [{ name: "Justin" }][0].name
    let found = User.find(id)
    return found != null || allowed
}
}
"#;
    let lexed = lex(source, "app.dl");
    let parsed = parse(lexed.tokens);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);

    let generated = generate_rust(&parsed.program);
    assert!(generated.contains("densleaf_multiply"));
    assert!(generated.contains("densleaf_add"));
    assert!(generated.contains("densleaf_greater_equal"));
    assert!(generated.contains("densleaf_and"));
    assert!(generated.contains("densleaf_not"));
    assert!(generated.contains("densleaf_index"));
    assert!(generated.contains("densleaf_get_member"));
    assert!(generated.contains("densleaf_call_member"));
    assert!(generated.contains("DensleafValue::Type(\"User\".to_string())"));
    assert!(generated.contains("densleaf_not_equal"));
    assert!(generated.contains("densleaf_or"));
}

#[test]
fn generates_application_declaration_metadata_and_methods() {
    let source = r#"
model User { id: id }

middleware AuthMiddleware {
    handle(request) { return request }
}

migration CreateUsers {
    up() {}
}

policy UserPolicy for User {
    view(actor: User, target: User) { return true }
}
"#;
    let lexed = lex(source, "app.dl");
    let parsed = parse(lexed.tokens);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);

    let generated = generate_rust(&parsed.program);
    assert!(generated.contains("pub struct AuthMiddleware;"));
    assert!(generated.contains("DENSLEAF_KIND: &'static str = \"middleware\""));
    assert!(generated.contains("pub fn handle"));
    assert!(generated.contains("pub struct CreateUsers;"));
    assert!(generated.contains("DENSLEAF_KIND: &'static str = \"migration\""));
    assert!(generated.contains("pub fn up"));
    assert!(generated.contains("pub struct UserPolicy;"));
    assert!(generated.contains("DENSLEAF_KIND: &'static str = \"policy\""));
    assert!(generated.contains("TARGET_MODEL: &'static str = \"User\""));
    assert!(generated.contains("pub fn view"));
}

#[test]
fn generates_messaging_declaration_metadata_and_event_data() {
    let source = r#"
model User { id: id }

event UserCreated {
    user: User
}

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
"#;
    let lexed = lex(source, "messaging.dl");
    let parsed = parse(lexed.tokens);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);

    let generated = generate_rust(&parsed.program);
    assert!(generated.contains("pub struct UserCreated"));
    assert!(generated.contains("DENSLEAF_KIND: &'static str = \"event\""));
    assert!(generated.contains("pub struct SendWelcomeMail;"));
    assert!(generated.contains("DENSLEAF_KIND: &'static str = \"listener\""));
    assert!(generated.contains("EVENT_NAME: &'static str = \"UserCreated\""));
    assert!(generated.contains("pub struct WelcomeNotification;"));
    assert!(generated.contains("DENSLEAF_KIND: &'static str = \"notification\""));
    assert!(generated.contains("pub fn channels"));
    assert!(generated.contains("pub fn message"));
    assert!(generated.contains("pub struct WelcomeMail;"));
    assert!(generated.contains("DENSLEAF_KIND: &'static str = \"mail\""));
    assert!(generated.contains("pub fn subject"));
    assert!(generated.contains("pub fn body"));
}
