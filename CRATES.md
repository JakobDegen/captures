# More Powerful Closure Captures

This crate provides simple macros letting you express more powerful closure captures. For example,
you can capture the clone of a value:

```rust
use std::rc::Rc;

let my_val = Rc::new(1);
captures::capture!(clone my_val, move || {
    // `my_val` is cloned here!
});
```

You can also capture arbitrary expressions and override the Edition-2021 capture semantics. Best of
all, you can even specify that your closure should not capture any variables outside the ones you've
listed:

```rust
let a = 1;
let b = 2;
captures::capture_only!(clone a, move || {
    a + b // errors: `b` is unknown
})
```

Consult the [full documentation][documentation] for the details.

[documentation]: https://docs.rs/captures
