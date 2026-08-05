//! Layered-architecture discipline gate (ADR-0148 / ADR-0182 / ADR-0183 / ADR-0184).
//!
//! # Why this crate exists
//!
//! Oyatie's hyperscaler-shape architecture is a stack of **layered
//! separations**:
//!
//! - ADR-0148 — Cilium L3/L4 + Istio Ambient L7, zero feature overlap.
//! - ADR-0182 — API gateway (north-south) vs service mesh (east-west),
//!   zero overlap.
//! - ADR-0183 — Cedar (app authz) vs Kyverno (K8s admission), zero
//!   overlap.
//! - ADR-0184 — Valkey 8.1 cluster for Tier-3 cache; Redis 7.4+ + Memcached
//!   rejected.
//!
//! The discipline is *zero feature overlap per layer*. A µservice that
//! declares both north-south AND east-west ownership, or imports both
//! Cedar AND Kyverno for runtime decisions, or imports both Valkey/Redis
//! AND Memcached as cache backends, violates the layer boundary.
//!
//! This crate is the pure-fn kernel behind the
//! `layered-architecture-discipline` Rust gate packet; any legacy
//! `oya gate validate layered-architecture-discipline` invocation is a
//! local bridge/provenance surface only.
//!
//! # Layer
//!
//! `domain` (port-in-kernel, ADR-0056). The kernel performs no I/O;
//! callers pass [`ManifestDocument`] records loaded from disk.
//!
//! # Naming justification
//!
//! `oya-check-layered-architecture-discipline` follows BNF v4.1:
//! `oya-<topic:check>-<axis:layered-architecture-discipline>`.
//!
//! # References
//!
//! - ADR-0148 — service-mesh canonical (Cilium L3/L4 + Istio Ambient L7
//!   layered).
//! - ADR-0182 — API gateway vs service mesh separation.
//! - ADR-0183 — policy engine separation (Cedar vs Kyverno).
//! - ADR-0184 — storage tier layering.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]
#![allow(clippy::result_large_err)]

use std::fmt;

/// One supplied µservice manifest. Callers load `microservices/<ms>/manifest.json`
/// and the µservice's `iac/helm/<ms>/values.yaml` (or app-tier source) and
/// pass the concatenated text plus the µservice name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestDocument {
    /// µservice name (e.g. "api-gateway", "tasks").
    pub microservice: String,
    /// Path the contents were loaded from (used in violation messages).
    pub path: String,
    /// The on-disk JSON / YAML / TOML text the validator searches.
    pub contents: String,
}

/// Successful audit summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayeredDisciplineReport {
    pub manifests_checked: usize,
    pub microservices_audited: usize,
    pub gateway_owners_detected: usize,
    pub waypoint_enrolled_count: usize,
}

/// A boundary violation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayeredArchitectureViolation {
    pub microservice: String,
    pub manifest_path: String,
    pub kind: ViolationKind,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ViolationKind {
    /// A µservice carries both north-south (gateway) AND east-west (mesh)
    /// traffic-direction declarations for the same workload.
    GatewayAndMeshConflict,
    /// A µservice imports / declares both Cedar (app authz) AND Kyverno
    /// (K8s admission) as runtime decision engines for app traffic.
    CedarAndKyvernoConflict,
    /// A µservice declares both Valkey/Redis Cluster usage AND Memcached
    /// usage as Tier-3 cache backends.
    CacheBackendConflict,
    /// A µservice declares `cilium_l4=false` or `ambient_ztunnel=false`
    /// without `north_south_only=true` (only api-gateway may opt out).
    MeshTierUnderclaimed,
    /// A µservice declares `north_south_only=true` but is not api-gateway.
    NorthSouthOnlyMisplaced,
}

impl fmt::Display for LayeredArchitectureViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}): {:?} — {}",
            self.microservice, self.manifest_path, self.kind, self.summary
        )
    }
}

/// Audit entrypoint. Returns the report on success; the first violation
/// as `Err` on failure (fail-closed).
pub fn validate_layered_discipline<I>(
    manifests: I,
) -> Result<LayeredDisciplineReport, LayeredArchitectureViolation>
where
    I: IntoIterator<Item = ManifestDocument>,
{
    let (report, violations) = audit_all_violations(manifests);
    if let Some(first) = violations.into_iter().next() {
        Err(first)
    } else {
        Ok(report)
    }
}

