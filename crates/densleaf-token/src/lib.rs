#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub const fn new(offset: usize, line: usize, column: usize) -> Self {
        Self {
            offset,
            line,
            column,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub file: String,
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(file: impl Into<String>, start: Position, end: Position) -> Self {
        Self {
            file: file.into(),
            start,
            end,
        }
    }

    pub fn cover(&self, other: &Span) -> Self {
        debug_assert_eq!(self.file, other.file);
        Self {
            file: self.file.clone(),
            start: self.start,
            end: other.end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Model,
    Controller,
    Return,
    True,
    False,
    Identifier(String),
    StringLiteral(String),
    IntegerLiteral(i64),
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    Dot,
    Equal,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
