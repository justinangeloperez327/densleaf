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

The parser is a recursive-descent parser that consumes tokens and constructs a Densleaf-specific AST. It understands model declarations, controller declarations, methods, parameters, blocks, `let`, `return`, primitive literals, `null`, arrays, objects, identifiers, and grouped expressions.

The expression parser is intentionally still small. Operator precedence, calls, and member access will be layered onto this foundation rather than implemented as ad-hoc special cases.

### AST

The AST represents Densleaf concepts directly. `ModelDefinition` and `ControllerDefinition` are language nodes, not aliases for Rust syntax. Arrays and objects are Densleaf expression nodes rather than pre-lowered Rust collections.

### Semantic analysis

The semantic pass detects duplicate models, controllers, fields, methods, and parameters; validates current built-in type references; resolves parameters and local `let` bindings; reports unknown local names; and rejects duplicate object keys.

The goal is to catch ordinary application mistakes before Rust code generation.

### Rust code generator

The first backend lowers validated Densleaf AST nodes to deterministic, readable Rust. Built-in Densleaf types are mapped inside this backend. Generated Rust is not part of Densleaf's public grammar.

The generated value representation currently supports strings, integers, booleans, null, arrays, objects, and unit. Backend lowering shields ordinary Densleaf variable reads from Rust move semantics so Rust ownership rules do not become accidental Densleaf language rules.

### CLI

`densleaf check <file>` runs lexing, parsing, semantic analysis, and diagnostic rendering.

`densleaf build <file>` runs the same validation pipeline and writes generated Rust into `.densleaf/generated/`.
