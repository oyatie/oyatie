//! Provider-neutral cold-bootstrap intent for the provisioning-console host.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use sha2::{Digest, Sha256};

const SHA256_PREFIX: &str = "sha256:";
const SHA256_HEX_LEN: usize = 64;
const DEVELOPMENT_CONSOLE_ENDPOINT: &str = "console.oyatie.dev";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BootstrapTargetRole {
    ProvisioningConsole,
    GenesisDisasterRecovery,
}

impl BootstrapTargetRole {
    const fn label(self) -> &'static str {
        match self {
            Self::ProvisioningConsole => "provisioning_console",
            Self::GenesisDisasterRecovery => "genesis_disaster_recovery",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BootstrapLifecycle {
    CreateIfAbsent,
}

impl BootstrapLifecycle {
    const fn label(self) -> &'static str {
        match self {
            Self::CreateIfAbsent => "create_if_absent",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BootstrapLogicalPlacement {
    pub environment: String,    // data_class: PUBLIC
    pub control_domain: String, // data_class: INTERNAL_ONLY
    pub geography: String,      // data_class: PUBLIC
    pub ordinal: u16,           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BootstrapEndpointIntent {
    pub dns_name: String,                   // data_class: PUBLIC
    pub outbound_tunnel_only: bool,         // data_class: PUBLIC
    pub require_edge_identity: bool,        // data_class: PUBLIC
    pub require_application_identity: bool, // data_class: PUBLIC
    pub require_principal_match: bool,      // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BootstrapPredecessorDisposition {
    pub retire_before_create: Vec<String>, // data_class: INTERNAL_ONLY
    pub retire_after_acceptance: Vec<String>, // data_class: INTERNAL_ONLY
    pub require_zero_reference_proof: bool, // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BootstrapRollbackPolicy {
    DeleteGreenResourcesBeforeCutover,
    QuarantineAndRedirectAfterCutover,
}

impl BootstrapRollbackPolicy {
    const fn label(self) -> &'static str {
        match self {
            Self::DeleteGreenResourcesBeforeCutover => "delete_green_before_cutover",
            Self::QuarantineAndRedirectAfterCutover => "quarantine_and_redirect_after_cutover",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BootstrapResourceKind {
    Compute,
    Network,
    Identity,
    Dns,
    SecretReference,
    Release,
    Audit,
    Endpoint,
}

impl BootstrapResourceKind {
    const REQUIRED: [Self; 8] = [
        Self::Compute,
        Self::Network,
        Self::Identity,
        Self::Dns,
        Self::SecretReference,
        Self::Release,
        Self::Audit,
        Self::Endpoint,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Compute => "compute",
            Self::Network => "network",
            Self::Identity => "identity",
            Self::Dns => "dns",
            Self::SecretReference => "secret_reference",
            Self::Release => "release",
            Self::Audit => "audit",
            Self::Endpoint => "endpoint",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BootstrapResourceIntent {
    pub kind: BootstrapResourceKind, // data_class: PUBLIC
    pub logical_ref: String,         // data_class: INTERNAL_ONLY
    pub immutable_ref: String,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootstrapComposition {
    pub resources: Vec<BootstrapResourceIntent>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootstrapIntent {
    pub intent_id: String,                            // data_class: INTERNAL_ONLY
    pub generation: u64,                              // data_class: INTERNAL_ONLY
    pub target_role: BootstrapTargetRole,             // data_class: PUBLIC
    pub lifecycle: BootstrapLifecycle,                // data_class: PUBLIC
    pub machine_class_ref: String,                    // data_class: INTERNAL_ONLY
    pub release_ref: String,                          // data_class: INTERNAL_ONLY
    pub placement: BootstrapLogicalPlacement,         // data_class: INTERNAL_ONLY
    pub endpoint: BootstrapEndpointIntent,            // data_class: INTERNAL_ONLY
    pub predecessor: BootstrapPredecessorDisposition, // data_class: INTERNAL_ONLY
    pub rollback_policy: BootstrapRollbackPolicy,     // data_class: PUBLIC
    pub composition: BootstrapComposition,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BootstrapIntentError {
    EmptyField { field: &'static str },
    InvalidGeneration,
    UnsupportedTargetRole,
    InvalidEndpoint,
    InvalidImmutableReference { field: &'static str },
    ProviderDetailLeak { field: &'static str },
    InvalidPredecessorOrder,
    DuplicateResourceKind { kind: BootstrapResourceKind },
    MissingResourceKind { kind: BootstrapResourceKind },
}

impl std::fmt::Display for BootstrapIntentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyField { field } => write!(formatter, "{field} must not be empty"),
            Self::InvalidGeneration => formatter.write_str("generation must be non-zero"),
            Self::UnsupportedTargetRole => {
                formatter.write_str("bootstrap target role is not permitted")
            }
            Self::InvalidEndpoint => formatter.write_str("endpoint intent is not fail-closed"),
            Self::InvalidImmutableReference { field } => {
                write!(formatter, "{field} must be immutable and digest-addressed")
            }
            Self::ProviderDetailLeak { field } => {
                write!(formatter, "{field} contains provider-owned binding data")
            }
            Self::InvalidPredecessorOrder => {
                formatter.write_str("predecessor retirement order is invalid")
            }
            Self::DuplicateResourceKind { kind } => {
                write!(formatter, "resource kind {} is duplicated", kind.label())
            }
            Self::MissingResourceKind { kind } => {
                write!(formatter, "resource kind {} is missing", kind.label())
            }
        }
    }
}

impl std::error::Error for BootstrapIntentError {}

impl BootstrapIntent {
    pub fn validate(&self) -> Result<(), BootstrapIntentError> {
        if self.generation == 0 {
            return Err(BootstrapIntentError::InvalidGeneration);
        }
        if self.target_role != BootstrapTargetRole::ProvisioningConsole {
            return Err(BootstrapIntentError::UnsupportedTargetRole);
        }
        for (field, value) in [
            ("intent_id", self.intent_id.as_str()),
            ("machine_class_ref", self.machine_class_ref.as_str()),
            ("environment", self.placement.environment.as_str()),
            ("control_domain", self.placement.control_domain.as_str()),
            ("geography", self.placement.geography.as_str()),
        ] {
            validate_logical(field, value)?;
        }
        validate_digest("release_ref", &self.release_ref)?;
        if self.endpoint.dns_name != DEVELOPMENT_CONSOLE_ENDPOINT
            || !self.endpoint.outbound_tunnel_only
            || !self.endpoint.require_edge_identity
            || !self.endpoint.require_application_identity
            || !self.endpoint.require_principal_match
        {
            return Err(BootstrapIntentError::InvalidEndpoint);
        }
        validate_logical("endpoint.dns_name", &self.endpoint.dns_name)?;

        if self.predecessor.retire_before_create != ["legacy-remote-access-vpn".to_string()]
            || self.predecessor.retire_after_acceptance
                != ["legacy-external-health-monitor".to_string()]
            || !self.predecessor.require_zero_reference_proof
        {
            return Err(BootstrapIntentError::InvalidPredecessorOrder);
        }
        for predecessor in self
            .predecessor
            .retire_before_create
            .iter()
            .chain(self.predecessor.retire_after_acceptance.iter())
        {
            validate_logical("predecessor", predecessor)?;
        }
        self.composition.validate()
    }

    pub fn digest(&self) -> Result<String, BootstrapIntentError> {
        self.validate()?;
        let resources = self
            .composition
            .canonical_resources()
            .into_iter()
            .map(|resource| {
                format!(
                    "{}:{}:{}",
                    resource.kind.label(),
                    resource.logical_ref,
                    resource.immutable_ref
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        Ok(sha256(&format!(
            "intent_id={}|generation={}|target_role={}|lifecycle={}|machine_class_ref={}|release_ref={}|environment={}|control_domain={}|geography={}|ordinal={}|dns_name={}|outbound_tunnel_only={}|require_edge_identity={}|require_application_identity={}|require_principal_match={}|retire_before_create={}|retire_after_acceptance={}|zero_reference={}|rollback={}|resources={resources}",
            self.intent_id,
            self.generation,
            self.target_role.label(),
            self.lifecycle.label(),
            self.machine_class_ref,
            self.release_ref,
            self.placement.environment,
            self.placement.control_domain,
            self.placement.geography,
            self.placement.ordinal,
            self.endpoint.dns_name,
            self.endpoint.outbound_tunnel_only,
            self.endpoint.require_edge_identity,
            self.endpoint.require_application_identity,
            self.endpoint.require_principal_match,
            self.predecessor.retire_before_create.join(","),
            self.predecessor.retire_after_acceptance.join(","),
            self.predecessor.require_zero_reference_proof,
            self.rollback_policy.label(),
        )))
    }
}

impl BootstrapComposition {
    pub fn validate(&self) -> Result<(), BootstrapIntentError> {
        let mut kinds = BTreeSet::new();
        for resource in &self.resources {
            if !kinds.insert(resource.kind) {
                return Err(BootstrapIntentError::DuplicateResourceKind {
                    kind: resource.kind,
                });
            }
            validate_logical("resource.logical_ref", &resource.logical_ref)?;
            validate_digest("resource.immutable_ref", &resource.immutable_ref)?;
        }
        for kind in BootstrapResourceKind::REQUIRED {
            if !kinds.contains(&kind) {
                return Err(BootstrapIntentError::MissingResourceKind { kind });
            }
        }
        Ok(())
    }

    fn canonical_resources(&self) -> Vec<&BootstrapResourceIntent> {
        let mut resources: Vec<_> = self.resources.iter().collect();
        resources.sort_by_key(|resource| resource.kind);
        resources
    }
}

fn validate_logical(field: &'static str, value: &str) -> Result<(), BootstrapIntentError> {
    if value.trim().is_empty()
        || value.len() > 256
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
    {
        return Err(BootstrapIntentError::EmptyField { field });
    }
    let lower = value.to_ascii_lowercase();
    if [
        "ocid1.",
        "arn:",
        "e2.1.micro",
        "ap-chuncheon-1",
        "availability-domain",
        "compartment",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        return Err(BootstrapIntentError::ProviderDetailLeak { field });
    }
    Ok(())
}

fn validate_digest(field: &'static str, value: &str) -> Result<(), BootstrapIntentError> {
    if value.strip_prefix(SHA256_PREFIX).is_none_or(|hex| {
        hex.len() != SHA256_HEX_LEN
            || !hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }) {
        return Err(BootstrapIntentError::InvalidImmutableReference { field });
    }
    Ok(())
}

fn sha256(value: &str) -> String {
    let bytes = Sha256::digest(value.as_bytes());
    let mut output = String::with_capacity(SHA256_PREFIX.len() + SHA256_HEX_LEN);
    output.push_str(SHA256_PREFIX);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn intent() -> BootstrapIntent {
        BootstrapIntent {
            intent_id: "bootstrap-development-provisioning-console-1".to_string(),
            generation: 1,
            target_role: BootstrapTargetRole::ProvisioningConsole,
            lifecycle: BootstrapLifecycle::CreateIfAbsent,
            machine_class_ref: "machine-class/development-edge-v1".to_string(),
            release_ref: DIGEST.to_string(),
            placement: BootstrapLogicalPlacement {
                environment: "development".to_string(),
                control_domain: "control".to_string(),
                geography: "korea-northeast".to_string(),
                ordinal: 1,
            },
            endpoint: BootstrapEndpointIntent {
                dns_name: DEVELOPMENT_CONSOLE_ENDPOINT.to_string(),
                outbound_tunnel_only: true,
                require_edge_identity: true,
                require_application_identity: true,
                require_principal_match: true,
            },
            predecessor: BootstrapPredecessorDisposition {
                retire_before_create: vec!["legacy-remote-access-vpn".to_string()],
                retire_after_acceptance: vec!["legacy-external-health-monitor".to_string()],
                require_zero_reference_proof: true,
            },
            rollback_policy: BootstrapRollbackPolicy::QuarantineAndRedirectAfterCutover,
            composition: BootstrapComposition {
                resources: BootstrapResourceKind::REQUIRED
                    .into_iter()
                    .map(|kind| BootstrapResourceIntent {
                        kind,
                        logical_ref: format!("resource/{}", kind.label()),
                        immutable_ref: DIGEST.to_string(),
                    })
                    .collect(),
            },
        }
    }

    #[test]
    fn intent_is_deterministic_across_resource_input_order() {
        let first = intent();
        let mut second = intent();
        second.composition.resources.reverse();
        assert_eq!(
            first.digest().expect("valid"),
            second.digest().expect("valid")
        );
    }

    #[test]
    fn intent_rejects_a1_target() {
        let mut candidate = intent();
        candidate.target_role = BootstrapTargetRole::GenesisDisasterRecovery;
        assert_eq!(
            candidate.validate(),
            Err(BootstrapIntentError::UnsupportedTargetRole)
        );
    }

    #[test]
    fn intent_rejects_provider_owned_details() {
        let mut candidate = intent();
        candidate.machine_class_ref = "E2.1.Micro".to_string();
        assert_eq!(
            candidate.validate(),
            Err(BootstrapIntentError::ProviderDetailLeak {
                field: "machine_class_ref"
            })
        );
    }

    #[test]
    fn intent_rejects_wrong_retirement_order() {
        let mut candidate = intent();
        candidate.predecessor.retire_before_create =
            vec!["legacy-external-health-monitor".to_string()];
        assert_eq!(
            candidate.validate(),
            Err(BootstrapIntentError::InvalidPredecessorOrder)
        );
    }

    #[test]
    fn intent_requires_complete_composition() {
        let mut candidate = intent();
        candidate
            .composition
            .resources
            .retain(|resource| resource.kind != BootstrapResourceKind::Audit);
        assert_eq!(
            candidate.validate(),
            Err(BootstrapIntentError::MissingResourceKind {
                kind: BootstrapResourceKind::Audit
            })
        );
    }
}
