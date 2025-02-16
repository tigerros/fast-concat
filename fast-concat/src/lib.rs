#![no_std]
#![doc = include_str!("../README.md")]
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
pub use constcat::concat as constcat;
/// Concatenates string expressions.
///
/// If you only pass in literals or constants, you will get a const `&'static str` back.
/// Otherwise, this macro will create a buffer with the optimal capacity and push every string to it.
///
/// # Syntax
///
/// Any amount of expressions that evaluate to a `&str` separated by commas.
/// An expression can be prefixed with `const` to indicate that it is constant.
///
/// # Examples
///
/// ```rust
/// # use fast_concat_macro::fast_concat;
/// const CONST: &str = "const ";
/// let var = "var ";
/// let mut buf = String::new();
///
/// // use const keyword to indicate that it is constant
/// let expansion = fast_concat!("lit0 ", const CONST, var, "lit1 ", "lit2 ", 9, {
///     for i in 0..10 {
///         buf.push_str(&i.to_string());
///     }
///
///     &buf
/// });
///
/// buf.clear();
///
/// // what the value is
/// assert_eq!(expansion, "lit0 const var lit1 lit2 90123456789");
///
/// // what code gets generated
/// assert_eq!(expansion, {
///     extern crate alloc;
///     // constcat generates these
///     const _0: &'static str = "lit0 const ";
///     let _1: &str = var;
///     const _2: &'static str = "lit1 lit2 9";
///     let _3: &str = {
///         for i in 0..10 {
///             buf.push_str(&i.to_string());
///         }
///
///         &buf
///     };
///
///     let mut buf = alloc::string::String::with_capacity(0 + _0.len() + _1.len() + _2.len() + _3.len());
///     buf.push_str(_0);
///     buf.push_str(_1);
///     buf.push_str(_2);
///     buf.push_str(_3);
///     buf
/// });
/// ```
///
/// ```rust
/// # use fast_concat_macro::fast_concat;
///
/// const ASSETS_DIR: &str = "./assets";
/// const ICON: &str = fast_concat!(const ASSETS_DIR, '/', "icon.png");
///
/// assert_eq!(ICON, "./assets/icon.png");
/// assert_eq!(ICON, {
///     const OUTPUT: &'static str = ::fast_concat::constcat!(ASSETS_DIR, '/', "icon.png" , );
///     OUTPUT
/// });
/// ```
pub use fast_concat_macro::fast_concat;

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::string::{String, ToString};
    use fast_concat_macro::fast_concat;

    #[test]
    fn concat() {
        const CONST: &str = "const ";
        let var = "var ";
        let mut buf = String::new();

        macro_rules! impure_expr {
            ($buf:ident) => {{
                for i in 0..10 {
                    $buf.push_str(&i.to_string());
                }
                &$buf
            }};
        }

        let correct_output = "lit0 const var lit1 lit2 90123456789";

        assert_eq!(
            fast_concat!("lit0 ", const CONST, var, "lit1 ", "lit2 ", 9, impure_expr!(buf)),
            correct_output
        );

        let correct_output = "lit0 const var lit1 lit2 901234567890123456789";

        assert_eq!(
            fast_concat!("lit0 ", const CONST, var, "lit1 ", "lit2 ", 9, impure_expr!(buf)),
            correct_output
        );
    }
}
