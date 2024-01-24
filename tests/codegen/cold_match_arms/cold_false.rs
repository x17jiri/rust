// compile-flags: -O
#![crate_type = "lib"]

#[inline(never)]
#[no_mangle]
pub fn hot_function() {
    println!("hot");
}

#[inline(never)]
#[no_mangle]
pub fn cold_function() {
    println!("cold");
}

#[cold]
fn cold_path() {}

#[no_mangle]
pub fn f(x: bool) {
    match x {
        true => hot_function(),
        false => { cold_path(); cold_function() }
    }
}

// CHECK-LABEL: @f(
// CHECK: br i1 %x, label %bb2, label %bb1, !prof ![[NUM:[0-9]+]]
// CHECK: bb1:
// CHECK: cold_function
// CHECK: bb2:
// CHECK: hot_function
// CHECK: ![[NUM]] = !{!"branch_weights", i32 2000, i32 1}
