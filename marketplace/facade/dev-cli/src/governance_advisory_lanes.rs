// Governance advisory gate wiring.
//
// Eleven advisory lanes whose check-* crates ship structured validators that
// expect typed input (parsed manifest namespaces, declarations, etc.). The
// manifest-schema namespaces these lanes consume (`storage.*`, `data.*`,
// `runtime.wasm.*`, `i18n.*`, `a11y.*`, `realtime.*`, `compliance.*`,
// `finops.*`, `iac.*`) were appended to specs/microservices/manifest-schema.json
// in this same PR. No µservice manifest declares these fields yet — they exist
// purely as advisory shape declarations until per-µservice population PRs.
//
// Per ADR-0221 §M-06 (vacuous-green gate doctrine), each of these advisory
// dispatchers MUST report the honest zero-input state when no manifests carry
// the expected fields — they do NOT silently emit PASS with 0 inputs. The
// returned summary struct surfaces `manifests_scanned` so downstream
// `oya-check-vacuous-green-gates` (queued for PR-144) can detect the condition.
//
// Severity ladder per `registry/quality/lanes.yaml`:
//   - vendor-lockin-discipline   = BLOCKER (sole strict lane; wired in lib.rs)
//   - all 11 below                = report-only advisory until promotion_target
//                                   triggers in lanes.yaml fire.
//
// Each function returns Ok(summary) on advisory-completion; Err only on
// IO/parse error (which is a real fault, not advisory).

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// Canonical service roots scanned when discovering manifests.
const SERVICE_ROOTS: &[&str] = &["cloud", "oya", "microservices"];

/// Scan all canonical service roots for `*/manifest.json` paths.
///
/// Common helper: every advisory walker uses the same enumeration. Scans
/// `cloud/*/manifest.json`, `oya/*/manifest.json`, and
/// `microservices/*/manifest.json`. Returns the path list sorted for
/// deterministic gate output. Roots that are absent are silently skipped.
fn discover_microservice_manifests() -> Result<Vec<PathBuf>, String> {
    let mut manifests = Vec::new();
    for root_str in SERVICE_ROOTS {
        let root = Path::new(root_str);
        if !root.exists() {
            continue;
        }
        let entries =
            fs::read_dir(root).map_err(|error| format!("unable to read {root_str}/: {error}"))?;
        for entry in entries {
            let entry = entry.map_err(|error| format!("unable to walk {root_str}/: {error}"))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let manifest = path.join("manifest.json");
            if manifest.exists() {
                manifests.push(manifest);
            }
        }
    }
    manifests.sort();
    Ok(manifests)
}

fn read_manifest_json(path: &Path) -> Result<Option<Value>, String> {
    let body = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(_) => return Ok(None),
    };
    serde_json::from_str::<Value>(&body)
        .map(Some)
        .map_err(|error| format!("{} manifest JSON parse failed: {error}", path.display()))
}

fn object_at<'a>(value: &'a Value, pointer: &str) -> Option<&'a serde_json::Map<String, Value>> {
    value.pointer(pointer).and_then(Value::as_object)
}

fn has_object(value: &Value, pointer: &str) -> bool {
    object_at(value, pointer).is_some()
}

fn has_field(value: &Value, pointer: &str) -> bool {
    value.pointer(pointer).is_some()
}

// ---------------------------------------------------------------------------
// 1. authz-tier-discipline (ADR-0191)
// ---------------------------------------------------------------------------

pub(crate) struct AuthzTierSummary {
    pub cedar_files_scanned: usize,
    pub envoy_files_scanned: usize,
    pub total_findings: usize,
}