/// Full audit — returns the report AND every violation found.
pub fn audit_all_violations<I>(
    manifests: I,
) -> (LayeredDisciplineReport, Vec<LayeredArchitectureViolation>)
where
    I: IntoIterator<Item = ManifestDocument>,
{
    let manifests: Vec<ManifestDocument> = manifests.into_iter().collect();
    let mut violations = Vec::new();
    let mut microservices: std::collections::BTreeSet<String> = Default::default();
    let mut gateway_owners = 0usize;
    let mut waypoint_enrolled = 0usize;

    for manifest in &manifests {
        microservices.insert(manifest.microservice.clone());
        let lower = manifest.contents.to_ascii_lowercase();

        // 1. Gateway vs mesh conflict.
        //
        // Per ADR-0148/0182, the conflict states are:
        //   (a) µservice declares `north_south_only: true` AND ALSO
        //       declares `ambient_waypoint: true` — it claims to be
        //       the north-south gateway AND opt in to the east-west L7
        //       mesh waypoint. Mutually exclusive.
        //   (b) Helm-template-level annotation conflict: a single
        //       workload declares BOTH the
        //       `gateway.networking.k8s.io/managed-by:` annotation key
        //       AND the `istio.io/dataplane-mode: ambient` label at
        //       structured-key positions (line-anchored).
        //
        // The mesh_layering.cilium_l4 + ambient_ztunnel pair is
        // schema-required-true (advisory for north-south-only api-
        // gateway); it does NOT signal east-west ownership for conflict
        // purposes — only the helm-template annotation form does.
        let north_south_via_schema =
            line_has_key_value(&manifest.contents, "north_south_only", "true");
        let north_south_via_annotation = line_has_annotation_key_assignment(
            &manifest.contents,
            "gateway.networking.k8s.io/managed-by",
        );
        let manifest_waypoint_enrolled = declares_ambient_waypoint_true(&lower);
        let east_west_via_dataplane_annotation =
            line_has_annotation_key_value(&manifest.contents, "istio.io/dataplane-mode", "ambient");

        // Conflict shape (a): schema north_south_only=true AND ambient_waypoint=true.
        if north_south_via_schema && manifest_waypoint_enrolled {
            violations.push(LayeredArchitectureViolation {
                microservice: manifest.microservice.clone(),
                manifest_path: manifest.path.clone(),
                kind: ViolationKind::GatewayAndMeshConflict,
                summary: "manifest declares BOTH `mesh_layering.north_south_only: true` (claims \
                          north-south gateway ownership) AND `mesh_layering.ambient_waypoint: \
                          true` (opts into east-west L7 mesh waypoint); per ADR-0182 each \
                          µservice owns exactly one traffic direction"
                    .to_string(),
            });
        }

        // Conflict shape (b): helm-template annotation conflict.
        if north_south_via_annotation && east_west_via_dataplane_annotation {
            violations.push(LayeredArchitectureViolation {
                microservice: manifest.microservice.clone(),
                manifest_path: manifest.path.clone(),
                kind: ViolationKind::GatewayAndMeshConflict,
                summary: "manifest declares BOTH north-south \
                          ('gateway.networking.k8s.io/managed-by') AND east-west \
                          ('istio.io/dataplane-mode: ambient') traffic directions; per ADR-0182 \
                          each µservice owns exactly one direction"
                    .to_string(),
            });
        }

        if north_south_via_schema || north_south_via_annotation {
            gateway_owners += 1;
        }

        // 2. Cedar vs Kyverno conflict.
        if declares_cedar_runtime(&lower) && declares_kyverno_runtime_app(&lower) {
            violations.push(LayeredArchitectureViolation {
                microservice: manifest.microservice.clone(),
                manifest_path: manifest.path.clone(),
                kind: ViolationKind::CedarAndKyvernoConflict,
                summary: "manifest imports BOTH Cedar (app authz) AND Kyverno (K8s admission) \
                          for runtime app decisions; per ADR-0183 Cedar owns L7 app authz only, \
                          Kyverno owns K8s admission only — they may not both back the same \
                          decision path"
                    .to_string(),
            });
        }

        // 3. Cache backend conflict (Valkey/Redis AND Memcached).
        let uses_valkey_or_redis = declares_valkey_or_redis(&lower);
        let uses_memcached = declares_memcached(&lower);
        if uses_valkey_or_redis && uses_memcached {
            violations.push(LayeredArchitectureViolation {
                microservice: manifest.microservice.clone(),
                manifest_path: manifest.path.clone(),
                kind: ViolationKind::CacheBackendConflict,
                summary: "manifest declares BOTH Valkey/Redis-cluster usage AND Memcached usage \
                          as Tier-3 cache backends; per ADR-0184 the canonical Tier-3 backend is \
                          Valkey 8.1 (Memcached rejected for clustering / persistence / \
                          invalidation-stream gaps)"
                    .to_string(),
            });
        }

        // 4. Mesh tier under-claim (must declare cilium_l4 + ambient_ztunnel
        //    unless north_south_only).
        let north_south_only_claimed = declares_north_south_only(&lower);
        let cilium_declared = declares_cilium_tier_one(&lower);
        let ztunnel_declared = declares_ambient_ztunnel(&lower);
        if !north_south_only_claimed && (!cilium_declared || !ztunnel_declared) {
            violations.push(LayeredArchitectureViolation {
                microservice: manifest.microservice.clone(),
                manifest_path: manifest.path.clone(),
                kind: ViolationKind::MeshTierUnderclaimed,
                summary: "manifest does not declare both `cilium_l4: true` AND `ambient_ztunnel: \
                          true` (and is not the api-gateway with `north_south_only: true`); per \
                          ADR-0148 every µservice runs on both Tier-1 and Tier-2 layers"
                    .to_string(),
            });
        }

        // 5. north_south_only must only appear on api-gateway µservice.
        if north_south_only_claimed && manifest.microservice != "api-gateway" {
            violations.push(LayeredArchitectureViolation {
                microservice: manifest.microservice.clone(),
                manifest_path: manifest.path.clone(),
                kind: ViolationKind::NorthSouthOnlyMisplaced,
                summary: format!(
                    "manifest declares `north_south_only: true` but the µservice is {:?}; per \
                     ADR-0182 the sole north-south owner is the api-gateway µservice",
                    manifest.microservice
                ),
            });
        }

        // Track waypoint enrollment for the report.
        if declares_ambient_waypoint_true(&lower) {
            waypoint_enrolled += 1;
        }
    }

    let report = LayeredDisciplineReport {
        manifests_checked: manifests.len(),
        microservices_audited: microservices.len(),
        gateway_owners_detected: gateway_owners,
        waypoint_enrolled_count: waypoint_enrolled,
    };
    (report, violations)
}

