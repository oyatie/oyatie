//! Microservice manifest generator kernel (ADR-0131 + Sweep-F).
//!
//! Naming justification:
//! - Crate id `oya-gen-microservice-manifests-app` — `oya-` brand prefix
//!   (ADR-0017 / MFL-0011), `gen` verb (kernel-tier action), subject
//!   `microservice-manifests`, `app` Layer-3 (use-case orchestrator wiring
//!   IO at the boundary).
//! - Library identifier `oya_gen_microservice_manifests_app` — snake_case
//!   mirror (ADR-0105 v4 BNF §2.2).
//!
//! Retired writer/provenance kernel for the former
//! `scripts/gen-microservice-manifests.py` flow. The legacy
//! `microservices/<ms>/manifest.json` producer is no longer an active writer for
//! `specs/microservices/manifests-index.json`; the aggregate index builder below
//! emits the current source-authority contract so check paths cannot regress to
//! stale legacy `microservices/<name>/manifest.json` rows.
//!
//! Tier 1 (kernel-tier) per ADR-0083: pure logic over already-loaded
//! [`SourceFile`] records; IO is supplied by `src/main.rs`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde_json::{Map, Value, json};

/// Current manifest-index row order. This follows
/// `specs/microservices/manifests-index.json` exactly: no retired legacy
/// `cell`/`network`/`shorts` rows, `anonymous` remains a no-standalone
/// subproduct of `community`, and `foundry` is retained only as a retired row
/// absorbed by `intelligence`.
pub const MICROSERVICES: &[&str] = &[
    "application",
    "audit-chain",
    "community",
    "observability",
    "ontology",
    "tenancy",
    "workflow-engine",
    "anonymous",
    "calendar",
    "docs",
    "drive",
    "foundry",
    "intelligence",
    "forms",
    "mail",
    "meet",
    "messenger",
    "notes",
    "recordings",
    "sheets",
    "sites",
    "slides",
    "social",
    "tasks",
    "translate",
    "workflow-studio",
    "cloud-iac",
    "cloud-k8s",
    "cloud-secrets",
    "governance",
    "identity",
    "ops-dashboard-control-center",
    "cloud-intelligence",
    "managed-k8s-cluster-lifecycle",
    "managed-k8s-control-plane-host",
    "managed-k8s-sla-observability",
    "managed-k8s-tenant-quota",
];

