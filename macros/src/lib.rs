
use syn::{Token};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::token::{Plus, Minus, Star, Slash, EqEq, Le};

use quote::{quote, quote_spanned, TokenStreamExt, ToTokens};

extern crate proc_macro;
use proc_macro2::TokenStream;
use syn::spanned::Spanned;


#[derive(Clone)]
enum _Expr {
    _Number(syn::LitInt),
    _Identifier(_IdTypes),
    _String(syn::LitStr),
    _If {
        guard: Box<_Expr>,
        then: Box<_Expr>,
        other: Box<_Expr>,
    },
    _Procedure {
        params: Vec<_Expr>, //should be Identifier
        body: Box<_Expr>,
    },
    _Application {
        proc: Box<_Expr>,
        args: Vec<_Expr>,
    },
}

#[derive(Clone)]
enum _IdTypes {
    _Id(syn::Ident),
    _Op(syn::BinOp),
    _Bool(syn::LitBool)
}

#[derive(Clone)]
struct _Clause {
    name: _Expr, //should be Identifier
    binding: Box<_Expr>,
}

impl ToTokens for _Expr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let stream = match self {
            _Expr::_Number(n) => quote! { _Expr::_Number(#n) },
            _Expr::_Identifier(id) => quote! { _Expr::_Identifier(#id) },
            _Expr::_String(s) => quote! { _Expr::_String(#s) },
            _Expr::_If { guard, then, other } => {
                quote! {
                _Expr::_If {
                        guard: Box::new(#guard),
                        then: Box::new(#then),
                        other: Box::new(#other)
                    }
                }
            },
            _Expr::_Procedure { params, body } => {
                quote! {
                _Expr::_Procedure {
                        params: vec![#(#params),*],
                        body: Box::new(#body)
                    }
                }
            },
            _Expr::_Application { proc, args } => {
                quote! {
                    _Expr::_Application {
                        proc: Box::new(#proc),
                        args: vec![#(#args),*]
                    }
                }
            }
        };
        tokens.append_all(stream);
    }
}

impl ToTokens for _IdTypes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let stream = match self {
            _IdTypes::_Op(op) => quote! { _IdTypes::_Op(#op) },
            _IdTypes::_Id(id) => quote! { _IdTypes::_Id(#id) },
            _IdTypes::_Bool(bl) => quote! { _IdTypes::_Id(#bl) },
        };
        tokens.append_all(stream);
    }
}

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
                return parse_procedure(content);
            }

            // Try to parse declarations
            if content.peek(declare) {
                return parse_declare(content);
            }

            parse_application(content)
        } else {
            parse_atoms(input)
        }
    }
}

impl Parse for _IdTypes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if peek_op(&input) {
            let op: syn::BinOp = input.parse()?;
            return Ok(_IdTypes::_Op(op));
        }

        if input.peek(syn::Ident) {
            let id: syn::Ident = input.parse()?;
            return Ok(_IdTypes::_Id(id));
        }

        if input.peek(syn::LitBool) {
            let bl: syn::LitBool = input.parse()?;
            return Ok(_IdTypes::_Bool(bl));
        }

        return Err(input.error("Invalid Id Token"))
    }
}

impl Parse for _Clause {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Bracket) {
            let content;
            syn::bracketed!(content in input);

            let name: _Expr = _Expr::_Identifier(content.parse()?);
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



fn parse_declare(input: ParseBuffer) -> syn::Result<_Expr> {
    // get the 'declare' keyword
    syn::custom_keyword!(declare);
    let _ : declare = input.parse()?;

    // parse the clauses
    let content;
    syn::parenthesized!(content in input);
    let clauses: Vec<_Clause> = parse_zero_or_more(&content);

    // get the 'in' keyword
    let _ = input.parse::<Token![in]>()?;

    // unpack the clauses and body expression
    let body: _Expr = input.parse()?;
    let params: Vec<_Expr> = clauses.clone().into_iter()
        .map(|clause| clause.name).collect();
    let args: Vec<_Expr> = clauses.clone().into_iter()
        .map(|clause| *clause.binding).collect();

    // generate the AST node
    Ok(_Expr::_Application {
        proc: Box::new(_Expr::_Procedure {
            body: Box::new(body),
            params,
        }),
        args,
    })
}

fn parse_procedure(input: ParseBuffer) -> syn::Result<_Expr> {
    syn::custom_keyword!(proc);
    let _ : proc = input.parse()?;

    let param_content;
    syn::parenthesized!(param_content in input);
    let params: Vec<_Expr> = parse_zero_or_more(&param_content);

    let body: _Expr = input.parse()?;

    Ok(_Expr::_Procedure{
        params,
        body: Box::new(body),
    })
}

fn parse_application(input: ParseBuffer) -> syn::Result<_Expr> {
    let procedure: _Expr = input.parse()?;
    let args: Vec<_Expr> = parse_zero_or_more(&input);

    Ok(_Expr::_Application{
        proc: Box::new(procedure),
        args,
    })
}

fn parse_atoms(input: ParseStream) -> syn::Result<_Expr> {
    if input.peek(syn::LitStr) {
        Ok(_Expr::_String(input.parse()?))
    } else if input.peek(syn::LitInt) {
        Ok(_Expr::_Number(input.parse()?))
    } else if input.peek(syn::Ident) || input.peek(syn::LitBool) || peek_op(&input) {
        Ok(_Expr::_Identifier(input.parse()?))
    } else {
        Err(input.error("Failed to Parse Atom"))
    }
}

fn parse_zero_or_more<T: Parse>(input: &ParseBuffer) -> Vec<T> {
    let mut result: Vec<T> = Vec::new();
    while let Ok(item) = input.parse() {
        result.push(item);
    }
    result
}

fn peek_op(input: &ParseStream) -> bool {
    // Define a list of concrete token types
    let prim_ops: [&dyn Fn(&ParseStream) -> bool; 6] = [
        &|input| input.peek(Plus),
        &|input| input.peek(Minus),
        &|input| input.peek(Star),
        &|input| input.peek(Slash),
        &|input| input.peek(EqEq),
        &|input| input.peek(Le),
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
        _Expr::_Identifier(id) => {
            quote! {
                println!("{}Identifier: {}", #indent_str, stringify!(#id));
            }
        },
        _Expr::_String(s) => {
            quote! {
                println!("{}String: {}", #indent_str, #s);
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
        _Expr::_Procedure { params, body } => {
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
pub fn interp(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input as _Expr
    let expr = syn::parse_macro_input!(input as _Expr);

    let tokens = walk_ast(&expr);

    proc_macro::TokenStream::from(tokens)
}


fn walk_ast(expr: &_Expr) -> TokenStream {
    match expr {
        _Expr::_Number(n) => { quote! { #n } },
        _Expr::_Identifier(id_t) => {
            match id_t {
                _IdTypes::_Op(op) => quote! { #op },
                _IdTypes::_Id(id) => quote! { #id },
                _IdTypes::_Bool(bl) => quote! { #bl },
            }
        },
        _Expr::_String(s) => { quote! { #s } },
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
        _Expr::_Procedure { params, body } => {
            let param_tokens: Vec<TokenStream> = params.iter().map(|param| {
                walk_ast(param)
            }).collect();
            let body_exp: TokenStream = walk_ast(body);

            quote! {
               ( | #(#param_tokens),* | #body_exp )
            }
        },
        _Expr::_Application { proc, args } => {
            let arg_tokens: Vec<TokenStream> = args.iter().map(|arg| {
                walk_ast(arg)
            }).collect();

            match *proc.clone() {
                _Expr::_Identifier(id) => match id {
                    _IdTypes::_Op(_) => {
                        handle_prim(walk_ast(proc), arg_tokens)
                    }
                    _IdTypes::_Id(_) | _IdTypes::_Bool(_) => {
                        handle_proc(walk_ast(proc), arg_tokens)
                    }
                }
                _ => {
                    handle_proc(walk_ast(proc), arg_tokens)
                }
            }
        }
    }
}

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

fn handle_proc(proc: TokenStream, args: Vec<TokenStream>) -> TokenStream {
    quote! {
        #proc(#(#args),*)
    }
}

