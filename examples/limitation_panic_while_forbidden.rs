use assert_no_alloc::*;

#[global_allocator]
static A: AllocDisabler = AllocDisabler;

fn main() {
	forbid_alloc(|| {
		panic!("unrelated panic");
	});
}
