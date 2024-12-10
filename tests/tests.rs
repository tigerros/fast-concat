#![no_std]

extern crate alloc;
use alloc::string::{String, ToString};
use string_concat::string_concat_impl;
use fast_concat::fast_concat;

macro_rules! test_macro {
    ($macro:path) => {{
        const CONST: &str = "const";
        let var = "var";
        let mut buf = String::new();

        $macro!(CONST, var, "literal", "literal2", {
            for i in 0..10 {
                buf.push_str(&i.to_string());
            }

            &buf
        })
    }}
}

#[test]
fn concat_strings() {
    let correct_output = "constvarliteralliteral20123456789";

    // enable std to test this
    // assert_ne because concat-string is incorrect
    // assert_ne!(test_macro!(concat_string::concat_string), correct_output);
    assert_eq!(test_macro!(string_concat::string_concat), correct_output);
    assert_eq!(test_macro!(fast_concat), correct_output);
    
    const A: &str = "A";
    const C: &str = constcat::concat!(A, "666");
}