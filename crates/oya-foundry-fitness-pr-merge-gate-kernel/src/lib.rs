//! Engineering Excellence Council merge-gate kernel (M-CC-P07-IP-003).
//!
//! Pure I/O-free Rust replacement for the earlier Node.js merge-gate
//! script (retired 2026-05-14 per user directive "no shellscript no mjs
//! etc all rust"). Runners feed a PR body string to [`parse_council_signature`]
//! and check [`CouncilSignature::is_approved`] before allowing the merge
//! queue to admit a ChangeSet.
//!
//! Approved verdicts: `APPROVE` and `APPROVE_WITH_NITS`. Both an
//! `Architect:` line and a `Critic:` line must carry an approved verdict
//! for the gate to pass.

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CouncilVerdict {
    Approve,
    ApproveWithNits,
    Reject,
    Defer,
}

impl CouncilVerdict {
    pub fn parse(token: &str) -> Option<Self> {
        match token.to_ascii_uppercase().as_str() {
            "APPROVE" => Some(Self::Approve),
            "APPROVE_WITH_NITS" => Some(Self::ApproveWithNits),
            "REJECT" => Some(Self::Reject),
            "DEFER" => Some(Self::Defer),
            _ => None,
        }
    }

    pub fn is_approved(self) -> bool {
        matches!(self, Self::Approve | Self::ApproveWithNits)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Approve => "APPROVE",
            Self::ApproveWithNits => "APPROVE_WITH_NITS",
            Self::Reject => "REJECT",
            Self::Defer => "DEFER",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CouncilEntry {
    pub name: String,            // data_class: INTERNAL_ONLY
    pub verdict: CouncilVerdict, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CouncilSignature {
    pub architect: Option<CouncilEntry>, // data_class: INTERNAL_ONLY
    pub critic: Option<CouncilEntry>,    // data_class: INTERNAL_ONLY
}

impl CouncilSignature {
    pub fn is_approved(&self) -> bool {
        match (&self.architect, &self.critic) {
            (Some(a), Some(c)) => a.verdict.is_approved() && c.verdict.is_approved(),
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MergeGateError {
    MissingArchitect,
    MissingCritic,
    ArchitectNotApproved { verdict: CouncilVerdict },
    CriticNotApproved { verdict: CouncilVerdict },
}

impl MergeGateError {
    pub fn message(&self) -> String {
        match self {
            Self::MissingArchitect => "missing Architect signature".to_owned(),
            Self::MissingCritic => "missing Critic signature".to_owned(),
            Self::ArchitectNotApproved { verdict } => {
                format!("Architect verdict not approved: {}", verdict.as_str())
            }
            Self::CriticNotApproved { verdict } => {
                format!("Critic verdict not approved: {}", verdict.as_str())
            }
        }
    }
}

/// Parse a PR body and extract the Council signature block.
///
/// Block boundary: starts at a line matching `## Council Signature`
/// (case-insensitive), ends at the next `## ` heading or EOF. Within
/// the block, lines matching `Architect:` or `Critic:` followed by a
/// name and `(VERDICT)` populate the corresponding entry.
pub fn parse_council_signature(body: &str) -> CouncilSignature {
    let mut in_block = false;
    let mut sig = CouncilSignature::default();

    for raw in body.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if is_council_header(line) {
            in_block = true;
            continue;
        }
        if line.starts_with("## ") && in_block {
            break;
        }
        if !in_block {
            continue;
        }

        let stripped = strip_bullet_prefix(line);
        let Some((role, tail)) = split_role(stripped) else {
            continue;
        };
        let Some((name, verdict)) = parse_verdict_tail(tail) else {
            continue;
        };
        let entry = CouncilEntry {
            name: name.to_owned(),
            verdict,
        };
        match role {
            Role::Architect => sig.architect = Some(entry),
            Role::Critic => sig.critic = Some(entry),
        }
    }
    sig
}

/// Run the gate: parse + validate. Returns `Ok(signature)` if both
/// roles are present and approved; otherwise the first reason for
/// failure.
pub fn evaluate(body: &str) -> Result<CouncilSignature, MergeGateError> {
    let sig = parse_council_signature(body);
    match (&sig.architect, &sig.critic) {
        (None, _) => Err(MergeGateError::MissingArchitect),
        (Some(_), None) => Err(MergeGateError::MissingCritic),
        (Some(a), Some(c)) => {
            if !a.verdict.is_approved() {
                Err(MergeGateError::ArchitectNotApproved { verdict: a.verdict })
            } else if !c.verdict.is_approved() {
                Err(MergeGateError::CriticNotApproved { verdict: c.verdict })
            } else {
                Ok(sig)
            }
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Role {
    Architect,
    Critic,
}

fn is_council_header(line: &str) -> bool {
    let normalized = line.trim_start_matches('#').trim().to_ascii_lowercase();
    normalized == "council signature" && line.starts_with("##") && !line.starts_with("###")
}

fn strip_bullet_prefix(line: &str) -> &str {
    line.trim_start()
        .strip_prefix("- ")
        .or_else(|| line.trim_start().strip_prefix("* "))
        .unwrap_or(line)
}

fn split_role(line: &str) -> Option<(Role, &str)> {
    let colon = line.find(':')?;
    let head = line[..colon].trim().to_ascii_lowercase();
    let tail = line[colon + 1..].trim();
    match head.as_str() {
        "architect" => Some((Role::Architect, tail)),
        "critic" => Some((Role::Critic, tail)),
        _ => None,
    }
}

fn parse_verdict_tail(tail: &str) -> Option<(&str, CouncilVerdict)> {
    let open = tail.rfind('(')?;
    let close = tail.rfind(')')?;
    if close < open + 1 {
        return None;
    }
    let verdict_token = tail[open + 1..close].trim();
    let verdict = CouncilVerdict::parse(verdict_token)?;
    let name = tail[..open].trim();
    if name.is_empty() {
        return None;
    }
    Some((name, verdict))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path_passes() {
        let body =
            "## Council Signature\nArchitect: alice (APPROVE)\nCritic: bob (APPROVE_WITH_NITS)\n";
        let r = evaluate(body).unwrap();
        assert!(r.is_approved());
    }

    #[test]
    fn missing_critic_errors() {
        let body = "## Council Signature\nArchitect: alice (APPROVE)\n";
        assert!(matches!(evaluate(body), Err(MergeGateError::MissingCritic)));
    }

    #[test]
    fn missing_architect_errors() {
        let body = "## Council Signature\nCritic: bob (APPROVE)\n";
        assert!(matches!(
            evaluate(body),
            Err(MergeGateError::MissingArchitect)
        ));
    }

    #[test]
    fn rejected_verdict_flagged() {
        let body = "## Council Signature\nArchitect: alice (APPROVE)\nCritic: bob (REJECT)\n";
        assert!(matches!(
            evaluate(body),
            Err(MergeGateError::CriticNotApproved {
                verdict: CouncilVerdict::Reject
            })
        ));
    }

    #[test]
    fn no_signature_block_errors() {
        let body = "## Summary\nstuff\n";
        assert!(matches!(
            evaluate(body),
            Err(MergeGateError::MissingArchitect)
        ));
    }

    #[test]
    fn block_terminates_at_next_heading() {
        let body =
            "## Council Signature\nArchitect: alice (APPROVE)\n\n## Other\nCritic: bob (APPROVE)\n";
        assert!(matches!(evaluate(body), Err(MergeGateError::MissingCritic)));
    }

    #[test]
    fn case_insensitive_header_and_verdict() {
        let body = "## council signature\narchitect: alice (approve)\ncritic: bob (approve)\n";
        assert!(evaluate(body).unwrap().is_approved());
    }

    #[test]
    fn bulleted_lines_accepted() {
        let body = "## Council Signature\n- Architect: alice (APPROVE)\n- Critic: bob (APPROVE_WITH_NITS)\n";
        assert!(evaluate(body).unwrap().is_approved());
    }

    #[test]
    fn defer_verdict_blocks_merge() {
        let body = "## Council Signature\nArchitect: alice (DEFER)\nCritic: bob (APPROVE)\n";
        assert!(matches!(
            evaluate(body),
            Err(MergeGateError::ArchitectNotApproved {
                verdict: CouncilVerdict::Defer
            })
        ));
    }

    #[test]
    fn nested_h3_council_header_not_recognized() {
        let body = "### Council Signature\nArchitect: alice (APPROVE)\nCritic: bob (APPROVE)\n";
        // H3 is not a top-level signature block; gate must fail.
        assert!(matches!(
            evaluate(body),
            Err(MergeGateError::MissingArchitect)
        ));
    }

    #[test]
    fn empty_name_invalidates_line() {
        let body = "## Council Signature\nArchitect: (APPROVE)\nCritic: bob (APPROVE)\n";
        // Empty name → role line is dropped → MissingArchitect.
        assert!(matches!(
            evaluate(body),
            Err(MergeGateError::MissingArchitect)
        ));
    }

    #[test]
    fn verdict_parse_round_trips_known_tokens() {
        for v in [
            CouncilVerdict::Approve,
            CouncilVerdict::ApproveWithNits,
            CouncilVerdict::Reject,
            CouncilVerdict::Defer,
        ] {
            assert_eq!(CouncilVerdict::parse(v.as_str()), Some(v));
        }
    }
}
