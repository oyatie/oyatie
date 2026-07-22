//! Typed, fail-closed ADR frontmatter parser.
//!
//! This module intentionally implements the small metadata grammar used by ADRs,
//! rather than accepting a general YAML dialect. Frontmatter is untrusted data;
//! parsing never reads files or resolves references.
//!
//! ## Admission boundary
//!
//! This IR is a dormant candidate foundation. The current ADR population is not
//! fully accepted by this strict grammar, and no live consumer has been cut over
//! to it. A successfully parsed subset is never sufficient evidence of corpus
//! completeness or current authority. Unsupported generic nesting fails closed
//! so that no metadata can disappear silently. `HOLD(Planning)` remains in force
//! until the population, migration, projection, and independent review gates in
//! `../CORPUS-MIGRATION.md` are satisfied by their qualified owners.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;

use sha2::{Digest, Sha256};

/// Input for parsing one ADR source document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrParseInput {
    source_path: String,
    source: String,
}

impl AdrParseInput {
    #[must_use]
    pub fn new(source_path: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            source_path: source_path.into(),
            source: source.into(),
        }
    }

    #[must_use]
    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }
}

/// A byte range into the original, unmodified source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdrByteSpan {
    start: u64,
    end: u64,
}

impl AdrByteSpan {
    #[must_use]
    pub const fn start(self) -> u64 {
        self.start
    }
    #[must_use]
    pub const fn end(self) -> u64 {
        self.end
    }
}

/// Parsed scalar or flat list frontmatter value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdrFrontmatterValue {
    Scalar(String),
    List(Vec<String>),
    Null,
    Empty,
}

/// A retained frontmatter field, including its source spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrFrontmatterField {
    key: String,
    raw_value: String,
    value_span: AdrByteSpan,
    value: AdrFrontmatterValue,
}

impl AdrFrontmatterField {
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }
    #[must_use]
    pub fn raw_value(&self) -> &str {
        &self.raw_value
    }
    #[must_use]
    pub const fn value_span(&self) -> AdrByteSpan {
        self.value_span
    }
    #[must_use]
    pub const fn value(&self) -> &AdrFrontmatterValue {
        &self.value
    }
}

/// A canonical ADR identifier (`ADR-` followed by exactly four ASCII digits).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdrId(String);

impl AdrId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Numeric component of this canonical `ADR-NNNN` identifier.
    #[must_use]
    pub fn number(&self) -> u16 {
        let digits = &self.0[4..];
        let mut number = 0_u16;
        for byte in digits.bytes() {
            number = number
                .saturating_mul(10)
                .saturating_add(u16::from(byte - b'0'));
        }
        number
    }
}

/// Compatibility name for a validated canonical ADR identifier.
pub type CanonicalAdrId = AdrId;

/// A typed relationship edge with a canonical identifier and source-field span.
///
/// Item spelling is normalized to the exact `ADR-NNNN` token. The containing
/// [`AdrFrontmatterField`] retains the complete raw spelling, quotes, comments,
/// and byte span for provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrReference {
    id: AdrId,
    raw_value: String,
    field_span: AdrByteSpan,
}

impl AdrReference {
    #[must_use]
    pub const fn id(&self) -> &AdrId {
        &self.id
    }
    #[must_use]
    pub fn raw_value(&self) -> &str {
        &self.raw_value
    }
    #[must_use]
    pub const fn field_span(&self) -> AdrByteSpan {
        self.field_span
    }
}

/// A named group of affected surface values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrAffectedSurface {
    category: String,
    values: Vec<String>,
}

impl AdrAffectedSurface {
    #[must_use]
    pub fn category(&self) -> &str {
        &self.category
    }
    #[must_use]
    pub fn values(&self) -> &[String] {
        &self.values
    }
}

/// One ADR delivery commitment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrDeliverable {
    id: String,
    description: Option<String>,
    exit_criteria: Option<String>,
    verified_by: Option<String>,
}

impl AdrDeliverable {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    #[must_use]
    pub fn exit_criteria(&self) -> Option<&str> {
        self.exit_criteria.as_deref()
    }
    #[must_use]
    pub fn verified_by(&self) -> Option<&str> {
        self.verified_by.as_deref()
    }
}

/// SHA-256 digest of canonical ADR source bytes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdrContentHash([u8; 32]);

