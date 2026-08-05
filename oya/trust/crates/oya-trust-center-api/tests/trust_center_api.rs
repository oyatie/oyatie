// ADR-0083 Tier 3: integration tests use unwrap/expect/expect_err for invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use oya_trust_center_api::*;

const TENANT: &str = "ten_alpha";
const OTHER_TENANT: &str = "ten_beta";
const NOW: &str = "2026-07-01T06:30:00Z";
const RETAIN: &str = "2027-07-01T06:30:00Z";

#[test]
fn route_and_record_contracts_cover_trust_center_spec_slice() {
    let routes = trust_center_api_routes();
    assert_eq!(routes.len(), 7);
    assert!(routes.iter().any(|route| {
        route.method == "GET"
            && route.path == TRUST_CENTER_EVIDENCE_INDEX_PATH
            && route.response_records == vec![TRUST_CENTER_EVIDENCE_INDEX_RECORD_TYPE]
    }));
    assert!(routes.iter().any(|route| {
        route.method == "GET"
            && route.path == TRUST_CENTER_EVIDENCE_DETAIL_PATH
            && route.response_records == vec![TRUST_CENTER_EVIDENCE_ITEM_RECORD_TYPE]
    }));
    assert!(routes.iter().any(|route| {
        route.method == "GET"
            && route.path == TRUST_CENTER_SBOM_VEX_PATH
            && route.response_records == vec![TRUST_CENTER_SBOM_VEX_RECORD_TYPE]
    }));
    assert!(routes.iter().any(|route| {
        route.method == "GET"
            && route.path == TRUST_CENTER_CONTROL_FRESHNESS_PATH
            && route.response_records == vec![TRUST_CENTER_CONTROL_FRESHNESS_RECORD_TYPE]
    }));
    assert!(routes.iter().any(|route| {
        route.method == "GET"
            && route.path == TRUST_CENTER_COMPLIANCE_PACKS_PATH
            && route.response_records == vec![TRUST_CENTER_COMPLIANCE_PACK_RECORD_TYPE]
    }));
    assert!(routes.iter().any(|route| {
        route.method == "POST"
            && route.path == TRUST_CENTER_EXPORTS_PATH
            && route.response_records == vec![TRUST_CENTER_EXPORT_REQUEST_RECORD_TYPE]
    }));
    assert!(routes.iter().any(|route| {
        route.method == "GET"
            && route.path == TRUST_CENTER_ACCESS_AUDIT_PATH
            && route.response_records == vec![TRUST_CENTER_ACCESS_AUDIT_RECORD_TYPE]
    }));
    assert_eq!(TrustCenterApiStatus::Ok.code(), 200);
    assert_eq!(TrustCenterApiStatus::Accepted.code(), 202);
    assert_eq!(TrustCenterApiStatus::Unauthorized.code(), 401);
    assert_eq!(TrustCenterApiStatus::Forbidden.code(), 403);
    assert_eq!(TrustCenterApiStatus::UnprocessableEntity.code(), 422);
}

#[test]
fn evidence_index_is_server_tenant_scoped_and_payload_tenant_is_not_authority() {
    let mut model = fixture_model();
    let response = list_trust_center_evidence(
        &mut model,
        request(
            TrustCenterRole::TenantAdmin,
            TrustCenterPurpose::EvidenceRead,
            TRUST_CENTER_EVIDENCE_INDEX_SURFACE,
            TrustCenterEvidenceIndexQuery::default(),
        ),
    )
    .expect("tenant evidence index succeeds");

    assert_eq!(response.records.len(), 3);
    assert!(
        response
            .records
            .iter()
            .all(|record| record.common.tenant_id == TENANT)
    );
    assert!(
        response
            .records
            .iter()
            .all(|record| { record.common.record_type == TRUST_CENTER_EVIDENCE_INDEX_RECORD_TYPE })
    );
    assert!(response.records.iter().any(|record| {
        record.common.publishability_state == TrustCenterPublishabilityState::BlockedStaleEvidence
    }));
    assert!(response.records.iter().all(|record| {
        record.common.data_class != TrustCenterDataClass::OperatorSecurityInternal
    }));

    let asserted_other = TrustCenterEvidenceIndexQuery {
        asserted_tenant_id: Some(OTHER_TENANT.to_string()),
        ..Default::default()
    };
    let err = list_trust_center_evidence(
        &mut model,
        request(
            TrustCenterRole::TenantAdmin,
            TrustCenterPurpose::EvidenceRead,
            TRUST_CENTER_EVIDENCE_INDEX_SURFACE,
            asserted_other,
        ),
    )
    .expect_err("payload tenant assertion cannot become authority");
    assert_eq!(err.status_code(), 400);
    assert!(matches!(
        err,
        TrustCenterApiError::TenantAssertionMismatch { .. }
    ));
}

