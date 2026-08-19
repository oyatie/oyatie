//! Parameters the target types as its INDEX type.
//!
//! The same question the loop counter asks, one scope out: a name whose every read is an index
//! operand never has its signed value observed, so it does not need one. The difference is that a
//! parameter's value comes from a CALLER, which is why the call site converts.
//!
//! Split from `returns.rs` because a parameter and a result are proved from opposite directions —
//! a parameter from how the body USES it, a result from what the body PRODUCES.

use std::collections::BTreeSet;

use port_engine_api::Declaration;

use crate::vocabulary::{CHILD_BODY, CHILD_PARAM, SOURCE_INT};

/// Every parameter the signature made the target's index type.
///
/// Read by the BODY, so a read of one is not converted a second time: the signature and the body
/// must agree, and they agree by asking this rather than each deciding.
pub(crate) fn index_parameters(
    declaration: &Declaration,
    resolver: &crate::resolve::Resolver<'_>,
) -> BTreeSet<String> {
    declaration
        .children_of_kind(crate::vocabulary::CHILD_PARAM)
        .into_iter()
        .filter(|param| indexes_only_parameter(declaration, param, resolver))
        .map(|param| crate::naming::to_snake_case(&param.name))
        .collect()
}

/// Whether this parameter is used for NOTHING BUT indexing, and so is a `usize`.
///
/// The same question the loop counter asks, one scope out: a name whose every read is an index
/// operand never has its signed value observed, so it does not need one. The difference is that a
/// parameter's value comes from a CALLER, which is why the call site converts — see `body_call`.
///
/// Requires the source's own integer type, because a parameter of any other type is not a candidate
/// for the target's index type at all; and a BODY, because a signature-only declaration proves
/// nothing about how its parameters are used.
pub(crate) fn indexes_only_parameter(
    declaration: &Declaration,
    param: &Declaration,
    resolver: &crate::resolve::Resolver<'_>,
) -> bool {
    if param.type_ref.name != SOURCE_INT || resolver.idiom_index_counter().is_none() {
        return false;
    }
    let Some(body) = declaration.children_of_kind(CHILD_BODY).first().copied() else {
        return false;
    };
    crate::counters::indexes_only(body, &param.name)
}
