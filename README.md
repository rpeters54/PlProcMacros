# Lisp Proc Macros

Implements a simple lisp-like language using rust procedural macros.

Supports optional type annotations for procedures and declare expressions.

## Language Definition
```
<expr> ::= <num>
        |  <id> 
        |  <string>
        |  { if <expr> <expr> <expr> }
        |  { declare (<clause>*) <rettype>? in <expr> }
        |  { proc (<annot>*) <rettype>? <expr> }
        |  { <expr> <expr>* }
        ;
        
<annot> ::= <id> | [<id> : <type>]
<rettype> ::= : <type>         
<clause> ::= [ <annot> <expr> ]
```

*Note:* 'id' includes the binary operators: +, -, *, /, ==, <= 

## Files

- *macros/src/lib.rs*: Defines macros interp! and print_ast!
- *tests/src/main.rs*: Tests the macros with some simple programs

## Running

Running is as simple as cloning the repository and running cargo build, 
and then cargo run.

The 'main.rs' file can be used as a sandbox to test any programs.
