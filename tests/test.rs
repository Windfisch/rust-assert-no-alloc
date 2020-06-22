use assert_no_alloc::*;
use std::panic::catch_unwind;

#[global_allocator]
static A: AllocDisabler = AllocDisabler;

fn do_alloc() {
	let _tmp: Box<u32> = Box::new(42);
}

#[test]
fn ok_noop() {
	do_alloc();
}

#[test]
fn ok_simple() {
	assert_no_alloc(|| {
	});

	do_alloc();
}

#[test]
fn ok_nested() {
	assert_no_alloc(|| {
		assert_no_alloc(|| {
		});
	});

	do_alloc();
}

#[test]
#[should_panic(expected = "Tried to (de)allocate memory in a thread forbids allocator calls!")]
fn forbidden_simple() {
	assert_no_alloc(|| {
		do_alloc();
	});
}

#[test]
#[should_panic(expected = "Tried to (de)allocate memory in a thread forbids allocator calls!")]
fn forbidden_in_nested() {
	assert_no_alloc(|| {
		assert_no_alloc(|| {
			do_alloc();
		});
	});
}

#[test]
#[should_panic(expected = "Tried to (de)allocate memory in a thread forbids allocator calls!")]
fn forbidden_after_nested() {
	assert_no_alloc(|| {
		assert_no_alloc(|| {
		});
		do_alloc();
	});
}

#[test]
fn ok_after_unwind() {
	let result = catch_unwind(|| {
		assert_no_alloc(|| {
			do_alloc();
		});
	});

	assert!(result.is_err());

	do_alloc();
}

#[test]
#[should_panic(expected = "Tried to (de)allocate memory in a thread forbids allocator calls!")]
fn forbidden_after_unwind() {
	let result = catch_unwind(|| {
		assert_no_alloc(|| {
			do_alloc();
		});
	});

	assert!(result.is_err());

	assert_no_alloc(|| {
		do_alloc();
	});
}
