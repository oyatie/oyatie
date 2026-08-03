// ADR-0560 cache-wiring conformance gate: live-corpus self-test over the REAL
// policy + license + overlays + canary workflow (the slice-1 instalment of the
// ADR-0556-named cache-policy-conformance successor; asserting the FULL live CI
// cache configuration against the policy remains that gate's scope).
//
// Proves mechanically, on every PR:
//   1. the dark-wiring guarantee — while specs/cache-warm-license.json is
//      unlicensed, EVERY class resolves bypass (today's builds are untouched);
//   2. the cold-required floor — the four ADR-0556 one-way cold classes resolve
//      bypass even under a licensed fixture (pinned here as a ratchet: dropping
//      one from the policy DATA goes RED and requires superseding ADR-0556);
//   3. the kill-switch works — flipping the license fixture flips warm classes
//      between bypass and their classified modes, and never the cold ones;
//   4. the overlays parse, select the cache execution platform, set the posture
//      their name claims, and carry NO keyed identity material;
//   5. the root .buckconfig stays clean of any RE/cache section;
//   6. the canary workflow exists, is scheduled, restores no actions/cache, and
//      wires the cold proof (assert-cold) + structured record;
//   7. the canary's WARM probe proves participation, not just byte-equality — it
//      writes its own invocation record and hands it to assert-warm under the class
//      and mode it ran. Byte-equality alone is satisfied by a probe that served
//      nothing and rebuilt locally, so without (7) the canary can emit GREEN having
//      proven nothing — and that GREEN is the single artifact that licenses warm
//      reads fleet-wide (ADR-0556 D2).
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use ci_build_cache_policy as app;
use serde_json::{Value, json};

const CANARY_WORKFLOW_PATH: &str = ".github/workflows/cache-integrity-canary.yml";
/// The warm probe's own invocation record. Pinned so `assert-warm` cannot be pointed
/// at the COLD record (or any stale file) and still satisfy this gate.
const WARM_RECORD_PATH: &str = "/tmp/canary-warm-record.json";
const REQUIRED_WORKFLOW_PATH: &str = ".github/workflows/oya-ci-required.yml";
const COLD_REQUIRED_FLOOR: [&str; 4] = [
    "release-production-image",
    "integrity-canary",
    "untrusted-author-presubmit",
    "provenance-attestation",
];

fn repo_root() -> PathBuf {
    let cwd = std::env::current_dir().expect("current_dir");
    app::repo_root_from(&cwd).expect("failed to locate repo root from test current_dir")
}

fn licensed_fixture() -> Value {
    json!({ "warm_reads_licensed": true, "reason": "conformance fixture", "licensed_by_canary_run": "fixture" })
}

fn invocation_record_fixture(
    cache_hit_rate: f64,
    action_hits: u64,
    local: u64,
    remote: u64,
) -> Value {
    json!({
        "cache_hit_rate": cache_hit_rate,
        "run_action_cache_count": action_hits,
        "run_local_count": local,
        "run_remote_count": remote,
        "run_skipped_count": 0,
        "cache_upload_attempt_count": 0,
        "cache_upload_count": 0,
        "exit_result_name": "SUCCESS",
        "last_snapshot": { "re_action_cache_started": action_hits },
    })
}

#[test]
fn policy_and_license_parse_and_the_default_is_fail_closed() {
    let root = repo_root();
    let policy = app::load_policy(&root).expect("load real cache-warmth policy");
    let license = app::load_license(&root).expect("load real cache-warm license");

    let default = &policy["default_for_unlisted_classes"];
    assert_eq!(default["warmth"], "cold", "unlisted default must be cold");
    assert_eq!(default["cache_read"], false);
    assert_eq!(default["cache_write"], false);

    assert_eq!(
        app::canary_class(&policy),
        Some("integrity-canary"),
        "the policy must name its canary trust anchor"
    );
    assert!(license["warm_reads_licensed"].is_boolean());
}

