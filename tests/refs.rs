use captures::*;

// Verify the simple case
fn main() {
    let mut a = 10;
    let b = 12;
    let mut f = capture!(ref mut a, ref b, move || {
        *a += *b;
    });
    f();
    assert_eq!(a, 22);
}