impl AdrContentHash {
    #[must_use]
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

/// Namespace associated externally with an immutable ADR decision.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdrTenant(String);

impl AdrTenant {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Tenant-scoped identity that leaves content hashing unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrTenantIdentity {
    tenant: AdrTenant,
    content_hash: AdrContentHash,
}

impl AdrTenantIdentity {
    #[must_use]
    pub const fn tenant(&self) -> &AdrTenant {
        &self.tenant
    }
    #[must_use]
    pub const fn content_hash(&self) -> &AdrContentHash {
        &self.content_hash
    }
}

/// Externally namespaced ADR decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantAdrDecision {
    decision: AdrDecision,
    identity: AdrTenantIdentity,
}

impl TenantAdrDecision {
    #[must_use]
    pub const fn decision(&self) -> &AdrDecision {
        &self.decision
    }
    #[must_use]
    pub const fn identity(&self) -> &AdrTenantIdentity {
        &self.identity
    }
}

/// A complete parsed ADR decision IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrDecision {
    source_path: String,
    id: AdrId,
    title: String,
    status: String,
    date: String,
    owner: String,
    fields: Vec<AdrFrontmatterField>,
    edges: BTreeMap<String, Vec<AdrReference>>,
    affected_surfaces: Vec<AdrAffectedSurface>,
    deliverables: Vec<AdrDeliverable>,
    canonical_bytes: Vec<u8>,
    content_hash: AdrContentHash,
}

impl AdrDecision {
    #[must_use]
    pub fn source_path(&self) -> &str {
        &self.source_path
    }
    #[must_use]
    pub const fn id(&self) -> &AdrId {
        &self.id
    }
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }
    #[must_use]
    pub fn status(&self) -> &str {
        &self.status
    }
    #[must_use]
    pub fn date(&self) -> &str {
        &self.date
    }
    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }
    #[must_use]
    pub fn fields(&self) -> &[AdrFrontmatterField] {
        &self.fields
    }
    #[must_use]
    pub fn field(&self, key: &str) -> Option<&AdrFrontmatterField> {
        self.fields.iter().find(|field| field.key == key)
    }
    #[must_use]
    pub fn depends_on(&self) -> &[AdrReference] {
        self.edge("depends_on")
    }
    #[must_use]
    pub fn supersedes(&self) -> &[AdrReference] {
        self.edge("supersedes")
    }
    #[must_use]
    pub fn superseded_by(&self) -> &[AdrReference] {
        self.edge("superseded_by")
    }
    #[must_use]
    pub fn amends(&self) -> &[AdrReference] {
        self.edge("amends")
    }
    #[must_use]
    pub fn amended_by(&self) -> &[AdrReference] {
        self.edge("amended_by")
    }
    #[must_use]
    pub fn related(&self) -> &[AdrReference] {
        self.edge("related")
    }
    #[must_use]
    pub fn affected_surfaces(&self) -> &[AdrAffectedSurface] {
        &self.affected_surfaces
    }
    #[must_use]
    pub fn deliverables(&self) -> &[AdrDeliverable] {
        &self.deliverables
    }
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    #[must_use]
    pub const fn content_hash(&self) -> &AdrContentHash {
        &self.content_hash
    }

    /// Wrap this content identity in a tenant namespace without reparsing it.
    ///
    /// # Errors
    /// Returns [`AdrParseError::InvalidTenant`] for an empty or malformed tenant namespace.
    pub fn within_tenant(
        self,
        tenant: impl Into<String>,
    ) -> Result<TenantAdrDecision, AdrParseError> {
        let tenant = tenant.into();
        if tenant.is_empty()
            || tenant
                .bytes()
                .any(|byte| byte.is_ascii_whitespace() || byte == b'/')
        {
            return Err(AdrParseError::InvalidTenant { tenant });
        }
        Ok(TenantAdrDecision {
            identity: AdrTenantIdentity {
                tenant: AdrTenant(tenant),
                content_hash: self.content_hash.clone(),
            },
            decision: self,
        })
    }

    fn edge(&self, key: &str) -> &[AdrReference] {
        self.edges.get(key).map_or(&[], Vec::as_slice)
    }
}

