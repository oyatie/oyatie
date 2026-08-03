//! cloud-ci-crate-reference-integrity — a crate reference in a governed artifact that
//! names a package which no longer exists is a dangling reference, not documentation.
//!
//! ## The defect this closes
//!
//! When a crate is renamed or relocated, references to its OLD package name survive in
//! governed artifacts that NO compiler and NO existing gate ever reads. `cargo check` is
//! green, the build graph is green, and the reference is dead. An adversarial review of
//! eight real move batches found 89 such missed references, across five structural site
//! classes:
//!
//! - JSON OBJECT KEYS — a policy map keyed BY PACKAGE NAME. The moved crate strands its
//!   justification and the consuming gate silently stops covering it.
//! - GRAPH NODE LABELS — a generated architecture graph still labelling dead packages.
//!   (A stale *path prefix* around a LIVE package name is a different defect — path
//!   liveness — and is deliberately out of scope here.)
//! - SEAM DECLARATIONS — vendor-phaseout records naming the adapter crates that implement
//!   a seam, stored as paths whose last segment is the package name.
//! - RUNNABLE COMMANDS — a `-p <package>` invocation in a task or plan document, left
//!   broken. This one is the sharpest: a reader copies it and it fails.
//! - BUILD-GRAPH LABELS — `//path:target` references to build packages whose directory no
//!   longer holds a build file.
//! - DECISION-RECORD FRONTMATTER — the `crates:` sequence a decision record declares as its
//!   affected surface, which outlives the crates it names.
//!
//! ## The discrimination that makes this a real gate
//!
//! A DATED audit snapshot is a record of what the tree looked like on a date. Rewriting
//! its crate names to match today's would FALSIFY THE RECORD, not fix a reference. So a
//! naive "every named crate must be live" check is wrong on an entire corpus class. The
//! exclusion set is therefore DATA with a MANDATORY reason, shrink-only and staleness-
//! checked exactly like the baseline — an exclusion that suppresses nothing must go.
//!
//! ## No rule may be declared without a measured floor
//!
//! An earlier draft of this gate declared one rule DORMANT with `min_sites: 0`, on a
//! measurement that had only matched at column 0 and therefore missed 25 files whose key
//! is nested and indented. A declared-but-unmeasured rule is precisely the false-green
//! shape this gate exists to stop, so the dormancy escape hatch does not exist: every rule
//! carries a floor of at least one, and a rule declared with a zero floor is itself a
//! violation ([`CODE_RULE_WITHOUT_MEASURED_FLOOR`]). If a site class genuinely has no sites
//! today, it must not be declared at all.
//!
//! ## Why this kernel is pure
//!
//! Everything here is a function of already-collected observations or of a string that was
//! handed to it: no filesystem, no clock, no environment, no repo-specific literal. Every
//! violation class is provable from in-memory fixtures, and every repo-specific string
//! lives in the two JSON files. The I/O lives in the gate test, which is the only place
//! that touches a disk or the tracked-file boundary.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

pub const GATE_ID: &str = "cloud-ci-crate-reference-integrity";

/// The known-name census is implausibly small — a collector bug must never read as clean.
pub const CODE_IMPLAUSIBLE_CRATE_CENSUS: &str = "implausible_crate_census";
/// A rule's file glob matches zero tracked files: a governed file was renamed and
/// silently zeroed the rule.
pub const CODE_RULE_GLOB_MATCHES_NOTHING: &str = "rule_glob_matches_nothing";
/// A rule's structural locator produced zero sites: a JSON pointer or object key was
/// restructured out from under it.
pub const CODE_RULE_YIELDED_NO_SITES: &str = "rule_yielded_no_sites";
/// A rule produced fewer sites than its floor — partial collapse, same class as zero.
pub const CODE_RULE_BELOW_SITE_FLOOR: &str = "rule_below_site_floor";
/// A rule is declared with a floor of zero. There is no such thing as a covered-but-unmeasured
/// site class: a zero floor can never fail, so the declaration is decoration.
pub const CODE_RULE_WITHOUT_MEASURED_FLOOR: &str = "rule_without_measured_floor";
/// An exclusion matches no tracked file, suppresses no would-be finding, or carries no
/// reason. Dead slack that outlives the corpus it was written for.
pub const CODE_STALE_EXCLUSION: &str = "stale_exclusion";
/// A frozen baseline entry no longer corresponds to a live dangling reference: the
/// subject came back to life, the reference was fixed, or the file is gone.
pub const CODE_STALE_BASELINE_ENTRY: &str = "stale_baseline_entry";
/// THE DEFECT: a reference whose subject is in neither the workspace nor the lockfile,
/// not excluded, and not baselined.
pub const CODE_DANGLING_CRATE_REFERENCE: &str = "dangling_crate_reference";

pub const VIOLATION_CODES: [&str; 8] = [
    CODE_IMPLAUSIBLE_CRATE_CENSUS,
    CODE_RULE_GLOB_MATCHES_NOTHING,
    CODE_RULE_YIELDED_NO_SITES,
    CODE_RULE_BELOW_SITE_FLOOR,
    CODE_RULE_WITHOUT_MEASURED_FLOOR,
    CODE_STALE_EXCLUSION,
    CODE_STALE_BASELINE_ENTRY,
    CODE_DANGLING_CRATE_REFERENCE,
];

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Finding {
    pub code: String,
    pub subject: String,
    pub detail: String,
}

/// How a site's subject is decided live-or-dangling.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Resolution {
    /// Live iff the subject is in the known-name census (workspace packages ∪ lockfile
    /// package names). Used by every `package_name`-shaped subject.
    AgainstKnownNames,
    /// Live iff the collector resolved it against the tracked tree. Used by build-graph
    /// labels, whose liveness is "the referenced directory holds a build file" — a
    /// filesystem fact the kernel deliberately does not reach for.
    Prevalidated(bool),
}

/// One extracted reference occurrence, already normalized to its subject.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Site {
    pub rule_id: String,
    /// Repo-relative path of the file the reference lives in. Part of the key, so one
    /// file's tolerated reference never licenses another's.
    pub file: String,
    pub subject: String,
    pub resolution: Resolution,
    /// The exclusion glob that suppressed this site, if any. Attribution is the
    /// collector's job because only it knows the tracked-file boundary.
    pub excluded_by: Option<String>,
}

