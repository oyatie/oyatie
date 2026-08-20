//! Receipt-stage classification for history-only retirement facts.

use std::collections::BTreeMap;

use super::{ControlPlaneEntry, TreeEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReceiptStage {
    Dormant,
    PreparedNew,
    ClosureNew,
    ClosedCarried,
}

impl ReceiptStage {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Dormant => "dormant",
            Self::PreparedNew => "prepared-new",
            Self::ClosureNew => "closure-new",
            Self::ClosedCarried => "closed-carried",
        }
    }
}

/// Keep receipt presence atomic across the fixed three-entry control plane while allowing
/// each scope to advance independently. Only the explicitly amended carry-forward shape is
/// heterogeneous: a closed carried scope may coexist with a newly prepared scope.
pub(crate) fn validate_receipt_population(stages: &[ReceiptStage]) -> Result<bool, String> {
    if stages.is_empty() {
        return Err("retirement control plane has no entries".to_owned());
    }

    let has_dormant = stages.contains(&ReceiptStage::Dormant);
    if has_dormant {
        return if stages.iter().all(|stage| *stage == ReceiptStage::Dormant) {
            Ok(false)
        } else {
            Err("retirement receipt population is not atomic across all three scopes".to_owned())
        };
    }

    if stages.iter().all(|stage| *stage == stages[0]) {
        return Ok(true);
    }

    if stages.iter().all(|stage| {
        matches!(
            stage,
            ReceiptStage::ClosedCarried | ReceiptStage::PreparedNew
        )
    }) {
        return Ok(true);
    }

    Err("retirement receipt lifecycle mix is not an admitted atomic population".to_owned())
}

pub(crate) fn classify_stage(
    entry: &ControlPlaneEntry,
    protected: &BTreeMap<String, TreeEntry>,
    candidate: &BTreeMap<String, TreeEntry>,
) -> Result<ReceiptStage, String> {
    let pp = protected.contains_key(&entry.preparation_receipt_path);
    let pc = protected.contains_key(&entry.closure_receipt_path);
    let cp = candidate.contains_key(&entry.preparation_receipt_path);
    let cc = candidate.contains_key(&entry.closure_receipt_path);
    match (pp, pc, cp, cc) {
        (false, false, false, false) => Ok(ReceiptStage::Dormant),
        (false, false, true, false) => Ok(ReceiptStage::PreparedNew),
        (true, false, false, true) => Ok(ReceiptStage::ClosureNew),
        (false, true, false, true) => {
            let protected_entry = protected
                .get(&entry.closure_receipt_path)
                .expect("presence checked");
            let candidate_entry = candidate
                .get(&entry.closure_receipt_path)
                .expect("presence checked");
            if protected_entry.oid != candidate_entry.oid
                || protected_entry.mode != candidate_entry.mode
                || protected_entry.kind != candidate_entry.kind
            {
                return Err(format!(
                    "carried closure {} changed",
                    entry.closure_receipt_path
                ));
            }
            Ok(ReceiptStage::ClosedCarried)
        }
        _ => Err(format!(
            "invalid retirement receipt lifecycle for {}",
            entry.scope_ref
        )),
    }
}

pub(crate) fn receipt_for_stage<'a>(
    stage: ReceiptStage,
    control: &'a ControlPlaneEntry,
    _protected: &'a BTreeMap<String, TreeEntry>,
    candidate: &'a BTreeMap<String, TreeEntry>,
) -> Result<(&'a str, &'a TreeEntry), String> {
    let path = match stage {
        ReceiptStage::PreparedNew => control.preparation_receipt_path.as_str(),
        ReceiptStage::ClosureNew | ReceiptStage::ClosedCarried => {
            control.closure_receipt_path.as_str()
        }
        ReceiptStage::Dormant => return Err("dormant stage has no receipt".to_owned()),
    };
    candidate
        .get(path)
        .map(|entry| (path, entry))
        .ok_or_else(|| format!("candidate receipt {path} is absent"))
}