/// Fail-closed parsing errors for ADR frontmatter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdrParseError {
    MissingLeadingFrontmatter,
    UnterminatedFrontmatter,
    DuplicateFrontmatterKey {
        key: String,
        span: AdrByteSpan,
    },
    InvalidFrontmatter {
        message: String,
        span: AdrByteSpan,
    },
    UnsupportedFrontmatterNesting {
        span: AdrByteSpan,
    },
    InvalidAdrReference {
        key: String,
        value: String,
        span: AdrByteSpan,
    },
    InvalidAdrId {
        value: String,
        span: AdrByteSpan,
    },
    AdrIdPathMismatch {
        id: String,
        source_path: String,
    },
    InvalidDate {
        value: String,
        span: AdrByteSpan,
    },
    InvalidSourcePath {
        source_path: String,
    },
    InvalidAdrHeading {
        message: String,
        span: AdrByteSpan,
    },
    MissingRequiredField {
        key: String,
    },
    InvalidTenant {
        tenant: String,
    },
}

impl fmt::Display for AdrParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLeadingFrontmatter => {
                f.write_str("ADR frontmatter must begin at byte zero")
            }
            Self::UnterminatedFrontmatter => f.write_str("ADR frontmatter is unterminated"),
            Self::DuplicateFrontmatterKey { key, .. } => {
                write!(f, "duplicate ADR frontmatter key {key:?}")
            }
            Self::InvalidFrontmatter { message, .. } => {
                write!(f, "invalid ADR frontmatter: {message}")
            }
            Self::UnsupportedFrontmatterNesting { .. } => {
                f.write_str("unsupported ADR frontmatter nesting")
            }
            Self::InvalidAdrReference { key, value, .. } => {
                write!(f, "invalid ADR reference {value:?} in {key}")
            }
            Self::InvalidAdrId { value, .. } => {
                write!(f, "invalid canonical ADR identifier {value:?}")
            }
            Self::AdrIdPathMismatch { id, source_path } => {
                write!(
                    f,
                    "ADR identifier {id} does not match source path {source_path:?}"
                )
            }
            Self::InvalidDate { value, .. } => {
                write!(f, "invalid ADR calendar date {value:?}")
            }
            Self::InvalidSourcePath { source_path } => {
                write!(
                    f,
                    "invalid repository-relative ADR source path {source_path:?}"
                )
            }
            Self::InvalidAdrHeading { message, .. } => {
                write!(f, "invalid ADR heading: {message}")
            }
            Self::MissingRequiredField { key } => {
                write!(f, "missing required ADR frontmatter field {key}")
            }
            Self::InvalidTenant { tenant } => write!(f, "invalid ADR tenant namespace {tenant:?}"),
        }
    }
}

impl std::error::Error for AdrParseError {}

/// Parse strict leading ADR frontmatter into a content-addressed decision IR.
///
/// # Errors
/// Returns an error for malformed, nested beyond the supported contract, duplicate,
/// or semantically invalid frontmatter. No partial decision is returned.
pub fn parse_adr_decision(input: &AdrParseInput) -> Result<AdrDecision, AdrParseError> {
    validate_source_path(input.source_path())?;
    if !input.source.starts_with("---\n") && !input.source.starts_with("---\r\n") {
        return Err(AdrParseError::MissingLeadingFrontmatter);
    }
    let mut parser = FrontmatterParser::new(input.source());
    parser.parse()?;
    let id_field = require_field(&parser.fields, "id")?;
    let id = AdrId(require_scalar_field(id_field)?.to_owned());
    if !is_adr_id(id.as_str()) {
        return Err(AdrParseError::InvalidAdrId {
            value: id.0,
            span: id_field.value_span,
        });
    }
    validate_id_matches_path(&id, input.source_path())?;
    let status = require_scalar(&parser.fields, "status")?.to_owned();
    let date_field = require_field(&parser.fields, "date")?;
    let date = require_scalar_field(date_field)?.to_owned();
    if !is_calendar_date(&date) {
        return Err(AdrParseError::InvalidDate {
            value: date,
            span: date_field.value_span,
        });
    }
    let owner = require_scalar(&parser.fields, "owner")?.to_owned();
    let title = heading_title(input.source(), parser.cursor, &id)?;
    let mut edges = BTreeMap::new();
    for key in [
        "depends_on",
        "supersedes",
        "superseded_by",
        "amends",
        "amended_by",
        "related",
    ] {
        let values = parser
            .fields
            .iter()
            .find(|field| field.key == key)
            .map(field_values)
            .unwrap_or_default();
        let mut references = Vec::with_capacity(values.len());
        for value in values {
            if !is_adr_id(&value) {
                return Err(AdrParseError::InvalidAdrReference {
                    key: key.to_owned(),
                    value,
                    span: parser.field_span(key),
                });
            }
            references.push(AdrReference {
                id: AdrId(value.clone()),
                raw_value: value,
                field_span: parser.field_span(key),
            });
        }
        edges.insert(key.to_owned(), references);
    }
    let canonical_bytes = input.source.as_bytes().to_vec();
    let digest = Sha256::digest(&canonical_bytes);
    let mut bytes = [0_u8; 32];
    bytes.copy_from_slice(&digest);
    Ok(AdrDecision {
        source_path: input.source_path.clone(),
        id,
        title,
        status,
        date,
        owner,
        fields: parser.fields,
        edges,
        affected_surfaces: parser.affected_surfaces,
        deliverables: parser.deliverables,
        canonical_bytes,
        content_hash: AdrContentHash(bytes),
    })
}

