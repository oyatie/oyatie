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
#[serde(rename_all = "snake_case")]
pub enum HybridKem {
    X25519MlKem768,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HybridSignature {
    Ed25519MlDsa65,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EchPolicy {
    /// data_class: PUBLIC - endpoint privacy posture advertised by the endpoint class.
    pub enabled: bool,
    /// data_class: PUBLIC - whether clients may proceed without ECH.
    pub mandatory: bool,
    /// data_class: INTERNAL_ONLY - operational rotation objective, not a key value.
    pub key_rotation_hours: Option<u16>,
    /// data_class: PUBLIC - whether retry configs are greased for compatibility.
    pub grease_retry_configs: bool,
    /// data_class: PUBLIC - public cover name, not tenant or service secret material.
    pub outer_sni: Option<String>,
}

impl EchPolicy {
    pub fn external() -> Self {
        Self {
            enabled: true,
            mandatory: true,
            key_rotation_hours: Some(24),
            grease_retry_configs: true,
            outer_sni: Some("api.oyatie.dev".to_owned()),
        }
    }

    pub fn disabled() -> Self {
        Self {
            enabled: false,
            mandatory: false,
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
    /// data_class: PUBLIC - whether classical-only transport is refused.
    pub mandatory: bool,
    /// data_class: PUBLIC - standardized hybrid KEM identifier.
    pub kem: Option<HybridKem>,
    /// data_class: PUBLIC - standardized hybrid signature identifier.
    pub signature: Option<HybridSignature>,
    /// data_class: PUBLIC - whether classical fallback is allowed during rollout.
    pub classical_fallback: bool,
}

impl PqcPolicy {
    pub fn hybrid_required() -> Self {
        Self {
            enabled: true,
            mandatory: true,
            kem: Some(HybridKem::X25519MlKem768),
            signature: Some(HybridSignature::Ed25519MlDsa65),
            classical_fallback: true,
        }
    }

    pub fn hybrid_optional() -> Self {
        Self {
            enabled: true,
            mandatory: false,
            kem: Some(HybridKem::X25519MlKem768),
            signature: Some(HybridSignature::Ed25519MlDsa65),
            classical_fallback: true,
        }
    }

    pub fn disabled() -> Self {
        Self {
            enabled: false,
            mandatory: false,
            kem: None,
            signature: None,
            classical_fallback: true,
        }
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
            tls_profile: TlsProfile::StrictTls13,
            alt_svc: Some(r#"h3=":443"; ma=86400"#.to_owned()),
            fallback_timeout_ms: Some(DEFAULT_EXTERNAL_FALLBACK_TIMEOUT_MS),
            ech: EchPolicy::external(),
            pqc: PqcPolicy::hybrid_required(),
        }
    }

    pub fn inter_cell_grpc_h2(endpoint_id: impl Into<String>) -> Self {
        Self {
            endpoint_id: endpoint_id.into(),
            capability_class: TransportCapabilityClass::InterCell,
            protocol: TransportProtocol::GrpcHttp2,
            tls_profile: TlsProfile::SpiffeMtlsTls13,
            alt_svc: None,
            fallback_timeout_ms: None,
            ech: EchPolicy::disabled(),
            pqc: PqcPolicy::hybrid_optional(),
        }
    }

    pub fn internal_grpc_h2(endpoint_id: impl Into<String>) -> Self {
        Self {
            endpoint_id: endpoint_id.into(),
            capability_class: TransportCapabilityClass::Internal,
            protocol: TransportProtocol::GrpcHttp2,
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

        if !self.ech.enabled || !self.ech.mandatory {
            return invalid("ech", "external endpoints require mandatory ech");
        }
        if self.ech.key_rotation_hours.is_none_or(|hours| hours > 24) {
            return invalid(
                "ech.key_rotation_hours",
                "external ech rotation must be <= 24h",
            );
        }
        if !self.ech.grease_retry_configs {
            return invalid(
                "ech.grease_retry_configs",
                "external ech requires grease retry configs",
            );
        }
        if self.ech.outer_sni.as_deref().unwrap_or_default().is_empty() {
            return invalid("ech.outer_sni", "external ech requires public cover name");
        }
        if !self.pqc.enabled || !self.pqc.mandatory {
            return invalid("pqc", "external endpoints require mandatory hybrid pqc");
        }
        if self.pqc.kem.is_none() || self.pqc.signature.is_none() {
            return invalid("pqc", "hybrid pqc requires kem and signature identifiers");
        }
        if !self.pqc.classical_fallback {
            return invalid(
                "pqc.classical_fallback",
                "external pqc requires classical fallback during transition",
            );
        }
        Ok(())
    }

    fn validate_inter_cell(&self) -> Result<(), TransportProfileError> {
        self.validate_non_external("inter-cell endpoints require grpc over http2")?;
        if !self.pqc.enabled {
            return invalid(
                "pqc",
                "inter-cell endpoints must declare hybrid pqc posture",
            );
        }
        if self.pqc.kem.is_none() || self.pqc.signature.is_none() {
            return invalid("pqc", "hybrid pqc requires kem and signature identifiers");
        }
        Ok(())
    }

    fn validate_internal(&self) -> Result<(), TransportProfileError> {
        self.validate_non_external("internal endpoints require grpc over http2")?;
        if self.pqc.enabled && (self.pqc.kem.is_none() || self.pqc.signature.is_none()) {
            return invalid(
                "pqc",
                "enabled hybrid pqc requires kem and signature identifiers",
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    const CONTRACT_SPEC: &str = include_str!("../endpoint-transport-profile.contract.json");

    #[derive(Deserialize)]
    struct ContractSpec {
        required_fields: Vec<ContractField>,
        capability_classes: Vec<CapabilityClass>,
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
    }

    #[derive(Deserialize)]
    struct DeferredAdapter {
        name: String,
        status: String,
    }

    #[test]
    fn accepts_external_http3_with_alt_svc_ech_and_pqc() {
        let spec = TransportEndpointSpec::external_http3("public-inference-stream");

        assert_eq!(spec.protocol, TransportProtocol::Http3);
        assert_eq!(spec.capability_class, TransportCapabilityClass::External);
        assert!(spec.alt_svc.as_deref().unwrap_or_default().contains("h3"));
        assert_eq!(spec.ech.enabled, true);
        assert_eq!(spec.ech.mandatory, true);
        assert_eq!(spec.pqc.enabled, true);
        assert_eq!(spec.pqc.mandatory, true);
        assert_eq!(spec.pqc.classical_fallback, true);
        assert_eq!(spec.validate(), Ok(()));
    }

    #[test]
    fn rejects_external_profile_without_http3() {
        let mut spec = TransportEndpointSpec::external_http3("public-inference-stream");
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
    fn rejects_external_profile_without_alt_svc() {
        let mut spec = TransportEndpointSpec::external_http3("public-inference-stream");
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
        assert_eq!(spec.pqc.enabled, true);
        assert_eq!(spec.pqc.mandatory, false);
        assert_eq!(spec.validate(), Ok(()));
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
        let mut spec = TransportEndpointSpec::external_http3("public-inference-stream");
        spec.pqc.classical_fallback = false;

        assert_eq!(
            spec.validate(),
            Err(TransportProfileError::InvalidField {
                field: "pqc.classical_fallback",
                reason: "external pqc requires classical fallback during transition"
            })
        );
    }

    #[test]
    fn rejects_external_profile_with_wrong_alt_svc_port() {
        let mut spec = TransportEndpointSpec::external_http3("public-inference-stream");
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
        let mut spec = TransportEndpointSpec::external_http3("public-inference-stream");
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
    fn contract_artifact_declares_protocol_boundary_and_adapter_deferral() {
        let contract: ContractSpec = serde_json::from_str(CONTRACT_SPEC).expect("contract JSON");
        let field_names: Vec<&str> = contract
            .required_fields
            .iter()
            .map(|field| field.name.as_str())
            .collect();

        for field in [
            "endpoint_id",
            "capability_class",
            "protocol",
            "tls_profile",
            "alt_svc",
            "fallback_timeout_ms",
            "ech",
            "pqc",
        ] {
            assert!(field_names.contains(&field), "missing {field}");
        }

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
        assert_eq!(external.ech, "mandatory");
        assert_eq!(
            external.pqc,
            "mandatory_hybrid_with_classical_transition_fallback"
        );

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
        assert_eq!(inter_cell.pqc, "hybrid_declared_rollout_optional");

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
        assert_eq!(internal.pqc, "optional");

        let deferred = contract
            .deferred_adapters
            .iter()
            .find(|adapter| adapter.name == "s2n_quic")
            .expect("s2n_quic deferral");
        assert_eq!(deferred.status, "deferred_no_dependency");
    }
}
