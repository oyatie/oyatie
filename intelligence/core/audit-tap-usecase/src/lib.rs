//! Intelligence audit-tap usecase foundation.
//!
//! This crate builds metadata-only Intelligence audit tap records and hands
//! them to an injected audit-chain emitter port. It owns idempotency,
//! audit-envelope validation, evidence-ref canonicalization, and seal-receipt
//! binding checks, but deliberately performs no Ed25519 signing, Merkle-tree
//! construction, durable audit-chain writes, filesystem, network, or provider
//! calls.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IntelligenceAuditEventClass {
    ByokCredentialRotated,
    CredentialHandleResolved,
    DispatchCompleted,
    DispatchRequested,
    GuardrailDenied,
    ProviderDispatchCompleted,
    RouteSelected,
}

impl IntelligenceAuditEventClass {
    pub fn wire_label(self) -> &'static str {
        match self {
            Self::ByokCredentialRotated => "intelligence.byok_credential_rotated",
            Self::CredentialHandleResolved => "intelligence.credential_handle_resolved",
            Self::DispatchCompleted => "intelligence.dispatch_completed",
            Self::DispatchRequested => "intelligence.dispatch_requested",
            Self::GuardrailDenied => "intelligence.guardrail_denied",
            Self::ProviderDispatchCompleted => "intelligence.provider_dispatch_completed",
            Self::RouteSelected => "intelligence.route_selected",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InfrastructureProvider {
    Aws,
    Azure,
    Colo,
    Gcp,
    Oci,
    OnPrem,
    OyatieOwn,
    Sovereign,
}

impl InfrastructureProvider {
    pub fn wire_label(self) -> &'static str {
        match self {
            Self::Aws => "aws",
            Self::Azure => "azure",
            Self::Colo => "colo",
            Self::Gcp => "gcp",
            Self::Oci => "oci",
            Self::OnPrem => "on_prem",
            Self::OyatieOwn => "oyatie_own",
            Self::Sovereign => "sovereign",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CarbonIntensitySource {
    CacheHit,
    CacheMissFallback,
    ElectricityMaps,
    ProviderGridAverage,
    SovereignPublished,
}

impl CarbonIntensitySource {
    pub fn wire_label(self) -> &'static str {
        match self {
            Self::CacheHit => "cache_hit",
            Self::CacheMissFallback => "cache_miss_fallback",
            Self::ElectricityMaps => "electricitymaps",
            Self::ProviderGridAverage => "provider_grid_avg",
            Self::SovereignPublished => "sovereign_published",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntelligenceAuditTapInput {
    pub idempotency_key: String,                  // data_class: INTERNAL_ONLY
    pub audit_id: String,                         // data_class: INTERNAL_ONLY
    pub event_id: String,                         // data_class: INTERNAL_ONLY
    pub tenant_id: String,                        // data_class: INTERNAL_ONLY
    pub producer_id: String,                      // data_class: INTERNAL_ONLY
    pub trace_id: String,                         // data_class: INTERNAL_ONLY
    pub span_id: String,                          // data_class: INTERNAL_ONLY
    pub cell_id: String,                          // data_class: INTERNAL_ONLY
    pub jurisdiction_code: String,                // data_class: INTERNAL_ONLY
    pub sub_scope_path: String,                   // data_class: INTERNAL_ONLY
    pub hlc_timestamp: String,                    // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64,           // data_class: INTERNAL_ONLY
    pub event_class: IntelligenceAuditEventClass, // data_class: INTERNAL_ONLY
    pub action: String,                           // data_class: INTERNAL_ONLY
    pub decision: String,                         // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,             // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,              // data_class: INTERNAL_ONLY
    pub route_evidence_refs: Vec<String>,         // data_class: INTERNAL_ONLY
    pub guardrail_evidence_refs: Vec<String>,     // data_class: INTERNAL_ONLY
    pub credential_evidence_ref: Option<String>,  // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: Option<String>,    // data_class: INTERNAL_ONLY
    pub content_ref: Option<String>,              // data_class: INTERNAL_ONLY
    pub output_ref: Option<String>,               // data_class: INTERNAL_ONLY
    pub cost_usd_minor_units: i64,                // data_class: INTERNAL_ONLY
    pub co2_grams: u64,                           // data_class: INTERNAL_ONLY
    pub watt_hours: u64,                          // data_class: INTERNAL_ONLY
    pub infrastructure_provider: InfrastructureProvider, // data_class: INTERNAL_ONLY
    pub region: String,                           // data_class: INTERNAL_ONLY
    pub carbon_intensity_source: CarbonIntensitySource, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditTapRecordRequest {
    pub audit_id: String,                                // data_class: INTERNAL_ONLY
    pub event_id: String,                                // data_class: INTERNAL_ONLY
    pub tenant_id: String,                               // data_class: INTERNAL_ONLY
    pub producer_id: String,                             // data_class: INTERNAL_ONLY
    pub source_microservice: String,                     // data_class: INTERNAL_ONLY
    pub event_class: IntelligenceAuditEventClass,        // data_class: INTERNAL_ONLY
    pub action: String,                                  // data_class: INTERNAL_ONLY
    pub decision: String,                                // data_class: INTERNAL_ONLY
    pub trace_id: String,                                // data_class: INTERNAL_ONLY
    pub span_id: String,                                 // data_class: INTERNAL_ONLY
    pub cell_id: String,                                 // data_class: INTERNAL_ONLY
    pub jurisdiction_code: String,                       // data_class: INTERNAL_ONLY
    pub sub_scope_path: String,                          // data_class: INTERNAL_ONLY
    pub hlc_timestamp: String,                           // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64,                  // data_class: INTERNAL_ONLY
    pub cost_usd_minor_units: i64,                       // data_class: INTERNAL_ONLY
    pub co2_grams: u64,                                  // data_class: INTERNAL_ONLY
    pub watt_hours: u64,                                 // data_class: INTERNAL_ONLY
    pub infrastructure_provider: InfrastructureProvider, // data_class: INTERNAL_ONLY
    pub region: String,                                  // data_class: INTERNAL_ONLY
    pub carbon_intensity_source: CarbonIntensitySource,  // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                      // data_class: INTERNAL_ONLY
    pub resource_refs: Vec<String>,                      // data_class: INTERNAL_ONLY
    pub canonical_fingerprint: String,                   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditTapSealReceipt {
    pub audit_id: String,           // data_class: INTERNAL_ONLY
    pub event_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub sequence: u64,              // data_class: INTERNAL_ONLY
    pub chain_ref: String,          // data_class: INTERNAL_ONLY
    pub merkle_root_ref: String,    // data_class: INTERNAL_ONLY
    pub signature_ref: String,      // data_class: INTERNAL_ONLY
    pub outbox_ref: String,         // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditTapEmissionFailure {
    pub reason: String,       // data_class: INTERNAL_ONLY
    pub evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntelligenceAuditTapStatus {
    Sealed,
    Denied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntelligenceAuditTapDenialKind {
    EmitterFailed,
    IdempotencyConflict,
    InvalidInput,
    SealBindingDrift,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntelligenceAuditTapReceipt {
    pub idempotency_key: String,            // data_class: INTERNAL_ONLY
    pub audit_id: String,                   // data_class: INTERNAL_ONLY
    pub event_id: String,                   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub status: IntelligenceAuditTapStatus, // data_class: PUBLIC
    pub denial_kind: Option<IntelligenceAuditTapDenialKind>, // data_class: INTERNAL_ONLY
    pub denial_reasons: Vec<String>,        // data_class: INTERNAL_ONLY
    pub seal: Option<AuditTapSealReceipt>,  // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,         // data_class: INTERNAL_ONLY
}

pub trait AuditTapEmitterPort {
    fn emit(
        &mut self,
        request: AuditTapRecordRequest,
    ) -> Result<AuditTapSealReceipt, AuditTapEmissionFailure>;
}

#[derive(Debug)]
pub struct IntelligenceAuditTapUsecase<E> {
    emitter: E,
    receipts_by_idempotency_key: BTreeMap<String, (AuditTapIntent, IntelligenceAuditTapReceipt)>,
}

impl<E> IntelligenceAuditTapUsecase<E>
where
    E: AuditTapEmitterPort,
{
    pub fn new(emitter: E) -> Self {
        Self {
            emitter,
            receipts_by_idempotency_key: BTreeMap::new(),
        }
    }

    pub fn into_inner(self) -> E {
        self.emitter
    }

    pub fn emit(&mut self, input: IntelligenceAuditTapInput) -> IntelligenceAuditTapReceipt {
        let invalid = invalid_input_reasons(&input);
        if !invalid.is_empty() {
            return denied_receipt_from_parts(
                safe_receipt_metadata(&input.idempotency_key, "redacted-invalid-idempotency-key"),
                safe_receipt_metadata(&input.audit_id, "redacted-invalid-audit-id"),
                safe_receipt_metadata(&input.event_id, "redacted-invalid-event-id"),
                safe_receipt_tenant(&input.tenant_id),
                IntelligenceAuditTapDenialKind::InvalidInput,
                invalid,
                vec!["validation:intelligence-audit-tap-input".to_owned()],
            );
        }

        let request = record_request_for(&input);
        let intent = AuditTapIntent {
            canonical_fingerprint: request.canonical_fingerprint.clone(),
        };

        if let Some((existing_intent, existing_receipt)) =
            self.receipts_by_idempotency_key.get(&input.idempotency_key)
        {
            if existing_intent == &intent {
                return existing_receipt.clone();
            }
            return denied_receipt_from_validated_input(
                &input,
                IntelligenceAuditTapDenialKind::IdempotencyConflict,
                vec!["idempotency key already used for different audit tap intent".to_owned()],
                vec![
                    input.request_evidence_ref.clone(),
                    "validation:intelligence-audit-tap-idempotency-conflict".to_owned(),
                ],
            );
        }

        let seal = match self.emitter.emit(request.clone()) {
            Ok(seal) => seal,
            Err(failure) => {
                return denied_receipt_from_validated_input(
                    &input,
                    IntelligenceAuditTapDenialKind::EmitterFailed,
                    vec![safe_failure_reason(&failure.reason)],
                    vec![
                        input.request_evidence_ref.clone(),
                        safe_evidence_ref(&failure.evidence_ref),
                    ],
                );
            }
        };

        let seal_binding_reasons = seal_binding_reasons(&input, &seal);
        if !seal_binding_reasons.is_empty() {
            return denied_receipt_from_validated_input(
                &input,
                IntelligenceAuditTapDenialKind::SealBindingDrift,
                seal_binding_reasons,
                vec![
                    input.request_evidence_ref.clone(),
                    "validation:intelligence-audit-tap-seal-binding".to_owned(),
                ],
            );
        }

        let receipt = IntelligenceAuditTapReceipt {
            idempotency_key: input.idempotency_key.clone(),
            audit_id: input.audit_id.clone(),
            event_id: input.event_id.clone(),
            tenant_id: input.tenant_id.clone(),
            status: IntelligenceAuditTapStatus::Sealed,
            denial_kind: None,
            denial_reasons: Vec::new(),
            seal: Some(seal.clone()),
            evidence_refs: sorted_unique([request.evidence_refs, seal.evidence_refs].concat()),
        };
        self.receipts_by_idempotency_key
            .insert(input.idempotency_key.clone(), (intent, receipt.clone()));
        receipt
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AuditTapIntent {
    canonical_fingerprint: String,
}

fn record_request_for(input: &IntelligenceAuditTapInput) -> AuditTapRecordRequest {
    let evidence_refs = sorted_unique(
        [
            vec![
                input.request_evidence_ref.clone(),
                input.policy_decision_ref.clone(),
            ],
            input.route_evidence_refs.clone(),
            input.guardrail_evidence_refs.clone(),
            optional_ref_vec(input.credential_evidence_ref.as_ref()),
            optional_ref_vec(input.provider_evidence_ref.as_ref()),
        ]
        .concat(),
    );
    let resource_refs = sorted_unique(
        [
            optional_ref_vec(input.content_ref.as_ref()),
            optional_ref_vec(input.output_ref.as_ref()),
        ]
        .concat(),
    );
    let canonical_fingerprint = canonical_fingerprint(input, &evidence_refs, &resource_refs);

    AuditTapRecordRequest {
        audit_id: input.audit_id.clone(),
        event_id: input.event_id.clone(),
        tenant_id: input.tenant_id.clone(),
        producer_id: input.producer_id.clone(),
        source_microservice: "intelligence".to_owned(),
        event_class: input.event_class,
        action: input.action.clone(),
        decision: input.decision.clone(),
        trace_id: input.trace_id.clone(),
        span_id: input.span_id.clone(),
        cell_id: input.cell_id.clone(),
        jurisdiction_code: input.jurisdiction_code.clone(),
        sub_scope_path: input.sub_scope_path.clone(),
        hlc_timestamp: input.hlc_timestamp.clone(),
        occurred_at_epoch_seconds: input.occurred_at_epoch_seconds,
        cost_usd_minor_units: input.cost_usd_minor_units,
        co2_grams: input.co2_grams,
        watt_hours: input.watt_hours,
        infrastructure_provider: input.infrastructure_provider,
        region: input.region.clone(),
        carbon_intensity_source: input.carbon_intensity_source,
        evidence_refs,
        resource_refs,
        canonical_fingerprint,
    }
}

fn canonical_fingerprint(
    input: &IntelligenceAuditTapInput,
    evidence_refs: &[String],
    resource_refs: &[String],
) -> String {
    let entries = [
        canonical_entry("schema", "oyatie/log/v2"),
        canonical_entry("audit_id", &input.audit_id),
        canonical_entry("event_id", &input.event_id),
        canonical_entry("tenant_id", &input.tenant_id),
        canonical_entry("producer_id", &input.producer_id),
        canonical_entry("source_microservice", "intelligence"),
        canonical_entry("event_class", input.event_class.wire_label()),
        canonical_entry("action", &input.action),
        canonical_entry("decision", &input.decision),
        canonical_entry("trace_id", &input.trace_id),
        canonical_entry("span_id", &input.span_id),
        canonical_entry("cell_id", &input.cell_id),
        canonical_entry("jurisdiction_code", &input.jurisdiction_code),
        canonical_entry("sub_scope_path", &input.sub_scope_path),
        canonical_entry("hlc_timestamp", &input.hlc_timestamp),
        canonical_entry(
            "occurred_at_epoch_seconds",
            &input.occurred_at_epoch_seconds.to_string(),
        ),
        canonical_entry(
            "cost_usd_minor_units",
            &input.cost_usd_minor_units.to_string(),
        ),
        canonical_entry("co2_grams", &input.co2_grams.to_string()),
        canonical_entry("watt_hours", &input.watt_hours.to_string()),
        canonical_entry("provider", input.infrastructure_provider.wire_label()),
        canonical_entry("region", &input.region),
        canonical_entry(
            "carbon_intensity_source",
            input.carbon_intensity_source.wire_label(),
        ),
        canonical_vec_entry("evidence_refs", evidence_refs),
        canonical_vec_entry("resource_refs", resource_refs),
    ];
    entries.concat()
}

fn invalid_input_reasons(input: &IntelligenceAuditTapInput) -> Vec<String> {
    let mut reasons = Vec::new();
    require_metadata_ref("idempotency key", &input.idempotency_key, &mut reasons);
    require_metadata_ref("audit id", &input.audit_id, &mut reasons);
    require_metadata_ref("event id", &input.event_id, &mut reasons);
    require_tenant_id(&input.tenant_id, &mut reasons);
    require_metadata_ref("producer id", &input.producer_id, &mut reasons);
    require_metadata_ref("trace id", &input.trace_id, &mut reasons);
    require_metadata_ref("span id", &input.span_id, &mut reasons);
    require_metadata_ref("cell id", &input.cell_id, &mut reasons);
    require_metadata_ref("jurisdiction code", &input.jurisdiction_code, &mut reasons);
    require_metadata_ref("sub-scope path", &input.sub_scope_path, &mut reasons);
    require_hlc_timestamp(&input.hlc_timestamp, &mut reasons);
    require_nonzero(
        input.occurred_at_epoch_seconds,
        "occurred_at_epoch_seconds",
        &mut reasons,
    );
    require_metadata_ref("action", &input.action, &mut reasons);
    require_metadata_ref("decision", &input.decision, &mut reasons);
    require_evidence_ref(
        "request evidence ref",
        &input.request_evidence_ref,
        &mut reasons,
    );
    require_evidence_ref(
        "policy decision ref",
        &input.policy_decision_ref,
        &mut reasons,
    );
    validate_evidence_refs(
        "route evidence ref",
        &input.route_evidence_refs,
        &mut reasons,
    );
    validate_evidence_refs(
        "guardrail evidence ref",
        &input.guardrail_evidence_refs,
        &mut reasons,
    );
    require_optional_evidence_ref(
        "credential evidence ref",
        input.credential_evidence_ref.as_deref(),
        &mut reasons,
    );
    require_optional_evidence_ref(
        "provider evidence ref",
        input.provider_evidence_ref.as_deref(),
        &mut reasons,
    );
    require_optional_resource_ref("content ref", input.content_ref.as_deref(), &mut reasons);
    require_optional_resource_ref("output ref", input.output_ref.as_deref(), &mut reasons);
    if input.cost_usd_minor_units < 0 {
        reasons.push("cost_usd_minor_units must be non-negative".to_owned());
    }
    require_metadata_ref("region", &input.region, &mut reasons);
    sorted_unique(reasons)
}

fn seal_binding_reasons(
    input: &IntelligenceAuditTapInput,
    seal: &AuditTapSealReceipt,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if seal.audit_id != input.audit_id {
        reasons.push("seal audit_id does not match request".to_owned());
    }
    if seal.event_id != input.event_id {
        reasons.push("seal event_id does not match request".to_owned());
    }
    if seal.tenant_id != input.tenant_id {
        reasons.push("seal tenant_id does not match request".to_owned());
    }
    if seal.sequence == 0 {
        reasons.push("seal sequence must be nonzero".to_owned());
    }
    require_prefix_ref(
        "seal chain ref",
        &seal.chain_ref,
        &["audit-chain://", "chain://"],
        &mut reasons,
    );
    require_prefix_ref(
        "seal merkle root ref",
        &seal.merkle_root_ref,
        &["merkle://", "merkle-sha256:"],
        &mut reasons,
    );
    require_prefix_ref(
        "seal signature ref",
        &seal.signature_ref,
        &["sigref://", "signature://"],
        &mut reasons,
    );
    require_prefix_ref(
        "seal outbox ref",
        &seal.outbox_ref,
        &["outbox://", "audit-outbox://"],
        &mut reasons,
    );
    require_evidence_refs("seal evidence ref", &seal.evidence_refs, &mut reasons);
    sorted_unique(reasons)
}

fn denied_receipt_from_validated_input(
    input: &IntelligenceAuditTapInput,
    denial_kind: IntelligenceAuditTapDenialKind,
    denial_reasons: Vec<String>,
    evidence_refs: Vec<String>,
) -> IntelligenceAuditTapReceipt {
    denied_receipt_from_parts(
        input.idempotency_key.clone(),
        input.audit_id.clone(),
        input.event_id.clone(),
        input.tenant_id.clone(),
        denial_kind,
        denial_reasons,
        evidence_refs,
    )
}

fn denied_receipt_from_parts(
    idempotency_key: String,
    audit_id: String,
    event_id: String,
    tenant_id: String,
    denial_kind: IntelligenceAuditTapDenialKind,
    denial_reasons: Vec<String>,
    evidence_refs: Vec<String>,
) -> IntelligenceAuditTapReceipt {
    IntelligenceAuditTapReceipt {
        idempotency_key,
        audit_id,
        event_id,
        tenant_id,
        status: IntelligenceAuditTapStatus::Denied,
        denial_kind: Some(denial_kind),
        denial_reasons: sorted_unique(denial_reasons),
        seal: None,
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn optional_ref_vec(value: Option<&String>) -> Vec<String> {
    value.into_iter().cloned().collect()
}

fn canonical_entry(label: &str, value: &str) -> String {
    format!("{}:{}{}:{}", label.len(), label, value.len(), value)
}

fn canonical_vec_entry(label: &str, values: &[String]) -> String {
    let mut encoded = canonical_entry(label, &values.len().to_string());
    for value in values {
        encoded.push_str(&canonical_entry("item", value));
    }
    encoded
}

fn safe_receipt_metadata(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || value != trimmed
        || contains_whitespace(trimmed)
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
    {
        fallback.to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn safe_receipt_tenant(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || value != trimmed
        || !trimmed.starts_with("ten_")
        || contains_whitespace(trimmed)
        || trimmed.contains('/')
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
    {
        "redacted-invalid-tenant-id".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn require_tenant_id(value: &str, reasons: &mut Vec<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        reasons.push("tenant id is required".to_owned());
    } else if value != trimmed
        || !trimmed.starts_with("ten_")
        || contains_whitespace(trimmed)
        || trimmed.contains('/')
    {
        reasons.push("tenant id is invalid".to_owned());
    }
}

fn require_hlc_timestamp(value: &str, reasons: &mut Vec<String>) {
    require_metadata_ref("hlc timestamp", value, reasons);
    if !value.contains("/lc=") {
        reasons.push("hlc timestamp must include logical counter".to_owned());
    }
}

fn require_nonzero(value: u64, label: &str, reasons: &mut Vec<String>) {
    if value == 0 {
        reasons.push(format!("{label} must be nonzero"));
    }
}

fn require_metadata_ref(label: &str, value: &str, reasons: &mut Vec<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        reasons.push(format!("{label} is required"));
    } else if value != trimmed
        || contains_whitespace(trimmed)
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
    {
        reasons.push(format!("{label} must be audit-safe metadata"));
    }
}

fn require_evidence_refs(label: &str, refs: &[String], reasons: &mut Vec<String>) {
    if refs.is_empty() {
        reasons.push(format!("{label} is required"));
    }
    validate_evidence_refs(label, refs, reasons);
}

fn validate_evidence_refs(label: &str, refs: &[String], reasons: &mut Vec<String>) {
    for value in refs {
        require_evidence_ref(label, value, reasons);
    }
}

fn require_optional_evidence_ref(label: &str, value: Option<&str>, reasons: &mut Vec<String>) {
    if let Some(value) = value {
        require_evidence_ref(label, value, reasons);
    }
}

fn require_evidence_ref(label: &str, value: &str, reasons: &mut Vec<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        reasons.push(format!("{label} is required"));
    } else if value != trimmed
        || contains_whitespace(trimmed)
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
        || !trimmed.contains(':')
    {
        reasons.push(format!("{label} must be an opaque evidence ref"));
    }
}

fn require_optional_resource_ref(label: &str, value: Option<&str>, reasons: &mut Vec<String>) {
    if let Some(value) = value {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            reasons.push(format!("{label} cannot be blank"));
        } else if value != trimmed
            || contains_whitespace(trimmed)
            || contains_raw_secret_material(trimmed)
            || contains_raw_content_material(trimmed)
            || !trimmed.contains("://")
        {
            reasons.push(format!("{label} must be an opaque resource ref"));
        }
    }
}

fn require_prefix_ref(label: &str, value: &str, prefixes: &[&str], reasons: &mut Vec<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        reasons.push(format!("{label} is required"));
    } else if value != trimmed
        || contains_whitespace(trimmed)
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
        || !prefixes.iter().any(|prefix| trimmed.starts_with(prefix))
    {
        reasons.push(format!("{label} must be an opaque seal ref"));
    }
}

fn safe_evidence_ref(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "audit-tap:missing-evidence-ref".to_owned()
    } else if value != trimmed
        || contains_whitespace(trimmed)
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
        || !trimmed.contains(':')
    {
        "audit-tap:unsafe-evidence-ref".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn safe_failure_reason(_value: &str) -> String {
    "audit tap emitter failed".to_owned()
}

fn contains_whitespace(value: &str) -> bool {
    value.chars().any(char::is_whitespace)
}

fn contains_raw_secret_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("sk-")
        || lower.contains("sk-")
        || lower.contains("bearer")
        || lower.contains("authorization:")
        || lower.contains("api_key=")
        || lower.contains("openai_api_key")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw model")
        || lower.contains("write an email")
        || lower.contains("customer message")
        || lower.contains("model answer")
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakeAuditTapEmitter {
        calls: usize,
        requests: Vec<AuditTapRecordRequest>,
    }

    impl AuditTapEmitterPort for FakeAuditTapEmitter {
        fn emit(
            &mut self,
            request: AuditTapRecordRequest,
        ) -> Result<AuditTapSealReceipt, AuditTapEmissionFailure> {
            self.calls += 1;
            self.requests.push(request.clone());
            Ok(AuditTapSealReceipt {
                audit_id: request.audit_id,
                event_id: request.event_id,
                tenant_id: request.tenant_id,
                sequence: self.calls as u64,
                chain_ref: "audit-chain://intelligence/ten_a/2026-05-23".to_owned(),
                merkle_root_ref: format!("merkle://intelligence/root/{}", self.calls),
                signature_ref: format!("sigref://ed25519/intelligence/{}", self.calls),
                outbox_ref: format!("outbox://audit-tap/{}", self.calls),
                evidence_refs: vec![format!("seal:{}", self.calls)],
            })
        }
    }

    fn input(idempotency_key: &str) -> IntelligenceAuditTapInput {
        IntelligenceAuditTapInput {
            idempotency_key: idempotency_key.to_owned(),
            audit_id: "audit_01".to_owned(),
            event_id: "evt_01".to_owned(),
            tenant_id: "ten_a".to_owned(),
            producer_id: "intelligence-dispatch".to_owned(),
            trace_id: "trace-01".to_owned(),
            span_id: "span-01".to_owned(),
            cell_id: "cell-us-east-1a".to_owned(),
            jurisdiction_code: "US".to_owned(),
            sub_scope_path: "intelligence/dispatch/openai".to_owned(),
            hlc_timestamp: "2026-05-23T11:00:00Z/lc=1".to_owned(),
            occurred_at_epoch_seconds: 1_779_538_800,
            event_class: IntelligenceAuditEventClass::DispatchCompleted,
            action: "provider_dispatch".to_owned(),
            decision: "allow".to_owned(),
            request_evidence_ref: "req:audit-tap".to_owned(),
            policy_decision_ref: "cedar:decision:1".to_owned(),
            route_evidence_refs: vec!["route:openai".to_owned()],
            guardrail_evidence_refs: vec!["guardrail:allow".to_owned()],
            credential_evidence_ref: Some("credential:handle:1".to_owned()),
            provider_evidence_ref: Some("openai:responses:resp_1".to_owned()),
            content_ref: Some("contentref://prompt/1".to_owned()),
            output_ref: Some("contentref://output/1".to_owned()),
            cost_usd_minor_units: 1,
            co2_grams: 2,
            watt_hours: 3,
            infrastructure_provider: InfrastructureProvider::OyatieOwn,
            region: "oyatie_own:iad-1".to_owned(),
            carbon_intensity_source: CarbonIntensitySource::ProviderGridAverage,
        }
    }

    #[test]
    fn emits_metadata_only_audit_tap_record_with_seal_receipt() {
        let mut usecase = IntelligenceAuditTapUsecase::new(FakeAuditTapEmitter::default());

        let receipt = usecase.emit(input("idem-1"));
        let emitter = usecase.into_inner();
        let request = emitter.requests.first().expect("request recorded");
        let debug = format!("{request:?}{receipt:?}");

        assert_eq!(receipt.status, IntelligenceAuditTapStatus::Sealed);
        assert_eq!(request.source_microservice, "intelligence");
        assert_eq!(
            request.event_class.wire_label(),
            "intelligence.dispatch_completed"
        );
        assert_eq!(
            request.evidence_refs,
            vec![
                "cedar:decision:1".to_owned(),
                "credential:handle:1".to_owned(),
                "guardrail:allow".to_owned(),
                "openai:responses:resp_1".to_owned(),
                "req:audit-tap".to_owned(),
                "route:openai".to_owned(),
            ]
        );
        assert!(request.canonical_fingerprint.contains("oyatie/log/v2"));
        assert!(request.canonical_fingerprint.contains("oyatie_own"));
        assert!(!debug.contains("sk-"));
        assert!(!debug.contains("write an email"));
        assert!(!debug.contains("raw model answer"));
    }

    #[test]
    fn idempotent_replay_returns_original_and_conflict_denies_without_second_emit() {
        let mut usecase = IntelligenceAuditTapUsecase::new(FakeAuditTapEmitter::default());

        let first = usecase.emit(input("idem-1"));
        let replay = usecase.emit(input("idem-1"));
        let mut drifted = input("idem-1");
        drifted.decision = "deny".to_owned();
        let conflict = usecase.emit(drifted);
        let emitter = usecase.into_inner();

        assert_eq!(first, replay);
        assert_eq!(conflict.status, IntelligenceAuditTapStatus::Denied);
        assert_eq!(
            conflict.denial_kind,
            Some(IntelligenceAuditTapDenialKind::IdempotencyConflict)
        );
        assert_eq!(emitter.calls, 1);
    }

    #[test]
    fn invalid_raw_refs_deny_before_emitter_call() {
        let mut usecase = IntelligenceAuditTapUsecase::new(FakeAuditTapEmitter::default());
        let mut bad = input("idem-bad");
        bad.content_ref = Some("write an email to the customer".to_owned());
        bad.provider_evidence_ref = Some("Authorization: Bearer sk-test".to_owned());

        let receipt = usecase.emit(bad);
        let emitter = usecase.into_inner();

        assert_eq!(receipt.status, IntelligenceAuditTapStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(IntelligenceAuditTapDenialKind::InvalidInput)
        );
        assert_eq!(emitter.calls, 0);
    }

    #[test]
    fn invalid_secret_shaped_identity_fields_are_redacted_in_denial_receipt() {
        let mut usecase = IntelligenceAuditTapUsecase::new(FakeAuditTapEmitter::default());
        let mut bad = input("sk-test-idem");
        bad.audit_id = "Bearer raw audit token".to_owned();
        bad.event_id = "write an email to the customer".to_owned();
        bad.tenant_id = "sk-test-tenant".to_owned();

        let receipt = usecase.emit(bad);
        let debug = format!("{receipt:?}");

        assert_eq!(receipt.status, IntelligenceAuditTapStatus::Denied);
        assert_eq!(receipt.idempotency_key, "redacted-invalid-idempotency-key");
        assert_eq!(receipt.audit_id, "redacted-invalid-audit-id");
        assert_eq!(receipt.event_id, "redacted-invalid-event-id");
        assert_eq!(receipt.tenant_id, "redacted-invalid-tenant-id");
        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("Bearer"));
        assert!(!debug.contains("write an email"));
    }

    #[test]
    fn delimiter_shaped_intent_changes_do_not_collide_under_same_idempotency_key() {
        let mut usecase = IntelligenceAuditTapUsecase::new(FakeAuditTapEmitter::default());
        let mut first = input("idem-delimiter");
        first.audit_id = "audit_01|event_id=evt_02".to_owned();
        first.event_id = "evt_01".to_owned();
        let mut second = input("idem-delimiter");
        second.audit_id = "audit_01".to_owned();
        second.event_id = "evt_02|event_id=evt_01".to_owned();

        let first_receipt = usecase.emit(first);
        let conflict = usecase.emit(second);
        let emitter = usecase.into_inner();

        assert_eq!(first_receipt.status, IntelligenceAuditTapStatus::Sealed);
        assert_eq!(conflict.status, IntelligenceAuditTapStatus::Denied);
        assert_eq!(
            conflict.denial_kind,
            Some(IntelligenceAuditTapDenialKind::IdempotencyConflict)
        );
        assert_eq!(emitter.calls, 1);
    }

    struct FailingEmitter;

    impl AuditTapEmitterPort for FailingEmitter {
        fn emit(
            &mut self,
            _request: AuditTapRecordRequest,
        ) -> Result<AuditTapSealReceipt, AuditTapEmissionFailure> {
            Err(AuditTapEmissionFailure {
                reason: "write an email to the customer".to_owned(),
                evidence_ref: " raw evidence ".to_owned(),
            })
        }
    }

    #[test]
    fn emitter_failure_is_sanitized_and_not_cached_as_success() {
        let mut usecase = IntelligenceAuditTapUsecase::new(FailingEmitter);

        let receipt = usecase.emit(input("idem-fail"));
        let debug = format!("{receipt:?}");

        assert_eq!(receipt.status, IntelligenceAuditTapStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(IntelligenceAuditTapDenialKind::EmitterFailed)
        );
        assert_eq!(receipt.denial_reasons, vec!["audit tap emitter failed"]);
        assert!(
            receipt
                .evidence_refs
                .contains(&"audit-tap:unsafe-evidence-ref".to_owned())
        );
        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("write an email"));
        assert!(!debug.contains("raw evidence"));
    }

    struct DriftEmitter;

    impl AuditTapEmitterPort for DriftEmitter {
        fn emit(
            &mut self,
            request: AuditTapRecordRequest,
        ) -> Result<AuditTapSealReceipt, AuditTapEmissionFailure> {
            Ok(AuditTapSealReceipt {
                audit_id: request.audit_id,
                event_id: request.event_id,
                tenant_id: "ten_other".to_owned(),
                sequence: 1,
                chain_ref: "audit-chain://intelligence/ten_other/2026-05-23".to_owned(),
                merkle_root_ref: "merkle://intelligence/root/1".to_owned(),
                signature_ref: "sigref://ed25519/intelligence/1".to_owned(),
                outbox_ref: "outbox://audit-tap/1".to_owned(),
                evidence_refs: vec!["seal:1".to_owned()],
            })
        }
    }

    #[test]
    fn seal_binding_drift_denies_without_cached_success() {
        let mut usecase = IntelligenceAuditTapUsecase::new(DriftEmitter);

        let receipt = usecase.emit(input("idem-drift"));

        assert_eq!(receipt.status, IntelligenceAuditTapStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(IntelligenceAuditTapDenialKind::SealBindingDrift)
        );
        assert!(receipt.seal.is_none());
    }

    #[test]
    fn audit_envelope_v2_fields_are_required() {
        let mut usecase = IntelligenceAuditTapUsecase::new(FakeAuditTapEmitter::default());
        let mut bad = input("idem-envelope");
        bad.hlc_timestamp = "2026-05-23T11:00:00Z".to_owned();
        bad.occurred_at_epoch_seconds = 0;
        bad.region.clear();
        bad.cost_usd_minor_units = -1;

        let receipt = usecase.emit(bad);
        let emitter = usecase.into_inner();

        assert_eq!(receipt.status, IntelligenceAuditTapStatus::Denied);
        assert!(
            receipt
                .denial_reasons
                .contains(&"hlc timestamp must include logical counter".to_owned())
        );
        assert!(
            receipt
                .denial_reasons
                .contains(&"occurred_at_epoch_seconds must be nonzero".to_owned())
        );
        assert!(
            receipt
                .denial_reasons
                .contains(&"cost_usd_minor_units must be non-negative".to_owned())
        );
        assert!(
            receipt
                .denial_reasons
                .contains(&"region is required".to_owned())
        );
        assert_eq!(emitter.calls, 0);
    }

    #[test]
    fn credential_event_can_emit_without_route_or_guardrail_evidence() {
        let mut usecase = IntelligenceAuditTapUsecase::new(FakeAuditTapEmitter::default());
        let mut credential_event = input("idem-credential");
        credential_event.event_class = IntelligenceAuditEventClass::CredentialHandleResolved;
        credential_event.action = "credential_resolve".to_owned();
        credential_event.route_evidence_refs.clear();
        credential_event.guardrail_evidence_refs.clear();
        credential_event.provider_evidence_ref = None;
        credential_event.content_ref = None;
        credential_event.output_ref = None;

        let receipt = usecase.emit(credential_event);
        let emitter = usecase.into_inner();

        assert_eq!(receipt.status, IntelligenceAuditTapStatus::Sealed);
        assert_eq!(emitter.calls, 1);
        assert_eq!(
            emitter.requests[0].evidence_refs,
            vec![
                "cedar:decision:1".to_owned(),
                "credential:handle:1".to_owned(),
                "req:audit-tap".to_owned(),
            ]
        );
        assert!(emitter.requests[0].resource_refs.is_empty());
    }
}
