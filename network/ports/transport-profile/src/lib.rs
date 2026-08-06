#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const MAX_EXTERNAL_FALLBACK_TIMEOUT_MS: u32 = 12_500;
pub const DEFAULT_EXTERNAL_FALLBACK_TIMEOUT_MS: u32 = 500;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportCapabilityClass {
    External,
    InterCell,
    Internal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportProtocol {
    Http3,
    Http2,
    Http11,
    GrpcHttp2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TlsProfile {
    StrictTls13,
    SpiffeMtlsTls13,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum HybridKem {
    #[serde(rename = "x25519mlkem768")]
    X25519MlKem768,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum HybridSignature {
    #[serde(rename = "ed25519+ml_dsa_65")]
    Ed25519MlDsa65,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TlsSupportedGroup {
    #[serde(rename = "x25519mlkem768")]
    X25519MlKem768,
    #[serde(rename = "x25519")]
    X25519,
}

const PQC_TRANSITION_SUPPORTED_GROUPS: [TlsSupportedGroup; 2] =
    [TlsSupportedGroup::X25519MlKem768, TlsSupportedGroup::X25519];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EchPolicy {
    /// data_class: PUBLIC - endpoint privacy posture advertised by the endpoint class.
    pub enabled: bool,
    /// data_class: PUBLIC - whether the endpoint must publish and maintain ECH support.
    pub support_required: bool,
    /// data_class: PUBLIC - whether plaintext-SNI fallback remains allowed during transition.
    pub plaintext_sni_fallback_allowed: bool,
    /// data_class: INTERNAL_ONLY - operational rotation objective, not a key value.
    pub key_rotation_hours: Option<u16>,
    /// data_class: PUBLIC - whether retry configs are greased for compatibility.
    pub grease_retry_configs: bool,
    /// data_class: PUBLIC - public cover name, not tenant or service secret material.
    pub outer_sni: Option<String>,
}

impl EchPolicy {
    fn adr_0354_external() -> Self {
        Self {
            enabled: true,
            support_required: true,
            plaintext_sni_fallback_allowed: true,
            key_rotation_hours: Some(24),
            grease_retry_configs: true,
            outer_sni: Some("api.oyatie.dev".to_owned()),
        }
    }

    pub fn disabled() -> Self {
        Self {
            enabled: false,
            support_required: false,
            plaintext_sni_fallback_allowed: false,
            key_rotation_hours: None,
            grease_retry_configs: false,
            outer_sni: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PqcPolicy {
    /// data_class: PUBLIC - whether hybrid post-quantum negotiation is expected.
    pub enabled: bool,
    /// data_class: PUBLIC - whether the endpoint must offer hybrid negotiation.
    pub hybrid_negotiation_required: bool,
    /// data_class: PUBLIC - standardized hybrid KEM identifier.
    pub kem: Option<HybridKem>,
    /// data_class: PUBLIC - standardized hybrid signature identifier.
    pub signature: Option<HybridSignature>,
    /// data_class: PUBLIC - concrete TLS 1.3 supported_groups order advertised by this endpoint.
    pub supported_groups: Vec<TlsSupportedGroup>,
    /// data_class: PUBLIC - whether classical fallback is allowed during rollout.
    pub classical_transition_fallback_allowed: bool,
}

impl PqcPolicy {
    pub fn transition_supported_groups() -> Vec<TlsSupportedGroup> {
        PQC_TRANSITION_SUPPORTED_GROUPS.to_vec()
    }

    fn adr_0354_hybrid_required() -> Self {
        Self {
            enabled: true,
            hybrid_negotiation_required: true,
            kem: Some(HybridKem::X25519MlKem768),
            signature: Some(HybridSignature::Ed25519MlDsa65),
            supported_groups: Self::transition_supported_groups(),
            classical_transition_fallback_allowed: true,
        }
    }

    pub fn disabled() -> Self {
        Self {
            enabled: false,
            hybrid_negotiation_required: false,
            kem: None,
            signature: None,
            supported_groups: Vec::new(),
            classical_transition_fallback_allowed: false,
        }
    }

    pub fn tls_supported_groups(&self) -> Vec<TlsSupportedGroup> {
        self.supported_groups.clone()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransportEndpointSpec {
    /// data_class: INTERNAL_ONLY - stable platform endpoint identifier, not customer data.
    pub endpoint_id: String,
    /// data_class: PUBLIC - coarse exposure class for policy and runtime selection.
    pub capability_class: TransportCapabilityClass,
    /// data_class: PUBLIC - wire protocol family for the declared endpoint.
    pub protocol: TransportProtocol,
    /// data_class: PUBLIC - bounded fallback wire protocol, when the endpoint class permits one.
    pub fallback_protocol: Option<TransportProtocol>,
    /// data_class: PUBLIC - minimum TLS/authentication posture required by the class.
    pub tls_profile: TlsProfile,
    /// data_class: PUBLIC - HTTP Alt-Svc value for public protocol upgrade discovery.
    pub alt_svc: Option<String>,
    /// data_class: PUBLIC - discovery/fallback budget in milliseconds.
    pub fallback_timeout_ms: Option<u32>,
    /// data_class: PUBLIC - encrypted client hello posture, not key material.
    pub ech: EchPolicy,
    /// data_class: PUBLIC - hybrid post-quantum posture, not key material.
    pub pqc: PqcPolicy,
}

impl TransportEndpointSpec {
    pub fn external_http3(endpoint_id: impl Into<String>) -> Self {
        Self {
            endpoint_id: endpoint_id.into(),
            capability_class: TransportCapabilityClass::External,
            protocol: TransportProtocol::Http3,
            fallback_protocol: Some(TransportProtocol::Http2),
            tls_profile: TlsProfile::StrictTls13,
            alt_svc: Some(r#"h3=":443"; ma=86400"#.to_owned()),
            fallback_timeout_ms: Some(DEFAULT_EXTERNAL_FALLBACK_TIMEOUT_MS),
            ech: EchPolicy::disabled(),
            pqc: PqcPolicy::disabled(),
        }
    }

    pub fn inter_cell_grpc_h2(endpoint_id: impl Into<String>) -> Self {
        Self {
            endpoint_id: endpoint_id.into(),
            capability_class: TransportCapabilityClass::InterCell,
            protocol: TransportProtocol::GrpcHttp2,
            fallback_protocol: None,
            tls_profile: TlsProfile::SpiffeMtlsTls13,
            alt_svc: None,
            fallback_timeout_ms: None,
            ech: EchPolicy::disabled(),
            pqc: PqcPolicy::disabled(),
        }
    }

    pub fn internal_grpc_h2(endpoint_id: impl Into<String>) -> Self {
        Self {
            endpoint_id: endpoint_id.into(),
            capability_class: TransportCapabilityClass::Internal,
            protocol: TransportProtocol::GrpcHttp2,
            fallback_protocol: None,
            tls_profile: TlsProfile::SpiffeMtlsTls13,
            alt_svc: None,
            fallback_timeout_ms: None,
            ech: EchPolicy::disabled(),
            pqc: PqcPolicy::disabled(),
        }
    }

    pub fn validate(&self) -> Result<(), TransportProfileError> {
        if self.endpoint_id.trim().is_empty() {
            return Err(TransportProfileError::MissingField("endpoint_id"));
        }

        match self.capability_class {
            TransportCapabilityClass::External => self.validate_external(),
            TransportCapabilityClass::InterCell => self.validate_inter_cell(),
            TransportCapabilityClass::Internal => self.validate_internal(),
        }
    }

    fn validate_external(&self) -> Result<(), TransportProfileError> {
        if self.protocol != TransportProtocol::Http3 {
            return invalid("protocol", "external endpoints require http3");
        }
        if self.tls_profile != TlsProfile::StrictTls13 {
            return invalid("tls_profile", "external endpoints require strict tls 1.3");
        }
        if self.fallback_protocol != Some(TransportProtocol::Http2) {
            return invalid(
                "fallback_protocol",
                "external endpoints require http2 fallback",
            );
        }

        let alt_svc = self
            .alt_svc
            .as_deref()
            .ok_or(TransportProfileError::MissingField("alt_svc"))?;
        if !advertises_h3_443(alt_svc) {
            return invalid("alt_svc", "external endpoints must advertise h3 on :443");
        }

        let fallback_timeout_ms = self
            .fallback_timeout_ms
            .ok_or(TransportProfileError::MissingField("fallback_timeout_ms"))?;
        if fallback_timeout_ms > MAX_EXTERNAL_FALLBACK_TIMEOUT_MS {
            return invalid(
                "fallback_timeout_ms",
                "external fallback timeout exceeds policy budget",
            );
        }
        if fallback_timeout_ms == 0 {
            return invalid(
                "fallback_timeout_ms",
                "external fallback timeout must be positive",
            );
        }

        self.validate_optional_external_ech()?;
        self.validate_optional_pqc()?;
        self.reject_unaccepted_advanced_profile()
    }

    fn validate_optional_external_ech(&self) -> Result<(), TransportProfileError> {
        if !self.ech.enabled {
            if self.ech != EchPolicy::disabled() {
                return invalid(
                    "ech",
                    "disabled ech must not carry advanced-profile metadata",
                );
            }
            return Ok(());
        }
        if !self.ech.support_required {
            return invalid("ech.support_required", "enabled ech must require support");
        }
        if !self.ech.plaintext_sni_fallback_allowed {
            return invalid(
                "ech.plaintext_sni_fallback_allowed",
                "external ech must allow plaintext-sni fallback during transition",
            );
        }
        if !matches!(self.ech.key_rotation_hours, Some(1..=24)) {
            return invalid(
                "ech.key_rotation_hours",
                "external ech rotation must be between 1h and 24h",
            );
        }
        if !self.ech.grease_retry_configs {
            return invalid(
                "ech.grease_retry_configs",
                "external ech requires grease retry configs",
            );
        }
        let outer_sni = self.ech.outer_sni.as_deref().unwrap_or_default();
        if !is_shared_oyatie_cover_name(outer_sni) {
            return invalid(
                "ech.outer_sni",
                "external ech outer_sni must be a shared oyatie.dev cover name",
            );
        }
        Ok(())
    }

    fn validate_optional_pqc(&self) -> Result<(), TransportProfileError> {
        if !self.pqc.enabled {
            if self.pqc != PqcPolicy::disabled() {
                return invalid(
                    "pqc",
                    "disabled pqc must not carry advanced-profile metadata",
                );
            }
            return Ok(());
        }
        if !self.pqc.hybrid_negotiation_required {
            return invalid(
                "pqc.hybrid_negotiation_required",
                "enabled pqc must require hybrid negotiation",
            );
        }
        if self.pqc.kem.is_none() || self.pqc.signature.is_none() {
            return invalid("pqc", "hybrid pqc requires kem and signature identifiers");
        }
        if !self.pqc.classical_transition_fallback_allowed {
            return invalid(
                "pqc.classical_transition_fallback_allowed",
                "enabled pqc requires classical fallback during transition",
            );
        }
        validate_transition_supported_groups(&self.pqc.supported_groups)?;
        Ok(())
    }

    fn validate_inter_cell(&self) -> Result<(), TransportProfileError> {
        self.validate_non_external("inter-cell endpoints require grpc over http2")?;
        self.validate_optional_pqc()?;
        self.reject_unaccepted_advanced_profile()
    }

    fn validate_internal(&self) -> Result<(), TransportProfileError> {
        self.validate_non_external("internal endpoints require grpc over http2")?;
        self.validate_optional_pqc()?;
        self.reject_unaccepted_advanced_profile()
    }

    fn reject_unaccepted_advanced_profile(&self) -> Result<(), TransportProfileError> {
        if self.ech.enabled || self.pqc.enabled {
            return invalid(
                "advanced_profile",
                "ECH/PQC activation requires a separate Accepted authority profile",
            );
        }
        Ok(())
    }

    fn validate_non_external(
        &self,
        protocol_reason: &'static str,
    ) -> Result<(), TransportProfileError> {
        if self.protocol != TransportProtocol::GrpcHttp2 {
            return invalid("protocol", protocol_reason);
        }
        if self.tls_profile != TlsProfile::SpiffeMtlsTls13 {
            return invalid(
                "tls_profile",
                "non-external endpoints require spiffe mtls tls 1.3",
            );
        }
        if self.alt_svc.is_some() {
            return invalid(
                "alt_svc",
                "non-external endpoints do not advertise http3 upgrade",
            );
        }
        if self.fallback_protocol.is_some() {
            return invalid(
                "fallback_protocol",
                "non-external endpoints do not use protocol fallback",
            );
        }
        if self.fallback_timeout_ms.is_some() {
            return invalid(
                "fallback_timeout_ms",
                "non-external endpoints do not use client protocol fallback",
            );
        }
        if self.ech != EchPolicy::disabled() {
            return invalid("ech", "non-external endpoints do not use ech");
        }
        Ok(())
    }
}

pub trait TransportProfilePort {
    fn endpoint_profile(
        &self,
        endpoint_id: &str,
    ) -> Result<Option<TransportEndpointSpec>, TransportProfileError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransportProfileError {
    MissingField(&'static str),
    InvalidField {
        field: &'static str,
        reason: &'static str,
    },
}

fn invalid<T>(field: &'static str, reason: &'static str) -> Result<T, TransportProfileError> {
    Err(TransportProfileError::InvalidField { field, reason })
}

fn advertises_h3_443(alt_svc: &str) -> bool {
    alt_svc.split(',').any(|service| {
        service
            .trim()
            .split(';')
            .next()
            .map(str::trim)
            .is_some_and(|authority| authority == r#"h3=":443""#)
    })
}

fn validate_transition_supported_groups(
    supported_groups: &[TlsSupportedGroup],
) -> Result<(), TransportProfileError> {
    if supported_groups.is_empty() {
        return invalid(
            "pqc.supported_groups",
            "pqc supported_groups must declare x25519mlkem768 and x25519 fallback",
        );
    }
    if supported_groups.first() != Some(&TlsSupportedGroup::X25519MlKem768) {
        return invalid(
            "pqc.supported_groups",
            "pqc supported_groups must start with x25519mlkem768",
        );
    }
    if !supported_groups.contains(&TlsSupportedGroup::X25519) {
        return invalid(
            "pqc.supported_groups",
            "pqc transition supported_groups must include x25519 classical fallback",
        );
    }
    if supported_groups != PQC_TRANSITION_SUPPORTED_GROUPS {
        return invalid(
            "pqc.supported_groups",
            "pqc supported_groups must exactly declare x25519mlkem768 then x25519",
        );
    }
    Ok(())
}

const SHARED_ECH_OUTER_SNI_ALLOWLIST: &[&str] = &["api.oyatie.dev"];

fn is_shared_oyatie_cover_name(outer_sni: &str) -> bool {
    if outer_sni.is_empty()
        || outer_sni.trim() != outer_sni
        || outer_sni.len() > 253
        || !outer_sni.is_ascii()
        || outer_sni
            .bytes()
            .any(|byte| byte.is_ascii_control() || matches!(byte, b'*' | b'/' | b':' | b'@'))
    {
        return false;
    }

    let lower = outer_sni.to_ascii_lowercase();
    if !lower.split('.').all(is_valid_dns_label) {
        return false;
    }

    SHARED_ECH_OUTER_SNI_ALLOWLIST
        .iter()
        .any(|allowed| *allowed == lower)
}

fn is_valid_dns_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 63
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    const CONTRACT_SPEC: &str = include_str!("../endpoint-transport-profile.contract.json");

    #[derive(Deserialize)]
    struct ContractSpec {
        required_fields: Vec<ContractField>,
        capability_classes: Vec<CapabilityClass>,
        canonical_external_endpoint: TransportEndpointSpec,
        advanced_profiles: Vec<AdvancedProfile>,
        deferred_adapters: Vec<DeferredAdapter>,
    }

    #[derive(Deserialize)]
    struct ContractField {
        name: String,
    }

    #[derive(Deserialize)]
    struct CapabilityClass {
        name: String,
        default_protocol: String,
        tls_profile: String,
        alt_svc: String,
        fallback_timeout_ms: serde_json::Value,
        ech: String,
        pqc: String,
        #[serde(default)]
        http3_correctness_dependency: Option<String>,
    }

    #[derive(Deserialize)]
    struct AdvancedProfile {
        profile_id: String,
        authority_status: String,
        activation: String,
        runtime_activation_while_proposed: String,
        base_constructor_activation: String,
        canonical_external_endpoint: TransportEndpointSpec,
    }

    #[derive(Deserialize)]
    struct DeferredAdapter {
        name: String,
        status: String,
    }

    #[test]
    fn proposed_adr_0354_overlay_shape_is_inert_at_runtime() {
        let spec = advanced_external();

        assert_eq!(spec.protocol, TransportProtocol::Http3);
        assert_eq!(spec.capability_class, TransportCapabilityClass::External);
        assert!(spec.alt_svc.as_deref().unwrap_or_default().contains("h3"));
        assert_eq!(spec.ech.enabled, true);
        assert_eq!(spec.ech.support_required, true);
        assert_eq!(spec.ech.plaintext_sni_fallback_allowed, true);
        assert_eq!(spec.pqc.enabled, true);
        assert_eq!(spec.pqc.hybrid_negotiation_required, true);
        assert_eq!(
            spec.pqc.supported_groups,
            PqcPolicy::transition_supported_groups()
        );
        assert_eq!(spec.pqc.classical_transition_fallback_allowed, true);
        assert_eq!(spec.validate(), proposed_profile_activation_error());
    }

    #[test]
    fn base_constructors_do_not_silently_enable_proposed_ech_or_pqc() {
        let external = TransportEndpointSpec::external_http3("public-inference-stream");
        let inter_cell = TransportEndpointSpec::inter_cell_grpc_h2("cell-rebalance-events");
        let internal = TransportEndpointSpec::internal_grpc_h2("internal-policy");

        for spec in [&external, &inter_cell, &internal] {
            assert_eq!(spec.ech, EchPolicy::disabled());
            assert_eq!(spec.pqc, PqcPolicy::disabled());
            assert_eq!(spec.validate(), Ok(()));
        }
    }

    #[test]
    fn rejects_external_profile_without_http3() {
        let mut spec = advanced_external();
        spec.protocol = TransportProtocol::Http2;

        assert_eq!(
            spec.validate(),
            Err(TransportProfileError::InvalidField {
                field: "protocol",
                reason: "external endpoints require http3"
            })
        );
    }

    #[test]
    fn rejects_external_profile_without_strict_tls13() {
        let mut spec = advanced_external();
        spec.tls_profile = TlsProfile::SpiffeMtlsTls13;

        assert_eq!(
            spec.validate(),
            Err(TransportProfileError::InvalidField {
                field: "tls_profile",
                reason: "external endpoints require strict tls 1.3"
            })
        );
    }

    #[test]
    fn rejects_external_profile_without_http2_fallback() {
        let mut spec = advanced_external();
        spec.fallback_protocol = None;

        assert_eq!(
            spec.validate(),
            Err(TransportProfileError::InvalidField {
                field: "fallback_protocol",
                reason: "external endpoints require http2 fallback"
            })
        );
    }

    #[test]
    fn rejects_external_profile_without_alt_svc() {
        let mut spec = advanced_external();
        spec.alt_svc = None;

        assert_eq!(
            spec.validate(),
            Err(TransportProfileError::MissingField("alt_svc"))
        );
    }

    #[test]
    fn keeps_inter_cell_transport_on_grpc_http2_without_alt_svc() {
        let spec = TransportEndpointSpec::inter_cell_grpc_h2("cell-rebalance-events");

        assert_eq!(spec.protocol, TransportProtocol::GrpcHttp2);
        assert_eq!(spec.tls_profile, TlsProfile::SpiffeMtlsTls13);
        assert_eq!(spec.alt_svc, None);
        assert_eq!(spec.ech.enabled, false);
        assert_eq!(spec.pqc, PqcPolicy::disabled());
        assert_eq!(spec.validate(), Ok(()));
    }

    #[test]
    fn rejects_internal_profile_with_protocol_fallback() {
        let mut spec = TransportEndpointSpec::internal_grpc_h2("internal-policy");
        spec.fallback_protocol = Some(TransportProtocol::Http2);

        assert_eq!(
            spec.validate(),
            Err(TransportProfileError::InvalidField {
                field: "fallback_protocol",
                reason: "non-external endpoints do not use protocol fallback"
            })
        );
    }

    #[test]
    fn rejects_internal_profile_with_quic_upgrade_advertising() {
        let mut spec = TransportEndpointSpec::internal_grpc_h2("cell-lifecycle-internal");
        spec.alt_svc = Some(r#"h3=":443""#.to_owned());

        assert_eq!(
            spec.validate(),
            Err(TransportProfileError::InvalidField {
                field: "alt_svc",
                reason: "non-external endpoints do not advertise http3 upgrade"
            })
        );
    }

    #[test]
    fn rejects_inter_cell_profile_with_http3_runtime_pull() {
        let mut spec = TransportEndpointSpec::inter_cell_grpc_h2("cell-rebalance-events");
        spec.protocol = TransportProtocol::Http3;

        assert_eq!(
            spec.validate(),
            Err(TransportProfileError::InvalidField {
                field: "protocol",
                reason: "inter-cell endpoints require grpc over http2"
            })
        );
    }

    #[test]
    fn rejects_external_profile_without_pqc_transition_fallback() {
        let mut spec = advanced_external();
        spec.pqc.classical_transition_fallback_allowed = false;

        assert_eq!(
            spec.validate(),
            Err(TransportProfileError::InvalidField {
                field: "pqc.classical_transition_fallback_allowed",
                reason: "enabled pqc requires classical fallback during transition"
            })
        );
    }

    #[test]
    fn rejects_external_profile_with_incomplete_hybrid_pqc_identifiers() {
        let mut missing_kem = advanced_external();
        missing_kem.pqc.kem = None;
        assert_eq!(
            missing_kem.validate(),
            Err(TransportProfileError::InvalidField {
                field: "pqc",
                reason: "hybrid pqc requires kem and signature identifiers"
            })
        );

        let mut missing_signature = advanced_external();
        missing_signature.pqc.signature = None;
        assert_eq!(
            missing_signature.validate(),
            Err(TransportProfileError::InvalidField {
                field: "pqc",
                reason: "hybrid pqc requires kem and signature identifiers"
            })
        );
    }

    #[test]
    fn external_pqc_supported_groups_pin_hybrid_first_and_transition_fallback() {
        let spec = advanced_external();

        assert_eq!(
            spec.pqc.supported_groups,
            PqcPolicy::transition_supported_groups()
        );
        assert_eq!(
            validate_transition_supported_groups(&spec.pqc.supported_groups),
            Ok(())
        );
    }

    #[test]
    fn rejects_missing_or_misordered_pqc_transition_supported_groups() {
        let mut missing_groups = advanced_external();
        missing_groups.pqc.supported_groups = Vec::new();
        assert_eq!(
            missing_groups.validate(),
            Err(TransportProfileError::InvalidField {
                field: "pqc.supported_groups",
                reason: "pqc supported_groups must declare x25519mlkem768 and x25519 fallback"
            })
        );

        let mut missing_classical = advanced_external();
        missing_classical.pqc.supported_groups = vec![TlsSupportedGroup::X25519MlKem768];
        assert_eq!(
            missing_classical.validate(),
            Err(TransportProfileError::InvalidField {
                field: "pqc.supported_groups",
                reason: "pqc transition supported_groups must include x25519 classical fallback"
            })
        );

        let mut misordered = advanced_inter_cell();
        misordered.pqc.supported_groups =
            vec![TlsSupportedGroup::X25519, TlsSupportedGroup::X25519MlKem768];
        assert_eq!(
            misordered.validate(),
            Err(TransportProfileError::InvalidField {
                field: "pqc.supported_groups",
                reason: "pqc supported_groups must start with x25519mlkem768"
            })
        );
    }

    #[test]
    fn rejects_endpoint_json_without_pqc_supported_groups() {
        let mut endpoint =
            serde_json::to_value(TransportEndpointSpec::external_http3("api-gateway-public"))
                .expect("serialize endpoint spec");
        endpoint
            .as_object_mut()
            .expect("endpoint object")
            .get_mut("pqc")
            .expect("pqc object")
            .as_object_mut()
            .expect("pqc policy object")
            .remove("supported_groups");

        let error = serde_json::from_value::<TransportEndpointSpec>(endpoint)
            .expect_err("endpoint JSON must declare concrete pqc supported_groups");
        assert!(
            error
                .to_string()
                .contains("missing field `supported_groups`"),
            "{error}"
        );
    }

    #[test]
    fn rejects_inter_cell_profile_without_required_hybrid_negotiation() {
        let mut spec = advanced_inter_cell();
        spec.pqc.hybrid_negotiation_required = false;

        assert_eq!(
            spec.validate(),
            Err(TransportProfileError::InvalidField {
                field: "pqc.hybrid_negotiation_required",
                reason: "enabled pqc must require hybrid negotiation"
            })
        );
    }

    #[test]
    fn rejects_external_profile_without_ech_transition_fallback() {
        let mut spec = advanced_external();
        spec.ech.plaintext_sni_fallback_allowed = false;

        assert_eq!(
            spec.validate(),
            Err(TransportProfileError::InvalidField {
                field: "ech.plaintext_sni_fallback_allowed",
                reason: "external ech must allow plaintext-sni fallback during transition"
            })
        );
    }

    #[test]
    fn rejects_external_profile_with_wrong_alt_svc_port() {
        let mut spec = advanced_external();
        spec.alt_svc = Some(r#"h3=":8443"; ma=86400"#.to_owned());

        assert_eq!(
            spec.validate(),
            Err(TransportProfileError::InvalidField {
                field: "alt_svc",
                reason: "external endpoints must advertise h3 on :443"
            })
        );
    }

    #[test]
    fn rejects_inter_cell_profile_with_ech_rotation_metadata() {
        let mut spec = TransportEndpointSpec::inter_cell_grpc_h2("cell-rebalance-events");
        spec.ech.key_rotation_hours = Some(24);

        assert_eq!(
            spec.validate(),
            Err(TransportProfileError::InvalidField {
                field: "ech",
                reason: "non-external endpoints do not use ech"
            })
        );
    }

    #[test]
    fn rejects_internal_profile_with_ech_grease_metadata() {
        let mut spec = TransportEndpointSpec::internal_grpc_h2("cell-lifecycle-internal");
        spec.ech.grease_retry_configs = true;

        assert_eq!(
            spec.validate(),
            Err(TransportProfileError::InvalidField {
                field: "ech",
                reason: "non-external endpoints do not use ech"
            })
        );
    }

    #[test]
    fn rejects_external_profile_without_ech_rotation_posture() {
        let mut spec = advanced_external();
        spec.ech.grease_retry_configs = false;

        assert_eq!(
            spec.validate(),
            Err(TransportProfileError::InvalidField {
                field: "ech.grease_retry_configs",
                reason: "external ech requires grease retry configs"
            })
        );
    }

    #[test]
    fn rejects_external_profile_with_zero_ech_rotation_window() {
        let mut spec = advanced_external();
        spec.ech.key_rotation_hours = Some(0);

        assert_eq!(
            spec.validate(),
            Err(TransportProfileError::InvalidField {
                field: "ech.key_rotation_hours",
                reason: "external ech rotation must be between 1h and 24h"
            })
        );
    }

    #[test]
    fn rejects_external_profile_with_non_cover_outer_sni() {
        let mut tenant_specific_sni = advanced_external();
        tenant_specific_sni.ech.outer_sni = Some("ten_alpha.api.oyatie.dev".to_owned());
        assert_eq!(
            tenant_specific_sni.validate(),
            Err(TransportProfileError::InvalidField {
                field: "ech.outer_sni",
                reason: "external ech outer_sni must be a shared oyatie.dev cover name"
            })
        );

        let mut foreign_sni = advanced_external();
        foreign_sni.ech.outer_sni = Some("api.example.com".to_owned());
        assert_eq!(
            foreign_sni.validate(),
            Err(TransportProfileError::InvalidField {
                field: "ech.outer_sni",
                reason: "external ech outer_sni must be a shared oyatie.dev cover name"
            })
        );

        let mut tenant_like_sni = advanced_external();
        tenant_like_sni.ech.outer_sni = Some("tenant-alpha.oyatie.dev".to_owned());
        assert_eq!(
            tenant_like_sni.validate(),
            Err(TransportProfileError::InvalidField {
                field: "ech.outer_sni",
                reason: "external ech outer_sni must be a shared oyatie.dev cover name"
            })
        );

        let overlong_label = "a".repeat(64);
        let mut overlong_sni = advanced_external();
        overlong_sni.ech.outer_sni = Some(format!("{overlong_label}.oyatie.dev"));
        assert_eq!(
            overlong_sni.validate(),
            Err(TransportProfileError::InvalidField {
                field: "ech.outer_sni",
                reason: "external ech outer_sni must be a shared oyatie.dev cover name"
            })
        );
    }

    #[test]
    fn rejects_inter_cell_profile_without_pqc_transition_fallback() {
        let mut spec = advanced_inter_cell();
        spec.pqc.classical_transition_fallback_allowed = false;

        assert_eq!(
            spec.validate(),
            Err(TransportProfileError::InvalidField {
                field: "pqc.classical_transition_fallback_allowed",
                reason: "enabled pqc requires classical fallback during transition"
            })
        );
    }

    #[test]
    fn rejects_unknown_root_fields_in_endpoint_json() {
        let mut endpoint =
            serde_json::to_value(TransportEndpointSpec::external_http3("api-gateway-public"))
                .expect("serialize endpoint spec");
        endpoint
            .as_object_mut()
            .expect("endpoint object")
            .insert("legacy_tls12_grace".to_owned(), serde_json::json!(true));

        let error = serde_json::from_value::<TransportEndpointSpec>(endpoint)
            .expect_err("unknown security-critical endpoint fields are rejected");
        assert!(
            error
                .to_string()
                .contains("unknown field `legacy_tls12_grace`"),
            "{error}"
        );
    }

    #[test]
    fn contract_artifact_declares_protocol_boundary_and_adapter_deferral() {
        let contract: ContractSpec = serde_json::from_str(CONTRACT_SPEC).expect("contract JSON");
        let mut field_names: Vec<&str> = contract
            .required_fields
            .iter()
            .map(|field| field.name.as_str())
            .collect();
        let canonical_endpoint =
            serde_json::to_value(TransportEndpointSpec::external_http3("api-gateway-public"))
                .expect("serialize endpoint spec");
        let mut canonical_fields: Vec<&str> = canonical_endpoint
            .as_object()
            .expect("endpoint spec object")
            .keys()
            .map(String::as_str)
            .collect();
        field_names.sort_unstable();
        canonical_fields.sort_unstable();
        assert_eq!(field_names, canonical_fields);

        let external = contract
            .capability_classes
            .iter()
            .find(|class| class.name == "external")
            .expect("external class");
        assert_eq!(external.default_protocol, "http3");
        assert_eq!(external.tls_profile, "strict_tls13");
        assert_eq!(external.alt_svc, "required");
        assert_eq!(external.fallback_timeout_ms["default"], 500);
        assert_eq!(external.fallback_timeout_ms["max"], 12_500);
        assert_eq!(external.ech, "optional_profile_gated");
        assert_eq!(external.pqc, "optional_profile_gated");
        assert_eq!(
            external.http3_correctness_dependency.as_deref(),
            Some("forbidden")
        );
        assert_eq!(
            contract.canonical_external_endpoint,
            TransportEndpointSpec::external_http3("api-gateway-public")
        );
        assert_eq!(contract.canonical_external_endpoint.validate(), Ok(()));

        let inter_cell = contract
            .capability_classes
            .iter()
            .find(|class| class.name == "inter_cell")
            .expect("inter_cell class");
        assert_eq!(inter_cell.default_protocol, "grpc_http2");
        assert_eq!(inter_cell.tls_profile, "spiffe_mtls_tls13");
        assert_eq!(inter_cell.alt_svc, "forbidden");
        assert_eq!(inter_cell.fallback_timeout_ms, "forbidden");
        assert_eq!(inter_cell.ech, "forbidden");
        assert_eq!(inter_cell.pqc, "optional_profile_gated");

        let internal = contract
            .capability_classes
            .iter()
            .find(|class| class.name == "internal")
            .expect("internal class");
        assert_eq!(internal.default_protocol, "grpc_http2");
        assert_eq!(internal.tls_profile, "spiffe_mtls_tls13");
        assert_eq!(internal.alt_svc, "forbidden");
        assert_eq!(internal.fallback_timeout_ms, "forbidden");
        assert_eq!(internal.ech, "forbidden");
        assert_eq!(internal.pqc, "optional_profile_gated");

        let advanced = contract
            .advanced_profiles
            .iter()
            .find(|profile| profile.profile_id == "adr-0354-ech-hybrid-pqc")
            .expect("ADR-0354 advanced profile");
        assert_eq!(advanced.authority_status, "Proposed");
        assert_eq!(advanced.activation, "separate_accepted_profile_required");
        assert_eq!(advanced.runtime_activation_while_proposed, "forbidden");
        assert_eq!(advanced.base_constructor_activation, "forbidden");
        assert_eq!(
            advanced.canonical_external_endpoint,
            apply_adr_0354_shape(TransportEndpointSpec::external_http3("api-gateway-public"))
        );
        assert_eq!(
            advanced.canonical_external_endpoint.validate(),
            proposed_profile_activation_error()
        );

        let deferred = contract
            .deferred_adapters
            .iter()
            .find(|adapter| adapter.name == "s2n_quic")
            .expect("s2n_quic deferral");
        assert_eq!(deferred.status, "deferred_no_dependency");
    }

    #[test]
    fn proposed_adr_d7_example_matches_only_the_explicit_advanced_overlay() {
        let adr = adr_0354();
        let example = extract_json_between(
            &adr,
            "<!-- transport-profile-external-example:start -->",
            "<!-- transport-profile-external-example:end -->",
        );
        let endpoint: TransportEndpointSpec =
            serde_json::from_str(example).expect("ADR §D-7 endpoint example");

        assert_eq!(
            endpoint,
            apply_adr_0354_shape(TransportEndpointSpec::external_http3("api-gateway-public"))
        );
        assert_ne!(
            endpoint,
            TransportEndpointSpec::external_http3("api-gateway-public")
        );
        assert_eq!(endpoint.validate(), proposed_profile_activation_error());
    }

    fn advanced_external() -> TransportEndpointSpec {
        apply_adr_0354_shape(TransportEndpointSpec::external_http3(
            "public-inference-stream",
        ))
    }

    fn advanced_inter_cell() -> TransportEndpointSpec {
        apply_adr_0354_shape(TransportEndpointSpec::inter_cell_grpc_h2(
            "cell-rebalance-events",
        ))
    }

    fn apply_adr_0354_shape(mut spec: TransportEndpointSpec) -> TransportEndpointSpec {
        match spec.capability_class {
            TransportCapabilityClass::External => {
                spec.ech = EchPolicy::adr_0354_external();
                spec.pqc = PqcPolicy::adr_0354_hybrid_required();
            }
            TransportCapabilityClass::InterCell => {
                spec.pqc = PqcPolicy::adr_0354_hybrid_required();
            }
            TransportCapabilityClass::Internal => {}
        }
        spec
    }

    fn proposed_profile_activation_error() -> Result<(), TransportProfileError> {
        Err(TransportProfileError::InvalidField {
            field: "advanced_profile",
            reason: "ECH/PQC activation requires a separate Accepted authority profile",
        })
    }

    fn extract_json_between<'a>(body: &'a str, start: &str, end: &str) -> &'a str {
        let after_start = body.split_once(start).expect("start marker").1;
        let before_end = after_start.split_once(end).expect("end marker").0;
        before_end
            .split_once("```json")
            .expect("json fence")
            .1
            .split_once("```")
            .expect("closing json fence")
            .0
            .trim()
    }

    fn adr_0354() -> String {
        // Historical §D-7 JSON example markers remain on the archived 0354 body;
        // live apex 0705 consolidates doctrine without those HTML comment anchors.
        // include_str keeps the example hermetic under Buck sandboxes that may not
        // materialize the full docs tree at runtime.
        include_str!(
            "../../../../docs/adr-archive/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md"
        )
        .to_string()
    }

}