#[test]
fn payload_tenant_assertions_fail_closed_for_every_spec_endpoint() {
    let mut model = fixture_model();

    let index_query = TrustCenterEvidenceIndexQuery {
        asserted_tenant_id: Some(OTHER_TENANT.to_string()),
        ..Default::default()
    };
    assert_tenant_assertion_mismatch(
        list_trust_center_evidence(
            &mut model,
            request(
                TrustCenterRole::TenantAdmin,
                TrustCenterPurpose::EvidenceRead,
                TRUST_CENTER_EVIDENCE_INDEX_SURFACE,
                index_query,
            ),
        )
        .expect_err("evidence index must reject payload tenant authority"),
    );

    assert_tenant_assertion_mismatch(
        get_trust_center_evidence_detail(
            &mut model,
            request(
                TrustCenterRole::TenantAdmin,
                TrustCenterPurpose::EvidenceRead,
                TRUST_CENTER_EVIDENCE_DETAIL_SURFACE,
                TrustCenterEvidenceDetailQuery {
                    asserted_tenant_id: Some(OTHER_TENANT.to_string()),
                    evidence_id: "ev_alpha_current".to_string(),
                },
            ),
        )
        .expect_err("evidence detail must reject payload tenant authority"),
    );

    let controls_query = TrustCenterControlFreshnessQuery {
        asserted_tenant_id: Some(OTHER_TENANT.to_string()),
        ..Default::default()
    };
    assert_tenant_assertion_mismatch(
        get_trust_center_control_freshness(
            &mut model,
            request(
                TrustCenterRole::TenantAdmin,
                TrustCenterPurpose::ControlEvidenceRead,
                TRUST_CENTER_CONTROL_FRESHNESS_SURFACE,
                controls_query,
            ),
        )
        .expect_err("control freshness must reject payload tenant authority"),
    );

    let sbom_query = TrustCenterSbomVexQuery {
        asserted_tenant_id: Some(OTHER_TENANT.to_string()),
        ..Default::default()
    };
    assert_tenant_assertion_mismatch(
        get_trust_center_sbom_vex(
            &mut model,
            request(
                TrustCenterRole::TenantAdmin,
                TrustCenterPurpose::SecurityEvidenceRead,
                TRUST_CENTER_SBOM_VEX_SURFACE,
                sbom_query,
            ),
        )
        .expect_err("SBOM/VEX view must reject payload tenant authority"),
    );

    let packs_query = TrustCenterCompliancePackQuery {
        asserted_tenant_id: Some(OTHER_TENANT.to_string()),
        ..Default::default()
    };
    assert_tenant_assertion_mismatch(
        get_trust_center_compliance_packs(
            &mut model,
            request(
                TrustCenterRole::TenantAdmin,
                TrustCenterPurpose::ComplianceRead,
                TRUST_CENTER_COMPLIANCE_PACKS_SURFACE,
                packs_query,
            ),
        )
        .expect_err("compliance-pack view must reject payload tenant authority"),
    );

    assert_tenant_assertion_mismatch(
        create_trust_center_export_request(
            &mut model,
            request(
                TrustCenterRole::TenantAdmin,
                TrustCenterPurpose::ExportRequest,
                TRUST_CENTER_EXPORT_REQUEST_SURFACE,
                TrustCenterExportRequestInput {
                    asserted_tenant_id: Some(OTHER_TENANT.to_string()),
                    purpose: "customer_security_review".to_string(),
                    framework: "SOC2-readiness".to_string(),
                    time_window_start_trusted: "2026-06-01T00:00:00Z".to_string(),
                    time_window_end_trusted: NOW.to_string(),
                    evidence_ids: vec!["ev_alpha_current".to_string()],
                    expires_at_trusted: RETAIN.to_string(),
                },
            ),
        )
        .expect_err("export request must reject payload tenant authority"),
    );

    let audit_query = TrustCenterAccessAuditQuery {
        asserted_tenant_id: Some(OTHER_TENANT.to_string()),
        ..Default::default()
    };
    assert_tenant_assertion_mismatch(
        get_trust_center_access_audit(
            &mut model,
            request(
                TrustCenterRole::TenantAdmin,
                TrustCenterPurpose::AccessAuditRead,
                TRUST_CENTER_ACCESS_AUDIT_SURFACE,
                audit_query,
            ),
        )
        .expect_err("access audit must reject payload tenant authority"),
    );
}

