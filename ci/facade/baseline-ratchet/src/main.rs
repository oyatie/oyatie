//! # oya-cloud-ci-firewall-signoff-fixer (automation-default surface)
//!
//! Founder directive 2026-06-12: "flagging, red gating isn't enough — automate everything
//! that can be automated canonically, universally." An INERT sign-off door entry
//! (FRIC-1781280001; symmetrized FRIC-1781460000) is PROVABLY dead — its key is absent from
//! the CANDIDATE tree's live current/proposed sets (the merge-base frozen face does not keep
//! an orphaned entry alive), so the retirement is mechanically derivable.
//! This fixer derives and applies it: it removes the inert keys from
//! `_sign_off_additions` and appends the grouped audit records to
//! `_sign_off_retirements`. The firewall gate's inert-door RED remains the enforcement
//! BACKSTOP; its failure detail prints exactly this command
//! ([`ci_baseline_ratchet::SIGNOFF_FIXER_COMMAND`]).
//!
//! CANONICAL: the door file is parsed by the one existing parser
//! ([`SignOff::from_value`]) and the inert set by the one existing detector
//! ([`inert_signoff_entries`]) — no duplicated predicate. UNIVERSAL: every repo-specific
//! path is a lib constant consumed by gate test and fixer alike; the face path comes from
//! the committed ratchet policy (DATA).
//!
//! SELF-VALIDATION (refuse-on-failure, like every fixer): before writing, the rewritten
//! text is reparsed and checked — the surviving entry set must equal (before \ inert),
//! the recomputed inert set must be empty, and the retirement audit records must carry
//! every retired key. Any violation aborts without touching the file.
//!
//! Run on a SETTLED tree (committed face == regenerated face): the fixer reads the
//! COMMITTED face as current/proposed. A mis-derivation on an unsettled tree fails toward
//! RED at the gate (a wrongly-retired live admission resurfaces as growth), never toward
//! laundering.
//!
//! Usage:
//!   oya-cloud-ci-firewall-signoff-fixer [--repo-root <path>] [--fix]
//!
//! Without `--fix` it reports the derived retirements and exits non-zero if any exist
//! (check mode); with `--fix` it applies them.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use ci_baseline_ratchet::{
    Baseline, FROZEN_SNAPSHOT_PATH, FrozenBaseline, RATCHET_POLICY_PATH, SIGNOFF_FIXER_COMMAND,
    SIGNOFF_PATH, SignOff, baseline_keys_map, inert_signoff_entries,
};
use serde_json::{Value, json};

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("oya-cloud-ci-firewall-signoff-fixer: {error}");
            std::process::exit(2);
        }
    }
}