/// What the collector saw for one rule.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RuleObservation {
    pub rule_id: String,
    pub sites: Vec<Site>,
    /// `file_globs` entries that matched zero tracked files.
    pub globs_matching_nothing: Vec<String>,
    /// Values at a rule's structural sites that are not package-name shaped. Load-bearing:
    /// seam fields hold free text alongside real package paths, and free text is never a
    /// finding. Counted so a collapse to "everything is free text" is visible.
    pub non_name_values_ignored: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Observed {
    pub rules: Vec<RuleObservation>,
    /// Live workspace package names ∪ `Cargo.lock` package names. The lockfile union is
    /// load-bearing: without it every third-party package named in a documented command
    /// is a false positive.
    pub known_names: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RulePolicy {
    pub rule_id: String,
    pub min_sites: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExclusionPolicy {
    pub glob: String,
    /// REQUIRED. An exclusion without a stated reason is indistinguishable from laundering.
    pub reason: String,
    /// Tracked files the glob matched, counted by the collector.
    pub tracked_files_matched: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Policy {
    pub rules: Vec<RulePolicy>,
    pub exclusions: Vec<ExclusionPolicy>,
    /// Frozen shrink-only debt, keyed `<rule_id>::<file>::<subject>`.
    pub baseline: BTreeSet<String>,
    pub min_known_names: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Clone, Debug)]
pub struct Report {
    pub verdict: Verdict,
    pub findings: Vec<Finding>,
    /// Non-excluded sites considered.
    pub sites_checked: usize,
    /// Dangling references tolerated by the frozen baseline.
    pub dangling_tolerated: usize,
    pub known_name_count: usize,
}

/// Baseline / finding key. No line numbers and no occurrence counts: both churn on
/// unrelated edits to the same file, which is the instability that makes a baseline
/// unmergeable. `rule_id` keeps a name tolerated in prose from licensing the same name in
/// a runnable command; `file` scopes debt per file.
#[must_use]
pub fn finding_key(rule_id: &str, file: &str, subject: &str) -> String {
    format!("{rule_id}::{file}::{subject}")
}

/// A package reference is name-shaped iff it is a lowercase kebab identifier. Anything
/// else at a structural site is free text, never a finding.
#[must_use]
pub fn is_package_name_shaped(value: &str) -> bool {
    !value.is_empty()
        && value.split('-').all(|seg| {
            !seg.is_empty() && seg.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

/// Extract `[package] name = "..."` from a manifest. Deliberately narrow: only the `name`
/// key inside the `[package]` table, so a `[dependencies]` or `[lib]` entry of the same
/// name cannot be mistaken for a package declaration.
#[must_use]
pub fn package_name_from_manifest(manifest: &str) -> Option<String> {
    let mut in_package = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(rest) = line.strip_prefix("name") {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix('=') {
                return Some(rest.trim().trim_matches('"').to_owned());
            }
        }
    }
    None
}

/// Every `[[package]] name` in a lockfile. Unioned into the known-name census so a
/// documented command naming a third-party package is not a false positive.
#[must_use]
pub fn lock_package_names(lock: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut in_package = false;
    for line in lock.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[[package]]";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(rest) = line.strip_prefix("name") {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix('=') {
                names.insert(rest.trim().trim_matches('"').to_owned());
            }
        }
    }
    names
}

// ---------------------------------------------------------------------------------------
// Path globbing. `*` matches within one path segment, `**` matches zero or more segments.
// Hand-rolled because the two forms above are the whole vocabulary the policy uses, and a
// glob dependency inside the gate fleet would buy nothing for four characters of syntax.
// ---------------------------------------------------------------------------------------

/// Does `path` (a `/`-separated repo-relative path) match `pattern`?
#[must_use]
pub fn path_glob_matches(pattern: &str, path: &str) -> bool {
    let pat: Vec<&str> = pattern.split('/').collect();
    let seg: Vec<&str> = path.split('/').collect();
    glob_segments(&pat, &seg)
}

fn glob_segments(pat: &[&str], seg: &[&str]) -> bool {
    match pat.split_first() {
        None => seg.is_empty(),
        Some((&"**", rest)) => {
            // Zero or more segments.
            for skip in 0..=seg.len() {
                if glob_segments(rest, &seg[skip..]) {
                    return true;
                }
            }
            false
        }
        Some((head, rest)) => match seg.split_first() {
            Some((first, seg_rest)) if glob_segment_matches(head, first) => {
                glob_segments(rest, seg_rest)
            }
            _ => false,
        },
    }
}

/// Match one path segment against one glob segment (`*` = any run, `?` = one char).
#[must_use]
pub fn glob_segment_matches(pattern: &str, name: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let n: Vec<char> = name.chars().collect();
    let (mut pi, mut ni, mut star, mut mark) = (0usize, 0usize, usize::MAX, 0usize);
    while ni < n.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == n[ni]) {
            pi += 1;
            ni += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = pi;
            mark = ni;
            pi += 1;
        } else if star != usize::MAX {
            pi = star + 1;
            mark += 1;
            ni = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

// ---------------------------------------------------------------------------------------
// Structural extraction. Every site class is one `kind` here plus one object in the policy;
// adding a site TYPE is a data edit, and only a genuinely new SHAPE needs code.
// ---------------------------------------------------------------------------------------

/// What one rule found in one file's bytes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExtractOutcome {
    /// Subjects after the rule's `subject` normalization and shape filter.
    pub subjects: Vec<String>,
    /// Values at a structural site that were not package-name shaped. Free text at a seam
    /// field is never a finding, but a collapse to "everything is free text" must be visible.
    pub non_name_values_ignored: usize,
}

fn strings_at(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) => out.push(s.clone()),
        Value::Array(items) => {
            for item in items {
                strings_at(item, out);
            }
        }
        _ => {}
    }
}

/// Resolve a JSON pointer whose segments may be `*` (each child) or a trailing `**`
/// (this node and every descendant object).
fn resolve_pointer<'a>(root: &'a Value, pointer: &str) -> Vec<&'a Value> {
    let trimmed = pointer.trim_start_matches('/');
    if trimmed.is_empty() {
        return vec![root];
    }
    let mut current: Vec<&Value> = vec![root];
    for segment in trimmed.split('/') {
        let mut next: Vec<&Value> = Vec::new();
        for node in &current {
            match segment {
                "**" => descend(node, &mut next),
                "*" => match node {
                    Value::Object(map) => next.extend(map.values()),
                    Value::Array(items) => next.extend(items.iter()),
                    _ => {}
                },
                literal => {
                    if let Some(child) = node.get(literal) {
                        next.push(child);
                    }
                }
            }
        }
        current = next;
    }
    current
}

