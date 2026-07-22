//! Authority-neutral, content-addressed ADR census receipts.
//!
//! The pure receipt builder never reads the working tree. The thin CLI adapter supplies only
//! tracked git blobs selected from an explicit, immutable commit.

use std::collections::BTreeSet;
use std::fmt;
use std::io::{Read, Write};
use std::process::{Command, Stdio};

use sha2::{Digest, Sha256};

use crate::{AdrByteSpan, AdrParseError, AdrParseInput, DOC_PARSER_VERSION, parse_adr_decision};

/// Stable selector identity for direct tracked ADR children only.
pub const SELECTOR_ID: &str = "docs-decisions-direct-adr-v1";
const PARSER_API: &str = "corpus-doc-parser::parse_adr_decision";
const DIAGNOSTIC_POLICY: &str = "first-error-only";

/// A git-backed input blob. Test callers may construct the same pure input without git.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CensusSource {
    pub kind: CensusSourceKind,
    pub path: String,
    pub blob_oid: String,
    pub bytes: Vec<u8>,
}

/// The role an input blob plays in a census receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CensusSourceKind {
    Decision,
    Parser,
}

/// Complete immutable input to the pure census builder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CensusInput {
    pub repository_commit: String,
    pub repository_tree: String,
    pub docs_tree: String,
    pub selector_id: String,
    pub parser_sources: Vec<CensusSource>,
    pub decision_sources: Vec<CensusSource>,
}

/// Fail-closed violations before a receipt can be emitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CensusViolation {
    InvalidObjectId,
    InvalidCommit,
    SelectorPath,
    DuplicatePath,
    SourceKind,
    ParserSource,
    Git,
}

impl fmt::Display for CensusViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidObjectId => "invalid immutable git object id",
            Self::InvalidCommit => "commit selector must be exactly 40 lowercase hex characters",
            Self::SelectorPath => "source path is outside the direct ADR selector",
            Self::DuplicatePath => "selector produced duplicate source paths",
            Self::SourceKind => "source has the wrong census role",
            Self::ParserSource => "parser source set is invalid",
            Self::Git => "git object read failed",
        })
    }
}

impl std::error::Error for CensusViolation {}

/// One recorded parser failure. `raw` is diagnostic data, never interpreted as instructions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstError {
    kind: String,
    span: Option<(u64, u64)>,
    raw: String,
}

impl FirstError {
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }
    #[must_use]
    pub const fn span(&self) -> Option<(u64, u64)> {
        self.span
    }
    #[must_use]
    pub fn raw(&self) -> &str {
        &self.raw
    }
}

/// A single direct-child ADR blob and its deterministic parser outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CensusEntry {
    path: String,
    blob_oid: String,
    sha256: String,
    outcome: String,
    first_error: Option<FirstError>,
}

impl CensusEntry {
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    #[must_use]
    pub fn first_error(&self) -> Option<&FirstError> {
        self.first_error.as_ref()
    }
}

/// Deterministic, non-authoritative census receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CensusReceipt {
    repository_commit: String,
    repository_tree: String,
    docs_tree: String,
    selector_id: String,
    entries: Vec<CensusEntry>,
    aggregate_fold: String,
    canonical_digest: String,
    canonical_bytes: Vec<u8>,
}

impl CensusReceipt {
    #[must_use]
    pub fn repository_commit(&self) -> &str {
        &self.repository_commit
    }
    #[must_use]
    pub fn docs_tree(&self) -> &str {
        &self.docs_tree
    }
    #[must_use]
    pub fn selector_id(&self) -> &str {
        &self.selector_id
    }
    #[must_use]
    pub fn entries(&self) -> &[CensusEntry] {
        &self.entries
    }
    #[must_use]
    pub fn aggregate_fold(&self) -> &str {
        &self.aggregate_fold
    }
    #[must_use]
    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    #[must_use]
    pub const fn claim_ceiling(&self) -> &'static str {
        "BLOCKED/HOLD"
    }
}

