// governance-check-authz-tier-discipline LIVE-TREE gate (ADR-0191).
//
// Sibling `authz_tier_discipline.rs` proves the kernel correct on hand-written fixtures. It says
// nothing about this repository, and until this file existed nothing did: the crate's only Cargo
// consumer was marketplace/facade/dev-cli, which no workflow invokes, so the doctrine had never
// produced a verdict about the tree it governs — while 103 of the 750 tracked Cedar policies were
// already reaching for edge-tier attributes the origin PDP is not supposed to see.
//
// The kernel is pure; this is the CALLER that walks the real repository and hands it observations
// as DATA. Walk failures are ERRORS, never omitted observations: a policy dropped from the census
// because its contents failed to read would quietly shrink the frozen map, and a shrink reads as
// repair.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use check_authz_tier_discipline::{Finding, scan_cedar, scan_envoy_filter};

const POLICY_PATH: &str =
    "governance/check/authz-tier-discipline/authz-tier-discipline-policy.json";
const MAX_SCANNED_BYTES: u64 = 4_194_304;

struct Policy {
    min_tracked_files: usize,
    min_cedar_policies: usize,
    min_envoy_configs: usize,
    frozen_tier_leaks: BTreeMap<String, usize>,
}

struct Observed {
    cedar_policies: usize,
    envoy_configs: usize,
    tracked_files: usize,
    /// `<file>::<needle>` -> how many lines in that file reach for that needle.
    ///
    /// Deliberately NOT line-anchored: the kernel reports a line number, but a line number moves
    /// whenever anything above it is edited, so a line-keyed baseline forces a blind re-freeze on
    /// edits that changed nothing about tier discipline — and an edit above a guarded construct
    /// can leave no legal edit at all. Deliberately NOT a bare set either: `<file>::<needle>`
    /// alone would let a second `bot_score` line be added to a file that already has one without
    /// the ratchet noticing, so the multiplicity is carried as the value.
    tier_leaks: BTreeMap<String, usize>,
    /// Findings retained whole, for the human-readable report only.
    findings: Vec<Finding>,
}

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join(POLICY_PATH).is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root (the dir holding {POLICY_PATH})");
}

fn load_policy(root: &Path) -> Policy {
    let raw = std::fs::read_to_string(root.join(POLICY_PATH)).expect("read policy");
    let doc: serde_json::Value = serde_json::from_str(&raw).expect("policy parses");
    let number = |key: &str| -> usize {
        usize::try_from(
            doc[key]
                .as_u64()
                .unwrap_or_else(|| panic!("policy field {key} missing or not a number")),
        )
        .expect("policy number fits usize")
    };
    let frozen = doc["frozen_tier_leaks"]
        .as_object()
        .expect("policy field frozen_tier_leaks missing or not an object")
        .iter()
        .map(|(key, value)| {
            let count = usize::try_from(
                value
                    .as_u64()
                    .unwrap_or_else(|| panic!("frozen_tier_leaks[{key}] is not a number")),
            )
            .expect("count fits usize");
            (key.clone(), count)
        })
        .collect();
    Policy {
        min_tracked_files: number("min_tracked_files"),
        min_cedar_policies: number("min_cedar_policies"),
        min_envoy_configs: number("min_envoy_configs"),
        frozen_tier_leaks: frozen,
    }
}

/// The tracked file list, from git — the same corpus boundary every other live gate here uses.
///
/// Walking the working tree instead would measure a different corpus than CI does the moment an
/// ignored `*.cedar` or `*.yaml` exists on disk, and with the map pinned by equality that is a red
/// gate CI cannot reproduce.
fn tracked_files(root: &Path) -> Result<Vec<String>, String> {
    let out = Command::new("git")
        .current_dir(root)
        .args(["ls-files", "-z"])
        .output()
        .map_err(|e| format!("git ls-files failed to start: {e}"))?;
    if !out.status.success() {
        return Err(format!("git ls-files exited with {}", out.status));
    }
    let text =
        String::from_utf8(out.stdout).map_err(|e| format!("git ls-files output not UTF-8: {e}"))?;
    Ok(text
        .split('\0')
        .filter(|entry| !entry.is_empty())
        .map(str::to_owned)
        .collect())
}

/// Is this an Envoy edge-filter configuration?
///
/// Structured config only. `docs/adr-archive/ADR-0044-service-mesh-istio-ambient-and-envoy-*.md`
/// and `k8s/runbooks/envoy-sni-debug.md` discuss the edge tier in prose and configure nothing;
/// admitting prose would put permanent unrepairable text into a corpus whose whole purpose is to
/// name configuration that reaches across the tier boundary.
fn is_envoy_config(relative: &str) -> bool {
    (relative.ends_with(".yaml") || relative.ends_with(".yml") || relative.ends_with(".json"))
        && relative.to_ascii_lowercase().contains("envoy")
}