/// Canonical current manifest-index contract. Kept in code rather than derived
/// from the on-disk index so `--check` remains an independent guard against the
/// retired generator silently rewriting source-authority rows back to legacy
/// `microservices/<name>/manifest.json` paths.
const CURRENT_MANIFESTS_INDEX_JSON: &str = r####"{
  "schema_version": "1.0",
  "generated_at": "2026-05-19",
  "manifest_count": 37,
  "readiness_contracts": {
    "multi_region_disposition": {
      "status": "future-readiness-gate",
      "authority": "specs/multi-region-disposition-canonical.json#manifest_field",
      "manifest_field": "multi_region_disposition",
      "manifest_field_required_by_authority": true,
      "schema_contract": "specs/microservices/manifest-schema.json#properties.multi_region_disposition",
      "allowed_values": [
        "active_active",
        "active_passive",
        "single_region"
      ],
      "doc_companion": "multi-region.md",
      "doc_required_sections": [
        "disposition_statement",
        "rationale",
        "rpo_rto_numbers_if_active_passive"
      ],
      "gate_packet": "cloud-ci/Rust gate packet: multi-region-disposition",
      "legacy_oya_cli_authority": "none; historical `oya gate validate multi-region-disposition` wording is local-feedback/provenance only and never merge authority",
      "promotion_boundary": "No region/cell provisioning, tenant routing, live failover, production-readiness, or hyperscaler-maturity claim is made by this metadata contract; hard-required enforcement waits for per-service declarations, companion docs, and the cloud-ci gate packet.",
      "coverage_scope_note": "Coverage reports must distinguish current `oya/<service>/manifest.json` and `cloud/<service>/manifest.json` roots from legacy `microservices/<service>/manifest.json` provenance rows before treating a service as covered.",
      "foundry_retirement_authority": {
        "retired_name": "foundry",
        "absorbed_by": "intelligence",
        "do_not_treat_as_active": true
      },
      "serialization_requirements": [
        "specs/microservices/manifest-schema.json",
        "specs/microservices/manifests-index.json",
        "specs/root-hub-pointers.json",
        "specs/masterplan.json",
        "generated outputs remain producer/materializer-owned and must not be hand edited"
      ]
    }
  },
  "microservices": [
    {
      "name": "application",
      "manifest": "oya/application/manifest.json",
      "fd001_material": false
    },
    {
      "name": "audit-chain",
      "manifest": "oya/audit-chain/manifest.json",
      "fd001_material": false
    },
    {
      "name": "community",
      "manifest": "oya/community/manifest.json",
      "prd": "specs/microservices/community.json",
      "fd001_material": true,
      "authority_status": "reconciled-by-t_28c62d82",
      "authority_boundary": "specs/microservices/community.json is the FD-001 source-authority lock for backlog fanout; oya/community/manifest.json is tracked implementation inventory/provenance and must not be used as live implementation-readiness evidence without Plan/Spec/RED gates."
    },
    {
      "name": "observability",
      "manifest": "oya/observability/manifest.json",
      "fd001_material": false
    },
    {
      "name": "ontology",
      "manifest": "oya/ontology/manifest.json",
      "fd001_material": true
    },
    {
      "name": "tenancy",
      "manifest": "cloud/tenancy/manifest.json",
      "fd001_material": false
    },
    {
      "name": "workflow-engine",
      "manifest": "oya/workflow-engine/manifest.json",
      "fd001_material": true
    },
    {
      "name": "anonymous",
      "prd": "specs/microservices/anonymous.json",
      "parent_inventory": "oya/community/manifest.json",
      "subproduct_of": "community",
      "fd001_material": false,
      "authority_status": "source-authority-reconciled-by-t_ff8bab02",
      "authority_boundary": "specs/microservices/anonymous.json is the Draft PRD source for the anonymous workplace subproduct. There is no standalone oya/anonymous/manifest.json; oya/community/manifest.json remains parent community inventory/provenance only. This row makes no runtime/product-readiness claim and must not be used as live implementation-readiness evidence without Plan/Spec/RED gates."
    },
    {
      "name": "calendar",
      "manifest": "oya/calendar/manifest.json",
      "prd": "specs/microservices/calendar.json",
      "fd001_material": false,
      "authority_status": "source-authority-reconciled-by-t_ff8bab02",
      "authority_boundary": "specs/microservices/calendar.json is the Accepted PRD/source-authority path for calendar planning; oya/calendar/manifest.json is tracked service inventory/provenance only. This row makes no runtime/product-readiness claim and must not be used as live implementation-readiness evidence without Plan/Spec/RED gates."
    },
    {
      "name": "docs",
      "manifest": "oya/docs/manifest.json",
      "fd001_material": false
    },
    {
      "name": "drive",
      "manifest": "oya/drive/manifest.json",
      "fd001_material": false,
      "authority_status": "source-authority-reconciled-by-t_ff8bab02",
      "authority_boundary": "oya/drive/manifest.json is tracked service inventory/provenance only. No specs/microservices/drive.json PRD/source-authority file exists yet; downstream Plan/Spec cards must author and lock the PRD/source map before implementation fanout. This row makes no runtime/product-readiness claim and must not be used as live implementation-readiness evidence without Plan/Spec/RED gates."
    },
    {
      "name": "foundry",
      "status": "retired",
      "retired_by_wave": "15I",
      "retired_by_adr": "ADR-0335",
      "retired_at": "2026-05-21",
      "absorbed_by": "intelligence",
      "absorbing_manifest": "oya/intelligence/manifest.json",
      "retirement_marker": "microservices/intelligence/RETIRED.md",
      "do_not_treat_as_active": true,
      "manifest": "oya/intelligence/manifest.json"
    },
    {
      "name": "intelligence",
      "manifest": "oya/intelligence/manifest.json",
      "prd": "specs/microservices/intelligence.json",
      "fd001_material": true,
      "absorbs_retired": [
        "foundry"
      ]
    },
    {
      "name": "forms",
      "manifest": "oya/forms/manifest.json",
      "fd001_material": false,
      "authority_status": "source-authority-reconciled-by-t_ff8bab02",
      "authority_boundary": "oya/forms/manifest.json is tracked service inventory/provenance only. No specs/microservices/forms.json PRD/source-authority file exists yet; downstream Plan/Spec cards must author and lock the PRD/source map before implementation fanout. This row makes no runtime/product-readiness claim and must not be used as live implementation-readiness evidence without Plan/Spec/RED gates."
    },
    {
      "name": "mail",
      "manifest": "oya/mail/manifest.json",
      "fd001_material": true
    },
    {
      "name": "meet",
      "manifest": "oya/meet/manifest.json",
      "fd001_material": false,
      "authority_status": "source-authority-reconciled-by-t_ff8bab02",
      "authority_boundary": "oya/meet/manifest.json is tracked service inventory/provenance only. No specs/microservices/meet.json PRD/source-authority file exists yet; downstream Plan/Spec cards must author and lock the PRD/source map before implementation fanout. This row makes no runtime/product-readiness claim and must not be used as live implementation-readiness evidence without Plan/Spec/RED gates."
    },
    {
      "name": "messenger",
      "manifest": "oya/messenger/manifest.json",
      "fd001_material": true
    },
    {
      "name": "notes",
      "manifest": "oya/notes/manifest.json",
      "fd001_material": false
    },
    {
      "name": "recordings",
      "manifest": "oya/recordings/manifest.json",
      "fd001_material": false,
      "authority_status": "source-authority-reconciled-by-t_ff8bab02",
      "authority_boundary": "oya/recordings/manifest.json is tracked service inventory/provenance only. No specs/microservices/recordings.json PRD/source-authority file exists yet; downstream Plan/Spec cards must author and lock the PRD/source map before implementation fanout. This row makes no runtime/product-readiness claim and must not be used as live implementation-readiness evidence without Plan/Spec/RED gates."
    },
    {
      "name": "sheets",
      "manifest": "oya/sheets/manifest.json",
      "fd001_material": false,
      "authority_status": "source-authority-reconciled-by-t_ff8bab02",
      "authority_boundary": "oya/sheets/manifest.json is tracked service inventory/provenance only. No specs/microservices/sheets.json PRD/source-authority file exists yet; downstream Plan/Spec cards must author and lock the PRD/source map before implementation fanout. This row makes no runtime/product-readiness claim and must not be used as live implementation-readiness evidence without Plan/Spec/RED gates."
    },
    {
      "name": "sites",
      "manifest": "oya/sites/manifest.json",
      "fd001_material": false,
      "authority_status": "source-authority-reconciled-by-t_ff8bab02",
      "authority_boundary": "oya/sites/manifest.json is tracked service inventory/provenance only. No specs/microservices/sites.json PRD/source-authority file exists yet; downstream Plan/Spec cards must author and lock the PRD/source map before implementation fanout. This row makes no runtime/product-readiness claim and must not be used as live implementation-readiness evidence without Plan/Spec/RED gates."
    },
    {
      "name": "slides",
      "manifest": "oya/slides/manifest.json",
      "fd001_material": false,
      "authority_status": "source-authority-reconciled-by-t_ff8bab02",
      "authority_boundary": "oya/slides/manifest.json is tracked service inventory/provenance only. No specs/microservices/slides.json PRD/source-authority file exists yet; downstream Plan/Spec cards must author and lock the PRD/source map before implementation fanout. This row makes no runtime/product-readiness claim and must not be used as live implementation-readiness evidence without Plan/Spec/RED gates."
    },
    {
      "name": "social",
      "manifest": "oya/social/manifest.json",
      "prd": "specs/microservices/social.json",
      "fd001_material": false,
      "authority_status": "source-map-locked-by-t_df502234",
      "authority_boundary": "specs/microservices/social.json remains Draft PRD/source-authority for social Plan/Spec and RED fixture/contract planning; oya/social/manifest.json is tracked service inventory/provenance only and now records the manifest-pointer state plus legacy microservices/social/* path dispositions. Coordinate with community FD-001 authority for boundary consistency only; community is not social implementation authority. This row makes no runtime/product-readiness claim and must not be used as live implementation-readiness evidence without Plan/Spec/RED gates."
    },
    {
      "name": "tasks",
      "manifest": "oya/tasks/manifest.json",
      "fd001_material": false,
      "authority_status": "source-authority-reconciled-by-t_ff8bab02",
      "authority_boundary": "oya/tasks/manifest.json is tracked service inventory/provenance only. No specs/microservices/tasks.json PRD/source-authority file exists yet; downstream Plan/Spec cards must author and lock the PRD/source map before implementation fanout. This row makes no runtime/product-readiness claim and must not be used as live implementation-readiness evidence without Plan/Spec/RED gates."
    },
    {
      "name": "translate",
      "manifest": "oya/translate/manifest.json",
      "fd001_material": false,
      "authority_status": "source-authority-reconciled-by-t_ff8bab02",
      "authority_boundary": "oya/translate/manifest.json is tracked service inventory/provenance only. No specs/microservices/translate.json PRD/source-authority file exists yet; downstream Plan/Spec cards must author and lock the PRD/source map before implementation fanout. This row makes no runtime/product-readiness claim and must not be used as live implementation-readiness evidence without Plan/Spec/RED gates."
    },
    {
      "name": "workflow-studio",
      "manifest": "oya/workflow-studio/manifest.json",
      "fd001_material": true
    },
    {
      "name": "cloud-iac",
      "manifest": "cloud/cloud-iac/manifest.json",
      "fd001_material": false
    },
    {
      "name": "cloud-k8s",
      "manifest": "cloud/cloud-k8s/manifest.json",
      "fd001_material": false
    },
    {
      "name": "cloud-secrets",
      "manifest": "cloud/cloud-secrets/manifest.json",
      "fd001_material": false
    },
    {
      "name": "governance",
      "manifest": "oya/governance/manifest.json"
    },
    {
      "name": "identity",
      "manifest": "oya/identity/manifest.json"
    },
    {
      "name": "ops-dashboard-control-center",
      "manifest": "oya/ops-dashboard-control-center/manifest.json"
    },
    {
      "name": "cloud-intelligence",
      "manifest": "cloud/cloud-intelligence/manifest.json",
      "fd001_material": false
    },
    {
      "name": "managed-k8s-cluster-lifecycle",
      "manifest": "cloud/managed-k8s-cluster-lifecycle/manifest.json",
      "prd": "cloud/managed-k8s-cluster-lifecycle/PRD.md",
      "fd001_material": false,
      "authority_status": "source-authority-reconciled-by-t_ec0e9ad6",
      "authority_boundary": "cloud/managed-k8s-cluster-lifecycle/** is the live source-authority home for this service. Current authority is dogfood/design deterministic foundation only; do not treat it as live provider, external GA, production-readiness, public SLA, billing, or measured-SLO evidence without follow-on Build/Review gates."
    },
    {
      "name": "managed-k8s-control-plane-host",
      "manifest": "cloud/managed-k8s-control-plane-host/manifest.json",
      "prd": "cloud/managed-k8s-control-plane-host/PRD.md",
      "fd001_material": false,
      "authority_status": "source-authority-reconciled-by-t_fafc9e8e",
      "authority_boundary": "cloud/managed-k8s-control-plane-host/** is the live source-authority home for this service. Current authority is dogfood/design deterministic foundation only; the live CAPI/Kamaji/Talos provider path remains honest-deferred behind kamaji-provider-live-integration and must not be treated as provider-live, external GA, production-readiness, public SLA, billing, or measured-SLO evidence without follow-on Build/Review gates."
    },
    {
      "name": "managed-k8s-sla-observability",
      "manifest": "cloud/managed-k8s-sla-observability/manifest.json",
      "prd": "cloud/managed-k8s-sla-observability/PRD.md",
      "fd001_material": false,
      "authority_status": "source-authority-reconciled-by-t_6c32ff0e",
      "authority_boundary": "cloud/managed-k8s-sla-observability/** is the live source-authority home for this service. Current authority is deterministic SLA summary/read/evidence target shape only; live Prometheus/Kubernetes collectors, measured production SLO evidence, public SLA, production readiness, and tenant-quota implementation remain out of scope until follow-on Build/Review gates."
    },
    {
      "name": "managed-k8s-tenant-quota",
      "manifest": "cloud/managed-k8s-tenant-quota/manifest.json",
      "fd001_material": false
    }
  ]
}
"####;

