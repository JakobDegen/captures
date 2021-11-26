use captures::*;

// Check that `capture_only` does what we expected
fn basic() {
    let a = 1;
    let b = 2;
    let f = capture_only!(clone a, move || {
        let mut total = 0;
        total += a;
        total += b;
        total
    });
}

fn main() {
    basic();
}
