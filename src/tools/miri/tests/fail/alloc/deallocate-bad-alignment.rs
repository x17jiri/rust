use std::alloc::{alloc, dealloc, Layout};

fn main() {
    unsafe {
        let x = alloc(Layout::from_size_align_unchecked(1, 1));
        dealloc(x, Layout::from_size_align_unchecked(1, 2)); //~ERROR: has size 1 and alignment 1, but gave size 1 and alignment 2
    }
}