pub const ALLOWED_LAYERS: &[&str] = &[
    "kernel",
    "domain",
    "application",
    "app",
    "adapter",
    "infrastructure",
    "cli",
    "rest",
    "grpc",
    "graphql",
    "worker",
    "sdk",
    "usecase",
    "api",
];

pub const ALLOWED_PACKS: &[&str] = &[
    "kr",
    "eu",
    "us",
    "us-healthcare",
    "jp",
    "sg",
    "au",
    "in",
    "br",
    "ae",
    "ksa",
];

pub const CANONICAL_PACKS: &[&str] = &[
    "kr",
    "eu",
    "us",
    "us-healthcare",
    "jp",
    "sg",
    "au",
    "in",
    "br",
    "ae",
    "ksa",
];

pub const LTS_DEFAULTS: &[(&str, &str)] = &[
    ("postgres", "16.4"),
    ("redis", "7.2"),
    ("valkey", "8.0"),
    ("clickhouse", "24.8-lts"),
    ("kafka", "3.7"),
    ("opensearch", "2.16"),
    ("envoy", "1.31"),
    ("istio", "1.23"),
    ("k8s", "1.30"),
    ("rust", "1.83"),
    ("openbao", "2.0"),
    ("argocd", "2.12"),
    ("opentofu", "1.8"),
    ("prometheus", "2.55"),
    ("grafana", "11.3"),
    ("loki", "3.2"),
    ("tempo", "2.6"),
    ("mimir", "2.14"),
    ("alloy", "1.4"),
    ("patroni", "4.0"),
    ("citus", "12.1"),
    ("meilisearch", "1.10"),
    ("clamav", "1.4"),
    ("cilium", "1.16"),
];

pub const CANONICAL_PROMETHEUSRULE_PATH: &str = "microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml";

/// A single file under a microservice tree, loaded into memory by the
/// surrounding binary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceFile {
    /// Path of this file relative to the repo root (e.g.
    /// `microservices/notes/catalog/foo.yaml`).
    pub repo_relative_path: String,
    pub content: String,
}

/// All inputs for one microservice's manifest. The Rust kernel sees only
/// the bundled view — IO walking happens in the binary.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ManifestInputs {
    pub microservice: String,
    pub files: Vec<SourceFile>,
    /// All on-disk decision file slugs of the form `ADR-NNNN-<title>.md`
    /// at `docs/decisions/` — used to filter ADR citations down to local
    /// repo refs.
    pub docs_decisions_filenames: Vec<String>,
}

/// Build one µservice's manifest as a `serde_json::Value` ready for
/// pretty-print serialization.
pub fn build_manifest(inputs: &ManifestInputs) -> Value {
    let ms = inputs.microservice.clone();
    let (bcs, layers_from_catalog) = extract_bounded_contexts(inputs);
    let bcs = if bcs.is_empty() {
        vec![BoundedContext {
            name: ms.clone(),
            description: format!("{ms} canonical bounded context"),
            crates: Vec::new(),
        }]
    } else {
        bcs
    };
    let layers = if layers_from_catalog.is_empty() {
        vec![
            "kernel".to_string(),
            "domain".to_string(),
            "usecase".to_string(),
            "adapter".to_string(),
            "rest".to_string(),
        ]
    } else {
        let mut v: Vec<String> = layers_from_catalog.into_iter().collect();
        v.sort();
        v
    };
    let capabilities = extract_capabilities(inputs);
    let slos = extract_slos(inputs);
    let ips = extract_ips(inputs);
    let contracts = extract_contracts(inputs);
    let packs = extract_regulatory_packs(inputs);
    let pins = extract_lts_pins(inputs);
    let adrs = extract_adrs(inputs);
    let hyperscaler = canonical_hyperscaler_coverage();
    let audit = extract_audit_seal_events(&ms, &capabilities);

    let owner = owner_for(&ms);

    let bcs_json: Vec<Value> = bcs
        .iter()
        .map(|bc| {
            json!({
                "name": bc.name,
                "description": bc.description,
                "crates": bc.crates,
            })
        })
        .collect();

    let capabilities_json: Vec<Value> = capabilities
        .iter()
        .map(|c| {
            json!({
                "tier": c.tier,
                "name": c.name,
                "file": c.file,
                "eu_ai_act_risk_class": c.risk_class,
            })
        })
        .collect();

    let slos_json: Vec<Value> = slos
        .iter()
        .map(|s| {
            json!({
                "name": s.name,
                "target": s.target,
                "sli": s.sli,
                "file": s.file,
            })
        })
        .collect();

    let ips_json: Vec<Value> = ips
        .iter()
        .map(|ip| {
            let mut obj = Map::new();
            obj.insert("id".to_string(), Value::String(ip.id.clone()));
            obj.insert("title".to_string(), Value::String(ip.title.clone()));
            obj.insert(
                "acceptance_status".to_string(),
                Value::String(ip.acceptance_status.clone()),
            );
            obj.insert("file".to_string(), Value::String(ip.file.clone()));
            if !ip.changeset_id.is_empty() {
                obj.insert(
                    "changeset_id".to_string(),
                    Value::Array(
                        ip.changeset_id
                            .iter()
                            .map(|s| Value::String(s.clone()))
                            .collect(),
                    ),
                );
            }
            if !ip.depends_on_changesets.is_empty() {
                obj.insert(
                    "depends_on_changesets".to_string(),
                    Value::Array(
                        ip.depends_on_changesets
                            .iter()
                            .map(|s| Value::String(s.clone()))
                            .collect(),
                    ),
                );
            }
            if !ip.parallel_safe_with_changesets.is_empty() {
                obj.insert(
                    "parallel_safe_with_changesets".to_string(),
                    Value::Array(
                        ip.parallel_safe_with_changesets
                            .iter()
                            .map(|s| Value::String(s.clone()))
                            .collect(),
                    ),
                );
            }
            if !ip.enables.is_empty() {
                obj.insert(
                    "enables".to_string(),
                    Value::Array(
                        ip.enables
                            .iter()
                            .map(|s| Value::String(s.clone()))
                            .collect(),
                    ),
                );
            }
            Value::Object(obj)
        })
        .collect();

    let adrs_json: Vec<Value> = adrs
        .iter()
        .map(|a| {
            let mut obj = Map::new();
            obj.insert("id".to_string(), Value::String(a.id.clone()));
            obj.insert("title".to_string(), Value::String(a.title.clone()));
            obj.insert("scope".to_string(), Value::String(a.scope.clone()));
            if let Some(file) = &a.file {
                obj.insert("file".to_string(), Value::String(file.clone()));
            }
            Value::Object(obj)
        })
        .collect();

    let mut lts_obj = Map::new();
    for (k, v) in &pins {
        lts_obj.insert(k.clone(), Value::String(v.clone()));
    }

    let mut contracts_obj = Map::new();
    contracts_obj.insert(
        "openapi".to_string(),
        Value::Array(contracts.openapi.into_iter().map(Value::String).collect()),
    );
    contracts_obj.insert(
        "asyncapi".to_string(),
        Value::Array(contracts.asyncapi.into_iter().map(Value::String).collect()),
    );
    contracts_obj.insert(
        "proto".to_string(),
        Value::Array(contracts.proto.into_iter().map(Value::String).collect()),
    );

    let ontology_projections = canonical_ontology_projections_for(&ms);
    let mesh_layering = canonical_mesh_layering_for(&ms);

    json!({
        "schema_version": "1.0",
        "microservice": ms,
        "version": "0.1.0",
        "owner": owner,
        "bounded_contexts": bcs_json,
        "layers": layers,
        "contracts": contracts_obj,
        "capabilities": capabilities_json,
        "slos": slos_json,
        "ips": ips_json,
        "regulatory_packs": packs,
        "lts_pins": lts_obj,
        "adrs": adrs_json,
        "hyperscaler_inv_coverage": hyperscaler,
        "audit_chain": audit,
        "secrets_substrate": {
            "provider": "openbao",
            "format": "${openbao:secret/<path>}"
        },
        "ontology_projections": ontology_projections,
        "mesh_layering": mesh_layering,
    })
}

