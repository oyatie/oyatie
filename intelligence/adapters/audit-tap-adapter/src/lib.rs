//! Intelligence audit-tap adapter foundation.
//!
//! This crate provides a deterministic metadata-only adapter seam between the
//! Intelligence audit-tap usecase and a future audit-chain sidecar/worker. It
//! validates sidecar configuration, validates audit-event envelope metadata,
//! builds audit-chain request envelopes, maps sidecar outcome metadata into
//! seal receipts or sanitized failures, and performs no Ed25519 signing,
//! Merkle-tree construction, durable audit-chain writes, filesystem, network,
//! queue, or cloud-runtime work.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use intelligence_audit_tap_usecase::{
    AuditTapEmissionFailure, AuditTapEmitterPort, AuditTapRecordRequest, AuditTapSealReceipt,
    CarbonIntensitySource, InfrastructureProvider, IntelligenceAuditEventClass,
    IntelligenceAuditTapDenialKind, IntelligenceAuditTapInput, IntelligenceAuditTapReceipt,
    IntelligenceAuditTapStatus, IntelligenceAuditTapUsecase,
};

const ADAPTER_REFERENCE_REF: &str = "spec://oyatie/intelligence/audit-tap-adapter-foundation";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditChainTransportMode {
    EnvelopeOnly,
    AuditChainSidecar,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditChainAdapterConfig {
    pub chain_writer_ref: String,                // data_class: INTERNAL_ONLY
    pub merkle_sealer_ref: String,               // data_class: INTERNAL_ONLY
    pub signature_key_ref: String,               // data_class: SECRET_REF
    pub outbox_ref: String,                      // data_class: INTERNAL_ONLY
    pub audit_chain_audience_ref: String,        // data_class: INTERNAL_ONLY
    pub transport_mode: AuditChainTransportMode, // data_class: INTERNAL_ONLY
}

impl AuditChainAdapterConfig {
    pub fn new(
        chain_writer_ref: impl Into<String>,
        merkle_sealer_ref: impl Into<String>,
        signature_key_ref: impl Into<String>,
        outbox_ref: impl Into<String>,
        audit_chain_audience_ref: impl Into<String>,
    ) -> Self {
        Self {
            chain_writer_ref: chain_writer_ref.into(),
            merkle_sealer_ref: merkle_sealer_ref.into(),
            signature_key_ref: signature_key_ref.into(),
            outbox_ref: outbox_ref.into(),
            audit_chain_audience_ref: audit_chain_audience_ref.into(),
            transport_mode: AuditChainTransportMode::EnvelopeOnly,
        }
    }

    pub fn with_transport_mode(mut self, transport_mode: AuditChainTransportMode) -> Self {
        self.transport_mode = transport_mode;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditChainAdapterConfigError {
    EmptyChainWriterRef,
    InvalidChainWriterRef,
    EmptyMerkleSealerRef,
    InvalidMerkleSealerRef,
    EmptySignatureKeyRef,
    InvalidSignatureKeyRef,
    EmptyOutboxRef,
    InvalidOutboxRef,
    EmptyAuditChainAudienceRef,
    InvalidAuditChainAudienceRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditChainStatus {
    Sealed {
        sequence: u64,
        chain_ref: String,
        merkle_root_ref: String,
        signature_ref: String,
        outbox_ref: String,
        evidence_ref: String,
    },
    Denied {
        evidence_ref: String,
    },
    RateLimited {
        evidence_ref: String,
    },
    ChainError {
        evidence_ref: String,
    },
    AuthError {
        evidence_ref: String,
    },
    InvalidRequest {
        evidence_ref: String,
    },
    Timeout {
        evidence_ref: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditChainRequestEnvelope {
    pub transport_mode: AuditChainTransportMode, // data_class: INTERNAL_ONLY
    pub chain_writer_ref: String,                // data_class: INTERNAL_ONLY
    pub merkle_sealer_ref: String,               // data_class: INTERNAL_ONLY
    pub signature_key_ref: String,               // data_class: SECRET_REF
    pub outbox_ref: String,                      // data_class: INTERNAL_ONLY
    pub audit_chain_audience_ref: String,        // data_class: INTERNAL_ONLY
    pub audit_id: String,                        // data_class: INTERNAL_ONLY
    pub event_id: String,                        // data_class: INTERNAL_ONLY
    pub tenant_id: String,                       // data_class: INTERNAL_ONLY
    pub producer_id: String,                     // data_class: INTERNAL_ONLY
    pub source_microservice: String,             // data_class: INTERNAL_ONLY
    pub event_class: IntelligenceAuditEventClass, // data_class: INTERNAL_ONLY
    pub action: String,                          // data_class: INTERNAL_ONLY
    pub decision: String,                        // data_class: INTERNAL_ONLY
    pub trace_id: String,                        // data_class: INTERNAL_ONLY
    pub span_id: String,                         // data_class: INTERNAL_ONLY
    pub cell_id: String,                         // data_class: INTERNAL_ONLY
    pub jurisdiction_code: String,               // data_class: INTERNAL_ONLY
    pub sub_scope_path: String,                  // data_class: INTERNAL_ONLY
    pub hlc_timestamp: String,                   // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64,          // data_class: INTERNAL_ONLY
    pub cost_usd_minor_units: i64,               // data_class: INTERNAL_ONLY
    pub co2_grams: u64,                          // data_class: INTERNAL_ONLY
    pub watt_hours: u64,                         // data_class: INTERNAL_ONLY
    pub infrastructure_provider: InfrastructureProvider, // data_class: INTERNAL_ONLY
    pub region: String,                          // data_class: INTERNAL_ONLY
    pub carbon_intensity_source: CarbonIntensitySource, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,              // data_class: INTERNAL_ONLY
    pub resource_refs: Vec<String>,              // data_class: INTERNAL_ONLY
    pub canonical_fingerprint: String,           // data_class: INTERNAL_ONLY
    pub adapter_reference_refs: Vec<String>,     // data_class: PUBLIC
}

#[derive(Debug)]
pub struct AuditTapAdapter {
    config: AuditChainAdapterConfig,
    next_status: AuditChainStatus,
    last_envelope: Option<AuditChainRequestEnvelope>,
}

impl AuditTapAdapter {
    pub fn try_new(
        config: AuditChainAdapterConfig,
        next_status: AuditChainStatus,
    ) -> Result<Self, AuditChainAdapterConfigError> {
        validate_config(&config)?;
        Ok(Self {
            config,
            next_status,
            last_envelope: None,
        })
    }

    pub fn last_envelope(&self) -> Option<&AuditChainRequestEnvelope> {
        self.last_envelope.as_ref()
    }

    pub fn set_next_status(&mut self, next_status: AuditChainStatus) {
        self.next_status = next_status;
    }

    pub fn emit(
        &mut self,
        request: AuditTapRecordRequest,
    ) -> Result<AuditTapSealReceipt, AuditTapEmissionFailure> {
        validate_request(&request)?;
        validate_status_metadata(&self.next_status)?;
        let envelope = self.build_envelope(&request);
        self.last_envelope = Some(envelope);
        seal_or_failure_from_status(&request, &self.next_status)
    }

    fn build_envelope(&self, request: &AuditTapRecordRequest) -> AuditChainRequestEnvelope {
        AuditChainRequestEnvelope {
            transport_mode: self.config.transport_mode,
            chain_writer_ref: self.config.chain_writer_ref.clone(),
            merkle_sealer_ref: self.config.merkle_sealer_ref.clone(),
            signature_key_ref: self.config.signature_key_ref.clone(),
            outbox_ref: self.config.outbox_ref.clone(),
            audit_chain_audience_ref: self.config.audit_chain_audience_ref.clone(),
            audit_id: request.audit_id.clone(),
            event_id: request.event_id.clone(),
            tenant_id: request.tenant_id.clone(),
            producer_id: request.producer_id.clone(),
            source_microservice: request.source_microservice.clone(),
            event_class: request.event_class,
            action: request.action.clone(),
            decision: request.decision.clone(),
            trace_id: request.trace_id.clone(),
            span_id: request.span_id.clone(),
            cell_id: request.cell_id.clone(),
            jurisdiction_code: request.jurisdiction_code.clone(),
            sub_scope_path: request.sub_scope_path.clone(),
            hlc_timestamp: request.hlc_timestamp.clone(),
            occurred_at_epoch_seconds: request.occurred_at_epoch_seconds,
            cost_usd_minor_units: request.cost_usd_minor_units,
            co2_grams: request.co2_grams,
            watt_hours: request.watt_hours,
            infrastructure_provider: request.infrastructure_provider,
            region: request.region.clone(),
            carbon_intensity_source: request.carbon_intensity_source,
            evidence_refs: request.evidence_refs.clone(),
            resource_refs: request.resource_refs.clone(),
            canonical_fingerprint: request.canonical_fingerprint.clone(),
            adapter_reference_refs: vec![ADAPTER_REFERENCE_REF.to_owned()],
        }
    }
}

impl AuditTapEmitterPort for AuditTapAdapter {
    fn emit(
        &mut self,
        request: AuditTapRecordRequest,
    ) -> Result<AuditTapSealReceipt, AuditTapEmissionFailure> {
        Self::emit(self, request)
    }
}

fn validate_config(config: &AuditChainAdapterConfig) -> Result<(), AuditChainAdapterConfigError> {
    require_config_ref(
        &config.chain_writer_ref,
        AuditChainAdapterConfigError::EmptyChainWriterRef,
        AuditChainAdapterConfigError::InvalidChainWriterRef,
    )?;
    require_config_ref(
        &config.merkle_sealer_ref,
        AuditChainAdapterConfigError::EmptyMerkleSealerRef,
        AuditChainAdapterConfigError::InvalidMerkleSealerRef,
    )?;
    require_config_ref(
        &config.signature_key_ref,
        AuditChainAdapterConfigError::EmptySignatureKeyRef,
        AuditChainAdapterConfigError::InvalidSignatureKeyRef,
    )?;
    require_config_ref(
        &config.outbox_ref,
        AuditChainAdapterConfigError::EmptyOutboxRef,
        AuditChainAdapterConfigError::InvalidOutboxRef,
    )?;
    require_config_ref(
        &config.audit_chain_audience_ref,
        AuditChainAdapterConfigError::EmptyAuditChainAudienceRef,
        AuditChainAdapterConfigError::InvalidAuditChainAudienceRef,
    )?;
    Ok(())
}

fn require_config_ref(
    value: &str,
    empty: AuditChainAdapterConfigError,
    invalid: AuditChainAdapterConfigError,
) -> Result<(), AuditChainAdapterConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(empty);
    }
    if is_safe_config_ref(trimmed) && value == trimmed {
        Ok(())
    } else {
        Err(invalid)
    }
}

fn validate_request(request: &AuditTapRecordRequest) -> Result<(), AuditTapEmissionFailure> {
    let mut reasons = Vec::new();
    require_metadata_ref("audit id", &request.audit_id, &mut reasons);
    require_metadata_ref("event id", &request.event_id, &mut reasons);
    require_tenant_id(&request.tenant_id, &mut reasons);
    require_metadata_ref("producer id", &request.producer_id, &mut reasons);
    if request.source_microservice != "intelligence" {
        reasons.push("source microservice must be intelligence".to_owned());
    }
    require_metadata_ref("action", &request.action, &mut reasons);
    require_metadata_ref("decision", &request.decision, &mut reasons);
    require_metadata_ref("trace id", &request.trace_id, &mut reasons);
    require_metadata_ref("span id", &request.span_id, &mut reasons);
    require_metadata_ref("cell id", &request.cell_id, &mut reasons);
    require_metadata_ref(
        "jurisdiction code",
        &request.jurisdiction_code,
        &mut reasons,
    );
    require_metadata_ref("sub-scope path", &request.sub_scope_path, &mut reasons);
    require_hlc_timestamp(&request.hlc_timestamp, &mut reasons);
    if request.occurred_at_epoch_seconds == 0 {
        reasons.push("occurred_at_epoch_seconds must be nonzero".to_owned());
    }
    if request.cost_usd_minor_units < 0 {
        reasons.push("cost_usd_minor_units must be non-negative".to_owned());
    }
    require_metadata_ref("region", &request.region, &mut reasons);
    require_evidence_refs("evidence ref", &request.evidence_refs, &mut reasons);
    validate_resource_refs("resource ref", &request.resource_refs, &mut reasons);
    require_metadata_ref(
        "canonical fingerprint",
        &request.canonical_fingerprint,
        &mut reasons,
    );

    if reasons.is_empty() {
        Ok(())
    } else {
        Err(dispatch_failure(
            "audit-chain request metadata invalid",
            "validation:audit-chain-request",
        ))
    }
}

fn validate_status_metadata(status: &AuditChainStatus) -> Result<(), AuditTapEmissionFailure> {
    let mut reasons = Vec::new();
    match status {
        AuditChainStatus::Sealed {
            sequence,
            chain_ref,
            merkle_root_ref,
            signature_ref,
            outbox_ref,
            evidence_ref,
        } => {
            if *sequence == 0 {
                reasons.push("seal sequence must be nonzero".to_owned());
            }
            require_prefix_ref(
                "chain ref",
                chain_ref,
                &["audit-chain://", "chain://"],
                &mut reasons,
            );
            require_prefix_ref(
                "merkle root ref",
                merkle_root_ref,
                &["merkle://", "merkle-sha256:"],
                &mut reasons,
            );
            require_prefix_ref(
                "signature ref",
                signature_ref,
                &["sigref://", "signature://"],
                &mut reasons,
            );
            require_prefix_ref(
                "outbox ref",
                outbox_ref,
                &["outbox://", "audit-outbox://"],
                &mut reasons,
            );
            require_evidence_ref("status evidence ref", evidence_ref, &mut reasons);
        }
        AuditChainStatus::Denied { evidence_ref }
        | AuditChainStatus::RateLimited { evidence_ref }
        | AuditChainStatus::ChainError { evidence_ref }
        | AuditChainStatus::AuthError { evidence_ref }
        | AuditChainStatus::InvalidRequest { evidence_ref }
        | AuditChainStatus::Timeout { evidence_ref } => {
            require_evidence_ref("status evidence ref", evidence_ref, &mut reasons);
        }
    }

    if reasons.is_empty() {
        Ok(())
    } else {
        Err(dispatch_failure(
            "audit-chain sidecar status metadata invalid",
            "validation:audit-chain-status",
        ))
    }
}

fn seal_or_failure_from_status(
    request: &AuditTapRecordRequest,
    status: &AuditChainStatus,
) -> Result<AuditTapSealReceipt, AuditTapEmissionFailure> {
    match status {
        AuditChainStatus::Sealed {
            sequence,
            chain_ref,
            merkle_root_ref,
            signature_ref,
            outbox_ref,
            evidence_ref,
        } => Ok(AuditTapSealReceipt {
            audit_id: request.audit_id.clone(),
            event_id: request.event_id.clone(),
            tenant_id: request.tenant_id.clone(),
            sequence: *sequence,
            chain_ref: chain_ref.clone(),
            merkle_root_ref: merkle_root_ref.clone(),
            signature_ref: signature_ref.clone(),
            outbox_ref: outbox_ref.clone(),
            evidence_refs: vec![evidence_ref.clone()],
        }),
        AuditChainStatus::Denied { evidence_ref } => Err(dispatch_failure(
            "audit-chain sidecar denied request",
            evidence_ref,
        )),
        AuditChainStatus::RateLimited { evidence_ref } => Err(dispatch_failure(
            "audit-chain sidecar rate limited request",
            evidence_ref,
        )),
        AuditChainStatus::ChainError { evidence_ref } => Err(dispatch_failure(
            "audit-chain sidecar failed request",
            evidence_ref,
        )),
        AuditChainStatus::AuthError { evidence_ref } => Err(dispatch_failure(
            "audit-chain sidecar authentication failed",
            evidence_ref,
        )),
        AuditChainStatus::InvalidRequest { evidence_ref } => Err(dispatch_failure(
            "audit-chain sidecar rejected invalid request",
            evidence_ref,
        )),
        AuditChainStatus::Timeout { evidence_ref } => Err(dispatch_failure(
            "audit-chain sidecar timed out request",
            evidence_ref,
        )),
    }
}

fn dispatch_failure(reason: &str, evidence_ref: &str) -> AuditTapEmissionFailure {
    AuditTapEmissionFailure {
        reason: reason.to_owned(),
        evidence_ref: safe_evidence_ref(evidence_ref),
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
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
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
    for value in refs {
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

fn validate_resource_refs(label: &str, refs: &[String], reasons: &mut Vec<String>) {
    for value in refs {
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
        "audit-chain:missing-evidence-ref".to_owned()
    } else if value != trimmed
        || contains_whitespace(trimmed)
        || contains_raw_secret_material(trimmed)
        || contains_raw_content_material(trimmed)
        || !trimmed.contains(':')
    {
        "audit-chain:unsafe-evidence-ref".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn is_safe_config_ref(value: &str) -> bool {
    !value.trim().is_empty()
        && value == value.trim()
        && (value.contains("://") || value.contains(':'))
        && !contains_whitespace(value)
        && !contains_raw_secret_material(value)
        && !contains_raw_content_material(value)
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
        || lower.contains("private key")
        || lower.contains("-----begin")
        || lower.contains("ed25519_private_key")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw model")
        || lower.contains("write an email")
        || lower.contains("customer message")
        || lower.contains("model answer")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> AuditChainAdapterConfig {
        AuditChainAdapterConfig::new(
            "audit-chain://writer/intelligence",
            "merkle://sealer/intelligence",
            "sigkey://ed25519/intelligence/audit-tap",
            "outbox://audit-tap/intelligence",
            "audience://audit-chain/intelligence",
        )
    }

    fn sealed_status() -> AuditChainStatus {
        AuditChainStatus::Sealed {
            sequence: 42,
            chain_ref: "audit-chain://intelligence/ten_a/2026-05-24".to_owned(),
            merkle_root_ref: "merkle://intelligence/root/42".to_owned(),
            signature_ref: "sigref://ed25519/intelligence/42".to_owned(),
            outbox_ref: "outbox://audit-tap/42".to_owned(),
            evidence_ref: "seal:evidence:42".to_owned(),
        }
    }

    fn record_request() -> AuditTapRecordRequest {
        AuditTapRecordRequest {
            audit_id: "audit_01".to_owned(),
            event_id: "evt_01".to_owned(),
            tenant_id: "ten_a".to_owned(),
            producer_id: "intelligence-dispatch".to_owned(),
            source_microservice: "intelligence".to_owned(),
            event_class: IntelligenceAuditEventClass::DispatchCompleted,
            action: "provider_dispatch".to_owned(),
            decision: "allow".to_owned(),
            trace_id: "trace-01".to_owned(),
            span_id: "span-01".to_owned(),
            cell_id: "cell-us-east-1a".to_owned(),
            jurisdiction_code: "US".to_owned(),
            sub_scope_path: "intelligence/dispatch/openai".to_owned(),
            hlc_timestamp: "2026-05-24T21:40:00Z/lc=1".to_owned(),
            occurred_at_epoch_seconds: 1_779_646_800,
            cost_usd_minor_units: 1,
            co2_grams: 2,
            watt_hours: 3,
            infrastructure_provider: InfrastructureProvider::OyatieOwn,
            region: "oyatie_own:iad-1".to_owned(),
            carbon_intensity_source: CarbonIntensitySource::ProviderGridAverage,
            evidence_refs: vec![
                "cedar:decision:1".to_owned(),
                "req:audit-tap".to_owned(),
                "route:openai".to_owned(),
            ],
            resource_refs: vec![
                "contentref://prompt/1".to_owned(),
                "contentref://output/1".to_owned(),
            ],
            canonical_fingerprint: "schema:oyatie/log/v2|audit:01".to_owned(),
        }
    }

    fn audit_input(idempotency_key: &str) -> IntelligenceAuditTapInput {
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
            hlc_timestamp: "2026-05-24T21:40:00Z/lc=1".to_owned(),
            occurred_at_epoch_seconds: 1_779_646_800,
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
    fn builds_metadata_only_audit_chain_envelope_and_seal_receipt() {
        let mut adapter =
            AuditTapAdapter::try_new(config(), sealed_status()).expect("config valid");

        let seal = adapter.emit(record_request()).expect("sealed");
        let envelope = adapter.last_envelope().expect("envelope recorded");
        let debug = format!("{envelope:?}{seal:?}");

        assert_eq!(seal.sequence, 42);
        assert_eq!(seal.audit_id, "audit_01");
        assert_eq!(seal.event_id, "evt_01");
        assert_eq!(seal.tenant_id, "ten_a");
        assert_eq!(
            envelope.transport_mode,
            AuditChainTransportMode::EnvelopeOnly
        );
        assert_eq!(envelope.source_microservice, "intelligence");
        assert_eq!(
            envelope.event_class,
            IntelligenceAuditEventClass::DispatchCompleted
        );
        assert_eq!(
            envelope.evidence_refs,
            vec!["cedar:decision:1", "req:audit-tap", "route:openai"]
        );
        assert!(
            envelope
                .adapter_reference_refs
                .contains(&"spec://oyatie/intelligence/audit-tap-adapter-foundation".to_owned())
        );
        assert!(!debug.contains("sk-"));
        assert!(!debug.contains("write an email"));
        assert!(!debug.contains("raw model answer"));
    }

    #[test]
    fn usecase_emits_through_adapter_without_exposing_secret_or_content_bytes() {
        let adapter = AuditTapAdapter::try_new(config(), sealed_status()).expect("config valid");
        let mut usecase = IntelligenceAuditTapUsecase::new(adapter);

        let receipt = usecase.emit(audit_input("idem-adapter"));
        let adapter = usecase.into_inner();
        let envelope = adapter.last_envelope().expect("adapter called");
        let debug = format!("{receipt:?}{envelope:?}");

        assert_eq!(receipt.status, IntelligenceAuditTapStatus::Sealed);
        assert_eq!(envelope.audit_id, "audit_01");
        assert!(
            envelope
                .resource_refs
                .contains(&"contentref://prompt/1".to_owned())
        );
        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("write an email"));
        assert!(!debug.contains("raw prompt"));
    }

    #[test]
    fn rejects_invalid_config_or_raw_key_refs() {
        assert_eq!(
            AuditTapAdapter::try_new(
                AuditChainAdapterConfig::new(
                    "",
                    "merkle://sealer",
                    "sigkey://safe",
                    "outbox://tap",
                    "audience://audit"
                ),
                sealed_status(),
            )
            .unwrap_err(),
            AuditChainAdapterConfigError::EmptyChainWriterRef
        );
        assert_eq!(
            AuditTapAdapter::try_new(
                AuditChainAdapterConfig::new(
                    "audit-chain://writer",
                    "merkle://sealer",
                    "-----BEGIN PRIVATE KEY----- raw",
                    "outbox://tap",
                    "audience://audit",
                ),
                sealed_status(),
            )
            .unwrap_err(),
            AuditChainAdapterConfigError::InvalidSignatureKeyRef
        );
    }

    #[test]
    fn rejects_invalid_request_metadata_before_envelope_side_effects() {
        let mut adapter =
            AuditTapAdapter::try_new(config(), sealed_status()).expect("config valid");
        let mut bad = record_request();
        bad.tenant_id = "tenant/raw".to_owned();
        bad.evidence_refs = vec!["Authorization: Bearer sk-test".to_owned()];
        bad.resource_refs = vec!["write an email to a customer".to_owned()];

        let failure = adapter.emit(bad).expect_err("invalid request denied");
        let debug = format!("{failure:?}{adapter:?}");

        assert_eq!(failure.reason, "audit-chain request metadata invalid");
        assert_eq!(failure.evidence_ref, "validation:audit-chain-request");
        assert!(adapter.last_envelope().is_none());
        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("write an email"));
    }

    #[test]
    fn maps_audit_chain_outcomes_to_sanitized_failures() {
        for status in [
            AuditChainStatus::Denied {
                evidence_ref: "audit-chain:denied".to_owned(),
            },
            AuditChainStatus::RateLimited {
                evidence_ref: "audit-chain:rate-limited".to_owned(),
            },
            AuditChainStatus::ChainError {
                evidence_ref: "audit-chain:error".to_owned(),
            },
            AuditChainStatus::AuthError {
                evidence_ref: "audit-chain:auth".to_owned(),
            },
            AuditChainStatus::InvalidRequest {
                evidence_ref: "audit-chain:invalid".to_owned(),
            },
            AuditChainStatus::Timeout {
                evidence_ref: "audit-chain:timeout".to_owned(),
            },
        ] {
            let mut adapter = AuditTapAdapter::try_new(config(), status).expect("config valid");
            let failure = adapter
                .emit(record_request())
                .expect_err("status maps to failure");
            assert!(failure.reason.starts_with("audit-chain sidecar"));
            assert!(failure.evidence_ref.starts_with("audit-chain:"));
            assert!(adapter.last_envelope().is_some());
        }
    }

    #[test]
    fn invalid_seal_status_metadata_fails_before_envelope_side_effects() {
        let mut adapter = AuditTapAdapter::try_new(
            config(),
            AuditChainStatus::Sealed {
                sequence: 0,
                chain_ref: "audit-chain://intelligence/ten_a/2026-05-24".to_owned(),
                merkle_root_ref: "merkle://intelligence/root/42".to_owned(),
                signature_ref: "sigref://ed25519/intelligence/42".to_owned(),
                outbox_ref: "outbox://audit-tap/42".to_owned(),
                evidence_ref: "seal:evidence:42".to_owned(),
            },
        )
        .expect("config valid");

        let failure = adapter
            .emit(record_request())
            .expect_err("invalid status denied");

        assert_eq!(
            failure.reason,
            "audit-chain sidecar status metadata invalid"
        );
        assert_eq!(failure.evidence_ref, "validation:audit-chain-status");
        assert!(adapter.last_envelope().is_none());
    }

    #[test]
    fn failures_never_contain_raw_secret_or_document_bytes() {
        let mut adapter =
            AuditTapAdapter::try_new(config(), sealed_status()).expect("config valid");
        let mut bad = record_request();
        bad.canonical_fingerprint = "raw prompt: write an email with bearer sk-test".to_owned();

        let failure = adapter.emit(bad).expect_err("raw request denied");
        let debug = format!("{failure:?}{adapter:?}");

        assert_eq!(failure.reason, "audit-chain request metadata invalid");
        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("write an email"));
        assert!(!debug.contains("raw prompt"));
    }
}