#[test]
fn evidence_detail_fails_closed_for_cross_tenant_stale_missing_and_operator_only_records() {
    let mut model = fixture_model();

    let cross_tenant = get_trust_center_evidence_detail(
        &mut model,
        detail_request("ev_other_current", TrustCenterRole::TenantAdmin),
    )
    .expect_err("detail route must fail closed for cross-tenant evidence ids");
    assert_eq!(cross_tenant.status_code(), 403);
    assert!(matches!(
        cross_tenant,
        TrustCenterApiError::EvidenceTenantMismatch { .. }
    ));

    let stale = get_trust_center_evidence_detail(
        &mut model,
        detail_request("ev_alpha_stale", TrustCenterRole::TenantAdmin),
    )
    .expect_err("stale evidence cannot be read as detail/exportable proof");
    assert_eq!(stale.status_code(), 422);
    assert!(matches!(
        stale,
        TrustCenterApiError::EvidenceNotFresh { .. }
    ));

    let operator_only = get_trust_center_evidence_detail(
        &mut model,
        detail_request("ev_alpha_operator", TrustCenterRole::OyatieOperator),
    )
    .expect_err("operator-only raw detail remains inaccessible through customer API");
    assert_eq!(operator_only.status_code(), 403);
    assert!(matches!(
        operator_only,
        TrustCenterApiError::OperatorOnlyDetailDenied
    ));
}

#[test]
fn authorization_fixture_enforces_role_purpose_data_class_and_publishability() {
    let mut model = fixture_model();

    let reviewer_without_grant = get_trust_center_evidence_detail(
        &mut model,
        request_with_principal(
            principal(
                TrustCenterRole::SecurityComplianceReviewer,
                TrustCenterPurpose::EvidenceRead,
            ),
            authorization_for(
                "principal_security_compliance_reviewer",
                &[TRUST_CENTER_EVIDENCE_DETAIL_SURFACE],
                &[TrustCenterPurpose::EvidenceRead],
                &[TrustCenterDataClass::TenantTrustEvidence],
                &[TrustCenterPublishabilityState::PublishableCustomerSafe],
            ),
            TrustCenterEvidenceDetailQuery {
                asserted_tenant_id: None,
                evidence_id: "ev_alpha_current".to_string(),
            },
        ),
    )
    .expect_err("reviewer needs active grant for tenant evidence room access");
    assert!(matches!(
        reviewer_without_grant,
        TrustCenterApiError::RoleDenied { .. }
    ));

    let mut reviewer = principal(
        TrustCenterRole::SecurityComplianceReviewer,
        TrustCenterPurpose::EvidenceRead,
    );
    reviewer.access_grant_id = Some("grant_reviewer_1".to_string());
    reviewer.expires_at_trusted = Some(RETAIN.to_string());
    let regulated_denied = get_trust_center_evidence_detail(
        &mut model,
        request_with_principal(
            reviewer,
            authorization_for(
                "principal_security_compliance_reviewer",
                &[TRUST_CENTER_EVIDENCE_DETAIL_SURFACE],
                &[TrustCenterPurpose::EvidenceRead],
                &[TrustCenterDataClass::TenantTrustEvidence],
                &[TrustCenterPublishabilityState::PublishableCustomerSafe],
            ),
            TrustCenterEvidenceDetailQuery {
                asserted_tenant_id: None,
                evidence_id: "ev_alpha_regulated".to_string(),
            },
        ),
    )
    .expect_err("authorization fixture must deny ungranted regulated export data class");
    assert!(matches!(
        regulated_denied,
        TrustCenterApiError::DataClassDenied { .. }
    ));

    let wrong_purpose = get_trust_center_evidence_detail(
        &mut model,
        request(
            TrustCenterRole::TenantAdmin,
            TrustCenterPurpose::ComplianceRead,
            TRUST_CENTER_EVIDENCE_DETAIL_SURFACE,
            TrustCenterEvidenceDetailQuery {
                asserted_tenant_id: None,
                evidence_id: "ev_alpha_current".to_string(),
            },
        ),
    )
    .expect_err("endpoint-specific purpose is required");
    assert!(matches!(
        wrong_purpose,
        TrustCenterApiError::PurposeDenied { .. }
    ));
}