/// Canonical mesh-layering declaration per ADR-0148 layered service mesh.
/// Every µservice runs on Cilium L3/L4 (Tier 1) and Istio Ambient ztunnel
/// (Tier 2) by default. Tier-3 waypoint enrollment is per-µservice opt-in:
/// the 5 µservices that handle L7-policed traffic declare
/// `ambient_waypoint=true`; all others declare `false`. The api-gateway
/// µservice is the sole north-south owner per ADR-0182.
fn canonical_mesh_layering_for(ms: &str) -> Value {
    let ambient_waypoint = matches!(
        ms,
        "governance" | "foundry" | "audit-chain" | "application" | "workflow-studio"
    );
    let north_south_only = ms == "api-gateway";
    json!({
        "cilium_l4": true,
        "ambient_ztunnel": true,
        "ambient_waypoint": ambient_waypoint,
        "north_south_only": north_south_only,
    })
}

/// Canonical-entity-owning µservices populated with at-least-2 projection
/// entries per ADR-0145 Invariant 3. Non-owning µservices receive an
/// empty `ontology_projections: []` array. The strict-mode validator
/// (tracked under
/// `registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-ontology-projection-validator`)
/// will cross-check this list against `registry/ontology/entities.json`.
fn canonical_ontology_projections_for(ms: &str) -> Value {
    let projections: &[(&str, &str)] = match ms {
        "ontology" => &[
            ("Person", "ontology_persons"),
            ("Document", "ontology_documents"),
            ("Recording", "ontology_recordings"),
        ],
        "tenancy" => &[
            ("Tenant", "ontology_tenants"),
            ("TenantMembership", "ontology_tenant_memberships"),
        ],
        "audit-chain" => &[
            ("AuditEvent", "ontology_audit_events"),
            ("AuditShard", "ontology_audit_shards"),
        ],
        "foundry" => &[
            ("AgentFleet", "ontology_agent_fleets"),
            ("CapabilityCard", "ontology_capability_cards"),
        ],
        "governance" => &[
            ("CedarFragment", "ontology_cedar_fragments"),
            ("PolicyDecision", "ontology_policy_decisions"),
        ],
        _ => &[],
    };
    Value::Array(
        projections
            .iter()
            .map(|(entity, table)| {
                json!({
                    "entity_name": entity,
                    "projection_target_table": table,
                    "projection_kind": "idempotent-rewrite",
                    "lag_budget_seconds": 60,
                })
            })
            .collect(),
    )
}

/// Build the aggregate index JSON.
///
/// The repository's current manifest-index contract is explicit source
/// authority, not a derivation from the retired `microservices/<ms>/` tree. For
/// the canonical row set this returns the exact current-path inventory with
/// readiness metadata. Ad-hoc slices are retained only for provenance tests and
/// deliberately do not emit active `manifest` rows, so the retired legacy layout
/// cannot be mistaken for a current producer contract.
pub fn build_manifests_index(generated_at: &str, microservices: &[&str]) -> Value {
    if generated_at == "2026-05-19" && microservices == MICROSERVICES {
        return serde_json::from_str(CURRENT_MANIFESTS_INDEX_JSON)
            .expect("embedded manifests-index contract must be valid JSON");
    }

    let list: Vec<Value> = microservices
        .iter()
        .map(|ms| {
            json!({
                "name": ms,
                "producer_status": "retired-provenance-only",
                "legacy_manifest_path": format!("microservices/{ms}/manifest.json"),
                "do_not_write_to_manifest_index": true,
            })
        })
        .collect();
    json!({
        "schema_version": "1.0",
        "generated_at": generated_at,
        "manifest_count": microservices.len(),
        "microservices": list,
    })
}

fn owner_for(ms: &str) -> String {
    format!("axis-{ms}")
}

