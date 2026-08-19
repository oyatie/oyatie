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
use crate::vocabulary::{
    ATTR_BUNDLE, ATTR_SITE, CHILD_IMPLEMENTS, CHILD_METHOD, CHILD_PROMOTED, POSITION_TRAIT,
};

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
        // A pure SUPERTRAIT BUNDLE is implemented once, for everything that qualifies, where the
        // trait is declared. The source satisfies such an interface structurally, so a per-type
        // impl says something weaker AND conflicts with the blanket one — which is not a
        // redundancy the target tolerates, it is a coherence error.
        .filter(|observed| observed.attr(ATTR_BUNDLE) != Some("true"))
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
        .map(|method| implementing_method(method, declaration, resolver))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(RustItem::TraitImpl {
        docs: satisfaction_docs(observed, resolver)?,
        trait_path,
        self_ty: self_ty.clone(),
        methods,
    })
}

/// The impl's documentation, which is the SOURCE's and nothing else.
///
/// This used to append "Ported from an implicit interface: the source was observed satisfying `X`
/// at <site>." A reviewer reading the emitted crate found that sentence in the public rustdoc and
/// named it as a translator's working note shipped as API documentation — and they were right. A
/// doc comment is what a CALLER reads; how the engine came to emit an impl is not something a
/// caller can act on, and it tells them the crate was generated, which is the one thing this engine
/// is trying not to say.
///
/// The provenance is not lost. Which satisfactions were observed and where is exactly what the plan
/// and the receipt record, and that is where provenance belongs: the emitted crate is the PRODUCT,
/// not the record of how it was made.
fn satisfaction_docs(
    observed: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<Vec<String>, TransformError> {
    docs_of(observed, resolver)
}

/// One trait method, carrying the BODY the type would otherwise have put in an inherent block.
///
/// The pair this replaces — an inherent `describe` beside a trait `describe` that forwards to it —
/// compiles only because an inherent method wins path resolution, and deleting the inherent one
/// turns the forward into infinite recursion. A stack overflow introduced by REMOVING code.
///
/// SIGNATURE from the trait's method and BODY from the type's own, because they answer different
/// questions: the trait fixes one receiver for every implementor, and the body is what this
/// implementor does. A body written under `&self` typechecks under `&mut self`, which is the
/// direction the union can move it.
///
/// A PROMOTED method has no body of its own — what it does is forward to the embedded field — so
/// its forwarding body is built here directly rather than delegated to an inherent twin.
///
/// # Errors
/// [`TransformError`] from the signature layer, or when the type has neither an own method of that
/// name nor a promoted one, which would mean the satisfaction names a method nothing provides.
fn implementing_method(
    observed: &Declaration,
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<RustFn, TransformError> {
    let mut rendered = method_signature(
        observed,
        resolver,
        Visibility::Inherited,
        Body::None,
        &declaration.name,
        crate::body::ResultShape::Inherited,
    )?;
    rendered.receiver = Some(method_receiver(observed, resolver, &declaration.name)?);

    if let Some(own) = declaration
        .children_of_kind(CHILD_METHOD)
        .into_iter()
        .find(|method| method.name == observed.name)
    {
        let translated = method_signature(
            own,
            resolver,
            Visibility::Inherited,
            Body::Translate,
            &declaration.name,
            crate::body::ResultShape::Inherited,
        )?;
        rendered.body = translated.body;
        return Ok(rendered);
    }

    let promoted = declaration
        .children_of_kind(CHILD_PROMOTED)
        .into_iter()
        .find(|method| method.name == observed.name)
        .ok_or_else(|| TransformError::MissingDatum {
            construction: "trait impl".to_owned(),
            name: observed.name.clone(),
            datum: "method body",
        })?;
    rendered.body = crate::promote::forwarding_body(promoted, declaration, resolver)?;
    Ok(rendered)
}