#[test]
fn control_sbom_compliance_export_and_access_audit_routes_fail_closed_and_emit_events() {
    let mut model = fixture_model();

    let controls = get_trust_center_control_freshness(
        &mut model,
        request(
            TrustCenterRole::TenantAdmin,
            TrustCenterPurpose::ControlEvidenceRead,
            TRUST_CENTER_CONTROL_FRESHNESS_SURFACE,
            TrustCenterControlFreshnessQuery::default(),
        ),
    )
    .expect("control freshness route succeeds for current evidence");
    assert_eq!(controls.records.len(), 1);
    assert_eq!(controls.records[0].source_evidence_ref, "ev_alpha_current");

    let sbom = get_trust_center_sbom_vex(
        &mut model,
        request(
            TrustCenterRole::TenantAdmin,
            TrustCenterPurpose::SecurityEvidenceRead,
            TRUST_CENTER_SBOM_VEX_SURFACE,
            TrustCenterSbomVexQuery::default(),
        ),
    )
    .expect("SBOM/VEX route succeeds");
    assert_eq!(sbom.records.len(), 1);
    assert!(!sbom.records[0].raw_scanner_output_exposed);
    assert!(!sbom.records[0].exploit_detail_exposed);

    let packs = get_trust_center_compliance_packs(
        &mut model,
        request(
            TrustCenterRole::TenantAdmin,
            TrustCenterPurpose::ComplianceRead,
            TRUST_CENTER_COMPLIANCE_PACKS_SURFACE,
            TrustCenterCompliancePackQuery::default(),
        ),
    )
    .expect("compliance-pack route succeeds");
    assert_eq!(packs.records[0].compliance_pack_id, "pack_soc2_ready");

    let export = create_trust_center_export_request(
        &mut model,
        request(
            TrustCenterRole::TenantAdmin,
            TrustCenterPurpose::ExportRequest,
            TRUST_CENTER_EXPORT_REQUEST_SURFACE,
            TrustCenterExportRequestInput {
                asserted_tenant_id: None,
                purpose: "customer_security_review".to_string(),
                framework: "SOC2-readiness".to_string(),
                time_window_start_trusted: "2026-06-01T00:00:00Z".to_string(),
                time_window_end_trusted: NOW.to_string(),
                evidence_ids: vec![
                    "ev_alpha_current".to_string(),
                    "ev_alpha_regulated".to_string(),
                ],
                expires_at_trusted: RETAIN.to_string(),
            },
        ),
    )
    .expect("export request stub succeeds for current tenant-scoped evidence");
    assert_eq!(export.approval_state, "operator_review_required");
    assert_eq!(
        export.manifest_ref, None,
        "stub must not fake an export package"
    );

    let stale_export = create_trust_center_export_request(
        &mut model,
        request(
            TrustCenterRole::TenantAdmin,
            TrustCenterPurpose::ExportRequest,
            TRUST_CENTER_EXPORT_REQUEST_SURFACE,
            TrustCenterExportRequestInput {
                asserted_tenant_id: None,
                purpose: "customer_security_review".to_string(),
                framework: "SOC2-readiness".to_string(),
                time_window_start_trusted: "2026-06-01T00:00:00Z".to_string(),
                time_window_end_trusted: NOW.to_string(),
                evidence_ids: vec!["ev_alpha_stale".to_string()],
                expires_at_trusted: RETAIN.to_string(),
            },
        ),
    )
    .expect_err("stale evidence cannot enter an export request");
    assert_eq!(stale_export.status_code(), 422);

    record_trust_center_access_grant_created(
        &mut model,
        request(
            TrustCenterRole::TenantAdmin,
            TrustCenterPurpose::GrantManagement,
            TRUST_CENTER_GRANT_WRITE_SURFACE,
            TrustCenterGrantEventInput {
                asserted_tenant_id: None,
                grant_id: "grant_reviewer_1".to_string(),
                reviewer_principal_id: "principal_reviewer".to_string(),
                expires_at_trusted: RETAIN.to_string(),
            },
        ),
    )
    .expect("grant audit event emits");
    record_trust_center_export_downloaded(
        &mut model,
        request(
            TrustCenterRole::TenantAdmin,
            TrustCenterPurpose::Download,
            TRUST_CENTER_EXPORT_DOWNLOAD_SURFACE,
            TrustCenterExportDownloadInput {
                asserted_tenant_id: None,
                export_request_id: export.export_request_id.clone(),
                artifact_ref: "auditor-room://stub".to_string(),
            },
        ),
    )
    .expect("download audit event emits");
    record_trust_center_publishability_state_changed(
        &mut model,
        request(
            TrustCenterRole::OyatieOperator,
            TrustCenterPurpose::PublishabilityReview,
            TRUST_CENTER_PUBLISHABILITY_SURFACE,
            TrustCenterPublishabilityDecisionInput {
                asserted_tenant_id: None,
                decision_id: "decision_publish_ev_alpha_current".to_string(),
                evidence_id: "ev_alpha_current".to_string(),
                previous_state: TrustCenterPublishabilityState::BlockedSecurityPrivacyReview,
                new_state: TrustCenterPublishabilityState::PublishableCustomerSafe,
                reason: "security/privacy review approved customer-safe summary".to_string(),
                expires_at_trusted_or_retention_until: RETAIN.to_string(),
            },
        ),
    )
    .expect("publishability decision emits append-only audit event");

    let audit = get_trust_center_access_audit(
        &mut model,
        request(
            TrustCenterRole::TenantAdmin,
            TrustCenterPurpose::AccessAuditRead,
            TRUST_CENTER_ACCESS_AUDIT_SURFACE,
            TrustCenterAccessAuditQuery::default(),
        ),
    )
    .expect("tenant admin can read trust-center access audit");
    let event_types = audit
        .records
        .iter()
        .map(|record| record.event_type.as_str())
        .collect::<Vec<_>>();
    assert!(event_types.contains(&"trust_center.evidence_export_requested"));
    assert!(event_types.contains(&"trust_center.access_grant_created"));
    assert!(event_types.contains(&"trust_center.evidence_export_downloaded"));
    assert!(event_types.contains(&"trust_center.publishability_state_changed"));
    assert!(model.emitted_audit_event_refs().len() >= 7);
}

