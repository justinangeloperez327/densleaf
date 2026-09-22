use std::collections::HashMap;

use densleaf_ast::{
    ControllerDefinition, Declaration, Expression, ModelDefinition, Program, Statement,
    TypeReference,
};
use densleaf_diagnostic::Diagnostic;
use densleaf_token::Span;

const BUILTIN_TYPES: &[&str] = &["id", "int", "bool", "string"];

pub fn analyze(program: &Program) -> Vec<Diagnostic> {
    let mut analyzer = Analyzer::default();
    analyzer.analyze_program(program);
    analyzer.diagnostics
}

#[derive(Default)]
struct Analyzer {
    diagnostics: Vec<Diagnostic>,
}

impl Analyzer {
    fn analyze_program(&mut self, program: &Program) {
        let mut models: HashMap<&str, &Span> = HashMap::new();
        let mut controllers: HashMap<&str, &Span> = HashMap::new();

        for declaration in &program.declarations {
            match declaration {
                Declaration::Model(model) => {
                    if let Some(previous) = models.insert(&model.name, &model.name_span) {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!("duplicate model `{}`", model.name),
                                model.name_span.clone(),
                            )
                            .with_note(format!(
                                "`{}` was already declared at {}:{}:{}",
                                model.name,
                                previous.file,
                                previous.start.line,
                                previous.start.column
                            )),
                        );
                    }
                    self.analyze_model(model);
                }
                Declaration::Controller(controller) => {
                    if let Some(previous) =
                        controllers.insert(&controller.name, &controller.name_span)
                    {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!("duplicate controller `{}`", controller.name),
                                controller.name_span.clone(),
                            )
                            .with_note(format!(
                                "`{}` was already declared at {}:{}:{}",
                                controller.name,
                                previous.file,
                                previous.start.line,
                                previous.start.column
                            )),
                        );
                    }
                    self.analyze_controller(controller);
                }
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
                if !bindings.contains_key(name) {
                    self.diagnostics.push(
                        Diagnostic::error(format!("unknown name `{name}`"), span.clone())
                            .with_help("declare the name with `let` or add it as a parameter"),
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
            Expression::Grouped { expression, .. } => {
                self.analyze_expression(expression, bindings);
            }
            Expression::StringLiteral { .. }
            | Expression::IntegerLiteral { .. }
            | Expression::BooleanLiteral { .. }
            | Expression::NullLiteral { .. } => {}
        }
    }

    fn check_type(&mut self, type_reference: &TypeReference) {
        if !BUILTIN_TYPES.contains(&type_reference.name.as_str()) {
            self.diagnostics.push(
                Diagnostic::error(
                    format!("unknown type `{}`", type_reference.name),
                    type_reference.span.clone(),
                )
                .with_help("use one of the currently supported types: id, int, bool, string"),
            );
        }
    }
}
