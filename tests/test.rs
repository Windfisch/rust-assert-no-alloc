use assert_no_alloc::*;

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

/*
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
}*/
