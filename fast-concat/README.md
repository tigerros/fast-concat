The fastest, constant able (thanks to [constcat](https://github.com/rossmacarthur/constcat)), no-std compatible way to concatenate `&str`s.
Due to how macros work, you need to import constcat as a separate dependency.

# Comparison with other macros

This is as fast or faster than the fastest string concatenating crates (I checked those in [hoodie/concatenation_benchmarks-rs](https://github.com/hoodie/concatenation_benchmarks-rs#additional-macro-benches)).

Those have other problems too:
- `concat_string_macro` evaluates expressions twice and requires std.
- `concat_strs_macro` doesn't work for certain expressions.
- `string_concat_macro` is the best, but it doesn't have the last two of the optimizations below.
  As a nitpick, it also requires that you `use string_concat::string_concat_impl`.
  I know, I know. Grasping at straws, but I wanted to go over all the differences.

# Optimizations

- Each expression gets a variable and thus won't be evaluated twice at runtime.
- If you pass multiple constants or literals in a row, they will be concatenated at compile time and pushed to the buffer only once.
- Passing only constants or literals makes the macro act as the [`concat!`] macro, and a literal will be returned.