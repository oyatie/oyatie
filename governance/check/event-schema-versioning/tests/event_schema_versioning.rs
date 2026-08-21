// governance-check-event-schema-versioning live-tree gate (ADR-0154).
//
// The kernel is pure; this is the CALLER that walks the real repository and hands it observations
// as DATA. Before this file existed the doctrine had never produced a verdict about this codebase:
// the crate's only consumer was marketplace/facade/dev-cli, which no workflow invokes, so every
// case it had ever run was a hand-written fixture — and the AsyncAPI corpus turns out to be 87%
// non-compliant.
//
// Walk failures are ERRORS, never omitted observations: a document dropped from the census because
// its contents failed to read would quietly shrink the frozen set, and a shrink reads as repair.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use check_event_schema_versioning::{AsyncApiDocument, Report, audit_all};

const POLICY_PATH: &str =
    "governance/check/event-schema-versioning/event-schema-versioning-policy.json";
const MAX_SCANNED_BYTES: u64 = 4_194_304;

struct Policy {
    min_tracked_files: usize,
    min_asyncapi_documents: usize,
    min_documents_with_version_field: usize,
    min_capabilities: usize,
    frozen_unversioned_documents: BTreeSet<String>,
}

struct Observed {
    report: Report,
    /// One key per AsyncAPI document that declares no canonical event `version` header. The
    /// document PATH is the key: an AsyncAPI description is versioned or it is not, so the file is
    /// the finest honest granularity the kernel supports, and it is stable under every edit that
    /// does not move or rename the contract.
    unversioned: BTreeSet<String>,
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
    Policy {
        min_tracked_files: number("min_tracked_files"),
        min_asyncapi_documents: number("min_asyncapi_documents"),
        min_documents_with_version_field: number("min_documents_with_version_field"),
        min_capabilities: number("min_capabilities"),
        frozen_unversioned_documents: doc["frozen_unversioned_documents"]
            .as_array()
            .expect("policy field frozen_unversioned_documents missing or not an array")
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .expect("frozen_unversioned_documents holds a non-string")
                    .to_owned()
            })
            .collect(),
    }
}

/// The tracked file list, from git — the same corpus boundary every other live gate here uses.
/// Walking the working tree instead would measure a different corpus than CI does the moment an
/// ignored `*.yaml` exists on disk, and with the set pinned by equality that is a red CI cannot
/// reproduce.
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

fn capability_of(relative: &str) -> String {
    relative
        .split_once("/contracts/")
        .map_or_else(
            || relative.split('/').next().unwrap_or(relative),
            |(head, _)| head,
        )
        .to_owned()
}

/// Is this document an AsyncAPI description?
///
/// The marker is `asyncapi:` at column ZERO. The kernel itself only checks that the token appears
/// ANYWHERE, which is right for a kernel handed a curated batch and wrong for a corpus walk: an
/// indented `asyncapi:` is a nested key in some catalog or Helm value, describes no messages, and
/// would enter the census as permanent unrepairable debt.
fn is_asyncapi(text: &str) -> bool {
    text.lines().any(|line| line.starts_with("asyncapi:"))
}

fn observe(root: &Path) -> Result<Observed, String> {
    let tracked = tracked_files(root)?;
    let mut documents: Vec<AsyncApiDocument> = Vec::new();
    for relative in &tracked {
        if !(relative.ends_with(".yaml") || relative.ends_with(".yml")) {
            continue;
        }
        let path = root.join(relative);
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
        // LOSSY, never skipped: a non-UTF-8 payload still carries ASCII `asyncapi:` perfectly well.
        let contents = String::from_utf8_lossy(&bytes).into_owned();
        if !is_asyncapi(&contents) {
            continue;
        }
        documents.push(AsyncApiDocument {
            path: relative.clone(),
            microservice: capability_of(relative),
            contents,
        });
    }
    documents.sort_by(|a, b| a.path.cmp(&b.path));

    let (report, findings) = audit_all(documents);
    let mut unversioned = BTreeSet::new();
    for finding in &findings {
        if !unversioned.insert(finding.path.clone()) {
            return Err(format!(
                "{} was reported twice; the same document entered the census under one key more \
                 than once, so one report is invisible to the ratchet",
                finding.path
            ));
        }
    }

    Ok(Observed {
        report,
        unversioned,
        tracked_files: tracked.len(),
    })
}

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
        "census: {} AsyncAPI documents over {} tracked files, {} declaring a canonical `version` \
         header, {} capabilities; {} unversioned\n",
        observed.report.documents_checked,
        observed.tracked_files,
        observed.report.documents_with_version_field,
        observed.report.microservices_audited,
        observed.unversioned.len(),
    )
}

