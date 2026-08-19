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
) -> Result<Vec<RustFn>, TransformError> {
    declaration
        .children_of_kind(CHILD_PROMOTED)
        .into_iter()
        .map(|promoted| forwarding_method(promoted, declaration, resolver))
        .collect()
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
    )?;

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
