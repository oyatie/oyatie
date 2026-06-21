//! Tenancy contract family: tenant resource, lifecycle states, isolation
//! posture.
//!
//! Precedent: the AWS SaaS Well-Architected lens isolation models (silo /
//! pool / bridge) and cell-based architecture (AWS cell-based architecture
//! guidance; Azure deployment stamps): every tenant is pinned to exactly one
//! cell, and tenant-scoped resources must live in the owning tenant's cell.
//! The lifecycle is a closed state machine — create lands in `Provisioning`,
//! and `Retired` is terminal.

use serde::{Deserialize, Serialize};

use crate::{ContractViolation, MAX_DISPLAY_NAME_LEN, MAX_ID_LEN, check_slug, check_text};

/// Tenant lifecycle states (closed set).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantLifecycleState {
    /// Created but not yet serving traffic (initial state for create).
    Provisioning,
    /// Fully operational.
    Active,
    /// Reversibly halted (no data deletion; PDP fails closed).
    Suspended,
    /// Terminal: deprovisioned; the id is never reused.
    Retired,
}

impl TenantLifecycleState {
    /// The stable, lowercase slug for this state. This is the canonical wire /
    /// column serialization (matches the `snake_case` serde rename) — callers
    /// MUST use it instead of `format!("{self:?}")`, whose output is a Rust
    /// debug artifact that can silently drift from the persisted contract.
    #[must_use]
    pub fn slug(self) -> &'static str {
        match self {
            Self::Provisioning => "provisioning",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Retired => "retired",
        }
    }

    /// The state a newly created tenant starts in.
    #[must_use]
    pub fn initial() -> Self {
        Self::Provisioning
    }

    /// Whether tenant workloads may serve in this state.
    #[must_use]
    pub fn is_operational(self) -> bool {
        matches!(self, Self::Active)
    }

    /// Whether the state is terminal.
    #[must_use]
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Retired)
    }
}

/// Lifecycle operations a control plane may apply to an existing tenant.
/// (Create is not listed: it constructs the tenant in
/// [`TenantLifecycleState::initial`] rather than transitioning one.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantLifecycleOperation {
    /// Provisioning -> Active (provisioning completed).
    Activate,
    /// Active -> Suspended (reversible halt).
    Suspend,
    /// Suspended -> Active (reverse of suspend).
    Resume,
    /// Any non-terminal state -> Retired (terminal).
    Retire,
}

impl TenantLifecycleOperation {
    fn name(self) -> &'static str {
        match self {
            Self::Activate => "activate",
            Self::Suspend => "suspend",
            Self::Resume => "resume",
            Self::Retire => "retire",
        }
    }

    /// The closed transition function. Returns the next state, or the exact
    /// violation when the operation is not allowed from `from`.
    pub fn apply(
        self,
        from: TenantLifecycleState,
    ) -> Result<TenantLifecycleState, ContractViolation> {
        use TenantLifecycleOperation as Op;
        use TenantLifecycleState as S;
        match (from, self) {
            (S::Provisioning, Op::Activate) => Ok(S::Active),
            (S::Active, Op::Suspend) => Ok(S::Suspended),
            (S::Suspended, Op::Resume) => Ok(S::Active),
            (S::Provisioning | S::Active | S::Suspended, Op::Retire) => Ok(S::Retired),
            (from, operation) => Err(ContractViolation::InvalidTransition {
                from: from.slug(),
                operation: operation.name(),
            }),
        }
    }
}

/// Tenant isolation posture, per the AWS SaaS Well-Architected lens models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IsolationPosture {
    /// Silo: dedicated infrastructure per tenant.
    Siloed,
    /// Pool: shared infrastructure with row/namespace-level isolation.
    Pooled,
    /// Bridge: silo for stateful tiers, pool for stateless tiers.
    Bridged,
}

/// A tenant: the FD-001 unit of isolation, billing, and residency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tenant {
    pub tenant_id: String,                   // data_class: TENANT_SCOPED
    pub display_name: String,                // data_class: INTERNAL_ONLY
    pub state: TenantLifecycleState,         // data_class: INTERNAL_ONLY
    pub isolation_posture: IsolationPosture, // data_class: INTERNAL_ONLY
    /// The cell this tenant is pinned to (cell-based architecture: one
    /// tenant, one home cell; migrations are explicit control-plane moves).
    pub cell_id: String, // data_class: INTERNAL_ONLY
    /// Data-residency zone constraint, when the tenant has one.
    pub residency_zone: Option<String>, // data_class: INTERNAL_ONLY
}