// ---- structural detectors (line-anchored to avoid free-text discussion
// false positives; match only declaration-form keys + values) ----

/// Line-anchored check that the file contains a JSON or YAML
/// declaration of the form `<key>: <value>` (or `"<key>": <value>`).
/// Avoids matching free-text prose that mentions the key without
/// declaring it.
fn line_has_key_value(contents: &str, key: &str, value: &str) -> bool {
    let needle_json = format!("\"{key}\": {value}");
    let needle_yaml = format!("{key}: {value}");
    for line in contents.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(&needle_json) || trimmed.starts_with(&needle_yaml) {
            return true;
        }
    }
    false
}

/// Line-anchored check for an annotation/label key declaration. The
/// canonical form for K8s annotation keys is:
///   "<key>": "<value>"      (JSON)
///   <key>: <value>          (YAML)
/// Free-text prose ("the gateway.networking.k8s.io/managed-by
/// annotation says ...") never starts a line with the key.
fn line_has_annotation_key_assignment(contents: &str, key: &str) -> bool {
    let json_form = format!("\"{key}\":");
    let yaml_form = format!("{key}:");
    for line in contents.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(&json_form) || trimmed.starts_with(&yaml_form) {
            return true;
        }
    }
    false
}

fn line_has_annotation_key_value(contents: &str, key: &str, value: &str) -> bool {
    let json_form = format!("\"{key}\": \"{value}\"");
    let yaml_form = format!("{key}: {value}");
    for line in contents.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(&json_form) || trimmed.starts_with(&yaml_form) {
            return true;
        }
    }
    false
}

