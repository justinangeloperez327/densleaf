# Densleaf Language Design

Densleaf begins as an application framework with its own DSL/language frontend. It may evolve further as the grammar becomes more general, but the present goal is application-oriented language design rather than a general-purpose language.

## Principles

1. Application intent over implementation mechanics.
2. Densleaf controls its own grammar.
3. Rust is an implementation/backend detail.
4. Strong static semantics should coexist with simple syntax.
5. Framework concepts may become first-class grammar.
6. Compiler errors must be understandable to application developers.
7. Avoid unnecessary ceremony.
8. Hidden complexity must still have deterministic semantics.
9. Prefer compile-time validation where practical.
10. Syntax must not be dictated by generated Rust.

## Current scope

The 0.1 language surface contains only the compiler foundation needed for `model`, `controller`, controller methods, parameters, basic `return` statements, literals, identifiers, and initial built-in type references.

Densleaf's AST models Densleaf concepts directly. A model is not stored as a Rust-struct AST and a controller is not stored as a Rust-`impl` AST. The Rust backend decides how those concepts are lowered.
