// governance-check-idempotency-key-coverage live-tree gate (ADR-0149, restated in ADR-0709).
//
// The kernel is pure; this is the CALLER that walks the real repository and hands it observations
// as DATA. Until this file existed the doctrine had never produced a verdict about this codebase:
// the crate's only consumer was marketplace/facade/dev-cli, which no workflow invokes, so every
// case it had ever run was a hand-written fixture.
//
// Walk failures are ERRORS, never omitted observations. A document dropped from the census because
// its metadata or contents failed to read would quietly shrink the frozen set, and a shrink reads
// as repair.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use check_idempotency_key_coverage::{CoverageReport, OpenApiDocument, audit_all};

const POLICY_PATH: &str =
    "governance/check/idempotency-key-coverage/idempotency-key-coverage-policy.json";
const MAX_SCANNED_BYTES: u64 = 4_194_304;

struct Policy {
    min_openapi_documents: usize,
    min_state_changing_operations: usize,
    min_capabilities: usize,
    min_tracked_files: usize,
    frozen_uncovered_operations: BTreeSet<String>,
}

struct Observed {
    report: CoverageReport,
    /// One key per uncovered state-changing operation: `<doc path>::<route>::<verb>`.
    ///
    /// Deliberately NOT line-anchored. A line number moves whenever anything above the operation
    /// is edited, so a line-keyed baseline forces a re-freeze on edits that changed nothing about
    /// coverage — and an edit above a guarded construct can leave no legal edit at all.
    uncovered: BTreeSet<String>,
    tracked_files: usize,
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
    let frozen: BTreeSet<String> = doc["frozen_uncovered_operations"]
        .as_array()
        .expect("policy field frozen_uncovered_operations missing or not an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("frozen_uncovered_operations holds a non-string")
                .to_owned()
        })
        .collect();
    Policy {
        min_openapi_documents: number("min_openapi_documents"),
        min_state_changing_operations: number("min_state_changing_operations"),
        min_capabilities: number("min_capabilities"),
        min_tracked_files: number("min_tracked_files"),
        frozen_uncovered_operations: frozen,
    }
}

/// The tracked file list, from git — the same corpus boundary every other live gate here uses.
///
/// Reading the working tree instead would measure a different corpus than CI does the moment a
/// developer has an ignored or untracked `*.yaml` on disk, and with the set pinned by equality
/// that is a red gate CI cannot reproduce.
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

/// The capability a contract belongs to: everything before `/contracts/`, else the first segment.
///
/// Only ever used for the human-readable report and the capability floor; the frozen keys carry
/// the full document path, so a re-home never silently re-labels a frozen finding.
fn capability_of(relative: &str) -> String {
    relative
        .split_once("/contracts/")
        .map_or_else(
            || relative.split('/').next().unwrap_or(relative),
            |(head, _)| head,
        )
        .to_owned()
}

/// Is this document an OpenAPI description?
///
/// The marker is `openapi:` at column ZERO. An indented `openapi:` is a nested key inside some
/// other document (a Helm value, a catalog entry) and describes no operations; admitting those
/// would put non-contracts into a census whose floor is meant to prove the contract corpus is
/// intact.
fn is_openapi(text: &str) -> bool {
    text.lines().any(|line| line.starts_with("openapi:"))
}

/// The route a line names, if it names one.
///
/// OpenAPI path keys are frequently QUOTED in this repo, because AIP-136 custom methods embed a
/// colon (`"/principals/{id}:suspend"`). A bare `starts_with('/')` test misses every one of them
/// and the operation underneath would be attributed to whatever route happened to appear earlier
/// in the file — three distinct operations collapsing onto one key, which is exactly the
/// unattributable freeze this gate is built to avoid.
fn route_named_by(trimmed: &str) -> Option<String> {
    let trimmed = trimmed.trim_end();
    for quote in ['"', '\''] {
        if let Some(rest) = trimmed.strip_prefix(quote) {
            let end = rest.find(quote)?;
            let route = &rest[..end];
            return route.starts_with('/').then(|| route.to_owned());
        }
    }
    if !trimmed.starts_with('/') {
        return None;
    }
    let route = trimmed.strip_suffix(':')?;
    Some(route.to_owned())
}

