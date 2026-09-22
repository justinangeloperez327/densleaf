use std::collections::{HashMap, HashSet};

use densleaf_ast::{
    ControllerDefinition, Declaration, Expression, ModelDefinition, Program, Statement,
    TypeReference,
};
use densleaf_diagnostic::Diagnostic;
use densleaf_token::Span;

const BUILTIN_TYPES: &[&str] = &["id", "int", "bool", "string"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GlobalKind {
    Model,
    Controller,
}

impl GlobalKind {
    fn label(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Controller => "controller",
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
}

impl Analyzer {
    fn analyze_program(&mut self, program: &Program) {
        self.collect_globals(program);

        for declaration in &program.declarations {
            match declaration {
                Declaration::Model(model) => self.analyze_model(model),
                Declaration::Controller(controller) => self.analyze_controller(controller),
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
        let mut fields: HashMap<&str, &Span> = HashMap::new();
        for field in &model.fields {
            if let Some(previous) = fields.insert(&field.name, &field.name_span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        format!("duplicate field `{}`", field.name),
                        field.name_span.clone(),
                    )
                    .with_note(format!(
                        "`{}` is already declared on `{}` at {}:{}:{}",
                        field.name,
                        model.name,
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
        let mut methods: HashMap<&str, &Span> = HashMap::new();

        for method in &controller.methods {
            if let Some(previous) = methods.insert(&method.name, &method.name_span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        format!("duplicate controller method `{}`", method.name),
                        method.name_span.clone(),
                    )
                    .with_note(format!(
                        "`{}` is already declared on `{}` at {}:{}:{}",
                        method.name,
                        controller.name,
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
        {
            self.diagnostics.push(
                Diagnostic::error(
                    format!("unknown type `{}`", type_reference.name),
                    type_reference.span.clone(),
                )
                .with_help("use a built-in type (id, int, bool, string) or a declared model type"),
            );
        }
    }
}
