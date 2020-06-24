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

// check for mutually exclusive features.
#[cfg(all(feature = "disable_release", feature = "warn_release"))]
compile_error!("disable_release cannot be active at the same time with warn_release");


#[cfg(not(all(feature = "disable_release", not(debug_assertions))))] // if not disabled
thread_local! {
	static ALLOC_FORBID_COUNT: Cell<u32> = Cell::new(0);
	
	#[cfg(any( all(feature="warn_debug", debug_assertions), all(feature="warn_release", not(debug_assertions)) ))]
	static ALLOC_VIOLATION_COUNT: Cell<u32> = Cell::new(0);
}

#[cfg(all(feature = "disable_release", not(debug_assertions)))] // if disabled
pub fn assert_no_alloc<T, F: FnOnce() -> T> (func: F) -> T { // no-op
	func()
}

#[cfg(not(all(feature = "disable_release", not(debug_assertions))))] // if not disabled
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


	#[cfg(any( all(feature="warn_debug", debug_assertions), all(feature="warn_release", not(debug_assertions)) ))] // if warn mode is selected
	let old_violation_count = violation_count();


	let guard = Guard::new(); // increment the forbid counter
	let ret = func();
	std::mem::drop(guard);    // decrement the forbid counter


	#[cfg(any( all(feature="warn_debug", debug_assertions), all(feature="warn_release", not(debug_assertions)) ))] // if warn mode is selected
	if violation_count() > old_violation_count {
		eprintln!("Tried to (de)allocate memory in a thread that forbids allocator calls!");
	}


	return ret;
}

#[cfg(any( all(feature="warn_debug", debug_assertions), all(feature="warn_release", not(debug_assertions)) ))] // if warn mode is selected
pub fn violation_count() -> u32 {
	ALLOC_VIOLATION_COUNT.with(|c| c.get())
}

#[cfg(any( all(feature="warn_debug", debug_assertions), all(feature="warn_release", not(debug_assertions)) ))] // if warn mode is selected
pub fn reset_violation_count() {
	ALLOC_VIOLATION_COUNT.with(|c| c.set(0));
}




pub struct AllocDisabler;

#[cfg(not(all(feature = "disable_release", not(debug_assertions))))] // if not disabled
impl AllocDisabler {
	fn check(&self, _layout: Layout) {
		let forbid_count = ALLOC_FORBID_COUNT.with(|f| f.get());
		if forbid_count > 0 {
			// we may not use println! here, as it will stall when unwinding from a panic.

			#[cfg(any( all(feature="warn_debug", debug_assertions), all(feature="warn_release", not(debug_assertions)) ))] // if warn mode is selected
			ALLOC_VIOLATION_COUNT.with(|c| c.set(c.get()+1));

			#[cfg(any( all(not(feature="warn_debug"), debug_assertions), all(not(feature="warn_release"), not(debug_assertions)) ))] // if abort mode is selected
			std::alloc::handle_alloc_error(_layout);
		}
	}
}

#[cfg(all(feature = "disable_release", not(debug_assertions)))] // if disabled
impl AllocDisabler {
	fn check(&self, _layout: Layout) {} // no-op
}

unsafe impl GlobalAlloc for AllocDisabler {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		self.check(layout);
		System.alloc(layout)
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		self.check(layout);
		System.dealloc(ptr, layout)
	}
}