fn read_tracked(root: &Path, relative: &str) -> Result<Option<String>, String> {
    let path = root.join(relative);
    // Every failure below is an ERROR, never an omitted observation.
    let metadata = std::fs::metadata(&path)
        .map_err(|e| format!("metadata {relative} failed: {e} (tracked but unreadable)"))?;
    if !metadata.is_file() {
        return Ok(None); // a tracked symlink to a directory carries no policy text
    }
    if metadata.len() > MAX_SCANNED_BYTES {
        return Err(format!(
            "{relative} is {} bytes, over the {MAX_SCANNED_BYTES}-byte scan cap — raise the cap \
             deliberately rather than dropping the file from the census",
            metadata.len()
        ));
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("read {relative} failed: {e}"))?;
    // LOSSY, never skipped: a non-UTF-8 payload still carries the ASCII needles perfectly well.
    Ok(Some(String::from_utf8_lossy(&bytes).into_owned()))
}

fn observe(root: &Path) -> Result<Observed, String> {
    let tracked = tracked_files(root)?;
    let mut findings: Vec<Finding> = Vec::new();
    let mut cedar_policies = 0usize;
    let mut envoy_configs = 0usize;

    for relative in &tracked {
        if relative.ends_with(".cedar") {
            let Some(body) = read_tracked(root, relative)? else {
                continue;
            };
            cedar_policies += 1;
            findings.extend(scan_cedar(relative, &body).findings);
        } else if is_envoy_config(relative) {
            let Some(body) = read_tracked(root, relative)? else {
                continue;
            };
            envoy_configs += 1;
            findings.extend(scan_envoy_filter(relative, &body).findings);
        }
    }

    let mut tier_leaks: BTreeMap<String, usize> = BTreeMap::new();
    for finding in &findings {
        *tier_leaks
            .entry(format!("{}::{}", finding.file, finding.needle))
            .or_default() += 1;
    }

    Ok(Observed {
        cedar_policies,
        envoy_configs,
        tracked_files: tracked.len(),
        tier_leaks,
        findings,
    })
}

/// The live walk, done ONCE for the whole binary: it is a pure function of the tree, so every test
/// re-walking it would recompute the same answer over ~16k tracked paths.
fn live() -> &'static (Policy, Observed) {
    static LIVE: OnceLock<(Policy, Observed)> = OnceLock::new();
    LIVE.get_or_init(|| {
        let root = repo_root();
        let policy = load_policy(&root);
        let observed = observe(&root).expect("live walk");
        (policy, observed)
    })
}

fn census(observed: &Observed) -> String {
    let mut by_needle: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in &observed.findings {
        *by_needle.entry(finding.needle.as_str()).or_default() += 1;
    }
    let mut out = format!(
        "census: {} Cedar policies + {} Envoy configs over {} tracked files; {} tier leaks across \
         {} (file, needle) keys\n",
        observed.cedar_policies,
        observed.envoy_configs,
        observed.tracked_files,
        observed.findings.len(),
        observed.tier_leaks.len(),
    );
    for (needle, count) in by_needle {
        out.push_str(&format!("  {needle}: {count}\n"));
    }
    out
}

/// ANTI-VACUITY, asserted before any equality below is read.
///
/// A ratchet pinned by equality cannot distinguish "the corpus was repaired" from "the walk
/// collapsed"; both drive the observed map toward empty. These floors are the machine oracle that
/// separates them. Every floor counts SUBJECT FILES, never findings, so repairing a tier leak
/// moves the frozen map and leaves all three floors exactly where they are — no floor here can red
/// on honest progress.
#[test]
fn the_policy_corpus_is_intact() {
    let (policy, observed) = live();
    assert!(
        observed.tracked_files >= policy.min_tracked_files,
        "git ls-files returned {} tracked paths, below the floor of {} — the corpus walk is broken \
         and every count below is meaningless\n{}",
        observed.tracked_files,
        policy.min_tracked_files,
        census(observed)
    );
    assert!(
        observed.cedar_policies >= policy.min_cedar_policies,
        "{} Cedar policies found, below the floor of {}. Authorization policy does not disappear \
         in bulk; a drop here is a narrowed scan, and a narrowed scan reports a clean tier \
         boundary it never read\n{}",
        observed.cedar_policies,
        policy.min_cedar_policies,
        census(observed)
    );
    assert!(
        observed.envoy_configs >= policy.min_envoy_configs,
        "{} Envoy edge configs found, below the floor of {} — the edge half of the tier boundary \
         went unscanned, so its zero findings are not evidence\n{}",
        observed.envoy_configs,
        policy.min_envoy_configs,
        census(observed)
    );
}

