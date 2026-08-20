// governance-check-cursor-pagination-coverage live-tree gate (ADR-0150).
//
// The kernel is pure; this is the CALLER that walks the real repository and hands it observations
// as DATA. Until this file existed the doctrine had never produced a verdict about this codebase:
// the crate's only consumer was marketplace/facade/dev-cli, which no workflow invokes, so every
// case it had ever run was a hand-written fixture in `src/lib.rs` — four documents this file's
// author typed. Those fixture tests are still there and are still the proof that the kernel is
// correct; this file is the separate proof that the kernel has ever been pointed at the tree.
//
// Walk failures are ERRORS, never omitted observations. A document dropped from the census because
// its metadata or contents failed to read would quietly shrink the frozen set, and a shrink reads
// as repair.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use check_cursor_pagination_coverage::{CoverageReport, OpenApiDocument, audit_all};

const POLICY_PATH: &str =
    "governance/check/cursor-pagination-coverage/cursor-pagination-coverage-policy.json";
const MAX_SCANNED_BYTES: u64 = 4_194_304;

/// The two shapes ADR-0150 forbids, as the kernel words them. Matched on the kernel's own message
/// text so a kernel that renamed a finding breaks this caller loudly instead of silently sorting
/// every finding into the wrong arm.
const MISSING_CURSOR: &str = "list operation missing cursor + page_size parameters";
const OFFSET_OR_PAGE: &str = "offset/page pagination is FORBIDDEN";

/// How many drifted keys a failure message names before it truncates. Long enough that the
/// ordinary one-or-two-key drift is always fully named, short enough that a corpus-wide collapse
/// does not bury the census line under a thousand paths.
const REPORT_LIMIT: usize = 40;

struct Policy {
    min_tracked_files: usize,
    min_openapi_documents: usize,
    min_get_operations: usize,
    min_list_operations: usize,
    min_capabilities: usize,
    frozen_list_ops_missing_cursor: BTreeSet<String>,
    frozen_offset_or_page_operations: BTreeSet<String>,
}

struct Observed {
    report: CoverageReport,
    /// One key per collection-shaped `get:` that declares no `cursor` + `page_size` pair:
    /// `<document path>::<route>`.
    ///
    /// Deliberately NOT line-anchored. A line number moves whenever anything above the operation
    /// is edited, so a line-keyed baseline forces a blind re-freeze on edits that changed nothing
    /// about pagination — and an edit above a guarded construct can leave no legal edit at all.
    /// A route hosts at most one `get:` per document, so the route is the finest key that is
    /// stable, and `observe` REFUSES to fold two findings onto one key rather than assume it.
    list_ops_missing_cursor: BTreeSet<String>,
    /// One key per `get:` operation declaring a forbidden `offset` / `page` query parameter:
    /// `<document path>::<route>::<parameter>`.
    ///
    /// This arm is EMPTY today and is pinned by EQUALITY against an empty frozen set — never by a
    /// floor. A floor on a term whose honest target is zero reds the moment the corpus is clean,
    /// which is the shape of guard that gets deleted for crying wolf.
    offset_or_page_operations: BTreeSet<String>,
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
    let set = |key: &str| -> BTreeSet<String> {
        doc[key]
            .as_array()
            .unwrap_or_else(|| panic!("policy field {key} missing or not an array"))
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .unwrap_or_else(|| panic!("{key} holds a non-string"))
                    .to_owned()
            })
            .collect()
    };
    Policy {
        min_tracked_files: number("min_tracked_files"),
        min_openapi_documents: number("min_openapi_documents"),
        min_get_operations: number("min_get_operations"),
        min_list_operations: number("min_list_operations"),
        min_capabilities: number("min_capabilities"),
        frozen_list_ops_missing_cursor: set("frozen_list_ops_missing_cursor"),
        frozen_offset_or_page_operations: set("frozen_offset_or_page_operations"),
    }
}

/// The tracked file list, from git — the same corpus boundary every other live gate here uses.
///
/// Reading the working tree instead would measure a different corpus than CI does the moment a
/// developer has an ignored or untracked `*.yaml` on disk, and with the sets pinned by equality
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
/// The marker is `openapi:` at column ZERO, the same test the sibling idempotency-key gate uses so
/// the two agree about what the contract corpus IS. An indented `openapi:` is a nested key inside
/// some other document (a Helm value, a catalog entry) and describes no operations; admitting
/// those would put non-contracts into a census whose floor exists to prove the contract corpus is
/// intact.
fn is_openapi(text: &str) -> bool {
    text.lines().any(|line| line.starts_with("openapi:"))
}

