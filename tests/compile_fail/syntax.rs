use captures::*;

// Detects garbage and tells us about all of it
fn multierror() {
    capture!(
        garbage a,
        garbage a b c d e f,
        mut garbage a b,
        all mut a,
        clone mut mut,
        clone ;,
        ref clone a,
        mut clone a,
        with clone b = ex,
        with a = ref mut,
        with a = 1 2 3 4,
    );
}

fn main() {
    multierror();
}
