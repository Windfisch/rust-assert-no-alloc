/* assert_no_alloc -- A custom Rust allocator allowing to temporarily disable
 * memory (de)allocations for a thread.
 *
 * Copyright (c) 2020 Florian Jung <flo@windfis.ch>
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * Redistributions of source code must retain the above copyright notice, this
 * list of conditions and the following disclaimer.
 *
 * THIS SOFTWARE IS PROVIDED BY THE AUTHOR "AS IS" AND ANY EXPRESS OR IMPLIED
 * WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO
 * EVENT SHALL THE AUTHOR BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS;
 * OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
 * WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR
 * OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF
 * ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

use std::alloc::{System,GlobalAlloc,Layout};
use std::cell::Cell;

thread_local! {
	static ALLOC_FORBID_COUNT: Cell<u32> = Cell::new(0);
}

pub struct AllocDisabler;

pub fn assert_no_alloc<T, F: FnOnce() -> T> (func: F) -> T {
	// RAII guard for managing the forbid counter. This is to ensure correct behaviour
	// when catch_unwind is used
	struct Guard;
	impl Guard {
		fn new() -> Guard {
			ALLOC_FORBID_COUNT.with(|c| c.set(c.get()+1));
			Guard
		}
	}
	impl Drop for Guard {
		fn drop(&mut self) {
			ALLOC_FORBID_COUNT.with(|c| c.set(c.get()-1));
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
				let old = ALLOC_FORBID_COUNT.with(|c| c.get());
				ALLOC_FORBID_COUNT.with(|c| c.set(0));
				Guard(old)
			}
		}
		impl Drop for Guard {
			fn drop(&mut self) {
				ALLOC_FORBID_COUNT.with(|c| c.set(self.0));
			}
		}

		let forbid_count = ALLOC_FORBID_COUNT.with(|f| f.get());
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