fn declares_cedar_runtime(lower: &str) -> bool {
    // Cedar PDP integration for runtime app-tier authz.
    (lower.contains("ext_authz") && lower.contains("cedar"))
        || lower.contains("oya-shared-cedar")
        || lower.contains("cedar evaluator")
}

fn declares_kyverno_runtime_app(lower: &str) -> bool {
    // Kyverno being used for *application-tier* runtime app decisions
    // (NOT for K8s admission, which is the canonical use).
    // The distinguishing marker is invoking Kyverno in app source code
    // rather than via ClusterPolicy CRs.
    if !lower.contains("kyverno") {
        return false;
    }
    lower.contains("github.com/kyverno/runtime")
        || lower.contains("github.com/kyverno")
        || lower.contains("kyverno-runtime-app")
        || lower.contains("kyverno_pdp")
}

fn declares_valkey_or_redis(lower: &str) -> bool {
    lower.contains("valkey-cluster")
        || lower.contains("valkey 8")
        || lower.contains("redis-cluster")
        || lower.contains("oya_cache_backend\": \"valkey")
        || lower.contains("oya_cache_backend: valkey")
}

fn declares_memcached(lower: &str) -> bool {
    lower.contains("memcached") && !lower.contains("rejected") && !lower.contains("not memcached")
}

fn declares_cilium_tier_one(lower: &str) -> bool {
    lower.contains("\"cilium_l4\": true")
        || lower.contains("cilium_l4: true")
        || lower.contains("ciliumnetworkpolicy")
}

fn declares_ambient_ztunnel(lower: &str) -> bool {
    lower.contains("\"ambient_ztunnel\": true")
        || lower.contains("ambient_ztunnel: true")
        || lower.contains("istio.io/dataplane-mode: ambient")
}

fn declares_north_south_only(lower: &str) -> bool {
    lower.contains("\"north_south_only\": true") || lower.contains("north_south_only: true")
}

