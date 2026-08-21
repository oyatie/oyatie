use std::collections::BTreeSet;

use crate::GATE_ID;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: &'static str,
    pub path: String,
    pub detail: String,
}

impl Finding {
    pub(crate) fn new(
        code: &'static str,
        path: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            code,
            path: path.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateError {
    Io(String),
}

impl std::fmt::Display for GateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateError::Io(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for GateError {}

#[derive(Debug, Clone)]
pub struct GateReport {
    pub verdict: Verdict,
    pub findings: BTreeSet<Finding>,
}

pub(crate) fn report(findings: BTreeSet<Finding>) -> GateReport {
    GateReport {
        verdict: if findings.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        },
        findings,
    }
}

pub fn render_findings(report: &GateReport) -> String {
    if report.findings.is_empty() {
        return format!("{GATE_ID}: GREEN — deps.toml owned updater contract is valid");
    }
    let mut out = format!(
        "{GATE_ID}: RED — {} dependency automation finding(s)",
        report.findings.len()
    );
    for finding in &report.findings {
        out.push_str(&format!(
            "\n{} {}: {}",
            finding.code, finding.path, finding.detail
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_green_is_compact() {
        let report = report(BTreeSet::new());
        assert_eq!(report.verdict, Verdict::Green);
        assert!(render_findings(&report).contains("GREEN"));
    }
}
