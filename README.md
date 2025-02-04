# Lisp Proc Macros

Implements a simple lisp-like language using rust procedural macros.

## Language Definition
```
<expr> ::= <num>
        |  <id>
        |  <string>
        |  { if <expr> <expr> <expr> }
        |  { declare (<clause>*) in <expr> }
        |  { proc (<id>*) <expr> }
        |  { <expr> <expr>* }
        ;
        
<clause> ::= [ <id> <expr> ]
```

*Note:* 'id' includes the binary operators: +, -, *, /, ==, <= 

## Files

- *macros/src/lib.rs*: Defines macros interp! and print_ast!
- *tests/src/main.rs*: Tests the macros with some simple programs

## Running

Running is as simple as cloning the repository and running cargo build, 
and then cargo run.

The 'main.rs' file can be used as a sandbox to test any programs.