fn declares_ambient_waypoint_true(lower: &str) -> bool {
    lower.contains("\"ambient_waypoint\": true") || lower.contains("ambient_waypoint: true")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(ms: &str, contents: &str) -> ManifestDocument {
        ManifestDocument {
            microservice: ms.to_string(),
            path: format!("microservices/{ms}/manifest.json"),
            contents: contents.to_string(),
        }
    }

    const CONFORMANT_TASKS_MANIFEST: &str = r#"
{
  "microservice": "tasks",
  "mesh_layering": {
    "cilium_l4": true,
    "ambient_ztunnel": true,
    "ambient_waypoint": false,
    "north_south_only": false
  }
}
"#;

    const CONFORMANT_API_GATEWAY_MANIFEST: &str = r#"
{
  "microservice": "api-gateway",
  "mesh_layering": {
    "cilium_l4": true,
    "ambient_ztunnel": true,
    "ambient_waypoint": false,
    "north_south_only": true
  },
  "annotations": {
    "gateway.networking.k8s.io/managed-by": "envoy-gateway"
  }
}
"#;

    const CONFORMANT_GOVERNANCE_MANIFEST: &str = r#"
{
  "microservice": "governance",
  "mesh_layering": {
    "cilium_l4": true,
    "ambient_ztunnel": true,
    "ambient_waypoint": true
  }
}
"#;

    #[test]
    fn passes_on_conformant_tasks_manifest() {
        let report = validate_layered_discipline(vec![mk("tasks", CONFORMANT_TASKS_MANIFEST)])
            .expect("tasks manifest must pass");
        assert_eq!(report.manifests_checked, 1);
        assert_eq!(report.microservices_audited, 1);
        assert_eq!(report.gateway_owners_detected, 0);
        assert_eq!(report.waypoint_enrolled_count, 0);
    }

    #[test]
    fn passes_on_api_gateway_north_south_only() {
        let report =
            validate_layered_discipline(vec![mk("api-gateway", CONFORMANT_API_GATEWAY_MANIFEST)])
                .expect("api-gateway manifest must pass");
        assert_eq!(report.gateway_owners_detected, 1);
    }

    #[test]
    fn passes_on_governance_waypoint_enrolled() {
        let report =
            validate_layered_discipline(vec![mk("governance", CONFORMANT_GOVERNANCE_MANIFEST)])
                .expect("governance waypoint manifest must pass");
        assert_eq!(report.waypoint_enrolled_count, 1);
    }

    #[test]
    fn fails_on_gateway_and_mesh_conflict() {
        // A µservice declaring BOTH north-south gateway annotation AND
        // east-west ambient dataplane is the canonical layer-boundary
        // violation.
        let bad = r#"
{
  "microservice": "tasks",
  "annotations": {
    "gateway.networking.k8s.io/managed-by": "envoy-gateway"
  },
  "mesh_layering": {
    "cilium_l4": true,
    "ambient_ztunnel": true,
    "ambient_waypoint": false
  },
  "labels": {
    "istio.io/dataplane-mode": "ambient"
  }
}
"#;
        let err = validate_layered_discipline(vec![mk("tasks", bad)]).expect_err("must fail");
        assert_eq!(err.kind, ViolationKind::GatewayAndMeshConflict);
    }

    #[test]
    fn fails_on_cedar_and_kyverno_runtime_conflict() {
        let bad = r#"
{
  "microservice": "tasks",
  "imports": [
    "ext_authz cedar evaluator",
    "import \"github.com/kyverno/runtime\""
  ],
  "mesh_layering": {
    "cilium_l4": true,
    "ambient_ztunnel": true,
    "ambient_waypoint": false
  }
}
"#;
        let err = validate_layered_discipline(vec![mk("tasks", bad)]).expect_err("must fail");
        assert_eq!(err.kind, ViolationKind::CedarAndKyvernoConflict);
    }

    #[test]
    fn fails_on_valkey_and_memcached_conflict() {
        let bad = r#"
{
  "microservice": "tasks",
  "cache": {
    "primary": "valkey-cluster",
    "secondary": "memcached"
  },
  "mesh_layering": {
    "cilium_l4": true,
    "ambient_ztunnel": true,
    "ambient_waypoint": false
  }
}
"#;
        let err = validate_layered_discipline(vec![mk("tasks", bad)]).expect_err("must fail");
        assert_eq!(err.kind, ViolationKind::CacheBackendConflict);
    }

    #[test]
    fn fails_on_mesh_tier_underclaim() {
        // Missing cilium_l4: true.
        let bad = r#"
{
  "microservice": "tasks",
  "mesh_layering": {
    "ambient_ztunnel": true,
    "ambient_waypoint": false
  }
}
"#;
        let err = validate_layered_discipline(vec![mk("tasks", bad)]).expect_err("must fail");
        assert_eq!(err.kind, ViolationKind::MeshTierUnderclaimed);
    }

    #[test]
    fn fails_on_north_south_only_on_non_gateway() {
        let bad = r#"
{
  "microservice": "tasks",
  "mesh_layering": {
    "cilium_l4": true,
    "ambient_ztunnel": true,
    "ambient_waypoint": false,
    "north_south_only": true
  }
}
"#;
        let err = validate_layered_discipline(vec![mk("tasks", bad)]).expect_err("must fail");
        assert_eq!(err.kind, ViolationKind::NorthSouthOnlyMisplaced);
    }

    #[test]
    fn audit_all_violations_returns_full_list() {
        let bad_a = r#"
{
  "microservice": "tasks",
  "mesh_layering": {"cilium_l4": true, "ambient_ztunnel": true, "ambient_waypoint": false, "north_south_only": true}
}
"#;
        let bad_b = r#"
{
  "microservice": "calendar",
  "cache": {"primary": "valkey-cluster", "secondary": "memcached"},
  "mesh_layering": {"cilium_l4": true, "ambient_ztunnel": true, "ambient_waypoint": false}
}
"#;
        let (report, violations) =
            audit_all_violations(vec![mk("tasks", bad_a), mk("calendar", bad_b)]);
        assert_eq!(report.manifests_checked, 2);
        assert_eq!(violations.len(), 2);
        let kinds: std::collections::BTreeSet<_> = violations.iter().map(|v| v.kind).collect();
        assert!(kinds.contains(&ViolationKind::NorthSouthOnlyMisplaced));
        assert!(kinds.contains(&ViolationKind::CacheBackendConflict));
    }

    #[test]
    fn passes_on_empty_input() {
        let report = validate_layered_discipline(Vec::<ManifestDocument>::new())
            .expect("empty input must pass");
        assert_eq!(report.manifests_checked, 0);
        assert_eq!(report.microservices_audited, 0);
    }
}
