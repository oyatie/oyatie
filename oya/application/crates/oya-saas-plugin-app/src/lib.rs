//! SaaS plugin runtime — Wasmtime-sandboxed invocation contract.
//!
//! Final-shape contract surface for `plugin.invocation` per M03-P04-IP-002.
//! The actual Wasmtime execution lives behind an adapter (added in a later
//! IP); this preview crate owns:
//! * [`PluginInvocation`] — request/response audit contract,
//! * [`PluginContext`] — per-call tenant + regional-pack envelope,
//! * a deterministic recorded-invocation registry the bench harness exercises.
//!
//! ADR-0023: plugin invocations carry tenant context + Cosign-signed manifest
//! reference; the runtime refuses to dispatch against unregistered manifests.
//!
//! No external Rust deps — std + workspace path deps only per ADR-0015.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

use marketplace_plugin_kernel::{
    MarketplaceError, MarketplaceRegistry, PluginManifestId, TrustTier,
};

const INVOCATION_ID_PREFIX: &str = "inv_";
const PLUGIN_INVOCATION_SCHEMA_VERSION: u32 = 1;

/// Errors raised by the plugin runtime.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PluginRuntimeError {
    InvalidInvocationId,
    InvalidTenantId,
    InvalidRegionalPack,
    UnknownManifest,
    DuplicateInvocation,
    UnsafeTrustTier,
    PayloadTooLarge,
    Marketplace(MarketplaceErrorKind),
}

/// Mirror of [`MarketplaceError`] kinds the runtime can surface.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MarketplaceErrorKind {
    InvalidManifestId,
    UnknownManifest,
    Other,
}

impl From<MarketplaceError> for PluginRuntimeError {
    fn from(value: MarketplaceError) -> Self {
        let kind = match value {
            MarketplaceError::InvalidManifestId => MarketplaceErrorKind::InvalidManifestId,
            MarketplaceError::UnknownManifest => MarketplaceErrorKind::UnknownManifest,
            _ => MarketplaceErrorKind::Other,
        };
        Self::Marketplace(kind)
    }
}

const MAX_PAYLOAD_BYTES: usize = 64 * 1024;

/// Plugin invocation identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PluginInvocationId {
    pub value: String, // data_class: INTERNAL_ONLY
}

impl PluginInvocationId {
    pub fn new(value: impl Into<String>) -> Result<Self, PluginRuntimeError> {
        let value = value.into();
        if value.starts_with(INVOCATION_ID_PREFIX) && value.len() > INVOCATION_ID_PREFIX.len() {
            Ok(Self { value })
        } else {
            Err(PluginRuntimeError::InvalidInvocationId)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// Outcome of a sandboxed plugin invocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PluginInvocationOutcome {
    Succeeded,
    Failed,
    Timeout,
    SandboxDenied,
}

/// Per-call envelope passed to the Wasmtime sandbox.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginContext {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub regional_pack: String,           // data_class: INTERNAL_ONLY
    pub workflow_run_id: Option<String>, // data_class: INTERNAL_ONLY
    pub deadline_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
    pub allowed_trust_tier: TrustTier,   // data_class: INTERNAL_ONLY
}

impl PluginContext {
    pub fn new(
        tenant_id: impl Into<String>,
        regional_pack: impl Into<String>,
        workflow_run_id: Option<String>,
        deadline_epoch_seconds: u64,
        allowed_trust_tier: TrustTier,
    ) -> Result<Self, PluginRuntimeError> {
        let tenant_id = tenant_id.into();
        if !is_tenant(&tenant_id) {
            return Err(PluginRuntimeError::InvalidTenantId);
        }
        let regional_pack = regional_pack.into();
        if !is_regional_pack(&regional_pack) {
            return Err(PluginRuntimeError::InvalidRegionalPack);
        }
        Ok(Self {
            tenant_id,
            regional_pack,
            workflow_run_id,
            deadline_epoch_seconds,
            allowed_trust_tier,
        })
    }
}

/// Recorded plugin invocation audit row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginInvocation {
    pub id: PluginInvocationId,           // data_class: INTERNAL_ONLY
    pub manifest_id: PluginManifestId,    // data_class: INTERNAL_ONLY
    pub context: PluginContext,           // data_class: INTERNAL_ONLY
    pub payload_bytes: Vec<u8>,           // data_class: INTERNAL_ONLY
    pub outcome: PluginInvocationOutcome, // data_class: INTERNAL_ONLY
    pub started_at_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
    pub finished_at_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
    pub schema_version: u32,              // data_class: INTERNAL_ONLY
}

