//! Our macro expands to something vaguely like this:
//!
//! ```text
//! {
//!     
//!     let x = x.clone(), // for `mut clone x`
//!     let y = &mut y, // for `ref mut y`
//!     let z = &z, // for `mut ref z`
//!     let w = expr, // for `with w = expr`
//!
//!     |old_sig| { // Keep the old closure signature
//!         let _ = &b; // for `all b`
//!         old_body // old closure body
//!     }
//! }
//! ```

use proc_macro2::{Ident, Punct, Spacing, TokenStream};
use quote::{quote, quote_spanned, ToTokens};

use crate::parse::*;

pub struct Changes {
    pub exterior: TokenStream,
    pub interior: TokenStream,
    pub exempt: Vec<Ident>,
}

impl Changes {
    pub fn from_input(input: &Input, only: bool) -> Changes {
        let mut exempt = Vec::new();
        let mut ext = TokenStream::new();
        let mut int = TokenStream::new();

        for d in &input.assigned {
            let mu = &d.mu;
            let mut int_upvar = d.upvar.clone();
            if only {
                make_mixed!(int_upvar);
            }
            ext.extend(quote!(let #mu #int_upvar = ));
            match &d.ty {
                DirectiveType::Clone(sp) => {
                    let sp = *sp;
                    let ext_upvar = &d.upvar;
                    ext.extend(quote_spanned![sp=> ::core::clone::Clone::clone(&#ext_upvar)]);
                }
                DirectiveType::With(expr) => {
                    (&expr).to_tokens(&mut ext);
                }
                DirectiveType::Ref(sp, mu) => {
                    let mut ref_punc = Punct::new('&', Spacing::Alone);
                    ref_punc.set_span(*sp);
                    let ext_upvar = &d.upvar;
                    ext.extend(quote!(#ref_punc #mu #ext_upvar));
                }
            }
            ext.extend(quote!(;));
        }

        for d in &input.all {
            let upvar = &d.upvar;
            exempt.push(upvar.clone());
            int.extend(quote!(let _ = &#upvar;));
        }

        Changes {
            exterior: ext,
            interior: int,
            exempt,
        }
    }
}
