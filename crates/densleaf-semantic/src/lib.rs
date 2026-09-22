use std::collections::{HashMap, HashSet};

use densleaf_ast::{
    ControllerDefinition, Declaration, EventDefinition, Expression, FieldDefinition,
    ListenerDefinition, MailDefinition, MethodDefinition, MiddlewareDefinition,
    MigrationDefinition, ModelDefinition, NotificationDefinition, PolicyDefinition, Program,
    Statement, TypeReference,
};
use densleaf_diagnostic::Diagnostic;
use densleaf_token::Span;

const BUILTIN_TYPES: &[&str] = &["id", "int", "bool", "string"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GlobalKind {
    Model,
    Controller,
    Middleware,
    Migration,
    Policy,
    Event,
    Listener,
    Notification,
    Mail,
}

impl GlobalKind {
    fn label(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Controller => "controller",
            Self::Middleware => "middleware",
            Self::Migration => "migration",
            Self::Policy => "policy",
            Self::Event => "event",
            Self::Listener => "listener",
            Self::Notification => "notification",
            Self::Mail => "mail",
        }
    }
}

#[derive(Debug, Clone)]
struct GlobalSymbol {
    kind: GlobalKind,
    span: Span,
}

pub fn analyze(program: &Program) -> Vec<Diagnostic> {
    let mut analyzer = Analyzer::default();
    analyzer.analyze_program(program);
    analyzer.diagnostics
}

#[derive(Default)]
struct Analyzer {
    diagnostics: Vec<Diagnostic>,
    globals: HashMap<String, GlobalSymbol>,
    model_names: HashSet<String>,
    event_names: HashSet<String>,
}

impl Analyzer {
    fn analyze_program(&mut self, program: &Program) {
        self.collect_globals(program);

        for declaration in &program.declarations {
            match declaration {
                Declaration::Model(model) => self.analyze_model(model),
                Declaration::Controller(controller) => self.analyze_controller(controller),
                Declaration::Middleware(middleware) => self.analyze_middleware(middleware),
                Declaration::Migration(migration) => self.analyze_migration(migration),
                Declaration::Policy(policy) => self.analyze_policy(policy),
                Declaration::Event(event) => self.analyze_event(event),
                Declaration::Listener(listener) => self.analyze_listener(listener),
                Declaration::Notification(notification) => self.analyze_notification(notification),
                Declaration::Mail(mail) => self.analyze_mail(mail),
            }
        }
    }

    fn collect_globals(&mut self, program: &Program) {
        for declaration in &program.declarations {
            let (name, name_span, kind) = match declaration {
                Declaration::Model(model) => {
                    self.model_names.insert(model.name.clone());
                    (&model.name, &model.name_span, GlobalKind::Model)
                }
                Declaration::Controller(controller) => (
                    &controller.name,
                    &controller.name_span,
                    GlobalKind::Controller,
                ),
                Declaration::Middleware(middleware) => (
                    &middleware.name,
                    &middleware.name_span,
                    GlobalKind::Middleware,
                ),
                Declaration::Migration(migration) => {
                    (&migration.name, &migration.name_span, GlobalKind::Migration)
                }
                Declaration::Policy(policy) => {
                    (&policy.name, &policy.name_span, GlobalKind::Policy)
                }
                Declaration::Event(event) => {
                    self.event_names.insert(event.name.clone());
                    (&event.name, &event.name_span, GlobalKind::Event)
                }
                Declaration::Listener(listener) => {
                    (&listener.name, &listener.name_span, GlobalKind::Listener)
                }
                Declaration::Notification(notification) => (
                    &notification.name,
                    &notification.name_span,
                    GlobalKind::Notification,
                ),
                Declaration::Mail(mail) => (&mail.name, &mail.name_span, GlobalKind::Mail),
            };

            if let Some(previous) = self.globals.get(name) {
                let message = if previous.kind == kind {
                    format!("duplicate {} `{name}`", kind.label())
                } else {
                    format!("duplicate top-level declaration `{name}`")
                };

                self.diagnostics
                    .push(
                        Diagnostic::error(message, name_span.clone()).with_note(format!(
                            "`{name}` was already declared as a {} at {}:{}:{}",
                            previous.kind.label(),
                            previous.span.file,
                            previous.span.start.line,
                            previous.span.start.column
                        )),
                    );
            } else {
                self.globals.insert(
                    name.clone(),
                    GlobalSymbol {
                        kind,
                        span: name_span.clone(),
                    },
                );
            }
        }
    }

    fn analyze_model(&mut self, model: &ModelDefinition) {
        self.analyze_fields("model", &model.name, &model.fields);
    }

    fn analyze_event(&mut self, event: &EventDefinition) {
        self.analyze_fields("event", &event.name, &event.fields);
    }

