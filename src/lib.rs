extern crate proc_macro;

mod util;
mod concat_strings_parser;

use proc_macro::{TokenStream};
use proc_macro2::{Ident, Span};
use syn::{Expr, Lit, parse_macro_input, parse_quote, Stmt, Type};
use quote::{quote, ToTokens};
use crate::concat_strings_parser::ConcatStringsParser;

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

/// The most efficient way to concatenate `&str`s. It's clean too!
/// If you only concatenate literals or constants, you will get a `&'static str` back.
/// Otherwise, this macro will create a buffer with the optimal capacity and push every string to it.
///
/// # Syntax
///
/// Any amount of `&str`s separated by commas.
/// You will get a helpful error if you entered a wrong type.
///
/// # Comparison with other macros
///
/// This is as fast or faster than all other string concatenating crates (I checked those in [hoodie/concatenation_benchmarks-rs](https://github.com/hoodie/concatenation_benchmarks-rs#additional-macro-benches)).
///
/// The fastest of those have problems:
/// - `concat_string_macro` evaluates expressions twice and requires std.
/// - `concat_strs_macro` doesn't work for certain expressions.
/// - `string_concat_macro` is the best, but it doesn't have the last two of the optimizations below.
///   As a nitpick, it also requires that you `use string_concat::string_concat_impl`.
///   I know, I know. Grasping at straws, but I wanted to go over all the differences.
///
/// # Optimizations
///
/// - Each expression gets a variable and thus won't be evaluated twice at runtime.
/// - If you pass two or more literals in a row, they will be concatenated instead of pushing them to the buffer multiple times.
/// - If you only pass literals, this macro will act as the [`concat!`] macro and only a literal will be returned.
#[proc_macro]
pub fn concat_strings(input: TokenStream) -> TokenStream {
    let body = parse_macro_input!(input as ConcatStringsParser);

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