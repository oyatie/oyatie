//! The source's byte-order package, as the target's own integer conversions.
//!
//! The largest refusal in the corpus by both measures — 48 call sites across four of seven packages —
//! and it could not be answered by the ordinary call map for a structural reason: these are METHODS
//! on a package-level value, so the front end records no callee identity for them. What it does
//! record is the package's IMPORT PATH, which is what this keys on. An import may be aliased, and
//! `binary.BigEndian` and `bin.BigEndian` are one call written two ways.
//!
//! Built as a TREE rather than substituted into a text template. This engine has twice been bitten
//! by a rule that emitted target text: text is opaque to every rule downstream, so an accumulator
//! fold or an ownership decision cannot see through it. The pack names the pieces; the shape is code.

use port_engine_api::Declaration;
use port_engine_rust_ir::RustExpr;

use crate::body::Body;
use crate::error::TransformError;
use crate::vocabulary::{ATTR_PACKAGE_PATH, KIND_IDENT, KIND_SELECTOR};

/// A call to the source's byte-order package, or `None` if this is not one.
///
/// # Errors
/// [`TransformError::Unsupported`] when the call is one of these but does not have the arity the
/// source's own signature has — which would mean the front end recorded something this cannot read.
pub(crate) fn byte_order_call(
    callee: &Declaration,
    args: &[RustExpr],
    cx: &Body<'_>,
) -> Result<Option<RustExpr>, TransformError> {
    let rule = cx.resolver.byte_order_calls;
    if rule.package.is_empty() {
        return Ok(None);
    }
    // `<pkg>.<Order>.<Method>` — three levels, and the innermost must be the declared package.
    let (method, order_node) = (callee.name.as_str(), callee.children.first());
    let Some(order_node) = order_node.filter(|node| node.kind == KIND_SELECTOR) else {
        return Ok(None);
    };
    let is_package = order_node.children.first().is_some_and(|pkg| {
        pkg.kind == KIND_IDENT && pkg.attr(ATTR_PACKAGE_PATH) == Some(rule.package.as_str())
    });
    if !is_package {
        return Ok(None);
    }
    let Some(order) = rule.orders.get(&order_node.name) else {
        return Ok(None);
    };

    if let Some(target) = rule.reads.get(method) {
        let [source] = args else {
            return Ok(None);
        };
        // The source's read PANICS when the slice is short, and so does the fit — so the unwrap is
        // the source's own behaviour restated, not a failure mode this engine introduced.
        let fitted = RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(source.clone()),
                method: rule.fit_method.clone(),
                args: Vec::new(),
            }),
            method: rule.fit_unwrap.clone(),
            args: Vec::new(),
        };
        let path = rule
            .read_form
            .replace("{type}", target)
            .replace("{order}", order);
        return Ok(Some(RustExpr::Call {
            callee: Box::new(RustExpr::Path(path)),
            args: vec![fitted],
        }));
    }

    if rule.writes.contains_key(method) {
        let [destination, value] = args else {
            return Ok(None);
        };
        // The source WRITES INTO its first argument; the target converts and copies. Both leave the
        // destination holding the value's bytes in the declared order, and both panic on a short one.
        let bytes = RustExpr::MethodCall {
            receiver: Box::new(value.clone()),
            method: rule.write_form.replace("{order}", order),
            args: Vec::new(),
        };
        return Ok(Some(RustExpr::MethodCall {
            receiver: Box::new(destination.clone()),
            method: rule.write_method.clone(),
            args: vec![RustExpr::Reference {
                mutable: false,
                inner: Box::new(bytes),
            }],
        }));
    }
    Ok(None)
}

/// Whether this call WRITES INTO its first argument.
///
/// The source spells the write as a call — `PutUint32(b, v)` leaves `b` holding the bytes — and the
/// target spells it as a mutation. A binding that appears only as such an argument is never observed
/// assigned, so it comes out immutable and the emitted mutation does not compile.
pub(crate) fn writes_into_first_argument(node: &Declaration, resolver: &crate::resolve::Resolver<'_>) -> bool {
    let rule = resolver.byte_order_calls;
    if rule.package.is_empty() {
        return false;
    }
    let Some(callee) = node.children.first().filter(|c| c.kind == KIND_SELECTOR) else {
        return false;
    };
    if !rule.writes.contains_key(&callee.name) {
        return false;
    }
    callee
        .children
        .first()
        .filter(|order| order.kind == KIND_SELECTOR)
        .and_then(|order| order.children.first())
        .is_some_and(|pkg| {
            pkg.kind == KIND_IDENT && pkg.attr(ATTR_PACKAGE_PATH) == Some(rule.package.as_str())
        })
}
