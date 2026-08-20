//! Parameters, results, and method signatures — as IR nodes.

use std::collections::BTreeSet;

use port_engine_api::Declaration;
use port_engine_rust_ir::{Receiver, RustFn, RustParam, RustType, Visibility};

use crate::error::TransformError;
use crate::naming::{to_snake_case, visibility};
use crate::ownership::{binds_by_pointer, facts_of, parameter_target, receiver_for};
use crate::params::params;
use crate::results::{results, results_owned};
use crate::resolve::Resolver;
use crate::vocabulary::{
    ATTR_RECEIVER, CHILD_BODY, CHILD_IMPLEMENTS, CHILD_METHOD, CHILD_PARAM, CHILD_RESULT, FLAG_VARIADIC, POSITION_PARAM, POSITION_RESULT, POSITION_TRAIT_METHOD_PARAM,
};

/// Whether a method's body is translated, stubbed, or absent.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Body {
    /// Emit `todo!()`.
    Stub,
    /// Translate the source body, refusing anything outside the supported subset.
    Translate,
    /// Emit no body at all — a trait item is a signature.
    None,
}

/// The methods a type carries in its inherent `impl` block.
///
/// Declared methods and PROMOTED ones, in one block, because the target draws no distinction
/// between them: a caller of the source cannot tell whether a method was declared on the type or
/// lifted from an embedded field, and the emit should not make them tell either.
pub(crate) fn inherent_methods(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
    body: Body,
) -> Result<Vec<RustFn>, TransformError> {
    // A method the source's own interfaces claim is emitted in the TRAIT IMPL, not here. Both
    // would be an inherent method shadowing a trait method of the same name: it compiles because
    // inherent wins path resolution, and deleting the inherent one turns the trait impl's forward
    // into infinite recursion — a stack overflow introduced by removing code.
    let mut claimed = methods_claimed_by_traits(declaration, resolver);
    // THE DISPLAY METHOD too, when one will be emitted. See `impls::display_claims`.
    claimed.extend(crate::impls::display_claims(declaration, resolver));
    let mut methods = Vec::new();
    for method in declaration
        .children_of_kind(CHILD_METHOD)
        .into_iter()
        .filter(|method| !claimed.contains(&method.name))
    {
        match method_signature(
            method, resolver, Visibility::Public, body, &declaration.name,
            crate::body::ResultShape::Own,
        ) {
            Ok(built) => methods.push(built),
            Err(error) => resolver.drops.record(crate::dropped::DroppedMethod {
                owner: declaration.name.clone(),
                name: method.name.clone(),
                reason: crate::survey_cause::refusal_of(&error),
            }),
        }
    }
    methods.extend(crate::promote::promoted_methods(declaration, resolver, &claimed)?);
    methods.extend(emptiness_companion(&methods));
    Ok(methods)
}

/// Parse a pack-declared receiver form into the IR's receiver.
fn parse_receiver(form: &str, site: &str) -> Result<Receiver, TransformError> {
    match form {
        "&self" => Ok(Receiver::Shared),
        "&mut self" => Ok(Receiver::Exclusive),
        "self" => Ok(Receiver::Owned),
        other => Err(TransformError::Ownership {
            detail: format!("`{other}` is not a receiver form the target has, at `{site}`"),
        }),
    }
}

