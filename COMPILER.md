# Densleaf Compiler

## Pipeline

```text
Source
  ↓
Lexer
  ↓
Tokens
  ↓
Parser
  ↓
AST
  ↓
Semantic Analysis
  ↓
Rust Code Generator
  ↓
Generated Rust
```

## Stages

### Source and spans

Every token carries a source span with file, byte offsets, line, and column information. Later stages preserve spans on Densleaf AST nodes so errors can point back to application source.

### Lexer

The lexer converts characters into Densleaf tokens. It recognizes keywords, identifiers, literals, punctuation, comments, and end-of-file. Invalid characters and unterminated strings produce structured diagnostics rather than panics.

### Tokens

Tokens are syntax-level units only. They do not decide application semantics or generated Rust behavior.

### Parser

The parser is a recursive-descent parser that consumes tokens and constructs a Densleaf-specific AST. It currently understands model declarations, controller declarations, methods, parameters, blocks, return statements, and the initial expression forms.

### AST

The AST represents Densleaf concepts directly. `ModelDefinition` and `ControllerDefinition` are language nodes, not aliases for Rust syntax.

### Semantic analysis

The current semantic pass detects duplicate models, controllers, fields, methods, parameters, and unknown built-in type references. It returns diagnostics for normal source errors instead of panicking.

### Rust code generator

The first backend lowers validated Densleaf AST nodes to deterministic, readable Rust. Built-in Densleaf types are mapped inside this backend. Generated Rust is not part of Densleaf's public grammar.

The backend currently emits a small private value representation for controller return values so basic Densleaf strings, integers, booleans, and identifier returns can be expressed without forcing Rust syntax into the language.

### CLI

`densleaf check <file>` runs lexing, parsing, semantic analysis, and diagnostic rendering.

`densleaf build <file>` runs the same validation pipeline and writes generated Rust into `.densleaf/generated/`. Rust compilation is intentionally not required for successful Densleaf parsing at this milestone.
