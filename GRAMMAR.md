# Densleaf Grammar

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
    return_statement ;

return_statement =
    "return" expression ;

expression =
      string_literal
    | integer_literal
    | boolean_literal
    | identifier ;
```

Current built-in type names are:

```text
id
int
bool
string
```

The lexer also recognizes `{ } ( ) [ ] : , . =` so the token layer does not need to be redesigned when nearby grammar work starts. Brackets, dot, and equals are not yet part of a parsed language construct.

Line comments use `//`.

## Potential Future Grammar

Future releases may add variables, assignment, richer expressions, function calls, member access, conditions, loops, arrays, maps, richer models, relationships, routing, requests, responses, validation, database queries, and views.

This section is directional only. None of those constructs should be treated as implemented until they move into **Implemented Grammar**.
