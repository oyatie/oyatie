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
        // A satisfaction the target's trait cannot take the METHODS of is not emitted, and the
        // reason is the pack's. See `unsatisfiable`.
        .filter(|observed| !unsatisfiable(observed, declaration, resolver))
        .map(|observed| build_impl(observed, declaration, &self_ty, resolver))
        .chain(message_impl(declaration, &self_ty, resolver).transpose())
        .chain(display_impl(declaration, &self_ty, resolver).transpose())
        .collect()
}

/// The source's STRINGER as the target's display trait.
///
/// `String() string` and `Display` are the same contract — one method, no arguments, renders the
/// receiver as text — and each language's printing facilities go through its own one. Emitting it
/// as an inherent `fn string(&self) -> String` keeps the method and loses the contract: the ported
/// type then cannot be printed, formatted or converted by anything generic, which is most of what a
/// caller wants it for.
///
/// The BODY is reused exactly as the failure interface's message method is, because the two are the
/// same shape — a body whose tail is the text — and the renderer already knows how to hand a
/// formatting call to the formatter instead of allocating a string to copy.
///
/// Not emitted when the type ALREADY has a display impl from the failure interface. Two impls of
/// one trait for one type is a coherence error, and a type that is both an error and a stringer has
/// one rendering, not two.
fn display_impl(
    declaration: &Declaration,
    self_ty: &RustType,
    resolver: &Resolver<'_>,
) -> Result<Option<RustItem>, TransformError> {
    let wanted = resolver.display_method_source;
    if wanted.is_empty() || message_impl(declaration, self_ty, resolver)?.is_some() {
        return Ok(None);
    }
    let Some(method) = declaration
        .children_of_kind(crate::vocabulary::CHILD_METHOD)
        .into_iter()
        .find(|method| method.name == wanted)
    else {
        return Ok(None);
    };
    // NO PARAMETERS and ONE RESULT. A method that merely shares the name is a different method:
    // the source's contract is nullary, and one taking an argument cannot be the trait's.
    if !method.children_of_kind(crate::vocabulary::CHILD_PARAM).is_empty()
        || method.children_of_kind(crate::vocabulary::CHILD_RESULT).len() != 1
    {
        return Ok(None);
    }
    let built = crate::signature::method_signature(
        method,
        resolver,
        port_engine_rust_ir::Visibility::Inherited,
        crate::signature::Body::Translate,
        &declaration.name,
        crate::body::ResultShape::Own,
    )?;
    // A body that did not translate leaves the method where it was rather than emitting an impl
    // with nothing in it.
    let Some(body) = built.body else {
        return Ok(None);
    };
    // AN EARLY RETURN cannot come along. The display method's body yields the TEXT, and the impl's
    // body must yield a formatting RESULT — so only the tail can be rewritten into a write, and a
    // `return` of a string somewhere in the middle would return that string from `fmt`. Such a
    // method stays inherent rather than being reshaped, because reshaping it means rewriting every
    // exit and that is a rule about control flow rather than about the trait.
    if body.iter().any(returns_early) {
        return Ok(None);
    }
    Ok(Some(RustItem::MessageImpl {
        docs: built.docs,
        self_ty: self_ty.clone(),
        body,
    }))
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
    // THE TRAIT'S POSITION, because an impl's signature must be the trait's. The parameters here
    // are the ones the trait declared, so they take the trait's form — an impl that spelled an
    // interface parameter `&impl Trait` while the trait spelled it `&dyn Trait` is not an impl of
    // that trait at all.
    let mut rendered = crate::signature::method_signature_at(
        observed,
        resolver,
        Visibility::Inherited,
        Body::None,
        &declaration.name,
        crate::body::ResultShape::Inherited,
        crate::vocabulary::POSITION_TRAIT_METHOD_PARAM,
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


/// Whether this observed satisfaction cannot be spelled as a target trait impl.
///
/// The FAILURE interface is the case, and the mapping that covers it is right everywhere else: as a
/// BOUND the source's error interface is the target's error trait. What does not carry over is the
/// method set. The source's interface is satisfied by one method returning the message; the target's
/// trait declares no such method and takes the message from its display trait instead. Emitting the
/// source's method into the impl produced `method `error` is not a member of trait `StdError``.
///
/// Recorded as a DROP rather than refusing the type, on the same reasoning as a dropped method: the
/// conformance is one thing the type has, not the whole of it. The method itself is not lost — it is
/// left unclaimed, so it stays in the inherent block under its own name — and the drop is reported
/// so a reader learns the conformance is absent.
pub(crate) fn unsatisfiable(
    observed: &Declaration,
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> bool {
    let Some(convention) = resolver.failure else {
        return false;
    };
    if observed.type_ref.name != convention.source_type || convention.satisfaction_reason.is_empty()
    {
        return false;
    }
    resolver.drops.record(crate::dropped::DroppedMethod {
        owner: declaration.name.clone(),
        name: format!("impl {}", observed.type_ref.name),
        reason: convention.satisfaction_reason.clone(),
    });
    true
}

/// The DISPLAY impl a type earns by satisfying the source's error interface.
///
/// The source satisfies that interface with one method returning the message. The target's error
/// trait declares no such method — it takes the message from the display trait — so the method
/// BECOMES that impl, and the error impl follows from it. What the type had in the source, it has
/// here: it can be printed, it can be boxed as an error, and `?` accepts it.
///
/// This is the other half of what `unsatisfiable` refuses. Refusing the satisfaction was right and
/// is still right — the two traits do not take the same methods — but leaving the method as an
/// inherent `error()` produced a type carrying a message nothing could reach, beside a real display
/// impl for a different type in the same file. A reviewer named that pair as the clearest evidence
/// the code had been converted rather than written.
///
/// `None` unless the type actually satisfies the interface. A method that merely shares the name is
/// the corpus's own, and rewriting it would answer for something the pack never spoke about.
fn message_impl(
    declaration: &Declaration,
    self_ty: &RustType,
    resolver: &Resolver<'_>,
) -> Result<Option<RustItem>, TransformError> {
    let Some(convention) = resolver.failure else {
        return Ok(None);
    };
    if convention.message_method_source.is_empty() {
        return Ok(None);
    }
    let satisfies = declaration
        .children_of_kind(CHILD_IMPLEMENTS)
        .into_iter()
        .any(|observed| observed.type_ref.name == convention.source_type);
    if !satisfies {
        return Ok(None);
    }
    let Some(method) = declaration
        .children_of_kind(crate::vocabulary::CHILD_METHOD)
        .into_iter()
        .find(|method| method.name == convention.message_method_source)
    else {
        return Ok(None);
    };
    let built = crate::signature::method_signature(
        method,
        resolver,
        port_engine_rust_ir::Visibility::Inherited,
        crate::signature::Body::Translate,
        &declaration.name,
        crate::body::ResultShape::Own,
    )?;
    let Some(body) = built.body else {
        return Ok(None);
    };
    Ok(Some(RustItem::MessageImpl {
        docs: built.docs,
        self_ty: self_ty.clone(),
        body,
    }))
}

/// Whether the failure interface's satisfaction will be emitted as a display impl, which CLAIMS the
/// method it is built from.
pub(crate) fn message_claimed(
    observed: &Declaration,
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> bool {
    resolver.failure.is_some_and(|convention| {
        observed.type_ref.name == convention.source_type
            && !convention.message_method_source.is_empty()
            && declaration
                .children_of_kind(crate::vocabulary::CHILD_METHOD)
                .into_iter()
                .any(|method| method.name == convention.message_method_source)
    })
}

/// The method a display impl CLAIMS from this declaration, when one will be emitted.
///
/// Asked by [`crate::signature::inherent_methods`] so the method is not emitted twice — once as an
/// inherent `fn string` and once as the trait's `fmt`. Both would compile, and the inherent one
/// wins path resolution, so the duplicate is invisible until someone deletes it.
pub(crate) fn display_claims(declaration: &Declaration, resolver: &Resolver<'_>) -> Option<String> {
    let self_ty = RustType::path(to_pascal_case(&declaration.name));
    match display_impl(declaration, &self_ty, resolver) {
        Ok(Some(_)) => Some(resolver.display_method_source.to_owned()),
        _ => None,
    }
}

/// Whether this statement, or anything inside it, RETURNS.
fn returns_early(statement: &port_engine_rust_ir::RustStmt) -> bool {
    use port_engine_rust_ir::RustStmt;
    match statement {
        RustStmt::Return(_) => true,
        RustStmt::While { body, .. } | RustStmt::Loop(body) | RustStmt::ForIn { body, .. } | RustStmt::Block(body) => {
            body.iter().any(returns_early)
        }
        RustStmt::Semi(expr) | RustStmt::Tail(expr) => returns_in_expression(expr),
        _ => false,
    }
}

/// Whether this expression contains a `return`, which only the block-like forms can.
fn returns_in_expression(expr: &port_engine_rust_ir::RustExpr) -> bool {
    use port_engine_rust_ir::RustExpr;
    match expr {
        RustExpr::Block(body) => body.iter().any(returns_early),
        RustExpr::If {
            then, otherwise, ..
        } => {
            then.iter().any(returns_early)
                || otherwise.as_deref().is_some_and(returns_in_expression)
        }
        RustExpr::Match { arms, .. } => arms.iter().any(|arm| arm.body.iter().any(returns_early)),
        _ => false,
    }
}
