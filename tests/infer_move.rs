use captures::*;

fn takes_static<T: 'static + FnOnce() -> i32>(f: T) -> i32 {
    f()
}

// Checks that we correctly make a closure taking `clone` move
fn clone_dir() {
    let a: i32 = 0;
    let out = takes_static(capture!(
        clone mut a,
        || { a += 2; a }
    ));
    assert_eq!(out, 2);
}

// Checks that we correcty make a closure taking `with` move
fn with_dir() {
    let out = takes_static(capture!(
        with mut a = 10,
        || { a += 2; a }
    ));
    assert_eq!(out, 12);
}

// Checks that we don't make mistakes when combining lots of these
fn combination() {
    let a: i32 = 0;
    let b: i32 = 3;
    let out = takes_static(capture!(
        clone a,
        clone b,
        with mut c = 10,
        with d = 100,
        || { c += a + b + d; c }
    ));
    assert_eq!(out, 113);
}

// Dont emit garbage if the user provides one
fn unnecessary() {
    let out = takes_static(capture!(
        with d = 50,
        move || d + 10
    ));
    assert_eq!(out, 60);
}

fn main() {
    clone_dir();
    with_dir();
    combination();
    unnecessary();
}