/// Builds a receipt from immutable, already-selected source blobs.
///
/// # Errors
/// Returns a fail-closed violation for malformed object bindings, duplicate paths, or any source
/// outside the exact direct-child `docs/decisions/ADR-*.md` selector.
pub fn build_receipt(input: &CensusInput) -> Result<CensusReceipt, CensusViolation> {
    if !is_lower_hex_oid(&input.repository_commit)
        || !is_lower_hex_oid(&input.repository_tree)
        || !is_lower_hex_oid(&input.docs_tree)
    {
        return Err(CensusViolation::InvalidObjectId);
    }
    if input.selector_id != SELECTOR_ID {
        return Err(CensusViolation::SelectorPath);
    }
    validate_parser_sources(&input.parser_sources)?;

    let mut paths = BTreeSet::new();
    let mut sources = input.decision_sources.clone();
    sources.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
    let mut entries = Vec::with_capacity(sources.len());
    for source in sources {
        if source.kind != CensusSourceKind::Decision {
            return Err(CensusViolation::SourceKind);
        }
        if !is_direct_adr_path(&source.path) {
            return Err(CensusViolation::SelectorPath);
        }
        if !is_lower_hex_oid(&source.blob_oid) {
            return Err(CensusViolation::InvalidObjectId);
        }
        if !paths.insert(source.path.clone()) {
            return Err(CensusViolation::DuplicatePath);
        }
        entries.push(parse_entry(source));
    }

    let path_digest = sha256_hex(entries.iter().map(|entry| entry.path.as_bytes()));
    let parser_hashes: Vec<_> = input.parser_sources.iter().map(source_digest).collect();
    let entry_folds: Vec<String> = entries.iter().map(entry_json).collect();
    let aggregate_fold = sha256_hex(entry_folds.iter().map(String::as_bytes));
    let body = canonical_body(
        input,
        &path_digest,
        &parser_hashes,
        &entries,
        &aggregate_fold,
    );
    let canonical_digest = sha256_hex(std::iter::once(body.as_bytes()));
    let canonical_bytes =
        format!("{{{body},\"canonical_digest\":\"{canonical_digest}\"}}\n").into_bytes();
    Ok(CensusReceipt {
        repository_commit: input.repository_commit.clone(),
        repository_tree: input.repository_tree.clone(),
        docs_tree: input.docs_tree.clone(),
        selector_id: input.selector_id.clone(),
        entries,
        aggregate_fold,
        canonical_digest,
        canonical_bytes,
    })
}

/// Collects an exact direct-child ADR census from a 40-hex commit using only immutable git reads.
///
/// # Errors
/// Rejects symbolic revisions, non-blobs, non-UTF-8 paths, duplicate or recursive selector paths,
/// and any git failure. It never reads ambient working-tree files.
pub fn census_from_git(commit: &str) -> Result<CensusReceipt, CensusViolation> {
    if !is_lower_hex_oid(commit) {
        return Err(CensusViolation::InvalidCommit);
    }
    let commit = git_text(["rev-parse", "--verify", &format!("{commit}^{{commit}}")])?;
    if !is_lower_hex_oid(&commit) {
        return Err(CensusViolation::InvalidCommit);
    }
    let repository_tree = git_text(["rev-parse", &format!("{commit}^{{tree}}")])?;
    let docs_tree = git_text(["rev-parse", &format!("{commit}:docs/decisions")])?;
    let decision_sources = ls_tree_sources(&format!("{commit}:docs/decisions"), false)?;
    let parser_sources = ls_tree_sources(&commit, true)?
        .into_iter()
        .filter(|source| source.path.starts_with("governance/corpus/doc-parser/src/"))
        .collect();
    build_receipt(&CensusInput {
        repository_commit: commit,
        repository_tree,
        docs_tree,
        selector_id: SELECTOR_ID.to_owned(),
        parser_sources,
        decision_sources,
    })
}

fn validate_parser_sources(sources: &[CensusSource]) -> Result<(), CensusViolation> {
    if sources.is_empty() {
        return Err(CensusViolation::ParserSource);
    }
    let mut paths = BTreeSet::new();
    for source in sources {
        if source.kind != CensusSourceKind::Parser
            || !source.path.starts_with("governance/corpus/doc-parser/src/")
            || !is_lower_hex_oid(&source.blob_oid)
            || !paths.insert(source.path.clone())
        {
            return Err(CensusViolation::ParserSource);
        }
    }
    Ok(())
}

fn parse_entry(source: CensusSource) -> CensusEntry {
    let sha256 = sha256_hex(std::iter::once(source.bytes.as_slice()));
    let parsed = std::str::from_utf8(&source.bytes)
        .map_err(|_| FirstError {
            kind: "NonUtf8".to_owned(),
            span: None,
            raw: "ADR blob is not valid UTF-8".to_owned(),
        })
        .and_then(|text| {
            parse_adr_decision(&AdrParseInput::new(&source.path, text)).map_err(first_error)
        });
    match parsed {
        Ok(_) => CensusEntry {
            path: source.path,
            blob_oid: source.blob_oid,
            sha256,
            outcome: "parsed".to_owned(),
            first_error: None,
        },
        Err(error) => CensusEntry {
            path: source.path,
            blob_oid: source.blob_oid,
            sha256,
            outcome: "error".to_owned(),
            first_error: Some(error),
        },
    }
}

