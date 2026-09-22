use std::collections::HashMap;

use densleaf_ast::{ControllerDefinition, Declaration, ModelDefinition, Program, TypeReference};
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

            let mut parameters: HashMap<&str, &Span> = HashMap::new();
            for parameter in &method.parameters {
                if let Some(previous) = parameters.insert(&parameter.name, &parameter.name_span) {
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
