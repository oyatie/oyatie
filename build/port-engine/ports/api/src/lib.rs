//! # port-engine-api — the ports face of the owned deterministic port engine.
//!
//! ADR-0637 (archived; live via apex ADR-0704) D1 assigns this face the seams: `SourceModel`,
//! `RulePack`, `PackSemantics`, `TransformPlan`, `TargetIr`, `Renderer`, and the six-axis
//! `Receipt`. `port-engine-kernel` owns neutrality enforcement and the `plan` / `emit` / `verify`
//! entry points; the adapters implement the seams.
//!
//! Zero dependencies by design: seam types name no corpus type and carry no adapter machinery.
#![forbid(unsafe_code)]

/// This crate's own sources, for the engine-identity axis assembled by the facade.
mod sources;
pub use sources::CRATE_SOURCES;

mod declaration;
mod error;
mod value_rules;
mod failure;
mod identity;
mod ownership;
mod plan;
mod receipt;
mod seams;
mod type_ref;

pub use declaration::Declaration;
pub use error::PortError;
pub use value_rules::{Allocation, BinaryString, ChannelForms, ForeignType, BitPatternConstants, ByteOrderCalls, ReadableLiterals, FormatCalls, FormatFunction, SequenceAppend};
pub use failure::{
    DeriveRule, DocConvention, FailureConvention, FunctionMapping, IdiomRule, IntegerArithmetic,
};
pub use identity::{Digest, LanguagePair, PAIR_SEPARATOR, RegionId, RuleId, UnitId};
pub use ownership::{OwnershipFacts, PointerConstruction, PointerDisposition};
pub use plan::{PlanStep, TransformPlan};
pub use receipt::{RECEIPT_AXES, Receipt, ReceiptAxis};
pub use seams::{PackSemantics, Renderer, RulePack, SourceModel, TargetIr};
pub use type_ref::TypeRef;

/// Fail-closed readiness gate. `true` once the seam types are present.
#[must_use]
pub const fn w0_ready() -> bool {
    true
}
