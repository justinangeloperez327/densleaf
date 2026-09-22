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
    | controller_declaration ;

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
    primary_expression ;

primary_expression =
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

Line comments use `//`.

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
- trailing commas are accepted in arrays and objects.
- object keys are identifiers in the current grammar.

A collection is **not** a separate literal. A future `Collection<T>` abstraction may provide richer sequence behavior, but it should build on normal language values rather than introduce another punctuation form.

## Current statement semantics

Variables are introduced with `let`:

```densleaf
let users = []
let data = { users: users }

return data
```

A local name must already exist before it can be read. Parameters are visible inside their method body. Re-declaring a parameter or local name in the same method is rejected.

## Deliberately deferred

The following remain outside the implemented grammar until their semantics are designed:

- assignment after declaration
- operators and precedence
- function calls
- member access
- `if / else`
- loops
- functions
- user-defined general-purpose types
- enums
- optional types
- type inference beyond the current value representation
- framework routing, HTTP, database queries, validation, views, jobs, events, and similar application features

In particular, Densleaf does not add keywords such as `service`, `request`, `job`, or `event` merely to reduce boilerplate. A new keyword must provide distinct semantics that justify becoming part of the language.
