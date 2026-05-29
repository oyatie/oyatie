//! Advisory gate: enforce the tier boundary between Envoy edge authz and
//! Cedar origin authz per ADR-0191.
//!
//! Two scans, each reports findings:
//!
//! 1. [`scan_cedar`] — Cedar policy source must NOT reference edge concerns
//!    (ip, asn, geo, country, rate, waf, bot, ddos).
//! 2. [`scan_envoy_filter`] — Envoy filter config (YAML / JSON) must NOT
//!    reference origin concerns (oidc principal claims, acr, tenant
//!    identity, residency, purpose, data_class).
//!
//! Both are TEXTUAL heuristics: tokenize on word boundaries and look for
//! the canonical needles. The intent is to catch drift early, not to
//! mechanically verify policy semantics. False-positives are documented
//! per finding so authors can either suppress (with rationale) or refactor.
//!
//! A finding emits the source-line-number, the matching needle, the
//! kind, and a suggested remediation. The report is plain Rust data
//! (`serde`-serialisable) so CI lanes can render it however they need.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tier {
    Edge,
    Origin,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Finding {
    pub file: String,
    pub line: usize,
    pub needle: String,
    pub source_tier: Tier,
    pub wrong_concern_tier: Tier,
    pub remediation: String,
    pub line_text: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct DisciplineReport {
    pub findings: Vec<Finding>,
}

impl DisciplineReport {
    pub fn ok(&self) -> bool {
        self.findings.is_empty()
    }
}

const CEDAR_FORBIDDEN_NEEDLES: &[&str] = &[
    // Network-layer attributes belong at the edge per ADR-0191.
    "client_ip",
    "remote_ip",
    "source_ip",
    "asn",
    "geoip",
    "country_code",
    "geolocation",
    "rate_limit",
    "ratelimit",
    "waf",
    "bot_score",
    "ddos",
    "user_agent_regex",
];

const ENVOY_FORBIDDEN_NEEDLES: &[&str] = &[
    // Identity / principal-aware concerns belong at the origin per ADR-0191.
    "principal.acr",
    "acr_level",
    "acr_required",
    "principal.tenant_id",
    "tenant_residency",
    "data_class",
    "purpose_binding",
    "step_up_required",
    "cedar_principal",
    "oidc_subject",
];

/// Suppression marker: a line ending with `// authz-tier-discipline: ok
/// (<reason>)` is intentionally permitted.
fn line_is_suppressed(line: &str) -> bool {
    line.contains("authz-tier-discipline: ok")
}

pub fn scan_cedar(file: &str, body: &str) -> DisciplineReport {
    let mut findings = Vec::new();
    for (idx, line) in body.lines().enumerate() {
        if line_is_suppressed(line) {
            continue;
        }
        let lc = line.to_ascii_lowercase();
        for needle in CEDAR_FORBIDDEN_NEEDLES {
            if lc.contains(needle) {
                findings.push(Finding {
                    file: file.to_owned(),
                    line: idx + 1,
                    needle: (*needle).to_owned(),
                    source_tier: Tier::Origin,
                    wrong_concern_tier: Tier::Edge,
                    remediation: format!(
                        "Move '{needle}' enforcement to Envoy edge filter per ADR-0191. The origin Cedar PDP must not see network-layer attributes."
                    ),
                    line_text: line.trim().to_owned(),
                });
            }
        }
    }
    DisciplineReport { findings }
}

pub fn scan_envoy_filter(file: &str, body: &str) -> DisciplineReport {
    let mut findings = Vec::new();
    for (idx, line) in body.lines().enumerate() {
        if line_is_suppressed(line) {
            continue;
        }
        let lc = line.to_ascii_lowercase();
        for needle in ENVOY_FORBIDDEN_NEEDLES {
            if lc.contains(needle) {
                findings.push(Finding {
                    file: file.to_owned(),
                    line: idx + 1,
                    needle: (*needle).to_owned(),
                    source_tier: Tier::Edge,
                    wrong_concern_tier: Tier::Origin,
                    remediation: format!(
                        "Move '{needle}' enforcement to Cedar origin PDP per ADR-0191. The edge filter must not consume identity claims."
                    ),
                    line_text: line.trim().to_owned(),
                });
            }
        }
    }
    DisciplineReport { findings }
}

/// Combined scan: cedar policies + envoy filter configs.
pub fn scan_combined(cedar: &[(&str, &str)], envoy: &[(&str, &str)]) -> DisciplineReport {
    let mut report = DisciplineReport::default();
    for (f, b) in cedar {
        report.findings.extend(scan_cedar(f, b).findings);
    }
    for (f, b) in envoy {
        report.findings.extend(scan_envoy_filter(f, b).findings);
    }
    report
}
