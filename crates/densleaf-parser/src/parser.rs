use densleaf_ast::{
    BinaryOperator, ControllerDefinition, Declaration, EventDefinition, Expression, FieldDefinition,
    LetStatement, ListenerDefinition, MailDefinition, MethodDefinition, MiddlewareDefinition,
    MigrationDefinition, ModelDefinition, NotificationDefinition, ObjectEntry, Parameter,
    PolicyDefinition, Program, ReturnStatement, Statement, TypeReference, UnaryOperator,
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
                TokenKind::Middleware => {
                    if let Some(middleware) = self.parse_middleware() {
                        declarations.push(Declaration::Middleware(middleware));
                    }
                }
                TokenKind::Migration => {
                    if let Some(migration) = self.parse_migration() {
                        declarations.push(Declaration::Migration(migration));
                    }
                }
                TokenKind::Policy => {
                    if let Some(policy) = self.parse_policy() {
                        declarations.push(Declaration::Policy(policy));
                    }
                }
                TokenKind::Event => {
                    if let Some(event) = self.parse_event() {
                        declarations.push(Declaration::Event(event));
                    }
                }
                TokenKind::Listener => {
                    if let Some(listener) = self.parse_listener() {
                        declarations.push(Declaration::Listener(listener));
                    }
                }
                TokenKind::Notification => {
                    if let Some(notification) = self.parse_notification() {
                        declarations.push(Declaration::Notification(notification));
                    }
                }
                TokenKind::Mail => {
                    if let Some(mail) = self.parse_mail() {
                        declarations.push(Declaration::Mail(mail));
                    }
                }
                _ => {
                    let span = self.peek().span.clone();
                    self.diagnostics.push(
                        Diagnostic::error("expected a top-level declaration", span).with_help(
                            "start with `model`, `controller`, `middleware`, `migration`, `policy`, `event`, `listener`, `notification`, or `mail`",
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
            if let Some(field) = self.parse_field("model", &name) {
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

    fn parse_field(&mut self, owner_kind: &str, owner_name: &str) -> Option<FieldDefinition> {
        let (name, name_span) = self.expect_identifier("expected a field name")?;
        if !self.expect_simple(TokenKind::Colon, "expected `:` after the field name") {
            return None;
        }
        let (type_name, type_span) = match self.expect_identifier("expected a field type") {
            Some(value) => value,
            None => {
                if let Some(diagnostic) = self.diagnostics.last_mut() {
                    diagnostic.help =
                        Some(format!("try `{name}: string` on {owner_kind} `{owner_name}`"));
                }
                return None;
            }
        };
        let span = name_span.cover(&type_span);
        Some(FieldDefinition {
            name,
            name_span,
            type_reference: TypeReference {
                name: type_name,
                span: type_span,
            },
            span,
        })
    }

    fn parse_event(&mut self) -> Option<EventDefinition> {
        let start = self.advance().span.clone();
        let (name, name_span) = self.expect_identifier("expected an event name")?;
        if !self.expect_simple(TokenKind::LeftBrace, "expected `{` after the event name") {
            return None;
        }

        let mut fields = Vec::new();
        while !self.check_simple(&TokenKind::RightBrace) && !self.is_eof() {
            if let Some(field) = self.parse_field("event", &name) {
                fields.push(field);
            } else {
                self.synchronize_model_member();
            }
        }

        let end =
            self.expect_simple_span(TokenKind::RightBrace, "expected `}` to close the event")?;
        Some(EventDefinition {
            name,
            name_span,
            fields,
            span: start.cover(&end),
        })
    }

    fn parse_listener(&mut self) -> Option<ListenerDefinition> {
        let start = self.advance().span.clone();
        let (name, name_span) = self.expect_identifier("expected a listener name")?;
        if !self.expect_simple(TokenKind::Listens, "expected `listens` after the listener name") {
            return None;
        }
        let (event_name, event_span) =
            self.expect_identifier("expected an event name after `listens`")?;
        if !self.expect_simple(TokenKind::LeftBrace, "expected `{` after the event name") {
            return None;
        }

        let methods = self.parse_method_list("listener")?;
        let end =
            self.expect_simple_span(TokenKind::RightBrace, "expected `}` to close the listener")?;
        Some(ListenerDefinition {
            name,
            name_span,
            event: TypeReference {
                name: event_name,
                span: event_span,
            },
            methods,
            span: start.cover(&end),
        })
    }

    fn parse_notification(&mut self) -> Option<NotificationDefinition> {
        let start = self.advance().span.clone();
        let (name, name_span) = self.expect_identifier("expected a notification name")?;
        if !self.expect_simple(
            TokenKind::LeftBrace,
            "expected `{` after the notification name",
        ) {
            return None;
        }

        let methods = self.parse_method_list("notification")?;
        let end = self.expect_simple_span(
            TokenKind::RightBrace,
            "expected `}` to close the notification",
        )?;
        Some(NotificationDefinition {
            name,
            name_span,
            methods,
            span: start.cover(&end),
        })
    }

    fn parse_mail(&mut self) -> Option<MailDefinition> {
        let start = self.advance().span.clone();
        let (name, name_span) = self.expect_identifier("expected a mail name")?;
        if !self.expect_simple(TokenKind::LeftBrace, "expected `{` after the mail name") {
            return None;
        }

        let methods = self.parse_method_list("mail")?;
        let end =
            self.expect_simple_span(TokenKind::RightBrace, "expected `}` to close the mail")?;
        Some(MailDefinition {
            name,
            name_span,
            methods,
            span: start.cover(&end),
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

    fn parse_middleware(&mut self) -> Option<MiddlewareDefinition> {
        let start = self.advance().span.clone();
        let (name, name_span) = self.expect_identifier("expected a middleware name")?;
        if !self.expect_simple(
            TokenKind::LeftBrace,
            "expected `{` after the middleware name",
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
            "expected `}` to close the middleware",
        )?;
        Some(MiddlewareDefinition {
            name,
            name_span,
            methods,
            span: start.cover(&end),
        })
    }

    fn parse_migration(&mut self) -> Option<MigrationDefinition> {
        let start = self.advance().span.clone();
        let (name, name_span) = self.expect_identifier("expected a migration name")?;
        if !self.expect_simple(
            TokenKind::LeftBrace,
            "expected `{` after the migration name",
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

        let end =
            self.expect_simple_span(TokenKind::RightBrace, "expected `}` to close the migration")?;
        Some(MigrationDefinition {
            name,
            name_span,
            methods,
            span: start.cover(&end),
        })
    }

    fn parse_policy(&mut self) -> Option<PolicyDefinition> {
        let start = self.advance().span.clone();
        let (name, name_span) = self.expect_identifier("expected a policy name")?;
        if !self.expect_simple(TokenKind::For, "expected `for` after the policy name") {
            return None;
        }
        let (target_name, target_span) =
            self.expect_identifier("expected a model name after `for`")?;
        if !self.expect_simple(TokenKind::LeftBrace, "expected `{` after the policy target") {
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

        let end =
            self.expect_simple_span(TokenKind::RightBrace, "expected `}` to close the policy")?;
        Some(PolicyDefinition {
            name,
            name_span,
            target: TypeReference {
                name: target_name,
                span: target_span,
            },
            methods,
            span: start.cover(&end),
        })
    }

    fn parse_method_list(&mut self, declaration_kind: &str) -> Option<Vec<MethodDefinition>> {
        let mut methods = Vec::new();
        while !self.check_simple(&TokenKind::RightBrace) && !self.is_eof() {
            if let Some(method) = self.parse_controller_method() {
                methods.push(method);
            } else {
                self.synchronize_controller_member();
                if self.is_eof() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            format!("unterminated {declaration_kind} declaration"),
                            self.peek().span.clone(),
                        )
                        .with_help("close the declaration with `}`"),
                    );
                    return None;
                }
            }
        }
        Some(methods)
    }

    fn parse_controller_method(&mut self) -> Option<MethodDefinition> {
        let (name, name_span) = self.expect_identifier("expected a method name")?;
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

        Some(MethodDefinition {
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
                TokenKind::Let => {
                    if let Some(statement) = self.parse_let_statement() {
                        statements.push(Statement::Let(statement));
                    }
                }
                TokenKind::Return => {
                    if let Some(statement) = self.parse_return_statement() {
                        statements.push(Statement::Return(statement));
                    }
                }
                _ => {
                    let span = self.peek().span.clone();
                    self.diagnostics.push(
                        Diagnostic::error("expected a statement", span)
                            .with_help("start a statement with `let` or `return`"),
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

    fn parse_let_statement(&mut self) -> Option<LetStatement> {
        let start = self.advance().span.clone();
        let (name, name_span) = self.expect_identifier("expected a variable name after `let`")?;
        if !self.expect_simple(TokenKind::Equal, "expected `=` after the variable name") {
            return None;
        }
        let initializer = self.parse_expression()?;
        let span = start.cover(initializer.span());

        Some(LetStatement {
            name,
            name_span,
            initializer,
            span,
        })
    }

    fn parse_return_statement(&mut self) -> Option<ReturnStatement> {
        let start = self.advance().span.clone();
        let expression = self.parse_expression()?;
        let span = start.cover(expression.span());
        Some(ReturnStatement { expression, span })
    }

    fn parse_expression(&mut self) -> Option<Expression> {
        self.parse_or_expression()
    }

    fn parse_or_expression(&mut self) -> Option<Expression> {
        let mut expression = self.parse_and_expression()?;

        while self.match_simple(&TokenKind::OrOr) {
            let right = self.parse_and_expression()?;
            let span = expression.span().cover(right.span());
            expression = Expression::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::Or,
                right: Box::new(right),
                span,
            };
        }

        Some(expression)
    }

    fn parse_and_expression(&mut self) -> Option<Expression> {
        let mut expression = self.parse_equality_expression()?;

        while self.match_simple(&TokenKind::AndAnd) {
            let right = self.parse_equality_expression()?;
            let span = expression.span().cover(right.span());
            expression = Expression::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::And,
                right: Box::new(right),
                span,
            };
        }

        Some(expression)
    }

    fn parse_equality_expression(&mut self) -> Option<Expression> {
        let mut expression = self.parse_comparison_expression()?;

        loop {
            let operator = if self.match_simple(&TokenKind::EqualEqual) {
                Some(BinaryOperator::Equal)
            } else if self.match_simple(&TokenKind::BangEqual) {
                Some(BinaryOperator::NotEqual)
            } else {
                None
            };

            let Some(operator) = operator else {
                break;
            };

            let right = self.parse_comparison_expression()?;
            let span = expression.span().cover(right.span());
            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
                span,
            };
        }

        Some(expression)
    }

    fn parse_comparison_expression(&mut self) -> Option<Expression> {
        let mut expression = self.parse_additive_expression()?;

        loop {
            let operator = if self.match_simple(&TokenKind::Less) {
                Some(BinaryOperator::Less)
            } else if self.match_simple(&TokenKind::LessEqual) {
                Some(BinaryOperator::LessEqual)
            } else if self.match_simple(&TokenKind::Greater) {
                Some(BinaryOperator::Greater)
            } else if self.match_simple(&TokenKind::GreaterEqual) {
                Some(BinaryOperator::GreaterEqual)
            } else {
                None
            };

            let Some(operator) = operator else {
                break;
            };

            let right = self.parse_additive_expression()?;
            let span = expression.span().cover(right.span());
            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
                span,
            };
        }

        Some(expression)
    }

    fn parse_additive_expression(&mut self) -> Option<Expression> {
        let mut expression = self.parse_multiplicative_expression()?;

        loop {
            let operator = if self.match_simple(&TokenKind::Plus) {
                Some(BinaryOperator::Add)
            } else if self.match_simple(&TokenKind::Minus) {
                Some(BinaryOperator::Subtract)
            } else {
                None
            };

            let Some(operator) = operator else {
                break;
            };

            let right = self.parse_multiplicative_expression()?;
            let span = expression.span().cover(right.span());
            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
                span,
            };
        }

        Some(expression)
    }

    fn parse_multiplicative_expression(&mut self) -> Option<Expression> {
        let mut expression = self.parse_unary_expression()?;

        loop {
            let operator = if self.match_simple(&TokenKind::Star) {
                Some(BinaryOperator::Multiply)
            } else if self.match_simple(&TokenKind::Slash) {
                Some(BinaryOperator::Divide)
            } else if self.match_simple(&TokenKind::Percent) {
                Some(BinaryOperator::Remainder)
            } else {
                None
            };

            let Some(operator) = operator else {
                break;
            };

            let right = self.parse_unary_expression()?;
            let span = expression.span().cover(right.span());
            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
                span,
            };
        }

        Some(expression)
    }

    fn parse_unary_expression(&mut self) -> Option<Expression> {
        let (operator, start) = if self.check_simple(&TokenKind::Bang) {
            (UnaryOperator::Not, self.advance().span.clone())
        } else if self.check_simple(&TokenKind::Minus) {
            (UnaryOperator::Negate, self.advance().span.clone())
        } else {
            return self.parse_postfix_expression();
        };

        let operand = self.parse_unary_expression()?;
        let span = start.cover(operand.span());
        Some(Expression::Unary {
            operator,
            operand: Box::new(operand),
            span,
        })
    }

    fn parse_postfix_expression(&mut self) -> Option<Expression> {
        let mut expression = self.parse_primary_expression()?;

        loop {
            if self.match_simple(&TokenKind::Dot) {
                let (member, member_span) =
                    self.expect_identifier("expected a member name after `.`")?;
                let span = expression.span().cover(&member_span);
                expression = Expression::MemberAccess {
                    object: Box::new(expression),
                    member,
                    member_span,
                    span,
                };
                continue;
            }

            if self.match_simple(&TokenKind::LeftParen) {
                expression = self.finish_call_expression(expression)?;
                continue;
            }

            if self.match_simple(&TokenKind::LeftBracket) {
                let index = self.parse_expression()?;
                let end = self.expect_simple_span(
                    TokenKind::RightBracket,
                    "expected `]` after the index expression",
                )?;
                let span = expression.span().cover(&end);
                expression = Expression::Index {
                    object: Box::new(expression),
                    index: Box::new(index),
                    span,
                };
                continue;
            }

            break;
        }

        Some(expression)
    }

    fn finish_call_expression(&mut self, callee: Expression) -> Option<Expression> {
        let mut arguments = Vec::new();

        while !self.check_simple(&TokenKind::RightParen) && !self.is_eof() {
            arguments.push(self.parse_expression()?);

            if !self.match_simple(&TokenKind::Comma) {
                break;
            }
        }

        let end = self.expect_simple_span(
            TokenKind::RightParen,
            "expected `)` after the call arguments",
        )?;
        let span = callee.span().cover(&end);

        Some(Expression::Call {
            callee: Box::new(callee),
            arguments,
            span,
        })
    }

    fn parse_primary_expression(&mut self) -> Option<Expression> {
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
            TokenKind::Null => Some(Expression::NullLiteral { span: token.span }),
            TokenKind::Identifier(name) => Some(Expression::Identifier {
                name,
                span: token.span,
            }),
            TokenKind::LeftParen => self.parse_grouped_expression(token.span),
            TokenKind::LeftBracket => self.parse_array_literal(token.span),
            TokenKind::LeftBrace => self.parse_object_literal(token.span),
            _ => {
                self.diagnostics.push(
                    Diagnostic::error("expected an expression", token.span).with_help(
                        "use a literal, identifier, array `[]`, object `{}`, or grouped expression",
                    ),
                );
                None
            }
        }
    }

    fn parse_grouped_expression(&mut self, start: Span) -> Option<Expression> {
        let expression = self.parse_expression()?;
        let end = self.expect_simple_span(
            TokenKind::RightParen,
            "expected `)` after the grouped expression",
        )?;
        Some(Expression::Grouped {
            expression: Box::new(expression),
            span: start.cover(&end),
        })
    }

    fn parse_array_literal(&mut self, start: Span) -> Option<Expression> {
        let mut elements = Vec::new();

        while !self.check_simple(&TokenKind::RightBracket) && !self.is_eof() {
            elements.push(self.parse_expression()?);

            if !self.match_simple(&TokenKind::Comma) {
                break;
            }
        }

        let end = self.expect_simple_span(
            TokenKind::RightBracket,
            "expected `]` to close the array literal",
        )?;
        Some(Expression::ArrayLiteral {
            elements,
            span: start.cover(&end),
        })
    }

    fn parse_object_literal(&mut self, start: Span) -> Option<Expression> {
        let mut entries = Vec::new();

        while !self.check_simple(&TokenKind::RightBrace) && !self.is_eof() {
            let (key, key_span) = self.expect_identifier("expected an object key")?;
            if !self.expect_simple(TokenKind::Colon, "expected `:` after the object key") {
                return None;
            }
            let value = self.parse_expression()?;
            let span = key_span.cover(value.span());
            entries.push(ObjectEntry {
                key,
                key_span,
                value,
                span,
            });

            if !self.match_simple(&TokenKind::Comma) {
                break;
            }
        }

        let end = self.expect_simple_span(
            TokenKind::RightBrace,
            "expected `}` to close the object literal",
        )?;
        Some(Expression::ObjectLiteral {
            entries,
            span: start.cover(&end),
        })
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
