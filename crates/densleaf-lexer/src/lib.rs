use densleaf_diagnostic::Diagnostic;
use densleaf_token::{Position, Span, Token, TokenKind};

#[derive(Debug, Clone)]
pub struct LexOutput {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn lex(source: &str, file: impl Into<String>) -> LexOutput {
    Lexer::new(source, file.into()).tokenize()
}

struct Lexer<'a> {
    source: &'a str,
    file: String,
    offset: usize,
    line: usize,
    column: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str, file: String) -> Self {
        Self {
            source,
            file,
            offset: 0,
            line: 1,
            column: 1,
            tokens: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn tokenize(mut self) -> LexOutput {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.skip_whitespace();
                continue;
            }

            if ch == '/' && self.peek_next() == Some('/') {
                self.skip_comment();
                continue;
            }

            let start = self.position();
            match ch {
                '{' => self.single(TokenKind::LeftBrace, start),
                '}' => self.single(TokenKind::RightBrace, start),
                '(' => self.single(TokenKind::LeftParen, start),
                ')' => self.single(TokenKind::RightParen, start),
                '[' => self.single(TokenKind::LeftBracket, start),
                ']' => self.single(TokenKind::RightBracket, start),
                ':' => self.single(TokenKind::Colon, start),
                ',' => self.single(TokenKind::Comma, start),
                '.' => self.single(TokenKind::Dot, start),
                '+' => self.single(TokenKind::Plus, start),
                '-' => self.single(TokenKind::Minus, start),
                '*' => self.single(TokenKind::Star, start),
                '/' => self.single(TokenKind::Slash, start),
                '%' => self.single(TokenKind::Percent, start),
                '=' => self.one_or_two(TokenKind::Equal, TokenKind::EqualEqual, '=', start),
                '!' => self.one_or_two(TokenKind::Bang, TokenKind::BangEqual, '=', start),
                '<' => self.one_or_two(TokenKind::Less, TokenKind::LessEqual, '=', start),
                '>' => self.one_or_two(TokenKind::Greater, TokenKind::GreaterEqual, '=', start),
                '&' => self.paired_operator('&', TokenKind::AndAnd, start),
                '|' => self.paired_operator('|', TokenKind::OrOr, start),
                '"' => self.lex_string(start),
                c if c.is_ascii_digit() => self.lex_integer(start),
                c if is_identifier_start(c) => self.lex_identifier(start),
                _ => self.invalid_character(ch, start),
            }
        }

        let position = self.position();
        self.tokens.push(Token::new(
            TokenKind::Eof,
            Span::new(self.file.clone(), position, position),
        ));

        LexOutput {
            tokens: self.tokens,
            diagnostics: self.diagnostics,
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.offset..)?.chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        let mut chars = self.source.get(self.offset..)?.chars();
        chars.next()?;
        chars.next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.offset += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn position(&self) -> Position {
        Position::new(self.offset, self.line, self.column)
    }

    fn span_from(&self, start: Position) -> Span {
        Span::new(self.file.clone(), start, self.position())
    }

    fn single(&mut self, kind: TokenKind, start: Position) {
        self.advance();
        self.tokens.push(Token::new(kind, self.span_from(start)));
    }

    fn one_or_two(&mut self, single: TokenKind, double: TokenKind, second: char, start: Position) {
        self.advance();
        let kind = if self.peek() == Some(second) {
            self.advance();
            double
        } else {
            single
        };
        self.tokens.push(Token::new(kind, self.span_from(start)));
    }

    fn paired_operator(&mut self, expected: char, kind: TokenKind, start: Position) {
        self.advance();
        if self.peek() == Some(expected) {
            self.advance();
            self.tokens.push(Token::new(kind, self.span_from(start)));
        } else {
            self.diagnostics.push(
                Diagnostic::error(
                    format!("expected a second `{expected}`"),
                    self.span_from(start),
                )
                .with_help(format!(
                    "Densleaf uses `{expected}{expected}` for this logical operator"
                )),
            );
        }
    }

    fn invalid_character(&mut self, ch: char, start: Position) {
        self.advance();
        let span = self.span_from(start);
        self.diagnostics.push(
            Diagnostic::error(format!("invalid character `{ch}`"), span)
                .with_help("remove the character or replace it with valid Densleaf syntax"),
        );
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.advance();
        }
    }

    fn skip_comment(&mut self) {
        self.advance();
        self.advance();
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn lex_identifier(&mut self, start: Position) {
        let start_offset = self.offset;
        self.advance();
        while self.peek().is_some_and(is_identifier_continue) {
            self.advance();
        }
        let text = &self.source[start_offset..self.offset];
        let kind = match text {
            "model" => TokenKind::Model,
            "controller" => TokenKind::Controller,
            "middleware" => TokenKind::Middleware,
            "migration" => TokenKind::Migration,
            "policy" => TokenKind::Policy,
            "for" => TokenKind::For,
            "let" => TokenKind::Let,
            "return" => TokenKind::Return,
            "null" => TokenKind::Null,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            _ => TokenKind::Identifier(text.to_string()),
        };
        self.tokens.push(Token::new(kind, self.span_from(start)));
    }

    fn lex_integer(&mut self, start: Position) {
        let start_offset = self.offset;
        while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            self.advance();
        }
        let text = &self.source[start_offset..self.offset];
        match text.parse::<i64>() {
            Ok(value) => self.tokens.push(Token::new(
                TokenKind::IntegerLiteral(value),
                self.span_from(start),
            )),
            Err(_) => self.diagnostics.push(
                Diagnostic::error(
                    "integer literal is outside the supported range",
                    self.span_from(start),
                )
                .with_help("use an integer no larger than 9223372036854775807"),
            ),
        }
    }

    fn lex_string(&mut self, start: Position) {
        self.advance();
        let mut value = String::new();
        let mut terminated = false;

        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.advance();
                    terminated = true;
                    break;
                }
                '\n' => break,
                '\\' => {
                    self.advance();
                    match self.peek() {
                        Some('n') => {
                            self.advance();
                            value.push('\n');
                        }
                        Some('r') => {
                            self.advance();
                            value.push('\r');
                        }
                        Some('t') => {
                            self.advance();
                            value.push('\t');
                        }
                        Some('"') => {
                            self.advance();
                            value.push('"');
                        }
                        Some('\\') => {
                            self.advance();
                            value.push('\\');
                        }
                        Some(other) => {
                            self.advance();
                            value.push(other);
                        }
                        None => break,
                    }
                }
                _ => {
                    self.advance();
                    value.push(ch);
                }
            }
        }

        let span = self.span_from(start);
        if terminated {
            self.tokens
                .push(Token::new(TokenKind::StringLiteral(value), span));
        } else {
            self.diagnostics.push(
                Diagnostic::error("unterminated string literal", span)
                    .with_help("close the string with `\"` before the end of the line"),
            );
        }
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}
