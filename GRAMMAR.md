# Densleaf Grammar

## Design rule

Densleaf grammar optimizes for clarity and predictable semantics, not the fewest characters.

A construct should become syntax only when it represents a real language or framework concept. Ordinary application patterns should use ordinary language constructs instead of accumulating special keywords.

## Implemented Grammar

```ebnf
program =
    declaration* ;

declaration =
      model_declaration
    | controller_declaration
    | middleware_declaration
    | migration_declaration
    | policy_declaration
    | event_declaration
    | listener_declaration
    | notification_declaration
    | mail_declaration ;

model_declaration =
    "model" identifier "{"
        model_field*
    "}" ;

model_field =
    identifier ":" type_reference ;

controller_declaration =
    "controller" identifier "{"
        controller_method*
    "}" ;

controller_method =
    identifier "(" parameter_list? ")" block ;

middleware_declaration =
    "middleware" identifier "{"
        method_declaration*
    "}" ;

migration_declaration =
    "migration" identifier "{"
        method_declaration*
    "}" ;

policy_declaration =
    "policy" identifier "for" identifier "{"
        method_declaration*
    "}" ;

event_declaration =
    "event" identifier "{"
        field_declaration*
    "}" ;

listener_declaration =
    "listener" identifier "listens" identifier "{"
        method_declaration*
    "}" ;

notification_declaration =
    "notification" identifier "{"
        method_declaration*
    "}" ;

mail_declaration =
    "mail" identifier "{"
        method_declaration*
    "}" ;

method_declaration =
    identifier "(" parameter_list? ")" block ;

field_declaration =
    identifier ":" type_reference ;

parameter_list =
    parameter ("," parameter)* ;

parameter =
    identifier (":" type_reference)? ;

block =
    "{" statement* "}" ;

statement =
      let_statement
    | return_statement ;

let_statement =
    "let" identifier "=" expression ;

return_statement =
    "return" expression ;

expression =
    logical_or ;

logical_or =
    logical_and ("||" logical_and)* ;

logical_and =
    equality ("&&" equality)* ;

equality =
    comparison (("==" | "!=") comparison)* ;

comparison =
    additive (("<" | "<=" | ">" | ">=") additive)* ;

additive =
    multiplicative (("+" | "-") multiplicative)* ;

multiplicative =
    unary (("*" | "/" | "%") unary)* ;

unary =
      ("!" | "-") unary
    | postfix ;

postfix =
    primary postfix_part* ;

postfix_part =
      "." identifier
    | "(" argument_list? ")"
    | "[" expression "]" ;

argument_list =
    expression ("," expression)* ","? ;

primary =
      string_literal
    | integer_literal
    | boolean_literal
    | null_literal
    | identifier
    | grouped_expression
    | array_literal
    | object_literal ;

grouped_expression =
    "(" expression ")" ;

array_literal =
    "[" (expression ("," expression)* ","?)? "]" ;

object_literal =
    "{" (object_entry ("," object_entry)* ","?)? "}" ;

object_entry =
    identifier ":" expression ;
```

Current built-in type names are:

```text
id
int
bool
string
```

Declared model and event names are also valid type references.

Line comments use `//`.

Declaration words such as `model`, `event`, `listener`, `notification`, and `mail` are contextual keywords. They introduce declarations at the top level, but they remain valid names where the grammar expects an identifier. This keeps natural code such as `handle(event: UserCreated)` valid instead of forcing artificial variable names.

## Operator precedence

From lowest to highest:

```text
||
&&
== !=
< <= > >=
+ -
* / %
! -
member access / calls / indexing
primary values
```

Postfix expressions chain naturally:

```densleaf
User.find(id).items[0].name
```

This syntax is part of the language. Individual framework methods such as `User.find` are separate runtime/framework features and are not implied to be implemented merely because the call grammar exists.

Logical `&&` and `||` have short-circuit semantics.

## Literal conventions

Densleaf uses familiar literal syntax:

```densleaf
let numbers = [1, 2, 3]

let user = {
    name: "Justin",
    active: true
}

let nothing = null
```

- `[]` is an array literal.
- `{}` is an object literal.
- `null` represents the absence of a value.
- trailing commas are accepted in arrays, objects, and call arguments.
- object keys are identifiers in the current grammar.

A collection is **not** a separate literal. A future `Collection<T>` abstraction may provide richer sequence behavior, but it should build on normal language values rather than introduce another punctuation form.

## Current statement semantics

Variables are introduced with `let`:

```densleaf
let score = (10 + 5) * 2
let allowed = score >= 20 && true
let data = { allowed: allowed }

return data
```

A local name must already exist before it can be read. Parameters are visible inside their method body. All top-level declarations are collected before body analysis, so forward references are recognized. Re-declaring a parameter or local name in the same method is rejected.

## First-class application declarations

Densleaf treats framework concepts as grammar only when each carries compiler-visible semantics:

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

event UserCreated {
    user: User
}

listener SendWelcomeMail listens UserCreated {
    handle(event: UserCreated) {
        return event
    }
}

notification WelcomeNotification {
    channels() {
        return ["mail"]
    }

    message(user: User) {
        return "Welcome"
    }
}

mail WelcomeMail {
    subject() {
        return "Welcome"
    }

    body(user: User) {
        return "Hello"
    }
}
```

- A `middleware` declaration must expose `handle` as its pipeline entry point.
- A `migration` declaration must expose `up`; `down` is optional because not every migration is safely reversible.
- A `policy` explicitly targets a declared model with `for`. Policy method names represent authorization abilities and are intentionally not hard-coded to CRUD names.
- An `event` is typed application data intended for dispatch.
- A `listener` binds to one declared event with `listens`, must define `handle`, and receives that event as its first handler argument.
- A `notification` must define `channels` and `message`.
- A `mail` declaration must define `subject` and `body`.
- These declarations have their own AST nodes and compiler semantics. They are not aliases for generic types plus marker interfaces.
- Actual event dispatching, listener execution, notification delivery, mail transport, HTTP middleware execution, schema operations, and authorization enforcement remain later runtime/framework work.

## Deliberately deferred

The following remain outside the implemented grammar until their semantics are designed:

- reassignment and mutability
- `if / else`
- loops
- standalone functions
- user-defined general-purpose types
- enums
- optional types
- complete static expression type checking
- framework routing, HTTP execution, database query/runtime behavior, schema operations inside migrations, authorization enforcement, validation, views, jobs, event dispatch runtime, notification transport, mail transport, and similar application features

In particular, Densleaf does not add keywords such as `service`, `request`, or `job` merely to reduce boilerplate. A new keyword must provide distinct semantics that justify becoming part of the language.
