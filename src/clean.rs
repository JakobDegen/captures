use std::collections::HashSet;

use proc_macro2::{Group, Ident, Span, TokenStream, TokenTree};
use syn::visit_mut::{self, VisitMut};
use syn::Expr;

/// Replaces hygiene information in `Expr`, making all locals `mixed_site` except those in the
/// exempt list.
///
/// This respects shadowing.
pub fn clean(expr: &mut Expr, exempt: &[Ident]) {
    let mut state = CleaningState {
        exempt: HashSet::from_iter(exempt.into_iter().cloned()),
        shadowed: Vec::new(),
    };

    state.visit_expr_mut(expr);
}

/// Stores the state for changing hygiene information.
///
/// The `exempt` list contains the list of idents that are *currently* exempt from being cleaned.
/// This does not include those idents which are normally exempt but currently shadowed. The
/// shadowed idents are stored in the `shadowed` stack, and are popped off when their scope ends.
///
/// The reason we don't clean shadowed idents is to try and improve interactions with macros called
/// inside the closure; this way all variables that are local within the closure have `mixed_site`
/// hygiene.
struct CleaningState {
    exempt: HashSet<Ident>,
    shadowed: Vec<Ident>,
}

impl CleaningState {
    fn pop(&mut self, len: usize) {
        self.exempt.extend(self.shadowed.drain(len..));
    }
}

macro_rules! wrap_visitors {
    [$($name:ident , $t:ty);*] => {
        $(
            fn $name (&mut self, node: &mut $t) {
                let len = self.shadowed.len();
                visit_mut::$name(self, node);
                self.pop(len);
            }
        )*
    }
}

fn make_stream_mixed(s: TokenStream) -> TokenStream {
    s.into_iter()
        .map(|tt| match tt {
            TokenTree::Group(g) => TokenTree::Group({
                let mut out = Group::new(g.delimiter(), make_stream_mixed(g.stream()));
                out.set_span(g.span().resolved_at(Span::mixed_site()));
                out
            }),
            TokenTree::Ident(mut i) => TokenTree::Ident({
                make_mixed!(i);
                i
            }),
            TokenTree::Punct(mut p) => TokenTree::Punct({
                make_mixed!(p);
                p
            }),
            TokenTree::Literal(mut l) => TokenTree::Literal({
                make_mixed!(l);
                l
            }),
        })
        .collect()
}

impl VisitMut for CleaningState {
    fn visit_pat_ident_mut(&mut self, node: &mut syn::PatIdent) {
        visit_mut::visit_pat_ident_mut(self, node);
        // This is the only place new idents are introduced. Shadowed exempt idents remain shadowed
        // until the end of the current scope.
        if let Some(ident) = self.exempt.take(&node.ident) {
            self.shadowed.push(ident);
        }
        make_mixed!(node.ident);
    }

    fn visit_path_segment_mut(&mut self, node: &mut syn::PathSegment) {
        // Path segments are the only places local variables can be used. Setting all path segments
        // to `mixed_site` is fine, since we don't emit `$crate`
        // This also covers macro names in function like macros, which we want to be `mixed_site`
        if !self.exempt.contains(&node.ident) {
            make_mixed!(node.ident);
        }
    }

    // Need to re-order here, recursing on the RHS before the pattern, since that is the order in
    // which things arrive into scope. The default ordering on match arms is right, so no need to
    // do so there as well.
    fn visit_expr_let_mut(&mut self, node: &mut syn::ExprLet) {
        for att in &mut node.attrs {
            self.visit_attribute_mut(att);
        }
        self.visit_expr_mut(&mut node.expr);
        self.visit_pat_mut(&mut node.pat);
    }

    fn visit_local_mut(&mut self, node: &mut syn::Local) {
        for att in &mut node.attrs {
            self.visit_attribute_mut(att);
        }
        if let Some((_, expr)) = &mut node.init {
            self.visit_expr_mut(expr);
        }
        self.visit_pat_mut(&mut node.pat);
    }

    // We make sure all tokens passed to macros are `mixed_site`
    // FIXME: this is not strictly correct, but is the best possible approximation we can get
    // without eager macro expansion
    fn visit_macro_mut(&mut self, node: &mut syn::Macro) {
        visit_mut::visit_macro_mut(self, node);
        let s = std::mem::take(&mut node.tokens);
        node.tokens = make_stream_mixed(s);
    }

    fn visit_attribute_mut(&mut self, node: &mut syn::Attribute) {
        visit_mut::visit_attribute_mut(self, node);
        let s = std::mem::take(&mut node.tokens);
        node.tokens = make_stream_mixed(s);
    }

    // Cant just `wrap_visitors!` for `ExprIf`, since the `else` block is excluded
    fn visit_expr_if_mut(&mut self, node: &mut syn::ExprIf) {
        let len = self.shadowed.len();
        for att in &mut node.attrs {
            self.visit_attribute_mut(att);
        }
        self.visit_expr_mut(&mut node.cond);
        self.visit_block_mut(&mut node.then_branch);
        self.pop(len);
        if let Some((_, expr)) = &mut node.else_branch {
            self.visit_expr_mut(expr);
        }
    }

    wrap_visitors!(
        visit_block_mut, syn::Block;
        visit_expr_for_loop_mut, syn::ExprForLoop;
        visit_expr_while_mut, syn::ExprWhile;
        visit_arm_mut, syn::Arm
    );
}
