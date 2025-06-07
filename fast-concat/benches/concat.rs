extern crate alloc;
use alloc::string::{String, ToString};
use criterion::{criterion_group, criterion_main, Criterion};
use fast_concat_macro::fast_concat;
use std::hint::black_box;
use string_concat::string_concat_impl;

pub fn criterion_benchmark(c: &mut Criterion) {
    macro_rules! impure_expr {
        ($buf:ident) => {{
            for i in 0..10 {
                $buf.push_str(&i.to_string());
            }
            &$buf
        }};
    }

    const CONST: &str = "const ";
    let var = "var ";
    let mut buf = String::new();

    c.bench_function("fast-concat", |b| {
        b.iter(|| {
            black_box(
                fast_concat!("lit0 ", const CONST, var, "lit1 ", "lit2 ", 3, impure_expr!(buf)),
            )
        })
    });
    c.bench_function("string-concat", |b| {
        b.iter(|| {
            black_box(string_concat::string_concat!(
                "lit0 ",
                CONST,
                var,
                "lit1 ",
                "lit2 ",
                "3",
                impure_expr!(buf)
            ))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