/// The route a line names, if it names one.
///
/// OpenAPI path keys are frequently QUOTED in this repo, because AIP-136 custom methods embed a
/// colon (`"/principals/{id}:suspend"`). A bare `starts_with('/')` test misses every one of them,
/// and the operation underneath would then be attributed to whatever route happened to appear
/// earlier in the file — distinct operations collapsing onto one key, which is exactly the
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

/// The route key a line sits under: the nearest line ABOVE it at shallower indentation that names
/// a path.
fn enclosing_route(lines: &[&str], index: usize, indent: usize) -> Option<String> {
    for line in lines[..index].iter().rev() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let line_indent = line.len() - trimmed.len();
        if line_indent >= indent {
            continue;
        }
        if let Some(route) = route_named_by(trimmed) {
            return Some(route);
        }
    }
    None
}

/// Re-derive the route identity of the line the kernel flagged.
///
/// The kernel reports a 1-indexed line and a message; it does not report the route, because it is
/// a pure line scanner with no document model. Naming the route is CALLER work by construction,
/// and it is what turns an unattributable count into a reviewable diff.
fn route_at(path: &str, text: &str, line: usize) -> Result<(String, String), String> {
    let lines: Vec<&str> = text.lines().collect();
    let index = line
        .checked_sub(1)
        .ok_or_else(|| format!("{path}: kernel reported line 0"))?;
    let raw = lines
        .get(index)
        .ok_or_else(|| format!("{path}: kernel reported line {line}, past end of document"))?;
    let trimmed = raw.trim_start();
    let indent = raw.len() - trimmed.len();
    let route = enclosing_route(&lines, index, indent).ok_or_else(|| {
        format!(
            "{path}:{line}: no enclosing path key found above this finding. Refusing to guess: an \
             operation attributed to the wrong route silently merges two frozen keys into one and \
             hides a regression."
        )
    })?;
    Ok((route, trimmed.to_owned()))
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
                "{relative} is {} bytes, over the {MAX_SCANNED_BYTES}-byte scan cap — raise the \
                 cap deliberately rather than dropping the file from the census",
                metadata.len()
            ));
        }
        let bytes = std::fs::read(&path).map_err(|e| format!("read {relative} failed: {e}"))?;
        // LOSSY, never skipped: latin-1 and UTF-16 both carry ASCII `openapi:` and `get:` fine.
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

    let mut list_ops_missing_cursor = BTreeSet::new();
    let mut offset_or_page_operations = BTreeSet::new();
    for finding in &findings {
        let contents = by_path
            .get(&finding.path)
            .ok_or_else(|| format!("kernel reported {} which was never supplied", finding.path))?;
        let (route, line_text) = route_at(&finding.path, contents, finding.line)?;
        let (bucket, key) = if finding.message.starts_with(MISSING_CURSOR) {
            (
                &mut list_ops_missing_cursor,
                format!("{}::{route}", finding.path),
            )
        } else if finding.message.starts_with(OFFSET_OR_PAGE) {
            // `name: offset` / `name: page` — the parameter name is the whole distinguishing
            // fact, and a single operation may declare both.
            let parameter = line_text
                .split_once(':')
                .map_or(line_text.clone(), |(_, value)| value.trim().to_owned());
            (
                &mut offset_or_page_operations,
                format!("{}::{route}::{parameter}", finding.path),
            )
        } else {
            return Err(format!(
                "unrecognised kernel finding message '{}' at {}:{}. The caller sorts findings by \
                 the kernel's own wording; an unsorted message would land in neither frozen set \
                 and become invisible.",
                finding.message, finding.path, finding.line
            ));
        };
        if !bucket.insert(key.clone()) {
            return Err(format!(
                "{key} was produced twice. Two operations collapsed onto one frozen key, so one of \
                 them is invisible to the ratchet — disambiguate the key before re-freezing."
            ));
        }
    }

    Ok(Observed {
        report,
        list_ops_missing_cursor,
        offset_or_page_operations,
        tracked_files: tracked.len(),
    })
}

/// The live walk, done ONCE for the whole binary: it is a pure function of the tree, so every test
/// re-walking it would recompute the same answer over ~14k tracked paths.
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
        "census: {} OpenAPI documents over {} tracked files, {} get operations of which {} are \
         collection-shaped, {} capabilities; {} list operations without cursor + page_size, {} \
         forbidden offset/page parameters\n",
        observed.report.documents_checked,
        observed.tracked_files,
        observed.report.get_ops_checked,
        observed.report.list_ops_checked,
        observed.report.microservices_audited,
        observed.list_ops_missing_cursor.len(),
        observed.offset_or_page_operations.len(),
    )
}

