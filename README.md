# Densleaf

Densleaf is an experimental Rust-based application framework with its own developer-facing grammar. Application authors write Densleaf source (`.dl`), not Rust. The compiler frontend tokenizes, parses, validates, and lowers that source into generated Rust.

Densleaf has completed its **0.1 compiler foundation** and is building the **0.2 core language**. It is not production-ready.

## Current syntax

```densleaf
model User {
    id: id
    name: string
    active: bool
}

controller UserController {
    index() {
        let score = (10 + 5) * 2
        let allowed = score >= 20 && !false

        let users = [
            { name: "Justin", active: allowed },
        ]

        return users[0].name
    }
}
```

The grammar is intentionally not Rust. `model` and `controller` are first-class Densleaf concepts represented directly in the Densleaf AST. Rust is currently the first code-generation backend and remains an implementation detail.

Arrays use `[]`; objects use `{}`. Densleaf does not introduce a separate collection literal.

## Current expression language

Densleaf supports:

- arithmetic: `+`, `-`, `*`, `/`, `%`
- equality: `==`, `!=`
- comparisons: `<`, `<=`, `>`, `>=`
- logical expressions: `&&`, `||`, `!`
- member access: `user.name`
- calls: `target.method(value)`
- indexing: `users[0]`

The compiler understands call syntax independently from framework APIs. For example, parsing `User.find(id)` does not mean ORM lookup behavior is implemented yet.

## Design direction

Densleaf optimizes for code that is easy to understand rather than code with the fewest possible lines.

New keywords are added only when they provide distinct semantics. `middleware`, `migration`, `policy`, `event`, `listener`, `notification`, and `mail` are first-class because the compiler/framework must understand their pipeline, lifecycle, binding, dispatch, or delivery roles. Framework patterns such as services, requests, and jobs are not automatically promoted into grammar merely to shorten code.

## Application declarations

```densleaf
middleware AuthMiddleware {
    handle(request) {
        return request
    }
}

migration CreateUsers {
    up() {}
    down() {}
}

policy UserPolicy for User {
    view(actor: User, target: User) {
        return true
    }
}
```

These declarations are compiler concepts rather than `implements` conventions.

Messaging declarations are also first-class:

```densleaf
event UserCreated {
    user: User
}

listener SendWelcomeMail listens UserCreated {
    handle(event: UserCreated) {
        return event
    }
}

notification WelcomeNotification {
    channels() { return ["mail"] }
    message(user: User) { return "Welcome" }
}

mail WelcomeMail {
    subject() { return "Welcome" }
    body(user: User) { return "Hello" }
}
```

The grammar exists now; middleware pipelines, database schema execution, authorization enforcement, event dispatch, notification delivery, and mail transport will be implemented in their respective framework/runtime milestones.

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
- `densleaf-semantic` — semantic checks and name resolution
- `densleaf-codegen` — generated Rust backend
- `densleaf-cli` — `densleaf check` and `densleaf build`

## Run the CLI

```bash
cargo run -p densleaf-cli -- check examples/hello/app.dl
cargo run -p densleaf-cli -- build examples/hello/app.dl
```

`check` runs lexing, parsing, and semantic analysis. `build` performs the same checks and then writes generated Rust under `.densleaf/generated/`.

See `GRAMMAR.md`, `LANGUAGE_DESIGN.md`, `COMPILER.md`, and `ROADMAP.md` for the implemented boundary and direction.
