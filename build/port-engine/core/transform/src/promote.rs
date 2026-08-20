//! Methods a type gains through EMBEDDING rather than declaration.
//!
//! Go promotes: an anonymous field lifts the embedded type's methods into the outer type's method
//! set, and nothing forwards them. The target has no such rule, so what is implicit in the source
//! becomes an explicit forwarding method here.
//!
//! This is what closes `census/interfaces.md` §11 item 7. That census recorded a method only where
//! a declaration carried a receiver, so 2,747 CORE struct types have method sets larger than it
//! measured and 479 of them appear to have no methods at all — `runtime.codec` is
//! `type codec struct { Encoder; Decoder }` with zero declared methods, returned as a `Codec`. The
//! census names go/types as what would close it; the front end has go/types, so the promoted set
//! arrives as a fact and this module spends it.
//!
//! The RECEIVER is not a new decision. A forwarding method has no body of its own to observe, and
//! what it may do is decided entirely by the method it forwards to — so the front end carries the
//! embedded method's own ownership facts on the promoted node, and the same rules that decide a
//! declared method's receiver decide this one.

use std::collections::BTreeSet;

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustExpr, RustFn, RustStmt, Visibility};

use crate::error::TransformError;
use crate::naming::to_snake_case;
use crate::resolve::Resolver;
use crate::signature::{Body, method_signature};
use crate::vocabulary::{ATTR_VIA, CHILD_PROMOTED};

/// Every method the declaration gains through embedding, as a forwarding method.
///
/// # Errors
/// [`TransformError::MissingDatum`] when a promoted method carries no field path, and whatever the
/// signature layer refuses for a method the target cannot express.
pub(crate) fn promoted_methods(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
    claimed: &BTreeSet<String>,
) -> Result<Vec<RustFn>, TransformError> {
    declaration
        .children_of_kind(CHILD_PROMOTED)
        .into_iter()
        // A promoted method a trait claims is emitted in the TRAIT IMPL, which builds the same
        // forward. Emitting both would be the inherent-beside-trait pair this engine no longer
        // produces, and a promoted method is the one shape where the two bodies are identical —
        // which makes the shadowing easier to miss, not harder.
        .filter(|promoted| !claimed.contains(&promoted.name))
        .map(|promoted| forwarding_method(promoted, declaration, resolver))
        .collect()
}

/// The body a promoted method has: a call forwarded through the embedded field.
///
/// Exposed because a TRAIT IMPL needs it too. A promoted method has no body of its own — what it
/// does is forward — so a trait impl carrying that method builds the same forward rather than
/// delegating to an inherent twin, which is the shadowing pair this engine no longer emits.
///
/// # Errors
/// [`TransformError::MissingDatum`] when the promotion records no field path to forward through.
pub(crate) fn forwarding_body(
    promoted: &Declaration,
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<Option<Vec<RustStmt>>, TransformError> {
    Ok(forwarding_method(promoted, declaration, resolver)?.body)
}

fn forwarding_method(
    promoted: &Declaration,
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<RustFn, TransformError> {
    let via = promoted
        .attr(ATTR_VIA)
        .ok_or_else(|| TransformError::MissingDatum {
            construction: "promoted method".to_owned(),
            name: promoted.name.clone(),
            datum: ATTR_VIA,
        })?;

    let mut rendered = method_signature(
        promoted,
        resolver,
        Visibility::Public,
        Body::None,
        &declaration.name,
        crate::body::ResultShape::Inherited,
    )?;

    // A promotion through an ABSENT-CAPABLE field is not a forward the target can spell. The
    // source embeds a POINTER and promotes the pointee's methods, so calling one when the pointer is
    // absent is legal there and panics at the embedding; the target holds the field as an option,
    // which has no method of that name at all. The same question the ordinary call path asks — and
    // it must be asked here too, because a promoted method's body is synthesised rather than
    // translated, so nothing on the call path ever sees it.
    refuse_absent_capable_promotion(promoted, declaration, via, resolver)?;

    // The field path is walked OUTWARD from `self`, one field per segment, because embedding
    // nests: a method promoted through two levels is reached through two fields, and the source
    // spells that as one name.
    let mut receiver = RustExpr::SelfValue;
    for segment in via.split('.') {
        receiver = RustExpr::Field {
            base: Box::new(receiver),
            name: to_snake_case(segment),
        };
    }

    let call = RustExpr::MethodCall {
        receiver: Box::new(receiver),
        method: rendered.name.clone(),
        args: rendered
            .params
            .iter()
            .map(|param| RustExpr::Path(param.name.clone()))
            .collect(),
    };
    rendered.body = Some(vec![if rendered.ret.is_some() {
        RustStmt::Tail(call)
    } else {
        RustStmt::Semi(call)
    }]);
    Ok(rendered)
}

/// Refuse a promotion whose embedded field may hold NOTHING in the target.
///
/// The source embeds a pointer and promotes what it points at; the target holds that field as an
/// option, and an option has none of the pointee's methods. Neither repair is faithful: unwrapping
/// claims a value the source never promised, and the source's own behaviour when the pointer is
/// absent is to panic at the embedding, which is not something to reproduce deliberately.
///
/// Asked of the RESOLVER, like the ordinary call path, so a pointer the ownership rules gave a
/// borrow — which has no absent case — is not refused.
///
/// # Errors
/// [`TransformError::Unsupported`] naming the field, the method, and what is missing.
fn refuse_absent_capable_promotion(
    promoted: &Declaration,
    declaration: &Declaration,
    via: &str,
    resolver: &Resolver<'_>,
) -> Result<(), TransformError> {
    let Some(first) = via.split('.').next() else {
        return Ok(());
    };
    let Some(field) = declaration
        .children_of_kind(crate::vocabulary::CHILD_FIELD)
        .into_iter()
        .find(|field| field.name == first)
    else {
        return Ok(());
    };
    let Ok(resolved) = resolver.resolve_in(
        &field.type_ref,
        &declaration.name,
        crate::vocabulary::POSITION_FIELD,
    ) else {
        return Ok(());
    };
    if !resolved.spelling().starts_with("Option<") {
        return Ok(());
    }
    Err(TransformError::Unsupported {
        name: promoted.name.clone(),
        detail: format!(
            "`{}` is promoted through `{first}`, an embedded field the target holds as a value that \
             may be ABSENT, and `{}` is a method of what it holds rather than of the option. The \
             source panics at the embedding when the pointer is absent, which is not a behaviour to \
             reproduce deliberately, and unwrapping would claim a value the source never promised. \
             What is missing is a proof that this field is never absent",
            first, promoted.name
        ),
    })
}