#[test]
fn export_download_audit_fails_closed_for_cross_tenant_export_requests() {
    let mut model = fixture_model();
    let export = create_trust_center_export_request(
        &mut model,
        request(
            TrustCenterRole::TenantAdmin,
            TrustCenterPurpose::ExportRequest,
            TRUST_CENTER_EXPORT_REQUEST_SURFACE,
            TrustCenterExportRequestInput {
                asserted_tenant_id: None,
                purpose: "customer_security_review".to_string(),
                framework: "SOC2-readiness".to_string(),
                time_window_start_trusted: "2026-06-01T00:00:00Z".to_string(),
                time_window_end_trusted: NOW.to_string(),
                evidence_ids: vec!["ev_alpha_current".to_string()],
                expires_at_trusted: RETAIN.to_string(),
            },
        ),
    )
    .expect("tenant export request exists");

    let err = record_trust_center_export_downloaded(
        &mut model,
        request_for_tenant(
            OTHER_TENANT,
            TrustCenterRole::TenantAdmin,
            TrustCenterPurpose::Download,
            TRUST_CENTER_EXPORT_DOWNLOAD_SURFACE,
            TrustCenterExportDownloadInput {
                asserted_tenant_id: None,
                export_request_id: export.export_request_id,
                artifact_ref: "auditor-room://cross-tenant-stub".to_string(),
            },
        ),
    )
    .expect_err("download audit must reject cross-tenant export request ids");

    assert!(matches!(
        err,
        TrustCenterApiError::EvidenceTenantMismatch { .. }
    ));
}

#[test]
fn contract_replay_serializes_flat_common_required_fields() {
    let item = TrustCenterAccessAuditRecord {
        common: common(
            "audit_contract_replay",
            TRUST_CENTER_ACCESS_AUDIT_RECORD_TYPE,
            TENANT,
            TrustCenterDataClass::TenantTrustEvidence,
            TrustCenterFreshnessState::Current,
            TrustCenterPublishabilityState::TenantAdminOnly,
        ),
        event_type: "trust_center.evidence_item_viewed".to_string(),
        actor_principal_id: "principal_tenant_admin".to_string(),
        actor_role: TrustCenterRole::TenantAdmin,
        action: TRUST_CENTER_EVIDENCE_DETAIL_SURFACE.to_string(),
        target_record_id: Some("ev_alpha_current".to_string()),
        granted: true,
        occurred_at_trusted: NOW.to_string(),
        decision_id: "decision_contract_replay".to_string(),
    };
    let value = serde_json::to_value(item).expect("access audit serializes");
    for field in [
        "record_id",
        "record_type",
        "schema_version",
        "tenant_id",
        "audience_id",
        "source_system",
        "source_record_ref",
        "evidence_class",
        "data_class",
        "claim_tier",
        "freshness_state",
        "publishability_state",
        "redaction_policy_id",
        "audit_event_ref",
        "created_at_trusted",
        "expires_at_trusted_or_retention_until",
    ] {
        assert!(
            value.get(field).is_some(),
            "missing flattened common field {field}"
        );
    }
    assert_eq!(value["record_type"], TRUST_CENTER_ACCESS_AUDIT_RECORD_TYPE);
    assert_eq!(value["data_class"], "TENANT_TRUST_EVIDENCE");
}

