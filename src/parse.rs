use std::collections::HashSet;

use proc_macro2::{Ident, Span};
use syn::{
    parse::{Parse, ParseStream},
    Error, Expr, ExprClosure, Token,
};

/// Represents the entire parsed input to the macro
pub struct Input {
    pub assigned: Vec<AssignedDirective>,
    pub all: Vec<AllDirective>,
    pub closure: ExprClosure,
}

enum Directive {
    All(AllDirective),
    Assigned(AssignedDirective),
}

pub struct AllDirective {
    pub upvar: Ident,
}

pub struct AssignedDirective {
    /// `x` in `clone x`
    pub upvar: Ident,
    pub mu: Option<Token![mut]>,
    pub ty: DirectiveType,
}
pub enum DirectiveType {
    Ref(Span, Option<Token![mut]>),
    Clone(Span),
    With(Box<Expr>),
}

const EXPECTED_MSG: &'static str = "expected `ref`, `clone`, `with`, or `all`";

impl Parse for Directive {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![ref]) {
            let ref_span = input.parse::<Token![ref]>().unwrap().span;
            let sec_mu = input.parse::<Option<Token![mut]>>().unwrap();
            Ok(Directive::Assigned(AssignedDirective {
                upvar: input.parse::<syn::Ident>()?,
                mu: None,
                ty: DirectiveType::Ref(ref_span, sec_mu),
            }))
        } else if input.peek(syn::Ident) {
            let next = input.parse::<Ident>().unwrap();
            let mu = input.parse::<Option<Token![mut]>>().unwrap();
            match &*next.to_string() {
                "clone" => Ok(Directive::Assigned(AssignedDirective {
                    upvar: input.parse::<syn::Ident>()?,
                    mu,
                    ty: DirectiveType::Clone(next.span()),
                })),
                "with" => {
                    let upvar = input.parse::<syn::Ident>()?;
                    input.parse::<Token![=]>()?;
                    Ok(Directive::Assigned(AssignedDirective {
                        upvar,
                        mu,
                        ty: DirectiveType::With(Box::new(input.parse::<Expr>()?)),
                    }))
                }
                "all" => {
                    if let Some(mu) = mu {
                        Err(syn::Error::new(
                            mu.span,
                            "may not use mutability specifier with `all` directive",
                        ))
                    } else {
                        Ok(Directive::All(AllDirective {
                            upvar: input.parse::<syn::Ident>()?,
                        }))
                    }
                }
                _ => Err(syn::Error::new(next.span(), EXPECTED_MSG)),
            }
        } else {
            Err(input.error(EXPECTED_MSG))
        }
    }
}

/// Consumes token trees in the input up to and including the next comma.
fn skip_past_comma(input: ParseStream) {
    input
        .step(|cursor| {
            let mut rest = *cursor;
            while let Some((tt, next)) = rest.token_tree() {
                if let proc_macro2::TokenTree::Punct(p) = tt {
                    if p.as_char() == ',' {
                        return Ok(((), next));
                    }
                }
                rest = next;
            }
            Ok(((), rest))
        })
        .unwrap();
}

fn combine(opt: &mut Option<Error>, e: Error) {
    match opt {
        Some(err) => err.combine(e),
        None => *opt = Some(e),
    }
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut assigned = Vec::new();
        let mut all = Vec::new();
        let mut found = HashSet::new();
        let mut needs_move = false;
        // If we encounter an error while parsing, store it here. We'll continue parsing to be able
        // to emit as many errors as possible.
        let mut err: Option<syn::Error> = None;
        // Figure out if we should be parsing a further directive or the closure
        while !input.is_empty()
            && !{
                (input.peek(Token![#])
                    || input.peek(Token![async])
                    || input.peek(Token![static])
                    || input.peek(Token![|]))
                    || (input.peek(Token![move]) && input.peek2(Token![|]))
            }
        {
            let id = match input.parse::<Directive>() {
                Ok(Directive::All(dir)) => {
                    let id = dir.upvar.clone();
                    all.push(dir);
                    id
                }
                Ok(Directive::Assigned(dir)) => {
                    needs_move |=
                        matches!(&dir.ty, DirectiveType::Clone(_) | DirectiveType::With(_));
                    let id = dir.upvar.clone();
                    assigned.push(dir);
                    id
                }
                Err(e) => {
                    combine(&mut err, e);
                    // FIXME: This is slightly wrong, in particular, commas can appear in top level
                    // token trees in a `CaptureDirective` if that directive is a `with` directive
                    // having on the right hand side a closure expression taking multiple arguments.
                    // All other commas appear in sub-streams (as far as I can tell).
                    skip_past_comma(input);
                    continue;
                }
            };
            if found.contains(&id) {
                combine(
                    &mut err,
                    Error::new(
                        id.span(),
                        format!("cannot supply multiple directives for `{}`", id),
                    ),
                );
            } else {
                found.insert(id);
            }
            if let Err(e) = input.parse::<Token![,]>() {
                combine(&mut err, e);
            }
        }

        let mut closure = input.parse::<syn::ExprClosure>().map_err(|e| {
            combine(&mut err, e);
            err.take().unwrap()
        })?;
        if needs_move && closure.capture.is_none() {
            closure.capture = Some(Default::default());
        }
        if !closure.capture.is_some() {
            for dir in assigned.iter() {
                match &dir.ty {
                    DirectiveType::Ref(sp, _) => combine(
                        &mut err,
                        Error::new(
                            *sp,
                            format!("`ref` directives only allowed on `move` closures"),
                        ),
                    ),
                    _ => panic!("Bug: Somehow not `needs_move`"),
                }
            }
        }

        let attrs = std::mem::take(&mut closure.attrs);
        if !attrs.is_empty() {
            let add_err = Error::new_spanned(
                &attrs[0],
                "attributes are not allowed on the closure inside a `captures!`",
            );
            combine(&mut err, add_err);
        }
        if !input.is_empty() {
            let add_err = input.error("expected macro input to end");
            combine(&mut err, add_err);
        }
        if let Some(err) = err {
            Err(err)
        } else {
            Ok(Input {
                all,
                assigned,
                closure,
            })
        }
    }
}