/// Inputs to `plugin.invocation`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginInvoke {
    pub id: String,                       // data_class: INTERNAL_ONLY
    pub manifest_id: String,              // data_class: INTERNAL_ONLY
    pub context: PluginContext,           // data_class: INTERNAL_ONLY
    pub payload_bytes: Vec<u8>,           // data_class: INTERNAL_ONLY
    pub started_at_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
    pub finished_at_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
    pub outcome: PluginInvocationOutcome, // data_class: INTERNAL_ONLY
}

/// Plugin runtime — wraps a marketplace registry + audit ledger.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PluginRuntime {
    invocations: BTreeMap<PluginInvocationId, PluginInvocation>,
}

impl PluginRuntime {
    /// `plugin.invocation` — sandboxed dispatch (recorded only at preview tier).
    pub fn invoke(
        &mut self,
        registry: &MarketplaceRegistry,
        input: PluginInvoke,
    ) -> Result<PluginInvocation, PluginRuntimeError> {
        let id = PluginInvocationId::new(input.id)?;
        if self.invocations.contains_key(&id) {
            return Err(PluginRuntimeError::DuplicateInvocation);
        }
        let manifest_id = PluginManifestId::new(input.manifest_id)?;
        let manifest = registry
            .manifest(&manifest_id)
            .ok_or(PluginRuntimeError::UnknownManifest)?;
        if input.payload_bytes.len() > MAX_PAYLOAD_BYTES {
            return Err(PluginRuntimeError::PayloadTooLarge);
        }
        // ADR-0036 — community-tier plugins refuse to run in regions/tenants
        // that requested a verified-only context.
        if input.context.allowed_trust_tier == TrustTier::Verified
            && manifest.cosign_signature.value.is_empty()
        {
            return Err(PluginRuntimeError::UnsafeTrustTier);
        }
        let invocation = PluginInvocation {
            id: id.clone(),
            manifest_id,
            context: input.context,
            payload_bytes: input.payload_bytes,
            outcome: input.outcome,
            started_at_epoch_seconds: input.started_at_epoch_seconds,
            finished_at_epoch_seconds: input.finished_at_epoch_seconds,
            schema_version: PLUGIN_INVOCATION_SCHEMA_VERSION,
        };
        self.invocations.insert(id, invocation.clone());
        Ok(invocation)
    }

    pub fn invocation(&self, id: &PluginInvocationId) -> Option<&PluginInvocation> {
        self.invocations.get(id)
    }

    pub fn invocations(&self) -> impl Iterator<Item = &PluginInvocation> {
        self.invocations.values()
    }
}

fn is_tenant(value: &str) -> bool {
    value.starts_with("ten_") && value.len() > "ten_".len()
}