fn run() -> Result<i32, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut repo_root: Option<PathBuf> = None;
    let mut fix = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repo-root" => {
                i += 1;
                repo_root = args.get(i).map(PathBuf::from);
            }
            "--fix" => fix = true,
            other => return Err(format!("unknown argument {other}")),
        }
        i += 1;
    }
    let root = match repo_root {
        Some(root) => root,
        None => discover_repo_root()?,
    };

    // The FROZEN reference (fail-closed exactly like the gate: a missing snapshot names
    // the materialization remediation, never a silent empty reference).
    let snapshot_path = root.join(FROZEN_SNAPSHOT_PATH);
    let snapshot_text = std::fs::read_to_string(&snapshot_path).map_err(|e| {
        format!(
            "merge-base frozen baseline snapshot missing at {} ({e}) — materialize it: \
             buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .",
            snapshot_path.display()
        )
    })?;
    let snapshot: Value = serde_json::from_str(&snapshot_text)
        .map_err(|e| format!("parse {}: {e}", snapshot_path.display()))?;
    let frozen = FrozenBaseline::from_value(&snapshot)
        .map_err(|e| format!("invalid frozen baseline snapshot: {e}"))?;

    // The committed face (== proposed == current on a settled tree). The face path is
    // DATA in the committed ratchet policy.
    let policy = read_json(&root.join(RATCHET_POLICY_PATH))?;
    let face_path = policy
        .get("frozen_reference")
        .and_then(|f| f.get("face_path"))
        .and_then(Value::as_str)
        .ok_or("ratchet-policy.json missing frozen_reference.face_path")?;
    let face = read_json(&root.join(face_path))?;
    let proposed = Baseline::from_value(&face)
        .map_err(|e| format!("invalid committed baseline face {face_path}: {e}"))?;
    let current = baseline_keys_map(&proposed);

    // The door file, via the one parser.
    let door_path = root.join(SIGNOFF_PATH);
    let door_text =
        std::fs::read_to_string(&door_path).map_err(|e| format!("{}: {e}", door_path.display()))?;
    let door: Value = serde_json::from_str(&door_text)
        .map_err(|e| format!("parse {}: {e}", door_path.display()))?;
    let signoff = SignOff::from_value(&door);

    let inert = inert_signoff_entries(&frozen.baseline, &proposed, &current, &signoff);
    if inert.is_empty() {
        println!(
            "oya-cloud-ci-firewall-signoff-fixer: door clean — every sign-off entry \
             exempts a key the candidate still carries (current or proposed; merge-base \
             frozen face @ {} does not keep an orphaned entry alive)",
            frozen.merge_base
        );
        return Ok(0);
    }

    println!(
        "oya-cloud-ci-firewall-signoff-fixer: {} INERT door entr{} (key absent from the \
         CANDIDATE tree — current AND proposed — so the door admits nothing in this change; \
         a standing re-introduction ticket regardless of the merge-base frozen face @ {}):",
        inert.len(),
        if inert.len() == 1 { "y" } else { "ies" },
        frozen.merge_base
    );
    for (gate, code, key) in &inert {
        println!("  - {gate}/{code}: {key}");
    }
    if !fix {
        println!("derived retirement NOT applied (check mode). Apply: {SIGNOFF_FIXER_COMMAND}");
        return Ok(1);
    }

    let date = utc_date_today()?;
    let fixed = apply_retirements(
        &door,
        &inert,
        &date,
        "oya-cloud-ci-firewall-signoff-fixer --fix (mechanical inert-entry retirement, FRIC-1781280001)",
    )?;
    // The door file is human-audited JSON: pretty + trailing newline (the faces'
    // serialization contract; key order is preserved by the workspace serde_json).
    let fixed_text = serde_json::to_string_pretty(&fixed)
        .map_err(|e| format!("serialize door file: {e}"))?
        + "\n";

    // SELF-VALIDATION: reparse the bytes we are about to write and refuse on any drift.
    validate_retirement(
        &door,
        &fixed_text,
        &inert,
        &frozen.baseline,
        &proposed,
        &current,
    )?;

    std::fs::write(&door_path, &fixed_text).map_err(|e| format!("{}: {e}", door_path.display()))?;
    println!(
        "oya-cloud-ci-firewall-signoff-fixer: retired {} entr{} into _sign_off_retirements \
         ({}) — re-run the firewall gate to confirm GREEN",
        inert.len(),
        if inert.len() == 1 { "y" } else { "ies" },
        door_path.display()
    );
    Ok(0)
}

fn read_json(path: &std::path::Path) -> Result<Value, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
}

/// Walk up from cwd to the repo root (the dir holding the root hub pointer file).
fn discover_repo_root() -> Result<PathBuf, String> {
    let mut dir = std::env::current_dir().map_err(|e| e.to_string())?;
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    Err(
        "failed to locate repo root (no specs/root-hub-pointers.json up-tree); pass --repo-root"
            .to_owned(),
    )
}

