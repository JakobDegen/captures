use captures::*;

// check that `all` directives really do capture *everything*
fn main() {
    let mut a = (0, 0);
    let f = capture!(all a, || a.0 + 1);
    a.1 += 1;
    f();
}
