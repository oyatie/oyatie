//! The evaluator: parse the untrusted [`ResidencyContext`] strings, resolve
//! both endpoints to exactly one cell through the catalog port, resolve every
//! asserted transfer basis through the register port, and hand the result to
//! the pure cascade in [`crate::domain`].
//!
//! Everything that cannot be parsed or resolved denies. The only outcomes that
//! are NOT a decision are the two the caller must treat as "the control could
//! not run": a catalog or register that failed to answer
//! ([`ResidencyAdapterError::EvaluationFailed`]) and a catalog whose rows are
//! structurally unusable ([`ResidencyAdapterError::PolicyMalformed`]). Neither
//! is an allow — see [`crate::dispatch_permitted`].
//!
//! Region resolution is by CELL, not by region id: a region that hosts more
//! than one pack (the documented `pack-us` / `pack-us-healthcare` co-tenancy on
//! `us-ashburn-1`) needs the caller to say which cell the route touches, and
//! refuses the route when it does not.

use crate::domain::{EvaluationInputs, ResidencyOutcome, ResidencyRule, decide};
use crate::kernel::{
    NoTransferRegister, RegionRecord, ResidencyClass, ResidencyDataClass, ResidencyOperation,
    ResidencyOverlay, ResidencyRegionCatalog, ResidencyTransferRegister, TransferBasis,
};
use crate::{ResidencyAdapterError, ResidencyContext, ResidencyDecision, ResidencyPolicyEvaluator};

/// A hand-written mirror of the tenancy residency policy, evaluated against a
/// region catalog and a legal-transfer register.
///
/// The name says evaluator, not authority. `tenancy/policy/data-residency.cedar`
/// and `tenancy/policy/data-residency.md` remain the authority; this type must
/// be kept in step with them by hand, and nothing in this crate can prove that
/// it is (see "Gaps" in `lib.rs`).
///
/// The register defaults to [`NoTransferRegister`], which holds nothing. An
/// engine built with [`Self::try_new`] therefore authorises no SCC transfer and
/// no cross-jurisdiction migration at all: the caller's assertion has nothing
/// to be checked against, and an unchecked assertion is not a basis.
#[derive(Clone, Debug)]
pub struct ResidencyPolicyEngine<
    C: ResidencyRegionCatalog,
    R: ResidencyTransferRegister = NoTransferRegister,
> {
    catalog: C,
    register: R,
}

/// What resolving one endpoint produced.
enum CellResolution {
    /// Exactly one cell.
    Cell(Box<RegionRecord>),
    /// No cell of the requested pack in that region.
    Unknown,
    /// More than one cell, and the caller did not say which.
    Ambiguous,
}

impl<C: ResidencyRegionCatalog> ResidencyPolicyEngine<C, NoTransferRegister> {
    /// Build an engine over a catalog, with no legal-transfer register.
    ///
    /// An empty catalog is malformed, not permissive: with no rows every
    /// lookup misses, and a control that cannot resolve any region is a broken
    /// control that should be reported as broken rather than run.
    ///
    /// # Errors
    ///
    /// - [`ResidencyAdapterError::EvaluationFailed`] if the catalog cannot be
    ///   enumerated.
    /// - [`ResidencyAdapterError::PolicyMalformed`] if it is empty, holds a row
    ///   with a blank field, or lists the same `(region_id, pack_id)` cell
    ///   twice with different content.
    pub fn try_new(catalog: C) -> Result<Self, ResidencyAdapterError> {
        Self::try_new_with_register(catalog, NoTransferRegister)
    }
}