/// PURE: remove every inert `(gate, code, key)` from `_sign_off_additions` (dropping
/// emptied arrays/code maps/gate maps, but keeping `_sign_off_additions` itself — EMPTY
/// means the ratchet is fully closed) and append one grouped audit record per
/// `(gate, code)` to `_sign_off_retirements`.
fn apply_retirements(
    door: &Value,
    inert: &[(String, String, String)],
    date: &str,
    retired_by: &str,
) -> Result<Value, String> {
    let mut fixed = door.clone();

    let additions = fixed
        .get_mut("_sign_off_additions")
        .and_then(Value::as_object_mut)
        .ok_or("door file missing _sign_off_additions object")?;
    for (gate, code, key) in inert {
        let keys = additions
            .get_mut(gate)
            .and_then(Value::as_object_mut)
            .and_then(|codes| codes.get_mut(code))
            .and_then(Value::as_array_mut)
            .ok_or_else(|| format!("door file missing entry {gate}/{code}/{key}"))?;
        let before_len = keys.len();
        keys.retain(|k| k.as_str() != Some(key));
        if keys.len() == before_len {
            return Err(format!("door file entry {gate}/{code}/{key} not found"));
        }
    }
    // Drop emptied code arrays, then emptied gate maps.
    for codes in additions.values_mut() {
        if let Some(code_map) = codes.as_object_mut() {
            code_map.retain(|_, keys| keys.as_array().is_some_and(|a| !a.is_empty()));
        }
    }
    additions.retain(|_, codes| codes.as_object().is_some_and(|m| !m.is_empty()));

    // Grouped audit records, deterministic order (BTreeSet over (gate, code) pairs).
    let groups: BTreeSet<(&String, &String)> =
        inert.iter().map(|(gate, code, _)| (gate, code)).collect();
    let mut records: Vec<Value> = Vec::new();
    for (gate, code) in groups {
        let keys: Vec<&String> = inert
            .iter()
            .filter(|(g, c, _)| g == gate && c == code)
            .map(|(_, _, k)| k)
            .collect();
        records.push(json!({
            "date": date,
            "retired_by": retired_by,
            "rationale": "Mechanically derived: entry key absent from the CANDIDATE tree (current AND proposed) — the door admits nothing in the change under evaluation, so the exemption protects nothing and is a standing re-introduction ticket. Inert-ness is read against the candidate, not the merge-base frozen face, so PR-admission and push-admission agree (FRIC-1781280001 inert-door detector; ADR-0551 hardening; FRIC-1781460000 PR/push symmetry).",
            "gate": gate,
            "code": code,
            "keys": keys,
        }));
    }
    match fixed.get_mut("_sign_off_retirements") {
        Some(Value::Array(existing)) => existing.extend(records),
        Some(_) => return Err("door file _sign_off_retirements is not an array".to_owned()),
        None => {
            if let Some(obj) = fixed.as_object_mut() {
                obj.insert("_sign_off_retirements".to_owned(), Value::Array(records));
            } else {
                return Err("door file root is not an object".to_owned());
            }
        }
    }
    Ok(fixed)
}

/// PURE: the refuse-on-failure check over the EXACT bytes to be written.
fn validate_retirement(
    before: &Value,
    after_text: &str,
    inert: &[(String, String, String)],
    frozen: &Baseline,
    proposed: &Baseline,
    current: &std::collections::BTreeMap<
        String,
        std::collections::BTreeMap<String, BTreeSet<String>>,
    >,
) -> Result<(), String> {
    let after: Value = serde_json::from_str(after_text)
        .map_err(|e| format!("self-validation: rewritten door file does not parse: {e}"))?;
    let before_entries: BTreeSet<(String, String, String)> =
        SignOff::from_value(before).entries().into_iter().collect();
    let after_signoff = SignOff::from_value(&after);
    let after_entries: BTreeSet<(String, String, String)> =
        after_signoff.entries().into_iter().collect();
    let inert_set: BTreeSet<(String, String, String)> = inert.iter().cloned().collect();

    let expected: BTreeSet<(String, String, String)> =
        before_entries.difference(&inert_set).cloned().collect();
    if after_entries != expected {
        return Err(format!(
            "self-validation: surviving entry set mismatch — expected (before \\ inert), \
             got unexpected delta: missing {:?}, extra {:?} — REFUSING to write",
            expected.difference(&after_entries).collect::<Vec<_>>(),
            after_entries.difference(&expected).collect::<Vec<_>>(),
        ));
    }
    let residual = inert_signoff_entries(frozen, proposed, current, &after_signoff);
    if !residual.is_empty() {
        return Err(format!(
            "self-validation: inert entries remain after the fix: {residual:?} — REFUSING \
             to write"
        ));
    }
    // Every retired key must be carried by some retirement audit record.
    let recorded: BTreeSet<(String, String, String)> = after
        .get("_sign_off_retirements")
        .and_then(Value::as_array)
        .map(|records| {
            records
                .iter()
                .flat_map(|r| {
                    let gate = r.get("gate").and_then(Value::as_str).unwrap_or("");
                    let code = r.get("code").and_then(Value::as_str).unwrap_or("");
                    r.get("keys")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(|k| (gate.to_owned(), code.to_owned(), k.to_owned()))
                        .collect::<Vec<_>>()
                })
                .collect()
        })
        .unwrap_or_default();
    if !inert_set.is_subset(&recorded) {
        return Err(format!(
            "self-validation: retirement audit records missing keys {:?} — REFUSING to write",
            inert_set.difference(&recorded).collect::<Vec<_>>()
        ));
    }
    Ok(())
}

