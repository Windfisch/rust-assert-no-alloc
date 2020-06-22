use std::alloc::{System,GlobalAlloc,Layout};
use std::cell::RefCell;


thread_local! {
	static ALLOC_FORBID_COUNT: RefCell<u32> = RefCell::new(0);
}

pub struct AllocDisabler;

pub fn assert_no_alloc<T, F: FnOnce() -> T> (func: F) -> T {
	// RAII guard for managing the forbid counter. This is to ensure correct behaviour
	// when catch_unwind is used
	struct Guard;
	impl Guard {
		fn new() -> Guard {
			ALLOC_FORBID_COUNT.with(|c| *c.borrow_mut() += 1);
			Guard
		}
	}
	impl Drop for Guard {
		fn drop(&mut self) {
			ALLOC_FORBID_COUNT.with(|c| *c.borrow_mut() -= 1);
		}
	}

	let guard = Guard::new(); // increment the forbid counter
	let ret = func();
	std::mem::drop(guard);    // decrement the forbid counter
	return ret;
}

impl AllocDisabler {
	fn check(&self) {
		// RAII guard for managing the forbid counter. This is to ensure correct behaviour
		// when catch_unwind is used
		struct Guard(u32);
		impl Guard {
			fn new() -> Guard {
				let old = ALLOC_FORBID_COUNT.with(|c| *c.borrow());
				ALLOC_FORBID_COUNT.with(|c| *c.borrow_mut() = 0);
				Guard(old)
			}
		}
		impl Drop for Guard {
			fn drop(&mut self) {
				ALLOC_FORBID_COUNT.with(|c| *c.borrow_mut() = self.0);
			}
		}

		let forbid_count = ALLOC_FORBID_COUNT.with(|f| *f.borrow());
		if forbid_count > 0 {
			//ALLOC_FORBID_COUNT.with(|c| *c.borrow_mut() = 0); // avoid panics in the panic handler
			let _guard = Guard::new(); // set the forbid count to zero temporarily to avoid panics in the panic handler
			panic!("Tried to (de)allocate memory in a thread forbids allocator calls!");
			// if this panic is caught, then at some point guard.drop() will be called, which re-sets the forbid count to its actual state.
			// also, the guards in forbid_alloc() will correctly manage the pointer, so that after the counter should still have the correct
			// value after unwinding.
		}
	}
}

unsafe impl GlobalAlloc for AllocDisabler {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		self.check();
		System.alloc(layout)
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		self.check();
		System.dealloc(ptr, layout)
	}
}
