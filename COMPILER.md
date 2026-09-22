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

The lexer converts characters into Densleaf tokens. It recognizes keywords, identifiers, literals, punctuation, comments, arithmetic operators, comparison operators, equality operators, and logical operators. Invalid characters, incomplete logical operators, and unterminated strings produce structured diagnostics rather than panics.

### Tokens

Tokens are syntax-level units only. They do not decide application semantics or generated Rust behavior.

### Parser

The parser is a recursive-descent parser with explicit precedence levels. It understands models, controllers, methods, parameters, blocks, `let`, `return`, primitive literals, `null`, arrays, objects, grouped expressions, unary and binary operators, member access, calls, and indexing.

Postfix expressions are composable, so constructs such as `User.find(id).items[0].name` form one expression tree rather than framework-specific parser cases.

### AST

The AST represents Densleaf concepts directly. Operators, calls, member access, and indexing are Densleaf expression nodes rather than pre-lowered Rust syntax.

### Semantic analysis

Semantic analysis now uses two phases:

1. collect top-level models and controllers;
2. analyze models and controller bodies against the collected symbols.

This allows forward references while still reporting duplicate top-level declarations. It also validates model names as user-defined type references, resolves parameters and local `let` bindings, reports unknown names, and rejects duplicate object keys.

Full static operator type inference is intentionally not complete yet. The backend currently preserves deterministic runtime checks for operator misuse until the type system can reject those cases at compile time.

### Rust code generator

The Rust backend lowers expressions to a small generated Densleaf value runtime. Arithmetic uses checked integer operations, logical `&&` and `||` preserve short-circuit behavior, object member access and indexing have explicit runtime behavior, and top-level symbols are represented distinctly from local variables.

Call syntax is supported by the compiler, but application/framework call targets such as future ORM methods are still separate runtime features. A syntactically valid `User.find(id)` therefore does not imply that database lookup semantics already exist.

Generated Rust remains an implementation detail and is compiled in CI to verify backend validity.

### CLI

`densleaf check <file>` runs lexing, parsing, semantic analysis, and diagnostic rendering.

`densleaf build <file>` runs the same validation pipeline and writes generated Rust into `.densleaf/generated/`.
