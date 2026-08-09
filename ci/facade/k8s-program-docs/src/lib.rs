//! Deterministic R-DOC checks for the Kubernetes port program (ADR-0637 and ADR-0638).
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

const PROGRAM_ROOT: &str = "docs/programs/k8s-port";
/// Required line-oriented registry: `version=1`, then
/// `wave=<id>;ordinal=<u32>;completed=<true|false>;operations_entries=<operations-relative paths>;no_extraction_rationale=<text>`.
const WAVE_REGISTRY: &str = "docs/programs/k8s-port/wave-registry.rdoc";
const SIX_AXIS_TOKENS: [&str; 6] = [
    "`pin`",
    "snapshot_digest",
    "engine_digest",
    "rulepack_digest",
    "toolchain_digest",
    "formatter_digest",
];

/// Root scanned for INV-3. Every Rust file below it is read; nothing below it is written.
const OS_ROOT: &str = "os";
/// INV-3, frozen on `origin/dev` @ `5e452bd70`: 15 production sites plus one test fixture at
/// `os/core/k8s-control-domain/src/manifest_controller.rs:200`. Shrink-only — a unit that retires a
/// site lowers this constant in the same commit; no unit raises it. Hand-writing upstream Kubernetes
/// wire surface is what ADR-0704 charters as generated, so growth here is the defect.
pub const UPSTREAM_EMIT_SITE_CEILING: usize = 16;
/// The discriminator is the **API group**, never the token `apiVersion` (trap T-1): Talos emits
/// `apiVersion: v1alpha1` correctly in dozens of places, and those are Talos surface, not Kubernetes
/// surface. Bare `v1` is handled separately because it must not swallow `v1alpha1`/`v1beta1`.
const UPSTREAM_API_GROUPS: [&str; 6] = [
    "apps/v1",
    "rbac.authorization.k8s.io/v1",
    "kubelet.config.k8s.io/v1beta1",
    "apiserver.config.k8s.io/v1",
    "audit.k8s.io/v1",
    "pod-security.admission.config.k8s.io/v1",
];
const API_VERSION_MARKER: &str = "apiVersion: ";

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum FindingCode {
    EmptyProgramCorpus,
    MissingBaselineHeader,
    CompletedWaveWithoutJournal,
    RuleWithoutJournalReference,
    DoctrineWithoutAdr,
    PrescriptionStarvation,
    UpstreamKubernetesSurfaceGrowth,
}