/// Today's UTC date as `YYYY-MM-DD` (no external deps; Howard Hinnant's civil-from-days).
fn utc_date_today() -> Result<String, String> {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("system clock before epoch: {e}"))?
        .as_secs();
    let days = i64::try_from(secs / 86_400).map_err(|e| format!("epoch days overflow: {e}"))?;
    Ok(civil_from_days(days))
}

/// Days-since-epoch -> `YYYY-MM-DD` (proleptic Gregorian; H. Hinnant, `civil_from_days`).
fn civil_from_days(z: i64) -> String {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn frozen() -> Baseline {
        Baseline::from_value(&json!({
            "gates": {"g": {"c": {"mode": "baseline-block-on-new", "keys": ["frozen.rs"]}}}
        }))
        .unwrap()
    }

    fn proposed_with_live_admission() -> Baseline {
        Baseline::from_value(&json!({
            "gates": {"g": {"c": {"mode": "baseline-block-on-new",
                                   "keys": ["frozen.rs", "live-admitted.rs"]}}}
        }))
        .unwrap()
    }

    fn door() -> Value {
        json!({
            "_comment": "door",
            "_sign_off_additions": {
                "g": {"c": ["live-admitted.rs", "long-gone.rs"]}
            },
            "_sign_off_retirements": []
        })
    }

    #[test]
    fn derives_only_the_inert_entry_and_applies_a_validated_retirement() {
        let frozen = frozen();
        let proposed = proposed_with_live_admission();
        let current = baseline_keys_map(&proposed);
        let door = door();
        let signoff = SignOff::from_value(&door);

        // The one existing detector derives exactly the standing ticket; the in-flight
        // live admission is untouched.
        let inert = inert_signoff_entries(&frozen, &proposed, &current, &signoff);
        assert_eq!(
            inert,
            vec![("g".to_owned(), "c".to_owned(), "long-gone.rs".to_owned())]
        );

        let fixed = apply_retirements(&door, &inert, "2026-06-12", "fixer-test").unwrap();
        let text = serde_json::to_string_pretty(&fixed).unwrap() + "\n";
        validate_retirement(&door, &text, &inert, &frozen, &proposed, &current).unwrap();

        assert_eq!(
            fixed["_sign_off_additions"]["g"]["c"],
            json!(["live-admitted.rs"]),
            "the live admission survives; the inert key is gone"
        );
        let record = &fixed["_sign_off_retirements"][0];
        assert_eq!(record["date"], "2026-06-12");
        assert_eq!(record["gate"], "g");
        assert_eq!(record["code"], "c");
        assert_eq!(record["keys"], json!(["long-gone.rs"]));
    }

    #[test]
    fn retiring_the_last_key_drops_the_emptied_code_and_gate_maps() {
        let frozen = frozen();
        let proposed = frozen.clone();
        let current = baseline_keys_map(&proposed);
        let door = json!({
            "_sign_off_additions": {"g": {"c": ["long-gone.rs"]}, "g2": {"c2": ["frozen.rs"]}}
        });
        // g2/c2/frozen.rs — the key is in the frozen face under g/c, but the inert lookup is
        // (gate, code)-scoped and reads the CANDIDATE tree: g2/c2 carries no current/proposed
        // key, so the entry admits nothing and BOTH entries are inert here.
        let signoff = SignOff::from_value(&door);
        let inert = inert_signoff_entries(&frozen, &proposed, &current, &signoff);
        assert_eq!(inert.len(), 2, "per-(gate,code) scoping: {inert:?}");

        let fixed = apply_retirements(&door, &inert, "2026-06-12", "fixer-test").unwrap();
        assert_eq!(
            fixed["_sign_off_additions"],
            json!({}),
            "emptied code arrays and gate maps are dropped; the additions object survives"
        );
        assert_eq!(
            fixed["_sign_off_retirements"].as_array().unwrap().len(),
            2,
            "one grouped audit record per (gate, code)"
        );
        let text = serde_json::to_string_pretty(&fixed).unwrap() + "\n";
        validate_retirement(&door, &text, &inert, &frozen, &proposed, &current).unwrap();
    }

    #[test]
    fn validation_refuses_a_rewrite_that_loses_a_live_entry() {
        let frozen = frozen();
        let proposed = proposed_with_live_admission();
        let current = baseline_keys_map(&proposed);
        let door = door();
        let inert = vec![("g".to_owned(), "c".to_owned(), "long-gone.rs".to_owned())];

        // A corrupted rewrite that ALSO dropped the live admission must be refused.
        let corrupted = json!({
            "_sign_off_additions": {},
            "_sign_off_retirements": [{
                "date": "2026-06-12", "retired_by": "x", "rationale": "x",
                "gate": "g", "code": "c", "keys": ["long-gone.rs"]
            }]
        });
        let text = serde_json::to_string_pretty(&corrupted).unwrap();
        let err =
            validate_retirement(&door, &text, &inert, &frozen, &proposed, &current).unwrap_err();
        assert!(err.contains("REFUSING"), "{err}");
    }

    #[test]
    fn validation_refuses_a_rewrite_with_missing_audit_records() {
        let frozen = frozen();
        let proposed = proposed_with_live_admission();
        let current = baseline_keys_map(&proposed);
        let door = door();
        let inert = vec![("g".to_owned(), "c".to_owned(), "long-gone.rs".to_owned())];

        // Entry removed but no audit record appended: refused.
        let unaudited = json!({
            "_sign_off_additions": {"g": {"c": ["live-admitted.rs"]}},
            "_sign_off_retirements": []
        });
        let text = serde_json::to_string_pretty(&unaudited).unwrap();
        let err =
            validate_retirement(&door, &text, &inert, &frozen, &proposed, &current).unwrap_err();
        assert!(err.contains("audit records missing"), "{err}");
    }

    #[test]
    fn fixing_a_clean_door_is_a_no_op_shape() {
        // apply_retirements with an entry that is not present must refuse (fail-closed,
        // never silently "succeed" on a phantom retirement).
        let door = json!({"_sign_off_additions": {}});
        let inert = vec![("g".to_owned(), "c".to_owned(), "ghost.rs".to_owned())];
        assert!(apply_retirements(&door, &inert, "2026-06-12", "fixer-test").is_err());
    }

    #[test]
    fn civil_from_days_matches_known_dates() {
        assert_eq!(civil_from_days(0), "1970-01-01");
        assert_eq!(civil_from_days(19_723), "2024-01-01");
        // 2026-06-12 = 20_616 days after the epoch.
        assert_eq!(civil_from_days(20_616), "2026-06-12");
        let _ = utc_date_today().unwrap();
    }

    #[test]
    fn current_map_type_is_the_lib_contract() {
        // Compile-time pin: the fixer feeds the detector the same map shape the gate uses.
        let proposed = proposed_with_live_admission();
        let current: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> =
            baseline_keys_map(&proposed);
        assert!(current.contains_key("g"));
    }
}