#[test]
fn dark_wiring_guarantee_under_the_real_license() {
    let root = repo_root();
    let policy = app::load_policy(&root).expect("policy");
    let license = app::load_license(&root).expect("license");
    let licensed = license["warm_reads_licensed"].as_bool().unwrap();

    let classes: Vec<String> = policy["build_classes"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .chain(std::iter::once("not-a-classified-class".to_string()))
        .collect();

    for class in &classes {
        let r = app::resolve(&policy, &license, class).expect("resolve");
        if !licensed {
            assert_eq!(
                r.mode,
                app::CacheMode::Bypass,
                "DARK-WIRING VIOLATION: class `{class}` resolved `{}` while \
                 warm_reads_licensed=false — no lane may touch the cache today",
                r.mode
            );
        } else {
            // Once the license flips (CAS bring-up + first GREEN canary), warm
            // classes must match their classified posture and cold stays bypass.
            let entry = policy["build_classes"].get(class);
            let warm = entry
                .map(|e| e["warmth"] == "warm" && e["cache_read"] == true)
                .unwrap_or(false)
                && class != "integrity-canary";
            assert_eq!(r.mode != app::CacheMode::Bypass, warm, "class `{class}`");
        }
    }
}

#[test]
fn cold_required_floor_holds_even_under_a_licensed_fixture() {
    let root = repo_root();
    let policy = app::load_policy(&root).expect("policy");
    let license = licensed_fixture();
    for class in COLD_REQUIRED_FLOOR {
        let entry = policy["build_classes"]
            .get(class)
            .unwrap_or_else(|| panic!("ADR-0556 one-way cold class `{class}` missing from policy"));
        assert_eq!(entry["warmth"], "cold", "`{class}` left the cold floor");
        assert_eq!(entry["cache_read"], false, "`{class}` gained cache_read");
        assert_eq!(entry["cache_write"], false, "`{class}` gained cache_write");
        let r = app::resolve(&policy, &license, class).expect("resolve");
        assert_eq!(
            r.mode,
            app::CacheMode::Bypass,
            "one-way floor: `{class}` must bypass even when warm is licensed"
        );
    }
}

#[test]
fn kill_switch_flips_warm_classes_and_only_warm_classes() {
    let root = repo_root();
    let policy = app::load_policy(&root).expect("policy");
    let unlicensed = json!({ "warm_reads_licensed": false, "reason": "fixture" });
    let licensed = licensed_fixture();

    let mut saw_warm = false;
    for (class, entry) in policy["build_classes"].as_object().unwrap() {
        let off = app::resolve(&policy, &unlicensed, class).unwrap().mode;
        let on = app::resolve(&policy, &licensed, class).unwrap().mode;
        assert_eq!(off, app::CacheMode::Bypass);
        if entry["warmth"] == "warm" && entry["cache_read"] == true && class != "integrity-canary" {
            saw_warm = true;
            let expected = if entry["cache_write"] == true {
                app::CacheMode::WarmReadWrite
            } else {
                app::CacheMode::WarmReadOnly
            };
            assert_eq!(on, expected, "licensed warm class `{class}`");
        } else {
            assert_eq!(
                on,
                app::CacheMode::Bypass,
                "cold class `{class}` must stay bypass"
            );
        }
    }
    assert!(
        saw_warm,
        "policy carries no warm-eligible class — fixture rot?"
    );
}

#[test]
fn overlays_parse_select_the_cache_platform_and_carry_no_identity() {
    let root = repo_root();
    for (path, uploads, endpoint_marker) in [
        (app::OVERLAY_RW_PATH, "true", "nativelink-cas-writer"),
        (app::OVERLAY_RO_PATH, "false", "nativelink-cas-reader"),
    ] {
        let text =
            std::fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("read {path}: {e}"));
        let cfg = app::parse_buckconfig(&text);

        let build = cfg
            .get("build")
            .unwrap_or_else(|| panic!("{path}: no [build]"));
        assert_eq!(
            build["execution_platforms"], "toolchains//cache:cache-platform",
            "{path} must select the cache execution platform"
        );

        let oya = cfg
            .get("oya_cache")
            .unwrap_or_else(|| panic!("{path}: no [oya_cache]"));
        assert_eq!(oya["remote_cache_enabled"], "true", "{path}");
        assert_eq!(oya["allow_cache_uploads"], uploads, "{path}");

        let re = cfg
            .get("buck2_re_client")
            .unwrap_or_else(|| panic!("{path}: no [buck2_re_client]"));
        assert_eq!(re["tls"], "true", "{path}: keyed transport is TLS-only");
        for key in ["engine_address", "cas_address", "action_cache_address"] {
            assert!(
                re[key].contains(endpoint_marker),
                "{path}: {key} must point at the {endpoint_marker} endpoint, got {}",
                re[key]
            );
        }
        assert!(
            !re.contains_key("tls_client_cert"),
            "{path}: the keyed identity must come from secret-mounted env at emit time, \
             never from the checked-in overlay"
        );
        assert!(
            !text.contains("PRIVATE KEY") && !text.to_lowercase().contains("api-key"),
            "{path}: secret material in a checked-in overlay"
        );
    }
}