impl Tenant {
    /// Surface-all invariant check.
    pub fn validate(&self) -> Result<(), Vec<ContractViolation>> {
        let mut out = Vec::new();
        check_slug("tenant.tenant_id", &self.tenant_id, MAX_ID_LEN, &mut out);
        check_text(
            "tenant.display_name",
            &self.display_name,
            MAX_DISPLAY_NAME_LEN,
            &mut out,
        );
        check_slug("tenant.cell_id", &self.cell_id, MAX_ID_LEN, &mut out);
        if let Some(zone) = &self.residency_zone {
            check_slug("tenant.residency_zone", zone, MAX_ID_LEN, &mut out);
        }
        if out.is_empty() { Ok(()) } else { Err(out) }
    }

    /// Apply a lifecycle operation, returning the transitioned tenant.
    pub fn apply_operation(
        mut self,
        operation: TenantLifecycleOperation,
    ) -> Result<Self, ContractViolation> {
        self.state = operation.apply(self.state)?;
        Ok(self)
    }
}

/// A tenant-scoped resource record: the unit PDP decisions and cell routing
/// are made about.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TenantResource {
    /// Relative resource name, AIP-122 shape `collection/resource-id`.
    pub resource_name: String, // data_class: TENANT_SCOPED
    pub tenant_id: String,     // data_class: TENANT_SCOPED
    pub resource_kind: String, // data_class: INTERNAL_ONLY
    /// Data classification of the payload this record points at.
    pub data_class: String,
    /// The cell the resource physically lives in. MUST equal the owning
    /// tenant's cell (structural isolation invariant).
    pub cell_id: String, // data_class: INTERNAL_ONLY
}

impl TenantResource {
    /// Surface-all invariant check.
    pub fn validate(&self) -> Result<(), Vec<ContractViolation>> {
        let mut out = Vec::new();
        let mut segments = self.resource_name.split('/');
        let collection = segments.next().unwrap_or_default();
        let resource_id = segments.next().unwrap_or_default();
        if collection.is_empty() || resource_id.is_empty() || segments.next().is_some() {
            out.push(ContractViolation::InvalidShape {
                field: "tenant_resource.resource_name",
                detail: "resource name must be `collection/resource-id`".to_owned(),
            });
        } else {
            check_slug(
                "tenant_resource.resource_name",
                collection,
                MAX_ID_LEN,
                &mut out,
            );
            check_slug(
                "tenant_resource.resource_name",
                resource_id,
                MAX_ID_LEN,
                &mut out,
            );
        }
        check_slug(
            "tenant_resource.tenant_id",
            &self.tenant_id,
            MAX_ID_LEN,
            &mut out,
        );
        check_slug(
            "tenant_resource.resource_kind",
            &self.resource_kind,
            MAX_ID_LEN,
            &mut out,
        );
        check_slug(
            "tenant_resource.data_class",
            &self.data_class,
            MAX_ID_LEN,
            &mut out,
        );
        check_slug(
            "tenant_resource.cell_id",
            &self.cell_id,
            MAX_ID_LEN,
            &mut out,
        );
        if out.is_empty() { Ok(()) } else { Err(out) }
    }
}

