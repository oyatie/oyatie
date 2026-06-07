//! ADR-0051 mobile/native client fitness kernel.
//!
//! The lane is allowed to be active during Foundry preview with an explicit
//! web-canonical/no-native declaration. As soon as native project markers or
//! product records appear, the ADR-0051 quality bar becomes fail-closed.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileNativeManifest {
    pub current_wave: String,          // data_class: INTERNAL_ONLY
    pub empty_scope_rationale: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileNativeProductRecord {
    pub product_id: String,                     // data_class: INTERNAL_ONLY
    pub axis: String,                           // data_class: INTERNAL_ONLY
    pub status: String,                         // data_class: INTERNAL_ONLY
    pub canonical_web_reference: String,        // data_class: INTERNAL_ONLY
    pub target_matrix_ref: String,              // data_class: INTERNAL_ONLY
    pub tech_stack_rationale_ref: String,       // data_class: INTERNAL_ONLY
    pub store_policy_ref: String,               // data_class: INTERNAL_ONLY
    pub store_policy_validator_passed: bool,    // data_class: INTERNAL_ONLY
    pub accessibility_audit_ref: String,        // data_class: INTERNAL_ONLY
    pub accessibility_audit_passed: bool,       // data_class: INTERNAL_ONLY
    pub capability_parity_ref: String,          // data_class: INTERNAL_ONLY
    pub capability_parity_passed: bool,         // data_class: INTERNAL_ONLY
    pub sbom_ref: String,                       // data_class: INTERNAL_ONLY
    pub native_binary_blobs_without_sbom: u32,  // data_class: INTERNAL_ONLY
    pub crash_free_sessions_bps: Option<u32>,   // data_class: INTERNAL_ONLY
    pub crash_free_regression_bps: Option<u32>, // data_class: INTERNAL_ONLY
    pub cold_start_p99_ms: Option<u32>,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileNativeDiscoveryMarker {
    pub path: String,        // data_class: INTERNAL_ONLY
    pub marker_kind: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MobileNativePolicy {
    pub min_crash_free_sessions_bps: u32, // data_class: INTERNAL_ONLY
    pub max_crash_free_regression_bps: u32, // data_class: INTERNAL_ONLY
    pub max_cold_start_p99_ms: u32,       // data_class: INTERNAL_ONLY
}

impl MobileNativePolicy {
    pub fn adr_0051_quality_bar() -> Self {
        Self {
            min_crash_free_sessions_bps: 9_950,
            max_crash_free_regression_bps: 20,
            max_cold_start_p99_ms: 2_000,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileNativeReport {
    pub current_wave: String,               // data_class: INTERNAL_ONLY
    pub native_products_checked: usize,     // data_class: INTERNAL_ONLY
    pub native_markers_checked: usize,      // data_class: INTERNAL_ONLY
    pub quality_bar_records_checked: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MobileNativeError {
    InvalidPolicy,
    MissingCurrentWave,
    UnknownCurrentWave {
        current_wave: String,
    },
    MissingEmptyScopeRationale,
    NativeMarkersWithoutDeclaredProducts {
        markers: usize,
    },
    DeclaredProductsWithoutNativeMarkers,
    DuplicateProductId {
        product_id: String,
    },
    MissingField {
        product_id: String,
        field: &'static str,
    },
    UnknownStatus {
        product_id: String,
        status: String,
    },
    MissingMetric {
        product_id: String,
        metric: &'static str,
    },
    CrashFreeBelowThreshold {
        product_id: String,
        actual_bps: u32,
        minimum_bps: u32,
    },
    CrashFreeRegressionTooHigh {
        product_id: String,
        actual_bps: u32,
        maximum_bps: u32,
    },
    ColdStartTooHigh {
        product_id: String,
        actual_ms: u32,
        maximum_ms: u32,
    },
    StorePolicyValidatorFailed {
        product_id: String,
    },
    AccessibilityAuditFailed {
        product_id: String,
    },
    CapabilityParityFailed {
        product_id: String,
    },
    NativeBinaryBlobMissingSbom {
        product_id: String,
        blobs: u32,
    },
}

pub fn validate_mobile_native<R, M>(
    manifest: MobileNativeManifest,
    records: R,
    markers: M,
    policy: MobileNativePolicy,
) -> Result<MobileNativeReport, MobileNativeError>
where
    R: IntoIterator<Item = MobileNativeProductRecord>,
    M: IntoIterator<Item = MobileNativeDiscoveryMarker>,
{
    validate_policy(policy)?;
    let current_wave = manifest.current_wave.trim();
    if current_wave.is_empty() {
        return Err(MobileNativeError::MissingCurrentWave);
    }
    if !known_wave(current_wave) {
        return Err(MobileNativeError::UnknownCurrentWave {
            current_wave: current_wave.to_string(),
        });
    }

    let marker_count = markers.into_iter().count();
    let records = records.into_iter().collect::<Vec<_>>();
    if records.is_empty() {
        if !usable_ref(&manifest.empty_scope_rationale) {
            return Err(MobileNativeError::MissingEmptyScopeRationale);
        }
        if marker_count > 0 {
            return Err(MobileNativeError::NativeMarkersWithoutDeclaredProducts {
                markers: marker_count,
            });
        }
        return Ok(MobileNativeReport {
            current_wave: current_wave.to_string(),
            native_products_checked: 0,
            native_markers_checked: 0,
            quality_bar_records_checked: 0,
        });
    }

    if marker_count == 0 {
        return Err(MobileNativeError::DeclaredProductsWithoutNativeMarkers);
    }

    let mut seen_product_ids = BTreeSet::new();
    let mut quality_bar_records_checked = 0usize;
    for record in &records {
        validate_required_product_fields(record)?;
        if !seen_product_ids.insert(record.product_id.clone()) {
            return Err(MobileNativeError::DuplicateProductId {
                product_id: record.product_id.clone(),
            });
        }
        match record.status.as_str() {
            "native-in-scope" => {
                validate_quality_bar(record, policy)?;
                quality_bar_records_checked += 1;
            }
            status => {
                return Err(MobileNativeError::UnknownStatus {
                    product_id: record.product_id.clone(),
                    status: status.to_string(),
                });
            }
        }
    }

    Ok(MobileNativeReport {
        current_wave: current_wave.to_string(),
        native_products_checked: records.len(),
        native_markers_checked: marker_count,
        quality_bar_records_checked,
    })
}

fn validate_policy(policy: MobileNativePolicy) -> Result<(), MobileNativeError> {
    if policy.min_crash_free_sessions_bps > 10_000 || policy.max_cold_start_p99_ms == 0 {
        Err(MobileNativeError::InvalidPolicy)
    } else {
        Ok(())
    }
}

fn validate_required_product_fields(
    record: &MobileNativeProductRecord,
) -> Result<(), MobileNativeError> {
    for (field, value) in [
        ("product_id", &record.product_id),
        ("axis", &record.axis),
        ("status", &record.status),
        ("canonical_web_reference", &record.canonical_web_reference),
        ("target_matrix_ref", &record.target_matrix_ref),
        ("tech_stack_rationale_ref", &record.tech_stack_rationale_ref),
        ("store_policy_ref", &record.store_policy_ref),
        ("accessibility_audit_ref", &record.accessibility_audit_ref),
        ("capability_parity_ref", &record.capability_parity_ref),
        ("sbom_ref", &record.sbom_ref),
    ] {
        if !usable_ref(value) {
            return Err(MobileNativeError::MissingField {
                product_id: record.product_id.clone(),
                field,
            });
        }
    }
    Ok(())
}

fn validate_quality_bar(
    record: &MobileNativeProductRecord,
    policy: MobileNativePolicy,
) -> Result<(), MobileNativeError> {
    let crash_free_sessions_bps = metric(
        record,
        record.crash_free_sessions_bps,
        "crash_free_sessions_bps",
    )?;
    if crash_free_sessions_bps < policy.min_crash_free_sessions_bps {
        return Err(MobileNativeError::CrashFreeBelowThreshold {
            product_id: record.product_id.clone(),
            actual_bps: crash_free_sessions_bps,
            minimum_bps: policy.min_crash_free_sessions_bps,
        });
    }

    let crash_free_regression_bps = metric(
        record,
        record.crash_free_regression_bps,
        "crash_free_regression_bps",
    )?;
    if crash_free_regression_bps > policy.max_crash_free_regression_bps {
        return Err(MobileNativeError::CrashFreeRegressionTooHigh {
            product_id: record.product_id.clone(),
            actual_bps: crash_free_regression_bps,
            maximum_bps: policy.max_crash_free_regression_bps,
        });
    }

    let cold_start_p99_ms = metric(record, record.cold_start_p99_ms, "cold_start_p99_ms")?;
    if cold_start_p99_ms > policy.max_cold_start_p99_ms {
        return Err(MobileNativeError::ColdStartTooHigh {
            product_id: record.product_id.clone(),
            actual_ms: cold_start_p99_ms,
            maximum_ms: policy.max_cold_start_p99_ms,
        });
    }

    if !record.store_policy_validator_passed {
        return Err(MobileNativeError::StorePolicyValidatorFailed {
            product_id: record.product_id.clone(),
        });
    }
    if !record.accessibility_audit_passed {
        return Err(MobileNativeError::AccessibilityAuditFailed {
            product_id: record.product_id.clone(),
        });
    }
    if !record.capability_parity_passed {
        return Err(MobileNativeError::CapabilityParityFailed {
            product_id: record.product_id.clone(),
        });
    }
    if record.native_binary_blobs_without_sbom > 0 {
        return Err(MobileNativeError::NativeBinaryBlobMissingSbom {
            product_id: record.product_id.clone(),
            blobs: record.native_binary_blobs_without_sbom,
        });
    }
    Ok(())
}

fn metric(
    record: &MobileNativeProductRecord,
    value: Option<u32>,
    metric: &'static str,
) -> Result<u32, MobileNativeError> {
    value.ok_or_else(|| MobileNativeError::MissingMetric {
        product_id: record.product_id.clone(),
        metric,
    })
}

fn usable_ref(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty() && !matches!(value, "n/a" | "N/A" | "none" | "None" | "-")
}

fn known_wave(value: &str) -> bool {
    matches!(
        value,
        "W-Foundation"
            | "W-Foundry-Preview"
            | "W-Cloud-Preview"
            | "W-SaaS-Preview"
            | "W-Search-Preview"
            | "W-Workspace-Preview"
            | "W-Workspace-Stable"
            | "W-Vertical-Pilot"
            | "W-Vertical-Fan-Out"
            | "W-Cloud-Stable"
            | "W-Search-Stable"
            | "W-Ads-Preview"
            | "W-Ads-Stable"
            | "W-DataCenter-Operations"
            | "W-Robotics-Vision-Speech"
            | "W-AI-Model-Substrate"
            | "W-AI-Model-Stable"
            | "W-Region-Fan-Out"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_explicit_web_only_preview_scope_without_native_markers() {
        let report = validate_mobile_native(
            manifest(),
            [],
            [],
            MobileNativePolicy::adr_0051_quality_bar(),
        )
        .expect("web-only preview accepted");

        assert_eq!(report.native_products_checked, 0);
        assert_eq!(report.native_markers_checked, 0);
    }

    #[test]
    fn rejects_native_markers_without_declared_products() {
        assert_eq!(
            validate_mobile_native(
                manifest(),
                [],
                [marker("apps/workspace/ios/App.swift", "swift")],
                MobileNativePolicy::adr_0051_quality_bar(),
            ),
            Err(MobileNativeError::NativeMarkersWithoutDeclaredProducts { markers: 1 })
        );
    }

    #[test]
    fn accepts_native_product_with_adr0051_quality_evidence() {
        let report = validate_mobile_native(
            manifest(),
            [native_product()],
            [marker("apps/workspace/ios/App.swift", "swift")],
            MobileNativePolicy::adr_0051_quality_bar(),
        )
        .expect("quality evidence accepted");

        assert_eq!(report.native_products_checked, 1);
        assert_eq!(report.quality_bar_records_checked, 1);
    }

    #[test]
    fn rejects_crash_free_sessions_below_threshold() {
        let mut product = native_product();
        product.crash_free_sessions_bps = Some(9_949);

        assert_eq!(
            validate_mobile_native(
                manifest(),
                [product],
                [marker("apps/workspace/ios/App.swift", "swift")],
                MobileNativePolicy::adr_0051_quality_bar(),
            ),
            Err(MobileNativeError::CrashFreeBelowThreshold {
                product_id: "workspace-mail-mobile".into(),
                actual_bps: 9_949,
                minimum_bps: 9_950,
            })
        );
    }

    #[test]
    fn rejects_cold_start_over_two_seconds() {
        let mut product = native_product();
        product.cold_start_p99_ms = Some(2_001);

        assert!(matches!(
            validate_mobile_native(
                manifest(),
                [product],
                [marker(
                    "apps/workspace/android/AndroidManifest.xml",
                    "android-manifest"
                )],
                MobileNativePolicy::adr_0051_quality_bar(),
            ),
            Err(MobileNativeError::ColdStartTooHigh { .. })
        ));
    }

    #[test]
    fn rejects_missing_sbom_for_native_binary_blob() {
        let mut product = native_product();
        product.native_binary_blobs_without_sbom = 1;

        assert!(matches!(
            validate_mobile_native(
                manifest(),
                [product],
                [marker(
                    "apps/workspace/android/AndroidManifest.xml",
                    "android-manifest"
                )],
                MobileNativePolicy::adr_0051_quality_bar(),
            ),
            Err(MobileNativeError::NativeBinaryBlobMissingSbom { .. })
        ));
    }

    fn manifest() -> MobileNativeManifest {
        MobileNativeManifest {
            current_wave: "W-Foundry-Preview".into(),
            empty_scope_rationale: "ADR-0051 keeps native out of scope before W-Workspace-Stable"
                .into(),
        }
    }

    fn marker(path: &str, kind: &str) -> MobileNativeDiscoveryMarker {
        MobileNativeDiscoveryMarker {
            path: path.into(),
            marker_kind: kind.into(),
        }
    }

    fn native_product() -> MobileNativeProductRecord {
        MobileNativeProductRecord {
            product_id: "workspace-mail-mobile".into(),
            axis: "workspace".into(),
            status: "native-in-scope".into(),
            canonical_web_reference: "docs/products/workspace/PRD.md#mail".into(),
            target_matrix_ref: "docs/products/workspace/mobile.md#target-matrix".into(),
            tech_stack_rationale_ref: "docs/products/workspace/mobile.md#tech-stack".into(),
            store_policy_ref: "packs/kr/localization/README.md#mobile-store-policy".into(),
            store_policy_validator_passed: true,
            accessibility_audit_ref: "artifact://mobile/workspace-mail/accessibility.json".into(),
            accessibility_audit_passed: true,
            capability_parity_ref: "artifact://mobile/workspace-mail/parity.json".into(),
            capability_parity_passed: true,
            sbom_ref: "artifact://mobile/workspace-mail/sbom.spdx.json".into(),
            native_binary_blobs_without_sbom: 0,
            crash_free_sessions_bps: Some(9_950),
            crash_free_regression_bps: Some(20),
            cold_start_p99_ms: Some(2_000),
        }
    }
}
