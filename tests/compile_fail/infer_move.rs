use captures::*;

fn takes_static<T: 'static + FnOnce() -> i32>(f: T) -> i32 {
    f()
}

// Dont emit a move if we dont need to
fn main() {
    let mut a: i32 = 0;
    let _ = takes_static(capture!(
        all a,
        || { a += 2; a }
    ));
}
