#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![deny(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::must_use_candidate)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
#![allow(clippy::module_name_repetitions)]
#![warn(
    clippy::arithmetic_side_effects,
    clippy::unreachable,
    clippy::unchecked_duration_subtraction,
    clippy::todo,
    clippy::string_slice,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::exit,
    clippy::as_conversions,
    clippy::large_futures,
    clippy::large_stack_arrays,
    clippy::large_stack_frames,
    clippy::modulo_one,
    clippy::mem_replace_with_uninit,
    clippy::iterator_step_by_zero,
    clippy::invalid_regex,
    clippy::print_stdout,
    clippy::print_stderr
)]

extern crate proc_macro;

mod fast_concat_parser;

use crate::fast_concat_parser::FastConcatParser;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, parse_quote, Expr, Stmt};

struct OutExpression {
    pub expr: Expr,
    pub is_const: bool,
}

// CLIPPY: Doesn't actually panic
#[allow(clippy::missing_panics_doc)]
#[proc_macro]
pub fn fast_concat(input: TokenStream) -> TokenStream {
    let body = parse_macro_input!(input as FastConcatParser);

    if body.items.is_empty() {
        return quote!("").into_token_stream().into();
    }

    let mut consts = Vec::<Expr>::new();
    let mut out_expressions = Vec::<OutExpression>::with_capacity(body.items.len());

    for item in body.items {
        if item.is_const || matches!(item.expr, Expr::Lit(_)) {
            consts.push(item.expr);
        } else {
            if !consts.is_empty() {
                if consts.len() == 1 {
                    // CLIPPY: Above is len check
                    #[allow(clippy::unwrap_used)]
                    let const_e = consts.pop().unwrap();
                    out_expressions.push(OutExpression {
                        is_const: true,
                        expr: parse_quote!(#const_e),
                    });
                } else {
                    out_expressions.push(OutExpression {
                        is_const: true,
                        expr: parse_quote!(::fast_concat::constcat!(#(#consts,)*)),
                    });
                }

                consts.clear();
            }

            out_expressions.push(OutExpression {
                is_const: false,
                expr: item.expr,
            });
        }
    }

    if !consts.is_empty() {
        out_expressions.push(OutExpression {
            is_const: true,
            expr: parse_quote!(::fast_concat::constcat!(#(#consts,)*)),
        });
    }

    if out_expressions.len() == 1 {
        // CLIPPY: Above is len check
        #[allow(clippy::unwrap_used)]
        let OutExpression { is_const, expr } = out_expressions.pop().unwrap();

        return if is_const {
            quote!({
                const OUTPUT: &'static str = #expr;
                OUTPUT
            })
            .into_token_stream()
            .into()
        } else {
            quote!(#expr).into_token_stream().into()
        };
    }

    let mut variable_idents = Vec::<Ident>::new();
    let mut variable_declarations = Vec::<Stmt>::new();

    for (i, out_expression) in out_expressions.into_iter().enumerate() {
        let OutExpression { is_const, expr } = out_expression;
        let variable_ident = Ident::new(&format!("_{i}"), Span::call_site());

        variable_idents.push(variable_ident.clone());

        if is_const {
            variable_declarations.push(parse_quote!(const #variable_ident: &'static str = #expr;));
        } else {
            variable_declarations.push(parse_quote!(let #variable_ident: &str = #expr;));
        }
    }

    quote!({
        extern crate alloc;
        #(#variable_declarations)*
        let mut buf = alloc::string::String::with_capacity(0 #(+ #variable_idents.len())*);
        #(buf.push_str(#variable_idents);)*
        buf
    })
    .into_token_stream()
    .into()
}
