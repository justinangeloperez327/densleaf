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

Declared model names are also valid type references.

Line comments use `//`.

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

A local name must already exist before it can be read. Parameters are visible inside their method body. Top-level models and controllers are collected before body analysis, so forward references are recognized. Re-declaring a parameter or local name in the same method is rejected.

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
- framework routing, HTTP, database queries, validation, views, jobs, events, and similar application features

In particular, Densleaf does not add keywords such as `service`, `request`, `job`, or `event` merely to reduce boilerplate. A new keyword must provide distinct semantics that justify becoming part of the language.