fn require_scalar<'a>(
    fields: &'a [AdrFrontmatterField],
    key: &str,
) -> Result<&'a str, AdrParseError> {
    require_scalar_field(require_field(fields, key)?)
}

fn require_field<'a>(
    fields: &'a [AdrFrontmatterField],
    key: &str,
) -> Result<&'a AdrFrontmatterField, AdrParseError> {
    let Some(field) = fields.iter().find(|field| field.key == key) else {
        return Err(AdrParseError::MissingRequiredField {
            key: key.to_owned(),
        });
    };
    Ok(field)
}

fn require_scalar_field(field: &AdrFrontmatterField) -> Result<&str, AdrParseError> {
    match &field.value {
        AdrFrontmatterValue::Scalar(value) if !value.is_empty() => Ok(value),
        _ => Err(AdrParseError::InvalidFrontmatter {
            message: format!("{} must be a non-empty scalar", field.key),
            span: field.value_span,
        }),
    }
}

fn field_values(field: &AdrFrontmatterField) -> Vec<String> {
    match &field.value {
        AdrFrontmatterValue::Scalar(value) => vec![value.clone()],
        AdrFrontmatterValue::List(values) => values.clone(),
        AdrFrontmatterValue::Null | AdrFrontmatterValue::Empty => Vec::new(),
    }
}

fn is_adr_id(value: &str) -> bool {
    value.len() == 8
        && value.starts_with("ADR-")
        && value.as_bytes()[4..].iter().all(u8::is_ascii_digit)
}

fn validate_id_matches_path(id: &AdrId, source_path: &str) -> Result<(), AdrParseError> {
    let file_name = source_path.rsplit('/').next().unwrap_or(source_path);
    let matches = file_name == format!("{}.md", id.as_str())
        || file_name
            .strip_prefix(id.as_str())
            .is_some_and(|suffix| suffix.starts_with('-') && suffix.ends_with(".md"));
    if matches {
        Ok(())
    } else {
        Err(AdrParseError::AdrIdPathMismatch {
            id: id.as_str().to_owned(),
            source_path: source_path.to_owned(),
        })
    }
}

fn validate_source_path(source_path: &str) -> Result<(), AdrParseError> {
    let Some(file_name) = source_path.strip_prefix("docs/decisions/") else {
        return Err(AdrParseError::InvalidSourcePath {
            source_path: source_path.to_owned(),
        });
    };
    if file_name.is_empty()
        || file_name.contains('/')
        || file_name.contains('\\')
        || file_name == "."
        || file_name == ".."
    {
        return Err(AdrParseError::InvalidSourcePath {
            source_path: source_path.to_owned(),
        });
    }
    Ok(())
}