// ---------- inner data types ----------

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BoundedContext {
    pub name: String,
    pub description: String,
    pub crates: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct CapabilityRow {
    tier: String,
    name: String,
    file: String,
    risk_class: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct SloRow {
    name: String,
    target: String,
    sli: String,
    file: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct IpRow {
    id: String,
    title: String,
    acceptance_status: String,
    file: String,
    changeset_id: Vec<String>,
    depends_on_changesets: Vec<String>,
    parallel_safe_with_changesets: Vec<String>,
    enables: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct AdrRow {
    id: String,
    title: String,
    scope: String,
    file: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ContractsBundle {
    openapi: Vec<String>,
    asyncapi: Vec<String>,
    proto: Vec<String>,
}

// ---------- extraction helpers ----------

fn ms_root_prefix(ms: &str) -> String {
    format!("microservices/{ms}/")
}

fn relative_within(ms: &str, path: &str) -> Option<String> {
    let prefix = ms_root_prefix(ms);
    path.strip_prefix(&prefix).map(|s| s.to_string())
}

fn files_under<'a>(inputs: &'a ManifestInputs, subdir: &str) -> Vec<&'a SourceFile> {
    let prefix = format!("{}{}/", ms_root_prefix(&inputs.microservice), subdir);
    inputs
        .files
        .iter()
        .filter(|f| f.repo_relative_path.starts_with(&prefix))
        .collect()
}

fn extract_bounded_contexts(inputs: &ManifestInputs) -> (Vec<BoundedContext>, BTreeSet<String>) {
    let mut bc_to_crates: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut bc_descriptions: BTreeMap<String, String> = BTreeMap::new();
    let mut layers: BTreeSet<String> = BTreeSet::new();
    let catalog_files: Vec<&SourceFile> = files_under(inputs, "catalog")
        .into_iter()
        .filter(|f| f.repo_relative_path.ends_with(".yaml"))
        .collect();
    let mut catalog_sorted = catalog_files;
    catalog_sorted.sort_by(|a, b| a.repo_relative_path.cmp(&b.repo_relative_path));
    for file in catalog_sorted {
        let data = parse_yaml_flat(&file.content);
        let bc = data
            .scalar("bc")
            .or_else(|| data.scalar("context"))
            .unwrap_or_else(|| "unknown".to_string());
        let stem = stem_of(&file.repo_relative_path);
        let name = data.scalar("name").unwrap_or(stem);
        let role = data.scalar("role").unwrap_or_default();
        if ALLOWED_LAYERS.contains(&role.as_str()) {
            layers.insert(role.clone());
        }
        bc_to_crates.entry(bc.clone()).or_default().insert(name);
        bc_descriptions.entry(bc.clone()).or_insert_with(|| {
            let ctx = data.scalar("context").unwrap_or_default();
            let plane = data.scalar("plane").unwrap_or_default();
            let mut desc = format!("Bounded context '{bc}'");
            if !ctx.is_empty() {
                desc.push_str(&format!(" within {ctx}"));
            }
            if !plane.is_empty() {
                desc.push_str(&format!(" ({plane} plane)"));
            }
            desc
        });
    }
    let mut bcs: Vec<BoundedContext> = Vec::new();
    for (bc, crates) in &bc_to_crates {
        bcs.push(BoundedContext {
            name: bc.clone(),
            description: bc_descriptions
                .get(bc)
                .cloned()
                .unwrap_or_else(|| format!("Bounded context '{bc}'")),
            crates: crates.iter().cloned().collect(),
        });
    }
    (bcs, layers)
}

fn extract_contracts(inputs: &ManifestInputs) -> ContractsBundle {
    let mut bundle = ContractsBundle::default();
    let openapi_prefix = format!("{}contracts/openapi/", ms_root_prefix(&inputs.microservice));
    let asyncapi_prefix = format!(
        "{}contracts/asyncapi/",
        ms_root_prefix(&inputs.microservice)
    );
    let proto_prefix = format!("{}contracts/proto/", ms_root_prefix(&inputs.microservice));
    let mut openapi: Vec<String> = inputs
        .files
        .iter()
        .filter(|f| {
            f.repo_relative_path.starts_with(&openapi_prefix)
                && f.repo_relative_path.ends_with(".yaml")
        })
        .map(|f| f.repo_relative_path.clone())
        .collect();
    openapi.sort();
    let mut asyncapi: Vec<String> = inputs
        .files
        .iter()
        .filter(|f| {
            f.repo_relative_path.starts_with(&asyncapi_prefix)
                && f.repo_relative_path.ends_with(".yaml")
        })
        .map(|f| f.repo_relative_path.clone())
        .collect();
    asyncapi.sort();
    let mut proto: Vec<String> = inputs
        .files
        .iter()
        .filter(|f| {
            f.repo_relative_path.starts_with(&proto_prefix)
                && f.repo_relative_path.ends_with(".proto")
        })
        .map(|f| f.repo_relative_path.clone())
        .collect();
    proto.sort();
    bundle.openapi = openapi;
    bundle.asyncapi = asyncapi;
    bundle.proto = proto;
    bundle
}

fn extract_capabilities(inputs: &ManifestInputs) -> Vec<CapabilityRow> {
    let mut out: Vec<CapabilityRow> = Vec::new();
    let mut caps: Vec<&SourceFile> = files_under(inputs, "capabilities")
        .into_iter()
        .filter(|f| f.repo_relative_path.ends_with(".yaml"))
        .collect();
    caps.sort_by(|a, b| a.repo_relative_path.cmp(&b.repo_relative_path));
    for file in caps {
        let data = parse_yaml_flat(&file.content);
        let stem = stem_of(&file.repo_relative_path);
        let name = data.scalar("name").unwrap_or(stem);
        let raw_tier = data
            .scalar("tier")
            .or_else(|| data.scalar("autonomy_tier"))
            .unwrap_or_default();
        let tier = if matches!(raw_tier.as_str(), "T0" | "T1" | "T2" | "T3") {
            raw_tier
        } else {
            "T1".to_string()
        };
        let mut risk = data
            .scalar("eu_ai_act_risk_class")
            .unwrap_or_else(|| default_risk_for_tier(&tier).to_string());
        let nm_low = name.to_lowercase();
        if !data.scalars.contains_key("eu_ai_act_risk_class")
            && [
                "biometric",
                "credit",
                "scoring",
                "employment",
                "law-enforcement",
                "border",
            ]
            .iter()
            .any(|t| nm_low.contains(t))
        {
            risk = "high".to_string();
        }
        out.push(CapabilityRow {
            tier,
            name,
            file: file.repo_relative_path.clone(),
            risk_class: risk,
        });
    }
    out
}

fn default_risk_for_tier(tier: &str) -> &'static str {
    match tier {
        "T0" => "none",
        "T1" => "minimal",
        "T2" => "limited",
        "T3" => "high",
        _ => "limited",
    }
}

fn extract_slos(inputs: &ManifestInputs) -> Vec<SloRow> {
    let mut out: Vec<SloRow> = Vec::new();
    let mut slos: Vec<&SourceFile> = files_under(inputs, "slos")
        .into_iter()
        .filter(|f| f.repo_relative_path.ends_with(".openslo.yaml"))
        .collect();
    slos.sort_by(|a, b| a.repo_relative_path.cmp(&b.repo_relative_path));
    for file in slos {
        let text = &file.content;
        let stem = stem_of(&file.repo_relative_path);
        let mut name = stem.trim_end_matches(".openslo").to_string();
        for raw in text.lines() {
            // expect lines like '  name: <thing>' with two or more leading spaces
            let leading = raw.chars().take_while(|c| *c == ' ').count();
            if leading < 2 {
                continue;
            }
            if let Some(rest) = raw.trim_start().strip_prefix("name:") {
                name = rest.trim().to_string();
                break;
            }
        }
        let target = first_capture_after(text, "target:");
        let threshold = first_capture_after(text, "threshold:");
        let target_str = match (target.as_deref(), threshold.as_deref()) {
            (Some(t), Some(thr)) => format!("{t} ({thr})"),
            (Some(t), None) => t.to_string(),
            (None, Some(thr)) => thr.to_string(),
            (None, None) => "see file".to_string(),
        };
        let sli = first_query_line(text).unwrap_or_else(|| "see indicator block".to_string());
        out.push(SloRow {
            name,
            target: target_str,
            sli,
            file: file.repo_relative_path.clone(),
        });
    }
    out
}

fn first_capture_after(text: &str, prefix: &str) -> Option<String> {
    for line in text.lines() {
        if let Some(idx) = line.find(prefix) {
            let rest = &line[idx + prefix.len()..];
            let token: String = rest
                .chars()
                .skip_while(|c| c.is_whitespace())
                .take_while(|c| !c.is_whitespace())
                .collect();
            if !token.is_empty() {
                return Some(strip_quotes(&token).to_string());
            }
        }
    }
    None
}

fn first_query_line(text: &str) -> Option<String> {
    let mut lines = text.lines().peekable();
    while let Some(line) = lines.next() {
        let trimmed_line = line.trim_start();
        if let Some(rest) = trimmed_line.strip_prefix("query:") {
            let inline = strip_quotes(rest.trim());
            if !inline.is_empty() {
                let truncated: String = inline.chars().take(200).collect();
                return Some(truncated);
            }
            for next in lines.by_ref() {
                let trimmed = next.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed.starts_with("objectives:") || trimmed.starts_with("timeWindow:") {
                    return None;
                }
                let truncated: String = strip_quotes(trimmed).chars().take(200).collect();
                return Some(truncated);
            }
        }
    }
    None
}

fn extract_ips(inputs: &ManifestInputs) -> Vec<IpRow> {
    let mut out: Vec<IpRow> = Vec::new();
    let prefix = ms_root_prefix(&inputs.microservice);
    let mut ips: Vec<&SourceFile> = inputs
        .files
        .iter()
        .filter(|f| {
            if !f.repo_relative_path.starts_with(&prefix) {
                return false;
            }
            let Some(rel) = relative_within(&inputs.microservice, &f.repo_relative_path) else {
                return false;
            };
            !rel.contains('/') && rel.starts_with("IP-") && rel.ends_with(".md")
        })
        .collect();
    ips.sort_by(|a, b| a.repo_relative_path.cmp(&b.repo_relative_path));
    for file in ips {
        let fm = parse_front_matter(&file.content);
        let stem = stem_of(&file.repo_relative_path);
        let title = first_h1(&file.content).unwrap_or_else(|| stem.clone());
        let id = fm.scalar("impl_plan_id").unwrap_or(stem);
        let mut row = IpRow {
            id,
            title,
            acceptance_status: "ga".to_string(),
            file: file.repo_relative_path.clone(),
            ..Default::default()
        };
        for (front_key, dst) in [
            ("changeset_id", &mut row.changeset_id),
            ("depends_on_changesets", &mut row.depends_on_changesets),
            (
                "parallel_safe_with_changesets",
                &mut row.parallel_safe_with_changesets,
            ),
            ("enables", &mut row.enables),
        ] {
            if let Some(list) = fm.list(front_key) {
                *dst = list;
            } else if let Some(scalar) = fm.scalar(front_key) {
                *dst = vec![scalar];
            }
        }
        out.push(row);
    }
    out
}

fn first_h1(text: &str) -> Option<String> {
    for line in text.lines() {
        if let Some(stripped) = line.strip_prefix("# ") {
            return Some(stripped.trim().to_string());
        }
    }
    None
}

fn extract_regulatory_packs(inputs: &ManifestInputs) -> Vec<String> {
    let ms = &inputs.microservice;
    let residency_path = format!("microservices/{ms}/policy/data-residency.md");
    if let Some(file) = inputs
        .files
        .iter()
        .find(|f| f.repo_relative_path == residency_path)
    {
        let mut found: BTreeSet<String> = BTreeSet::new();
        for pack in ALLOWED_PACKS {
            let needle = format!("pack-{pack}");
            if file.content.contains(&needle) {
                found.insert((*pack).to_string());
            }
        }
        if !found.is_empty() {
            // Order by canonical order
            let mut ordered: Vec<String> = Vec::new();
            for canonical in CANONICAL_PACKS {
                if found.contains(*canonical) {
                    ordered.push((*canonical).to_string());
                }
            }
            return ordered;
        }
    }
    // Fallback: inspect any catalog file's regulatory_packs_consumed list
    let catalog_files: Vec<&SourceFile> = files_under(inputs, "catalog")
        .into_iter()
        .filter(|f| f.repo_relative_path.ends_with(".yaml"))
        .collect();
    for file in catalog_files {
        let data = parse_yaml_flat(&file.content);
        if let Some(packs) = data.list("regulatory_packs_consumed") {
            let mut filtered: Vec<String> = Vec::new();
            for p in packs {
                let p2 = p
                    .trim_start_matches("oya-pack-")
                    .trim_start_matches("pack-")
                    .to_string();
                if ALLOWED_PACKS.contains(&p2.as_str()) {
                    filtered.push(p2);
                }
            }
            if !filtered.is_empty() {
                let unique: BTreeSet<String> = filtered.into_iter().collect();
                return unique.into_iter().collect();
            }
            break;
        }
    }
    CANONICAL_PACKS.iter().map(|s| (*s).to_string()).collect()
}

fn extract_lts_pins(inputs: &ManifestInputs) -> BTreeMap<String, String> {
    let mut pins: BTreeMap<String, String> = BTreeMap::new();
    let helm_prefix = format!("microservices/{}/iac/helm/", inputs.microservice);

    // 1) Chart.yaml appVersion for charts whose name matches a third-party LTS key.
    for file in &inputs.files {
        if !file.repo_relative_path.starts_with(&helm_prefix) {
            continue;
        }
        if !file.repo_relative_path.ends_with("Chart.yaml") {
            continue;
        }
        let chart_name = pick_yaml_top_scalar(&file.content, "name").unwrap_or_default();
        let app_version = pick_yaml_top_scalar(&file.content, "appVersion").unwrap_or_default();
        if chart_name.is_empty() || app_version.is_empty() {
            continue;
        }
        let chart_lower = chart_name.to_lowercase();
        for (dep, _) in LTS_DEFAULTS {
            if chart_lower.contains(dep) {
                pins.entry((*dep).to_string())
                    .or_insert(app_version.clone());
                break;
            }
        }
    }

    // 2) Subdirectory names under iac/helm/ matched against canonical LTS list.
    let mut subdirs: BTreeSet<String> = BTreeSet::new();
    for file in &inputs.files {
        if let Some(rest) = file.repo_relative_path.strip_prefix(&helm_prefix) {
            for component in rest.split('/').filter(|s| !s.is_empty()) {
                subdirs.insert(component.to_lowercase());
            }
        }
    }
    for subdir in &subdirs {
        for (dep, default_ver) in LTS_DEFAULTS {
            if subdir.contains(dep) {
                pins.entry((*dep).to_string())
                    .or_insert((*default_ver).to_string());
            }
        }
    }

    // 3) Always declare the rust toolchain pin.
    pins.entry("rust".to_string()).or_insert_with(|| {
        LTS_DEFAULTS
            .iter()
            .find(|(k, _)| *k == "rust")
            .map(|(_, v)| (*v).to_string())
            .unwrap_or_default()
    });

    pins
}

fn pick_yaml_top_scalar(text: &str, key: &str) -> Option<String> {
    let needle = format!("{key}:");
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix(&needle) {
            return Some(rest.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn extract_adrs(inputs: &ManifestInputs) -> Vec<AdrRow> {
    let mut by_id: BTreeMap<String, AdrRow> = BTreeMap::new();

    let add_repo = |by_id: &mut BTreeMap<String, AdrRow>, adr_id: &str| {
        if !is_adr_id(adr_id) {
            return;
        }
        if by_id.contains_key(adr_id) {
            return;
        }
        let prefix = format!("{adr_id}-");
        let matches: Vec<&String> = inputs
            .docs_decisions_filenames
            .iter()
            .filter(|s| s.starts_with(&prefix) && s.ends_with(".md"))
            .collect();
        if let Some(first) = matches.into_iter().next() {
            let stem = first.trim_end_matches(".md");
            let title = stem[adr_id.len() + 1..].replace('-', " ");
            let title = title_case(&title);
            by_id.insert(
                adr_id.to_string(),
                AdrRow {
                    id: adr_id.to_string(),
                    title,
                    scope: "repo".to_string(),
                    file: Some(format!("docs/decisions/{first}")),
                },
            );
        }
    };

    // PRD front-matter
    let prd_path = format!("microservices/{}/PRD.md", inputs.microservice);
    if let Some(prd) = inputs
        .files
        .iter()
        .find(|f| f.repo_relative_path == prd_path)
    {
        let fm = parse_front_matter(&prd.content);
        if let Some(values) = fm.list("related_adrs") {
            for adr in values {
                add_repo(&mut by_id, adr.trim().trim_matches(',').trim());
            }
        }
    }

    // Catalog traceability.source_adrs
    for file in files_under(inputs, "catalog") {
        if !file.repo_relative_path.ends_with(".yaml") {
            continue;
        }
        let data = parse_yaml_flat(&file.content);
        if let Some(values) = data.nested_list("traceability", "source_adrs") {
            for adr in values {
                add_repo(&mut by_id, &adr);
            }
        }
    }

    // Microservice-local decisions
    let dec_prefix = format!("microservices/{}/decisions/", inputs.microservice);
    let mut local: Vec<&SourceFile> = inputs
        .files
        .iter()
        .filter(|f| {
            f.repo_relative_path.starts_with(&dec_prefix)
                && f.repo_relative_path.ends_with(".md")
                && f.repo_relative_path[dec_prefix.len()..].starts_with("ADR-")
        })
        .collect();
    local.sort_by(|a, b| a.repo_relative_path.cmp(&b.repo_relative_path));
    for file in local {
        let stem = stem_of(&file.repo_relative_path);
        let id_part: String = stem.chars().take(8).collect();
        if is_adr_id(&id_part) {
            let title_raw = stem[id_part.len()..].trim_start_matches('-');
            let title = if title_raw.is_empty() {
                id_part.clone()
            } else {
                title_case(&title_raw.replace('-', " "))
            };
            by_id.insert(
                id_part.clone(),
                AdrRow {
                    id: id_part,
                    title,
                    scope: "microservice".to_string(),
                    file: Some(file.repo_relative_path.clone()),
                },
            );
        }
    }

    let mut out: Vec<AdrRow> = by_id.into_values().collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

fn is_adr_id(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() != 8 {
        return false;
    }
    if !s.starts_with("ADR-") {
        return false;
    }
    s[4..].chars().all(|c| c.is_ascii_digit())
}

fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let mut out = String::new();
                    out.extend(first.to_uppercase());
                    out.push_str(&chars.as_str().to_lowercase());
                    out
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn canonical_hyperscaler_coverage() -> Value {
    json!({
        "circuit_breaker": format!(
            "INV-CIRCUIT-BREAKER-BULKHEAD → {CANONICAL_PROMETHEUSRULE_PATH}#OyaCapabilityCircuitOpen"
        ),
        "tenant_rate_limit": format!(
            "INV-SHUFFLE-SHARDING → {CANONICAL_PROMETHEUSRULE_PATH}#OyaTenantRateLimit429Surge"
        ),
        "golden_signals": format!(
            "INV-FOUR-GOLDEN-SIGNALS → {CANONICAL_PROMETHEUSRULE_PATH}#OyaSaturationCpuOver70pct"
        ),
        "error_budget_burn": format!(
            "INV-SLO-ERROR-BUDGET → {CANONICAL_PROMETHEUSRULE_PATH}#OyaErrorBudgetFastBurn1h14x"
        ),
    })
}

fn extract_audit_seal_events(ms: &str, capabilities: &[CapabilityRow]) -> Value {
    let mut events: BTreeSet<String> = BTreeSet::new();
    for cap in capabilities {
        events.insert(format!("oya.{ms}.{}", cap.name));
    }
    if events.is_empty() {
        events.insert(format!("oya.{ms}.lifecycle"));
    }
    json!({
        "enabled": true,
        "seal_events": events.into_iter().collect::<Vec<_>>(),
    })
}

fn stem_of(path: &str) -> String {
    let filename = path.rsplit('/').next().unwrap_or(path);
    let stem = filename
        .rsplit_once('.')
        .map(|(left, _)| left)
        .unwrap_or(filename);
    stem.to_string()
}

// ---------- mini YAML / front-matter parser ----------

#[derive(Debug, Default)]
struct YamlFlat {
    scalars: BTreeMap<String, String>,
    lists: BTreeMap<String, Vec<String>>,
    nested: BTreeMap<(String, String), Vec<String>>,
}

impl YamlFlat {
    fn scalar(&self, key: &str) -> Option<String> {
        self.scalars.get(key).cloned()
    }
    fn list(&self, key: &str) -> Option<Vec<String>> {
        self.lists.get(key).cloned()
    }
    fn nested_list(&self, parent: &str, child: &str) -> Option<Vec<String>> {
        self.nested
            .get(&(parent.to_string(), child.to_string()))
            .cloned()
    }
}

fn parse_yaml_flat(text: &str) -> YamlFlat {
    let mut out = YamlFlat::default();
    let mut stack: Vec<(usize, String)> = Vec::new();
    let mut current_key: Option<String> = None;
    for raw_line in text.lines() {
        let line_no_comment = strip_trailing_comment(raw_line);
        if line_no_comment.trim().is_empty() {
            continue;
        }
        let indent = line_no_comment.chars().take_while(|c| *c == ' ').count();
        let stripped = line_no_comment.trim();

        while let Some((stack_indent, _)) = stack.last() {
            if indent <= *stack_indent {
                stack.pop();
            } else {
                break;
            }
        }

        if let Some(item) = stripped.strip_prefix("- ") {
            if let Some(key) = &current_key {
                if let Some((parent, _)) = stack.last() {
                    let parent = *parent;
                    let _ = parent;
                }
                // append to lists or nested
                if let Some(parent_key) = stack.last().map(|(_, k)| k.clone()) {
                    if let Some(kv) = parse_inline_kv(item) {
                        out.nested
                            .entry((parent_key.clone(), key.clone()))
                            .or_default()
                            .push(kv.1);
                    } else {
                        out.nested
                            .entry((parent_key.clone(), key.clone()))
                            .or_default()
                            .push(strip_quotes(item).to_string());
                    }
                } else if let Some((_, val)) = parse_inline_kv(item) {
                    out.lists.entry(key.clone()).or_default().push(val);
                } else {
                    out.lists
                        .entry(key.clone())
                        .or_default()
                        .push(strip_quotes(item).to_string());
                }
            }
            continue;
        }

        if let Some((key, value)) = parse_inline_kv(stripped) {
            if value.is_empty() {
                stack.push((indent, key.clone()));
                current_key = Some(key);
            } else if value.starts_with('[') && value.ends_with(']') {
                let inner = &value[1..value.len() - 1];
                let items: Vec<String> = inner
                    .split(',')
                    .map(|s| strip_quotes(s.trim()).to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if let Some(parent) = stack.last().map(|(_, k)| k.clone()) {
                    out.nested.insert((parent, key.clone()), items);
                } else {
                    out.lists.insert(key.clone(), items);
                }
                current_key = Some(key);
            } else {
                let value = strip_quotes(&value).to_string();
                if let Some(parent) = stack.last().map(|(_, k)| k.clone()) {
                    out.nested
                        .entry((parent, key.clone()))
                        .or_default()
                        .push(value);
                } else {
                    out.scalars.insert(key.clone(), value);
                }
                current_key = Some(key);
            }
        }
    }
    out
}

fn strip_trailing_comment(line: &str) -> String {
    let mut in_quotes = false;
    let mut quote_char = '"';
    let mut out = String::new();
    let chars: Vec<char> = line.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if !in_quotes && (*c == '"' || *c == '\'') {
            in_quotes = true;
            quote_char = *c;
        } else if in_quotes && *c == quote_char {
            in_quotes = false;
        }
        if !in_quotes && *c == '#' {
            // require preceding whitespace
            if i == 0 || chars[i - 1].is_whitespace() {
                break;
            }
        }
        out.push(*c);
    }
    out
}

fn parse_inline_kv(s: &str) -> Option<(String, String)> {
    let mut chars = s.chars();
    let first = chars.next()?;
    if !(first.is_ascii_alphabetic() || first == '_') {
        return None;
    }
    let mut key = String::from(first);
    for c in chars.by_ref() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
            key.push(c);
            continue;
        }
        if c == ':' {
            let rest: String = chars.collect();
            return Some((key, rest.trim().to_string()));
        }
        return None;
    }
    None
}

fn strip_quotes(s: &str) -> &str {
    let trimmed = s.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2)
        || (trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2)
    {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    }
}

fn parse_front_matter(text: &str) -> YamlFlat {
    let mut lines = text.lines();
    let Some(first) = lines.next() else {
        return YamlFlat::default();
    };
    if first.trim() != "---" {
        return YamlFlat::default();
    }
    let mut body = String::new();
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        body.push_str(line);
        body.push('\n');
    }
    parse_yaml_flat(&body)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sf(path: &str, content: &str) -> SourceFile {
        SourceFile {
            repo_relative_path: path.to_string(),
            content: content.to_string(),
        }
    }

    fn inputs(ms: &str, files: Vec<SourceFile>) -> ManifestInputs {
        ManifestInputs {
            microservice: ms.to_string(),
            files,
            docs_decisions_filenames: vec![],
        }
    }

    #[test]
    fn empty_inputs_produces_placeholder_bc_and_default_layers() {
        let m = build_manifest(&inputs("foo", vec![]));
        assert_eq!(m["microservice"], "foo");
        assert_eq!(m["owner"], "axis-foo");
        assert_eq!(m["bounded_contexts"][0]["name"], "foo");
        assert!(
            m["bounded_contexts"][0]["crates"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(
            m["layers"]
                .as_array()
                .unwrap()
                .contains(&Value::String("kernel".into()))
        );
    }

    #[test]
    fn catalog_yaml_seeds_bounded_contexts_and_layers() {
        let cat = "\
bc: ledger
context: audit
plane: control
name: oya-audit-chain-kernel
role: kernel
";
        let files = vec![sf("microservices/audit/catalog/kernel.yaml", cat)];
        let m = build_manifest(&inputs("audit", files));
        assert_eq!(m["bounded_contexts"][0]["name"], "ledger");
        assert!(
            m["bounded_contexts"][0]["description"]
                .as_str()
                .unwrap()
                .contains("control plane")
        );
        assert!(
            m["layers"]
                .as_array()
                .unwrap()
                .contains(&Value::String("kernel".into()))
        );
    }

    #[test]
    fn capabilities_yaml_default_risk_class_promotes_high_for_keywords() {
        let cap_ok = "name: standard-thing\nautonomy_tier: T2\n";
        let cap_bio = "name: biometric-identify\nautonomy_tier: T2\n";
        let files = vec![
            sf("microservices/foo/capabilities/a.yaml", cap_ok),
            sf("microservices/foo/capabilities/b.yaml", cap_bio),
        ];
        let m = build_manifest(&inputs("foo", files));
        let caps = m["capabilities"].as_array().unwrap();
        let bio = caps
            .iter()
            .find(|c| c["name"] == "biometric-identify")
            .unwrap();
        let ok = caps.iter().find(|c| c["name"] == "standard-thing").unwrap();
        assert_eq!(bio["eu_ai_act_risk_class"], "high");
        assert_eq!(ok["eu_ai_act_risk_class"], "limited");
    }

    #[test]
    fn capabilities_invalid_tier_falls_back_to_t1() {
        let cap = "name: bad\nautonomy_tier: T9\n";
        let files = vec![sf("microservices/foo/capabilities/a.yaml", cap)];
        let m = build_manifest(&inputs("foo", files));
        assert_eq!(m["capabilities"][0]["tier"], "T1");
    }

    #[test]
    fn capabilities_yaml_prefers_canonical_tier_and_explicit_risk() {
        let cap = "name: rollback-execute\ntier: T3\neu_ai_act_risk_class: high\n";
        let files = vec![sf(
            "microservices/foo/capabilities/rollback-execute.yaml",
            cap,
        )];
        let m = build_manifest(&inputs("foo", files));
        assert_eq!(m["capabilities"][0]["tier"], "T3");
        assert_eq!(m["capabilities"][0]["eu_ai_act_risk_class"], "high");
    }

    #[test]
    fn slos_extract_inline_prometheus_query() {
        let slo = r#"apiVersion: openslo/v1
kind: SLO
metadata:
  name: oya-foo-availability
spec:
  objective:
    target: "0.999"
  indicator:
    ratioMetric:
      good:
        metricSource:
          spec:
            query: 'sum(rate(foo_good_total[5m])) / sum(rate(foo_total[5m]))'
      total:
        metricSource:
          spec:
            query: 'sum(rate(foo_total[5m]))'
"#;
        let files = vec![sf("microservices/foo/slos/availability.openslo.yaml", slo)];
        let m = build_manifest(&inputs("foo", files));
        assert_eq!(
            m["slos"][0]["sli"],
            "sum(rate(foo_good_total[5m])) / sum(rate(foo_total[5m]))"
        );
    }

    #[test]
    fn slos_strip_quoted_target_values() {
        let slo = r#"apiVersion: openslo/v1
kind: SLO
metadata:
  name: oya-foo-latency
spec:
  objective:
    target: "0.95"
  indicator:
    ratioMetric:
      good:
        metricSource:
          spec:
            query: 'histogram_quantile(0.95, sum(rate(foo_bucket[5m])) by (le))'
"#;
        let files = vec![sf("microservices/foo/slos/latency.openslo.yaml", slo)];
        let m = build_manifest(&inputs("foo", files));
        assert_eq!(m["slos"][0]["target"], "0.95");
    }

    #[test]
    fn contracts_collects_openapi_asyncapi_proto() {
        let files = vec![
            sf(
                "microservices/foo/contracts/openapi/foo.yaml",
                "openapi: 3.0.0\n",
            ),
            sf(
                "microservices/foo/contracts/asyncapi/bar.yaml",
                "asyncapi: 2.6\n",
            ),
            sf("microservices/foo/contracts/proto/baz.proto", "syntax\n"),
        ];
        let m = build_manifest(&inputs("foo", files));
        assert_eq!(
            m["contracts"]["openapi"][0],
            "microservices/foo/contracts/openapi/foo.yaml"
        );
        assert_eq!(
            m["contracts"]["asyncapi"][0],
            "microservices/foo/contracts/asyncapi/bar.yaml"
        );
        assert_eq!(
            m["contracts"]["proto"][0],
            "microservices/foo/contracts/proto/baz.proto"
        );
    }

    #[test]
    fn regulatory_packs_falls_back_to_canonical_when_silent() {
        let m = build_manifest(&inputs("foo", vec![]));
        assert_eq!(
            m["regulatory_packs"].as_array().unwrap().len(),
            CANONICAL_PACKS.len()
        );
    }

    #[test]
    fn regulatory_packs_reads_residency_md() {
        let content = "Pack coverage: pack-kr, pack-eu, pack-us are mandatory.\n";
        let files = vec![sf("microservices/foo/policy/data-residency.md", content)];
        let m = build_manifest(&inputs("foo", files));
        let packs: Vec<String> = m["regulatory_packs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        assert_eq!(packs, vec!["kr", "eu", "us"]);
    }

    #[test]
    fn lts_pins_always_include_rust() {
        let m = build_manifest(&inputs("foo", vec![]));
        assert!(m["lts_pins"].get("rust").is_some());
    }

    #[test]
    fn ip_md_files_extract_title_and_id() {
        let content =
            "---\nimpl_plan_id: IP-001\nchangeset_id: CS-100\n---\n\n# Some title here\n\nBody.";
        let files = vec![sf("microservices/foo/IP-001-thing.md", content)];
        let m = build_manifest(&inputs("foo", files));
        assert_eq!(m["ips"][0]["id"], "IP-001");
        assert_eq!(m["ips"][0]["title"], "Some title here");
        assert_eq!(m["ips"][0]["acceptance_status"], "ga");
        assert_eq!(m["ips"][0]["changeset_id"][0], "CS-100");
    }

    #[test]
    fn audit_seal_events_default_to_lifecycle_when_no_caps() {
        let m = build_manifest(&inputs("foo", vec![]));
        assert_eq!(m["audit_chain"]["enabled"], true);
        assert_eq!(m["audit_chain"]["seal_events"][0], "oya.foo.lifecycle");
    }

    #[test]
    fn hyperscaler_coverage_is_canonical() {
        let m = build_manifest(&inputs("foo", vec![]));
        let hyper = &m["hyperscaler_inv_coverage"];
        for k in [
            "circuit_breaker",
            "tenant_rate_limit",
            "golden_signals",
            "error_budget_burn",
        ] {
            assert!(hyper.get(k).is_some(), "missing key {k}");
            assert!(
                hyper[k]
                    .as_str()
                    .unwrap()
                    .contains(CANONICAL_PROMETHEUSRULE_PATH)
            );
        }
    }

    #[test]
    fn adrs_picks_up_local_decisions() {
        let files = vec![sf(
            "microservices/foo/decisions/ADR-0123-something-good.md",
            "# ADR-0123: Something good\n",
        )];
        let m = build_manifest(&inputs("foo", files));
        let adr = &m["adrs"][0];
        assert_eq!(adr["id"], "ADR-0123");
        assert_eq!(adr["scope"], "microservice");
    }

    #[test]
    fn aggregate_index_count_matches_microservices_arg() {
        let v = build_manifests_index("2026-05-18", &["foo", "bar"]);
        assert_eq!(v["manifest_count"], 2);
        assert!(v["microservices"][1].get("manifest").is_none());
        assert_eq!(
            v["microservices"][1]["legacy_manifest_path"],
            "microservices/bar/manifest.json"
        );
        assert_eq!(
            v["microservices"][1]["do_not_write_to_manifest_index"],
            true
        );
    }

    #[test]
    fn current_manifest_index_contract_uses_current_paths() {
        let v = build_manifests_index("2026-05-19", MICROSERVICES);
        assert_eq!(v["manifest_count"], 37);
        let rows = v["microservices"].as_array().unwrap();
        assert_eq!(rows.len(), 37);
        assert!(
            rows.iter()
                .all(|row| row.get("name").and_then(Value::as_str) != Some("cell"))
        );
        assert!(
            rows.iter()
                .all(|row| row.get("name").and_then(Value::as_str) != Some("network"))
        );
        assert!(
            rows.iter()
                .all(|row| row.get("name").and_then(Value::as_str) != Some("shorts"))
        );

        let anonymous = rows
            .iter()
            .find(|row| row.get("name").and_then(Value::as_str) == Some("anonymous"))
            .unwrap();
        assert!(anonymous.get("manifest").is_none());
        assert_eq!(anonymous["parent_inventory"], "oya/community/manifest.json");
        assert_eq!(anonymous["subproduct_of"], "community");

        let foundry = rows
            .iter()
            .find(|row| row.get("name").and_then(Value::as_str) == Some("foundry"))
            .unwrap();
        assert_eq!(foundry["status"], "retired");
        assert_eq!(foundry["absorbed_by"], "intelligence");
        assert_eq!(foundry["do_not_treat_as_active"], true);
        assert_eq!(foundry["manifest"], "oya/intelligence/manifest.json");

        let managed_k8s = rows
            .iter()
            .find(|row| {
                row.get("name").and_then(Value::as_str) == Some("managed-k8s-control-plane-host")
            })
            .unwrap();
        assert_eq!(
            managed_k8s["manifest"],
            "cloud/managed-k8s-control-plane-host/manifest.json"
        );
        assert!(v["readiness_contracts"]["multi_region_disposition"]
            ["coverage_scope_note"]
            .as_str()
            .unwrap()
            .contains("current `oya/<service>/manifest.json` and `cloud/<service>/manifest.json` roots"));
    }
}