    fn analyze_fields(&mut self, kind: &str, owner_name: &str, fields: &[FieldDefinition]) {
        let mut field_names: HashMap<&str, &Span> = HashMap::new();

        for field in fields {
            if let Some(previous) = field_names.insert(&field.name, &field.name_span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        format!("duplicate field `{}` on {kind}", field.name),
                        field.name_span.clone(),
                    )
                    .with_note(format!(
                        "`{}` is already declared on `{}` at {}:{}:{}",
                        field.name,
                        owner_name,
                        previous.file,
                        previous.start.line,
                        previous.start.column
                    )),
                );
            }
            self.check_type(&field.type_reference);
        }
    }

    fn analyze_controller(&mut self, controller: &ControllerDefinition) {
        self.analyze_methods("controller", &controller.name, &controller.methods);
    }

    fn analyze_middleware(&mut self, middleware: &MiddlewareDefinition) {
        self.analyze_methods("middleware", &middleware.name, &middleware.methods);

        if !middleware
            .methods
            .iter()
            .any(|method| method.name == "handle")
        {
            self.diagnostics.push(
                Diagnostic::error(
                    format!("middleware `{}` must define `handle`", middleware.name),
                    middleware.name_span.clone(),
                )
                .with_help("add `handle(request) { ... }` as the middleware entry point"),
            );
        }
    }

    fn analyze_migration(&mut self, migration: &MigrationDefinition) {
        self.analyze_methods("migration", &migration.name, &migration.methods);

        if !migration.methods.iter().any(|method| method.name == "up") {
            self.diagnostics.push(
                Diagnostic::error(
                    format!("migration `{}` must define `up`", migration.name),
                    migration.name_span.clone(),
                )
                .with_help("add `up() { ... }` to describe the forward migration"),
            );
        }
    }

    fn analyze_policy(&mut self, policy: &PolicyDefinition) {
        if !self.model_names.contains(&policy.target.name) {
            self.diagnostics.push(
                Diagnostic::error(
                    format!(
                        "policy `{}` targets unknown model `{}`",
                        policy.name, policy.target.name
                    ),
                    policy.target.span.clone(),
                )
                .with_help("the name after `for` must refer to a declared model"),
            );
        }

        self.analyze_methods("policy", &policy.name, &policy.methods);
    }

    fn analyze_listener(&mut self, listener: &ListenerDefinition) {
        if !self.event_names.contains(&listener.event.name) {
            self.diagnostics.push(
                Diagnostic::error(
                    format!(
                        "listener `{}` listens to unknown event `{}`",
                        listener.name, listener.event.name
                    ),
                    listener.event.span.clone(),
                )
                .with_help("the name after `listens` must refer to a declared event"),
            );
        }

        self.analyze_methods("listener", &listener.name, &listener.methods);

        let handle = listener
            .methods
            .iter()
            .find(|method| method.name == "handle");
        let Some(handle) = handle else {
            self.diagnostics.push(
                Diagnostic::error(
                    format!("listener `{}` must define `handle`", listener.name),
                    listener.name_span.clone(),
                )
                .with_help("add `handle(event) { ... }` as the listener entry point"),
            );
            return;
        };

        let Some(event_parameter) = handle.parameters.first() else {
            self.diagnostics.push(
                Diagnostic::error(
                    format!(
                        "listener `{}` handle method must accept the event",
                        listener.name
                    ),
                    handle.name_span.clone(),
                )
                .with_help(format!(
                    "add a first parameter such as `event: {}`",
                    listener.event.name
                )),
            );
            return;
        };

        if let Some(type_reference) = &event_parameter.type_reference
            && type_reference.name != listener.event.name
        {
            self.diagnostics.push(
                Diagnostic::error(
                    format!(
                        "listener `{}` handles `{}`, not `{}`",
                        listener.name, listener.event.name, type_reference.name
                    ),
                    type_reference.span.clone(),
                )
                .with_help(format!(
                    "type the first handle parameter as `{}` or leave it inferred",
                    listener.event.name
                )),
            );
        }
    }

    fn analyze_notification(&mut self, notification: &NotificationDefinition) {
        self.analyze_methods("notification", &notification.name, &notification.methods);
        self.require_method(
            "notification",
            &notification.name,
            &notification.name_span,
            &notification.methods,
            "channels",
            "add `channels() { return [...] }` to declare delivery channels",
        );
        self.require_method(
            "notification",
            &notification.name,
            &notification.name_span,
            &notification.methods,
            "message",
            "add `message(...) { ... }` to define notification content",
        );
    }

    fn analyze_mail(&mut self, mail: &MailDefinition) {
        self.analyze_methods("mail", &mail.name, &mail.methods);
        self.require_method(
            "mail",
            &mail.name,
            &mail.name_span,
            &mail.methods,
            "subject",
            "add `subject() { ... }` to define the mail subject",
        );
        self.require_method(
            "mail",
            &mail.name,
            &mail.name_span,
            &mail.methods,
            "body",
            "add `body(...) { ... }` to define the mail body",
        );
    }

    fn require_method(
        &mut self,
        kind: &str,
        owner_name: &str,
        owner_span: &Span,
        methods: &[MethodDefinition],
        method_name: &str,
        help: &str,
    ) {
        if !methods.iter().any(|method| method.name == method_name) {
            self.diagnostics.push(
                Diagnostic::error(
                    format!("{kind} `{owner_name}` must define `{method_name}`"),
                    owner_span.clone(),
                )
                .with_help(help),
            );
        }
    }

    fn analyze_methods(&mut self, kind: &str, owner_name: &str, methods: &[MethodDefinition]) {
        let mut method_names: HashMap<&str, &Span> = HashMap::new();

        for method in methods {
            if let Some(previous) = method_names.insert(&method.name, &method.name_span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        format!("duplicate {kind} method `{}`", method.name),
                        method.name_span.clone(),
                    )
                    .with_note(format!(
                        "`{}` is already declared on `{}` at {}:{}:{}",
                        method.name,
                        owner_name,
                        previous.file,
                        previous.start.line,
                        previous.start.column
                    )),
                );
            }

            let mut bindings: HashMap<String, Span> = HashMap::new();
            for parameter in &method.parameters {
                if let Some(previous) =
                    bindings.insert(parameter.name.clone(), parameter.name_span.clone())
                {
                    self.diagnostics.push(
                        Diagnostic::error(
                            format!("duplicate parameter `{}`", parameter.name),
                            parameter.name_span.clone(),
                        )
                        .with_note(format!(
                            "`{}` was already declared in method `{}` at {}:{}:{}",
                            parameter.name,
                            method.name,
                            previous.file,
                            previous.start.line,
                            previous.start.column
                        )),
                    );
                }

                if let Some(type_reference) = &parameter.type_reference {
                    self.check_type(type_reference);
                }
            }

            for statement in &method.body {
                self.analyze_statement(statement, &mut bindings);
            }
        }
    }

    fn analyze_statement(&mut self, statement: &Statement, bindings: &mut HashMap<String, Span>) {
        match statement {
            Statement::Let(let_statement) => {
                self.analyze_expression(&let_statement.initializer, bindings);

                if let Some(previous) =
                    bindings.insert(let_statement.name.clone(), let_statement.name_span.clone())
                {
                    self.diagnostics.push(
                        Diagnostic::error(
                            format!("duplicate local binding `{}`", let_statement.name),
                            let_statement.name_span.clone(),
                        )
                        .with_note(format!(
                            "`{}` is already bound at {}:{}:{}",
                            let_statement.name,
                            previous.file,
                            previous.start.line,
                            previous.start.column
                        )),
                    );
                }
            }
            Statement::Return(return_statement) => {
                self.analyze_expression(&return_statement.expression, bindings);
            }
        }
    }

    fn analyze_expression(&mut self, expression: &Expression, bindings: &HashMap<String, Span>) {
        match expression {
            Expression::Identifier { name, span } => {
                if !bindings.contains_key(name) && !self.globals.contains_key(name) {
                    self.diagnostics.push(
                        Diagnostic::error(format!("unknown name `{name}`"), span.clone())
                            .with_help(
                                "declare the name with `let`, add it as a parameter, or reference a declared top-level symbol",
                            ),
                    );
                }
            }
            Expression::ArrayLiteral { elements, .. } => {
                for element in elements {
                    self.analyze_expression(element, bindings);
                }
            }
            Expression::ObjectLiteral { entries, .. } => {
                let mut keys: HashMap<&str, &Span> = HashMap::new();
                for entry in entries {
                    if let Some(previous) = keys.insert(&entry.key, &entry.key_span) {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!("duplicate object key `{}`", entry.key),
                                entry.key_span.clone(),
                            )
                            .with_note(format!(
                                "`{}` was already defined at {}:{}:{}",
                                entry.key,
                                previous.file,
                                previous.start.line,
                                previous.start.column
                            )),
                        );
                    }
                    self.analyze_expression(&entry.value, bindings);
                }
            }
            Expression::Grouped { expression, .. }
            | Expression::Unary {
                operand: expression,
                ..
            } => {
                self.analyze_expression(expression, bindings);
            }
            Expression::Binary { left, right, .. } => {
                self.analyze_expression(left, bindings);
                self.analyze_expression(right, bindings);
            }
            Expression::MemberAccess { object, .. } => {
                self.analyze_expression(object, bindings);
            }
            Expression::Call {
                callee, arguments, ..
            } => {
                self.analyze_expression(callee, bindings);
                for argument in arguments {
                    self.analyze_expression(argument, bindings);
                }
            }
            Expression::Index { object, index, .. } => {
                self.analyze_expression(object, bindings);
                self.analyze_expression(index, bindings);
            }
            Expression::StringLiteral { .. }
            | Expression::IntegerLiteral { .. }
            | Expression::BooleanLiteral { .. }
            | Expression::NullLiteral { .. } => {}
        }
    }

    fn check_type(&mut self, type_reference: &TypeReference) {
        if !BUILTIN_TYPES.contains(&type_reference.name.as_str())
            && !self.model_names.contains(&type_reference.name)
            && !self.event_names.contains(&type_reference.name)
        {
            self.diagnostics.push(
                Diagnostic::error(
                    format!("unknown type `{}`", type_reference.name),
                    type_reference.span.clone(),
                )
                .with_help(
                    "use a built-in type (id, int, bool, string), declared model, or declared event type",
                ),
            );
        }
    }
}