fn heading_title(source: &str, start: usize, id: &AdrId) -> Result<String, AdrParseError> {
    let mut cursor = start;
    while cursor < source.len() {
        let rest = &source[cursor..];
        let line_end = rest
            .find('\n')
            .map_or(source.len(), |offset| cursor + offset);
        let mut content_end = line_end;
        if content_end > cursor && source.as_bytes()[content_end - 1] == b'\r' {
            content_end -= 1;
        }
        let line = &source[cursor..content_end];
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("<!--") {
            let Some(value) = trimmed.strip_prefix("# ") else {
                return Err(AdrParseError::InvalidAdrHeading {
                    message: "first content line must be an H1".to_owned(),
                    span: span(cursor, content_end),
                });
            };
            let Some((heading_id, title)) = split_heading(value) else {
                return Err(AdrParseError::InvalidAdrHeading {
                    message: "H1 must use '<id>: <title>' or '<id> — <title>'".to_owned(),
                    span: span(cursor, content_end),
                });
            };
            if heading_id.trim() != id.as_str() {
                return Err(AdrParseError::InvalidAdrHeading {
                    message: format!(
                        "H1 id {:?} does not match {}",
                        heading_id.trim(),
                        id.as_str()
                    ),
                    span: span(cursor, content_end),
                });
            }
            let title = title.trim();
            if title.is_empty() {
                return Err(AdrParseError::InvalidAdrHeading {
                    message: "H1 title must be non-empty".to_owned(),
                    span: span(cursor, content_end),
                });
            }
            return Ok(title.to_owned());
        }
        cursor = if line_end < source.len() {
            line_end + 1
        } else {
            line_end
        };
    }
    Err(AdrParseError::InvalidAdrHeading {
        message: "ADR body must contain an H1".to_owned(),
        span: span(start, source.len()),
    })
}

fn split_heading(value: &str) -> Option<(&str, &str)> {
    match (value.find(':'), value.find(" — ")) {
        (Some(colon), Some(dash)) if dash < colon => {
            Some((&value[..dash], &value[dash + " — ".len()..]))
        }
        (Some(colon), _) => Some((&value[..colon], &value[colon + 1..])),
        (None, Some(dash)) => Some((&value[..dash], &value[dash + " — ".len()..])),
        (None, None) => None,
    }
}

fn is_calendar_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if !bytes[..4].iter().all(u8::is_ascii_digit)
        || !bytes[5..7].iter().all(u8::is_ascii_digit)
        || !bytes[8..].iter().all(u8::is_ascii_digit)
    {
        return false;
    }
    let number = |digits: &[u8]| {
        digits
            .iter()
            .fold(0_u16, |value, digit| value * 10 + u16::from(*digit - b'0'))
    };
    let year = number(&bytes[..4]);
    let month = number(&bytes[5..7]);
    let day = number(&bytes[8..]);
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=max_day).contains(&day)
}

struct FrontmatterParser<'a> {
    source: &'a str,
    cursor: usize,
    fields: Vec<AdrFrontmatterField>,
    affected_surfaces: Vec<AdrAffectedSurface>,
    deliverables: Vec<AdrDeliverable>,
}

