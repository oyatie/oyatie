//! Typed refusals from transform apply.
//!
//! Every one names WHAT it refused and, where an analysis exists, where that analysis lives. A
//! refusal is a finding; a guess is a defect the receipt would certify as reproducible.

use std::fmt;

use port_engine_api::PortError;

/// Typed refusal from transform apply.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransformError {
    /// Pack did not declare construction/precondition/captures for a planned rule.
    MissingSemantics {
        /// Rule missing semantics.
        rule: String,
        /// Which field was absent.
        field: &'static str,
    },
    /// Precondition evaluation refused.
    Precondition {
        /// Rule being applied.
        rule: String,
        /// Unit under transform.
        unit: String,
        /// Precondition id that failed.
        precondition: String,
    },
    /// A form the pack has declined to decide, in the pack's own words.
    ///
    /// Distinct from a refusal for missing capability: nothing is unknowable here, someone has not
    /// yet chosen. Carrying the pack's text means the reason a reader sees and the reason the
    /// digest holds cannot drift apart.
    UndecidedForm {
        /// The form id the pack records a reason under.
        form: String,
        /// The declaration that hit it.
        name: String,
        /// The pack's recorded reason.
        reason: String,
    },
    /// An initialiser the target cannot evaluate at compile time.
    NotConstantExpression {
        /// The declaration being initialised.
        name: String,
        /// What about the initialiser is not constant.
        detail: String,
    },
    /// Construction id is not one of the known set.
    UnknownConstruction {
        /// Rule being applied.
        rule: String,
        /// Construction id found.
        construction: String,
    },
    /// A construction was applied to a declaration kind it cannot build from.
    ConstructionKindMismatch {
        /// Construction that was asked.
        construction: String,
        /// Declaration kind it was asked to build from.
        kind: String,
        /// Declaration name, for locating it.
        name: String,
    },
    /// A construction needs a declared datum the model does not carry.
    MissingDatum {
        /// Construction that needs it.
        construction: String,
        /// Declaration that lacks it.
        name: String,
        /// What was missing.
        datum: &'static str,
    },
    /// A type spelling resolves to nothing: not declared in the unit, not in the pack's type map.
    UnmappedType {
        /// Unit under transform.
        unit: String,
        /// Declaration whose type could not be resolved.
        name: String,
        /// The unresolvable source type spelling.
        type_ref: String,
    },
    /// A declaration is captured by no rule and deferred by no policy.
    UncapturedDeclaration {
        /// Unit that declares it.
        unit: String,
        /// Declaration name.
        name: String,
        /// Declaration kind that nothing selects.
        kind: String,
    },
    /// A construct the engine does not translate yet, refused by name.
    Unsupported {
        /// Declaration that carries it.
        name: String,
        /// What is unsupported, and where the program records the analysis.
        detail: String,
    },
    /// The plan named a unit the model does not carry.
    UnitNotInModel {
        /// The absent unit.
        unit: String,
    },
    /// No ownership disposition could be decided for a pointer.
    Ownership {
        /// What the analysis said.
        detail: String,
    },
    /// IR / syn assembly refused.
    Ir(PortError),
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSemantics { rule, field } => {
                write!(f, "transform missing `{field}` for rule `{rule}`")
            }
            Self::Precondition {
                rule,
                unit,
                precondition,
            } => write!(
                f,
                "transform precondition `{precondition}` failed for rule `{rule}` unit `{unit}`"
            ),
            Self::UndecidedForm { form, name, reason } => write!(
                f,
                "`{name}` is a `{form}`, whose target form the pack has not decided: {reason}"
            ),
            Self::NotConstantExpression { name, detail } => {
                write!(f, "`{name}` cannot be a static: {detail}")
            }
            Self::UnknownConstruction { rule, construction } => write!(
                f,
                "transform unknown construction `{construction}` for rule `{rule}`"
            ),
            Self::ConstructionKindMismatch {
                construction,
                kind,
                name,
            } => write!(
                f,
                "transform construction `{construction}` cannot build from a `{kind}` declaration \
                 (`{name}`)"
            ),
            Self::MissingDatum {
                construction,
                name,
                datum,
            } => write!(
                f,
                "transform construction `{construction}` needs `{datum}` for declaration `{name}`"
            ),
            Self::UnmappedType {
                unit,
                name,
                type_ref,
            } => write!(
                f,
                "transform cannot resolve type `{type_ref}` for `{name}` in unit `{unit}`: it is \
                 declared nowhere in the unit and the pack's type map does not carry it"
            ),
            Self::UncapturedDeclaration { unit, name, kind } => write!(
                f,
                "transform refuses to drop `{name}`: unit `{unit}` declares it as `{kind}`, no \
                 rule captures that kind, and the pack does not defer it"
            ),
            Self::Unsupported { name, detail } => {
                write!(f, "transform refuses `{name}`: {detail}")
            }
            Self::UnitNotInModel { unit } => {
                write!(
                    f,
                    "transform planned unit `{unit}` is absent from the model"
                )
            }
            Self::Ownership { detail } => write!(f, "transform ownership: {detail}"),
            Self::Ir(err) => write!(f, "transform IR assembly failed: {err}"),
        }
    }
}

impl std::error::Error for TransformError {}