fn drift(
    label: &str,
    frozen: &BTreeSet<String>,
    observed: &BTreeSet<String>,
    repair: &str,
) -> String {
    let appeared: Vec<&String> = observed.difference(frozen).collect();
    let gone: Vec<&String> = frozen.difference(observed).collect();
    let mut message = String::new();
    if !appeared.is_empty() {
        message.push_str(&format!(
            "{} NEW {label} finding(s), not in the frozen set. {repair}\n",
            appeared.len()
        ));
        for key in appeared.iter().take(REPORT_LIMIT) {
            message.push_str(&format!("  + {key}\n"));
        }
        if appeared.len() > REPORT_LIMIT {
            message.push_str(&format!("  … {} more\n", appeared.len() - REPORT_LIMIT));
        }
    }
    if !gone.is_empty() {
        message.push_str(&format!(
            "{} frozen {label} finding(s) are no longer produced. If you fixed them, strike these \
             exact lines from the policy in THIS change so the win is recorded. If you did not, \
             the scan narrowed and is now reporting green over documents it stopped reading.\n",
            gone.len()
        ));
        for key in gone.iter().take(REPORT_LIMIT) {
            message.push_str(&format!("  - {key}\n"));
        }
        if gone.len() > REPORT_LIMIT {
            message.push_str(&format!("  … {} more\n", gone.len() - REPORT_LIMIT));
        }
    }
    message
}