#[test]
fn root_buckconfig_stays_dark() {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(".buckconfig")).expect("read root .buckconfig");
    let cfg = app::parse_buckconfig(&text);
    assert!(
        !cfg.contains_key("buck2_re_client"),
        "root .buckconfig grew a [buck2_re_client] section — cache wiring must stay opt-in \
         (ADR-0560 dark-wiring invariant)"
    );
    assert!(
        !cfg.contains_key("oya_cache"),
        "root .buckconfig grew an [oya_cache] section — cache wiring must stay opt-in"
    );
    assert_eq!(
        cfg["build"]["execution_platforms"], "prelude//platforms:default",
        "the default execution platform must stay the prelude default"
    );
}

#[test]
fn canary_workflow_is_scheduled_cold_and_wires_the_proof() {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(CANARY_WORKFLOW_PATH)).unwrap_or_else(|e| {
        panic!(
            "read {CANARY_WORKFLOW_PATH}: {e} — the canary MUST ship \
                                    with the CAS wiring (ADR-0556 D2: no canary, no warm)"
        )
    });
    assert!(
        text.contains("schedule:"),
        "canary must be cron-scheduled (ADR-0556 D4.3)"
    );
    assert!(
        !text.contains("actions/cache@"),
        "FROM-EMPTY VIOLATION: the canary workflow restores a cache — the proof is circular \
         (ADR-0556 D5 cold-must-stay)"
    );
    assert!(
        text.contains("--unstable-write-invocation-record"),
        "canary must capture the structured invocation record"
    );
    assert!(
        text.contains("assert-cold"),
        "canary must mechanically prove zero cache participation (assert-cold)"
    );
    assert!(
        text.contains("integrity-canary"),
        "canary must run under the integrity-canary build class"
    );
    assert!(
        text.contains("canary-verdict"),
        "canary must emit the structured verdict artifact"
    );
}

/// Slice the body of a named workflow step out of `text`, bounded by the name of the
/// step that follows it.
fn workflow_step<'a>(text: &'a str, name: &str, next: &str) -> Option<&'a str> {
    text.split(name)
        .nth(1)
        .and_then(|tail| tail.split(next).next())
}

