//! Provides two macros for more powerful closure captures.
//!
//! # Background
//!
//! Closures in Rust, despite being extremely powerful, do not offer many options for modifying the
//! way in which they capture their output. A particular pain point is often needing to `.clone()`
//! an `Arc<T>` or `Rc<T>` for the closure to capture. This pattern does not compile:
//! ```compile_fail
//! # use std::rc::Rc;
//! fn needs_static<T: FnOnce() -> i32 + 'static>(f: T) -> i32 {
//!     f()
//! }
//!
//! let local: Rc<i32> = Rc::new(1);
//! // Try and capture a clone of the `Rc`
//! let mut f = || {
//!     let in_closure = local.clone();
//!     *in_closure.as_ref()
//! };
//! // `f` is not `'static`!
//! assert_eq!(needs_static(f), 1);
//! // `local` has not been captured!
//! assert_eq!(*local.as_ref(), 1);
//! ```
//!
//! That's because when writing `local.clone()` in the body of the closure, that `clone` call is not
//! executed until the closure is called; this means that the closure is actually capturing a
//! `&local` and so it's not `'static`! Making `f` a `move` closure does not fix this, since then
//! `local` will be captured by value, and the later `local.as_ref()` statement will fail. What we
//! want instead is for the `.clone()` to be executed when the closure is created:
//! ```
//! # use std::rc::Rc;
//! fn needs_static<T: FnOnce() -> i32 + 'static>(f: T) -> i32 {
//!     f()
//! }
//!
//! let local: Rc<i32> = Rc::new(1);
//! // Actually capture a clone of the `Rc`
//! let cloned = local.clone();
//! let f = move || {
//!     let in_closure = cloned;
//!     *in_closure.as_ref()
//! };
//! // `f` is now `'static`!
//! assert_eq!(needs_static(f), 1);
//! // `local` has not been captured!
//! assert_eq!(*local.as_ref(), 1);
//! ```
//!
//! # Usage
//!
//! The `capture!` and `capture_only!` macros are invoked with a comma-seperated
//! list of "capture directives" and finally a closure expression. One example of a capture
//! directive is the `clone x` directive, which indicates that a clone of `x` should be captured in
//! place of `x`. As such, the example above can be re-written to:
//! ```
//! # use std::rc::Rc;
//! use captures::capture;
//! fn needs_static<T: FnOnce() -> i32 + 'static>(f: T) -> i32 {
//!     f()
//! }
//!
//! let local: Rc<i32> = Rc::new(1);
//! // Actually capture a clone of the `Rc`
//! let f = capture!(clone local,
//!     move || {
//!         let in_closure = local;
//!         *in_closure.as_ref()
//!     }
//! );
//! // `f` is still `'static`!
//! assert_eq!(needs_static(f), 1);
//! // `local` has not been captured!
//! assert_eq!(*local.as_ref(), 1);
//! ```
//!
//! ## Capture Directives
//!
//! These capture directives are currently supported:
//!
//!  - `clone x` captures a clone of `x`.
//!  - `with x = expr` captures a value `x` that is computed from `expr`.
//!  - `all x` captures all of `x`. Beginning in Rust 2021, writing `x.y` in your closure would lead
//!    to only the `y` field of `x` being captured. Specifying `all x` causes all of `x` to be
//!    captured instead. This does not influence whether `x` is captured by value or by reference -
//!    if the closure is a `move` closure, it will still be captured by value, and if it is a
//!    non-`move` closure, the compiler's standard inference algorithm is allowed to make the
//!    decision.
//  - `rename x y` captures `y` outside the closure, but renames it to `x` and allows it to be
//    accessed as `x` inside the body of the closure. This does not force all of `y` to be
//    captured, and it does not influence whether `y` or any of its fields are captured by value or
//    by reference. (not yet supported)
//!
//! To avoid surprises and compilation errors, if you specify a `clone` or `with` directive, then
//! this macro will turn your closure into a move closure if it was not one already. Because of
//! this, if your closure is a `move` closure - either because you explicitly marked it as such or
//! because you used a `with` or `clone` directive - then you may additionally specify these
//! directives:
// FIXME: Decide if its not better to require that the user specify the `move` instead of
// "inferring" it.
//!
//!  - `ref x` captures `x` by immutable reference.
//!  - `ref mut x` captures `x` by mutable reference.
//!
//! The `x` in all of these directives must simply be the name of a local variable. Some more
//! complicated things may be supported in the future. There is at the moment also no support for
//! combining directives. I will add this once I figure out a pretty and consistent way to do it.
//!
//! ## Mutability
//!
//! In Rust, captured variables that are captured by value inherit the mutability of the value they
//! reference. For example,
//! ```compile_fail
//! let a = 1; // immutable
//! let _ = move || {
//!     a += 1;
//!     a
//! };
//! ```
//! does not compile, but if `a` is marked as mutable
//! ```
//! let mut a = 1; // now mutable
//! let _ = move || {
//!     a += 1;
//!     a
//! };
//! ```
//! it does.
//!
//! Unfortunately, this crate does not have the necessary information to reproduce this behavior in
//! general. `clone` and `with` directives create new variables for which it is not clear what their
//! mutability should be. The current policy is for all of them to default to immutable. This may be
//! changed in the future (obviously respecting semver) if it is determined that this is not the
//! best option. If you do want these values to be mutable, you can request that by prefixing the
//! variable with a `mut`. For example,
//!
//! ```compile_fail
//! # use captures::capture;
//! let mut v = vec![1, 2]; // despite being mutable here
//! let _ = capture!(clone v, || {
//!     v.push(3); // we cannot push to `v`, since it is not mutable
//!     v
//! });
//! ```
//! We can fix this via:
//! ```
//! # use captures::capture;
//! let mut v = vec![1, 2];
//! let _ = capture!(clone mut v, || {
//!     v.push(3); // we can push now
//!     v
//! });
//! ```
//! This will still emit a warning because the mutability of the variable `v` outside the closure is
//! unused. Writing instead `let v = vec![1, 2];` would continue to compile and the warning would not
//! be emitted.
//!
//! `all` directives are not affected by this. Variables captured under such a directive, if
//! captured by value, correctly inherit their mutability. As such, the `mut` prefix is not
//! supported on these directives.
//!
//! # `capture_only`
//!
//! The `capture_only` macro behaves exactly like the `capture` macro, with the exception that it
//! additionally prevents any variables that do not have an associated capture directive from
//! being captured. For example,
//! ```compile_fail
//! # use captures::capture_only;
//! let a = 1;
//! let mut b = 10;
//! let mut f = capture_only!(all a, || {
//!     b += 1; // error
//!     a + 1
//! });
//! assert_eq!(f(), 2);
//! assert_eq!(b, 11);
//! ```
//! does not compile, with an error message indicating that there is no local variable `b`.
//! Switching `capture_only` to `capture` would allow the above code to compile. If you would like
//! to indicate that `b` may also be captured, but do not want to add any restrictions on how, you
//! can add an `all` directive:
//! ```
//! # use captures::capture_only;
//! let a = 1;
//! let mut b = 10;
//! let mut f = capture_only!(all a, all b, || {
//!     b += 1; // compiles
//!     a + 1
//! });
//! assert_eq!(f(), 2);
//! assert_eq!(b, 11);
//! ```
//!
use proc_macro2::TokenStream;
use quote::quote;

