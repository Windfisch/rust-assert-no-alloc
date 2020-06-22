assert_no_alloc
===============

This crate provides a custom allocator that allows to temporarily disable
memory (de)allocations for a thread. If a (de)allocation is attempted
anyway, the program will panic.

How to use
----------

First, use the allocator provided by this crate. Add this to `main.rs`:

```rust
use assert_no_alloc::*;

#[global_allocator]
static A: AllocDisabler = AllocDisabler;
```

Second, wrap code sections that may not allocate like this:

```rust
assert_no_alloc(|| {
	println!("This code can not allocate.");
});
```

See [examples/main.rs](examples/main.rs) for an example.

Advanced use
------------

Values can be returned using:

```rust
let answer = assert_no_alloc(|| { 42 });
```

`assert_no_alloc()` calls can be nested.

The generated panic can be caught using `catch_unwind()`.

Limitations
-----------

Note that calling `panic!()` itself can allocate memory. This can cause panics
while panicking, an error considered fatal by rust. This crate prevents this
from happening on its own panic by temporarily allowing allocations before
calling `panic!()`. On foreign panics, however, this is not ensured.
See [here](examples/limitation_panic_while_forbidden.rs) for an example of this
happening or run `cargo run --example limitation_panic_while_forbidden`.

Also using `catch_unwind()` while allocations are forbidden, a double panic
occurs. See [here](examples/limitation_catch_unwind_while_forbidden.rs) or run
`cargo run --example limitation_catch_unwind_while_forbidden`.

Both issues seem to cause undefined behaviour, terminating the programs with
`Illegal instruction` (`SIGILL`).
