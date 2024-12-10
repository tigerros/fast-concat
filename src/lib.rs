extern crate proc_macro;

mod util;
mod concat_strings_parser;
mod concat_paths_parser;

use proc_macro::{TokenStream};
use proc_macro2::{Ident, Span};
use syn::{Expr, Lit, parse_macro_input, parse_quote, Stmt, Type};
use quote::{quote, ToTokens};
use crate::concat_paths_parser::ConcatPathsParser;
use crate::concat_strings_parser::ConcatStringsParser;

// Closures are hard
fn get_var_ident(var_decls: &[(Ident, Stmt)]) -> Ident {
    Ident::new(&util::concat_strings!("v", &var_decls.len().to_string()), Span::call_site())
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
/// In other words, it will return a [`String`].
///
/// # Syntax
///
/// Any amount of `&str`s separated by commas.
/// You will get a helpful error if you entered a wrong type.
/// You can also start your input with `strict:`, in which case the macro will always return
/// a [`String`]. However, returning `&'static str` involves no runtime.
///
/// # Comparison with other macros
///
/// This is faster than all other string concatenating crates (I checked those in [hoodie/concatenation_benchmarks-rs](https://github.com/hoodie/concatenation_benchmarks-rs#additional-macro-benches)).
///
/// However, efficiency is not the biggest problem.
/// All these crates (except [string_concat](https://crates.io/crates/string_concat))
/// evaluate each expression twice; once when calling `.len()` to calculate the string buffer capacity,
/// and once when actually pushing the expression to the buffer.
/// **That's slower and will break things if the expression isn't pure.**
/// This macro prevents this, but also has other optimizations, which none of the other crates do.
///
/// # Optimizations
///
/// - Each expression will get it's own variable and thus won't be evaluated twice at runtime.
/// - If you pass two or more literals in a row, they will be concatenated instead of pushing them to the buffer multiple times.
/// - If you only pass literals, this macro will act as the [`concat!`] macro and no runtime
/// code will be generated. For convenience, the [`concat!`] macro will not be generated either.
#[proc_macro]
pub fn concat_strings(input: TokenStream) -> TokenStream {
    let body = parse_macro_input!(input as ConcatStringsParser);

    if body.args.is_empty() {
        let expanded = if body.strict {
            quote!(::alloc::string::String::new())
        } else {
            quote!("")
        };

        return expanded.into_token_stream().into();
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
        #(#var_decl_stmts)*
        let mut buf = ::alloc::string::String::with_capacity(0 #(+ #var_idents.len())*);
        #(buf.push_str(#var_idents);)*
        buf
    }).into_token_stream().into()
}

/// Pass in expressions that evaluate to `&`[`OsStr`],
/// and the macro will efficiently concatenate them into a [`PathBuf`].
/// However, you can also pass in string literals to concatenate them at compile time.
///
/// Technically, you can push anything to a [`PathBuf`],
/// that implements [`AsRef<Path>`] (not just [`OsStr`]),
/// but those types don't have a consistent way to get their length.
/// There is no performance hit, since
/// [`Path`] is actually just a wrapper around [`OsStr`],
/// and converting [`OsStr`] to it is cost-free (see [`Path::new`]).
///
/// [`OsStr`]: std::ffi::OsStr
/// [`PathBuf`]: std::path::PathBuf
/// [`Path`]: std::path::Path
/// [`Path::new`]: std::path::Path::new
#[proc_macro]
pub fn concat_paths(input: TokenStream) -> TokenStream {
    let body = parse_macro_input!(input as ConcatPathsParser);

    if body.args.is_empty() {
        return quote!(::std::path::PathBuf::new()).into_token_stream().into();
    }

    // This is not all literals combined. It's literals combined if they're next to each other.
    // A non-literal expression breaks the chain.
    let mut combined_lit = String::new();
    // The Ident is the var name, the Stmt is the actual code.
    let mut var_decls: Vec<(Ident, Stmt)> = Vec::with_capacity(body.args.len());
    let mut only_lits = true;
    let var_type: Type = parse_quote!(&::std::ffi::OsStr);

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
        combined_lit.push('/');
    }

    // Remove last /
    combined_lit.pop();

    if !combined_lit.is_empty() {
        let var_ident = get_var_ident(&var_decls);
        var_decls.push((var_ident.clone(), parse_quote!(let #var_ident: #var_type = #combined_lit;)));
    }

    if only_lits {
        return quote!(#combined_lit).into_token_stream().into();
    }

    let var_idents = var_decls.iter().map(|v| &v.0).collect::<Vec<_>>();
    let var_decl_stmts = var_decls.iter().map(|v| &v.1);

    quote!({
        #(#var_decl_stmts)*
        let mut buf = ::std::path::PathBuf::with_capacity(0 #(+ #var_idents.len())*);
        #(buf.push(#var_idents);)*
        buf
    }).into_token_stream().into()
}
