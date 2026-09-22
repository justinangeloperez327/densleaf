# Densleaf Language Design

Densleaf begins as an application framework with its own language frontend. It may evolve further as the grammar becomes more general, but the present goal is application-oriented language design rather than a general-purpose language.

## Principles

1. Application intent over implementation mechanics.
2. Densleaf controls its own grammar.
3. Rust is an implementation/backend detail.
4. Strong static semantics should coexist with simple syntax.
5. Optimize for low mental overhead, not minimum line count.
6. A keyword must represent meaningful semantics, not merely rename a pattern.
7. Compiler errors must be understandable to application developers.
8. Avoid unnecessary ceremony without hiding important concepts.
9. Hidden implementation complexity must still have deterministic language semantics.
10. Prefer compile-time validation where practical.
11. Syntax must not be dictated by generated Rust.
12. Prefer one obvious way to express a common operation.

## Readability over compression

Densleaf intentionally prefers:

```densleaf
controller UserController {
    show(id: id) {
        let user = null

        return user
    }
}
```

over compressed forms whose behavior is less obvious.

Shorter code is useful only when it also improves comprehension. The language should remain understandable when an application becomes large.

## First-class concepts

`model` and `controller` remain first-class because they carry framework semantics that differ from ordinary values.

Densleaf should be conservative with additional framework keywords. `middleware`, `migration`, `policy`, `event`, `listener`, `notification`, and `mail` qualify because they carry pipeline, lifecycle, binding, dispatch, or delivery semantics that the compiler and runtime must understand. Concepts such as services, requests, and jobs should not automatically become grammar. They can begin as ordinary types or library abstractions and become first-class only if distinct compiler/framework semantics justify it.

## Values

The initial core value syntax is intentionally familiar:

```densleaf
let items = []
let data = {}
let missing = null
```

Arrays use `[]`. Objects use `{}`.

A collection is expected to be a richer runtime/library abstraction rather than another literal syntax. This keeps the grammar small while leaving room for operations such as mapping, filtering, grouping, and pagination later.

## Backend independence

Densleaf's AST models Densleaf concepts directly. A model is not stored as a Rust-struct AST and a controller is not stored as a Rust-`impl` AST. The Rust backend decides how those concepts are lowered.

Ordinary Densleaf variable reads also should not expose Rust ownership mechanics. The backend may clone or otherwise adapt values as needed so Densleaf semantics stay predictable.
