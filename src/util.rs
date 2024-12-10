/// This is in the crate because you can't use procedural macros in the crate where they're defined.
/// This is a little workaround. Not as efficient though.
macro_rules! concat_strings {
    () => {
        ::std::string::String::new()
    };

    ($($s:literal),+) => {
        ::core::concat!($($s),+)
    };

    ($($s:expr),+) => {{
        let mut buf = ::std::string::String::with_capacity(0$(+$s.len())+);
        $(buf.push_str($s);)+
        buf
    }};
}

pub(crate) use concat_strings;