/// The route key an operation sits under: the nearest line ABOVE it at shallower indentation that
/// names a path.
fn enclosing_route(lines: &[&str], op_index: usize, op_indent: usize) -> Option<String> {
    for line in lines[..op_index].iter().rev() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = line.len() - trimmed.len();
        if indent >= op_indent {
            continue;
        }
        if let Some(route) = route_named_by(trimmed) {
            return Some(route);
        }
    }
    None
}

const STATE_CHANGING_VERBS: [&str; 4] = ["post:", "put:", "patch:", "delete:"];

/// Re-derive the (route, verb) identity of the operation the kernel flagged at `line`.
///
/// The kernel reports a 1-indexed start line and a verb; it does not report the route, because it
/// is a pure line scanner with no document model. Naming the route here is CALLER work by
/// construction, and it is what turns an unattributable count into a reviewable diff.
fn key_for(path: &str, text: &str, line: usize, verb: &str) -> Result<String, String> {
    let lines: Vec<&str> = text.lines().collect();
    let index = line
        .checked_sub(1)
        .ok_or_else(|| format!("{path}: kernel reported line 0"))?;
    let raw = lines
        .get(index)
        .ok_or_else(|| format!("{path}: kernel reported line {line}, past end of document"))?;
    let trimmed = raw.trim_start();
    if !trimmed.starts_with(&format!("{verb}:")) {
        return Err(format!(
            "{path}:{line}: kernel reported verb '{verb}' but the line reads '{trimmed}' — caller \
             and kernel disagree about the document, so every key below would be misattributed"
        ));
    }
    let indent = raw.len() - trimmed.len();
    let route = enclosing_route(&lines, index, indent).ok_or_else(|| {
        format!(
            "{path}:{line}: no enclosing path key found above this '{verb}' operation. Refusing to \
             guess: an operation attributed to the wrong route silently merges two frozen keys \
             into one and hides a regression."
        )
    })?;
    Ok(format!("{path}::{route}::{verb}"))
}