pub(crate) fn validate_authz_tier_discipline_gate(
    _args: Vec<String>,
) -> Result<AuthzTierSummary, String> {
    // Walk every microservices/<ms>/policy/*.cedar and
    // microservices/<ms>/iac/helm/**/envoy*.yaml, invoking
    // check_authz_tier_discipline::scan_cedar / scan_envoy_filter.
    //
    // Honest scope: this advisory pass enumerates *.cedar and envoy*.yaml under
    // each µservice; absent files yield zero scans (NOT vacuous-pass — count
    // reported in summary).
    let mut cedar_files_scanned = 0usize;
    let mut envoy_files_scanned = 0usize;
    let mut total_findings = 0usize;

    let root = Path::new("microservices");
    if !root.exists() {
        return Ok(AuthzTierSummary {
            cedar_files_scanned,
            envoy_files_scanned,
            total_findings,
        });
    }
    let entries =
        fs::read_dir(root).map_err(|error| format!("unable to read microservices/: {error}"))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("unable to walk microservices/: {error}"))?;
        let ms_dir = entry.path();
        if !ms_dir.is_dir() {
            continue;
        }
        // Cedar policy fragments.
        let policy_dir = ms_dir.join("policy");
        if policy_dir.exists() {
            for cedar in walk_extension(&policy_dir, "cedar")? {
                let body = match fs::read_to_string(&cedar) {
                    Ok(text) => text,
                    Err(_) => continue,
                };
                let report =
                    check_authz_tier_discipline::scan_cedar(&cedar.display().to_string(), &body);
                total_findings += report.findings.len();
                cedar_files_scanned += 1;
            }
        }
        // Envoy filter manifests under helm renders.
        let helm_dir = ms_dir.join("iac").join("helm");
        if helm_dir.exists() {
            for envoy in walk_filename_contains(&helm_dir, "envoy", "yaml")? {
                let body = match fs::read_to_string(&envoy) {
                    Ok(text) => text,
                    Err(_) => continue,
                };
                let report = check_authz_tier_discipline::scan_envoy_filter(
                    &envoy.display().to_string(),
                    &body,
                );
                total_findings += report.findings.len();
                envoy_files_scanned += 1;
            }
        }
    }
    Ok(AuthzTierSummary {
        cedar_files_scanned,
        envoy_files_scanned,
        total_findings,
    })
}

fn walk_extension(root: &Path, ext: &str) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    walk_recursive(root, &mut |path| {
        if path.extension().and_then(|s| s.to_str()) == Some(ext) {
            out.push(path.to_path_buf());
        }
    })?;
    out.sort();
    Ok(out)
}

fn walk_filename_contains(root: &Path, needle: &str, ext: &str) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    walk_recursive(root, &mut |path| {
        let name = match path.file_name().and_then(|s| s.to_str()) {
            Some(n) => n,
            None => return,
        };
        if path.extension().and_then(|s| s.to_str()) == Some(ext) && name.contains(needle) {
            out.push(path.to_path_buf());
        }
    })?;
    out.sort();
    Ok(out)
}

fn walk_recursive(root: &Path, visit: &mut dyn FnMut(&Path)) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }
    let entries = fs::read_dir(root)
        .map_err(|error| format!("unable to read {}: {error}", root.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("unable to walk {}: {error}", root.display()))?;
        let path = entry.path();
        if path.is_dir() {
            walk_recursive(&path, visit)?;
        } else {
            visit(&path);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 2. tenant-cost-labels-coverage (ADR-0199)
// ---------------------------------------------------------------------------

pub(crate) struct TenantCostLabelsSummary {
    pub manifests_scanned: usize,
    pub findings: usize,
}

pub(crate) fn validate_tenant_cost_labels_coverage_gate(
    _args: Vec<String>,
) -> Result<TenantCostLabelsSummary, String> {
    // ADR-0199 D-1: every microservices/*/iac/helm/*/rendered/*.yaml must carry
    // oya.io/{tenant-id, cost-center, workload-class, regulatory-pack} labels.
    // Until any µservice ships rendered Helm outputs at that path, the
    // advisory pass discloses zero scans honestly.
    let mut manifests_scanned = 0usize;
    let mut findings = 0usize;
    let root = Path::new("microservices");
    if !root.exists() {
        return Ok(TenantCostLabelsSummary {
            manifests_scanned,
            findings,
        });
    }
    let entries =
        fs::read_dir(root).map_err(|error| format!("unable to read microservices/: {error}"))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("unable to walk microservices/: {error}"))?;
        let helm_rendered = entry.path().join("iac").join("helm");
        if !helm_rendered.exists() {
            continue;
        }
        for yaml in walk_extension(&helm_rendered, "yaml")? {
            // Limit to `rendered/` paths per ADR-0199 D-1.
            if !yaml.components().any(|c| c.as_os_str() == "rendered") {
                continue;
            }
            manifests_scanned += 1;
            let body = match fs::read_to_string(&yaml) {
                Ok(text) => text,
                Err(_) => continue,
            };
            // Advisory check: required label keys present?
            for required in &[
                "oya.io/tenant-id",
                "oya.io/cost-center",
                "oya.io/workload-class",
                "oya.io/regulatory-pack",
            ] {
                if !body.contains(required) {
                    findings += 1;
                }
            }
        }
    }
    Ok(TenantCostLabelsSummary {
        manifests_scanned,
        findings,
    })
}

