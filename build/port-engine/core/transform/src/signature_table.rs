//! What a call's DESTINATION wants.
//!
//! The body translator knows what an expression is and not where it is going, and several
//! translations need the second. `&x` yields a pointer whose target form is an ownership decision;
//! a bare string literal is a borrowed `&'static str` where the pack maps `string` to an owned
//! `String`. Both were refused for the same reason — measured over the seven surveyed corpora, 11
//! of 33 `&` sites are `f(&x)`, where the destination is the CALLEE's parameter.
//!
//! Every one of those destinations is a signature the engine has already translated. This is that
//! translation, done once and keyed by the identity a call records, so an argument site can ask
//! instead of guess.
//!
//! It answers the ARGUMENT position only. `x := &T{..}`, `return &T{..}` and `x = &T{..}` are 17 of
//! the 33 sites and their destinations are a local's inferred type and a function's result — known
//! to the engine, and not yet reaching the expression walk. Those still refuse, and the refusal
//! now says which position it is rather than claiming the target has no form.
//!
//! WHAT IT DOES NOT ANSWER, and refuses by name rather than approximating:
//!
//!   * A METHOD. The source spells `value.Method()` with no package-qualified identity, so a
//!     method's key is its receiver type rather than a path, and receiver resolution is a question
//!     this table does not ask. 52 of the calls in `uuid` are methods.
//!   * A FOREIGN function — `fmt.Sprintf`, `encoding/hex.Encode`. Its signature is not in the
//!     snapshot at all, so there is nothing to translate. The pack answers for the ones that
//!     matter through `function_map`; the rest have no answer and must not get a guess.
//!
//! Built with NO construction overrides. The one override the pack declares is `rust_const`
//! mapping `string` to `&str`, and a function parameter is never inside a constant — so the base
//! type map is the right answer here and a construction-specific table would be answering a
//! question no parameter asks.

use std::collections::BTreeMap;

use port_engine_api::{PackSemantics, SourceModel};
use port_engine_rust_ir::RustType;

use crate::ownership::OwnershipContext;
use crate::resolve::{LocalScope, Resolver};
use crate::vocabulary::{CHILD_PARAM, KIND_FUNC, TYPE_POINTER};

/// One parameter's translated destination.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ParamTarget {
    /// The target type the parameter holds.
    pub(crate) ty: RustType,
    /// The disposition that chose it, when the parameter is a pointer.
    ///
    /// Carried so an argument site can find the SAME rule the parameter used, rather than
    /// recovering a decision by matching the spelling it produced.
    pub(crate) disposition: Option<String>,
}

/// Callee identity → the destinations of its parameters, in order.
#[derive(Debug, Default)]
pub(crate) struct SignatureTable {
    by_callee: BTreeMap<String, Vec<ParamTarget>>,
}

impl SignatureTable {
    /// The destination of argument `index` of `callee`, or `None` when this table cannot say.
    ///
    /// `None` is not a failure and not an empty answer: it means the callee is a method, is
    /// foreign, or has a signature the engine itself could not translate. The caller refuses or
    /// leaves the argument alone; it must never read `None` as "no conversion needed".
    pub(crate) fn param(&self, callee: &str, index: usize) -> Option<&ParamTarget> {
        self.by_callee.get(callee)?.get(index)
    }

    /// Translate every free function in the model once.
    ///
    /// A signature the engine cannot translate is OMITTED rather than fatal. The table exists to
    /// answer questions about destinations, and a destination the engine could not name is one no
    /// argument site should be given an answer for — the declaration owning that signature will
    /// refuse on its own terms when its turn comes.
    pub(crate) fn build(
        model: &dyn SourceModel,
        semantics: &dyn PackSemantics,
        ownership: &OwnershipContext<'_>,
    ) -> Self {
        let mut by_callee = BTreeMap::new();
        for unit in model.units() {
            let Some(declarations) = model.declarations(&unit) else {
                continue;
            };
            let scope = LocalScope::of(&declarations);
            let resolver = Resolver {
                scope: &scope,
                type_map: semantics.type_map(),
                overrides: None,
                constructors: semantics.type_constructors(),
                copy_types: semantics.copy_types(),
                cast_types: semantics.cast_types(),
                zero_values: semantics.zero_values(),
                trait_object_forms: semantics.trait_object_forms(),
                failure: semantics.failure_convention(),
                function_map: semantics.function_map(),
                integer_arithmetic: semantics.integer_arithmetic(),
                doc_convention: semantics.doc_convention(),
                receiver: semantics.trait_receiver(),
                deferred: semantics.deferred_kinds(),
                ownership,
                unit: &unit,
                signatures: &Self::default(),
            };
            for declaration in declarations.iter().filter(|d| d.kind == KIND_FUNC) {
                let Ok(params) = crate::params::params(declaration, &resolver, &unit.0) else {
                    continue;
                };
                let targets = declaration
                    .children_of_kind(CHILD_PARAM)
                    .into_iter()
                    .zip(params)
                    .map(|(source, param)| ParamTarget {
                        ty: param.ty,
                        disposition: match source.type_ref.kind == TYPE_POINTER {
                            true => ownership.decided_for(&format!(
                                "{}::{}({})",
                                unit.0, declaration.name, source.name
                            )),
                            false => None,
                        },
                    })
                    .collect();
                by_callee.insert(format!("{}.{}", unit.0, declaration.name), targets);
            }
        }
        Self { by_callee }
    }
}
