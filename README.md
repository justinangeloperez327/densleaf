# Densleaf

Densleaf is an experimental Rust-based application framework with its own developer-facing grammar. Application authors write Densleaf source (`.dl`), not Rust. The current compiler frontend tokenizes, parses, validates, and lowers that source into generated Rust.

Densleaf is at the **0.1 compiler-foundation stage**. It is not production-ready.

## Current syntax

```densleaf
model User {
    id: id
    name: string
    email: string
}

controller UserController {
    index() {
        return "Users"
    }
}
```

The grammar above is intentionally not Rust. `model` and `controller` are first-class Densleaf concepts represented directly in the Densleaf AST. Rust is currently the first code-generation backend and remains an implementation detail.

## Compiler pipeline

```text
Densleaf source
    ↓
Lexer
    ↓
Tokens
    ↓
Parser
    ↓
Densleaf AST
    ↓
Semantic analysis
    ↓
Rust code generation
```

## Workspace

- `densleaf-token` — tokens and source spans
- `densleaf-diagnostic` — structured source-aware diagnostics
- `densleaf-lexer` — lexical analysis
- `densleaf-ast` — Densleaf-specific AST
- `densleaf-parser` — recursive-descent parser
- `densleaf-semantic` — initial semantic checks
- `densleaf-codegen` — generated Rust backend
- `densleaf-cli` — `densleaf check` and `densleaf build`

## Run the CLI

```bash
cargo run -p densleaf-cli -- check examples/hello/app.dl
cargo run -p densleaf-cli -- build examples/hello/app.dl
```

`check` runs lexing, parsing, and semantic analysis. `build` performs the same checks and then writes generated Rust under `.densleaf/generated/`.

## Intentionally not implemented

The current milestone does **not** include routing, HTTP, ORM/database access, migrations, relationships, views, authentication, authorization, sessions, validation framework features, queues, cache, events, scheduling, mail, notifications, uploads, WebSockets, a template engine, LLVM/Cranelift backends, language-server support, generics, advanced pattern matching, or async language syntax.

See `GRAMMAR.md`, `COMPILER.md`, and `ROADMAP.md` for the implemented boundary and direction.
