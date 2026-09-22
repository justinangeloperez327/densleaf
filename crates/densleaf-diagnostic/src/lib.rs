use densleaf_token::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

impl Severity {
    fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Note => "note",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub help: Option<String>,
    pub note: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            span,
            help: None,
            note: None,
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn render(&self, source: &str) -> String {
        let line_number = self.span.start.line.max(1);
        let column = self.span.start.column.max(1);
        let source_line = source.lines().nth(line_number - 1).unwrap_or("");
        let width = self
            .span
            .end
            .offset
            .saturating_sub(self.span.start.offset)
            .max(1);
        let caret_width = width.min(source_line.len().saturating_sub(column - 1).max(1));
        let gutter_width = line_number.to_string().len();
        let mut output = format!(
            "{}: {}\n\n --> {}:{}:{}\n{:>gutter_width$} |\n{:>gutter_width$} | {}\n{:>gutter_width$} | {}{} {}",
            self.severity.as_str(),
            self.message,
            self.span.file,
            line_number,
            column,
            "",
            line_number,
            source_line,
            "",
            " ".repeat(column.saturating_sub(1)),
            "^".repeat(caret_width),
            self.message,
        );

        if let Some(help) = &self.help {
            output.push_str(&format!("\n\nhelp: {help}"));
        }
        if let Some(note) = &self.note {
            output.push_str(&format!("\n\nnote: {note}"));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use densleaf_token::{Position, Span};

    use super::*;

    #[test]
    fn renders_source_aware_diagnostics() {
        let span = Span::new(
            "app.dl",
            Position::new(13, 2, 5),
            Position::new(17, 2, 9),
        );
        let diagnostic = Diagnostic::error("duplicate field `name`", span)
            .with_help("rename or remove the duplicate field");
        let rendered = diagnostic.render("model User {\n    name: string\n}");
        assert!(rendered.contains("app.dl:2:5"));
        assert!(rendered.contains("^^^^"));
        assert!(rendered.contains("help: rename or remove the duplicate field"));
    }
}
