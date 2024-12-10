extern crate alloc;

use std::ffi::OsStr;
use string_concat::string_concat_impl;

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
    
    assert_eq!(test_macro!(concat_string::concat_string), correct_output);
    assert_eq!(test_macro!(string_concat::string_concat), correct_output);
    assert_eq!(test_macro!(concat_x::concat_strings), correct_output);

    concat_x::concat_strings!(strict: "aa");
    concat_x::concat_paths!(OsStr::new("aaf"));
}