use densleaf_ast::{
    ControllerDefinition, ControllerMethod, Declaration, Expression, ModelDefinition, ModelField,
    Parameter, Program, ReturnStatement, Statement, TypeReference,
};
use densleaf_diagnostic::Diagnostic;
use densleaf_token::{Span, Token, TokenKind};

#[derive(Debug, Clone)]
pub struct ParseOutput {
    pub program: Program,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse(tokens: Vec<Token>) -> ParseOutput {
    Parser::new(tokens).parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            diagnostics: Vec::new(),
        }
    }

    fn parse_program(mut self) -> ParseOutput {
        let mut declarations = Vec::new();

        while !self.is_eof() {
            match self.peek_kind() {
                TokenKind::Model => {
                    if let Some(model) = self.parse_model() {
                        declarations.push(Declaration::Model(model));
                    }
                }
                TokenKind::Controller => {
                    if let Some(controller) = self.parse_controller() {
                        declarations.push(Declaration::Controller(controller));
                    }
                }
                _ => {
                    let span = self.peek().span.clone();
                    self.diagnostics.push(
                        Diagnostic::error("expected `model` or `controller` declaration", span)
                            .with_help(
                                "start a top-level declaration with `model` or `controller`",
                            ),
                    );
                    self.advance();
                }
            }
        }

        ParseOutput {
            program: Program { declarations },
            diagnostics: self.diagnostics,
        }
    }

    fn parse_model(&mut self) -> Option<ModelDefinition> {
        let start = self.advance().span.clone();
        let (name, name_span) = self.expect_identifier("expected a model name")?;
        if !self.expect_simple(TokenKind::LeftBrace, "expected `{` after the model name") {
            return None;
        }

        let mut fields = Vec::new();
        while !self.check_simple(&TokenKind::RightBrace) && !self.is_eof() {
            if let Some(field) = self.parse_model_field(&name) {
                fields.push(field);
            } else {
                self.synchronize_model_member();
            }
        }

        let end =
            self.expect_simple_span(TokenKind::RightBrace, "expected `}` to close the model")?;
        Some(ModelDefinition {
            name,
            name_span,
            fields,
            span: start.cover(&end),
        })
    }

    fn parse_model_field(&mut self, model_name: &str) -> Option<ModelField> {
        let (name, name_span) = self.expect_identifier("expected a field name")?;
        if !self.expect_simple(TokenKind::Colon, "expected `:` after the field name") {
            return None;
        }
        let (type_name, type_span) = match self.expect_identifier("expected a field type") {
            Some(value) => value,
            None => {
                if let Some(diagnostic) = self.diagnostics.last_mut() {
                    diagnostic.help = Some(format!("try `{name}: string` on `{model_name}`"));
                }
                return None;
            }
        };
        let span = name_span.cover(&type_span);
        Some(ModelField {
            name,
            name_span,
            type_reference: TypeReference {
                name: type_name,
                span: type_span,
            },
            span,
        })
    }

    fn parse_controller(&mut self) -> Option<ControllerDefinition> {
        let start = self.advance().span.clone();
        let (name, name_span) = self.expect_identifier("expected a controller name")?;
        if !self.expect_simple(
            TokenKind::LeftBrace,
            "expected `{` after the controller name",
        ) {
            return None;
        }

        let mut methods = Vec::new();
        while !self.check_simple(&TokenKind::RightBrace) && !self.is_eof() {
            if let Some(method) = self.parse_controller_method() {
                methods.push(method);
            } else {
                self.synchronize_controller_member();
            }
        }

        let end = self.expect_simple_span(
            TokenKind::RightBrace,
            "expected `}` to close the controller",
        )?;
        Some(ControllerDefinition {
            name,
            name_span,
            methods,
            span: start.cover(&end),
        })
    }

    fn parse_controller_method(&mut self) -> Option<ControllerMethod> {
        let (name, name_span) = self.expect_identifier("expected a controller method name")?;
        if !self.expect_simple(TokenKind::LeftParen, "expected `(` after the method name") {
            return None;
        }

        let parameters = self.parse_parameters()?;
        self.expect_simple_span(
            TokenKind::RightParen,
            "expected `)` after the method parameters",
        )?;
        let (body, block_span) = self.parse_block()?;
        let method_span = name_span.cover(&block_span);

        Some(ControllerMethod {
            name,
            name_span,
            parameters,
            body,
            span: method_span,
        })
    }

    fn parse_parameters(&mut self) -> Option<Vec<Parameter>> {
        let mut parameters = Vec::new();
        if self.check_simple(&TokenKind::RightParen) {
            return Some(parameters);
        }

        loop {
            let (name, name_span) = self.expect_identifier("expected a parameter name")?;
            let mut parameter_span = name_span.clone();
            let type_reference = if self.match_simple(&TokenKind::Colon) {
                let (type_name, type_span) = self.expect_identifier("expected a parameter type")?;
                parameter_span = name_span.cover(&type_span);
                Some(TypeReference {
                    name: type_name,
                    span: type_span,
                })
            } else {
                None
            };
            parameters.push(Parameter {
                name,
                name_span,
                type_reference,
                span: parameter_span,
            });

            if !self.match_simple(&TokenKind::Comma) {
                break;
            }
        }

        Some(parameters)
    }

    fn parse_block(&mut self) -> Option<(Vec<Statement>, Span)> {
        let start = self.expect_simple_span(
            TokenKind::LeftBrace,
            "expected `{` to start the method body",
        )?;
        let mut statements = Vec::new();

        while !self.check_simple(&TokenKind::RightBrace) && !self.is_eof() {
            match self.peek_kind() {
                TokenKind::Return => {
                    if let Some(statement) = self.parse_return_statement() {
                        statements.push(Statement::Return(statement));
                    }
                }
                _ => {
                    let span = self.peek().span.clone();
                    self.diagnostics.push(
                        Diagnostic::error("expected a `return` statement", span).with_help(
                            "controller methods currently support only basic `return` statements",
                        ),
                    );
                    self.advance();
                }
            }
        }

        let end = self.expect_simple_span(
            TokenKind::RightBrace,
            "expected `}` to close the method body",
        )?;
        Some((statements, start.cover(&end)))
    }

    fn parse_return_statement(&mut self) -> Option<ReturnStatement> {
        let start = self.advance().span.clone();
        let expression = self.parse_expression()?;
        let span = start.cover(expression.span());
        Some(ReturnStatement { expression, span })
    }

    fn parse_expression(&mut self) -> Option<Expression> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::StringLiteral(value) => Some(Expression::StringLiteral {
                value,
                span: token.span,
            }),
            TokenKind::IntegerLiteral(value) => Some(Expression::IntegerLiteral {
                value,
                span: token.span,
            }),
            TokenKind::True => Some(Expression::BooleanLiteral {
                value: true,
                span: token.span,
            }),
            TokenKind::False => Some(Expression::BooleanLiteral {
                value: false,
                span: token.span,
            }),
            TokenKind::Identifier(name) => Some(Expression::Identifier {
                name,
                span: token.span,
            }),
            _ => {
                self.diagnostics.push(
                    Diagnostic::error("expected a value after `return`", token.span)
                        .with_help("return a string, integer, boolean, or identifier"),
                );
                None
            }
        }
    }

    fn synchronize_model_member(&mut self) {
        while !self.is_eof() && !self.check_simple(&TokenKind::RightBrace) {
            if matches!(self.peek_kind(), TokenKind::Identifier(_)) {
                return;
            }
            self.advance();
        }
    }

    fn synchronize_controller_member(&mut self) {
        let mut depth = 0usize;
        while !self.is_eof() {
            match self.peek_kind() {
                TokenKind::LeftBrace => {
                    depth += 1;
                    self.advance();
                }
                TokenKind::RightBrace if depth > 0 => {
                    depth -= 1;
                    self.advance();
                }
                TokenKind::RightBrace => return,
                TokenKind::Identifier(_) if depth == 0 => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn expect_identifier(&mut self, message: &str) -> Option<(String, Span)> {
        let token = self.advance().clone();
        if let TokenKind::Identifier(name) = token.kind {
            Some((name, token.span))
        } else {
            self.diagnostics
                .push(Diagnostic::error(message, token.span.clone()));
            None
        }
    }

    fn expect_simple(&mut self, expected: TokenKind, message: &str) -> bool {
        self.expect_simple_span(expected, message).is_some()
    }

    fn expect_simple_span(&mut self, expected: TokenKind, message: &str) -> Option<Span> {
        if self.check_simple(&expected) {
            Some(self.advance().span.clone())
        } else {
            self.diagnostics
                .push(Diagnostic::error(message, self.peek().span.clone()));
            None
        }
    }

    fn match_simple(&mut self, expected: &TokenKind) -> bool {
        if self.check_simple(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check_simple(&self, expected: &TokenKind) -> bool {
        std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(expected)
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current.min(self.tokens.len().saturating_sub(1))]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn advance(&mut self) -> &Token {
        let index = self.current.min(self.tokens.len().saturating_sub(1));
        if !matches!(self.tokens[index].kind, TokenKind::Eof) {
            self.current += 1;
        }
        &self.tokens[index]
    }
}