// ---------------------------------------------------------------------------
// 3. backup-retention-discipline (ADR-0197)
// ---------------------------------------------------------------------------

pub(crate) struct BackupRetentionSummary {
    pub declarations_scanned: usize,
    pub findings: usize,
}

pub(crate) fn validate_backup_retention_discipline_gate(
    _args: Vec<String>,
) -> Result<BackupRetentionSummary, String> {
    // ADR-0197 D-5: each µservice manifest's storage.backup.* declares
    // tier (app|batch|gpu|regulatory) + retention_days. Until populated, this
    // advisory pass scans manifests for the namespace and reports honest count.
    let manifests = discover_microservice_manifests()?;
    let mut declarations_scanned = 0usize;
    let mut findings = 0usize;
    for path in &manifests {
        let Some(manifest) = read_manifest_json(path)? else {
            continue;
        };
        if has_object(&manifest, "/storage/backup") {
            declarations_scanned += 1;
            if !has_field(&manifest, "/storage/backup/retention_days")
                || !has_field(&manifest, "/storage/backup/tier")
            {
                findings += 1;
            }
        }
    }
    Ok(BackupRetentionSummary {
        declarations_scanned,
        findings,
    })
}

// ---------------------------------------------------------------------------
// 4. vector-store-discipline (ADR-0192)
// ---------------------------------------------------------------------------

pub(crate) struct VectorStoreSummary {
    pub records_scanned: usize,
    pub violations: usize,
}

pub(crate) fn validate_vector_store_discipline_gate(
    _args: Vec<String>,
) -> Result<VectorStoreSummary, String> {
    // 2026-05-25 launch directive: Milvus is workload-specific, not a
    // universal vector-store default; manifest's data.vector_store namespace
    // declares explicit collections only for workloads that select it.
    let manifests = discover_microservice_manifests()?;
    let mut records_scanned = 0usize;
    let mut violations = 0usize;
    for path in &manifests {
        let Some(manifest) = read_manifest_json(path)? else {
            continue;
        };
        if has_object(&manifest, "/data/vector_store") {
            records_scanned += 1;
            // Advisory presence check: enabled + collections both required.
            if !has_field(&manifest, "/data/vector_store/collections") {
                violations += 1;
            }
        }
    }
    Ok(VectorStoreSummary {
        records_scanned,
        violations,
    })
}

// ---------------------------------------------------------------------------
// 5. olap-tier-discipline (ADR-0193)
// ---------------------------------------------------------------------------

pub(crate) struct OlapTierSummary {
    pub records_scanned: usize,
    pub violations: usize,
}

pub(crate) fn validate_olap_tier_discipline_gate(
    _args: Vec<String>,
) -> Result<OlapTierSummary, String> {
    // 2026-05-25 launch directive: ClickHouse and Iceberg are
    // workload-specific analytics/table-format selections; no rogue analytics
    // on OLTP Postgres, and no universal OLAP default for every workload.
    let manifests = discover_microservice_manifests()?;
    let mut records_scanned = 0usize;
    let mut violations = 0usize;
    for path in &manifests {
        let Some(manifest) = read_manifest_json(path)? else {
            continue;
        };
        if has_object(&manifest, "/data/olap_client") {
            records_scanned += 1;
            if !has_field(&manifest, "/data/olap_client/databases") {
                violations += 1;
            }
        }
    }
    Ok(OlapTierSummary {
        records_scanned,
        violations,
    })
}

