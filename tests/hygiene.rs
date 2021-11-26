use captures::*;

// Don't wipe the surrounding context
fn context() {
    const FOO: i32 = 0;
    struct Bar;
    fn func() {}

    let mut a = Bar;
    let mut f = capture_only!(all a, || {
        if FOO > 1 {
            a = Bar;
            func();
        }
    });
    f();
}

// we can shadow things
fn shadow() {
    let mut a = 5;
    let mut f = capture_only!(all a, || {
        a += 1;
        a = { // 9
            let a = a;
            a + 3
        };
        a = if let a @ 10..=15 = a {
            a + 3
        } else {
            a
        };
        a = match a {
            a @ 9 => 100,
            a => 50
        };
    });
    f();
    assert_eq!(a, 100);
}

fn main() {
    context();
    shadow();
}
