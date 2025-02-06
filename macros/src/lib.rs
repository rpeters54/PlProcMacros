
use syn::{Token};
use syn::parse::{Parse, ParseBuffer, ParseStream};

use quote::{quote, quote_spanned, TokenStreamExt, ToTokens};

extern crate proc_macro;
use proc_macro2::TokenStream;
use syn::spanned::Spanned;


// Intermediate Representation
#[derive(Clone)]
enum _Expr {
    _Number(syn::LitInt),
    _Identifier {
        id: syn::Ident,
        type_annotation: Option<syn::Type>,
    },
    _Bool(syn::LitBool),
    _String(syn::LitStr),
    _BinOp(syn::BinOp),
    _If {
        guard: Box<_Expr>,
        then: Box<_Expr>,
        other: Box<_Expr>,
    },
    _Procedure {
        params: Vec<_Expr>, //should be Identifier
        body: Box<_Expr>,
        type_annotation: Option<syn::Type>,
    },
    _Application {
        proc: Box<_Expr>,
        args: Vec<_Expr>,
    },
}

#[derive(Clone)]
struct _Clause {
    name: _Expr, //should be Identifier
    binding: Box<_Expr>,
}

// Converts an expression to a token stream
impl ToTokens for _Expr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let stream = match self {
            _Expr::_Number(n) => quote! { _Expr::_Number(#n) },
            _Expr::_Bool(bl) => quote! { _Expr::_Bool(#bl) },
            _Expr::_Identifier{id, type_annotation} => {
                quote! {
                    _Expr::_Identifier {
                        id: #id,
                        type_annotation: #type_annotation,
                    }
                }
            },
            _Expr::_String(s) => quote! { _Expr::_String(#s) },
            _Expr::_BinOp(b) => quote! { _Expr::_BinOp(#b) },
            _Expr::_If { guard, then, other } => {
                quote! {
                _Expr::_If {
                        guard: #guard,
                        then: #then,
                        other: #other,
                    }
                }
            },
            _Expr::_Procedure { params, body, type_annotation } => {
                quote! {
                _Expr::_Procedure {
                        params: #(#params),*,
                        body: #body,
                        type_annotation: #type_annotation,
                    }
                }
            },
            _Expr::_Application { proc, args } => {
                quote! {
                    _Expr::_Application {
                        proc: #proc,
                        args: #(#args),*,
                    }
                }
            }
        };
        tokens.append_all(stream);
    }
}

// Parses an expression
impl Parse for _Expr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        syn::custom_keyword!(proc);
        syn::custom_keyword!(declare);

        return if input.peek(syn::token::Brace) {
            let content;
            syn::braced!(content in input);

            if content.peek(syn::Token![if]) {
                return parse_if_expr(content);
            }

            // Try to parse procedures
            if content.peek(proc) {
                return parse_procedure(&content);
            }

            // Try to parse declarations
            if content.peek(declare) {
                return parse_declare(&content);
            }

            parse_application(&content)
        } else {
            parse_atoms(input)
        }
    }
}


// Parse a clause
impl Parse for _Clause {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Bracket) {
            let content;
            syn::bracketed!(content in input);

            let name: _Expr = parse_optional_annotation(&content)?;
            let binding: _Expr = content.parse()?;
            Ok(Self {
                name,
                binding: Box::new(binding),
            })
        } else {
            Err(input.error("Missing Clause Parens"))
        }
    }
}

// Parse an if-statement
fn parse_if_expr(input: ParseBuffer) -> syn::Result<_Expr> {
    let _ = input.parse::<Token![if]>()?;
    let guard: _Expr = input.parse()?;
    let then: _Expr = input.parse()?;
    let other: _Expr = input.parse()?;
    Ok(_Expr::_If {
        guard: Box::new(guard),
        then: Box::new(then),
        other: Box::new(other),
    })
}

// Parse the declare syntactic sugar into a procedure and application
fn parse_declare(input: &ParseBuffer) -> syn::Result<_Expr> {
    // get the 'declare' keyword
    syn::custom_keyword!(declare);
    let _ : declare = input.parse()?;

    // parse the clauses
    let content;
    syn::parenthesized!(content in input);
    let clauses: Vec<_Clause> = parse_zero_or_more(&content);

    // parse optional type annotation
    let type_annotation = if input.peek(Token![:]) {
        let _ = input.parse::<Token![:]>()?;
        Some(input.parse::<syn::Type>()?)
    } else {
        None
    };

    // get the 'in' keyword
    let _ = input.parse::<Token![in]>()?;

    // unpack body expression
    let body: _Expr = input.parse()?;

    // split clauses into names and expressions
    let params: Vec<_Expr> = clauses.clone().into_iter()
        .map(|clause| clause.name).collect();
    let args: Vec<_Expr> = clauses.clone().into_iter()
        .map(|clause| *clause.binding).collect();

    // generate the AST node
    Ok(_Expr::_Application {
        proc: Box::new(_Expr::_Procedure {
            body: Box::new(body),
            params,
            type_annotation,
        }),
        args,
    })
}

// parse a procedure
fn parse_procedure(input: &ParseBuffer) -> syn::Result<_Expr> {
    syn::custom_keyword!(proc);
    let _ : proc = input.parse()?;

    // parse the parameters
    let param_content;
    syn::parenthesized!(param_content in input);
    let params: Vec<_Expr> = parse_zero_or_more_compound(&param_content, parse_optional_annotation);

    // parse optional type annotation
    let type_annotation = if input.peek(Token![:]) {
        let _ = input.parse::<Token![:]>()?;
        Some(input.parse::<syn::Type>()?)
    } else {
        None
    };

    // parse the body
    let body: _Expr = input.parse()?;

    Ok(_Expr::_Procedure{
        params,
        body: Box::new(body),
        type_annotation,
    })
}

fn parse_optional_annotation(input: &ParseBuffer) -> syn::Result<_Expr> {
    if input.peek(syn::token::Bracket) {
        let content;
        syn::bracketed!(content in input);
        let id = content.parse::<syn::Ident>()?;
        let _ = content.parse::<Token![:]>()?;
        let type_annotation = Some(content.parse::<syn::Type>()?);
        Ok(_Expr::_Identifier {
            id,
            type_annotation,
        })
    } else {
        let id = input.parse::<syn::Ident>()?;
        Ok(_Expr::_Identifier {
            id,
            type_annotation: None,
        })
    }
}

// parse an application
fn parse_application(input: &ParseBuffer) -> syn::Result<_Expr> {
    let procedure: _Expr = input.parse()?;
    let args: Vec<_Expr> = parse_zero_or_more(&input);

    Ok(_Expr::_Application{
        proc: Box::new(procedure),
        args,
    })
}

// parse the base expressions
fn parse_atoms(input: ParseStream) -> syn::Result<_Expr> {
    if input.peek(syn::LitStr) {
        Ok(_Expr::_String(input.parse()?))
    } else if input.peek(syn::LitInt) {
        Ok(_Expr::_Number(input.parse()?))
    } else if input.peek(syn::LitBool) {
        Ok(_Expr::_Bool(input.parse()?))
    } else if input.peek(syn::Ident) {
        Ok(_Expr::_Identifier{
            id: input.parse()?,
            type_annotation: None,
        })
    } else if peek_op(&input) {
        Ok(_Expr::_BinOp(input.parse()?))
    } else {
        Err(input.error("Failed to Parse Atom"))
    }

}

// helper function for parsing zero-or-more tokens
fn parse_zero_or_more<T: Parse>(input: &ParseBuffer) -> Vec<T> {
    let mut result: Vec<T> = Vec::new();
    while let Ok(item) = input.parse() {
        result.push(item);
    }
    result
}

fn parse_zero_or_more_compound<T: Parse>(input: &ParseBuffer, getter: fn(&ParseBuffer) -> syn::Result<T>) -> Vec<T> {
    let mut result: Vec<T> = Vec::new();
    while let Ok(item) = getter(input) {
        result.push(item);
    }
    result
}

// helper function to check if the next token is one of the
// accepted binary operators
fn peek_op(input: &ParseStream) -> bool {
    // Define a list of concrete token types
    let prim_ops: [&dyn Fn(&ParseStream) -> bool; 18] = [
        &|input| input.peek(Token![+]),
        &|input| input.peek(Token![-]),
        &|input| input.peek(Token![*]),
        &|input| input.peek(Token![/]),
        &|input| input.peek(Token![%]),
        &|input| input.peek(Token![&]),
        &|input| input.peek(Token![|]),
        &|input| input.peek(Token![^]),
        &|input| input.peek(Token![<<]),
        &|input| input.peek(Token![>>]),
        &|input| input.peek(Token![==]),
        &|input| input.peek(Token![<=]),
        &|input| input.peek(Token![>=]),
        &|input| input.peek(Token![<]),
        &|input| input.peek(Token![>]),
        &|input| input.peek(Token![!=]),
        &|input| input.peek(Token![&&]),
        &|input| input.peek(Token![||]),
    ];

    // Check if any of the tokens are present in the input
    prim_ops.iter().any(|op| op(input))
}



// Procedural macro to print _Expr details
#[proc_macro]
pub fn print_ast(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input as _Expr
    let expr = syn::parse_macro_input!(input as _Expr);

    // Generate a function that will print the expression details
    let print_fn = generate_print_expr(&expr, 0);

    proc_macro::TokenStream::from(print_fn)
}

// Recursive function to generate printing logic
fn generate_print_expr(expr: &_Expr, indent: usize) -> TokenStream {
    let indent_str = " ".repeat(indent * 2);

    match expr {
        _Expr::_Number(n) => {
            quote! {
                println!("{}Number: {}", #indent_str, #n);
            }
        },
        _Expr::_Bool(b) => {
            quote! {
                println!("{}Bool: {}", #indent_str, stringify!(#b));
            }
        },
        _Expr::_Identifier{id, type_annotation} => {
            quote! {
                println!("{}Identifier: {} : {}", #indent_str,
                    stringify!(#id), stringify!(#type_annotation));
            }
        },
        _Expr::_String(s) => {
            quote! {
                println!("{}String: {}", #indent_str, #s);
            }
        },
        _Expr::_BinOp(b) => {
            quote! {
                println!("{}BinOp: {}", #indent_str, stringify!(#b));
            }
        },
        _Expr::_If { guard, then, other } => {
            let guard_print = generate_print_expr(guard, indent + 1);
            let then_print = generate_print_expr(then, indent + 1);
            let other_print = generate_print_expr(other, indent + 1);

            quote! {
                println!("{}If Expression:", #indent_str);
                println!("{}Guard:", #indent_str);
                #guard_print
                println!("{}Then:", #indent_str);
                #then_print
                println!("{}Other:", #indent_str);
                #other_print
            }
        },
        _Expr::_Procedure { params, body, type_annotation } => {
            let params_print = params.iter().enumerate().map(|(i, param)| {
                let param_print = generate_print_expr(param, indent + 1);
                quote! {
                    println!("{}Param {}:", #indent_str, #i);
                    #param_print
                }
            });

            let body_print = generate_print_expr(body, indent + 1);

            quote! {
                println!("{}Procedure:", #indent_str);
                println!("{}Parameters:", #indent_str);
                #(#params_print)*
                println!("{}Return Type Annotation: {}", #indent_str, stringify!(#type_annotation));
                println!("{}Body:", #indent_str);
                #body_print
            }
        },
        _Expr::_Application { proc, args } => {
            let proc_print = generate_print_expr(proc, indent + 1);

            let args_print = args.iter().enumerate().map(|(i, arg)| {
                let arg_print = generate_print_expr(arg, indent + 1);
                quote! {
                    println!("{}Arg {}:", #indent_str, #i);
                    #arg_print
                }
            });

            quote! {
                println!("{}Application:", #indent_str);
                println!("{}Procedure:", #indent_str);
                #proc_print
                println!("{}Arguments:", #indent_str);
                #(#args_print)*
            }
        }
    }
}

#[proc_macro]
// expands the program to valid rust syntax, so it can be evaluated
pub fn interp(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input as _Expr
    let expr = syn::parse_macro_input!(input as _Expr);

    let tokens = walk_ast(&expr);

    let wrapper = quote! {
        println!("{}", #tokens);
    };

    proc_macro::TokenStream::from(wrapper)
}

// recursively descends the ast, producing valid rust tokens
fn walk_ast(expr: &_Expr) -> TokenStream {
    match expr {
        _Expr::_Number(n) => { quote! { #n } },
        _Expr::_Bool(bl) => quote! { #bl },
        _Expr::_Identifier { id, type_annotation } => {
            if let Some(ty) = type_annotation {
                quote! { #id: #ty }
            } else {
                quote! { #id }
            }
        },
        _Expr::_String(s) => { quote! { #s } },
        _Expr::_BinOp(b) => { quote! { #b } },
        _Expr::_If { guard, then, other } => {
            let guard_exp = walk_ast(guard);
            let then_exp = walk_ast(then);
            let other_exp = walk_ast(other);

            quote! {
                if #guard_exp {
                    #then_exp
                } else {
                    #other_exp
                }
            }
        },
        _Expr::_Procedure { params, body, type_annotation } => {
            let param_tokens: Vec<TokenStream> = params.iter().map(|param| {
                walk_ast(param)
            }).collect();
            let body_exp: TokenStream = walk_ast(body);

            match type_annotation {
                Some(type_annotation) => {
                    quote! {
                       ( | #(#param_tokens),* | -> #type_annotation { #body_exp })
                    }
                },
                None => {
                    quote! {
                       ( | #(#param_tokens),* | #body_exp )
                    }
                },
            }
        },
        _Expr::_Application { proc, args } => {
            let arg_tokens: Vec<TokenStream> = args.iter().map(|arg| {
                walk_ast(arg)
            }).collect();

            match *proc.clone() {
                _Expr::_BinOp(_) => {
                    handle_prim(walk_ast(proc), arg_tokens)
                }
                _ => {
                    handle_proc(walk_ast(proc), arg_tokens)
                }
            }
        }
    }
}

// helper case for handling infix binary operators
fn handle_prim(op: TokenStream, args: Vec<TokenStream>) -> TokenStream {
    if args.len() != 2 {
        let span = op.span();
        quote_spanned! {
            span => compile_error!("Arity Error");
        }
    } else {
        let arg0 = &args[0];
        let arg1 = &args[1];
        quote! {
            (#arg0 #op #arg1)
        }
    }
}

// helper case for normal procedures
fn handle_proc(proc: TokenStream, args: Vec<TokenStream>) -> TokenStream {
    quote! {
        #proc(#(#args),*)
    }
}