fn first_error(error: AdrParseError) -> FirstError {
    let (kind, span) = match &error {
        AdrParseError::MissingLeadingFrontmatter => ("MissingLeadingFrontmatter", None),
        AdrParseError::UnterminatedFrontmatter => ("UnterminatedFrontmatter", None),
        AdrParseError::DuplicateFrontmatterKey { span, .. } => {
            ("DuplicateFrontmatterKey", Some(span))
        }
        AdrParseError::InvalidFrontmatter { span, .. } => ("InvalidFrontmatter", Some(span)),
        AdrParseError::UnsupportedFrontmatterNesting { span } => {
            ("UnsupportedFrontmatterNesting", Some(span))
        }
        AdrParseError::InvalidAdrReference { span, .. } => ("InvalidAdrReference", Some(span)),
        AdrParseError::InvalidAdrId { span, .. } => ("InvalidAdrId", Some(span)),
        AdrParseError::AdrIdPathMismatch { .. } => ("AdrIdPathMismatch", None),
        AdrParseError::InvalidDate { span, .. } => ("InvalidDate", Some(span)),
        AdrParseError::InvalidSourcePath { .. } => ("InvalidSourcePath", None),
        AdrParseError::InvalidAdrHeading { span, .. } => ("InvalidAdrHeading", Some(span)),
        AdrParseError::MissingRequiredField { .. } => ("MissingRequiredField", None),
        AdrParseError::InvalidTenant { .. } => ("InvalidTenant", None),
    };
    FirstError {
        kind: kind.to_owned(),
        span: span.map(span_tuple),
        raw: error.to_string(),
    }
}

fn span_tuple(span: &AdrByteSpan) -> (u64, u64) {
    (span.start(), span.end())
}

fn source_digest(source: &CensusSource) -> String {
    format!(
        "{}:{}:{}",
        source.path,
        source.blob_oid,
        sha256_hex(std::iter::once(source.bytes.as_slice()))
    )
}

fn canonical_body(
    input: &CensusInput,
    path_digest: &str,
    parser_hashes: &[String],
    entries: &[CensusEntry],
    aggregate_fold: &str,
) -> String {
    let parser_sources = parser_hashes
        .iter()
        .map(|value| json_string(value))
        .collect::<Vec<_>>()
        .join(",");
    let entries = entries.iter().map(entry_json).collect::<Vec<_>>().join(",");
    format!(
        "\"aggregate_fold\":\"{aggregate_fold}\",\"claim_ceiling\":\"BLOCKED/HOLD\",\"diagnostic_policy\":\"{DIAGNOSTIC_POLICY}\",\"docs_tree\":\"{}\",\"entries\":[{entries}],\"parser_api\":\"{PARSER_API}\",\"parser_source_hashes\":[{parser_sources}],\"parser_version\":\"{DOC_PARSER_VERSION}\",\"repository_commit\":\"{}\",\"repository_tree\":\"{}\",\"selector\":{{\"id\":\"{}\",\"path_digest\":\"{path_digest}\"}}",
        input.docs_tree, input.repository_commit, input.repository_tree, input.selector_id,
    )
}

fn entry_json(entry: &CensusEntry) -> String {
    let error = entry.first_error.as_ref().map_or_else(
        || "null".to_owned(),
        |error| {
            let span = error.span.map_or_else(
                || "null".to_owned(),
                |(start, end)| format!("[{start},{end}]"),
            );
            format!(
                "{{\"kind\":{},\"raw\":{},\"span\":{span}}}",
                json_string(&error.kind),
                json_string(&error.raw)
            )
        },
    );
    format!(
        "{{\"blob_oid\":\"{}\",\"first_error\":{error},\"outcome\":\"{}\",\"path\":{},\"sha256\":\"{}\"}}",
        entry.blob_oid,
        entry.outcome,
        json_string(&entry.path),
        entry.sha256
    )
}

fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c <= '\u{1f}' => {
                use std::fmt::Write;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn is_direct_adr_path(path: &str) -> bool {
    path.starts_with("docs/decisions/ADR-")
        && path.ends_with(".md")
        && !path["docs/decisions/".len()..].contains('/')
        && !path.contains("..")
        && path.is_ascii()
}

fn is_lower_hex_oid(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256_hex<'a>(parts: impl Iterator<Item = &'a [u8]>) -> String {
    let mut digest = Sha256::new();
    for part in parts {
        digest.update(part);
    }
    format!("{:x}", digest.finalize())
}

fn git_text<const N: usize>(args: [&str; N]) -> Result<String, CensusViolation> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|_| CensusViolation::Git)?;
    if !output.status.success() {
        return Err(CensusViolation::Git);
    }
    String::from_utf8(output.stdout)
        .map_err(|_| CensusViolation::Git)
        .map(|text| text.trim_end_matches('\n').to_owned())
}