/// The methods a trait requires, as bodiless signatures.
///
/// Visibility is [`Visibility::Inherited`] and not a choice: a trait item is as public as its
/// trait and may not say so. The previous renderer concatenated the source's `pub ` prefix into
/// the trait body, which `syn` parses and `rustc` rejects — and which a golden over the parsed
/// output would have frozen in place.
///
/// The RECEIVER is a choice, and the pack makes it. It cannot be recovered from the source: an
/// interface says nothing about how an implementation binds its receiver, and the implementations
/// are not all in view. `&self` was being emitted for every method, which is why the fixture's
/// `Rename(next string)` — a mutator — became a signature no mutating implementation can satisfy.
pub(crate) fn trait_methods(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<Vec<RustFn>, TransformError> {
    declaration
        .children_of_kind(CHILD_METHOD)
        .into_iter()
        .map(|method| {
            let mut rendered = method_signature_at(
                method,
                resolver,
                Visibility::Inherited,
                Body::None,
                &declaration.name,
                crate::body::ResultShape::Own,
                POSITION_TRAIT_METHOD_PARAM,
            )?;
            rendered.receiver = Some(method_receiver(method, resolver, &declaration.name)?);
            Ok(rendered)
        })
        .collect()
}

/// The receiver one trait method binds.
///
/// The OBSERVED mode wins. A source interface says nothing about how an implementation binds its
/// receiver, so P1 made this a declared pack decision — one mode for every trait method, which put
/// `&mut self` on getters. With the implementors observed, the mode is derived per method:
/// exclusive exactly when some implementor mutates through it. The pack's decision is the fallback
/// and now covers only the interfaces nothing was seen to implement.
///
/// # Errors
/// [`TransformError::Unsupported`] when neither answers, because emitting a guessed receiver is
/// what this whole path exists to stop.
pub(crate) fn method_receiver(
    method: &Declaration,
    resolver: &Resolver<'_>,
    owner: &str,
) -> Result<Receiver, TransformError> {
    if let Some(observed) = method.attr(ATTR_RECEIVER) {
        return declared_receiver(observed, owner);
    }
    let (mode, _reason) = resolver
        .trait_receiver()
        .ok_or_else(|| TransformError::Unsupported {
            name: owner.to_owned(),
            detail: "no implementor of this interface was observed and the pack declares no \
                     trait-receiver mode, so the receiver would be a guess"
                .to_owned(),
        })?;
    declared_receiver(mode, owner)
}

/// Map the pack's declared mode onto the IR's receiver.
pub(crate) fn declared_receiver(mode: &str, owner: &str) -> Result<Receiver, TransformError> {
    match mode {
        "shared" => Ok(Receiver::Shared),
        "exclusive" => Ok(Receiver::Exclusive),
        "owned" => Ok(Receiver::Owned),
        other => Err(TransformError::Unsupported {
            name: owner.to_owned(),
            detail: format!("`{other}` is not a receiver mode the target has"),
        }),
    }
}

pub(crate) fn method_signature(
    method: &Declaration,
    resolver: &Resolver<'_>,
    vis: Visibility,
    body: Body,
    owner: &str,
    result: crate::body::ResultShape,
) -> Result<RustFn, TransformError> {
    method_signature_at(method, resolver, vis, body, owner, result, POSITION_PARAM)
}

/// The same, with the position its PARAMETERS occupy stated.
///
/// A trait's declared method and every impl of it must agree, so both pass the trait-method
/// position and an inherent method passes the ordinary one. Threaded rather than inferred from the
/// body's presence: an impl method HAS a body and still needs the trait's answer, and inferring it
/// from what happened to be in hand is how a signature and its implementation come to disagree.
#[allow(clippy::too_many_arguments)]
pub(crate) fn method_signature_at(
    method: &Declaration,
    resolver: &Resolver<'_>,
    vis: Visibility,
    body: Body,
    owner: &str,
    result: crate::body::ResultShape,
    position: &str,
) -> Result<RustFn, TransformError> {

    // A pointer receiver used to be refused outright, because `&self` drops the mutation it
    // permits and `&mut self` claims one the source may not perform, and nothing reported which.
    // The front end now reports it, so the guess became a decision — made by the pack over
    // observed facts, and recorded per site.
    //
    // A VALUE receiver is not an aliasing question — the source already copied, so there are no
    // aliases to reason about — but it is not `self` either. Go's value receiver COPIES and the
    // caller's value survives the call; Rust's `self` CONSUMES, so calling the method twice would
    // stop compiling. `&self` is the form with the source's permissions.
    //
    // A value receiver the body mutates is a different shape again: the mutation is on the copy
    // and needs a local binding, which is body-translation work rather than a receiver form. It
    // refuses instead of silently picking one of the two wrong answers.
    let receiver = if binds_by_pointer(method) {
        let site = format!("{owner}::{}", method.name);
        parse_receiver(&receiver_for(method, &site, resolver.ownership)?, &site)?
    } else if facts_of(method).mutated {
        return Err(TransformError::Unsupported {
            name: method.name.clone(),
            detail: "value receiver mutated in the body: the source mutates its own COPY, which \
                     needs a local binding rather than a receiver form — `self` would consume the \
                     caller's value and `&mut self` would claim a mutation the caller can see"
                .to_owned(),
        });
    } else {
        Receiver::Shared
    };

    // The BODY first, because the signature is built from what it did: a parameter the body folded
    // away needs no `mut`, and that is an outcome rather than a prediction.
    let mut consumed = std::collections::BTreeSet::new();
    let statements = match body {
        // A rung that does not translate bodies REFUSES the method it cannot write. It used to emit
        // a body that panics, which compiles, passes every gate that reads the output as Rust, and
        // turns an untranslated method into a runtime abort at the caller — the one failure this
        // engine exists to prevent, dressed as success.
        Body::Stub => {
            return Err(TransformError::Unsupported {
                name: method.name.clone(),
                detail: format!(
                    "`{owner}` is captured by a rule that does not translate method bodies, and \
                     `{}` has one; emitting the method without it would compile and panic where \
                     the source computed something",
                    method.name
                ),
            });
        }
        Body::Translate => {
            let source = method
                .children_of_kind(CHILD_BODY)
                .first()
                .copied()
                .ok_or_else(|| TransformError::MissingDatum {
                    construction: "rust_struct_body".to_owned(),
                    name: method.name.clone(),
                    datum: "body",
                })?;
            let (translated, folded) =
                crate::body::statements(&source.children, method, resolver, result, Some(owner))?;
            consumed = folded;
            Some(translated)
        }
        Body::None => None,
    };

    Ok(RustFn {
        // A method's documentation is emitted, like every other declaration's. It was being
        // dropped here while the front end captured it — the same silent loss as the interface
        // methods, and invisible for the same reason: nothing looks for prose that is absent.
        docs: crate::docs::docs_of(method, resolver)?,
        vis,
        name: to_snake_case(&method.name),
        receiver: Some(receiver),
        params: crate::params::params_at(method, resolver, owner, &consumed, position)?,
        ret: match result {
            crate::body::ResultShape::Own => results(method, resolver)?,
            // The trait fixed it, and this call exists only for the body it produces.
            crate::body::ResultShape::Inherited => results_owned(method, resolver)?,
        },
        attrs: crate::params::inline_attrs(statements.as_deref(), vis, resolver),
        body: statements,
    })
}

/// Visibility for a declaration, as a value the IR places rather than a prefix a string carries.
pub(crate) fn declared_visibility(declaration: &Declaration) -> Visibility {
    visibility(declaration)
}

/// The method names some observed interface satisfaction will carry into a trait impl.
///
/// Those are emitted THERE and not in the inherent block, because a type carrying both an inherent
/// `describe` and a trait `describe` is the shadowing footgun above. A method that satisfies no
/// interface keeps its inherent impl, because there is no trait to put it in.
fn methods_claimed_by_traits(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> BTreeSet<String> {
    declaration
        .children_of_kind(CHILD_IMPLEMENTS)
        .into_iter()
        // A satisfaction that will NOT be emitted claims nothing. Claiming it would delete the
        // method from the inherent block on the strength of a trait impl that never appears, which
        // loses the method entirely — the silent hole this engine exists to avoid.
        //
        // The FAILURE interface is the exception, and it is not one: its satisfaction is emitted,
        // as a display impl built from this very method. So the method IS claimed — by that impl
        // rather than by a trait impl of the source's interface. See `impls::message_impl`.
        .filter(|observed| {
            !crate::impls::unsatisfiable(observed, declaration, resolver)
                || crate::impls::message_claimed(observed, declaration, resolver)
        })
        .flat_map(|observed| observed.children_of_kind(CHILD_METHOD))
        .map(|method| method.name.clone())
        .collect()
}

/// `is_empty`, beside a public `len` that has one.
///
/// DERIVED, not invented: `is_empty` is `len() == 0` and nothing else, so this adds no meaning the
/// source did not already have — it adds the SPELLING every Rust caller reaches for first. The
/// target's own lint requires it (`clippy::len_without_is_empty`), and under the deny-warnings
/// policy this engine is held to that makes a type with a public `len` and no `is_empty` a crate
/// that does not build.
///
/// The source has no such convention, which is exactly why nothing carries it across. Go's
/// `len(c)` is a builtin over the value; the target's is a method on the type, and a method brings
/// the type's obligations with it.
fn emptiness_companion(methods: &[RustFn]) -> Option<RustFn> {
    let length = methods.iter().find(|method| {
        method.name == "len"
            && method.vis == Visibility::Public
            && method.params.is_empty()
            && method.receiver.is_some()
    })?;
    // ALREADY THERE — a source that declares its own emptiness test keeps it, and a second one
    // would be a duplicate definition rather than a convenience.
    if methods.iter().any(|method| method.name == "is_empty") {
        return None;
    }
    Some(RustFn {
        docs: vec![" Whether the collection contains no elements.".to_owned()],
        vis: Visibility::Public,
        name: "is_empty".to_owned(),
        receiver: length.receiver.clone(),
        params: Vec::new(),
        ret: Some(port_engine_rust_ir::RustType::path("bool")),
        attrs: Vec::new(),
        body: Some(vec![port_engine_rust_ir::RustStmt::Tail(
            port_engine_rust_ir::RustExpr::Binary {
                op: port_engine_rust_ir::BinaryOp::Eq,
                lhs: Box::new(port_engine_rust_ir::RustExpr::MethodCall {
                    receiver: Box::new(port_engine_rust_ir::RustExpr::SelfValue),
                    method: "len".to_owned(),
                    args: Vec::new(),
                }),
                rhs: Box::new(port_engine_rust_ir::RustExpr::Literal("0".to_owned())),
            },
        )]),
    })
}