/// ANTI-VACUITY, asserted before any equality below is read.
///
/// A set-equality ratchet cannot distinguish "the corpus was repaired" from "the walk collapsed
/// and saw nothing" — both drive the observed sets toward empty. These floors are the machine
/// oracle that separates them. Every floor counts SUBJECTS — documents, operations, capabilities,
/// tracked paths — and never findings, so declaring `cursor` + `page_size` on a collection
/// endpoint moves the FROZEN SET and leaves every floor where it is. No floor here can red on
/// honest progress: `min_list_operations` is the only one a repair touches at all, and adding a
/// cursor parameter can only raise it, because the kernel treats a declared cursor parameter as
/// evidence that the operation is collection-shaped.
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
        observed.report.get_ops_checked >= policy.min_get_operations,
        "{} get operations parsed, below the floor of {} — the operation extractor stopped \
         matching, and an extractor that finds no operations reports a perfectly paginated API it \
         never read\n{}",
        observed.report.get_ops_checked,
        policy.min_get_operations,
        census(observed)
    );
    assert!(
        observed.report.list_ops_checked >= policy.min_list_operations,
        "{} collection-shaped get operations recognised, below the floor of {} — the list-shape \
         heuristic stopped matching, which silently empties the subject of this entire doctrine\n{}",
        observed.report.list_ops_checked,
        policy.min_list_operations,
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

/// THE GATE, arm one: a SHRINK-ONLY, TWO-SIDED ratchet on the SET of collection endpoints that
/// declare no cursor pagination.
///
/// A SET, not a count. A count tells a reviewer that the number moved and nothing about which
/// endpoint moved, so re-freezing it is a blind edit; a set diff names the route that gained or
/// lost `cursor` + `page_size` and is reviewable on its face.
///
/// TWO-SIDED. A new uncovered collection endpoint appears in `observed - frozen` and blocks:
/// ADR-0150 is mandatory, so a new list endpoint without cursor pagination is born-blocking. A
/// repaired one appears in `frozen - observed` and ALSO blocks until struck in the SAME change,
/// because a one-sided ceiling cannot tell debt-paid-off from a scan that collapsed, and the
/// second is the failure this whole programme exists to remove.
#[test]
fn list_operations_missing_cursor_equal_the_frozen_set() {
    let (policy, observed) = live();
    let message = drift(
        "missing-cursor",
        &policy.frozen_list_ops_missing_cursor,
        &observed.list_ops_missing_cursor,
        "ADR-0150 is mandatory: declare `cursor` and `page_size` query parameters (inline or by \
         $ref to the shared components) rather than baselining the endpoint.",
    );
    assert!(message.is_empty(), "{message}{}", census(observed));
}

/// THE GATE, arm two: offset/page pagination is FORBIDDEN, and the live corpus obeys — today.
///
/// Pinned by EQUALITY against an empty frozen set, and deliberately NOT by a floor. A floor on a
/// term whose honest target is zero goes red exactly when the doctrine succeeds, and a guard that
/// reds on success is deleted rather than obeyed. Equality against empty is the right shape: it
/// costs nothing while the corpus is clean, it names the offending parameter the moment one
/// appears, and if someone ever does baseline one here, the same two-sided rule forces it back out
/// the moment it is repaired.
///
/// This arm is NOT vacuous: its subject is the same 100+ document, 300+ operation corpus the arm
/// above reads, and `injecting_each_forbidden_shape_into_a_real_contract_reddens_the_kernel` proves
/// on live document text that the arm still fires.
#[test]
fn offset_or_page_parameters_equal_the_frozen_set() {
    let (policy, observed) = live();
    let message = drift(
        "offset/page",
        &policy.frozen_offset_or_page_operations,
        &observed.offset_or_page_operations,
        "ADR-0150 FORBIDS offset/page pagination outright — this is not baseline-able debt; \
         replace the parameter with `cursor` + `page_size`.",
    );
    assert!(message.is_empty(), "{message}{}", census(observed));
}

/// The gate is DEMONSTRATED CAPABLE OF FAILING against the REAL corpus, not just fixtures.
///
/// A green ratchet proves nothing on its own: a caller that silently produced zero findings would
/// satisfy every assertion above by reporting a perfectly paginated API. Both defect shapes are
/// injected into the text of a REAL, currently-CLEAN contract and the finding count must rise —
/// which also proves the clean side of the corpus is real rather than unread.
///
/// The injected text is deliberately mechanical (`name: offset`, an `items:`/`next_cursor`
/// envelope with no cursor parameters) and carries NO prose describing the violation, because the
/// kernel is a line scanner: a probe whose comment spelled out the defect could satisfy the very
/// rule it was meant to break, and the run would go green for the wrong reason.
#[test]
fn injecting_each_forbidden_shape_into_a_real_contract_reddens_the_kernel() {
    let (_, observed) = live();
    let root = repo_root();

    let mut proved_missing_cursor: Option<(String, usize, usize)> = None;
    let mut proved_offset: Option<(String, usize, usize)> = None;
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
        let document = |body: String| OpenApiDocument {
            path: relative.clone(),
            microservice: capability_of(&relative),
            contents: body,
        };
        let (_, before) = audit_all(vec![document(contents.clone())]);

        let listing = format!(
            "{contents}\n  /oya-lane-c-probe-listing:\n    get:\n      responses:\n        \
             '200':\n          schema:\n            properties:\n              items: {{}}\n      \
             next_cursor: {{}}\n"
        );
        let (_, after_listing) = audit_all(vec![document(listing)]);
        if after_listing.len() > before.len() && proved_missing_cursor.is_none() {
            proved_missing_cursor = Some((relative.clone(), before.len(), after_listing.len()));
        }

        let offset = format!(
            "{contents}\n  /oya-lane-c-probe-offset:\n    get:\n      parameters:\n        - in: \
             query\n          name: offset\n          schema: {{type: integer}}\n      responses: \
             {{}}\n"
        );
        let (_, after_offset) = audit_all(vec![document(offset)]);
        if after_offset.len() > before.len() && proved_offset.is_none() {
            proved_offset = Some((relative.clone(), before.len(), after_offset.len()));
        }
        if proved_missing_cursor.is_some() && proved_offset.is_some() {
            break;
        }
    }

    let (listing_doc, listing_before, listing_after) = proved_missing_cursor.expect(
        "appending a collection-shaped get with no cursor parameters to EVERY live OpenAPI \
         document left the finding count unchanged — the kernel has stopped reading the documents \
         this gate hands it, so every green above is vacuous",
    );
    let (offset_doc, offset_before, offset_after) = proved_offset.expect(
        "appending an `offset` query parameter to EVERY live OpenAPI document left the finding \
         count unchanged — the FORBIDDEN arm cannot fire, so its empty frozen set is not evidence",
    );

    // The plant must be found by the SAME caller path the gate uses, under the SAME key shape, or
    // the count rose for some reason that has nothing to do with the injected route.
    let planted = std::fs::read(root.join(&offset_doc)).expect("re-read probe document");
    let planted = String::from_utf8_lossy(&planted).into_owned();
    let planted = format!(
        "{planted}\n  /oya-lane-c-probe-offset:\n    get:\n      parameters:\n        - in: \
         query\n          name: offset\n          schema: {{type: integer}}\n      responses: \
         {{}}\n"
    );
    let (_, findings) = audit_all(vec![OpenApiDocument {
        path: offset_doc.clone(),
        microservice: capability_of(&offset_doc),
        contents: planted.clone(),
    }]);
    let named = findings.iter().any(|finding| {
        finding.message.starts_with(OFFSET_OR_PAGE)
            && route_at(&offset_doc, &planted, finding.line)
                .is_ok_and(|(route, _)| route == "/oya-lane-c-probe-offset")
    });
    assert!(
        named,
        "the finding count rose but no finding resolves to the injected route \
         /oya-lane-c-probe-offset — the mutation passed for the wrong reason"
    );

    println!(
        "mutation proof: appending a cursor-less collection get to {listing_doc} raised findings \
         {listing_before} -> {listing_after}; appending `name: offset` to {offset_doc} raised \
         findings {offset_before} -> {offset_after}, attributed to the injected route\n{}",
        census(observed)
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