/// ANTI-VACUITY, asserted before any equality below is read.
///
/// A set-equality ratchet cannot distinguish "the corpus was repaired" from "the walk collapsed";
/// both drive the observed set toward empty. These floors are the machine oracle that separates
/// them. None of them can red on honest progress: repairing an unversioned contract RAISES
/// `documents_with_version_field` and leaves the other three untouched.
#[test]
fn the_asyncapi_corpus_is_intact() {
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
        observed.report.documents_checked >= policy.min_asyncapi_documents,
        "{} AsyncAPI documents found, below the floor of {}. Event contracts do not disappear in \
         bulk; a drop here is a narrowed scan, and a narrowed scan reports a clean tree it never \
         read\n{}",
        observed.report.documents_checked,
        policy.min_asyncapi_documents,
        census(observed)
    );
    assert!(
        observed.report.documents_with_version_field >= policy.min_documents_with_version_field,
        "only {} documents declare a canonical `version` header, below the floor of {} — the \
         version detector stopped matching, which would report the whole corpus as debt\n{}",
        observed.report.documents_with_version_field,
        policy.min_documents_with_version_field,
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

/// THE GATE: a SHRINK-ONLY, TWO-SIDED ratchet on the SET of unversioned event contracts.
///
/// A SET, not a count. The cost of this regression is paid by CONSUMERS: an event whose schema has
/// no version header cannot be evolved compatibly, and the break surfaces at the subscriber, in
/// production, far from the change. A count would tell a reviewer only that the number moved; the
/// set names the contract.
///
/// TWO-SIDED. A new unversioned AsyncAPI document blocks — born-blocking, so the 87% debt cannot
/// grow. A repaired one ALSO blocks until struck from the policy in the SAME change, because a
/// one-sided ceiling reads a collapsed walk exactly like a repaired corpus.
#[test]
fn unversioned_documents_equal_the_frozen_set() {
    let (policy, observed) = live();

    let appeared: Vec<&String> = observed
        .unversioned
        .difference(&policy.frozen_unversioned_documents)
        .collect();
    let repaired: Vec<&String> = policy
        .frozen_unversioned_documents
        .difference(&observed.unversioned)
        .collect();

    let mut message = String::new();
    if !appeared.is_empty() {
        message.push_str(&format!(
            "{} AsyncAPI document(s) declare no canonical event `version` header and are not in \
             the frozen set. ADR-0154 requires one: declare `version` in the message `headers:` \
             block (or `event_version` / `schema_version`) rather than baselining it.\n",
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
            "{} frozen document(s) are no longer reported. If you versioned them, strike these \
             lines from `frozen_unversioned_documents` in THIS change so the win is recorded. If \
             you did not, the scan narrowed and is reporting green over contracts it stopped \
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
/// satisfy every assertion above by reporting a perfectly clean tree. This strips the version
/// declaration out of a real, currently-COMPLIANT contract and asserts the kernel notices — which
/// also proves the compliant side of the census is real rather than a detector accident.
#[test]
fn stripping_the_version_header_from_a_real_compliant_contract_reddens_the_kernel() {
    let (_, observed) = live();
    assert!(
        observed.report.documents_with_version_field > 0,
        "no compliant contract exists to mutate; the corpus is not what this test assumes\n{}",
        census(observed)
    );

    let root = repo_root();
    let mut proved: Option<(String, usize, usize)> = None;
    for relative in tracked_files(&root).expect("git ls-files") {
        if !(relative.ends_with(".yaml") || relative.ends_with(".yml")) {
            continue;
        }
        let Ok(bytes) = std::fs::read(root.join(&relative)) else {
            continue;
        };
        let contents = String::from_utf8_lossy(&bytes).into_owned();
        if !is_asyncapi(&contents) {
            continue;
        }
        let (_, before) = audit_all(vec![AsyncApiDocument {
            path: relative.clone(),
            microservice: capability_of(&relative),
            contents: contents.clone(),
        }]);
        if !before.is_empty() {
            continue; // already unversioned; stripping proves nothing
        }
        // Remove every version declaration from this document, in memory only.
        let stripped: String = contents
            .lines()
            .filter(|line| {
                let lower = line.trim_start().to_ascii_lowercase();
                !(lower.starts_with("version:")
                    || lower.starts_with("event_version")
                    || lower.starts_with("schema_version")
                    || lower.contains("pattern: \"^[0-9]+"))
            })
            .collect::<Vec<_>>()
            .join("\n");
        let (_, after) = audit_all(vec![AsyncApiDocument {
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
        "stripping the version declaration from EVERY compliant live AsyncAPI document left the \
         finding count unchanged. Either no contract in this tree is actually versioned, or the \
         kernel has stopped reading the documents this gate hands it — both mean every green \
         above is vacuous.",
    );
    println!(
        "mutation proof: stripping the version declaration from {document} raised findings \
         {before} -> {after}"
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