impl FindingCode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::EmptyProgramCorpus => "R-DOC-EMPTY-PROGRAM-CORPUS",
            Self::MissingBaselineHeader => "R-DOC-BASELINE-HEADER-MISSING",
            Self::CompletedWaveWithoutJournal => "R-DOC-COMPLETED-WAVE-JOURNAL-MISSING",
            Self::RuleWithoutJournalReference => "R-DOC-RULE-JOURNAL-REFERENCE-MISSING",
            Self::DoctrineWithoutAdr => "R-DOC-DOCTRINE-ADR-OVERDUE",
            Self::PrescriptionStarvation => "R-DOC-PRESCRIPTION-STARVATION",
            Self::UpstreamKubernetesSurfaceGrowth => "R-DOC-OS-UPSTREAM-K8S-SURFACE-GROWTH",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    pub code: FindingCode,
    pub path: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Counters {
    pub scanned_population: usize,
    pub finding_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evaluation {
    pub counters: Counters,
    pub findings: Vec<Finding>,
}

impl Evaluation {
    pub fn is_green(&self) -> bool {
        self.findings.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkdownDocument {
    pub path: String,
    pub contents: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JournalEntry {
    pub id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletedWave {
    pub id: String,
    pub ordinal: u32,
    pub journal_entries: Vec<JournalEntry>,
    pub no_extraction_rationale: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuleKind {
    Neutral,
    Corpus,
}

impl RuleKind {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Corpus => "corpus",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleRecord {
    pub path: String,
    pub id: String,
    pub kind: RuleKind,
    pub operations_journal_reference: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoctrineRecord {
    pub path: String,
    pub id: String,
    pub authored_wave: u32,
    pub binding_outside_lane: bool,
    pub adr_reference: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Corpus {
    pub documents: Vec<MarkdownDocument>,
    pub completed_waves: Vec<CompletedWave>,
    pub rules: Vec<RuleRecord>,
    pub doctrine: Vec<DoctrineRecord>,
    pub prescription_count: usize,
    /// INV-3 observed value: upstream-Kubernetes `apiVersion:` emit sites across `os/**/*.rs`.
    pub upstream_emit_sites: usize,
    /// Denominator for the line above. A zero here means the scan found nothing to read, which is a
    /// false green, so the loader refuses it rather than reporting `0 <= 16`.
    pub os_rust_files: usize,
}

pub fn evaluate(corpus: &Corpus) -> Evaluation {
    let mut findings = Vec::new();
    if corpus.documents.is_empty() {
        findings.push(finding(
            FindingCode::EmptyProgramCorpus,
            PROGRAM_ROOT,
            "no program Markdown documents were scanned",
        ));
    }

    for document in &corpus.documents {
        if !has_baseline_header(&document.contents) {
            findings.push(finding(
                FindingCode::MissingBaselineHeader,
                &document.path,
                "required Baseline version header lacks a repository baseline, upstream baseline, or six-axis tuple token",
            ));
        }
    }

    for wave in &corpus.completed_waves {
        if wave.journal_entries.is_empty() {
            findings.push(finding(
                FindingCode::CompletedWaveWithoutJournal,
                &wave.id,
                "completed wave has no non-empty operations-journal entry",
            ));
        }
    }

    for rule in &corpus.rules {
        if !non_empty(rule.operations_journal_reference.as_deref()) {
            findings.push(finding(
                FindingCode::RuleWithoutJournalReference,
                &rule.path,
                &format!(
                    "{} rule '{}' has no operations-journal reference",
                    rule.kind.as_str(),
                    rule.id
                ),
            ));
        }
    }

    let newest_completed_wave = corpus.completed_waves.iter().map(|wave| wave.ordinal).max();
    if let Some(newest_completed_wave) = newest_completed_wave {
        for doctrine in &corpus.doctrine {
            if doctrine.binding_outside_lane
                && newest_completed_wave.saturating_sub(doctrine.authored_wave) > 1
                && !non_empty(doctrine.adr_reference.as_deref())
            {
                findings.push(finding(
                    FindingCode::DoctrineWithoutAdr,
                    &doctrine.path,
                    &format!(
                        "doctrine '{}' is older than one completed wave without an ADR reference",
                        doctrine.id
                    ),
                ));
            }
        }
    }

    if corpus.prescription_count == 0 {
        let mut waves = corpus.completed_waves.iter().collect::<Vec<_>>();
        waves.sort_by_key(|wave| (wave.ordinal, &wave.id));
        for pair in waves.windows(2) {
            let earlier = pair[0];
            let later = pair[1];
            if !earlier.journal_entries.is_empty()
                && !later.journal_entries.is_empty()
                && !non_empty(later.no_extraction_rationale.as_deref())
            {
                findings.push(finding(
                    FindingCode::PrescriptionStarvation,
                    &later.id,
                    &format!(
                        "operations journal grew at completed gates '{}' and '{}' while prescriptions remain empty",
                        earlier.id, later.id
                    ),
                ));
            }
        }
    }

    if corpus.upstream_emit_sites > UPSTREAM_EMIT_SITE_CEILING {
        findings.push(finding(
            FindingCode::UpstreamKubernetesSurfaceGrowth,
            OS_ROOT,
            &format!(
                "{} upstream-Kubernetes apiVersion emit sites across {} Rust files exceeds the shrink-only ceiling of {}; the surface is chartered as generated, so consume the seam instead of hand-writing it",
                corpus.upstream_emit_sites, corpus.os_rust_files, UPSTREAM_EMIT_SITE_CEILING
            ),
        ));
    }

    findings.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.message.cmp(&right.message))
    });
    let counters = Counters {
        scanned_population: corpus.documents.len()
            + corpus.completed_waves.len()
            + corpus.rules.len()
            + corpus.doctrine.len()
            + corpus.prescription_count
            + corpus.os_rust_files,
        finding_count: findings.len(),
    };
    Evaluation { counters, findings }
}

pub fn load_repository(repo_root: &Path) -> Result<Corpus, LoadError> {
    load_repository_inner(repo_root).map_err(|mut error| {
        error.path = display_path(repo_root, Path::new(&error.path));
        error
    })
}

fn load_repository_inner(repo_root: &Path) -> Result<Corpus, LoadError> {
    let program_root = repo_root.join(PROGRAM_ROOT);
    let documents = load_markdown_documents(repo_root, &program_root)?;
    if documents.is_empty() {
        return Ok(Corpus::default());
    }
    let completed_waves = load_wave_registry(repo_root, &program_root)?;
    let rules = load_rule_records(repo_root)?;
    let doctrine = load_doctrine_records(repo_root, &program_root)?;
    let prescription_count = count_lane_entries(repo_root, &program_root.join("prescriptions"))?;
    let (upstream_emit_sites, os_rust_files) = scan_os_upstream_emit_sites(repo_root)?;
    Ok(Corpus {
        documents,
        completed_waves,
        rules,
        doctrine,
        prescription_count,
        upstream_emit_sites,
        os_rust_files,
    })
}

/// Counts INV-3 emit sites over `os/**/*.rs`, returning `(sites, files_read)`.
///
/// One line counts at most once, matching the section-8 `git grep -cE` reproducer, which counts
/// matching lines rather than matches.
fn scan_os_upstream_emit_sites(repo_root: &Path) -> Result<(usize, usize), LoadError> {
    let root = repo_root.join(OS_ROOT);
    let mut sites = 0;
    let mut files = 0;
    for path in collect_files(&root, "R-DOC-OS-TREE-UNREADABLE")? {
        if path.extension().is_some_and(|extension| extension == "rs") {
            sites += upstream_emit_sites(&read_utf8(&path, "R-DOC-OS-SOURCE-UNREADABLE")?);
            files += 1;
        }
    }
    if files == 0 {
        return Err(load_error(
            "R-DOC-OS-TREE-EMPTY",
            &root,
            "no Rust sources were scanned; a zero denominator would report the INV-3 ceiling as satisfied without reading anything",
        ));
    }
    Ok((sites, files))
}

/// Number of lines emitting an upstream-Kubernetes `apiVersion:`.
pub fn upstream_emit_sites(contents: &str) -> usize {
    contents.lines().filter(|line| emits_upstream_group(line)).count()
}

fn emits_upstream_group(line: &str) -> bool {
    line.match_indices(API_VERSION_MARKER).any(|(index, marker)| {
        let group = &line[index + marker.len()..];
        UPSTREAM_API_GROUPS
            .iter()
            .any(|candidate| group.starts_with(candidate))
            || group.strip_prefix("v1").is_some_and(|tail| {
                // Bare core/v1 only: end of line, the closing quote of a Rust literal, or the
                // literal two-character `\n` escape inside one. `v1alpha1` and `v1beta1` fall out
                // here, which is the whole of trap T-1.
                tail.is_empty() || tail.starts_with('"') || tail.starts_with("\\n")
            })
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadError {
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

impl fmt::Display for LoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}: {}", self.code, self.path, self.message)
    }
}

impl std::error::Error for LoadError {}

fn has_baseline_header(contents: &str) -> bool {
    let mut lines = contents.lines();
    while let Some(line) = lines.next() {
        if line.trim() == "## Baseline version header" {
            let section = lines
                .by_ref()
                .take_while(|next| !next.trim_start().starts_with("## "))
                .collect::<Vec<_>>()
                .join("\n");
            return section.contains("Repository baseline")
                && (section.contains("Kubernetes upstream")
                    || section.contains("Upstream Kubernetes pin"))
                && SIX_AXIS_TOKENS.iter().all(|token| section.contains(token));
        }
    }
    false
}

fn finding(code: FindingCode, path: &str, message: &str) -> Finding {
    Finding {
        code,
        path: path.to_owned(),
        message: message.to_owned(),
    }
}

fn non_empty(value: Option<&str>) -> bool {
    value.is_some_and(|value| !value.trim().is_empty())
}

fn load_markdown_documents(
    repo_root: &Path,
    program_root: &Path,
) -> Result<Vec<MarkdownDocument>, LoadError> {
    let paths = collect_files(program_root, "R-DOC-PROGRAM-DOCUMENTS-UNREADABLE")?;
    let mut documents = Vec::new();
    for path in paths {
        if path.extension().is_some_and(|extension| extension == "md") {
            documents.push(MarkdownDocument {
                path: display_path(repo_root, &path),
                contents: read_utf8(&path, "R-DOC-PROGRAM-DOCUMENT-UNREADABLE")?,
            });
        }
    }
    for relative in [
        "docs/adr-archive/ADR-0637-owned-deterministic-go-to-rust-port-engine.md",
        "docs/adr-archive/ADR-0638-mechanically-maintained-kubernetes-rust-port.md",
    ] {
        let path = repo_root.join(relative);
        documents.push(MarkdownDocument {
            path: relative.to_owned(),
            contents: read_utf8(&path, "R-DOC-PROGRAM-DOCUMENT-UNREADABLE")?,
        });
    }
    documents.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(documents)
}

fn load_wave_registry(
    repo_root: &Path,
    program_root: &Path,
) -> Result<Vec<CompletedWave>, LoadError> {
    let path = repo_root.join(WAVE_REGISTRY);
    if !path.exists() {
        return Err(load_error(
            "R-DOC-WAVE-REGISTRY-MISSING",
            &path,
            "wave registry is required; prose cannot attest that no wave completed",
        ));
    }

    let contents = read_utf8(&path, "R-DOC-WAVE-REGISTRY-MALFORMED")?;
    let mut lines = contents
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'));
    if lines.next().map(str::trim) != Some("version=1") {
        return Err(load_error(
            "R-DOC-WAVE-REGISTRY-MALFORMED",
            &path,
            "first non-comment line must be version=1",
        ));
    }

    let mut seen_ids = BTreeSet::new();
    let mut seen_ordinals = BTreeSet::new();
    let mut waves = Vec::new();
    for line in lines {
        let fields = parse_fields(line, &path, "R-DOC-WAVE-REGISTRY-MALFORMED")?;
        ensure_only_fields(
            &fields,
            &[
                "wave",
                "ordinal",
                "completed",
                "operations_entries",
                "no_extraction_rationale",
            ],
            &path,
            "R-DOC-WAVE-REGISTRY-MALFORMED",
        )?;
        let id = required_field(&fields, "wave", &path, "R-DOC-WAVE-REGISTRY-MALFORMED")?;
        let ordinal = required_field(&fields, "ordinal", &path, "R-DOC-WAVE-REGISTRY-MALFORMED")?
            .parse::<u32>()
            .map_err(|_| {
                load_error(
                    "R-DOC-WAVE-REGISTRY-MALFORMED",
                    &path,
                    "ordinal must be an unsigned integer",
                )
            })?;
        let completed =
            required_field(&fields, "completed", &path, "R-DOC-WAVE-REGISTRY-MALFORMED")?;
        if !seen_ids.insert(id.to_owned()) || !seen_ordinals.insert(ordinal) {
            return Err(load_error(
                "R-DOC-WAVE-REGISTRY-MALFORMED",
                &path,
                "wave identifiers and ordinals must be unique",
            ));
        }
        if completed == "true" {
            let entries = fields
                .get("operations_entries")
                .map_or(Ok(Vec::new()), |value| {
                    load_journal_entries(program_root, value)
                })?;
            waves.push(CompletedWave {
                id: id.to_owned(),
                ordinal,
                journal_entries: entries,
                no_extraction_rationale: fields.get("no_extraction_rationale").cloned(),
            });
        } else if completed != "false" {
            return Err(load_error(
                "R-DOC-WAVE-REGISTRY-MALFORMED",
                &path,
                "completed must be true or false",
            ));
        }
    }
    Ok(waves)
}

fn load_journal_entries(program_root: &Path, value: &str) -> Result<Vec<JournalEntry>, LoadError> {
    let mut entries = Vec::new();
    for entry in value.split(',').filter(|entry| !entry.trim().is_empty()) {
        let entry = entry.trim();
        let relative = Path::new(entry);
        if relative.is_absolute()
            || relative
                .components()
                .any(|component| component == Component::ParentDir)
        {
            return Err(load_error(
                "R-DOC-WAVE-REGISTRY-MALFORMED",
                program_root,
                "operations_entries paths must be relative to operations/ and cannot escape it",
            ));
        }
        let path = program_root.join("operations").join(relative);
        let metadata = fs::symlink_metadata(&path).map_err(|_| {
            load_error(
                "R-DOC-JOURNAL-ENTRY-UNREADABLE",
                &path,
                "operations journal entry metadata must be readable",
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(load_error(
                "R-DOC-JOURNAL-ENTRY-UNREADABLE",
                &path,
                "operations journal entries must be regular files, not symlinks",
            ));
        }
        let contents = read_utf8(&path, "R-DOC-JOURNAL-ENTRY-UNREADABLE")?;
        if contents.trim().is_empty() {
            return Err(load_error(
                "R-DOC-JOURNAL-ENTRY-EMPTY",
                &path,
                "operations journal entry is empty",
            ));
        }
        entries.push(JournalEntry {
            id: entry.to_owned(),
        });
    }
    Ok(entries)
}

/// Every rule file under `specs/port-rules` or `specs/k8s-port/rules` is a record with
/// `rule_id`, `rule_kind: neutral|corpus`, and `operations_journal_ref` front-matter fields.
fn load_rule_records(repo_root: &Path) -> Result<Vec<RuleRecord>, LoadError> {
    let mut records = Vec::new();
    for root in [
        repo_root.join("specs/port-rules"),
        repo_root.join("specs/k8s-port/rules"),
    ] {
        if !root.exists() {
            continue;
        }
        for path in collect_files(&root, "R-DOC-RULES-UNREADABLE")? {
            if path.extension().is_none_or(|extension| extension != "md") {
                return Err(load_error(
                    "R-DOC-RULE-METADATA-MALFORMED",
                    &path,
                    "rule records must be Markdown with YAML-style front matter",
                ));
            }
            let fields = parse_front_matter(
                &read_utf8(&path, "R-DOC-RULE-UNREADABLE")?,
                &path,
                "R-DOC-RULE-METADATA-MALFORMED",
            )?;
            ensure_only_fields(
                &fields,
                &["rule_id", "rule_kind", "operations_journal_ref"],
                &path,
                "R-DOC-RULE-METADATA-MALFORMED",
            )?;
            let rule_kind =
                match required_field(&fields, "rule_kind", &path, "R-DOC-RULE-METADATA-MALFORMED")?
                {
                    "neutral" => RuleKind::Neutral,
                    "corpus" => RuleKind::Corpus,
                    _ => {
                        return Err(load_error(
                            "R-DOC-RULE-METADATA-MALFORMED",
                            &path,
                            "rule_kind must be neutral or corpus",
                        ));
                    }
                };
            records.push(RuleRecord {
                path: display_path(repo_root, &path),
                id: required_field(&fields, "rule_id", &path, "R-DOC-RULE-METADATA-MALFORMED")?
                    .to_owned(),
                kind: rule_kind,
                operations_journal_reference: fields.get("operations_journal_ref").cloned(),
            });
        }
    }
    Ok(records)
}

fn load_doctrine_records(
    repo_root: &Path,
    program_root: &Path,
) -> Result<Vec<DoctrineRecord>, LoadError> {
    let root = program_root.join("doctrine");
    let mut records = Vec::new();
    for path in collect_files(&root, "R-DOC-DOCTRINE-UNREADABLE")? {
        if path.file_name().is_some_and(|name| name == "INDEX.md") {
            continue;
        }
        if path.extension().is_none_or(|extension| extension != "md") {
            return Err(load_error(
                "R-DOC-DOCTRINE-METADATA-MALFORMED",
                &path,
                "doctrine records must be Markdown with YAML-style front matter",
            ));
        }
        let fields = parse_front_matter(
            &read_utf8(&path, "R-DOC-DOCTRINE-UNREADABLE")?,
            &path,
            "R-DOC-DOCTRINE-METADATA-MALFORMED",
        )?;
        ensure_only_fields(
            &fields,
            &[
                "doctrine_id",
                "authored_wave",
                "binding_outside_lane",
                "adr_reference",
            ],
            &path,
            "R-DOC-DOCTRINE-METADATA-MALFORMED",
        )?;
        let authored_wave = required_field(
            &fields,
            "authored_wave",
            &path,
            "R-DOC-DOCTRINE-METADATA-MALFORMED",
        )?
        .parse::<u32>()
        .map_err(|_| {
            load_error(
                "R-DOC-DOCTRINE-METADATA-MALFORMED",
                &path,
                "authored_wave must be an unsigned integer",
            )
        })?;
        let binding_outside_lane = match required_field(
            &fields,
            "binding_outside_lane",
            &path,
            "R-DOC-DOCTRINE-METADATA-MALFORMED",
        )? {
            "true" => true,
            "false" => false,
            _ => {
                return Err(load_error(
                    "R-DOC-DOCTRINE-METADATA-MALFORMED",
                    &path,
                    "binding_outside_lane must be true or false",
                ));
            }
        };
        records.push(DoctrineRecord {
            path: display_path(repo_root, &path),
            id: required_field(
                &fields,
                "doctrine_id",
                &path,
                "R-DOC-DOCTRINE-METADATA-MALFORMED",
            )?
            .to_owned(),
            authored_wave,
            binding_outside_lane,
            adr_reference: fields.get("adr_reference").cloned(),
        });
    }
    Ok(records)
}

fn count_lane_entries(_repo_root: &Path, root: &Path) -> Result<usize, LoadError> {
    Ok(collect_files(root, "R-DOC-PRESCRIPTIONS-UNREADABLE")?
        .into_iter()
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .filter(|path| path.file_name().is_none_or(|name| name != "INDEX.md"))
        .count())
}

fn parse_front_matter(
    contents: &str,
    path: &Path,
    code: &'static str,
) -> Result<BTreeMap<String, String>, LoadError> {
    let mut lines = contents.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Err(load_error(code, path, "front matter must start with ---"));
    }
    let mut fields = BTreeMap::new();
    for line in lines {
        if line.trim() == "---" {
            return Ok(fields);
        }
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| load_error(code, path, "front-matter fields must be key: value"))?;
        if key.trim().is_empty()
            || fields
                .insert(key.trim().to_owned(), value.trim().to_owned())
                .is_some()
        {
            return Err(load_error(
                code,
                path,
                "front-matter fields must have unique non-empty keys",
            ));
        }
    }
    Err(load_error(
        code,
        path,
        "front matter is missing its closing ---",
    ))
}

fn parse_fields(
    line: &str,
    path: &Path,
    code: &'static str,
) -> Result<BTreeMap<String, String>, LoadError> {
    let mut fields = BTreeMap::new();
    for field in line.split(';') {
        let (key, value) = field.split_once('=').ok_or_else(|| {
            load_error(
                code,
                path,
                "registry fields must be key=value separated by semicolons",
            )
        })?;
        if key.trim().is_empty()
            || fields
                .insert(key.trim().to_owned(), value.trim().to_owned())
                .is_some()
        {
            return Err(load_error(
                code,
                path,
                "registry fields must have unique non-empty keys",
            ));
        }
    }
    Ok(fields)
}

fn ensure_only_fields(
    fields: &BTreeMap<String, String>,
    allowed: &[&str],
    path: &Path,
    code: &'static str,
) -> Result<(), LoadError> {
    if fields
        .keys()
        .any(|field| !allowed.contains(&field.as_str()))
    {
        return Err(load_error(code, path, "metadata contains an unknown field"));
    }
    Ok(())
}

fn required_field<'a>(
    fields: &'a BTreeMap<String, String>,
    name: &str,
    path: &Path,
    code: &'static str,
) -> Result<&'a str, LoadError> {
    fields
        .get(name)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| load_error(code, path, &format!("metadata field '{name}' is required")))
}

fn collect_files(root: &Path, code: &'static str) -> Result<Vec<PathBuf>, LoadError> {
    let metadata = fs::metadata(root)
        .map_err(|_| load_error(code, root, "directory is missing or unreadable"))?;
    if !metadata.is_dir() {
        return Err(load_error(code, root, "expected a directory"));
    }
    let mut paths = Vec::new();
    collect_files_inner(root, code, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_files_inner(
    root: &Path,
    code: &'static str,
    paths: &mut Vec<PathBuf>,
) -> Result<(), LoadError> {
    let mut entries = fs::read_dir(root)
        .map_err(|_| load_error(code, root, "directory is unreadable"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| load_error(code, root, "directory entry is unreadable"))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|_| load_error(code, &path, "file metadata is unreadable"))?;
        if metadata.file_type().is_symlink() {
            return Err(load_error(
                code,
                &path,
                "symbolic links are not allowed in scanned roots",
            ));
        }
        if metadata.is_dir() {
            collect_files_inner(&path, code, paths)?;
        } else if metadata.is_file() {
            paths.push(path);
        } else {
            return Err(load_error(code, &path, "only regular files are allowed"));
        }
    }
    Ok(())
}

fn read_utf8(path: &Path, code: &'static str) -> Result<String, LoadError> {
    fs::read_to_string(path).map_err(|_| load_error(code, path, "file must be readable UTF-8"))
}

fn display_path(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn load_error(code: &'static str, path: &Path, message: &str) -> LoadError {
    LoadError {
        code,
        path: path.to_string_lossy().replace('\\', "/"),
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header_document(path: &str) -> MarkdownDocument {
        MarkdownDocument {
            path: path.to_owned(),
            contents: "# fixture\n## Baseline version header\nRepository baseline\nKubernetes upstream\n`pin` snapshot_digest engine_digest rulepack_digest toolchain_digest formatter_digest\n".to_owned(),
        }
    }

    fn live_fixture() -> Corpus {
        Corpus {
            documents: vec![header_document("docs/programs/k8s-port/README.md")],
            completed_waves: vec![CompletedWave {
                id: "W0-A".to_owned(),
                ordinal: 0,
                journal_entries: vec![JournalEntry {
                    id: "W0-A/run-1.md".to_owned(),
                }],
                no_extraction_rationale: None,
            }],
            rules: vec![RuleRecord {
                path: "specs/port-rules/fixture.md".to_owned(),
                id: "fixture".to_owned(),
                kind: RuleKind::Neutral,
                operations_journal_reference: Some("W0-A/run-1".to_owned()),
            }],
            doctrine: Vec::new(),
            prescription_count: 1,
            upstream_emit_sites: UPSTREAM_EMIT_SITE_CEILING,
            os_rust_files: 373,
        }
    }

    fn has_code(evaluation: &Evaluation, code: FindingCode) -> bool {
        evaluation
            .findings
            .iter()
            .any(|finding| finding.code == code)
    }

    #[test]
    fn baseline_header_requires_every_axis_and_both_baselines() {
        let mut corpus = live_fixture();
        corpus.documents[0].contents = "## Baseline version header\nRepository baseline\nKubernetes upstream\n`pin` snapshot_digest engine_digest rulepack_digest toolchain_digest\n".to_owned();
        let evaluation = evaluate(&corpus);
        assert!(has_code(&evaluation, FindingCode::MissingBaselineHeader));
    }

    #[test]
    fn completed_wave_without_journal_is_red_even_with_zero_findings() {
        let mut corpus = live_fixture();
        corpus.completed_waves[0].journal_entries.clear();
        let evaluation = evaluate(&corpus);
        assert!(has_code(
            &evaluation,
            FindingCode::CompletedWaveWithoutJournal
        ));
    }
    #[test]
    fn malformed_rule_metadata_is_rejected_instead_of_skipped() {
        let error = parse_front_matter(
            "---\nrule_id fixture\n---\n",
            Path::new("specs/port-rules/fixture.md"),
            "R-DOC-RULE-METADATA-MALFORMED",
        )
        .expect_err("malformed metadata must fail closed");
        assert_eq!(error.code, "R-DOC-RULE-METADATA-MALFORMED");
    }
    #[test]
    fn missing_wave_registry_is_unconditionally_rejected() {
        let root = Path::new("/definitely-missing-r-doc-repository");
        let error = load_wave_registry(root, &root.join(PROGRAM_ROOT))
            .expect_err("missing wave registry must fail closed without a prose escape");
        assert_eq!(error.code, "R-DOC-WAVE-REGISTRY-MISSING");
    }

    #[test]
    fn rule_without_journal_reference_is_red() {
        let mut corpus = live_fixture();
        corpus.rules[0].operations_journal_reference = Some("   ".to_owned());
        let evaluation = evaluate(&corpus);
        assert!(has_code(
            &evaluation,
            FindingCode::RuleWithoutJournalReference
        ));
    }

    #[test]
    fn overdue_cross_lane_doctrine_without_adr_is_red() {
        let mut corpus = live_fixture();
        corpus.completed_waves.extend([
            CompletedWave {
                id: "W0-B".to_owned(),
                ordinal: 1,
                journal_entries: vec![JournalEntry { id: "b".to_owned() }],
                no_extraction_rationale: Some("fixture".to_owned()),
            },
            CompletedWave {
                id: "W0-C".to_owned(),
                ordinal: 2,
                journal_entries: vec![JournalEntry { id: "c".to_owned() }],
                no_extraction_rationale: Some("fixture".to_owned()),
            },
        ]);
        corpus.doctrine.push(DoctrineRecord {
            path: "docs/programs/k8s-port/doctrine/D-1.md".to_owned(),
            id: "D-1".to_owned(),
            authored_wave: 0,
            binding_outside_lane: true,
            adr_reference: None,
        });
        let evaluation = evaluate(&corpus);
        assert!(has_code(&evaluation, FindingCode::DoctrineWithoutAdr));
    }

    #[test]
    fn empty_prescriptions_after_two_live_completed_gates_is_red_unless_later_gate_explains_it() {
        let mut corpus = live_fixture();
        corpus.prescription_count = 0;
        corpus.completed_waves.push(CompletedWave {
            id: "W0-B".to_owned(),
            ordinal: 1,
            journal_entries: vec![JournalEntry {
                id: "W0-B/run-1.md".to_owned(),
            }],
            no_extraction_rationale: None,
        });
        assert!(has_code(
            &evaluate(&corpus),
            FindingCode::PrescriptionStarvation
        ));
        corpus.completed_waves[1].no_extraction_rationale =
            Some("no incident class repeated".to_owned());
        assert!(!has_code(
            &evaluate(&corpus),
            FindingCode::PrescriptionStarvation
        ));
    }

    #[test]
    fn true_zero_document_scan_is_red_and_counters_are_distinct() {
        let evaluation = evaluate(&Corpus::default());
        assert_eq!(evaluation.counters.scanned_population, 0);
        assert_eq!(evaluation.counters.finding_count, 1);
        assert!(has_code(&evaluation, FindingCode::EmptyProgramCorpus));
    }

    #[test]
    fn live_scan_with_zero_findings_is_green() {
        let evaluation = evaluate(&live_fixture());
        assert!(evaluation.is_green());
        assert!(evaluation.counters.scanned_population > 0);
        assert_eq!(evaluation.counters.finding_count, 0);
    }

    #[test]
    fn upstream_emit_site_ceiling_is_shrink_only() {
        let mut corpus = live_fixture();
        assert!(!has_code(
            &evaluate(&corpus),
            FindingCode::UpstreamKubernetesSurfaceGrowth
        ));
        corpus.upstream_emit_sites = UPSTREAM_EMIT_SITE_CEILING - 1;
        assert!(!has_code(
            &evaluate(&corpus),
            FindingCode::UpstreamKubernetesSurfaceGrowth
        ));
        corpus.upstream_emit_sites = UPSTREAM_EMIT_SITE_CEILING + 1;
        assert!(has_code(
            &evaluate(&corpus),
            FindingCode::UpstreamKubernetesSurfaceGrowth
        ));
    }

    #[test]
    fn the_api_group_is_the_discriminator_not_the_token_api_version() {
        // Every group in the section-3.2 table, one per line, plus the two bare-`v1` spellings the
        // reproducer admits: end of line and the `\n` escape inside a Rust string literal.
        let upstream = concat!(
            "apiVersion: v1\n",
            "    \"apiVersion: v1\\n\",\n",
            "        writeln!(out, \"apiVersion: v1\")?;\n",
            "apiVersion: apps/v1\n",
            "apiVersion: rbac.authorization.k8s.io/v1\n",
            "apiVersion: kubelet.config.k8s.io/v1beta1\n",
            "apiVersion: apiserver.config.k8s.io/v1\n",
            "apiVersion: audit.k8s.io/v1\n",
            "apiVersion: pod-security.admission.config.k8s.io/v1\n",
        );
        assert_eq!(upstream_emit_sites(upstream), 9);

        // Talos's own machine-config surface. These are correct Talos output, chartered by the port
        // of siderolabs/talos, and must never enter the count (trap T-1).
        let talos = concat!(
            "apiVersion: v1alpha1\n",
            "    \"apiVersion: v1alpha1\\n\",\n",
            "apiVersion: v1beta1\n",
            "apiVersion:\n",
            "let field = \"apiVersion\";\n",
        );
        assert_eq!(upstream_emit_sites(talos), 0);
    }

    #[test]
    fn one_line_counts_once_like_the_line_oriented_reproducer() {
        assert_eq!(
            upstream_emit_sites("apiVersion: apps/v1 apiVersion: audit.k8s.io/v1"),
            1
        );
    }

    #[test]
    fn live_tree_style_fixture_is_evaluated_without_filesystem_mutation() {
        let evaluation = evaluate(&live_fixture());
        assert_eq!(evaluation.findings, Vec::<Finding>::new());
        assert_eq!(evaluation.counters.finding_count, 0);
    }
}