impl<C: ResidencyRegionCatalog, R: ResidencyTransferRegister> ResidencyPolicyEngine<C, R> {
    /// Build an engine over a catalog and a legal-transfer register.
    ///
    /// # Errors
    ///
    /// As [`Self::try_new`].
    pub fn try_new_with_register(catalog: C, register: R) -> Result<Self, ResidencyAdapterError> {
        let rows = catalog.regions()?;
        if rows.is_empty() {
            return Err(ResidencyAdapterError::PolicyMalformed);
        }
        for row in &rows {
            row.validate()?;
        }
        // A cell is `(region_id, pack_id)`. Two rows for one cell that disagree
        // about its jurisdiction or role are a contradiction with no
        // last-one-wins resolution. Two rows for one REGION that name different
        // packs are the documented co-tenancy, not a contradiction — they are
        // resolved by the caller declaring which cell the route touches.
        for (index, row) in rows.iter().enumerate() {
            if rows.iter().take(index).any(|earlier| {
                earlier.region_id == row.region_id
                    && earlier.pack_id == row.pack_id
                    && earlier != row
            }) {
                return Err(ResidencyAdapterError::PolicyMalformed);
            }
        }
        Ok(Self { catalog, register })
    }

    /// The catalog this engine reads.
    pub const fn catalog(&self) -> &C {
        &self.catalog
    }

    /// The legal-transfer register this engine checks asserted bases against.
    pub const fn register(&self) -> &R {
        &self.register
    }

    fn resolve(
        &self,
        region_id: &str,
        declared_pack: Option<&str>,
    ) -> Result<CellResolution, ResidencyAdapterError> {
        let rows = self.catalog.rows_for(region_id)?;
        for row in &rows {
            // A catalog is a port, so a row can be malformed even though
            // `try_new` validated what `regions()` reported. Refusing here
            // keeps a blank jurisdiction from comparing equal to everything,
            // and a row answering for a different region from being believed.
            row.validate()?;
            if row.region_id != region_id {
                return Err(ResidencyAdapterError::PolicyMalformed);
            }
        }
        let candidates: Vec<&RegionRecord> = match declared_pack {
            Some(pack) => rows.iter().filter(|row| row.is_in_pack(pack)).collect(),
            None => rows.iter().collect(),
        };
        let Some(first) = candidates.first() else {
            return Ok(CellResolution::Unknown);
        };
        if candidates.iter().any(|row| row != first) {
            return Ok(CellResolution::Ambiguous);
        }
        Ok(CellResolution::Cell(Box::new((*first).clone())))
    }

    /// Resolve the caller's asserted bases against the register.
    ///
    /// Returns `(scc_verified, permit_verified)`. A basis counts only when the
    /// register holds a row for THIS tenant and THIS route — a register row is
    /// not a bearer token that authorises any transfer that cites it.
    fn verify_bases(
        &self,
        ctx: &ResidencyContext,
        source: &RegionRecord,
        destination: &RegionRecord,
    ) -> Result<(bool, bool), ResidencyAdapterError> {
        let mut scc_verified = false;
        let mut permit_verified = false;
        for basis in ctx.transfer_bases() {
            match basis {
                TransferBasis::StandardContractualClauses { .. } => {
                    if scc_verified || !basis.is_complete_scc() {
                        continue;
                    }
                    let (Some(register_ref), Some(purpose)) =
                        (basis.scc_register_ref(), basis.scc_transfer_purpose())
                    else {
                        continue;
                    };
                    let Some(entry) = self.register.scc_entry(register_ref)? else {
                        continue;
                    };
                    scc_verified = entry.register_ref == register_ref
                        && entry.tenant_id == ctx.tenant_id
                        && entry.source_jurisdiction == source.jurisdiction
                        && entry.destination_jurisdiction == destination.jurisdiction
                        && entry.transfer_purpose == purpose
                        && entry.adequacy_or_safeguard
                        && entry.supplementary_measures;
                }
                TransferBasis::CrossJurisdictionCedarPermit { .. } => {
                    if permit_verified || !basis.is_valid_cross_jurisdiction_permit() {
                        continue;
                    }
                    let Some(permit_id) = basis.cedar_permit_id() else {
                        continue;
                    };
                    let Some(entry) = self.register.cross_jurisdiction_permit(permit_id)? else {
                        continue;
                    };
                    permit_verified = entry.permit_id == permit_id
                        && entry.tenant_id == ctx.tenant_id
                        && entry.source_jurisdiction == source.jurisdiction
                        && entry.destination_jurisdiction == destination.jurisdiction
                        && entry.audit_chain_emit;
                }
            }
        }
        Ok((scc_verified, permit_verified))
    }

