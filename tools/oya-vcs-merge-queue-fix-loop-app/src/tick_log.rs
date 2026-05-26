//! Convergence-proof tick-log rendering (IP-006 §"Per-tick evidence").
//!
//! Renders [`TickEntry`] values from the kernel into the JSON envelope
//! used at `registry/merge-queue-tick-log.json`. Kept
//! out of the kernel because the kernel is pure and IO-free; this is
//! the serialization adapter.

use oya_vcs_review_mergequeue_kernel::parked_state::ParkedReason;
use oya_vcs_review_mergequeue_kernel::scheduler::{TickAction, TickEntry};
use oya_vcs_review_mergequeue_kernel::speculative_rebase::RebaseDecision;

/// Single line of the tick log; one per scheduler tick.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TickLogEntry<'a> {
    pub entry: &'a TickEntry,
}

/// Serialize the full registry envelope around an iterable of TickEntry.
pub fn render_tick_log_registry<'a, I>(meta_block: &str, entries: I) -> String
where
    I: IntoIterator<Item = &'a TickEntry>,
{
    let body = entries
        .into_iter()
        .map(render_one)
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"_meta\":{meta_block},\"entries\":[{body}]}}")
}

fn render_one(entry: &TickEntry) -> String {
    let action_json = render_action(&entry.action);
    format!(
        "{{\"action\":{action},\"current_head_sha\":{head},\"epoch\":{epoch},\"parked_count\":{pc},\"queue_depth\":{qd},\"tick_number\":{tn}}}",
        action = action_json,
        head = json_string(&entry.current_head_sha),
        epoch = entry.epoch,
        pc = entry.parked_count,
        qd = entry.queue_depth,
        tn = entry.tick_number,
    )
}

fn render_action(action: &TickAction) -> String {
    match action {
        TickAction::AdmitPr {
            pr_number,
            changeset_id,
        } => format!(
            "{{\"changeset_id\":{cs},\"kind\":\"admit\",\"pr_number\":{pr}}}",
            cs = json_string(changeset_id),
            pr = pr_number
        ),
        TickAction::MergePr { pr_number } => {
            format!("{{\"kind\":\"merge\",\"pr_number\":{pr}}}", pr = pr_number)
        }
        TickAction::ParkPr { pr_number, reason } => format!(
            "{{\"kind\":\"park\",\"pr_number\":{pr},\"reason\":{r}}}",
            pr = pr_number,
            r = json_string(reason_wire(reason))
        ),
        TickAction::RevalidateParkedPr {
            pr_number,
            rebase_decision,
            attempts_used,
        } => format!(
            "{{\"attempts_used\":{used},\"kind\":\"revalidate\",\"pr_number\":{pr},\"rebase\":{rd}}}",
            used = attempts_used,
            pr = pr_number,
            rd = render_rebase(rebase_decision),
        ),
        TickAction::EvictPr {
            pr_number,
            attempts_used,
        } => format!(
            "{{\"attempts_used\":{used},\"kind\":\"evict\",\"pr_number\":{pr}}}",
            used = attempts_used,
            pr = pr_number
        ),
        TickAction::Idle => "{\"kind\":\"idle\"}".to_string(),
    }
}

fn render_rebase(rebase: &RebaseDecision) -> String {
    match rebase {
        RebaseDecision::FastPath {
            pr_number,
            base_sha,
            new_head_sha,
        } => format!(
            "{{\"base_sha\":{base},\"kind\":\"fast-path\",\"new_head_sha\":{head},\"pr_number\":{pr}}}",
            base = json_string(base_sha),
            head = json_string(new_head_sha),
            pr = pr_number
        ),
        RebaseDecision::Reproject {
            pr_number,
            new_base_sha,
            new_head_sha,
            skipped_generations,
        } => format!(
            "{{\"kind\":\"reproject\",\"new_base_sha\":{base},\"new_head_sha\":{head},\"pr_number\":{pr},\"skipped_generations\":{sg}}}",
            base = json_string(new_base_sha),
            head = json_string(new_head_sha),
            pr = pr_number,
            sg = skipped_generations
        ),
        RebaseDecision::NoOp { pr_number } => {
            format!("{{\"kind\":\"no-op\",\"pr_number\":{pr}}}", pr = pr_number)
        }
    }
}

fn reason_wire(reason: &ParkedReason) -> &'static str {
    reason.as_wire()
}

fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if (ch as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", ch as u32));
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_vcs_review_mergequeue_kernel::scheduler::{TickAction, TickEntry};

    #[test]
    fn renders_idle_tick() {
        let entry = TickEntry {
            tick_number: 1,
            action: TickAction::Idle,
            current_head_sha: "1".repeat(40),
            epoch: 100,
            queue_depth: 0,
            parked_count: 0,
        };
        let out = render_tick_log_registry("{}", std::iter::once(&entry));
        assert!(out.contains("\"kind\":\"idle\""));
        assert!(out.contains("\"tick_number\":1"));
    }

    #[test]
    fn renders_merge_and_park_actions() {
        let merge = TickEntry {
            tick_number: 1,
            action: TickAction::MergePr { pr_number: 7 },
            current_head_sha: "1".repeat(40),
            epoch: 100,
            queue_depth: 2,
            parked_count: 0,
        };
        let park = TickEntry {
            tick_number: 2,
            action: TickAction::ParkPr {
                pr_number: 8,
                reason: ParkedReason::CiFailure,
            },
            current_head_sha: "2".repeat(40),
            epoch: 101,
            queue_depth: 2,
            parked_count: 1,
        };
        let out = render_tick_log_registry("{}", [&merge, &park]);
        assert!(out.contains("\"kind\":\"merge\""));
        assert!(out.contains("\"kind\":\"park\""));
        assert!(out.contains("\"reason\":\"ci-failure\""));
    }

    #[test]
    fn renders_revalidate_with_reproject_rebase() {
        let entry = TickEntry {
            tick_number: 5,
            action: TickAction::RevalidateParkedPr {
                pr_number: 42,
                rebase_decision: RebaseDecision::Reproject {
                    pr_number: 42,
                    new_base_sha: "a".repeat(40),
                    new_head_sha: "b".repeat(40),
                    skipped_generations: 2,
                },
                attempts_used: 3,
            },
            current_head_sha: "a".repeat(40),
            epoch: 200,
            queue_depth: 1,
            parked_count: 1,
        };
        let out = render_tick_log_registry("{}", std::iter::once(&entry));
        assert!(out.contains("\"kind\":\"revalidate\""));
        assert!(out.contains("\"kind\":\"reproject\""));
        assert!(out.contains("\"skipped_generations\":2"));
    }
}
