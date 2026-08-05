//! SemVer'd oya-ci gate contract surface.
//!
//! ADR-0515 WS-D established the pure `Finding` / `evaluate_keyed` gate shape.
//! ADR-0528 extends that shape with `remediate`: a pure sibling that returns a
//! described edit or new file and never performs the write itself. Delivery of
//! described edits belongs to a privileged caller outside this contract.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Version of the source-level gate contract ABI exposed by this crate.
pub const SDK_ABI_VERSION: u32 = 1;

/// The canonical crate name ratified by ADR-0528.
pub const CONTRACT_CRATE_NAME: &str = "oya-ci-gate-contract";

/// The crate semver used by registries and gate manifests.
pub const CONTRACT_SEMVER: &str = match option_env!("CARGO_PKG_VERSION") {
    Some(version) => version,
    None => "0.1.0",
};

/// A keyed gate violation: stable `code` plus the offending unit `key`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Finding {
    pub code: String,
    pub key: String,
}

impl Finding {
    pub fn new(code: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            key: key.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Verdict {
    Green,
    Red,
}

/// Bare-code projection of a gate run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

impl Report {
    pub fn from_findings(findings: BTreeSet<Finding>) -> Self {
        let violations: BTreeSet<String> =
            findings.into_iter().map(|finding| finding.code).collect();
        let verdict = if violations.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        };
        Self {
            verdict,
            violations,
        }
    }
}

/// Half-open byte range `[start, end)` in an existing UTF-8 artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ByteRange {
    pub start: usize,
    pub end: usize,
}

impl ByteRange {
    pub fn new(start: usize, end: usize) -> Result<Self, ContractError> {
        if start > end {
            return Err(ContractError::InvalidByteRange { start, end });
        }
        Ok(Self { start, end })
    }
}

/// Described replacement for an existing artifact. This is data, not an applied write.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edit {
    pub path: String,
    pub byte_range: ByteRange,
    pub replacement: String,
}

impl Edit {
    pub fn new(
        path: impl Into<String>,
        byte_range: ByteRange,
        replacement: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            byte_range,
            replacement: replacement.into(),
        }
    }
}

/// Described creation of a new artifact. This is data, not an applied write.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewFile {
    pub path: String,
    pub body: String,
}

impl NewFile {
    pub fn new(path: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            body: body.into(),
        }
    }
}

/// Pure remediation response for a concrete finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Remediation {
    AutoFix(Edit),
    AutoGenerate(NewFile),
    None,
}

/// Registration-time declaration for every finding code a gate can emit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemediationTier {
    AutoFix,
    AutoGenerate,
    Block { rationale: String },
}

/// One stable violation code plus its remediation tier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateCode {
    pub code: String,
    pub remediation_tier: RemediationTier,
}

impl GateCode {
    pub fn new(code: impl Into<String>, remediation_tier: RemediationTier) -> Self {
        Self {
            code: code.into(),
            remediation_tier,
        }
    }
}

/// Published gate manifest carrying the semver/ABI and per-code remediation declarations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateManifest {
    pub gate_id: String,
    pub sdk_abi_version: u32,
    pub contract_semver: String,
    pub codes: Vec<GateCode>,
}

impl GateManifest {
    pub fn new(gate_id: impl Into<String>, codes: Vec<GateCode>) -> Result<Self, ContractError> {
        let manifest = Self {
            gate_id: gate_id.into(),
            sdk_abi_version: SDK_ABI_VERSION,
            contract_semver: CONTRACT_SEMVER.to_owned(),
            codes,
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if self.gate_id.trim().is_empty() {
            return Err(ContractError::EmptyGateId);
        }

        let mut seen = BTreeSet::new();
        for code in &self.codes {
            if code.code.trim().is_empty() {
                return Err(ContractError::EmptyCode);
            }
            if !seen.insert(code.code.clone()) {
                return Err(ContractError::DuplicateCode {
                    code: code.code.clone(),
                });
            }
            if let RemediationTier::Block { rationale } = &code.remediation_tier
                && rationale.trim().is_empty()
            {
                return Err(ContractError::EmptyBlockRationale {
                    code: code.code.clone(),
                });
            }
        }
        Ok(())
    }
}

/// The source-level oya-ci gate interface.
pub trait Gate {
    fn manifest(&self) -> &GateManifest;

    /// Pure detection over a producer-built face.
    fn evaluate_keyed(&self, face: &Value) -> BTreeSet<Finding>;

    /// Pure remediation over the same face. Implementations return described edits only.
    fn remediate(&self, finding: &Finding, face: &Value) -> Remediation;

