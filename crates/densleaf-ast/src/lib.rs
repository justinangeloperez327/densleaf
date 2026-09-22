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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Not,
    Negate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
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
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
        span: Span,
    },
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
        span: Span,
    },
    MemberAccess {
        object: Box<Expression>,
        member: String,
        member_span: Span,
        span: Span,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
        span: Span,
    },
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
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
            | Self::Grouped { span, .. }
            | Self::Unary { span, .. }
            | Self::Binary { span, .. }
            | Self::MemberAccess { span, .. }
            | Self::Call { span, .. }
            | Self::Index { span, .. } => span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeReference {
    pub name: String,
    pub span: Span,
}
