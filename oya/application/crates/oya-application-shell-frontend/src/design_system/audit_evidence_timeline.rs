//! DS-AUDIT_EVIDENCE_TIMELINE (`specs/design-system/audit-evidence-timeline.json`).
//!
//! Provenance timeline for changesets, incidents, compliance controls, agent
//! decision chains, and release evidence. Spec security invariants:
//!
//! 1. missing audit rows render as BLOCKING gaps, never warnings;
//! 2. evidence paths are immutable links to repo-tracked or signed external
//!    artifacts (closed [`EvidencePath`] enum — a mutable/raw URL is
//!    unrepresentable);
//! 3. export preserves redactions and chain-of-custody metadata (carried by
//!    construction in [`ExportPacket`]).

use leptos::prelude::*;

/// Spec `variants`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimelineVariant {
    ChangesetProvenance,
    IncidentCloseout,
    ComplianceControl,
    AgentDecisionChain,
    ReleaseEvidence,
}

impl TimelineVariant {
    pub const fn id(self) -> &'static str {
        match self {
            Self::ChangesetProvenance => "changeset-provenance",
            Self::IncidentCloseout => "incident-closeout",
            Self::ComplianceControl => "compliance-control",
            Self::AgentDecisionChain => "agent-decision-chain",
            Self::ReleaseEvidence => "release-evidence",
        }
    }
}

/// Spec `states` for a row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RowState {
    Complete,
    MissingRow,
    StaleEvidence,
    SignatureMissing,
    PermissionDenied,
    ExportReady,
}

/// Invariant 2: an evidence link is either a repo-tracked path or a signed
/// external artifact (digest + signature reference). There is no raw-URL
/// variant to mutate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvidencePath {
    RepoTracked {
        path: String,
    },
    SignedExternal {
        digest: String,
        signature_ref: String,
    },
}

impl EvidencePath {
    pub fn display(&self) -> String {
        match self {
            Self::RepoTracked { path } => path.clone(),
            Self::SignedExternal { digest, .. } => format!("signed:{digest}"),
        }
    }
}

/// One timeline row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceRow {
    pub state: RowState,
    pub event_type: String,
    pub timestamp: String,
    pub evidence: Option<EvidencePath>,
    pub signature_status: String,
    pub linked_change_id: String,
}

/// Severity the row renders with. Invariant 1 lives here: a missing row is
/// `Blocking`, full stop — there is no code path that softens it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RowSeverity {
    Ok,
    Warning,
    Blocking,
}

pub fn row_severity(row: &EvidenceRow) -> RowSeverity {
    match row.state {
        RowState::Complete | RowState::ExportReady => RowSeverity::Ok,
        RowState::StaleEvidence => RowSeverity::Warning,
        RowState::MissingRow | RowState::SignatureMissing | RowState::PermissionDenied => {
            RowSeverity::Blocking
        }
    }
}

/// Invariant 3: an export packet always carries redactions + chain-of-custody
/// metadata; both fields are mandatory at construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportPacket {
    rows: Vec<EvidenceRow>,
    redactions: Vec<String>,
    chain_of_custody: String,
}

impl ExportPacket {
    pub fn new(rows: Vec<EvidenceRow>, redactions: Vec<String>, chain_of_custody: String) -> Self {
        Self {
            rows,
            redactions,
            chain_of_custody,
        }
    }

    pub fn redactions(&self) -> &[String] {
        &self.redactions
    }

    pub fn chain_of_custody(&self) -> &str {
        &self.chain_of_custody
    }

    pub fn rows(&self) -> &[EvidenceRow] {
        &self.rows
    }
}

/// WCAG 2.2 AA timeline: an ordered list with per-row announcements covering
/// event type, timestamp, evidence path, freshness, signature status, and
/// linked change id; blocking gaps render with `role="alert"`.
#[component]
pub fn AuditEvidenceTimeline(variant: TimelineVariant, rows: Vec<EvidenceRow>) -> impl IntoView {
    view! {
        <section
            class="ds-audit-evidence-timeline"
            data-variant=variant.id()
            aria-label="Audit evidence timeline"
        >
            <ol>
                {rows
                    .into_iter()
                    .map(|row| {
                        let severity = row_severity(&row);
                        let blocking = severity == RowSeverity::Blocking;
                        let evidence_label = row
                            .evidence
                            .as_ref()
                            .map(EvidencePath::display)
                            .unwrap_or_else(|| "evidence missing".to_owned());
                        view! {
                            <li
                                data-severity=match severity {
                                    RowSeverity::Ok => "ok",
                                    RowSeverity::Warning => "warning",
                                    RowSeverity::Blocking => "blocking",
                                }
                                role=if blocking { "alert" } else { "listitem" }
                            >
                                <time>{row.timestamp.clone()}</time>
                                <strong>{row.event_type.clone()}</strong>
                                <span class="ds-evidence-path">{evidence_label}</span>
                                <span class="ds-signature-status">{row.signature_status.clone()}</span>
                                <span class="ds-linked-change">{row.linked_change_id.clone()}</span>
                                {blocking
                                    .then(|| view! {
                                        <p class="ds-blocking-gap">
                                            "Blocking gap: this row must be resolved before the timeline can certify"
                                        </p>
                                    })}
                            </li>
                        }
                    })
                    .collect_view()}
            </ol>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(state: RowState, evidence: Option<EvidencePath>) -> EvidenceRow {
        EvidenceRow {
            state,
            event_type: "changeset-merged".to_owned(),
            timestamp: "2026-06-10T03:00:00Z".to_owned(),
            evidence,
            signature_status: "ssh-signed".to_owned(),
            linked_change_id: "PR-652".to_owned(),
        }
    }

    #[test]
    fn missing_rows_are_blocking_not_warnings() {
        assert_eq!(
            row_severity(&row(RowState::MissingRow, None)),
            RowSeverity::Blocking
        );
        assert_eq!(
            row_severity(&row(RowState::SignatureMissing, None)),
            RowSeverity::Blocking
        );
        assert_eq!(
            row_severity(&row(RowState::StaleEvidence, None)),
            RowSeverity::Warning
        );
        assert_eq!(
            row_severity(&row(
                RowState::Complete,
                Some(EvidencePath::RepoTracked {
                    path: "evidence/audit-chain.jsonl".to_owned()
                })
            )),
            RowSeverity::Ok
        );
    }

    #[test]
    fn evidence_paths_are_repo_tracked_or_signed_only() {
        let repo = EvidencePath::RepoTracked {
            path: "evidence/audit-chain.jsonl".to_owned(),
        };
        let signed = EvidencePath::SignedExternal {
            digest: "blake3:abc123".to_owned(),
            signature_ref: "sig/release-7".to_owned(),
        };
        assert_eq!(repo.display(), "evidence/audit-chain.jsonl");
        assert_eq!(signed.display(), "signed:blake3:abc123");
    }

    #[test]
    fn export_preserves_redactions_and_chain_of_custody() {
        let packet = ExportPacket::new(
            vec![row(RowState::ExportReady, None)],
            vec!["tenant-name".to_owned()],
            "custody/REC-EXPORT-1".to_owned(),
        );
        assert_eq!(packet.redactions(), ["tenant-name".to_owned()]);
        assert_eq!(packet.chain_of_custody(), "custody/REC-EXPORT-1");
        assert_eq!(packet.rows().len(), 1);
    }
}
