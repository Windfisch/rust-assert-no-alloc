use assert_no_alloc::*;
use std::panic::catch_unwind;

#[global_allocator]
static A: AllocDisabler = AllocDisabler;

fn do_alloc() {
	let _tmp: Box<u32> = Box::new(42);
}

fn main() {
	let result = assert_no_alloc(|| {
		catch_unwind(|| {
			assert_no_alloc(|| {
				do_alloc();
			});
		})
	});
}