    /// Evaluate a route and report both the decision and the rule that made it.
    ///
    /// # Errors
    ///
    /// [`ResidencyAdapterError::EvaluationFailed`] when the catalog or register
    /// cannot answer, [`ResidencyAdapterError::PolicyMalformed`] when a
    /// resolved row is structurally unusable. Everything else is a decision,
    /// and every unrecognised input is a denial.
    pub fn evaluate_detailed(
        &self,
        ctx: &ResidencyContext,
    ) -> Result<ResidencyOutcome, ResidencyAdapterError> {
        let Some(data_class) = ResidencyDataClass::parse_label(&ctx.data_class) else {
            return Ok(denied_input(
                ResidencyDecision::DenyDataClass,
                ResidencyRule::UnknownDataClass,
            ));
        };
        let Some(residency_class) = ResidencyClass::parse_label(&ctx.residency_class) else {
            return Ok(denied_input(
                ResidencyDecision::DenyResidency,
                ResidencyRule::UnknownResidencyClass,
            ));
        };
        let Some(operation) = ResidencyOperation::parse_label(&ctx.operation) else {
            return Ok(denied_input(
                ResidencyDecision::DenyResidency,
                ResidencyRule::UnknownOperation,
            ));
        };
        let mut overlays = Vec::with_capacity(ctx.residency_overlays.len());
        for label in &ctx.residency_overlays {
            let Some(overlay) = ResidencyOverlay::parse_label(label) else {
                return Ok(denied_input(
                    ResidencyDecision::DenyJurisdictionPack,
                    ResidencyRule::UnknownResidencyOverlay,
                ));
            };
            overlays.push(overlay);
        }

        let source = match self.resolve(&ctx.source_region, ctx.tenant_pack.as_deref())? {
            CellResolution::Cell(record) => *record,
            CellResolution::Unknown => {
                return Ok(denied_input(
                    ResidencyDecision::DenyResidency,
                    ResidencyRule::UnknownRegion,
                ));
            }
            CellResolution::Ambiguous => {
                return Ok(denied_input(
                    ResidencyDecision::DenyResidency,
                    ResidencyRule::AmbiguousRegionRequiresPack,
                ));
            }
        };
        let destination =
            match self.resolve(&ctx.destination_region, ctx.destination_pack.as_deref())? {
                CellResolution::Cell(record) => *record,
                CellResolution::Unknown => {
                    return Ok(denied_input(
                        ResidencyDecision::DenyResidency,
                        ResidencyRule::UnknownRegion,
                    ));
                }
                CellResolution::Ambiguous => {
                    return Ok(denied_input(
                        ResidencyDecision::DenyResidency,
                        ResidencyRule::AmbiguousRegionRequiresPack,
                    ));
                }
            };

        let (scc_register_verified, permit_register_verified) =
            self.verify_bases(ctx, &source, &destination)?;

        Ok(decide(&EvaluationInputs {
            residency_class,
            operation,
            data_class,
            overlays: &overlays,
            source: &source,
            destination: &destination,
            transfer_basis: ctx.transfer_basis.as_ref(),
            additional_bases: &ctx.additional_transfer_bases,
            scc_register_verified,
            permit_register_verified,
        }))
    }
}

const fn denied_input(decision: ResidencyDecision, rule: ResidencyRule) -> ResidencyOutcome {
    ResidencyOutcome { decision, rule }
}

impl<C: ResidencyRegionCatalog, R: ResidencyTransferRegister> ResidencyPolicyEvaluator
    for ResidencyPolicyEngine<C, R>
{
    fn evaluate(&self, ctx: &ResidencyContext) -> Result<ResidencyDecision, ResidencyAdapterError> {
        Ok(self.evaluate_detailed(ctx)?.decision)
    }

    fn evaluate_outcome(
        &self,
        ctx: &ResidencyContext,
    ) -> Result<ResidencyOutcome, ResidencyAdapterError> {
        self.evaluate_detailed(ctx)
    }
}
