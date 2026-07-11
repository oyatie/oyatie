//! Sharded contract-slice policy loading.
//!
//! `contract-slice-policy.json` used to be the ONE file every new-slice PR edited to
//! append its entry: every landed slice PR re-dirtied every other open slice PR's
//! branch (a 100%-by-construction merge-conflict class — the same shape ADR-0538
//! fixed for workspace `Cargo.toml` membership). This module shards the mutable,
//! capability-owned fact (one slice = one committed file under `slices/`) and
//! GENERATES the aggregate from it, mirroring the repo's existing generated-face
//! doctrine (ADR-0539 `check_equals_fix`: a file is canonical iff its committed
//! bytes equal `canonicalize(bytes)`).
//!
//! Adding a slice is now "add one new file under `slices/`" — zero shared-file
//! edit. [`load_slice_fragments`] reads every fragment, fails closed (a keyed
//! [`Finding`], never a panic) on a non-JSON fragment or a slice_id collision
//! across fragments, and [`aggregate_policy`] rebuilds the exact policy document
//! `evaluate_configured` consumes, in deterministic (sorted-by-`slice_id`) order.

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::{Map, Value};

use crate::{Finding, GATE_ID, REQUIRED_PRIMARY_EXECUTION_PATH};

/// Fixed policy-wide metadata. Rarely changes, so it is a compile-time constant
/// rather than its own shared editable file — the recurring merge-conflict source
/// was the per-slice `slices` array, not this header.
const POLICY_PURPOSE: &str = "Paved-road owned-Rust/Buck2 gate that replaces the scripts/tests/*_check.py contract-slice validators. A worker adds a slice entry here plus a committed spec JSON; no new Python, shell, CLI, or crate. See README.md for the instantiation recipe.";
const POLICY_RELATED_ADRS: [&str; 3] = ["ADR-0515", "ADR-0523", "ADR-0528"];

/// The result of loading every fragment in `slices/`: the slices that parsed
/// cleanly (sorted by `slice_id`, ready for [`aggregate_policy`]) plus any
/// fail-closed structural [`Finding`]s. A non-empty `findings` set means at least
/// one fragment was excluded from `slices` — the load never panics or silently
/// drops a broken fragment.
#[derive(Debug, Clone, Default)]
pub struct FragmentLoad {
    pub slices: Vec<Value>,
    pub findings: BTreeSet<Finding>,
}

/// Load every `slices/*.json` fragment under `dir`. Fail-closed per fragment:
///
/// - a file whose contents do not parse as JSON, or that parses but has no
///   string `slice_id` field, yields `contract_slice_fragment_parse_error` (a
///   malformed fragment must never silently vanish from the policy);
/// - two fragments (any filenames) declaring the same `slice_id` both yield
///   `contract_slice_fragment_duplicate_slice_id` (a collision must never
///   silently pick one and drop the other).
///
/// A per-slice unknown-key typo is NOT re-validated here: once a fragment is
/// merged into the aggregate, [`crate::evaluate_configured`] already fails
/// closed on it (`contract_slice_unknown_policy_key`) — duplicating that check
/// at load time would just be two checks for one defect.
///
/// Non-`.json` files (e.g. a stray `README.md`) are ignored. Directory entries
/// are read in filename order (already `slice_id` order, since each fragment is
/// named `<slice_id>.json`) and the returned `slices` are additionally sorted by
/// `slice_id` so aggregation is deterministic regardless of filesystem
/// readdir order.
#[must_use]
pub fn load_slice_fragments(dir: &Path) -> FragmentLoad {
    let mut findings = BTreeSet::new();
    let mut by_id: std::collections::BTreeMap<String, Value> = std::collections::BTreeMap::new();
    // ids already found to collide across two-or-more fragments: excluded from
    // `by_id` entirely (fail closed — never silently admit either half of an
    // unresolved collision) and never re-admitted by a later fragment.
    let mut duplicate_ids: BTreeSet<String> = BTreeSet::new();

    let mut entries: Vec<_> = match std::fs::read_dir(dir) {
        Ok(read_dir) => read_dir.filter_map(Result::ok).collect(),
        Err(_) => {
            findings.insert(Finding {
                code: "contract_slice_fragment_parse_error".to_owned(),
                key: dir.display().to_string(),
            });
            return FragmentLoad {
                slices: Vec::new(),
                findings,
            };
        }
    };
    entries.sort_by_key(std::fs::DirEntry::path);

    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(std::ffi::OsStr::to_str) != Some("json") {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("<unnamed>")
            .to_owned();

        let parsed = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok());
        let Some(slice) = parsed else {
            findings.insert(Finding {
                code: "contract_slice_fragment_parse_error".to_owned(),
                key: file_name,
            });
            continue;
        };
        let slice_id = slice.get("slice_id").and_then(Value::as_str);
        let Some(slice_id) = slice_id else {
            findings.insert(Finding {
                code: "contract_slice_fragment_parse_error".to_owned(),
                key: file_name,
            });
            continue;
        };
        let slice_id = slice_id.to_owned();

        if duplicate_ids.contains(&slice_id) {
            findings.insert(Finding {
                code: "contract_slice_fragment_duplicate_slice_id".to_owned(),
                key: slice_id,
            });
            continue;
        }
        if by_id.remove(&slice_id).is_some() {
            // A second fragment declares an already-admitted slice_id: exclude
            // BOTH from the aggregate (never silently keep the first-seen one —
            // there is no principled way to pick a winner) and remember the id
            // so a third or later fragment with the same id is excluded too.
            findings.insert(Finding {
                code: "contract_slice_fragment_duplicate_slice_id".to_owned(),
                key: slice_id.clone(),
            });
            duplicate_ids.insert(slice_id);
            continue;
        }
        by_id.insert(slice_id, slice);
    }

    FragmentLoad {
        slices: by_id.into_values().collect(),
        findings,
    }
}

/// Rebuild the aggregate policy document from a clean [`FragmentLoad`]. Callers
/// should check `load.findings.is_empty()` first — this always returns a
/// document (using whatever slices did load) so a partial/fail-closed load never
/// panics, but a non-empty `findings` set means the aggregate is missing at
/// least one slice.
#[must_use]
pub fn aggregate_policy(load: &FragmentLoad) -> Value {
    let mut policy = Map::new();
    policy.insert("gate_id".to_owned(), Value::String(GATE_ID.to_owned()));
    policy.insert(
        "primary_execution_path".to_owned(),
        Value::String(REQUIRED_PRIMARY_EXECUTION_PATH.to_owned()),
    );
    policy.insert(
        "purpose".to_owned(),
        Value::String(POLICY_PURPOSE.to_owned()),
    );
    policy.insert(
        "related_adrs".to_owned(),
        Value::Array(
            POLICY_RELATED_ADRS
                .iter()
                .map(|adr| Value::String((*adr).to_owned()))
                .collect(),
        ),
    );
    policy.insert("slices".to_owned(), Value::Array(load.slices.clone()));
    Value::Object(policy)
}

/// Render a policy `Value` in this gate's canonical on-disk form: 2-space
/// indent, trailing newline. Shared by the materializer binary (which writes
/// `contract-slice-policy.json`) and the byte-parity test (which compares
/// against it), so the two can never disagree on formatting.
#[must_use]
pub fn render_policy_json(policy: &Value) -> String {
    match serde_json::to_string_pretty(policy) {
        Ok(mut rendered) => {
            rendered.push('\n');
            rendered
        }
        Err(_) => String::new(),
    }
}
