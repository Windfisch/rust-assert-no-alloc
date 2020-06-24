assert_no_alloc
===============

This crate provides a custom allocator that allows to temporarily disable
memory (de)allocations for a thread. If a (de)allocation is attempted
anyway, the program will abort or print a warning.

It uses thread local storage for the "disabled-flag/counter", and thus
should be thread safe, if the underlying allocator (currently hard-coded
to `std::alloc::System`) is.

Rationale
---------

No-allocation-zones are relevant e.g. in real-time scenarios like audio
callbacks. Allocation and deallocation can take unpredictable amounts of
time, and thus can *sometimes* lead to audible glitches because the audio
data is not served in time.

Debugging such problems can be hard, because it is difficult to reproduce
such problems consistently. Avoiding such problems is also hard, since
allocation/deallocation is a common thing to do and most libraries are not
explicit whether certain functions can allocate or not. Also, this might
even depend on the run-time situation (e.g. a `Vec::push` might allocate,
but it is guaranteed to not allocate *if* enough space has been `reserve()`d
before).

To aid the developer in tackling these problems, this crate offers an easy
way of detecting all forbidden allocations.

How to use
----------

First, configure the features: `warn_debug` and `warn_release` change the
behaviour from aborting your program into just printing an error message
on `stderr`. Aborting is useful for debugging purposes, as it allows you
to retrieve a stacktrace, while warning is less intrusive.

Note that you need to disable the (default-enabled) `disable_release` feature
by specify `default-features = false` if you want to use `warn_release`. If
`disable_release` is set (which is the default), then this crate will do
nothing if built in `--release` mode.

Second, use the allocator provided by this crate. Add this to `main.rs`:

```rust
use assert_no_alloc::*;

#[global_allocator]
static A: AllocDisabler = AllocDisabler;
```

Third, wrap code sections that may not allocate like this:

```rust
assert_no_alloc(|| {
	println!("This code can not allocate.");
});
```

Advanced use
------------

Values can be returned using:

```rust
let answer = assert_no_alloc(|| { 42 });
```

`assert_no_alloc()` calls can be nested.

The generated panic can be caught using `catch_unwind()`.

Examples
--------

See [examples/main.rs](examples/main.rs) for an example.

You can try out the different feature flags:

- `cargo run --example main` -> memory allocation of 4 bytes failed. Aborted (core dumped)
- `cargo run --example main  --release --no-default-features` -> same as above.
- `cargo run --example main --features=warn_debug` -> Tried to (de)allocate memory in a thread that forbids allocator calls! This will not be executed if the above allocation has aborted.
- `cargo run --example main --features=warn_release --release --no-default-features` -> same as above.
- `cargo run --example main --release` will not even check for forbidden allocations

Test suite
----------

The tests will fail to compile with the default features. Run them using:

```
cargo test --features=warn_debug
```
