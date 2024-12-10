extern crate proc_macro;

mod util;
mod fast_concat_parser;

use proc_macro::{TokenStream};
use proc_macro2::{Ident, Span};
use syn::{Expr, Lit, ExprLit, parse_macro_input, parse_quote, Stmt};
use quote::{quote, ToTokens};
use crate::fast_concat_parser::FastConcatParser;

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

    if body.items.is_empty() {
        return quote!("").into_token_stream().into();
    }

    // This is not all literals combined. It's literals combined if they're next to each other.
    // A non-literal expression breaks the chain.
    let mut combined_literals = String::new();
    let mut out_expressions = Vec::<Expr>::with_capacity(body.items.len());
    let mut only_literals = true;

    for item in body.items {
        let Expr::Lit(ExprLit { attrs: _, lit: Lit::Str(str_lit) }) = item.expr else {
            if !combined_literals.is_empty() {
                out_expressions.push(parse_quote!(#combined_literals));
                combined_literals.clear();
            }
            
            out_expressions.push(item.expr);
            only_literals = false;
            continue;
        };

        combined_literals.push_str(&str_lit.value());
    }

    if !combined_literals.is_empty() {
        out_expressions.push(parse_quote!(#combined_literals));
    }

    if only_literals {
        return quote!(#combined_literals).into_token_stream().into();
    }
    
    let mut variable_idents = Vec::<Ident>::new();
    let mut variable_declarations = Vec::<Stmt>::new();
    
    for (i, out_expression) in out_expressions.iter().enumerate() {
        let variable_ident = Ident::new(&format!("x{}", i), Span::call_site());
        
        variable_idents.push(variable_ident.clone());
        variable_declarations.push(parse_quote!(let #variable_ident: &str = #out_expression;));
    }

    quote!({
        extern crate alloc;
        #(#variable_declarations)*
        let mut buf = alloc::string::String::with_capacity(0 #(+ #variable_idents.len())*);
        #(buf.push_str(#variable_idents);)*
        buf
    }).into_token_stream().into()
}