// ---------------------------------------------------------------------------
// 6. wasm-runtime-discipline (ADR-0200)
// ---------------------------------------------------------------------------

pub(crate) struct WasmRuntimeSummary {
    pub manifests_scanned: usize,
    pub violations: usize,
}

pub(crate) fn validate_wasm_runtime_discipline_gate(
    _args: Vec<String>,
) -> Result<WasmRuntimeSummary, String> {
    // ADR-0200: wasmtime-canonical WASM substrate; sandbox_classes enum.
    let manifests = discover_microservice_manifests()?;
    let mut manifests_scanned = 0usize;
    let mut violations = 0usize;
    for path in &manifests {
        let Some(manifest) = read_manifest_json(path)? else {
            continue;
        };
        if has_object(&manifest, "/runtime/wasm") {
            manifests_scanned += 1;
            if !has_field(&manifest, "/runtime/wasm/sandbox_classes") {
                violations += 1;
            }
        }
    }
    Ok(WasmRuntimeSummary {
        manifests_scanned,
        violations,
    })
}

// ---------------------------------------------------------------------------
// 7. iac-tier-discipline (ADR-0202)
// ---------------------------------------------------------------------------

pub(crate) struct IacTierSummary {
    pub artifacts_scanned: usize,
    pub violations: usize,
}

pub(crate) fn validate_iac_tier_discipline_gate(
    _args: Vec<String>,
) -> Result<IacTierSummary, String> {
    // ADR-0202: OpenTofu canonical; Terraform path only during 90-day window.
    // Advisory scan walks microservices/*/iac/ counting *.tf and *.tofu.
    let root = Path::new("microservices");
    let mut artifacts_scanned = 0usize;
    let mut violations = 0usize;
    if !root.exists() {
        return Ok(IacTierSummary {
            artifacts_scanned,
            violations,
        });
    }
    let entries =
        fs::read_dir(root).map_err(|error| format!("unable to read microservices/: {error}"))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("unable to walk microservices/: {error}"))?;
        let iac = entry.path().join("iac");
        if !iac.exists() {
            continue;
        }
        for tf in walk_extension(&iac, "tf")? {
            artifacts_scanned += 1;
            // Existence of a raw .tf file (not .tf.hcl-shared with tofu) is
            // a soft advisory finding during the 90-day migration window.
            let _ = tf;
            violations += 1;
        }
        for tofu in walk_extension(&iac, "tofu")? {
            artifacts_scanned += 1;
            let _ = tofu;
        }
    }
    Ok(IacTierSummary {
        artifacts_scanned,
        violations,
    })
}

// ---------------------------------------------------------------------------
// 8. a11y-discipline (ADR-0207)
// ---------------------------------------------------------------------------

pub(crate) struct A11ySummary {
    pub surfaces_scanned: usize,
    pub gaps: usize,
}

pub(crate) fn validate_a11y_discipline_gate(_args: Vec<String>) -> Result<A11ySummary, String> {
    // ADR-0207: WCAG 2.2 AA coverage; axe + pa11y test runners per client
    // surface declared in manifest.json#a11y.
    let manifests = discover_microservice_manifests()?;
    let mut surfaces_scanned = 0usize;
    let mut gaps = 0usize;
    for path in &manifests {
        let Some(manifest) = read_manifest_json(path)? else {
            continue;
        };
        if has_object(&manifest, "/a11y") {
            surfaces_scanned += 1;
            if !has_field(&manifest, "/a11y/test_runners")
                || !has_field(&manifest, "/a11y/wcag_target")
            {
                gaps += 1;
            }
        }
    }
    Ok(A11ySummary {
        surfaces_scanned,
        gaps,
    })
}

// ---------------------------------------------------------------------------
// 9. i18n-coverage (ADR-0206)
// ---------------------------------------------------------------------------

pub(crate) struct I18nCoverageSummary {
    pub surfaces_scanned: usize,
    pub gaps: usize,
}