fn fixture_model() -> TrustCenterReadModel {
    let mut model = TrustCenterReadModel::default();
    model
        .upsert_evidence_item(evidence_item(
            "ev_alpha_current",
            TENANT,
            TrustCenterDataClass::TenantTrustEvidence,
            TrustCenterFreshnessState::Current,
            TrustCenterPublishabilityState::PublishableCustomerSafe,
        ))
        .unwrap();
    model
        .upsert_evidence_item(evidence_item(
            "ev_alpha_stale",
            TENANT,
            TrustCenterDataClass::TenantTrustEvidence,
            TrustCenterFreshnessState::Stale,
            TrustCenterPublishabilityState::BlockedStaleEvidence,
        ))
        .unwrap();
    model
        .upsert_evidence_item(evidence_item(
            "ev_alpha_operator",
            TENANT,
            TrustCenterDataClass::OperatorSecurityInternal,
            TrustCenterFreshnessState::Current,
            TrustCenterPublishabilityState::OperatorOnly,
        ))
        .unwrap();
    model
        .upsert_evidence_item(evidence_item(
            "ev_alpha_regulated",
            TENANT,
            TrustCenterDataClass::RegulatedExportEvidence,
            TrustCenterFreshnessState::Current,
            TrustCenterPublishabilityState::TenantAdminOnly,
        ))
        .unwrap();
    model
        .upsert_evidence_item(evidence_item(
            "ev_other_current",
            OTHER_TENANT,
            TrustCenterDataClass::TenantTrustEvidence,
            TrustCenterFreshnessState::Current,
            TrustCenterPublishabilityState::PublishableCustomerSafe,
        ))
        .unwrap();
    model.upsert_control_freshness(control_record()).unwrap();
    model.upsert_sbom_vex(sbom_record()).unwrap();
    model.upsert_compliance_pack(pack_record()).unwrap();
    model
}

fn common(
    record_id: &str,
    record_type: &str,
    tenant_id: &str,
    data_class: TrustCenterDataClass,
    freshness_state: TrustCenterFreshnessState,
    publishability_state: TrustCenterPublishabilityState,
) -> TrustCenterCommonFields {
    TrustCenterCommonFields {
        record_id: record_id.to_string(),
        record_type: record_type.to_string(),
        schema_version: TRUST_CENTER_SCHEMA_VERSION,
        tenant_id: tenant_id.to_string(),
        audience_id: "aud_customer_trust".to_string(),
        source_system: "security_validation_controls".to_string(),
        source_record_ref: format!("source/{record_id}"),
        evidence_class: "security_validation_controls".to_string(),
        data_class,
        claim_tier: TrustCenterClaimTier::SpecReady,
        freshness_state,
        publishability_state,
        redaction_policy_id: "redact_customer_safe_v1".to_string(),
        audit_event_ref: format!("audit/{record_id}"),
        created_at_trusted: NOW.to_string(),
        expires_at_trusted_or_retention_until: RETAIN.to_string(),
    }
}

fn evidence_item(
    record_id: &str,
    tenant_id: &str,
    data_class: TrustCenterDataClass,
    freshness_state: TrustCenterFreshnessState,
    publishability_state: TrustCenterPublishabilityState,
) -> TrustCenterEvidenceItemRecord {
    TrustCenterEvidenceItemRecord {
        common: common(
            record_id,
            TRUST_CENTER_EVIDENCE_ITEM_RECORD_TYPE,
            tenant_id,
            data_class,
            freshness_state,
            publishability_state,
        ),
        title: format!("Evidence {record_id}"),
        customer_safe_summary: "Customer-safe security validation summary".to_string(),
        source_links: vec![format!("audit-chain://{record_id}")],
        compliance_pack_ids: vec!["pack_soc2_ready".to_string()],
        service_ids: vec!["svc_trust_center".to_string()],
        redacted_fields: vec![
            "raw_scanner_output".to_string(),
            "exploit_payload".to_string(),
        ],
        operator_only_detail_present: data_class == TrustCenterDataClass::OperatorSecurityInternal,
        raw_operator_payload_exposed: false,
    }
}

