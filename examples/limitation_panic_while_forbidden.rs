use assert_no_alloc::*;

#[global_allocator]
static A: AllocDisabler = AllocDisabler;

fn main() {
	assert_no_alloc(|| {
		panic!("unrelated panic");
	});
}