impl<'a> FrontmatterParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            cursor: 0,
            fields: Vec::new(),
            affected_surfaces: Vec::new(),
            deliverables: Vec::new(),
        }
    }

    fn parse(&mut self) -> Result<(), AdrParseError> {
        let (_, _, next) = self.line();
        self.cursor = next;
        let mut active: Option<String> = None;
        while self.cursor < self.source.len() {
            let (line, start, next) = self.line();
            self.cursor = next;
            if line == "---" {
                return Ok(());
            }
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with(' ') || line.starts_with('\t') {
                self.parse_indented(line, start, active.as_deref())?;
                continue;
            }
            let Some((key, raw_value)) = line.split_once(':') else {
                return Err(self.invalid("expected key: value", start, line.len()));
            };
            if !valid_key(key) {
                return Err(self.invalid("invalid key", start, line.len()));
            }
            if self.fields.iter().any(|field| field.key == key) {
                return Err(AdrParseError::DuplicateFrontmatterKey {
                    key: key.to_owned(),
                    span: span(start, start + line.len()),
                });
            }
            let value_start = start + key.len() + 1;
            let value = parse_value(raw_value)
                .map_err(|message| self.invalid(&message, value_start, raw_value.len()))?;
            let supported_structured_value = match &value {
                AdrFrontmatterValue::Empty | AdrFrontmatterValue::Null => true,
                AdrFrontmatterValue::List(values) => values.is_empty(),
                AdrFrontmatterValue::Scalar(_) => false,
            };
            if matches!(key, "affected_surfaces" | "deliverables") && !supported_structured_value {
                return Err(self.invalid(
                    "structured field requires an empty value, null, [], or supported block form",
                    value_start,
                    raw_value.len(),
                ));
            }
            self.fields.push(AdrFrontmatterField {
                key: key.to_owned(),
                raw_value: raw_value.to_owned(),
                value_span: span(value_start, start + line.len()),
                value,
            });
            active = Some(key.to_owned());
        }
        Err(AdrParseError::UnterminatedFrontmatter)
    }

    fn parse_indented(
        &mut self,
        line: &str,
        start: usize,
        active: Option<&str>,
    ) -> Result<(), AdrParseError> {
        let Some(active) = active else {
            return Err(self.invalid("indented value without a key", start, line.len()));
        };
        if line.starts_with(" ") && !line.starts_with("  ") {
            return Err(self.invalid(
                "indentation must be a multiple of two spaces",
                start,
                line.len(),
            ));
        }
        if line.starts_with("\t") {
            return Err(self.invalid("tabs are not supported", start, line.len()));
        }
        if matches!(active, "affected_surfaces" | "deliverables")
            && self
                .fields
                .iter()
                .find(|field| field.key == active)
                .is_none_or(|field| !matches!(field.value, AdrFrontmatterValue::Empty))
        {
            return Err(self.invalid(
                "structured child cannot follow an inline, null, or [] parent value",
                start,
                line.len(),
            ));
        }
        self.extend_field_raw(active, start, line.len())?;
        if let Some(style) = self.block_scalar_style(active) {
            return self.parse_block_scalar(active, style, line, start);
        }
        match active {
            "affected_surfaces" => self.parse_surface(line, start),
            "deliverables" => self.parse_deliverable(line, start),
            _ => self.parse_list_item(active, line, start),
        }
    }

    fn extend_field_raw(
        &mut self,
        key: &str,
        line_start: usize,
        line_len: usize,
    ) -> Result<(), AdrParseError> {
        let Some(field_index) = self.fields.iter().position(|field| field.key == key) else {
            return Err(self.invalid("missing active frontmatter field", line_start, line_len));
        };
        let raw_start = self.fields[field_index].value_span.start as usize;
        let raw_end = line_start + line_len;
        self.fields[field_index].raw_value = self.source[raw_start..raw_end].to_owned();
        self.fields[field_index].value_span = span(raw_start, raw_end);
        Ok(())
    }

    fn block_scalar_style(&self, key: &str) -> Option<char> {
        self.fields
            .iter()
            .find(|field| field.key == key)
            .and_then(|field| field.raw_value.lines().next())
            .map(str::trim)
            .and_then(|value| match value {
                ">" => Some('>'),
                "|" => Some('|'),
                _ => None,
            })
    }

    fn parse_block_scalar(
        &mut self,
        key: &str,
        style: char,
        line: &str,
        start: usize,
    ) -> Result<(), AdrParseError> {
        let Some(content) = line.strip_prefix("  ") else {
            return Err(self.invalid(
                "block scalar content requires two-space indentation",
                start,
                line.len(),
            ));
        };
        let Some(field_index) = self.fields.iter().position(|field| field.key == key) else {
            return Err(self.invalid("missing block scalar field", start, line.len()));
        };
        let raw_start = self.fields[field_index].value_span.start as usize;
        let raw_end = start + line.len();
        let raw_value = self.source[raw_start..raw_end].to_owned();
        let field = &mut self.fields[field_index];
        field.raw_value = raw_value;
        field.value_span = span(raw_start, raw_end);
        match &mut field.value {
            AdrFrontmatterValue::Empty => {
                field.value = AdrFrontmatterValue::Scalar(content.to_owned());
            }
            AdrFrontmatterValue::Scalar(value) => {
                if style == '|' || content.is_empty() {
                    value.push('\n');
                } else if !value.is_empty() && !value.ends_with('\n') {
                    value.push(' ');
                }
                value.push_str(content);
            }
            AdrFrontmatterValue::List(_) | AdrFrontmatterValue::Null => {
                return Err(self.invalid("invalid block scalar state", start, line.len()));
            }
        }
        Ok(())
    }

    fn parse_list_item(
        &mut self,
        key: &str,
        line: &str,
        start: usize,
    ) -> Result<(), AdrParseError> {
        let Some(value) = line.strip_prefix("  - ") else {
            return Err(AdrParseError::UnsupportedFrontmatterNesting {
                span: span(start, start + line.len()),
            });
        };
        if value.contains(':') {
            return Err(AdrParseError::UnsupportedFrontmatterNesting {
                span: span(start, start + line.len()),
            });
        }
        let value = parse_scalar(value)
            .map_err(|message| self.invalid(&message, start + 4, value.len()))?;
        let Some(field) = self.fields.iter_mut().find(|field| field.key == key) else {
            return Err(self.invalid("missing list field", start, line.len()));
        };
        match &mut field.value {
            AdrFrontmatterValue::Empty => field.value = AdrFrontmatterValue::List(vec![value]),
            AdrFrontmatterValue::List(values) => values.push(value),
            _ => {
                return Err(self.invalid(
                    "block list requires an empty key value",
                    start,
                    line.len(),
                ));
            }
        }
        Ok(())
    }

    fn parse_surface(&mut self, line: &str, start: usize) -> Result<(), AdrParseError> {
        if let Some(value) = line.strip_prefix("    - ") {
            if value.contains(':') {
                return Err(AdrParseError::UnsupportedFrontmatterNesting {
                    span: span(start, start + line.len()),
                });
            }
            let parsed_value = parse_scalar(value)
                .map_err(|message| self.invalid(&message, start + 6, value.len()))?;
            let Some(surface) = self.affected_surfaces.last_mut() else {
                return Err(self.invalid("surface list item without category", start, line.len()));
            };
            surface.values.push(parsed_value);
            return Ok(());
        }
        if let Some((category, raw)) = line
            .strip_prefix("  ")
            .and_then(|value| value.split_once(':'))
        {
            if !valid_key(category) {
                return Err(self.invalid("invalid affected surface category", start, line.len()));
            }
            let values = field_value_list(parse_value(raw).map_err(|message| {
                self.invalid(&message, start + 2 + category.len() + 1, raw.len())
            })?);
            if self
                .affected_surfaces
                .iter()
                .any(|surface| surface.category == category)
            {
                return Err(AdrParseError::DuplicateFrontmatterKey {
                    key: category.to_owned(),
                    span: span(start, start + line.len()),
                });
            }
            self.affected_surfaces.push(AdrAffectedSurface {
                category: category.to_owned(),
                values,
            });
            return Ok(());
        }
        Err(AdrParseError::UnsupportedFrontmatterNesting {
            span: span(start, start + line.len()),
        })
    }

    fn parse_deliverable(&mut self, line: &str, start: usize) -> Result<(), AdrParseError> {
        if let Some(raw) = line.strip_prefix("  - id: ") {
            let id = parse_scalar(raw)
                .map_err(|message| self.invalid(&message, start + 8, raw.len()))?;
            self.deliverables.push(AdrDeliverable {
                id,
                description: None,
                exit_criteria: None,
                verified_by: None,
            });
            return Ok(());
        }
        let Some((key, raw)) = line
            .strip_prefix("    ")
            .and_then(|value| value.split_once(':'))
        else {
            return Err(AdrParseError::UnsupportedFrontmatterNesting {
                span: span(start, start + line.len()),
            });
        };
        let value = parse_scalar(raw)
            .map_err(|message| self.invalid(&message, start + 4 + key.len() + 1, raw.len()))?;
        let Some(deliverable) = self.deliverables.last_mut() else {
            return Err(self.invalid("deliverable property without item", start, line.len()));
        };
        let slot = match key {
            "description" => &mut deliverable.description,
            "exit_criteria" => &mut deliverable.exit_criteria,
            "verified_by" => &mut deliverable.verified_by,
            _ => {
                return Err(AdrParseError::UnsupportedFrontmatterNesting {
                    span: span(start, start + line.len()),
                });
            }
        };
        if slot.replace(value).is_some() {
            return Err(AdrParseError::DuplicateFrontmatterKey {
                key: key.to_owned(),
                span: span(start, start + line.len()),
            });
        }
        Ok(())
    }

    fn line(&self) -> (&'a str, usize, usize) {
        let rest = &self.source[self.cursor..];
        let line_end = rest
            .find('\n')
            .map_or(self.source.len(), |offset| self.cursor + offset);
        let mut content_end = line_end;
        if content_end > self.cursor && self.source.as_bytes()[content_end - 1] == b'\r' {
            content_end -= 1;
        }
        let next = if line_end < self.source.len() {
            line_end + 1
        } else {
            line_end
        };
        (&self.source[self.cursor..content_end], self.cursor, next)
    }

    fn invalid(&self, message: &str, start: usize, len: usize) -> AdrParseError {
        AdrParseError::InvalidFrontmatter {
            message: message.to_owned(),
            span: span(start, start + len),
        }
    }
    fn field_span(&self, key: &str) -> AdrByteSpan {
        self.fields
            .iter()
            .find(|field| field.key == key)
            .map_or(AdrByteSpan { start: 0, end: 0 }, |field| field.value_span)
    }
}

