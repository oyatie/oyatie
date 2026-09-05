use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use shared_platform_contracts_kernel::ContractViolation;
use shared_platform_contracts_kernel::pdp::EntityRef;

/// One entity in the per-request PIP slice: its typed uid, attribute map
/// (deterministic order), and parent edges (group membership, tenant
/// containment).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityRecord {
    pub uid: EntityRef, // data_class: TENANT_SCOPED
    /// Attribute map exposed to ABAC conditions (deterministic order).
    pub attributes: BTreeMap<String, serde_json::Value>, // data_class: TENANT_SCOPED
    /// Parent entity edges (e.g. Principal -> Group, Group -> Tenant).
    pub parents: Vec<EntityRef>, // data_class: TENANT_SCOPED
}

/// The entity slice a PEP assembles for one authorization request. The PDP
/// evaluates against EXACTLY this slice — it never reaches out to a PIP at
/// decision time (embedded-PDP doctrine: no network hop on the request path).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntitySlice {
    pub entities: Vec<EntityRecord>, // data_class: TENANT_SCOPED
}

impl EntitySlice {
    /// Surface-all invariant check: every uid is well-formed and no uid
    /// appears twice (a duplicate would make attribute resolution ambiguous).
    pub fn validate(&self) -> Result<(), Vec<ContractViolation>> {
        let mut out = Vec::new();
        let mut seen: Vec<&EntityRef> = Vec::new();
        for record in &self.entities {
            if record.uid.entity_type.is_empty() || record.uid.entity_id.is_empty() {
                out.push(ContractViolation::MissingValue {
                    field: "entity_slice.entities.uid",
                });
            }
            if seen.contains(&&record.uid) {
                out.push(ContractViolation::BrokenReference {
                    field: "entity_slice.entities",
                    detail: format!(
                        "duplicate entity uid {}::{}",
                        record.uid.entity_type, record.uid.entity_id
                    ),
                });
            }
            seen.push(&record.uid);
        }
        if out.is_empty() { Ok(()) } else { Err(out) }
    }
}
