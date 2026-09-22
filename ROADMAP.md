# Densleaf Roadmap

This roadmap is directional. Release numbers describe an intended sequence, not a promise that every item must land under exactly that version.

## 0.1 — Compiler Foundation

- Lexer
- Parser
- AST
- Diagnostics
- Semantic foundation
- Model grammar
- Controller grammar
- Rust code generation
- CLI

## 0.2 — Core Language

Implemented foundation:

- `let` variable declarations
- `null`
- array literals `[]`
- object literals `{}`
- grouped expressions
- local name resolution
- duplicate local/object-key diagnostics
- arithmetic operators and precedence
- equality and comparison operators
- short-circuit logical operators
- unary `!` and `-`
- member access
- function/method call expressions
- index expressions
- forward top-level symbol resolution
- declared model type references
- first-class `middleware` declarations
- first-class `migration` declarations
- first-class `policy ... for Model` declarations
- middleware `handle` contract
- migration `up` contract with optional `down`
- policy-to-model semantic validation
- first-class `event` data declarations
- first-class `listener ... listens Event` declarations
- listener-to-event semantic validation
- first-class `notification` declarations with `channels` and `message` contracts
- first-class `mail` declarations with `subject` and `body` contracts
- event types usable in fields and method parameters

Next:

- static expression type checking
- reassignment and mutability semantics
- `if / else`
- loops
- richer type inference

## 0.3 — Application Model

- richer models
- field attributes
- relationships
- model semantics

## 0.4 — Controllers

- return type semantics
- application responses
- controller semantics

Basic controller parameters already exist from the compiler-foundation stage. This milestone is for mature controller/application semantics rather than introducing parameter syntax for the first time.

## 0.5 — Routing and HTTP

## 0.6 — Database and Queries

## 0.7 — Relationships and Migration Runtime

The `migration` declaration grammar is introduced earlier. This milestone implements schema operations, execution, ordering, rollback behavior, and database integration.

## 0.8 — Validation and Views

## 0.9 — CLI, Testing and Runtime Maturity

## 1.0 — Stable Core Application Framework
