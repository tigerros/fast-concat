use criterion::{black_box, criterion_group, criterion_main, Criterion};
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

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fast-concat", |b| b.iter(|| black_box(test_macro!(fast_concat::fast_concat))));
    c.bench_function("concat-strings", |b| b.iter(|| black_box(test_macro!(fast_concat::fast_concat))));
    c.bench_function("string-concat", |b| b.iter(|| black_box(test_macro!(fast_concat::fast_concat))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);