/// Pure kernel of the warm-probe participation binding: given the canary workflow
/// text, return every reason its warm side fails to PROVE the cache served it.
///
/// Scoped to the two steps that own the warm path, on purpose. The neighbouring cold
/// assertions are whole-file `contains` checks and the COLD step already satisfies
/// every token they look for, so a whole-file check on the warm side would be
/// satisfied by the cold step and assert exactly nothing. That is the hole this
/// closes: byte-equality alone is also satisfied by a probe that served nothing and
/// rebuilt locally, so without a participation proof the canary can emit the GREEN
/// that licenses warm reads fleet-wide while proving nothing (ADR-0556 D2).
fn warm_probe_findings(text: &str) -> Vec<String> {
    let mut findings = Vec::new();

    match workflow_step(text, "- name: Warm-probe fetch", "- name: Canary verdict") {
        Some(probe) if probe.contains("continue-on-error") => findings.push(
            "the warm-probe step is non-binding (continue-on-error) — a probe that failed to \
             fetch must fail the canary"
                .to_string(),
        ),
        Some(probe) => {
            if !probe.contains(&format!(
                "--unstable-write-invocation-record {WARM_RECORD_PATH}"
            )) {
                findings.push(format!(
                    "the warm probe writes no structured invocation record at {WARM_RECORD_PATH} \
                     — with no counters there is nothing to prove participation from"
                ));
            }
        }
        None => findings.push(
            "the canary has no warm-probe step bounded by the verdict step — the warm side of \
             the trust anchor is missing entirely"
                .to_string(),
        ),
    }

    let Some(verdict) = workflow_step(
        text,
        "- name: Canary verdict",
        "- name: Cache-hit telemetry",
    ) else {
        findings.push(
            "the canary has no verdict step bounded by the telemetry step — nothing scores the \
             warm side"
                .to_string(),
        );
        return findings;
    };
    if verdict.contains("continue-on-error") {
        findings.push(
            "the verdict step is non-binding (continue-on-error) — a failed participation proof \
             must fail the canary"
                .to_string(),
        );
    }
    // Every invocation that scores a warm manifest must carry the full participation
    // binding on that SAME command line: the probe's own record, the class it ran as,
    // and the warm mode it dialed. Split across lines or left implicit, the verdict
    // could be produced without the proof.
    let scoring: Vec<&str> = verdict
        .lines()
        .filter(|line| line.contains("canary-verdict") && line.contains("--warm "))
        .collect();
    if scoring.is_empty() {
        findings.push(
            "no canary-verdict invocation scores a warm manifest (`--warm `) — the warm side is \
             never compared at all"
                .to_string(),
        );
    }
    for line in scoring {
        if !line.contains(&format!("--warm-record {WARM_RECORD_PATH}")) {
            findings.push(format!(
                "a canary-verdict invocation scores a warm manifest without \
                 `--warm-record {WARM_RECORD_PATH}` — a probe that fetched zero entries would \
                 still emit GREEN, fabricating the one piece of evidence that licenses warm \
                 reads fleet-wide (ADR-0556 D2)"
            ));
        }
        if !line.contains("--build-class integrity-canary") || !line.contains("--mode warm-ro") {
            findings.push(
                "a canary-verdict invocation scores a warm manifest without \
                 `--build-class integrity-canary --mode warm-ro` — under a bypass/cold mode the \
                 participation kernel skips every hit-count check and passes on zero cache \
                 participation"
                    .to_string(),
            );
        }
    }
    findings
}

#[test]
fn canary_warm_probe_must_prove_the_cache_served_it() {
    let root = repo_root();
    let live = std::fs::read_to_string(root.join(CANARY_WORKFLOW_PATH))
        .unwrap_or_else(|e| panic!("read {CANARY_WORKFLOW_PATH}: {e}"));

    let findings = warm_probe_findings(&live);
    assert!(
        findings.is_empty(),
        "the live canary warm probe cannot prove the cache served anything: {findings:#?}"
    );

    // RED fixtures: each is the LIVE workflow with exactly one binding removed. A gate
    // observed passing is not evidence — these are the standing demonstration that the
    // assertion above is capable of failing, and they rot loudly if the live steps
    // drift. Checked AFTER the live corpus so a real regression reports the real
    // finding rather than fixture rot.
    for (name, broken) in [
        (
            "probe writes no invocation record",
            live.replace(
                &format!(" --unstable-write-invocation-record {WARM_RECORD_PATH}"),
                "",
            ),
        ),
        (
            "verdict scores the warm manifest without the probe's record",
            live.replace(&format!(" --warm-record {WARM_RECORD_PATH}"), ""),
        ),
        (
            "mode weakened to bypass (skips every hit-count check)",
            live.replace("--mode warm-ro", "--mode bypass"),
        ),
        (
            "class weakened away from the canary",
            live.replace(
                "--build-class integrity-canary --mode warm-ro",
                "--build-class gate-fleet-shared-graph --mode warm-ro",
            ),
        ),
        (
            "warm-probe step removed",
            live.replace("- name: Warm-probe fetch", "- name: Something else"),
        ),
    ] {
        // `assert!` not `assert_ne!`: the operands are whole workflow files.
        assert!(broken != live, "RED fixture `{name}` mutated nothing — rot");
        assert!(
            !warm_probe_findings(&broken).is_empty(),
            "RED fixture `{name}` was accepted: this gate cannot fail, so it is not evidence"
        );
    }
}

