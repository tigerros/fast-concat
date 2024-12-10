extern crate proc_macro;

mod util;
mod fast_concat_parser;

use proc_macro::{TokenStream};
use proc_macro2::{Ident, Span};
use syn::{Expr, Lit, parse_macro_input, parse_quote, Stmt, Type};
use quote::{quote, ToTokens};
use crate::fast_concat_parser::FastConcatParser;

// Closures are hard
fn get_var_ident(var_decls: &[(Ident, Stmt)]) -> Ident {
    Ident::new(&util::concat_strings!("x", &var_decls.len().to_string()), Span::call_site())
}

fn break_combined_lit_chain(var_decls: &mut Vec<(Ident, Stmt)>, var_type: &Type, new_expr: Expr, combined_lit: &mut String) {
    if !combined_lit.is_empty() {
        let var_ident = get_var_ident(var_decls);
        var_decls.push((var_ident.clone(), parse_quote!(let #var_ident: #var_type = #combined_lit;)));
        combined_lit.clear();
    }

    let var_ident = get_var_ident(var_decls);
    var_decls.push((var_ident.clone(), parse_quote!(let #var_ident: #var_type = #new_expr;)));
}

/// Concatenates string expressions.
/// 
/// - Passing only literals will return a const `&'static str`.
/// - Passing
///
/// If you only pass in literals or constants, you will get a `&'static str` back.
/// Otherwise, this macro will create a buffer with the optimal capacity and push every string to it.
///
/// # Syntax
///
/// Any amount of expressions that evaluate to a `&str` separated by commas.
#[proc_macro]
pub fn fast_concat(input: TokenStream) -> TokenStream {
    let body = parse_macro_input!(input as FastConcatParser);

    if body.args.is_empty() {
        return quote!("").into_token_stream().into();
    }

    // This is not all literals combined. It's literals combined if they're next to each other.
    // A non-literal expression breaks the chain.
    let mut combined_lit = String::new();
    // The Ident is the var name, the Stmt is the actual code.
    let mut var_decls: Vec<(Ident, Stmt)> = Vec::with_capacity(body.args.len());
    let mut only_lits = true;
    let var_type: Type = parse_quote!(&str);

    for expr in body.args {
        let Expr::Lit(ref expr_lit) = expr else {
            break_combined_lit_chain(&mut var_decls, &var_type, expr, &mut combined_lit);
            only_lits = false;
            continue;
        };

        let Lit::Str(string_lit) = &expr_lit.lit else {
            break_combined_lit_chain(&mut var_decls, &var_type, expr, &mut combined_lit);
            only_lits = false;
            continue;
        };

        let string = string_lit.value();

        combined_lit.push_str(&string);
    }

    if !combined_lit.is_empty() {
        let var_ident = get_var_ident(&var_decls);
        var_decls.push((var_ident.clone(), parse_quote!(let #var_ident: &str = #combined_lit;)));
    }

    if only_lits {
        return quote!(#combined_lit).into_token_stream().into();
    }

    let var_idents = var_decls.iter().map(|v| &v.0).collect::<Vec<_>>();
    let var_decl_stmts = var_decls.iter().map(|v| &v.1);

    quote!({
        extern crate alloc;
        #(#var_decl_stmts)*
        let mut buf = alloc::string::String::with_capacity(0 #(+ #var_idents.len())*);
        #(buf.push_str(#var_idents);)*
        buf
    }).into_token_stream().into()
}