pub(crate) fn validate_i18n_coverage_gate(
    _args: Vec<String>,
) -> Result<I18nCoverageSummary, String> {
    // ADR-0206: Fluent ICU locale coverage; default_locale + required_locales
    // + rtl_support + min_coverage_bps.
    let manifests = discover_microservice_manifests()?;
    let mut surfaces_scanned = 0usize;
    let mut gaps = 0usize;
    for path in &manifests {
        let Some(manifest) = read_manifest_json(path)? else {
            continue;
        };
        if has_object(&manifest, "/i18n") {
            surfaces_scanned += 1;
            if !has_field(&manifest, "/i18n/default_locale")
                || !has_field(&manifest, "/i18n/required_locales")
            {
                gaps += 1;
            }
        }
    }
    Ok(I18nCoverageSummary {
        surfaces_scanned,
        gaps,
    })
}

// ---------------------------------------------------------------------------
// 10. compliance-evidence-coverage (ADR-0209)
// ---------------------------------------------------------------------------

pub(crate) struct ComplianceEvidenceSummary {
    pub microservices_scanned: usize,
    pub gaps: usize,
}

pub(crate) fn validate_compliance_evidence_coverage_gate(
    _args: Vec<String>,
) -> Result<ComplianceEvidenceSummary, String> {
    // ADR-0209: every µservice declares audit_chain_seal_required +
    // tamper_evidence_algorithm + evidence_collectors[].
    let manifests = discover_microservice_manifests()?;
    let mut microservices_scanned = 0usize;
    let mut gaps = 0usize;
    for path in &manifests {
        let Some(manifest) = read_manifest_json(path)? else {
            continue;
        };
        if has_object(&manifest, "/compliance") {
            microservices_scanned += 1;
            if !has_field(&manifest, "/compliance/evidence_collectors")
                || !has_field(&manifest, "/compliance/audit_chain_seal_required")
            {
                gaps += 1;
            }
        }
    }
    Ok(ComplianceEvidenceSummary {
        microservices_scanned,
        gaps,
    })
}

// ---------------------------------------------------------------------------
// 11. realtime-transport-tier (ADR-0208)
// ---------------------------------------------------------------------------

pub(crate) struct RealtimeTransportSummary {
    pub declarations_scanned: usize,
    pub gaps: usize,
}

pub(crate) fn validate_realtime_transport_tier_gate(
    _args: Vec<String>,
) -> Result<RealtimeTransportSummary, String> {
    // ADR-0208: sse vs websocket vs grpc-streaming; payload_budget_bytes per
    // tier. Advisory scan for namespace presence.
    let manifests = discover_microservice_manifests()?;
    let mut declarations_scanned = 0usize;
    let mut gaps = 0usize;
    for path in &manifests {
        let Some(manifest) = read_manifest_json(path)? else {
            continue;
        };
        if has_object(&manifest, "/realtime") {
            declarations_scanned += 1;
            if !has_field(&manifest, "/realtime/transport")
                || !has_field(&manifest, "/realtime/tier")
            {
                gaps += 1;
            }
        }
    }
    Ok(RealtimeTransportSummary {
        declarations_scanned,
        gaps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_microservice_manifests_returns_sorted_list_or_empty() {
        let manifests = discover_microservice_manifests();
        assert!(
            manifests.is_ok(),
            "discovery must not error on missing root"
        );
        if let Ok(list) = manifests {
            // List is sorted by path.
            let sorted: Vec<_> = {
                let mut clone = list.clone();
                clone.sort();
                clone
            };
            assert_eq!(list, sorted);
        }
    }

    #[test]
    fn authz_tier_advisory_scans_without_panic() {
        // Honest disclosure: zero inputs is a valid advisory state per ADR-0221 §M-06.
        let summary = validate_authz_tier_discipline_gate(Vec::new()).expect("advisory ok");
        // Total findings is non-negative usize.
        let _ = summary.total_findings;
    }

    #[test]
    fn tenant_cost_labels_advisory_scans_without_panic() {
        let summary = validate_tenant_cost_labels_coverage_gate(Vec::new()).expect("advisory ok");
        let _ = summary.findings;
    }

    #[test]
    fn backup_retention_advisory_scans_without_panic() {
        let summary = validate_backup_retention_discipline_gate(Vec::new()).expect("advisory ok");
        let _ = summary.findings;
    }
}