#[test]
fn required_workflow_cache_hit_report_is_binding() {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(REQUIRED_WORKFLOW_PATH)).unwrap_or_else(|e| {
        panic!(
            "read {REQUIRED_WORKFLOW_PATH}: {e} — the required CI workflow must ship the \
             cache-hit report guard"
        )
    });
    let telemetry_step = text
        .split("- name: Cache-hit telemetry + warm-mode guard (ADR-0560)")
        .nth(1)
        .and_then(|tail| {
            tail.split("- name: Upload cache-hit telemetry artifact")
                .next()
        })
        .expect("required workflow must contain the cache-hit telemetry guard step");
    assert!(
        telemetry_step.contains("--unstable-write-invocation-record")
            || text.contains(
                "--unstable-write-invocation-record /tmp/buck2-lane-invocation-record.json"
            ),
        "the buck2 lane must capture a structured invocation record before reporting cache health"
    );
    assert!(
        telemetry_step.contains(" report --record /tmp/buck2-lane-invocation-record.json")
            && telemetry_step.contains("--out /tmp/cache-hit-report.json"),
        "the cache-hit report must be generated from the structured invocation record"
    );
    assert!(
        telemetry_step.contains(" assert-warm --record /tmp/buck2-lane-invocation-record.json"),
        "warm/bypass cache participation must be asserted in the binding telemetry step"
    );
    assert!(
        !telemetry_step.contains("continue-on-error"),
        "the cache-hit telemetry guard must be binding; missing counters or 0% warm hits cannot pass"
    );

    let upload_step = text
        .split("- name: Upload cache-hit telemetry artifact")
        .nth(1)
        .and_then(|tail| {
            tail.split("- name: Upload runner disk reclaim operator artifact")
                .next()
        })
        .expect("required workflow must contain the cache-hit artifact upload step");
    assert!(
        upload_step.contains("name: cache-hit-report-buck2-lane")
            && upload_step.contains("path: /tmp/cache-hit-report.json"),
        "the cache-hit report artifact must be uploaded under the stable diagnostic name/path"
    );
    assert!(
        upload_step.contains("if-no-files-found: error"),
        "a missing cache-hit report must be RED, not a warning, once the required lane ran"
    );
    assert!(
        !upload_step.contains("continue-on-error"),
        "uploading the cache-hit report must stay binding so diagnostics cannot silently disappear"
    );
}

#[test]
fn cache_hit_guard_behavior_covers_bypass_warm_and_malformed_records() {
    let bypass_zero = invocation_record_fixture(0.0, 0, 12, 0);
    assert!(
        app::assert_warm_cache_participation(&bypass_zero, "gate-fleet-shared-graph", "bypass")
            .is_ok(),
        "current bypass/cold posture must stay allowed even with zero cache hits"
    );

    let warm_hit = invocation_record_fixture(0.25, 3, 9, 0);
    assert!(
        app::assert_warm_cache_participation(&warm_hit, "gate-fleet-shared-graph", "warm-rw")
            .is_ok(),
        "warm mode with a positive hit rate and positive action-cache count must pass"
    );

    let warm_zero = invocation_record_fixture(0.0, 0, 12, 0);
    let findings =
        app::assert_warm_cache_participation(&warm_zero, "gate-fleet-shared-graph", "warm-rw")
            .unwrap_err();
    assert!(
        findings.iter().any(|f| f.contains("0% hit rate"))
            && findings
                .iter()
                .any(|f| f.contains("run_action_cache_count=0")),
        "warm mode with 0% hits must be RED: {findings:?}"
    );

    let malformed = json!({ "exit_result_name": "SUCCESS" });
    let findings =
        app::assert_warm_cache_participation(&malformed, "gate-fleet-shared-graph", "warm-rw")
            .unwrap_err();
    assert!(
        findings
            .iter()
            .any(|f| f.contains("record-shape violation")),
        "missing or renamed cache counters must be RED: {findings:?}"
    );
}

#[test]
fn bundled_canary_targets_stay_inside_the_binding_gate_cone() {
    let policy = app::canary_policy().expect("bundled canary policy");
    let targets = policy["pinned_targets"].as_array().unwrap();
    assert!(!targets.is_empty());
    for target in targets {
        let t = target.as_str().unwrap();
        assert!(
            t.starts_with("//"),
            "pinned target `{t}` must be a repo-anchored pattern"
        );
    }
}
