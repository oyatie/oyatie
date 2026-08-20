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

use std::collections::{BTreeMap, BTreeSet};

use port_engine_api::{PackSemantics, SourceModel};
use port_engine_rust_ir::RustType;

use crate::ownership::OwnershipContext;
use crate::params::variadic_is_a_slice;
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
    /// Callees whose last parameter is VARIADIC in the source.
    ///
    /// The signature itself needs nothing special — the parameter is already a slice — but a CALL
    /// does: the trailing arguments have to be collected into one, and which of the target's
    /// sequence forms that is has not been decided.
    variadic: BTreeSet<String>,
    /// Callees that are only a SHORTHAND for a call the target already spells directly.
    ///
    /// Held here because it is a fact about another declaration that a CALL SITE needs, which is
    /// what this table is for. A call to one of these is the call it wraps.
    eta: BTreeMap<String, crate::eta::EtaWrapper>,
}

impl SignatureTable {
    /// The destination of argument `index` of `callee`, or `None` when this table cannot say.
    ///
    /// `None` is not a failure and not an empty answer: it means the callee is a method, is
    /// foreign, or has a signature the engine itself could not translate. The caller refuses or
    /// leaves the argument alone; it must never read `None` as "no conversion needed".
    /// The call this callee is only a shorthand for, when it is one.
    pub(crate) fn eta(&self, callee: &str) -> Option<&crate::eta::EtaWrapper> {
        self.eta.get(callee)
    }

    pub(crate) fn param(&self, callee: &str, index: usize) -> Option<&ParamTarget> {
        self.by_callee.get(callee)?.get(index)
    }

    /// Whether this callee's last parameter is variadic in the source.
    pub(crate) fn is_variadic(&self, callee: &str) -> bool {
        self.variadic.contains(callee)
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
        let mut variadic = BTreeSet::new();
        let mut eta = BTreeMap::new();
        let mapped: BTreeSet<String> = semantics.function_map().keys().cloned().collect();
        // Every module the emitted crate will have, so a signature naming a package outside them
        // refuses here rather than producing a path that resolves to nothing.
        let units: BTreeSet<String> = model.units().into_iter().map(|unit| unit.0).collect();
        for unit in model.units() {
            let Some(declarations) = model.declarations(&unit) else {
                continue;
            };
            let scope = LocalScope::with_lengths(
            &declarations,
            semantics.failure_convention(),
            semantics.length_functions(),
            &semantics.format_calls().functions.keys().cloned().collect(),
            semantics.length_argument_callees(),
        );
            // Every name: a SIGNATURE names no body, so nothing here can depend on whether another
            // declaration's body translated.
            let declared: BTreeSet<String> = declarations
                .iter()
                .map(|declaration| declaration.name.clone())
                .collect();
            // A THROWAWAY log: this pass builds SIGNATURES to key the table by and reports nothing.
            let drops = crate::dropped::DropLog::new();
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
                format_calls: semantics.format_calls(),
                unmappable_calls: semantics.unmappable_calls(),
                unmappable_types: semantics.unmappable_types(),
                unmappable_facts: semantics.unmappable_facts(),
                binary_string: semantics.binary_string(),
                bit_pattern_constants: semantics.bit_pattern_constants(),
                allocation: semantics.allocation(),
                sequence_append: semantics.sequence_append(),
                integer_arithmetic: semantics.integer_arithmetic(),
                doc_convention: semantics.doc_convention(),
                derives: semantics.derives(),
                idioms: semantics.idioms(),
                literal_constructors: semantics.literal_constructors(),
                receiver: semantics.trait_receiver(),
                deferred: semantics.deferred_kinds(),
                constant_map: semantics.constant_map(),
                prose_type_names: semantics.prose_type_names(),
                length_functions: semantics.length_functions(),
                undecided_forms: semantics.undecided_forms(),
                ownership,
                drops: &drops,
                emitted: &declared,
                units: &units,
                unit: &unit,
                signatures: &Self::default(),
            };
            for declaration in declarations.iter().filter(|d| d.kind == KIND_FUNC) {
                // Keyed by the same IDENTITY a call site carries — the unit path and the name —
                // because that is what the call names. Keyed by the bare name it matched nothing.
                // Keyed by the same IDENTITY a call site carries, and only where nothing takes
                // the function as a VALUE: every call becomes the call it wraps, so the declaration
                // can go — but a `f := rol31` would be left pointing at nothing.
                if let Some(shorthand) = crate::eta::wrapper(declaration, &mapped, &units)
                    && !crate::eta::used_as_value(&declarations, &declaration.name)
                {
                    eta.insert(format!("{}.{}", unit.0, declaration.name), shorthand);
                }
                // EMPTY: this table answers about signatures, and whether a body folds is a fact
                // about the body. A `mut` here is only ever read for a parameter's destination,
                // which the fold does not change.
                let Ok(params) =
                    crate::params::params(declaration, &resolver, &unit.0, &BTreeSet::new())
                else {
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
                let identity = format!("{}.{}", unit.0, declaration.name);
                if variadic_is_a_slice(declaration) {
                    variadic.insert(identity.clone());
                }
                by_callee.insert(identity, targets);
            }
        }
        Self {
            by_callee,
            variadic,
            eta,
        }
    }
}