fn control_record() -> TrustCenterControlFreshnessRecord {
    TrustCenterControlFreshnessRecord {
        common: common(
            "ctrl_alpha_sast",
            TRUST_CENTER_CONTROL_FRESHNESS_RECORD_TYPE,
            TENANT,
            TrustCenterDataClass::TenantTrustEvidence,
            TrustCenterFreshnessState::Current,
            TrustCenterPublishabilityState::PublishableCustomerSafe,
        ),
        control_id: "control_sast".to_string(),
        lane_id: "security_validation_controls".to_string(),
        service_id: Some("svc_trust_center".to_string()),
        compliance_pack_ids: vec!["pack_soc2_ready".to_string()],
        last_observed_at_trusted: NOW.to_string(),
        stale_after_trusted: RETAIN.to_string(),
        source_evidence_ref: "ev_alpha_current".to_string(),
    }
}

fn sbom_record() -> TrustCenterSbomVexViewRecord {
    TrustCenterSbomVexViewRecord {
        common: common(
            "sbom_alpha_api",
            TRUST_CENTER_SBOM_VEX_RECORD_TYPE,
            TENANT,
            TrustCenterDataClass::TenantTrustEvidence,
            TrustCenterFreshnessState::Current,
            TrustCenterPublishabilityState::PublishableSummaryOnly,
        ),
        artifact_ref: "oci://registry.example/trust-center-api@sha256:abc".to_string(),
        signed_sbom_ref: Some("sbom://trust-center-api/sha256-abc".to_string()),
        vex_ref: Some("vex://trust-center-api/sha256-abc".to_string()),
        vulnerability_status_counts: BTreeMap::from([
            ("not_affected".to_string(), 12),
            ("fixed".to_string(), 3),
        ]),
        exception_refs: vec!["vex_exception_1".to_string()],
        remediation_sla_class: "standard".to_string(),
        raw_scanner_output_exposed: false,
        exploit_detail_exposed: false,
    }
}

fn pack_record() -> TrustCenterCompliancePackViewRecord {
    TrustCenterCompliancePackViewRecord {
        common: common(
            "pack_soc2_ready",
            TRUST_CENTER_COMPLIANCE_PACK_RECORD_TYPE,
            TENANT,
            TrustCenterDataClass::RegulatedExportEvidence,
            TrustCenterFreshnessState::Current,
            TrustCenterPublishabilityState::TenantAdminOnly,
        ),
        compliance_pack_id: "pack_soc2_ready".to_string(),
        version: "2026.07".to_string(),
        regulator_references: vec!["SOC2-readiness-non-certification".to_string()],
        data_classes: vec![
            TrustCenterDataClass::TenantTrustEvidence,
            TrustCenterDataClass::RegulatedExportEvidence,
        ],
        residency_summary: "home-region retained customer-safe summary".to_string(),
        retention_days: 400,
        dr_floor_ref: Some("dr-floor://soc2-ready".to_string()),
        breach_workflow_ref: Some("workflow://breach-notification".to_string()),
        activated: true,
    }
}

fn request<T>(
    role: TrustCenterRole,
    purpose: TrustCenterPurpose,
    endpoint: &str,
    payload: T,
) -> TrustCenterApiRequest<T> {
    request_for_tenant(TENANT, role, purpose, endpoint, payload)
}

fn request_for_tenant<T>(
    tenant_id: &str,
    role: TrustCenterRole,
    purpose: TrustCenterPurpose,
    endpoint: &str,
    payload: T,
) -> TrustCenterApiRequest<T> {
    let principal = principal_for_tenant(tenant_id, role, purpose);
    let principal_id = principal.principal_id.clone();
    request_with_principal_for_tenant(
        tenant_id,
        principal,
        authorization_for_tenant(
            tenant_id,
            &principal_id,
            &[endpoint],
            &[purpose],
            &all_data_classes(),
            &all_publishability_states(),
        ),
        payload,
    )
}

fn request_with_principal<T>(
    principal: TrustCenterPrincipal,
    authorization: TrustCenterAuthorizationDecision,
    payload: T,
) -> TrustCenterApiRequest<T> {
    request_with_principal_for_tenant(TENANT, principal, authorization, payload)
}