/// THE GATE: a SHRINK-ONLY, TWO-SIDED ratchet on the MAP of `(file, needle)` tier leaks.
///
/// Keys, not a count. ADR-0191 splits authorization in two: network-layer attributes (ip, asn,
/// geo, rate, waf, bot, ddos) belong at the Envoy edge, identity-layer attributes (acr, tenant,
/// residency, purpose, data_class) belong at the Cedar origin PDP. A count would tell a reviewer
/// that the number moved and nothing about which policy moved; `<file>::<needle>` names the policy
/// and the attribute it reached for, and is reviewable on its face.
///
/// TWO-SIDED, over the UNION of both key sets. A new leak appears above its pin and blocks; a
/// repaired one falls below its pin and ALSO blocks, forcing the pin down in the same change. The
/// union matters: iterating only the frozen keys makes a NEW key invisible, which is the same hole
/// as an unratcheted count, one level down.
///
/// WHAT THIS DOES NOT DETECT, stated at the strength the mechanism has: a change that is net-zero
/// for one needle inside one file — remove one `bot_score` line and add another to the same policy
/// and the pin is unmoved. Keying by `(file, needle)` narrows that hole to a single file-attribute
/// pair; it does not close it. The suppression marker (`// authz-tier-discipline: ok (<reason>)`)
/// is the supported way to record a deliberate exception, and using it DOES move the pin.
#[test]
fn tier_leaks_equal_the_frozen_map() {
    let (policy, observed) = live();

    let keys: BTreeMap<&String, ()> = policy
        .frozen_tier_leaks
        .keys()
        .chain(observed.tier_leaks.keys())
        .map(|key| (key, ()))
        .collect();
    let drift: Vec<String> = keys
        .into_keys()
        .filter_map(|key| {
            let seen = observed.tier_leaks.get(key).copied().unwrap_or(0);
            let want = policy.frozen_tier_leaks.get(key).copied().unwrap_or(0);
            (seen != want).then(|| format!("  {key}: observed {seen}, frozen {want}"))
        })
        .collect();

    assert!(
        drift.is_empty(),
        "authz tier-leak drift, per (file, needle). ABOVE the pin: a policy reached across the \
         ADR-0191 tier boundary — move the concern to the tier that owns it, or record the \
         exception with a `// authz-tier-discipline: ok (<reason>)` marker on the line. BELOW the \
         pin: lower `frozen_tier_leaks` in THIS change so the win is recorded, or discover that \
         the scan narrowed and is reporting green over policies it stopped reading. Re-derive by \
         RUNNING this gate and reading 'observed N' from these lines; never by arithmetic on the \
         old values:\n{}\n{}",
        drift.join("\n"),
        census(observed)
    );
}

/// The gate is DEMONSTRATED CAPABLE OF FAILING against the REAL corpus, not just fixtures.
///
/// A green ratchet proves nothing on its own: a caller that silently produced zero findings would
/// satisfy every assertion above by reporting a perfectly disciplined tree. This injects each
/// side's defect shape into a copy of a REAL policy's text and asserts the count rises — then
/// injects the SAME defect carrying a suppression marker and asserts it does not, so the marker is
/// proven to be a documented exception rather than a blanket mute.
#[test]
fn injecting_each_tier_leak_into_a_real_policy_reddens_the_gate() {
    let root = repo_root();
    let tracked = tracked_files(&root).expect("git ls-files");

    let cedar = tracked
        .iter()
        .find(|relative| relative.ends_with(".cedar"))
        .expect("no tracked Cedar policy exists; the origin half of this gate has no subject");
    let cedar_body = read_tracked(&root, cedar)
        .expect("read cedar")
        .expect("cedar policy is a file");
    let cedar_before = scan_cedar(cedar, &cedar_body).findings.len();
    let leaked = format!("{cedar_body}\n  when {{ context.bot_score < 50 }};\n");
    assert_eq!(
        scan_cedar(cedar, &leaked).findings.len(),
        cedar_before + 1,
        "an edge-tier attribute injected into the live Cedar policy {cedar} did not raise the \
         finding count"
    );
    let suppressed = format!(
        "{cedar_body}\n  when {{ context.bot_score < 50 }}; // authz-tier-discipline: ok (probe)\n"
    );
    assert_eq!(
        scan_cedar(cedar, &suppressed).findings.len(),
        cedar_before,
        "the suppression marker failed to suppress on live policy text, so the frozen map is \
         pinned against a rule nobody can actually opt out of"
    );

    let envoy = tracked
        .iter()
        .find(|relative| is_envoy_config(relative))
        .expect("no tracked Envoy config exists; the edge half of this gate has no subject");
    let envoy_body = read_tracked(&root, envoy)
        .expect("read envoy")
        .expect("envoy config is a file");
    let envoy_before = scan_envoy_filter(envoy, &envoy_body).findings.len();
    let leaked = format!("{envoy_body}\n  header_to_metadata: principal.acr\n");
    assert_eq!(
        scan_envoy_filter(envoy, &leaked).findings.len(),
        envoy_before + 1,
        "an origin-tier claim injected into the live Envoy config {envoy} did not raise the \
         finding count"
    );

    println!(
        "mutation proof: {cedar} {cedar_before} -> {} on injection, back to {cedar_before} when \
         suppressed; {envoy} {envoy_before} -> {}",
        cedar_before + 1,
        envoy_before + 1
    );
}

/// Evidence, always printed, so a reader can tell a repaired corpus from a collapsed walk without
/// re-running anything.
#[test]
fn live_census_is_reported() {
    let (_, observed) = live();
    println!("{}", census(observed));
    assert!(observed.cedar_policies > 0);
}