fn ls_tree_sources(object: &str, recursive: bool) -> Result<Vec<CensusSource>, CensusViolation> {
    let mut command = Command::new("git");
    command.args(["ls-tree", "-z", "--full-tree"]);
    if recursive {
        command.arg("-r");
    }
    command.arg(object);
    let output = command.output().map_err(|_| CensusViolation::Git)?;
    if !output.status.success() {
        return Err(CensusViolation::Git);
    }
    let mut sources = Vec::new();
    for record in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let Some(tab) = record.iter().position(|byte| *byte == b'\t') else {
            return Err(CensusViolation::Git);
        };
        let (metadata, name_with_tab) = record.split_at(tab);
        let Some(name) = name_with_tab.get(1..) else {
            return Err(CensusViolation::Git);
        };
        let metadata = std::str::from_utf8(metadata).map_err(|_| CensusViolation::Git)?;
        let name = std::str::from_utf8(name).map_err(|_| CensusViolation::SelectorPath)?;
        if !(recursive || name.starts_with("ADR-") && name.ends_with(".md") && !name.contains('/'))
        {
            continue;
        }
        let mut fields = metadata.split(' ');
        let _mode = fields.next();
        let kind = fields.next();
        let oid = fields.next();
        let Some(oid) = oid else {
            return Err(CensusViolation::Git);
        };
        if fields.next().is_some() || kind != Some("blob") || !is_lower_hex_oid(oid) {
            return Err(CensusViolation::Git);
        }
        let path = if recursive {
            name.to_owned()
        } else {
            format!("docs/decisions/{name}")
        };
        sources.push(CensusSource {
            kind: if recursive {
                CensusSourceKind::Parser
            } else {
                CensusSourceKind::Decision
            },
            path,
            blob_oid: oid.to_owned(),
            bytes: Vec::new(),
        });
    }
    let blobs = git_blobs(sources.iter().map(|source| source.blob_oid.as_str()))?;
    for (source, bytes) in sources.iter_mut().zip(blobs) {
        source.bytes = bytes;
    }
    Ok(sources)
}

fn git_blobs<'a>(oids: impl Iterator<Item = &'a str>) -> Result<Vec<Vec<u8>>, CensusViolation> {
    let oids: Vec<_> = oids.collect();
    let mut child = Command::new("git")
        .args(["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|_| CensusViolation::Git)?;
    let Some(mut stdin) = child.stdin.take() else {
        return Err(CensusViolation::Git);
    };
    let requests = oids.iter().map(|oid| (*oid).to_owned()).collect::<Vec<_>>();
    let writer = std::thread::spawn(move || -> Result<(), CensusViolation> {
        for oid in requests {
            stdin
                .write_all(oid.as_bytes())
                .and_then(|_| stdin.write_all(b"\n"))
                .map_err(|_| CensusViolation::Git)?;
        }
        Ok(())
    });
    let Some(mut stdout) = child.stdout.take() else {
        return Err(CensusViolation::Git);
    };
    let mut bytes = Vec::new();
    stdout
        .read_to_end(&mut bytes)
        .map_err(|_| CensusViolation::Git)?;
    writer.join().map_err(|_| CensusViolation::Git)??;
    if !child.wait().map_err(|_| CensusViolation::Git)?.success() {
        return Err(CensusViolation::Git);
    }
    let mut cursor = 0_usize;
    let mut blobs = Vec::with_capacity(oids.len());
    for oid in oids {
        let Some(newline) = bytes[cursor..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|offset| cursor + offset)
        else {
            return Err(CensusViolation::Git);
        };
        let header =
            std::str::from_utf8(&bytes[cursor..newline]).map_err(|_| CensusViolation::Git)?;
        let mut fields = header.split(' ');
        if fields.next() != Some(oid) || fields.next() != Some("blob") {
            return Err(CensusViolation::Git);
        }
        let Some(size) = fields.next().and_then(|value| value.parse::<usize>().ok()) else {
            return Err(CensusViolation::Git);
        };
        if fields.next().is_some() {
            return Err(CensusViolation::Git);
        }
        let body_start = newline.checked_add(1).ok_or(CensusViolation::Git)?;
        let body_end = body_start.checked_add(size).ok_or(CensusViolation::Git)?;
        let Some(body) = bytes.get(body_start..body_end) else {
            return Err(CensusViolation::Git);
        };
        if bytes.get(body_end) != Some(&b'\n') {
            return Err(CensusViolation::Git);
        }
        blobs.push(body.to_vec());
        cursor = body_end.checked_add(1).ok_or(CensusViolation::Git)?;
    }
    if cursor != bytes.len() {
        return Err(CensusViolation::Git);
    }
    Ok(blobs)
}