fn request_with_principal_for_tenant<T>(
    tenant_id: &str,
    principal: TrustCenterPrincipal,
    authorization: TrustCenterAuthorizationDecision,
    payload: T,
) -> TrustCenterApiRequest<T> {
    TrustCenterApiRequest {
        boundary: TrustCenterBoundaryContext {
            request_id: format!("req_{}", principal.principal_id),
            tenant_id: tenant_id.to_string(),
            occurred_at_trusted: NOW.to_string(),
        },
        principal: Some(principal),
        authorization,
        payload,
    }
}

fn detail_request(
    evidence_id: &str,
    role: TrustCenterRole,
) -> TrustCenterApiRequest<TrustCenterEvidenceDetailQuery> {
    request(
        role,
        TrustCenterPurpose::EvidenceRead,
        TRUST_CENTER_EVIDENCE_DETAIL_SURFACE,
        TrustCenterEvidenceDetailQuery {
            asserted_tenant_id: None,
            evidence_id: evidence_id.to_string(),
        },
    )
}

fn principal(role: TrustCenterRole, purpose: TrustCenterPurpose) -> TrustCenterPrincipal {
    principal_for_tenant(TENANT, role, purpose)
}

fn principal_for_tenant(
    tenant_id: &str,
    role: TrustCenterRole,
    purpose: TrustCenterPurpose,
) -> TrustCenterPrincipal {
    let principal_id = match role {
        TrustCenterRole::TenantAdmin => "principal_tenant_admin",
        TrustCenterRole::SecurityComplianceReviewer => "principal_security_compliance_reviewer",
        TrustCenterRole::OyatieOperator => "principal_oyatie_operator",
        TrustCenterRole::Auditor => "principal_auditor",
    };
    TrustCenterPrincipal {
        tenant_id: tenant_id.to_string(),
        principal_id: principal_id.to_string(),
        role,
        purpose,
        audience_id: "aud_customer_trust".to_string(),
        access_grant_id: None,
        expires_at_trusted: None,
    }
}

fn authorization_for(
    principal_id: &str,
    endpoints: &[&str],
    purposes: &[TrustCenterPurpose],
    data_classes: &[TrustCenterDataClass],
    publishability_states: &[TrustCenterPublishabilityState],
) -> TrustCenterAuthorizationDecision {
    authorization_for_tenant(
        TENANT,
        principal_id,
        endpoints,
        purposes,
        data_classes,
        publishability_states,
    )
}

fn authorization_for_tenant(
    tenant_id: &str,
    principal_id: &str,
    endpoints: &[&str],
    purposes: &[TrustCenterPurpose],
    data_classes: &[TrustCenterDataClass],
    publishability_states: &[TrustCenterPublishabilityState],
) -> TrustCenterAuthorizationDecision {
    TrustCenterAuthorizationDecision {
        tenant_id: tenant_id.to_string(),
        principal_id: principal_id.to_string(),
        decision_id: format!("authz_decision_{principal_id}"),
        allowed_endpoints: endpoints
            .iter()
            .map(|endpoint| (*endpoint).to_string())
            .collect(),
        allowed_purposes: purposes.to_vec(),
        allowed_data_classes: data_classes.to_vec(),
        allowed_publishability_states: publishability_states.to_vec(),
    }
}

fn assert_tenant_assertion_mismatch(err: TrustCenterApiError) {
    assert_eq!(err.status_code(), 400);
    assert!(matches!(
        err,
        TrustCenterApiError::TenantAssertionMismatch { .. }
    ));
}

fn all_data_classes() -> Vec<TrustCenterDataClass> {
    vec![
        TrustCenterDataClass::PublicStatus,
        TrustCenterDataClass::TenantTrustEvidence,
        TrustCenterDataClass::RegulatedExportEvidence,
        TrustCenterDataClass::OperatorSecurityInternal,
    ]
}

fn all_publishability_states() -> Vec<TrustCenterPublishabilityState> {
    vec![
        TrustCenterPublishabilityState::PublishableCustomerSafe,
        TrustCenterPublishabilityState::PublishableSummaryOnly,
        TrustCenterPublishabilityState::TenantAdminOnly,
        TrustCenterPublishabilityState::OperatorOnly,
        TrustCenterPublishabilityState::BlockedMissingEvidence,
        TrustCenterPublishabilityState::BlockedStaleEvidence,
        TrustCenterPublishabilityState::BlockedSecurityPrivacyReview,
        TrustCenterPublishabilityState::NotApplicableWithPolicyReason,
    ]
}