/// `**`: this node plus every descendant OBJECT. Arrays are not emitted — every object
/// element of an array is itself a descendant, so emitting the array too would double-count.
fn descend<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
    if node.is_object() {
        out.push(node);
    }
    match node {
        Value::Object(map) => {
            for child in map.values() {
                descend(child, out);
            }
        }
        Value::Array(items) => {
            for child in items {
                descend(child, out);
            }
        }
        _ => {}
    }
}

/// The objects a resolved node contributes: itself if an object, its object elements if an
/// array.
fn objects_of<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
    match node {
        Value::Object(_) => out.push(node),
        Value::Array(items) => out.extend(items.iter().filter(|v| v.is_object())),
        _ => {}
    }
}

fn matches_where(node: &Value, condition: Option<&Value>) -> bool {
    let Some(Value::Object(map)) = condition else {
        return true;
    };
    map.iter().all(|(k, v)| node.get(k) == Some(v))
}

fn field_names(rule: &Value) -> Vec<String> {
    match &rule["field"] {
        Value::String(s) => vec![s.clone()],
        Value::Array(items) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect(),
        _ => Vec::new(),
    }
}

/// Split a markdown line on backticks so a fenced/inline code span is scanned as its own
/// chunk. Mirrors the "a command reference never spans a backtick" property.
fn code_chunks(line: &str) -> impl Iterator<Item = &str> {
    line.split('`')
}

fn clean_token(token: &str) -> &str {
    token.trim_matches(|c: char| {
        matches!(c, '"' | '\'' | '(' | ')' | '[' | ']' | ',' | ';' | '\\')
    })
}

/// `cargo [+toolchain] <command> ... -p <package>` — a RUNNABLE command left broken.
fn extract_cargo_package_flag(rule: &Value, content: &str, out: &mut Vec<String>) {
    let commands: Vec<&str> = rule["commands"]
        .as_array()
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let flags: Vec<&str> = rule["flags"]
        .as_array()
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let driver = rule["driver"].as_str().unwrap_or_default();

    for line in content.lines() {
        for chunk in code_chunks(line) {
            let tokens: Vec<&str> = chunk.split_whitespace().map(clean_token).collect();
            for (idx, token) in tokens.iter().enumerate() {
                if *token != driver {
                    continue;
                }
                let rest = &tokens[idx + 1..];
                // Skip an explicit toolchain selector (`cargo +nightly ...`).
                let rest = match rest.split_first() {
                    Some((first, tail)) if first.starts_with('+') => tail,
                    _ => rest,
                };
                if !starts_with_command(rest, &commands) {
                    continue;
                }
                collect_flag_values(rest, &flags, out);
            }
        }
    }
}

/// The tokens immediately after the driver must spell one of the declared subcommands.
/// Multi-word subcommands (`nextest run`) are declared space-separated in the policy.
fn starts_with_command(rest: &[&str], commands: &[&str]) -> bool {
    commands.iter().any(|command| {
        let words: Vec<&str> = command.split_whitespace().collect();
        rest.len() >= words.len() && rest[..words.len()] == words[..]
    })
}

fn collect_flag_values(tokens: &[&str], flags: &[&str], out: &mut Vec<String>) {
    let mut idx = 0usize;
    while idx < tokens.len() {
        let token = tokens[idx];
        for flag in flags {
            if token == *flag {
                if let Some(value) = tokens.get(idx + 1) {
                    out.push((*value).to_owned());
                }
            } else if let Some(value) = token.strip_prefix(&format!("{flag}=")) {
                out.push(value.to_owned());
            } else if flag.len() == 2 && token.len() > 2 && token.starts_with(flag) {
                // `-pfoo` — cargo accepts the attached short form.
                out.push(token[flag.len()..].to_owned());
            }
        }
        idx += 1;
    }
}

fn is_label_path_char(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '/' | '_' | '.' | '-')
}

fn is_label_target_char(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '.' | '-')
}

/// `//path/to/pkg:target` build-graph labels.
///
/// Two filters are mandatory, and both were measured: without the preceding-character
/// filter `//localhost:8080`, `//127.0.0.1:2379` and `//prometheus.svc:9090` all match;
/// without the root-dir allowlist every scheme-relative URL in prose becomes a "label".
/// The preceding-character filter is also what keeps this gate's own baseline file inert:
/// a baselined label is written `<rule>::<file>:://path:target`, and the `:` in front of
/// the `//` disqualifies it.
fn extract_build_target_labels(rule: &Value, content: &str, out: &mut Vec<String>) {
    let roots: Vec<&str> = rule["label_root_dirs"]
        .as_array()
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let chars: Vec<char> = content.chars().collect();
    let mut idx = 0usize;
    while idx + 1 < chars.len() {
        if chars[idx] != '/' || chars[idx + 1] != '/' {
            idx += 1;
            continue;
        }
        if idx > 0 {
            let prev = chars[idx - 1];
            if prev.is_ascii_alphanumeric() || matches!(prev, ':' | '.' | '-') {
                idx += 1;
                continue;
            }
        }
        let mut cursor = idx + 2;
        let start = cursor;
        while cursor < chars.len() && is_label_path_char(chars[cursor]) {
            cursor += 1;
        }
        let path: String = chars[start..cursor].iter().collect();
        if cursor >= chars.len() || chars[cursor] != ':' {
            idx += 1;
            continue;
        }
        cursor += 1;
        let target_start = cursor;
        while cursor < chars.len() && is_label_target_char(chars[cursor]) {
            cursor += 1;
        }
        let target: String = chars[target_start..cursor].iter().collect();
        idx += 1;
        if target.is_empty() || !target.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit()) {
            continue;
        }
        let segments: Vec<&str> = path.split('/').collect();
        if segments.len() < 2
            || segments.iter().any(|s| {
                s.is_empty() || !s.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
            })
        {
            continue;
        }
        let Some(first) = segments.first() else {
            continue;
        };
        if !roots.contains(first) {
            continue;
        }
        out.push(format!("//{path}:{target}"));
    }
}

