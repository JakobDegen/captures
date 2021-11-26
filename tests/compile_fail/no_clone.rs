use captures::*;

// Check the basic error message on a missing `Clone` impl
fn no_generic() {
    struct S;
    let s = S;

    let f = capture! (
        clone s,
        move || {
            let l = s;
            l
        }
    );
    f();
}

// Check the error message when the type is generic
fn no_derive() {
    struct S<T>(T);
    let s = S(0);

    let f = capture! (
        clone s,
        move || {
            let l = s;
            l
        }
    );
    f();
}

// Check the error message when only the generic parameter is missing the impl
fn no_impl() {
    struct NonClone;

    #[derive(Clone)]
    struct S<T>(T);
    let s = S(NonClone);

    let f = capture! (
        clone s,
        move || {
            let l = s;
            l
        }
    );
    f();
}

fn main() {
    no_generic();
    no_derive();
    no_impl();
}