    /// Bare-code projection of [`Gate::evaluate_keyed`].
    fn evaluate(&self, face: &Value) -> Report {
        Report::from_findings(self.evaluate_keyed(face))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractError {
    EmptyGateId,
    EmptyCode,
    EmptyBlockRationale { code: String },
    DuplicateCode { code: String },
    InvalidByteRange { start: usize, end: usize },
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyGateId => write!(f, "gate_id must not be empty"),
            Self::EmptyCode => write!(f, "gate code must not be empty"),
            Self::EmptyBlockRationale { code } => {
                write!(
                    f,
                    "block remediation tier for `{code}` requires a rationale"
                )
            }
            Self::DuplicateCode { code } => write!(f, "duplicate gate code `{code}`"),
            Self::InvalidByteRange { start, end } => {
                write!(f, "byte range start {start} must be <= end {end}")
            }
        }
    }
}

impl Error for ContractError {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeSet;

    struct FixtureGate {
        manifest: GateManifest,
    }

    impl FixtureGate {
        fn new() -> Self {
            Self {
                manifest: GateManifest::new(
                    "cloud-ci-manifest-hygiene",
                    vec![
                        GateCode::new("manifest_missing_license", RemediationTier::AutoFix),
                        GateCode::new(
                            "manifest_changes_runtime_semantics",
                            RemediationTier::Block {
                                rationale: "runtime semantics require review".to_owned(),
                            },
                        ),
                    ],
                )
                .expect("fixture manifest is valid"),
            }
        }
    }

    impl Gate for FixtureGate {
        fn manifest(&self) -> &GateManifest {
            &self.manifest
        }

        fn evaluate_keyed(&self, face: &serde_json::Value) -> BTreeSet<Finding> {
            let mut findings = BTreeSet::new();
            if face.get("has_license").and_then(serde_json::Value::as_bool) == Some(false) {
                findings.insert(Finding::new(
                    "manifest_missing_license",
                    "libs/example/Cargo.toml",
                ));
            }
            findings
        }

        fn remediate(&self, finding: &Finding, face: &serde_json::Value) -> Remediation {
            if finding.code == "manifest_missing_license"
                && face.get("has_license").and_then(serde_json::Value::as_bool) == Some(false)
            {
                Remediation::AutoFix(Edit::new(
                    "libs/example/Cargo.toml",
                    ByteRange::new(37, 37).expect("valid insert range"),
                    "license = \"Apache-2.0\"\n",
                ))
            } else {
                Remediation::None
            }
        }
    }

    #[test]
    fn evaluate_is_bare_code_projection_of_evaluate_keyed() {
        let gate = FixtureGate::new();
        let report = gate.evaluate(&json!({"has_license": false}));

        assert_eq!(report.verdict, Verdict::Red);
        assert_eq!(
            report.violations,
            BTreeSet::from(["manifest_missing_license".to_owned()])
        );
    }

    #[test]
    fn remediate_returns_described_edit_without_applying_it() {
        let gate = FixtureGate::new();
        let finding = Finding::new("manifest_missing_license", "libs/example/Cargo.toml");

        let remediation = gate.remediate(&finding, &json!({"has_license": false}));

        assert_eq!(
            remediation,
            Remediation::AutoFix(Edit::new(
                "libs/example/Cargo.toml",
                ByteRange::new(37, 37).expect("valid insert range"),
                "license = \"Apache-2.0\"\n",
            ))
        );
    }

    #[test]
    fn remediate_returns_none_after_the_described_fix_is_present() {
        let gate = FixtureGate::new();
        let finding = Finding::new("manifest_missing_license", "libs/example/Cargo.toml");

        let tier = gate
            .manifest()
            .codes
            .iter()
            .find(|code| code.code == "manifest_missing_license")
            .map(|code| &code.remediation_tier);

        assert_eq!(tier, Some(&RemediationTier::AutoFix));
        assert!(
            gate.evaluate_keyed(&json!({"has_license": true}))
                .is_empty()
        );
        assert_eq!(
            gate.remediate(&finding, &json!({"has_license": true})),
            Remediation::None
        );
    }

    #[test]
    fn gate_manifest_requires_block_rationale_per_code() {
        let manifest = GateManifest::new(
            "cloud-ci-example",
            vec![GateCode::new(
                "semantic_change",
                RemediationTier::Block {
                    rationale: String::new(),
                },
            )],
        );

        assert_eq!(
            manifest,
            Err(ContractError::EmptyBlockRationale {
                code: "semantic_change".to_owned(),
            })
        );
    }

    #[test]
    fn gate_manifest_rejects_duplicate_codes() {
        let manifest = GateManifest::new(
            "cloud-ci-example",
            vec![
                GateCode::new("duplicate", RemediationTier::AutoFix),
                GateCode::new("duplicate", RemediationTier::AutoGenerate),
            ],
        );

        assert_eq!(
            manifest,
            Err(ContractError::DuplicateCode {
                code: "duplicate".to_owned(),
            })
        );
    }

    #[test]
    fn byte_range_is_half_open_and_rejects_inverted_ranges() {
        assert_eq!(
            ByteRange::new(9, 3),
            Err(ContractError::InvalidByteRange { start: 9, end: 3 })
        );
    }
}
