use assert_no_alloc::*;
use std::panic::catch_unwind;

#[global_allocator]
static A: AllocDisabler = AllocDisabler;

fn check_and_reset() -> bool {
	let result = violation_count() > 0;
	reset_violation_count();
	result
}

fn do_alloc() {
	let _tmp: Box<u32> = Box::new(42);
}

#[test]
fn ok_noop() {
	assert_eq!(check_and_reset(), false);
	do_alloc();
	assert_eq!(check_and_reset(), false);
}

#[test]
fn ok_simple() {
	assert_eq!(check_and_reset(), false);
	assert_no_alloc(|| {
	});

	do_alloc();
	assert_eq!(check_and_reset(), false);
}

#[test]
fn ok_nested() {
	assert_eq!(check_and_reset(), false);
	assert_no_alloc(|| {
		assert_no_alloc(|| {
		});
	});

	do_alloc();
	assert_eq!(check_and_reset(), false);
}

#[test]
fn forbidden_simple() {
	assert_eq!(check_and_reset(), false);
	assert_no_alloc(|| {
		do_alloc();
	});
	assert_eq!(check_and_reset(), true);
}

#[test]
fn forbidden_in_nested() {
	assert_eq!(check_and_reset(), false);
	assert_no_alloc(|| {
		assert_no_alloc(|| {
			do_alloc();
		});
	});
	assert_eq!(check_and_reset(), true);
}

#[test]
fn forbidden_after_nested() {
	assert_eq!(check_and_reset(), false);
	assert_no_alloc(|| {
		assert_no_alloc(|| {
		});
		do_alloc();
	});
	assert_eq!(check_and_reset(), true);
}

#[test]
fn unwind_ok() {
	assert_eq!(check_and_reset(), false);
	assert_no_alloc(|| {
		let r = catch_unwind(|| {
			assert_no_alloc(|| {
				panic!();
			});
		});
		assert!(r.is_err());
	});
	reset_violation_count(); // unwinding might have allocated memory; we don't care about that.
	do_alloc();
	assert_eq!(check_and_reset(), false);
}

#[test]
fn unwind_nested() {
	assert_eq!(check_and_reset(), false);
	assert_no_alloc(|| {
		let r = catch_unwind(|| {
			assert_no_alloc(|| {
				panic!();
			});
		});
		assert!(r.is_err());
		
		reset_violation_count(); // unwinding might have allocated memory; we don't care about that.
		do_alloc();
		assert_eq!(check_and_reset(), true);
	});
}

#[test]
fn unwind_nested2() {
	assert_eq!(check_and_reset(), false);
	assert_no_alloc(|| {
		assert_no_alloc(|| {
		let r = catch_unwind(|| {
			assert_no_alloc(|| {
				assert_no_alloc(|| {
					panic!();
				});
			});
		});
		assert!(r.is_err());
		
		reset_violation_count(); // unwinding might have allocated memory; we don't care about that.
		do_alloc();
		assert_eq!(check_and_reset(), true);
		});
	});
	reset_violation_count(); // unwinding might have allocated memory; we don't care about that.
	do_alloc();
	assert_eq!(check_and_reset(), false);
}
