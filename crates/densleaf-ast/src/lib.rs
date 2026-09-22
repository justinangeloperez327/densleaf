use densleaf_token::Span;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declaration {
    Model(ModelDefinition),
    Controller(ControllerDefinition),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelDefinition {
    pub name: String,
    pub name_span: Span,
    pub fields: Vec<ModelField>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelField {
    pub name: String,
    pub name_span: Span,
    pub type_reference: TypeReference,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControllerDefinition {
    pub name: String,
    pub name_span: Span,
    pub methods: Vec<ControllerMethod>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControllerMethod {
    pub name: String,
    pub name_span: Span,
    pub parameters: Vec<Parameter>,
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub name_span: Span,
    pub type_reference: Option<TypeReference>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Let(LetStatement),
    Return(ReturnStatement),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetStatement {
    pub name: String,
    pub name_span: Span,
    pub initializer: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnStatement {
    pub expression: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectEntry {
    pub key: String,
    pub key_span: Span,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    StringLiteral {
        value: String,
        span: Span,
    },
    IntegerLiteral {
        value: i64,
        span: Span,
    },
    BooleanLiteral {
        value: bool,
        span: Span,
    },
    NullLiteral {
        span: Span,
    },
    Identifier {
        name: String,
        span: Span,
    },
    ArrayLiteral {
        elements: Vec<Expression>,
        span: Span,
    },
    ObjectLiteral {
        entries: Vec<ObjectEntry>,
        span: Span,
    },
    Grouped {
        expression: Box<Expression>,
        span: Span,
    },
}

impl Expression {
    pub fn span(&self) -> &Span {
        match self {
            Self::StringLiteral { span, .. }
            | Self::IntegerLiteral { span, .. }
            | Self::BooleanLiteral { span, .. }
            | Self::NullLiteral { span }
            | Self::Identifier { span, .. }
            | Self::ArrayLiteral { span, .. }
            | Self::ObjectLiteral { span, .. }
            | Self::Grouped { span, .. } => span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeReference {
    pub name: String,
    pub span: Span,
}