fn span(start: usize, end: usize) -> AdrByteSpan {
    AdrByteSpan {
        start: start as u64,
        end: end as u64,
    }
}
fn valid_key(key: &str) -> bool {
    let mut bytes = key.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
fn field_value_list(value: AdrFrontmatterValue) -> Vec<String> {
    match value {
        AdrFrontmatterValue::Scalar(value) => vec![value],
        AdrFrontmatterValue::List(values) => values,
        AdrFrontmatterValue::Null | AdrFrontmatterValue::Empty => Vec::new(),
    }
}

fn parse_value(raw: &str) -> Result<AdrFrontmatterValue, String> {
    let value = strip_comment(raw).trim();
    if value.is_empty() {
        return Ok(AdrFrontmatterValue::Empty);
    }
    if matches!(value, ">" | "|") {
        return Ok(AdrFrontmatterValue::Empty);
    }
    if matches!(value, "null" | "~") {
        return Ok(AdrFrontmatterValue::Null);
    }
    if value.starts_with('[') {
        if !value.ends_with(']') {
            return Err("unterminated inline list".to_owned());
        }
        let inner = &value[1..value.len() - 1];
        if inner.trim().is_empty() {
            return Ok(AdrFrontmatterValue::List(Vec::new()));
        }
        let mut values = Vec::new();
        for item in split_list(inner)? {
            values.push(parse_scalar(item)?);
        }
        return Ok(AdrFrontmatterValue::List(values));
    }
    Ok(AdrFrontmatterValue::Scalar(parse_scalar(value)?))
}

fn parse_scalar(raw: &str) -> Result<String, String> {
    let value = strip_comment(raw).trim();
    if value.is_empty() {
        return Ok(String::new());
    }
    let bytes = value.as_bytes();
    if matches!(bytes.first(), Some(b'\'' | b'\"')) {
        let quote = bytes[0];
        if bytes.len() < 2 || bytes.last().is_none_or(|last| *last != quote) {
            return Err("unterminated quoted scalar".to_owned());
        }
        let inner = &value[1..value.len() - 1];
        return Ok(if quote == b'\"' {
            unescape_double(inner)?
        } else {
            inner.replace("''", "'")
        });
    }
    if value.contains('[') || value.contains(']') || value.contains('{') || value.contains('}') {
        return Err("unsupported scalar syntax".to_owned());
    }
    Ok(value.to_owned())
}

fn strip_comment(value: &str) -> &str {
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quote == Some('"') {
            escaped = true;
            continue;
        }
        if matches!(character, '\'' | '\"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
            continue;
        }
        if character == '#' && quote.is_none() {
            return &value[..index];
        }
    }
    value
}

fn split_list(value: &str) -> Result<Vec<&str>, String> {
    let mut items = Vec::new();
    let mut quote = None;
    let mut escaped = false;
    let mut start = 0;
    for (index, character) in value.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quote == Some('"') {
            escaped = true;
            continue;
        }
        if matches!(character, '\'' | '\"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
            continue;
        }
        if character == ',' && quote.is_none() {
            items.push(&value[start..index]);
            start = index + 1;
        }
    }
    if quote.is_some() {
        return Err("unterminated quoted list item".to_owned());
    }
    items.push(&value[start..]);
    Ok(items)
}

fn unescape_double(value: &str) -> Result<String, String> {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        let Some(escaped) = chars.next() else {
            return Err("unterminated escape".to_owned());
        };
        output.push(match escaped {
            '\\' => '\\',
            '\"' => '\"',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            _ => return Err("unsupported escape".to_owned()),
        });
    }
    Ok(output)
}