/// The `crates:` sequence inside a decision record's YAML frontmatter, at ANY depth.
///
/// Structural, never regex: the key that matters in this corpus is NESTED under
/// `affected_surfaces` and therefore indented, which is exactly how a column-anchored
/// measurement concluded the field did not exist at all.
fn extract_yaml_frontmatter_fields(rule: &Value, content: &str, out: &mut Vec<String>) {
    let fields: Vec<&str> = rule["fields"]
        .as_array()
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let Some(front) = frontmatter(content) else {
        return;
    };
    let Ok(doc) = serde_yaml::from_str::<serde_yaml::Value>(front) else {
        return;
    };
    yaml_collect(&doc, &fields, out);
}

/// The `---`-delimited leading block of a markdown document.
#[must_use]
pub fn frontmatter(content: &str) -> Option<&str> {
    let rest = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

fn yaml_collect(node: &serde_yaml::Value, fields: &[&str], out: &mut Vec<String>) {
    match node {
        serde_yaml::Value::Mapping(map) => {
            for (key, value) in map {
                if key.as_str().is_some_and(|name| fields.contains(&name)) {
                    yaml_strings(value, out);
                }
                yaml_collect(value, fields, out);
            }
        }
        serde_yaml::Value::Sequence(items) => {
            for item in items {
                yaml_collect(item, fields, out);
            }
        }
        _ => {}
    }
}

fn yaml_strings(node: &serde_yaml::Value, out: &mut Vec<String>) {
    match node {
        serde_yaml::Value::String(s) => out.push(s.clone()),
        serde_yaml::Value::Sequence(items) => {
            for item in items {
                yaml_strings(item, out);
            }
        }
        _ => {}
    }
}

/// Extract every reference site one rule finds in one file's bytes. Pure: the content is
/// handed in, nothing is read.
///
/// `kind` is a closed enum of the collector SHAPES; every repo-specific string (which
/// pointer, which field, which command, which root dir) is a parameter carried by the rule
/// object, so adding a site TYPE is a policy edit and never a code change.
#[must_use]
pub fn extract_sites(rule: &Value, content: &str) -> ExtractOutcome {
    let mut raw: Vec<String> = Vec::new();
    match rule["kind"].as_str().unwrap_or_default() {
        "json_object_keys" => {
            let Ok(doc) = serde_json::from_str::<Value>(content) else {
                return ExtractOutcome::default();
            };
            let pointer = rule["json_pointer"].as_str().unwrap_or_default();
            for node in resolve_pointer(&doc, pointer) {
                if let Value::Object(map) = node {
                    raw.extend(map.keys().cloned());
                }
            }
        }
        "json_field_at_pointer" => {
            let Ok(doc) = serde_json::from_str::<Value>(content) else {
                return ExtractOutcome::default();
            };
            let pointer = rule["json_pointer"].as_str().unwrap_or_default();
            let fields = field_names(rule);
            let condition = rule.get("where");
            let mut objects: Vec<&Value> = Vec::new();
            for node in resolve_pointer(&doc, pointer) {
                objects_of(node, &mut objects);
            }
            for object in objects {
                if !matches_where(object, condition) {
                    continue;
                }
                for field in &fields {
                    if let Some(value) = object.get(field) {
                        strings_at(value, &mut raw);
                    }
                }
            }
        }
        "cargo_package_flag" => extract_cargo_package_flag(rule, content, &mut raw),
        "build_target_label" => extract_build_target_labels(rule, content, &mut raw),
        "yaml_frontmatter_field" => extract_yaml_frontmatter_fields(rule, content, &mut raw),
        _ => {}
    }

    let mut outcome = ExtractOutcome::default();
    let subject_kind = rule["subject"].as_str().unwrap_or_default();
    for value in raw {
        let candidate = match subject_kind {
            "package_name_last_path_segment" => {
                value.rsplit('/').next().unwrap_or_default().to_owned()
            }
            _ => value,
        };
        // A build-graph label is not name-shaped and is never meant to be: it resolves against
        // the tree, not the name census, so the shape filter must not touch it.
        if subject_kind == "build_target_label" || is_package_name_shaped(&candidate) {
            outcome.subjects.push(candidate);
        } else {
            outcome.non_name_values_ignored += 1;
        }
    }
    outcome
}

fn site_is_live(site: &Site, known: &BTreeSet<String>) -> bool {
    match site.resolution {
        Resolution::AgainstKnownNames => known.contains(&site.subject),
        Resolution::Prevalidated(resolves) => resolves,
    }
}

/// Evaluate reference integrity. Pure: no I/O, no clock, no environment.
#[must_use]
pub fn evaluate(observed: &Observed, policy: &Policy) -> Report {
    let mut findings: Vec<Finding> = Vec::new();

    // ANTI-VACUITY 1: the census itself. Every `package_name` verdict is relative to it,
    // so a broken census turns the whole gate into noise or into silence.
    if observed.known_names.len() < policy.min_known_names {
        findings.push(Finding {
            code: CODE_IMPLAUSIBLE_CRATE_CENSUS.to_owned(),
            subject: format!("{} known names", observed.known_names.len()),
            detail: format!(
                "the known-name census holds {} names, below the floor of {}. Workspace \
                 resolution or lockfile parsing is broken. A shrunken census makes LIVE crates \
                 look dangling and an empty one makes every reference look fine — either way \
                 this is a gate failure, never a verdict.",
                observed.known_names.len(),
                policy.min_known_names
            ),
        });
    }

    let by_id: BTreeMap<&str, &RuleObservation> = observed
        .rules
        .iter()
        .map(|r| (r.rule_id.as_str(), r))
        .collect();

    for rule in &policy.rules {
        let observation = by_id.get(rule.rule_id.as_str()).copied();

        if let Some(obs) = observation {
            for glob in &obs.globs_matching_nothing {
                findings.push(Finding {
                    code: CODE_RULE_GLOB_MATCHES_NOTHING.to_owned(),
                    subject: format!("{}::{glob}", rule.rule_id),
                    detail: format!(
                        "rule `{}` declares file glob `{glob}`, which matches ZERO tracked files. \
                         The governed artifact was renamed or deleted and the rule now covers \
                         nothing while still reporting clean. Repoint the glob at the artifact's \
                         new path, or retire the rule as a reviewed policy edit.",
                        rule.rule_id
                    ),
                });
            }
        }

        // ANTI-VACUITY 2: there is no such thing as a declared-but-unmeasured site class.
        if rule.min_sites == 0 {
            findings.push(Finding {
                code: CODE_RULE_WITHOUT_MEASURED_FLOOR.to_owned(),
                subject: rule.rule_id.clone(),
                detail: format!(
                    "rule `{}` declares a floor of ZERO, which no observation can ever fall below. \
                     A rule that cannot fail is decoration, and a site class declared without \
                     measuring it is the precise false-green this gate exists to stop: the last \
                     rule declared that way was believed to have no sites because the measurement \
                     matched at column 0 while the real key is nested and indented. Measure the \
                     rule against the live corpus and set `min_sites` to what it actually yields, \
                     or do not declare the rule at all.",
                    rule.rule_id
                ),
            });
        }

        // Exclusions are applied BEFORE the floors, so a broad exclusion that empties a
        // corpus trips a floor instead of buying a green.
        let sites =
            observation.map_or(0, |o| o.sites.iter().filter(|s| s.excluded_by.is_none()).count());

        if sites == 0 && rule.min_sites > 0 {
            findings.push(Finding {
                code: CODE_RULE_YIELDED_NO_SITES.to_owned(),
                subject: rule.rule_id.clone(),
                detail: format!(
                    "rule `{}` produced ZERO sites against a floor of {}. Its glob resolved but its \
                     structural locator found nothing — a JSON pointer, object key, or frontmatter \
                     field was restructured. The rule is blind and reporting clean.",
                    rule.rule_id, rule.min_sites
                ),
            });
        } else if sites < rule.min_sites {
            findings.push(Finding {
                code: CODE_RULE_BELOW_SITE_FLOOR.to_owned(),
                subject: rule.rule_id.clone(),
                detail: format!(
                    "rule `{}` produced {sites} sites, below its floor of {}. A partial collapse of \
                     the locator is the same class of blindness as a total one; if the corpus \
                     genuinely shrank, lower the floor as the SUBJECT of a reviewed change.",
                    rule.rule_id, rule.min_sites
                ),
            });
        }
    }

    // Per-site resolution. Findings are keyed and deduplicated: a name repeated in one
    // file is one debt, not N.
    let mut live_dangling_keys: BTreeSet<String> = BTreeSet::new();
    let mut new_dangling: BTreeMap<String, Site> = BTreeMap::new();
    let mut suppressed_by_glob: BTreeMap<&str, usize> = BTreeMap::new();
    let mut sites_checked = 0usize;
    let mut tolerated_keys: BTreeSet<String> = BTreeSet::new();

    for obs in &observed.rules {
        for site in &obs.sites {
            if site_is_live(site, &observed.known_names) {
                if site.excluded_by.is_none() {
                    sites_checked += 1;
                }
                continue;
            }
            if let Some(glob) = &site.excluded_by {
                *suppressed_by_glob.entry(glob.as_str()).or_default() += 1;
                continue;
            }
            sites_checked += 1;
            let key = finding_key(&site.rule_id, &site.file, &site.subject);
            live_dangling_keys.insert(key.clone());
            if policy.baseline.contains(&key) {
                tolerated_keys.insert(key);
            } else {
                new_dangling.entry(key).or_insert_with(|| site.clone());
            }
        }
    }

    for (key, site) in &new_dangling {
        findings.push(Finding {
            code: CODE_DANGLING_CRATE_REFERENCE.to_owned(),
            subject: key.clone(),
            detail: format!(
                "`{}` in `{}` names a package that exists in neither the workspace nor the \
                 lockfile. No compiler reads this site, so nothing else will ever report it. \
                 There are exactly three legitimate resolutions and only the author can choose: \
                 REPOINT it to the crate's new name, DELETE it because the crate is gone for \
                 good, or RECOGNIZE the site as a dated historical record — in which case add its \
                 corpus to `exclusions` with the reason, because rewriting a dated record \
                 falsifies it rather than fixing it.",
                site.subject, site.file
            ),
        });
    }

    // Exclusions are shrink-only and staleness-checked, exactly like the baseline.
    for exclusion in &policy.exclusions {
        if exclusion.reason.trim().is_empty() {
            findings.push(Finding {
                code: CODE_STALE_EXCLUSION.to_owned(),
                subject: exclusion.glob.clone(),
                detail: format!(
                    "exclusion `{}` carries no reason. The reason IS the discrimination this gate \
                     makes — without it there is no way to tell a dated record that must not be \
                     rewritten from debt being laundered.",
                    exclusion.glob
                ),
            });
            continue;
        }
        if exclusion.tracked_files_matched == 0 {
            findings.push(Finding {
                code: CODE_STALE_EXCLUSION.to_owned(),
                subject: exclusion.glob.clone(),
                detail: format!(
                    "exclusion `{}` matches ZERO tracked files. The corpus it was written for is \
                     gone; remove the entry in the same change, or the slack outlives the debt.",
                    exclusion.glob
                ),
            });
            continue;
        }
        if suppressed_by_glob
            .get(exclusion.glob.as_str())
            .copied()
            .unwrap_or(0)
            == 0
        {
            findings.push(Finding {
                code: CODE_STALE_EXCLUSION.to_owned(),
                subject: exclusion.glob.clone(),
                detail: format!(
                    "exclusion `{}` suppresses ZERO would-be findings. It is dead slack: it buys \
                     nothing today but silently licenses everything that lands under it tomorrow. \
                     Remove it.",
                    exclusion.glob
                ),
            });
        }
    }

    // Baseline fidelity: every frozen key must still produce its exact live finding.
    for key in &policy.baseline {
        if !live_dangling_keys.contains(key) {
            findings.push(Finding {
                code: CODE_STALE_BASELINE_ENTRY.to_owned(),
                subject: key.clone(),
                detail: format!(
                    "baselined dangling reference `{key}` produces no matching live finding: the \
                     subject came back to life, the reference was fixed, or the file was deleted \
                     or renamed. Remove the entry in the SAME change — a baseline that outlives \
                     its debt is pre-authorized slack for the next regression."
                ),
            });
        }
    }

    findings.sort();
    let verdict = if findings.is_empty() {
        Verdict::Green
    } else {
        Verdict::Red
    };
    Report {
        verdict,
        findings,
        sites_checked,
        dangling_tolerated: tolerated_keys.len(),
        known_name_count: observed.known_names.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn site(rule: &str, file: &str, subject: &str) -> Site {
        Site {
            rule_id: rule.to_owned(),
            file: file.to_owned(),
            subject: subject.to_owned(),
            resolution: Resolution::AgainstKnownNames,
            excluded_by: None,
        }
    }

    fn observed(rule: &str, sites: Vec<Site>, known: &[&str]) -> Observed {
        Observed {
            rules: vec![RuleObservation {
                rule_id: rule.to_owned(),
                sites,
                ..RuleObservation::default()
            }],
            known_names: known.iter().map(|n| (*n).to_owned()).collect(),
        }
    }

    fn rule(rule_id: &str, min_sites: usize) -> RulePolicy {
        RulePolicy {
            rule_id: rule_id.to_owned(),
            min_sites,
        }
    }

    fn policy(rules: Vec<RulePolicy>) -> Policy {
        Policy {
            rules,
            ..Policy::default()
        }
    }

    #[test]
    fn live_reference_is_green() {
        let o = observed("r", vec![site("r", "a.md", "live-crate")], &["live-crate"]);
        let report = evaluate(&o, &policy(vec![rule("r", 1)]));
        assert_eq!(report.verdict, Verdict::Green, "{:?}", report.findings);
        assert_eq!(report.sites_checked, 1);
    }

    /// THE DEFECT: a name in neither the workspace nor the lockfile.
    #[test]
    fn dangling_reference_fails_closed() {
        let o = observed("r", vec![site("r", "a.md", "moved-away-crate")], &["live-crate"]);
        let report = evaluate(&o, &policy(vec![rule("r", 1)]));
        assert_eq!(report.verdict, Verdict::Red);
        assert_eq!(report.findings[0].code, CODE_DANGLING_CRATE_REFERENCE);
        // The remedy must name all THREE resolutions, or a reader "fixes" a dated record.
        assert!(report.findings[0].detail.contains("REPOINT"));
        assert!(report.findings[0].detail.contains("DELETE"));
        assert!(report.findings[0].detail.contains("falsifies"));
    }

    /// The lockfile union: without it, a documented command naming a third-party package
    /// is a false positive. This is the single most likely way the gate becomes noise.
    #[test]
    fn lockfile_name_is_live_even_though_it_is_not_a_workspace_member() {
        let o = observed("r", vec![site("r", "a.md", "some-vendored-dep")], &["some-vendored-dep"]);
        assert_eq!(evaluate(&o, &policy(vec![rule("r", 1)])).verdict, Verdict::Green);
    }

    /// Build-graph labels resolve against the tree, not the name census.
    #[test]
    fn prevalidated_subject_bypasses_the_name_census() {
        let dead = Site {
            resolution: Resolution::Prevalidated(false),
            ..site("labels", "a.md", "//x/y:z")
        };
        let alive = Site {
            resolution: Resolution::Prevalidated(true),
            ..site("labels", "a.md", "//p/q:r")
        };
        let o = observed("labels", vec![dead, alive], &[]);
        let report = evaluate(&o, &policy(vec![rule("labels", 1)]));
        assert_eq!(report.verdict, Verdict::Red);
        assert_eq!(report.findings.len(), 1);
        assert!(report.findings[0].subject.ends_with("//x/y:z"));
    }

    #[test]
    fn baselined_reference_is_tolerated_and_counted() {
        let o = observed("r", vec![site("r", "a.md", "gone")], &[]);
        let mut p = policy(vec![rule("r", 1)]);
        p.baseline.insert("r::a.md::gone".to_owned());
        let report = evaluate(&o, &p);
        assert_eq!(report.verdict, Verdict::Green, "{:?}", report.findings);
        assert_eq!(report.dangling_tolerated, 1);
    }

    /// Per-file, per-rule scoping: one file's tolerated reference must not license the
    /// same name in another file, nor the same name at a different site class.
    #[test]
    fn baseline_is_scoped_per_file_and_per_rule() {
        let o = observed(
            "r",
            vec![site("r", "a.md", "gone"), site("r", "b.md", "gone")],
            &[],
        );
        let mut p = policy(vec![rule("r", 1)]);
        p.baseline.insert("r::a.md::gone".to_owned());
        let report = evaluate(&o, &p);
        assert_eq!(report.verdict, Verdict::Red);
        assert_eq!(report.findings.len(), 1);
        assert!(report.findings[0].subject.starts_with("r::b.md::"));
    }

    /// A name repeated in one file is ONE debt entry, not N. Occurrence counts churn on
    /// unrelated edits, which is what makes a baseline unmergeable.
    #[test]
    fn repeated_reference_in_one_file_is_one_finding() {
        let o = observed(
            "r",
            vec![site("r", "a.md", "gone"), site("r", "a.md", "gone")],
            &[],
        );
        let report = evaluate(&o, &policy(vec![rule("r", 1)]));
        assert_eq!(report.findings.len(), 1);
    }

    #[test]
    fn stale_baseline_entry_fails_closed() {
        let o = observed("r", vec![site("r", "a.md", "now-live")], &["now-live"]);
        let mut p = policy(vec![rule("r", 1)]);
        p.baseline.insert("r::a.md::now-live".to_owned());
        let report = evaluate(&o, &p);
        assert_eq!(report.verdict, Verdict::Red);
        assert_eq!(report.findings[0].code, CODE_STALE_BASELINE_ENTRY);
    }

    #[test]
    fn excluded_site_is_suppressed_and_keeps_its_exclusion_alive() {
        let excluded = Site {
            excluded_by: Some("dated/**".to_owned()),
            ..site("r", "dated/x.md", "gone")
        };
        let o = observed("r", vec![site("r", "a.md", "live"), excluded], &["live"]);
        let mut p = policy(vec![rule("r", 1)]);
        p.exclusions.push(ExclusionPolicy {
            glob: "dated/**".to_owned(),
            reason: "dated audit snapshot; rewriting it would falsify the record".to_owned(),
            tracked_files_matched: 1,
        });
        assert_eq!(evaluate(&o, &p).verdict, Verdict::Green);
    }

    #[test]
    fn exclusion_that_suppresses_nothing_is_stale() {
        let o = observed("r", vec![site("r", "a.md", "live")], &["live"]);
        let mut p = policy(vec![rule("r", 1)]);
        p.exclusions.push(ExclusionPolicy {
            glob: "dated/**".to_owned(),
            reason: "was needed once".to_owned(),
            tracked_files_matched: 3,
        });
        let report = evaluate(&o, &p);
        assert_eq!(report.verdict, Verdict::Red);
        assert_eq!(report.findings[0].code, CODE_STALE_EXCLUSION);
        assert!(report.findings[0].detail.contains("ZERO would-be findings"));
    }

    #[test]
    fn exclusion_without_a_reason_is_stale() {
        let excluded = Site {
            excluded_by: Some("dated/**".to_owned()),
            ..site("r", "dated/x.md", "gone")
        };
        let o = observed("r", vec![excluded], &[]);
        let mut p = policy(vec![rule("r", 1)]);
        p.exclusions.push(ExclusionPolicy {
            glob: "dated/**".to_owned(),
            reason: "   ".to_owned(),
            tracked_files_matched: 1,
        });
        let report = evaluate(&o, &p);
        assert!(report
            .findings
            .iter()
            .any(|f| f.code == CODE_STALE_EXCLUSION));
    }

    /// ANTI-VACUITY: a broad exclusion must not be able to buy a green by emptying a
    /// corpus. Floors are computed over NON-excluded sites, so the rule trips instead.
    #[test]
    fn exclusion_cannot_buy_vacuity_it_trips_the_floor_instead() {
        let excluded = Site {
            excluded_by: Some("**".to_owned()),
            ..site("r", "x.md", "gone")
        };
        let o = observed("r", vec![excluded], &[]);
        let mut p = policy(vec![rule("r", 5)]);
        p.exclusions.push(ExclusionPolicy {
            glob: "**".to_owned(),
            reason: "over-broad".to_owned(),
            tracked_files_matched: 1,
        });
        let report = evaluate(&o, &p);
        assert_eq!(report.verdict, Verdict::Red);
        assert_eq!(report.findings[0].code, CODE_RULE_YIELDED_NO_SITES);
    }

    #[test]
    fn glob_matching_nothing_fails_closed() {
        let o = Observed {
            rules: vec![RuleObservation {
                rule_id: "r".to_owned(),
                sites: vec![site("r", "a.md", "live")],
                globs_matching_nothing: vec!["specs/renamed-away.json".to_owned()],
                non_name_values_ignored: 0,
            }],
            known_names: ["live".to_owned()].into_iter().collect(),
        };
        let report = evaluate(&o, &policy(vec![rule("r", 1)]));
        assert_eq!(report.verdict, Verdict::Red);
        assert_eq!(report.findings[0].code, CODE_RULE_GLOB_MATCHES_NOTHING);
    }

    #[test]
    fn rule_below_its_floor_fails_closed() {
        let o = observed("r", vec![site("r", "a.md", "live")], &["live"]);
        let report = evaluate(&o, &policy(vec![rule("r", 9)]));
        assert_eq!(report.verdict, Verdict::Red);
        assert_eq!(report.findings[0].code, CODE_RULE_BELOW_SITE_FLOOR);
    }

    /// The lesson from the correction that produced this gate: a rule declared with a zero
    /// floor is unmeasured, and an unmeasured rule cannot fail. It is RED on sight.
    #[test]
    fn zero_floor_rule_is_red_even_when_it_has_sites() {
        let o = observed("r", vec![site("r", "a.md", "live")], &["live"]);
        let report = evaluate(&o, &policy(vec![rule("r", 0)]));
        assert_eq!(report.verdict, Verdict::Red);
        assert_eq!(report.findings[0].code, CODE_RULE_WITHOUT_MEASURED_FLOOR);
        assert!(report.findings[0].detail.contains("column 0"));
    }

    #[test]
    fn implausible_census_fails_rather_than_reporting_clean() {
        let o = observed("r", vec![site("r", "a.md", "live")], &["live"]);
        let mut p = policy(vec![rule("r", 1)]);
        p.min_known_names = 850;
        let report = evaluate(&o, &p);
        assert_eq!(report.verdict, Verdict::Red);
        assert_eq!(report.findings[0].code, CODE_IMPLAUSIBLE_CRATE_CENSUS);
    }

    #[test]
    fn findings_are_deterministically_ordered() {
        let o = observed(
            "r",
            vec![site("r", "z.md", "gone"), site("r", "a.md", "gone")],
            &[],
        );
        let report = evaluate(&o, &policy(vec![rule("r", 1)]));
        let subjects: Vec<&str> = report.findings.iter().map(|f| f.subject.as_str()).collect();
        assert_eq!(subjects, vec!["r::a.md::gone", "r::z.md::gone"]);
    }

    #[test]
    fn name_shape_filter_rejects_free_text() {
        assert!(is_package_name_shaped("oya-thing-app"));
        assert!(is_package_name_shaped("serde2"));
        assert!(!is_package_name_shaped("GitHub (current impl)"));
        assert!(!is_package_name_shaped("* (planned)"));
        assert!(!is_package_name_shaped("Trailing-"));
        assert!(!is_package_name_shaped("Upper-Case"));
        assert!(!is_package_name_shaped(""));
        assert!(!is_package_name_shaped("has_underscore"));
    }

    #[test]
    fn manifest_parser_reads_only_the_package_table() {
        let manifest = "[package]\nname = \"real-crate\"\n\n[lib]\nname = \"real_crate\"\n";
        assert_eq!(package_name_from_manifest(manifest), Some("real-crate".to_owned()));
        assert_eq!(
            package_name_from_manifest("[dependencies]\nname = \"not-a-package\"\n"),
            None
        );
    }

    #[test]
    fn lock_parser_reads_only_package_tables() {
        let lock = "[[package]]\nname = \"alpha\"\nversion = \"1\"\n\n[metadata]\nname = \"nope\"\n\n[[package]]\nname = \"beta\"\n";
        let names = lock_package_names(lock);
        assert!(names.contains("alpha") && names.contains("beta"));
        assert!(!names.contains("nope"));
    }

    #[test]
    fn path_globs_span_segments_only_through_double_star() {
        assert!(path_glob_matches("**/*.md", "README.md"));
        assert!(path_glob_matches("**/*.md", "a/b/c.md"));
        assert!(!path_glob_matches("**/*.md", "a/b/c.json"));
        assert!(path_glob_matches("docs/decisions/ADR-*.md", "docs/decisions/ADR-0630-x.md"));
        assert!(!path_glob_matches("docs/decisions/ADR-*.md", "docs/ADR-0630-x.md"));
        assert!(path_glob_matches("dated/**", "dated/a/b.md"));
        assert!(!path_glob_matches("dated/**", "other/a.md"));
        assert!(path_glob_matches("specs/x.json", "specs/x.json"));
    }

    // -- extraction: one pure test per collector shape, fixtures as literals ------------

    #[test]
    fn json_object_keys_reads_the_map_key_as_the_package_name() {
        let rule = json!({
            "kind": "json_object_keys",
            "json_pointer": "/justified_crates/*",
            "subject": "package_name"
        });
        let content = r#"{"justified_crates":{"axum":{"live-crate":"why","dead-crate":"why"}}}"#;
        let mut subjects = extract_sites(&rule, content).subjects;
        subjects.sort();
        assert_eq!(subjects, vec!["dead-crate".to_owned(), "live-crate".to_owned()]);
    }

    #[test]
    fn json_field_at_pointer_honours_the_where_filter() {
        let rule = json!({
            "kind": "json_field_at_pointer",
            "json_pointer": "/nodes",
            "where": { "kind": "crate" },
            "field": "label",
            "subject": "package_name"
        });
        let content = r#"{"nodes":[
            {"id":"crates/a","kind":"crate","label":"crate-a"},
            {"id":"svc","kind":"microservice","label":"not-a-crate-node"}
        ]}"#;
        let out = extract_sites(&rule, content);
        assert_eq!(out.subjects, vec!["crate-a".to_owned()]);
    }

    #[test]
    fn json_field_anywhere_takes_the_last_path_segment_and_ignores_free_text() {
        let rule = json!({
            "kind": "json_field_at_pointer",
            "json_pointer": "/**",
            "field": ["seam_adapter_trait", "seam_adapter_impls"],
            "subject": "package_name_last_path_segment"
        });
        let content = r#"{"entries":[
            {"seam_adapter_trait":"legacy/path/crates/seam-kernel",
             "seam_adapter_impls":["a/b/impl-one","GitHub (current impl)","* (planned)"]}
        ]}"#;
        let out = extract_sites(&rule, content);
        assert_eq!(out.subjects, vec!["seam-kernel".to_owned(), "impl-one".to_owned()]);
        assert_eq!(out.non_name_values_ignored, 2);
    }

    #[test]
    fn cargo_package_flag_finds_runnable_commands_only() {
        let rule = json!({
            "kind": "cargo_package_flag",
            "driver": "cargo",
            "commands": ["check", "build", "clippy", "test", "nextest run"],
            "flags": ["-p", "--package"],
            "subject": "package_name"
        });
        let content = "\
run `cargo check -p alpha` first
then cargo +nightly nextest run -p beta
and cargo test --package gamma
attached: cargo build -pdelta
not a command: the cargo -p story
also fine: cargo publish -p epsilon
";
        let out = extract_sites(&rule, content);
        assert_eq!(
            out.subjects,
            vec![
                "alpha".to_owned(),
                "beta".to_owned(),
                "gamma".to_owned(),
                "delta".to_owned()
            ],
            "only declared subcommands count; `publish` and prose are not runnable checks"
        );
    }

    #[test]
    fn build_target_labels_reject_host_ports_and_unknown_roots() {
        let rule = json!({
            "kind": "build_target_label",
            "label_root_dirs": ["ci", "libs"],
            "subject": "build_target_label"
        });
        let content = "\
target //ci/facade/thing:thing-gate and //libs/a/b:c
url //localhost:8080 and //127.0.0.1:2379 and //prometheus.svc:9090
scheme https://example.com:443 stays out
foreign root //vendor/x:y stays out
a baselined key rule::file.md:://ci/facade/thing:thing-gate stays out
";
        let out = extract_sites(&rule, content);
        assert_eq!(
            out.subjects,
            vec!["//ci/facade/thing:thing-gate".to_owned(), "//libs/a/b:c".to_owned()]
        );
    }

    /// The correction that produced this gate: the key is NESTED, so a column-anchored
    /// match sees nothing. Structural parsing finds it at any depth.
    #[test]
    fn yaml_frontmatter_field_is_found_at_any_depth() {
        let rule = json!({
            "kind": "yaml_frontmatter_field",
            "fields": ["crates"],
            "subject": "package_name"
        });
        let content = "\
---
id: ADR-0374
affected_surfaces:
  crates: [nested-indented-crate]
  microservices: [ci-webhook-gateway]
---

# body

crates: [not-frontmatter]
";
        let out = extract_sites(&rule, content);
        assert_eq!(out.subjects, vec!["nested-indented-crate".to_owned()]);
    }

    #[test]
    fn frontmatter_is_only_the_leading_block() {
        assert_eq!(frontmatter("---\na: 1\n---\nbody\n"), Some("a: 1"));
        assert_eq!(frontmatter("no frontmatter\n"), None);
    }

    /// The test that stops a code being unreachable: one `Observed` that reaches ALL of
    /// them, asserting the emitted set equals the registered set exactly.
    #[test]
    fn every_emitted_code_is_registered() {
        let excluded = Site {
            excluded_by: Some("kept/**".to_owned()),
            ..site("floor", "kept/x.md", "gone")
        };
        let observed = Observed {
            rules: vec![
                RuleObservation {
                    rule_id: "floor".to_owned(),
                    sites: vec![site("floor", "a.md", "gone"), excluded],
                    globs_matching_nothing: vec!["dead-glob".to_owned()],
                    non_name_values_ignored: 0,
                },
                RuleObservation {
                    rule_id: "empty".to_owned(),
                    ..RuleObservation::default()
                },
                RuleObservation {
                    rule_id: "unmeasured".to_owned(),
                    sites: vec![site("unmeasured", "b.md", "also-gone")],
                    ..RuleObservation::default()
                },
            ],
            known_names: BTreeSet::new(),
        };
        let policy = Policy {
            rules: vec![rule("floor", 900), rule("empty", 1), rule("unmeasured", 0)],
            exclusions: vec![
                ExclusionPolicy {
                    glob: "kept/**".to_owned(),
                    reason: "dated record".to_owned(),
                    tracked_files_matched: 1,
                },
                ExclusionPolicy {
                    glob: "dead/**".to_owned(),
                    reason: String::new(),
                    tracked_files_matched: 0,
                },
            ],
            baseline: ["floor::vanished.md::gone".to_owned()].into_iter().collect(),
            min_known_names: 850,
        };
        let report = evaluate(&observed, &policy);
        for f in &report.findings {
            assert!(VIOLATION_CODES.contains(&f.code.as_str()), "unregistered {}", f.code);
        }
        let codes: BTreeSet<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();
        assert_eq!(
            codes.len(),
            VIOLATION_CODES.len(),
            "every registered code must be reachable; reached {codes:?}"
        );
    }
}
