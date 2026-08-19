//! `impl Trait for Type`, from OBSERVED satisfaction rather than from structural matching.
//!
//! Source interfaces here are implicit: nothing in a type's declaration says which interfaces it
//! satisfies. `docs/programs/k8s-port/census/interfaces.md` measured what the two emission
//! strategies cost — 80,042 name-level structural matches against 1,316 pairs the source declares
//! outright — and concluded that the engine must emit from USAGE. The front end does the
//! observing; this module turns each observation into an item.
//!
//! Each method DELEGATES to the inherent method of the same name rather than carrying the body
//! itself. A body can live in exactly one place, and a type satisfying two interfaces that share a
//! method name would otherwise need it in both — so the inherent `impl` block stays the one home
//! for a translated body and the trait impls are bridges to it. The call is spelled as a path
//! (`Label::name(self)`) rather than as a method call (`self.name()`), because inside a trait impl
//! the method call resolves against the trait first and would recurse into itself.
//!
//! THE ORPHAN RULE DOES NOT BITE HERE, and it is worth recording why rather than adding a check
//! that cannot fire. Rust forbids implementing a foreign trait for a foreign type; the engine
//! emits every unit of one corpus as a MODULE of one crate, so both sides of every pair are local
//! by construction. It becomes reachable when a trait or a type crosses a crate boundary — the
//! `go-rt` runtime, or a corpus split across crates — and the census's 6 foreign-on-foreign
//! assertions are the population that will need the newtype treatment then.

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustExpr, RustFn, RustItem, RustStmt, RustType, Visibility};

use port_engine_api::DocConvention;

use crate::docs::docs_of;
use crate::error::TransformError;
use crate::naming::to_pascal_case;
use crate::resolve::Resolver;
use crate::signature::{Body, method_receiver, method_signature};
use crate::vocabulary::{ATTR_SITE, CHILD_IMPLEMENTS, CHILD_METHOD, POSITION_TRAIT};

/// Every trait impl a declaration's observed satisfactions call for.
///
/// # Errors
/// [`TransformError::UnmappedType`] when the trait's type does not resolve, and whatever the
/// signature layer refuses for a method the target cannot express.
pub(crate) fn trait_impls(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<Vec<RustItem>, TransformError> {
    let self_ty = RustType::path(to_pascal_case(&declaration.name));
    declaration
        .children_of_kind(CHILD_IMPLEMENTS)
        .into_iter()
        .map(|observed| build_impl(observed, declaration, &self_ty, resolver))
        .collect()
}

fn build_impl(
    observed: &Declaration,
    declaration: &Declaration,
    self_ty: &RustType,
    resolver: &Resolver<'_>,
) -> Result<RustItem, TransformError> {
    let trait_path = resolver.resolve_in(&observed.type_ref, &declaration.name, POSITION_TRAIT)?;
    let methods = observed
        .children_of_kind(CHILD_METHOD)
        .into_iter()
        .map(|method| delegating_method(method, declaration, resolver))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(RustItem::TraitImpl {
        docs: satisfaction_docs(observed, &trait_path, resolver.doc_convention),
        trait_path,
        self_ty: self_ty.clone(),
        methods,
    })
}

/// How the satisfaction was observed, carried into the emitted crate.
///
/// A declared assertion is checked by the source compiler; a flow-derived one is the front end's
/// inference from one use site. The two produce identical Rust, so a reader who needs to know
/// which they are looking at can only be told — and the emitted code is where they are reading.
fn satisfaction_docs(
    observed: &Declaration,
    trait_path: &RustType,
    convention: &DocConvention,
) -> Vec<String> {
    let mut docs = docs_of(observed, convention);
    let site = observed.attr(ATTR_SITE).unwrap_or("an unrecorded position");
    docs.push(format!(
        " Ported from an implicit interface: the source was observed satisfying `{}` at {site}.",
        trait_path.spelling()
    ));
    docs
}

/// One trait method, delegating to the inherent method of the same name.
fn delegating_method(
    method: &Declaration,
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<RustFn, TransformError> {
    let mut rendered = method_signature(
        method,
        resolver,
        Visibility::Inherited,
        Body::None,
        &declaration.name,
    )?;
    rendered.receiver = Some(method_receiver(method, resolver, &declaration.name)?);

    // The receiver argument is `self` in every form. What differs is what `self` IS, and the
    // signature above already fixed that — an exclusive receiver reborrows down to a shared one
    // where the inherent method wants less, which is the target's rule and not a decision here.
    let mut args = Vec::with_capacity(rendered.params.len() + 1);
    args.push(RustExpr::SelfValue);
    args.extend(
        rendered
            .params
            .iter()
            .map(|param| RustExpr::Path(param.name.clone())),
    );

    let call = RustExpr::Call {
        callee: Box::new(RustExpr::Path(format!(
            "{}::{}",
            to_pascal_case(&declaration.name),
            rendered.name
        ))),
        args,
    };
    // A method that returns nothing delegates as a STATEMENT. The tail form yields the same unit
    // value and reads as though the call were the answer, which is a claim about a method that has
    // none.
    rendered.body = Some(vec![if rendered.ret.is_some() {
        RustStmt::Tail(call)
    } else {
        RustStmt::Semi(call)
    }]);
    Ok(rendered)
}
