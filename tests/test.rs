use assert_no_alloc::*;
use std::{ops::Add, panic::catch_unwind};

#[global_allocator]
static A: AllocDisabler = AllocDisabler;

#[cfg(not(feature = "warn_debug"))]
compile_error!("The test suite requires the warn_debug feature to be enabled. Use `cargo test --features warn_debug`");

// This is only a kludge; what we actually want to check is "will do_alloc() be optimized out?", e.g. due to
// compiler optimizations turned on in --release mode. We can't do that, the closest we can get is to check
// whether debug_assertions are disabled, which coincidentially also happens in release mode.
#[cfg(not(debug_assertions))]
compile_error!("The test suite only works in debug mode. Use `cargo test --features warn_debug`");

#[cfg(feature = "warn_debug")]
fn check_and_reset() -> bool {
	let result = violation_count() > 0;
	reset_violation_count();
	result
}

// Provide a stub check_and_reset() function if warn_debug is disabled. This will never be compiled due to the
// compile_error!() above, but this stub ensures that the output will not be cluttered with spurious error
// messages.
#[cfg(not(feature = "warn_debug"))]
fn check_and_reset() -> bool { unreachable!() }

fn do_alloc() -> Box<u32> {
	Box::new(42)
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
	check_and_reset(); // unwinding might have allocated memory; we don't care about that.
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
		
		check_and_reset(); // unwinding might have allocated memory; we don't care about that.
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
		
		check_and_reset(); // unwinding might have allocated memory; we don't care about that.
		do_alloc();
		assert_eq!(check_and_reset(), true);
		});
	});
	check_and_reset(); // unwinding might have allocated memory; we don't care about that.
	do_alloc();
	assert_eq!(check_and_reset(), false);
}

#[test]
fn test_permit_alloc() {
	assert_eq!(check_and_reset(), false);
	assert_no_alloc(|| {
		permit_alloc(|| {
			do_alloc();
		})
	});
	assert_eq!(check_and_reset(), false);
}

#[test]
fn test_drop() {
	assert_eq!(check_and_reset(), false);
	assert_no_alloc(|| {
		let value = permit_alloc(|| do_alloc());
		// implicit drop
	});
	assert_eq!(check_and_reset(), true);
}

#[test]
fn test_permit_drop() {
	assert_eq!(check_and_reset(), false);
	assert_no_alloc(|| {
		let value = PermitDrop::new(permit_alloc(|| do_alloc()));
		// can call methods on wrapped value
		assert_eq!(value.abs_diff(32), 10);

		// implicit drop
	});
	assert_eq!(check_and_reset(), false);
}

#[test]
fn test_permit_drop_alloc() {
	struct S(Box<u32>);
	impl S {
		fn perform_alloc(&self) -> u32 {
			do_alloc().add(*self.0)
		}
	}

	assert_eq!(check_and_reset(), false);
	assert_no_alloc(|| {
		let value = PermitDrop::new(permit_alloc(|| S(do_alloc())));

		// calling allocating methods should still be recognized even if value is wrapped in PermitDrop
		value.perform_alloc();

		// implicit drop
	});
	assert_eq!(check_and_reset(), true);
}

#[test]
fn test_dealloc() {
	struct S(Box<u32>);
	impl S {
		fn consume_self(self) {}
	}

	assert_eq!(check_and_reset(), false);
	assert_no_alloc(|| {
		let value = permit_alloc(|| S(do_alloc()));
		// performs dealloc
		value.consume_self();
	});
	assert_eq!(check_and_reset(), true);
}

#[test]
fn test_permit_dealloc() {
	struct S(Box<u32>);
	impl S {
		fn consume_self(self) {}
	}

	assert_eq!(check_and_reset(), false);
	assert_no_alloc(|| {
		let value = permit_alloc(|| S(do_alloc()));

		permit_dealloc(|| {
			value.consume_self();
		});
	});
	assert_eq!(check_and_reset(), false);
}

#[test]
fn test_permit_dealloc_alloc() {
	assert_eq!(check_and_reset(), false);
	assert_no_alloc(|| {
		permit_dealloc(|| {
			// this is not permitted
			do_alloc();
		});
	});
	assert_eq!(check_and_reset(), true);
}
