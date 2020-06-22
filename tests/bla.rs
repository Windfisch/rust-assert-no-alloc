use assert_no_alloc::*;
use std::panic::catch_unwind;

#[global_allocator]
static A: AllocDisabler = AllocDisabler;

fn do_alloc() {
	let _tmp: Box<u32> = Box::new(42);
}

#[test]
fn test_ok_noop() {
	do_alloc();
}

#[test]
fn test_ok_simple() {
	forbid_alloc(|| {
	});

	do_alloc();
}

#[test]
fn test_ok_nested() {
	forbid_alloc(|| {
		forbid_alloc(|| {
		});
	});

	do_alloc();
}

#[test]
#[should_panic(expected = "Tried to (de)allocate memory in a thread forbids allocator calls!")]
fn test_forbidden_simple() {
	forbid_alloc(|| {
		do_alloc();
	});
}

#[test]
#[should_panic(expected = "Tried to (de)allocate memory in a thread forbids allocator calls!")]
fn test_forbidden_in_nested() {
	forbid_alloc(|| {
		forbid_alloc(|| {
			do_alloc();
		});
	});
}

#[test]
#[should_panic(expected = "Tried to (de)allocate memory in a thread forbids allocator calls!")]
fn test_forbidden_after_nested() {
	forbid_alloc(|| {
		forbid_alloc(|| {
		});
		do_alloc();
	});
}

#[test]
fn test_ok_after_unwind() {
	let result = catch_unwind(|| {
		forbid_alloc(|| {
			do_alloc();
		});
	});

	assert!(result.is_err());

	do_alloc();
}

#[test]
#[should_panic(expected = "Tried to (de)allocate memory in a thread forbids allocator calls!")]
fn test_forbidden_after_unwind() {
	let result = catch_unwind(|| {
		forbid_alloc(|| {
			do_alloc();
		});
	});

	assert!(result.is_err());

	forbid_alloc(|| {
		do_alloc();
	});
}
