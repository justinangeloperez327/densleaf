use densleaf_lexer::*;
use densleaf_token::TokenKind;

fn kinds(source: &str) -> Vec<TokenKind> {
    lex(source, "test.dl")
        .tokens
        .into_iter()
        .map(|token| token.kind)
        .collect()
}

#[test]
fn lexes_keywords_identifiers_and_literals() {
    assert_eq!(
        kinds("model User controller Api let value = null return true false \"hello\" 42"),
        vec![
            TokenKind::Model,
            TokenKind::Identifier("User".into()),
            TokenKind::Controller,
            TokenKind::Identifier("Api".into()),
            TokenKind::Let,
            TokenKind::Identifier("value".into()),
            TokenKind::Equal,
            TokenKind::Null,
            TokenKind::Return,
            TokenKind::True,
            TokenKind::False,
            TokenKind::StringLiteral("hello".into()),
            TokenKind::IntegerLiteral(42),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_punctuation_and_operators() {
    assert_eq!(
        kinds("{}()[]:,.= == != ! + - * / % < <= > >= && ||"),
        vec![
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::LeftBracket,
            TokenKind::RightBracket,
            TokenKind::Colon,
            TokenKind::Comma,
            TokenKind::Dot,
            TokenKind::Equal,
            TokenKind::EqualEqual,
            TokenKind::BangEqual,
            TokenKind::Bang,
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::AndAnd,
            TokenKind::OrOr,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn tracks_source_spans_and_comments() {
    let output = lex("// comment\nmodel User {}", "app.dl");
    assert!(output.diagnostics.is_empty());
    let model = &output.tokens[0];
    assert_eq!(model.span.start.line, 2);
    assert_eq!(model.span.start.column, 1);
    assert_eq!(model.span.file, "app.dl");
}

#[test]
fn reports_invalid_characters() {
    let output = lex("model @", "app.dl");
    assert_eq!(output.diagnostics.len(), 1);
    assert!(output.diagnostics[0].message.contains("invalid character"));
}

#[test]
fn reports_incomplete_logical_operators() {
    let output = lex("true & false | true", "app.dl");
    assert_eq!(output.diagnostics.len(), 2);
    assert!(
        output
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.message.contains("expected a second"))
    );
}

#[test]
fn reports_unterminated_strings() {
    let output = lex("return \"hello", "app.dl");
    assert_eq!(output.diagnostics.len(), 1);
    assert!(
        output.diagnostics[0]
            .message
            .contains("unterminated string")
    );
}

#[test]
fn lexes_application_declaration_keywords() {
    assert_eq!(
        kinds("middleware Auth migration CreateUsers policy UserPolicy for User"),
        vec![
            TokenKind::Middleware,
            TokenKind::Identifier("Auth".into()),
            TokenKind::Migration,
            TokenKind::Identifier("CreateUsers".into()),
            TokenKind::Policy,
            TokenKind::Identifier("UserPolicy".into()),
            TokenKind::For,
            TokenKind::Identifier("User".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_messaging_declaration_keywords() {
    assert_eq!(
        kinds(
            "event UserCreated listener SendMail listens UserCreated notification Welcome mail WelcomeMail"
        ),
        vec![
            TokenKind::Event,
            TokenKind::Identifier("UserCreated".into()),
            TokenKind::Listener,
            TokenKind::Identifier("SendMail".into()),
            TokenKind::Listens,
            TokenKind::Identifier("UserCreated".into()),
            TokenKind::Notification,
            TokenKind::Identifier("Welcome".into()),
            TokenKind::Mail,
            TokenKind::Identifier("WelcomeMail".into()),
            TokenKind::Eof,
        ]
    );
}
