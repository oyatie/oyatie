//! SCM event-identity and receipt-identity checks.

use serde_json::Value;

use super::{ControlPlaneEntry, ControlSelector, ReceiptStage, RetirementObjectSource};

/// Validate the controller-provided immutable SCM tuple before any policy may select a revision.
/// This deliberately accepts no ambient environment and no caller-selected candidate.
pub(crate) fn validate_event_identity(
    source: &impl RetirementObjectSource,
    event: &str,
    event_ref: &str,
    event_base_ref: &str,
    protected: &str,
    evaluated: &str,
    subject: &str,
) -> Result<(), String> {
    match event {
        "pull_request" => {
            if event_base_ref != "dev" {
                return Err("pull_request protected base ref must be dev".to_owned());
            }
            if !event_ref.starts_with("refs/pull/") {
                return Err("pull_request event ref must be a pull request ref".to_owned());
            }
            let parents = source.parents(evaluated)?;
            if parents != [protected.to_owned(), subject.to_owned()] {
                return Err("pull_request evaluated commit parents must be exactly [protected base, subject]".to_owned());
            }
            if subject == evaluated {
                return Err("pull_request subject must not equal evaluated merge commit".to_owned());
            }
        }
        "push" => {
            if event_base_ref != "refs/heads/dev" {
                return Err("push protected base ref must be refs/heads/dev".to_owned());
            }
            if event_ref != "refs/heads/dev" {
                return Err("push event ref must be refs/heads/dev".to_owned());
            }
            if subject != evaluated {
                return Err("push subject must equal evaluated commit".to_owned());
            }
            if source.parents(evaluated)? != [protected.to_owned()] {
                return Err(
                    "push evaluated commit parents must be exactly [protected base]".to_owned(),
                );
            }
        }
        "merge_group" => {
            if event_base_ref != "refs/heads/dev" {
                return Err("merge_group protected base ref must be refs/heads/dev".to_owned());
            }
            if !event_ref.starts_with("refs/heads/gh-readonly-queue/dev/") {
                return Err(
                    "merge_group event ref must be refs/heads/gh-readonly-queue/dev/...".to_owned(),
                );
            }
            if subject != evaluated {
                return Err("merge_group subject must equal evaluated commit".to_owned());
            }
        }
        _ => {
            return Err(
                "retirement SCM event must be pull_request, push, or merge_group".to_owned(),
            );
        }
    }
    Ok(())
}

pub(crate) fn selector_matches_path(selector: &ControlSelector, path: &str) -> bool {
    match selector.selector_type.as_str() {
        "exact" => selector.selector == path,
        "glob" => selector.selector.strip_suffix("/**").is_some_and(|prefix| {
            path.starts_with(prefix) && path.as_bytes().get(prefix.len()) == Some(&b'/')
        }),
        _ => false,
    }
}

pub(crate) fn validate_receipt_identity(
    stage: ReceiptStage,
    control: &ControlPlaneEntry,
    receipt_path: &str,
    receipt: &Value,
) -> Result<(), String> {
    let expected_id = match stage {
        ReceiptStage::PreparedNew => &control.preparation_artifact_id,
        ReceiptStage::ClosureNew | ReceiptStage::ClosedCarried => &control.closure_artifact_id,
        ReceiptStage::Dormant => return Err("dormant receipt identity".to_owned()),
    };
    if receipt.get("artifact_id").and_then(Value::as_str) != Some(expected_id)
        || receipt.get("scope_ref").and_then(Value::as_str) != Some(&control.scope_ref)
        || receipt
            .get("authority")
            .and_then(|authority| authority.get("planning_state"))
            .and_then(Value::as_str)
            != Some("HOLD(Planning)")
        || receipt
            .get("authority")
            .and_then(|authority| authority.get("dispatch_authorized"))
            .and_then(Value::as_bool)
            != Some(false)
    {
        return Err(format!(
            "receipt {receipt_path} is not bound to its control-plane identity"
        ));
    }
    if receipt.get("promoted_commit_oid").is_some()
        || receipt.get("postmerge_success").is_some()
        || receipt.get("verdict").is_some()
        || receipt.get("pass").is_some()
    {
        return Err(format!(
            "receipt {receipt_path} exceeds the E7 claim ceiling"
        ));
    }
    Ok(())
}