/// Takes a place with type having `.set_span(_)` and `.span()` methods
macro_rules! make_mixed {
    ($i:expr) => {
        let e = &mut $i;
        e.set_span(e.span().resolved_at(::proc_macro2::Span::mixed_site()));
    };
}

mod changes;
mod clean;
mod parse;

use changes::*;
use parse::*;

/// Captures variables into a closure with special semantics.
///
/// See the [crate level documentation][`crate`] for more info.
#[proc_macro]
pub fn capture(inp: proc_macro::TokenStream) -> proc_macro::TokenStream {
    main(inp.into(), false).into()
}

/// Captures only the listed variables into the closure.
///
/// See the [crate level documentation][`crate`] for more info.
#[proc_macro]
pub fn capture_only(inp: proc_macro::TokenStream) -> proc_macro::TokenStream {
    main(inp.into(), true).into()
}

fn main(inp: TokenStream, only: bool) -> TokenStream {
    let parsed: Input = match syn::parse2::<Input>(inp) {
        Ok(x) => x,
        Err(e) => return e.into_compile_error(),
    };

    let Changes {
        exterior,
        interior,
        exempt,
    } = Changes::from_input(&parsed, only);
    let syn::ExprClosure {
        attrs,
        asyncness,
        movability,
        capture,
        or1_token,
        inputs,
        or2_token,
        output,
        mut body,
    } = parsed.closure;

    assert!(attrs.is_empty());
    if only {
        clean::clean(&mut body, &exempt);
    }

    quote! {
        {
            #exterior
            #asyncness
            #movability
            #capture
            #or1_token
            #inputs
            #or2_token
            #output
            {
                #interior
                #body
            }
        }
    }
}