/// The structural cell/tenant isolation invariant, Rust-side mirror of the
/// Cedar `forbid` in `cedar/platform-policies.cedar`: a resource belongs to
/// its owning tenant AND lives in that tenant's cell. Surface-all.
pub fn check_resource_isolation(
    tenant: &Tenant,
    resource: &TenantResource,
) -> Result<(), Vec<ContractViolation>> {
    let mut out = Vec::new();
    if tenant.tenant_id != resource.tenant_id {
        out.push(ContractViolation::BrokenReference {
            field: "tenant_resource.tenant_id",
            detail: format!(
                "resource {} is owned by tenant {:?}, not {:?}",
                resource.resource_name, resource.tenant_id, tenant.tenant_id
            ),
        });
    }
    if tenant.cell_id != resource.cell_id {
        out.push(ContractViolation::BrokenReference {
            field: "tenant_resource.cell_id",
            detail: format!(
                "resource {} lives in cell {:?} but tenant {} is pinned to cell {:?}",
                resource.resource_name, resource.cell_id, tenant.tenant_id, tenant.cell_id
            ),
        });
    }
    if out.is_empty() { Ok(()) } else { Err(out) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant() -> Tenant {
        Tenant {
            tenant_id: "acme".to_owned(),
            display_name: "Acme Corp".to_owned(),
            state: TenantLifecycleState::Active,
            isolation_posture: IsolationPosture::Bridged,
            cell_id: "cell-001".to_owned(),
            residency_zone: Some("kr-seoul".to_owned()),
        }
    }

    fn resource() -> TenantResource {
        TenantResource {
            resource_name: "documents/doc-1".to_owned(),
            tenant_id: "acme".to_owned(),
            resource_kind: "document".to_owned(),
            data_class: "restricted".to_owned(),
            cell_id: "cell-001".to_owned(),
        }
    }

    #[test]
    fn valid_tenant_passes_and_round_trips() {
        let t = tenant();
        t.validate().unwrap();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(serde_json::from_str::<Tenant>(&json).unwrap(), t);
    }

    #[test]
    fn tenant_closed_schema_rejects_unknown_fields() {
        let mut value = serde_json::to_value(tenant()).unwrap();
        value["surprise"] = serde_json::json!("x");
        assert!(serde_json::from_value::<Tenant>(value).is_err());
    }

    #[test]
    fn lifecycle_happy_path_create_activate_suspend_resume_retire() {
        use TenantLifecycleOperation as Op;
        let t = Tenant {
            state: TenantLifecycleState::initial(),
            ..tenant()
        };
        let t = t.apply_operation(Op::Activate).unwrap();
        assert!(t.state.is_operational());
        let t = t.apply_operation(Op::Suspend).unwrap();
        assert_eq!(t.state, TenantLifecycleState::Suspended);
        let t = t.apply_operation(Op::Resume).unwrap();
        let t = t.apply_operation(Op::Retire).unwrap();
        assert!(t.state.is_terminal());
    }

    #[test]
    fn retired_is_terminal_for_every_operation() {
        use TenantLifecycleOperation as Op;
        for operation in [Op::Activate, Op::Suspend, Op::Resume, Op::Retire] {
            let err = operation.apply(TenantLifecycleState::Retired).unwrap_err();
            assert!(
                matches!(
                    err,
                    ContractViolation::InvalidTransition {
                        from: "retired",
                        ..
                    }
                ),
                "{operation:?}: {err}"
            );
        }
    }

    #[test]
    fn forbidden_transitions_are_rejected() {
        use TenantLifecycleOperation as Op;
        use TenantLifecycleState as S;
        for (from, operation) in [
            (S::Provisioning, Op::Suspend),
            (S::Provisioning, Op::Resume),
            (S::Active, Op::Activate),
            (S::Active, Op::Resume),
            (S::Suspended, Op::Suspend),
            (S::Suspended, Op::Activate),
        ] {
            assert!(operation.apply(from).is_err(), "{from:?} -{operation:?}");
        }
    }

    #[test]
    fn resource_name_must_be_collection_slash_id() {
        for bad in [
            "",
            "documents",
            "documents/doc-1/extra",
            "/doc-1",
            "documents/",
        ] {
            let r = TenantResource {
                resource_name: bad.to_owned(),
                ..resource()
            };
            assert!(r.validate().is_err(), "{bad:?} must be rejected");
        }
        resource().validate().unwrap();
    }

    #[test]
    fn isolation_invariant_flags_cross_tenant_and_cross_cell() {
        let t = tenant();
        check_resource_isolation(&t, &resource()).unwrap();
        let foreign = TenantResource {
            tenant_id: "globex".to_owned(),
            cell_id: "cell-002".to_owned(),
            ..resource()
        };
        let violations = check_resource_isolation(&t, &foreign).unwrap_err();
        assert_eq!(violations.len(), 2, "surface-all: {violations:?}");
    }
}