fn observe(root: &Path) -> Result<Observed, String> {
    let tracked = tracked_files(root)?;
    let mut documents: Vec<OpenApiDocument> = Vec::new();
    for relative in &tracked {
        if !(relative.ends_with(".yaml") || relative.ends_with(".yml")) {
            continue;
        }
        let path = root.join(relative);
        // Every failure below is an ERROR. Skipping an unreadable tracked contract would shrink
        // the frozen set without anyone repairing anything.
        let metadata = std::fs::metadata(&path)
            .map_err(|e| format!("metadata {relative} failed: {e} (tracked but unreadable)"))?;
        if !metadata.is_file() {
            continue; // a tracked symlink to a directory is not a contract document
        }
        if metadata.len() > MAX_SCANNED_BYTES {
            return Err(format!(
                "{relative} is {} bytes, over the {MAX_SCANNED_BYTES}-byte scan cap — raise the cap \
                 deliberately rather than dropping the file from the census",
                metadata.len()
            ));
        }
        let bytes = std::fs::read(&path).map_err(|e| format!("read {relative} failed: {e}"))?;
        // LOSSY, never skipped: latin-1 and UTF-16 both carry ASCII `openapi:` and `post:` fine.
        let contents = String::from_utf8_lossy(&bytes).into_owned();
        if !is_openapi(&contents) {
            continue;
        }
        documents.push(OpenApiDocument {
            path: relative.clone(),
            microservice: capability_of(relative),
            contents,
        });
    }
    documents.sort_by(|a, b| a.path.cmp(&b.path));

    let by_path: BTreeMap<String, String> = documents
        .iter()
        .map(|doc| (doc.path.clone(), doc.contents.clone()))
        .collect();
    let (report, findings) = audit_all(documents);

    let mut uncovered = BTreeSet::new();
    for finding in &findings {
        let contents = by_path
            .get(&finding.path)
            .ok_or_else(|| format!("kernel reported {} which was never supplied", finding.path))?;
        // The kernel's message is `state-changing operation '<verb>' missing …`; the verb is the
        // only structured field it carries besides path and line.
        let verb = finding
            .message
            .split('\'')
            .nth(1)
            .ok_or_else(|| format!("finding message shape changed: {}", finding.message))?;
        let key = key_for(&finding.path, contents, finding.line, verb)?;
        if !uncovered.insert(key.clone()) {
            return Err(format!(
                "{key} was produced twice. Two operations collapsed onto one frozen key, so one of \
                 them is invisible to the ratchet — disambiguate the key before re-freezing."
            ));
        }
    }

    Ok(Observed {
        report,
        uncovered,
        tracked_files: tracked.len(),
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
    format!(
        "census: {} OpenAPI documents over {} tracked files, {} state-changing operations ({} \
         covered), {} capabilities; {} uncovered operations\n",
        observed.report.documents_checked,
        observed.tracked_files,
        observed.report.state_changing_ops_checked,
        observed.report.state_changing_ops_covered,
        observed.report.microservices_audited,
        observed.uncovered.len(),
    )
}

/// ANTI-VACUITY, asserted before any equality below is read.
///
/// A set-equality ratchet cannot distinguish "the corpus was repaired" from "the walk collapsed
/// and saw nothing" — both drive the observed set toward empty. These floors are the machine
/// oracle that separates them. Every floor here has a live, non-zero target today, so none of them
/// reds on honest progress: repairing an idempotency gap moves the FROZEN SET, never these.
#[test]
fn the_openapi_corpus_is_intact() {
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
        observed.report.documents_checked >= policy.min_openapi_documents,
        "{} OpenAPI documents found, below the floor of {}. Contracts do not disappear in bulk; a \
         drop here is a narrowed scan, and a narrowed scan reports a clean tree it never read\n{}",
        observed.report.documents_checked,
        policy.min_openapi_documents,
        census(observed)
    );
    assert!(
        observed.report.state_changing_ops_checked >= policy.min_state_changing_operations,
        "{} state-changing operations parsed, below the floor of {} — the operation extractor \
         stopped matching\n{}",
        observed.report.state_changing_ops_checked,
        policy.min_state_changing_operations,
        census(observed)
    );
    assert!(
        observed.report.microservices_audited >= policy.min_capabilities,
        "{} capabilities audited, below the floor of {}\n{}",
        observed.report.microservices_audited,
        policy.min_capabilities,
        census(observed)
    );
}

/// THE GATE: a SHRINK-ONLY, TWO-SIDED ratchet on the SET of uncovered operations.
///
/// A SET, not a count. A count tells a reviewer that the number moved and nothing about which
/// operation moved, so re-freezing it is a blind edit; a set diff names the route that gained or
/// lost its `Idempotency-Key` and is reviewable on its face.
///
/// TWO-SIDED. A new uncovered operation appears in `observed - frozen` and blocks: ADR-0149 is
/// mandatory, so a new state-changing endpoint without an idempotency key is born-blocking. A
/// repaired one appears in `frozen - observed` and ALSO blocks, until it is struck from the policy
/// in the SAME change — because a one-sided ceiling cannot tell debt-paid-off from a scan that
/// collapsed, and the second is the failure mode this whole programme exists to remove.
#[test]
fn uncovered_operations_equal_the_frozen_set() {
    let (policy, observed) = live();

    let appeared: Vec<&String> = observed
        .uncovered
        .difference(&policy.frozen_uncovered_operations)
        .collect();
    let repaired: Vec<&String> = policy
        .frozen_uncovered_operations
        .difference(&observed.uncovered)
        .collect();

    let mut message = String::new();
    if !appeared.is_empty() {
        message.push_str(&format!(
            "{} state-changing operation(s) have NO Idempotency-Key and are not in the frozen set. \
             ADR-0149 is mandatory: declare the header (inline or \
             `$ref: '#/components/parameters/IdempotencyKey'`) rather than baselining it.\n",
            appeared.len()
        ));
        for key in appeared.iter().take(40) {
            message.push_str(&format!("  + {key}\n"));
        }
        if appeared.len() > 40 {
            message.push_str(&format!("  … {} more\n", appeared.len() - 40));
        }
    }
    if !repaired.is_empty() {
        message.push_str(&format!(
            "{} frozen operation(s) are no longer produced. If you covered them, strike these \
             lines from `frozen_uncovered_operations` in THIS change so the win is recorded. If \
             you did not, the scan narrowed and is now reporting green over documents it stopped \
             reading.\n",
            repaired.len()
        ));
        for key in repaired.iter().take(40) {
            message.push_str(&format!("  - {key}\n"));
        }
        if repaired.len() > 40 {
            message.push_str(&format!("  … {} more\n", repaired.len() - 40));
        }
    }
    assert!(message.is_empty(), "{message}{}", census(observed));
}

/// The gate is DEMONSTRATED CAPABLE OF FAILING against the REAL corpus, not just fixtures.
///
/// A green ratchet proves nothing on its own: a caller that silently produced zero findings would
/// satisfy every assertion above by reporting a perfectly clean tree. This strips the
/// `Idempotency-Key` parameter out of a real, currently-COVERED operation and asserts the kernel
/// notices — over live document text, so it also proves the covered side of the corpus is real.
#[test]
fn stripping_the_header_from_a_real_covered_operation_reddens_the_kernel() {
    let (_, observed) = live();
    assert!(
        observed.report.state_changing_ops_covered > 0,
        "no covered operation exists to mutate; the corpus is not what this test assumes\n{}",
        census(observed)
    );

    let root = repo_root();
    // Scan until a document is found whose declaration is actually LOAD-BEARING. Several contracts
    // define `IdempotencyKey` under `components:` and never `$ref` it from an operation; stripping
    // one of those is correctly a no-op, and treating the first document that merely MENTIONS the
    // token as proof would have made this test assert something the kernel never claimed.
    let mut proved: Option<(String, usize, usize)> = None;
    for relative in tracked_files(&root).expect("git ls-files") {
        if !(relative.ends_with(".yaml") || relative.ends_with(".yml")) {
            continue;
        }
        let Ok(bytes) = std::fs::read(root.join(&relative)) else {
            continue;
        };
        let contents = String::from_utf8_lossy(&bytes).into_owned();
        if !is_openapi(&contents) {
            continue;
        }
        let doc = OpenApiDocument {
            path: relative.clone(),
            microservice: capability_of(&relative),
            contents: contents.clone(),
        };
        let (_, before) = audit_all(vec![doc]);
        // Remove every idempotency declaration from this document, in memory only.
        let stripped: String = contents
            .lines()
            .filter(|line| {
                let lower = line.to_ascii_lowercase();
                !(lower.contains("idempotency-key") || lower.contains("idempotencykey"))
            })
            .collect::<Vec<_>>()
            .join("\n");
        if stripped.len() == contents.len() {
            continue; // this document declared no key; nothing to strip
        }
        let (_, after) = audit_all(vec![OpenApiDocument {
            path: relative.clone(),
            microservice: capability_of(&relative),
            contents: stripped,
        }]);
        if after.len() > before.len() {
            proved = Some((relative, before.len(), after.len()));
            break;
        }
    }
    let (document, before, after) = proved.expect(
        "stripping Idempotency-Key from EVERY live OpenAPI document left the finding count \
         unchanged. Either no operation in this tree is actually covered, or the kernel has \
         stopped reading the documents this gate hands it — both mean every green above is \
         vacuous.",
    );
    println!(
        "mutation proof: stripping Idempotency-Key from {document} raised findings {before} -> \
         {after}"
    );
}

/// Evidence, always printed, so a reader can tell a repaired corpus from a collapsed walk without
/// re-running anything.
#[test]
fn live_census_is_reported() {
    let (_, observed) = live();
    println!("{}", census(observed));
    assert!(observed.report.documents_checked > 0);
}