fn is_regional_pack(value: &str) -> bool {
    value.starts_with("oya-pack-") && value.len() > "oya-pack-".len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use marketplace_plugin_kernel::{PluginManifestRegister, Vertical};

    fn registered_registry() -> MarketplaceRegistry {
        let mut reg = MarketplaceRegistry::default();
        reg.register_manifest(PluginManifestRegister {
            id: "plg_sum_v1".to_string(),
            publisher_id: "pub_acme".to_string(),
            name: "Summarizer".to_string(),
            semver: "1.2.3".to_string(),
            cosign_signature: "cosign:sha256:abc".to_string(),
            entrypoint: "wasm/summarizer.wasm".to_string(),
            registered_at_epoch_seconds: 1_700_000_000,
        })
        .expect("manifest registered");
        // Verticals only consulted by listings; runtime uses manifest directly.
        let _ = Vertical::Agentic;
        reg
    }

    fn context() -> PluginContext {
        PluginContext::new(
            "ten_acme",
            "oya-pack-alpha",
            Some("wfr_001".to_string()),
            1_700_000_999,
            TrustTier::Reviewed,
        )
        .expect("valid context")
    }

    fn invoke_input(id: &str, manifest_id: &str) -> PluginInvoke {
        PluginInvoke {
            id: id.to_string(),
            manifest_id: manifest_id.to_string(),
            context: context(),
            payload_bytes: b"{\"q\":\"summarize\"}".to_vec(),
            started_at_epoch_seconds: 1_700_000_100,
            finished_at_epoch_seconds: 1_700_000_101,
            outcome: PluginInvocationOutcome::Succeeded,
        }
    }

    #[test]
    fn invoke_records_against_registered_manifest() {
        let registry = registered_registry();
        let mut runtime = PluginRuntime::default();
        let inv = runtime
            .invoke(&registry, invoke_input("inv_001", "plg_sum_v1"))
            .expect("invocation recorded");
        assert_eq!(inv.outcome, PluginInvocationOutcome::Succeeded);
        assert_eq!(runtime.invocations().count(), 1);
    }

    #[test]
    fn invoke_rejects_unknown_manifest_or_duplicate_id() {
        let registry = registered_registry();
        let mut runtime = PluginRuntime::default();
        let unknown = runtime
            .invoke(&registry, invoke_input("inv_unk", "plg_ghost"))
            .expect_err("unknown manifest rejected");
        assert_eq!(unknown, PluginRuntimeError::UnknownManifest);

        runtime
            .invoke(&registry, invoke_input("inv_dup", "plg_sum_v1"))
            .unwrap();
        let dup = runtime
            .invoke(&registry, invoke_input("inv_dup", "plg_sum_v1"))
            .expect_err("duplicate invocation rejected");
        assert_eq!(dup, PluginRuntimeError::DuplicateInvocation);
    }

    #[test]
    fn invoke_rejects_invalid_ids_tenant_and_pack() {
        let registry = registered_registry();
        let mut runtime = PluginRuntime::default();
        let bad_id = runtime
            .invoke(&registry, invoke_input("nope", "plg_sum_v1"))
            .expect_err("invocation id prefix enforced");
        assert_eq!(bad_id, PluginRuntimeError::InvalidInvocationId);

        let bad_tenant = PluginContext::new(
            "acme",
            "oya-pack-alpha",
            None,
            1_700_000_999,
            TrustTier::Reviewed,
        )
        .expect_err("tenant id prefix enforced");
        assert_eq!(bad_tenant, PluginRuntimeError::InvalidTenantId);

        let bad_pack =
            PluginContext::new("ten_acme", "kr", None, 1_700_000_999, TrustTier::Reviewed)
                .expect_err("regional pack prefix enforced");
        assert_eq!(bad_pack, PluginRuntimeError::InvalidRegionalPack);
    }

    #[test]
    fn payload_size_bounded_for_sandbox_safety() {
        let registry = registered_registry();
        let mut runtime = PluginRuntime::default();
        let too_big = runtime
            .invoke(
                &registry,
                PluginInvoke {
                    payload_bytes: vec![0u8; MAX_PAYLOAD_BYTES + 1],
                    ..invoke_input("inv_big", "plg_sum_v1")
                },
            )
            .expect_err("payload bounded");
        assert_eq!(too_big, PluginRuntimeError::PayloadTooLarge);
    }

    #[test]
    fn invocation_lookup_returns_recorded_row() {
        let registry = registered_registry();
        let mut runtime = PluginRuntime::default();
        let recorded = runtime
            .invoke(&registry, invoke_input("inv_lookup", "plg_sum_v1"))
            .unwrap();
        let fetched = runtime
            .invocation(&recorded.id)
            .expect("invocation present");
        assert_eq!(fetched.outcome, PluginInvocationOutcome::Succeeded);
        assert_eq!(fetched.manifest_id.as_str(), "plg_sum_v1");
    }
}
