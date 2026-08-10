#![forbid(unsafe_code)]
//! OOXML import/export/render contract model and fidelity fixture taxonomy domain.
//!
//! This crate owns the provider-neutral DOCX/XLSX/PPTX vocabulary used by
//! Drive-bound Docs, Sheets, Slides, and format workers. It deliberately models
//! fixture risk and Drive object binding before parser/renderer adapters so
//! untrusted Office files remain tenant-scoped, observable, and quarantine-aware.

use oya_office_drive_domain::{DriveObjectBinding, DriveObjectKind};
use oya_office_kernel::{DataClass, ObjectId, RequestId, TenantId};

/// Stable crate identifier used by workspace and Buck2 scaffold verification.
pub const CRATE_NAME: &str = "oya-office-format-domain";

/// Product vertical slice owned by this crate.
pub const VERTICAL_SLICE: &str = "format";

/// Source-shaped architectural layer represented by this crate.
pub const ARCHITECTURE_LAYER: &str = "domain";

/// Version for provider-neutral import/export contract specs.
pub const FORMAT_CONTRACT_SCHEMA_VERSION: u32 = 1;

/// Version for provider-neutral fixture scorecards.
pub const FORMAT_SCORECARD_SCHEMA_VERSION: u32 = 1;

/// Version for reproducible peer benchmark report records.
pub const BENCHMARK_REPORT_SCHEMA_VERSION: u32 = 1;

/// Version for the G082 format/benchmark lane contract.
pub const G082_FORMAT_BENCHMARK_CONTRACT_VERSION: &str = "g082-format-benchmark-v1";

/// Format/benchmark gate categories required before this lane can advance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FormatBenchmarkGateKind {
    /// Fixture origin, corpus id, tenant class, and data class are recorded.
    FixtureProvenance,
    /// DOCX/XLSX/PPTX fixture taxonomy is explicit and Drive-bound.
    FixtureTaxonomy,
    /// The first OOXML scope covers DOCX, XLSX, and PPTX separately.
    OoxmlFormatCoverage,
    /// Risky Office packages route to quarantine-capable worker lanes.
    SecurityQuarantine,
    /// Scorecard/report rows carry versioned reproducibility schema fields.
    ScorecardSchema,
    /// Google Workspace and ONLYOFFICE remain peer benchmark labels only.
    PeerBenchmarkWorkflow,
    /// Dry-run, credentialed-pending, measured, skipped/gap states are distinct.
    BenchmarkEvidenceState,
    /// Parity claims require credentialed measured metrics.
    NoParityClaimWithoutMeasuredMetrics,
}

/// One launch-blocking G082 format/benchmark gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormatBenchmarkGate {
    kind: FormatBenchmarkGateKind,
    evidence: &'static str,
    launch_blocking: bool,
}

impl FormatBenchmarkGate {
    /// Returns the gate category.
    #[must_use]
    pub const fn kind(self) -> FormatBenchmarkGateKind {
        self.kind
    }

    /// Returns the repo evidence that satisfies the gate.
    #[must_use]
    pub const fn evidence(self) -> &'static str {
        self.evidence
    }

    /// Returns true when this gate blocks launch/parity claims if absent.
    #[must_use]
    pub const fn launch_blocking(self) -> bool {
        self.launch_blocking
    }
}

const G082_FORMAT_BENCHMARK_GATES: [FormatBenchmarkGate; 8] = [
    FormatBenchmarkGate {
        kind: FormatBenchmarkGateKind::FixtureProvenance,
        evidence: "docs/format/fixture-corpus-taxonomy.md records fixture id, corpus id, Drive object binding, tenant/data-class handling, and source evidence.",
        launch_blocking: true,
    },
    FormatBenchmarkGate {
        kind: FormatBenchmarkGateKind::FixtureTaxonomy,
        evidence: "FormatFixtureSpec and fixture taxonomy docs classify smoke, representative, edge, and adversarial OOXML corpora.",
        launch_blocking: true,
    },
    FormatBenchmarkGate {
        kind: FormatBenchmarkGateKind::OoxmlFormatCoverage,
        evidence: "OfficeFormatKind plus Docs/Sheets/Slides parity matrices keep DOCX, XLSX, and PPTX separate.",
        launch_blocking: true,
    },
    FormatBenchmarkGate {
        kind: FormatBenchmarkGateKind::SecurityQuarantine,
        evidence: "FixtureFeature::requires_quarantine and FormatWorkerIsolationTier force macro, external-link, ZIP, and adversarial packages into quarantine lanes.",
        launch_blocking: true,
    },
    FormatBenchmarkGate {
        kind: FormatBenchmarkGateKind::ScorecardSchema,
        evidence: "FormatScorecard and BenchmarkReportRecord carry schema versions, metrics, workflow, command, environment, timestamp, and tenant-id class.",
        launch_blocking: true,
    },
    FormatBenchmarkGate {
        kind: FormatBenchmarkGateKind::PeerBenchmarkWorkflow,
        evidence: "BenchmarkPeer, BenchmarkHarnessPlan, and parity matrices model Google Workspace and ONLYOFFICE as benchmark peers, not runtime dependencies.",
        launch_blocking: true,
    },
    FormatBenchmarkGate {
        kind: FormatBenchmarkGateKind::BenchmarkEvidenceState,
        evidence: "BenchmarkReportEvidenceState separates implemented-local, credential-free dry-run, credentialed-pending, measured, skipped/gap, and not-applicable evidence.",
        launch_blocking: true,
    },
    FormatBenchmarkGate {
        kind: FormatBenchmarkGateKind::NoParityClaimWithoutMeasuredMetrics,
        evidence: "BenchmarkReportRecord::new rejects dry-run parity claims and measured rows without scorecard metrics.",
        launch_blocking: true,
    },
];

/// Returns the complete G082 format/benchmark lane gate set.
#[must_use]
pub const fn g082_format_benchmark_gates() -> &'static [FormatBenchmarkGate] {
    G082_FORMAT_BENCHMARK_GATES.as_slice()
}

/// Format-domain validation error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatDomainError {
    message: String,
}

impl FormatDomainError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the validation message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

impl core::fmt::Display for FormatDomainError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for FormatDomainError {}

macro_rules! fixture_identifier_type {
    ($(#[$meta:meta])* $name:ident, $kind:literal) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(String);

        impl $name {
            /// Creates a validated fixture identifier.
            pub fn new(value: impl Into<String>) -> Result<Self, FormatDomainError> {
                validate_fixture_identifier($kind, value).map(Self)
            }

            /// Returns the stable string representation.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }
    };
}

fixture_identifier_type!(
    /// Stable identifier for a benchmark/compatibility fixture corpus.
    FixtureCorpusId,
    "fixture corpus id"
);
fixture_identifier_type!(
    /// Stable identifier for one DOCX/XLSX/PPTX fixture.
    FormatFixtureId,
    "format fixture id"
);

fn validate_fixture_identifier(
    kind: &'static str,
    value: impl Into<String>,
) -> Result<String, FormatDomainError> {
    const MAX_IDENTIFIER_LEN: usize = 128;

    let value = value.into();
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(FormatDomainError::new(format!("{kind} must not be empty")));
    }
    if trimmed.len() > MAX_IDENTIFIER_LEN {
        return Err(FormatDomainError::new(format!("{kind} is too long")));
    }
    if !trimmed
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
    {
        return Err(FormatDomainError::new(format!(
            "{kind} contains an invalid character"
        )));
    }
    Ok(trimmed.to_owned())
}

/// Supported Office Open XML formats for the first production fixture taxonomy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OfficeFormatKind {
    /// Microsoft Word / OOXML document format.
    Docx,
    /// Microsoft Excel / OOXML workbook format.
    Xlsx,
    /// Microsoft PowerPoint / OOXML presentation format.
    Pptx,
}

impl OfficeFormatKind {
    /// Returns the canonical file extension without a leading dot.
    #[must_use]
    pub const fn canonical_extension(self) -> &'static str {
        match self {
            Self::Docx => "docx",
            Self::Xlsx => "xlsx",
            Self::Pptx => "pptx",
        }
    }

    /// Returns the canonical OOXML media type.
    #[must_use]
    pub const fn canonical_media_type(self) -> &'static str {
        match self {
            Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Self::Xlsx => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            Self::Pptx => {
                "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            }
        }
    }

    /// Returns the Drive object kind this format is allowed to bind to.
    #[must_use]
    pub const fn drive_object_kind(self) -> DriveObjectKind {
        match self {
            Self::Docx => DriveObjectKind::Document,
            Self::Xlsx => DriveObjectKind::Spreadsheet,
            Self::Pptx => DriveObjectKind::Presentation,
        }
    }

    /// Returns true when this format can bind to the provided Drive object kind.
    #[must_use]
    pub const fn supports_drive_object_kind(self, kind: DriveObjectKind) -> bool {
        matches!(
            (self, kind),
            (Self::Docx, DriveObjectKind::Document)
                | (Self::Xlsx, DriveObjectKind::Spreadsheet)
                | (Self::Pptx, DriveObjectKind::Presentation)
        )
    }
}

const G082_REQUIRED_OOXML_FORMATS: [OfficeFormatKind; 3] = [
    OfficeFormatKind::Docx,
    OfficeFormatKind::Xlsx,
    OfficeFormatKind::Pptx,
];

/// Returns the OOXML formats in scope for the G082 benchmark taxonomy.
#[must_use]
pub const fn g082_required_ooxml_formats() -> &'static [OfficeFormatKind] {
    G082_REQUIRED_OOXML_FORMATS.as_slice()
}

/// Fixture complexity tier used to plan parallel benchmark lanes and risk gates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FixtureComplexity {
    /// Minimal smoke fixture for cheap route/build verification.
    Smoke,
    /// Representative production-like fixture with common product features.
    Representative,
    /// Known fidelity edge case that should remain deterministic and bounded.
    EdgeCase,
    /// Hostile or high-risk fixture used only in quarantine-aware lanes.
    Adversarial,
}

/// Fine-grained feature tags used by format fixtures and benchmark scorecards.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FixtureFeature {
    /// Basic document paragraphs/runs.
    ParagraphText,
    /// Tables and table styling.
    Tables,
    /// Comments or annotations.
    Comments,
    /// Track-changes/revision markup.
    TrackChanges,
    /// Workbook formulas.
    Formulas,
    /// Charts or chart data caches.
    Charts,
    /// Pivot tables or pivot caches.
    PivotTables,
    /// Merged spreadsheet cells.
    MergedCells,
    /// Slides speaker notes.
    SpeakerNotes,
    /// Slide animations or transitions.
    Animations,
    /// Embedded images, audio, video, or attachments.
    EmbeddedMedia,
    /// Macro-enabled or macro-adjacent package content.
    MacroEnabled,
    /// External links, remote templates, or cross-workbook references.
    ExternalLinks,
    /// ZIP/container boundary edge cases.
    ZipContainerEdges,
    /// Accessibility metadata and alternate text.
    AccessibilityMetadata,
}

impl FixtureFeature {
    /// Returns true when the feature requires a quarantine-capable worker lane.
    #[must_use]
    pub const fn requires_quarantine(self) -> bool {
        matches!(
            self,
            Self::MacroEnabled | Self::ExternalLinks | Self::ZipContainerEdges
        )
    }
}

/// Worker isolation tier derived from fixture features and complexity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FormatWorkerIsolationTier {
    /// Normal bounded worker sandbox for benign fixtures.
    Standard,
    /// Quarantine-capable lane for hostile or ambiguous OOXML packages.
    Quarantine,
}

/// Import/export/roundtrip direction for a Drive-bound format job.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FormatJobDirection {
    /// Import an Office file into a Drive-bound Docs/Sheets/Slides object.
    Import,
    /// Export a Drive-bound Docs/Sheets/Slides object to an Office file.
    Export,
    /// Import then export the same fixture to measure round-trip behavior.
    RoundTrip,
}

/// Verified relationship between an OOXML fixture and a Drive object binding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatFixtureBinding {
    drive_binding: DriveObjectBinding,
    format_kind: OfficeFormatKind,
}

impl FormatFixtureBinding {
    /// Creates a format binding and fail-closes on mismatched Drive object kind.
    pub fn new(
        drive_binding: DriveObjectBinding,
        format_kind: OfficeFormatKind,
    ) -> Result<Self, FormatDomainError> {
        if !format_kind.supports_drive_object_kind(drive_binding.kind()) {
            return Err(FormatDomainError::new(
                "office format does not match Drive object kind",
            ));
        }
        Ok(Self {
            drive_binding,
            format_kind,
        })
    }

    /// Returns the Drive object binding.
    #[must_use]
    pub const fn drive_binding(&self) -> &DriveObjectBinding {
        &self.drive_binding
    }

    /// Returns the OOXML format kind.
    #[must_use]
    pub const fn format_kind(&self) -> OfficeFormatKind {
        self.format_kind
    }

    /// Returns the tenant id carried by the Drive binding.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        self.drive_binding.tenant_id()
    }

    /// Returns the object id carried by the Drive binding.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        self.drive_binding.object_id()
    }

    /// Returns the data classification carried by the Drive binding.
    #[must_use]
    pub const fn data_class(&self) -> DataClass {
        self.drive_binding.data_class()
    }
}

/// One fixture in the DOCX/XLSX/PPTX taxonomy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatFixtureSpec {
    fixture_id: FormatFixtureId,
    corpus_id: FixtureCorpusId,
    binding: FormatFixtureBinding,
    complexity: FixtureComplexity,
    features: Vec<FixtureFeature>,
}

impl FormatFixtureSpec {
    /// Creates a fixture spec with Drive binding, feature tags, and complexity.
    pub fn new(
        fixture_id: FormatFixtureId,
        corpus_id: FixtureCorpusId,
        binding: FormatFixtureBinding,
        complexity: FixtureComplexity,
        features: Vec<FixtureFeature>,
    ) -> Result<Self, FormatDomainError> {
        if features.is_empty() {
            return Err(FormatDomainError::new(
                "format fixture must declare at least one feature tag",
            ));
        }
        if matches!(complexity, FixtureComplexity::Smoke)
            && features.iter().any(|feature| feature.requires_quarantine())
        {
            return Err(FormatDomainError::new(
                "smoke fixtures must not require quarantine isolation",
            ));
        }
        Ok(Self {
            fixture_id,
            corpus_id,
            binding,
            complexity,
            features,
        })
    }

    /// Returns the fixture id.
    #[must_use]
    pub const fn fixture_id(&self) -> &FormatFixtureId {
        &self.fixture_id
    }

    /// Returns the corpus id.
    #[must_use]
    pub const fn corpus_id(&self) -> &FixtureCorpusId {
        &self.corpus_id
    }

    /// Returns the verified format/Drive binding.
    #[must_use]
    pub const fn binding(&self) -> &FormatFixtureBinding {
        &self.binding
    }

    /// Returns the fixture complexity.
    #[must_use]
    pub const fn complexity(&self) -> FixtureComplexity {
        self.complexity
    }

    /// Returns the fixture feature tags.
    #[must_use]
    pub fn features(&self) -> &[FixtureFeature] {
        self.features.as_slice()
    }

    /// Returns the OOXML format kind.
    #[must_use]
    pub const fn format_kind(&self) -> OfficeFormatKind {
        self.binding.format_kind()
    }

    /// Returns the tenant id carried by the Drive binding.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        self.binding.tenant_id()
    }

    /// Returns the object id carried by the Drive binding.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        self.binding.object_id()
    }

    /// Returns the data classification carried by the Drive binding.
    #[must_use]
    pub const fn data_class(&self) -> DataClass {
        self.binding.data_class()
    }

    /// Returns the worker isolation tier required by this fixture.
    #[must_use]
    pub fn required_isolation_tier(&self) -> FormatWorkerIsolationTier {
        if matches!(self.complexity, FixtureComplexity::Adversarial)
            || self
                .features
                .iter()
                .any(|feature| feature.requires_quarantine())
        {
            FormatWorkerIsolationTier::Quarantine
        } else {
            FormatWorkerIsolationTier::Standard
        }
    }
}

/// Drive-bound import/export/roundtrip job contract for format workers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatJobContract {
    job_id: RequestId,
    direction: FormatJobDirection,
    fixture_id: FormatFixtureId,
    corpus_id: FixtureCorpusId,
    binding: FormatFixtureBinding,
    complexity: FixtureComplexity,
    feature_count: usize,
    required_isolation_tier: FormatWorkerIsolationTier,
}

impl FormatJobContract {
    /// Creates a worker job contract from a validated fixture specification.
    pub fn from_fixture(
        job_id: RequestId,
        direction: FormatJobDirection,
        fixture: &FormatFixtureSpec,
    ) -> Result<Self, FormatDomainError> {
        if fixture.features().is_empty() {
            return Err(FormatDomainError::new(
                "format job fixture must declare feature tags",
            ));
        }
        Ok(Self {
            job_id,
            direction,
            fixture_id: fixture.fixture_id().clone(),
            corpus_id: fixture.corpus_id().clone(),
            binding: fixture.binding().clone(),
            complexity: fixture.complexity(),
            feature_count: fixture.features().len(),
            required_isolation_tier: fixture.required_isolation_tier(),
        })
    }

    /// Returns the job id.
    #[must_use]
    pub const fn job_id(&self) -> &RequestId {
        &self.job_id
    }

    /// Returns the job direction.
    #[must_use]
    pub const fn direction(&self) -> FormatJobDirection {
        self.direction
    }

    /// Returns the fixture id.
    #[must_use]
    pub const fn fixture_id(&self) -> &FormatFixtureId {
        &self.fixture_id
    }

    /// Returns the corpus id.
    #[must_use]
    pub const fn corpus_id(&self) -> &FixtureCorpusId {
        &self.corpus_id
    }

    /// Returns the Drive-bound format binding.
    #[must_use]
    pub const fn binding(&self) -> &FormatFixtureBinding {
        &self.binding
    }

    /// Returns the tenant id carried by the Drive binding.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        self.binding.tenant_id()
    }

    /// Returns the object id carried by the Drive binding.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        self.binding.object_id()
    }

    /// Returns the OOXML format kind.
    #[must_use]
    pub const fn format_kind(&self) -> OfficeFormatKind {
        self.binding.format_kind()
    }

    /// Returns the Drive data class.
    #[must_use]
    pub const fn data_class(&self) -> DataClass {
        self.binding.data_class()
    }

    /// Returns the fixture complexity copied into the job.
    #[must_use]
    pub const fn complexity(&self) -> FixtureComplexity {
        self.complexity
    }

    /// Returns the number of feature tags attached to the fixture.
    #[must_use]
    pub const fn feature_count(&self) -> usize {
        self.feature_count
    }

    /// Returns the isolation tier required by this job.
    #[must_use]
    pub const fn required_isolation_tier(&self) -> FormatWorkerIsolationTier {
        self.required_isolation_tier
    }
}

/// Versioned IO contract derived from a Drive-bound format job.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatContractSpec {
    schema_version: u32,
    direction: FormatJobDirection,
    format_kind: OfficeFormatKind,
    source_media_type: &'static str,
    target_media_type: &'static str,
    source_extension: &'static str,
    target_extension: &'static str,
    requires_drive_binding: bool,
}

impl FormatContractSpec {
    /// Derives a versioned format contract from a worker job.
    #[must_use]
    pub fn from_job(job: &FormatJobContract) -> Self {
        let format_kind = job.format_kind();
        let office_media_type = format_kind.canonical_media_type();
        let office_extension = format_kind.canonical_extension();
        let internal_media_type = format_kind.oya_office_internal_media_type();
        let internal_extension = format_kind.oya_office_internal_extension();
        let (source_media_type, target_media_type, source_extension, target_extension) =
            match job.direction() {
                FormatJobDirection::Import => (
                    office_media_type,
                    internal_media_type,
                    office_extension,
                    internal_extension,
                ),
                FormatJobDirection::Export => (
                    internal_media_type,
                    office_media_type,
                    internal_extension,
                    office_extension,
                ),
                FormatJobDirection::RoundTrip => (
                    office_media_type,
                    office_media_type,
                    office_extension,
                    office_extension,
                ),
            };

        Self {
            schema_version: FORMAT_CONTRACT_SCHEMA_VERSION,
            direction: job.direction(),
            format_kind,
            source_media_type,
            target_media_type,
            source_extension,
            target_extension,
            requires_drive_binding: true,
        }
    }

    /// Returns schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Returns job direction.
    #[must_use]
    pub const fn direction(&self) -> FormatJobDirection {
        self.direction
    }

    /// Returns Office format kind.
    #[must_use]
    pub const fn format_kind(&self) -> OfficeFormatKind {
        self.format_kind
    }

    /// Returns source media type.
    #[must_use]
    pub const fn source_media_type(&self) -> &'static str {
        self.source_media_type
    }

    /// Returns target media type.
    #[must_use]
    pub const fn target_media_type(&self) -> &'static str {
        self.target_media_type
    }

    /// Returns source extension.
    #[must_use]
    pub const fn source_extension(&self) -> &'static str {
        self.source_extension
    }

    /// Returns target extension.
    #[must_use]
    pub const fn target_extension(&self) -> &'static str {
        self.target_extension
    }

    /// Returns true when this contract requires a Drive object binding.
    #[must_use]
    pub const fn requires_drive_binding(&self) -> bool {
        self.requires_drive_binding
    }
}

impl OfficeFormatKind {
    /// Returns the internal Oya Office object media type for imported content.
    #[must_use]
    pub const fn oya_office_internal_media_type(self) -> &'static str {
        match self {
            Self::Docx => "application/vnd.oya-office.document+json",
            Self::Xlsx => "application/vnd.oya-office.workbook+json",
            Self::Pptx => "application/vnd.oya-office.presentation+json",
        }
    }

    /// Returns the internal Oya Office object extension for imported content.
    #[must_use]
    pub const fn oya_office_internal_extension(self) -> &'static str {
        match self {
            Self::Docx => "oyadoc",
            Self::Xlsx => "oyasheet",
            Self::Pptx => "oyadeck",
        }
    }
}

/// Score from 0 to 100 for one fidelity metric or an aggregate score.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FidelityScore(u8);

impl FidelityScore {
    /// Creates a score bounded to the inclusive 0-100 range.
    pub fn new(percent: u8) -> Result<Self, FormatDomainError> {
        if percent > 100 {
            return Err(FormatDomainError::new(
                "fidelity score percent must be between 0 and 100",
            ));
        }
        Ok(Self(percent))
    }

    /// Returns the score percentage.
    #[must_use]
    pub const fn as_percent(self) -> u8 {
        self.0
    }
}

/// Scorecard metric axis for DOCX/XLSX/PPTX import/export fidelity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FormatScoreMetric {
    /// Text, runs, paragraphs, strings, and cell values.
    TextContent,
    /// Document/workbook/deck structure.
    Structure,
    /// Styles, themes, and formatting.
    Styling,
    /// Page/slide/grid layout fidelity.
    Layout,
    /// Comments, annotations, and suggestions.
    Comments,
    /// Spreadsheet formula preservation/evaluation.
    Formulas,
    /// Charts, chart ranges, and chart caches.
    Charts,
    /// Images, audio, video, and embedded files.
    Media,
    /// Accessibility metadata and alternate text.
    AccessibilityMetadata,
    /// Security-sensitive sanitization for high-risk packages.
    SecuritySanitization,
}

/// Lossiness severity for one fidelity metric.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LossinessSeverity {
    /// No known loss.
    None,
    /// Minor loss that does not require destructive-save warning.
    Minor,
    /// Major loss requiring a destructive-save warning.
    Major,
    /// Blocking loss that should fail the workflow or force explicit override.
    Blocking,
}

impl LossinessSeverity {
    /// Returns true when the user must receive a destructive-save warning.
    #[must_use]
    pub const fn requires_destructive_save_warning(self) -> bool {
        matches!(self, Self::Major | Self::Blocking)
    }
}

/// Score for one metric axis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatMetricScore {
    metric: FormatScoreMetric,
    score: FidelityScore,
    lossiness: LossinessSeverity,
}

impl FormatMetricScore {
    /// Creates a metric score.
    pub fn new(
        metric: FormatScoreMetric,
        score: FidelityScore,
        lossiness: LossinessSeverity,
    ) -> Result<Self, FormatDomainError> {
        if matches!(lossiness, LossinessSeverity::Blocking) && score.as_percent() > 50 {
            return Err(FormatDomainError::new(
                "blocking lossiness must not report a high fidelity score",
            ));
        }
        Ok(Self {
            metric,
            score,
            lossiness,
        })
    }

    /// Returns metric axis.
    #[must_use]
    pub const fn metric(&self) -> FormatScoreMetric {
        self.metric
    }

    /// Returns metric score.
    #[must_use]
    pub const fn score(&self) -> FidelityScore {
        self.score
    }

    /// Returns lossiness severity.
    #[must_use]
    pub const fn lossiness(&self) -> LossinessSeverity {
        self.lossiness
    }
}

/// Versioned scorecard for one Drive-bound format job.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatScorecard {
    schema_version: u32,
    job: FormatJobContract,
    contract_spec: FormatContractSpec,
    metric_scores: Vec<FormatMetricScore>,
    overall_score: FidelityScore,
    destructive_save_warning_required: bool,
}

impl FormatScorecard {
    /// Creates a scorecard and derives the aggregate score/warning contract.
    pub fn new(
        job: FormatJobContract,
        metric_scores: Vec<FormatMetricScore>,
    ) -> Result<Self, FormatDomainError> {
        if metric_scores.is_empty() {
            return Err(FormatDomainError::new(
                "format scorecard must include at least one metric score",
            ));
        }
        if job.required_isolation_tier() == FormatWorkerIsolationTier::Quarantine
            && !metric_scores
                .iter()
                .any(|metric| metric.metric() == FormatScoreMetric::SecuritySanitization)
        {
            return Err(FormatDomainError::new(
                "quarantine format scorecards must include security sanitization",
            ));
        }

        let total: u16 = metric_scores
            .iter()
            .map(|metric| u16::from(metric.score().as_percent()))
            .sum();
        let average = total / metric_scores.len() as u16;
        let overall_score =
            FidelityScore::new(u8::try_from(average).expect("average is bounded by 100"))?;
        let destructive_save_warning_required = metric_scores
            .iter()
            .any(|metric| metric.lossiness().requires_destructive_save_warning());
        let contract_spec = FormatContractSpec::from_job(&job);

        Ok(Self {
            schema_version: FORMAT_SCORECARD_SCHEMA_VERSION,
            job,
            contract_spec,
            metric_scores,
            overall_score,
            destructive_save_warning_required,
        })
    }

    /// Returns schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Returns the Drive-bound job.
    #[must_use]
    pub const fn job(&self) -> &FormatJobContract {
        &self.job
    }

    /// Returns the derived IO contract spec.
    #[must_use]
    pub const fn contract_spec(&self) -> &FormatContractSpec {
        &self.contract_spec
    }

    /// Returns metric scores.
    #[must_use]
    pub fn metric_scores(&self) -> &[FormatMetricScore] {
        self.metric_scores.as_slice()
    }

    /// Returns aggregate score.
    #[must_use]
    pub const fn overall_score(&self) -> FidelityScore {
        self.overall_score
    }

    /// Returns true when the UI/API must surface destructive-save warning text.
    #[must_use]
    pub const fn destructive_save_warning_required(&self) -> bool {
        self.destructive_save_warning_required
    }
}

/// External peer used only for benchmark comparison and never as a runtime dependency.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BenchmarkPeer {
    /// Google Workspace Docs/Sheets/Slides and Drive export/editing surface.
    GoogleWorkspace,
    /// ONLYOFFICE Docs conversion/co-editing/document-server surface.
    OnlyOffice,
}

const G082_BENCHMARK_PEERS: [BenchmarkPeer; 2] =
    [BenchmarkPeer::GoogleWorkspace, BenchmarkPeer::OnlyOffice];

const GOOGLE_WORKSPACE_REFERENCE_URLS: [&str; 3] = [
    "https://developers.google.com/workspace/drive/api/guides/ref-export-formats",
    "https://developers.google.com/workspace/drive/api/guides/mime-types",
    "https://support.google.com/docs/answer/9406611",
];

const ONLYOFFICE_REFERENCE_URLS: [&str; 4] = [
    "https://api.onlyoffice.com/docs/docs-api/additional-api/conversion-api/",
    "https://api.onlyoffice.com/docs/docs-api/usage-api/config/",
    "https://api.onlyoffice.com/docs/docs-api/get-started/how-it-works/security/",
    "https://api.onlyoffice.com/docs/docs-api/get-started/how-it-works/co-editing/",
];

/// Returns the benchmark peer labels in scope for G082.
#[must_use]
pub const fn g082_benchmark_peers() -> &'static [BenchmarkPeer] {
    G082_BENCHMARK_PEERS.as_slice()
}

impl BenchmarkPeer {
    /// Returns official reference URLs used to design the peer benchmark harness.
    #[must_use]
    pub const fn official_reference_urls(self) -> &'static [&'static str] {
        match self {
            Self::GoogleWorkspace => GOOGLE_WORKSPACE_REFERENCE_URLS.as_slice(),
            Self::OnlyOffice => ONLYOFFICE_REFERENCE_URLS.as_slice(),
        }
    }

    /// Returns credential hooks required for future credentialed execution.
    #[must_use]
    pub const fn credential_hooks(self) -> &'static [BenchmarkCredentialHook] {
        match self {
            Self::GoogleWorkspace => GOOGLE_WORKSPACE_CREDENTIAL_HOOKS.as_slice(),
            Self::OnlyOffice => ONLYOFFICE_CREDENTIAL_HOOKS.as_slice(),
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::GoogleWorkspace => "google-workspace",
            Self::OnlyOffice => "onlyoffice",
        }
    }
}

/// Execution mode for peer benchmark harnesses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BenchmarkExecutionMode {
    /// Credential-free mode that produces plans/contracts without peer calls.
    CredentialFreeDryRun,
    /// Credentialed mode that may call peers through explicit hooks.
    CredentialedExternal,
}

impl BenchmarkExecutionMode {
    /// Returns true when external peer calls may be made.
    #[must_use]
    pub const fn external_calls_allowed(self) -> bool {
        matches!(self, Self::CredentialedExternal)
    }

    /// Returns true when the mode requires credentials before execution.
    #[must_use]
    pub const fn requires_credentials(self) -> bool {
        matches!(self, Self::CredentialedExternal)
    }
}

/// Normalized evidence state used by reproducible benchmark reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BenchmarkReportEvidenceState {
    /// Product-local Rust contracts, docs, or Buck2 targets are verified locally.
    ImplementedLocal,
    /// Credential-free benchmark shape exists; no external peer call or parity claim.
    CredentialFreeDryRun,
    /// Credentialed peer execution is shaped but blocked on explicit credentials/endpoints.
    CredentialedPending,
    /// Metric is defined but representative measurement has not been captured yet.
    MeasuredPending,
    /// Representative measured evidence exists for the exact peer/workflow/fixture/environment.
    Measured,
    /// Explicit out-of-scope, blocked, or future fixture gap that must remain visible.
    SkippedGap,
    /// Row is intentionally not compared for the peer or product slice.
    NotApplicable,
}

const G082_BENCHMARK_EVIDENCE_STATES: [BenchmarkReportEvidenceState; 6] = [
    BenchmarkReportEvidenceState::ImplementedLocal,
    BenchmarkReportEvidenceState::CredentialFreeDryRun,
    BenchmarkReportEvidenceState::CredentialedPending,
    BenchmarkReportEvidenceState::MeasuredPending,
    BenchmarkReportEvidenceState::Measured,
    BenchmarkReportEvidenceState::SkippedGap,
];

/// Returns the evidence states G082 requires benchmark reports to distinguish.
#[must_use]
pub const fn g082_benchmark_evidence_states() -> &'static [BenchmarkReportEvidenceState] {
    G082_BENCHMARK_EVIDENCE_STATES.as_slice()
}

impl BenchmarkReportEvidenceState {
    /// Returns the launch-scorecard label for this state.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ImplementedLocal => "implemented-local",
            Self::CredentialFreeDryRun => "credential-free dry-run",
            Self::CredentialedPending => "credentialed-pending",
            Self::MeasuredPending => "measured-pending",
            Self::Measured => "measured",
            Self::SkippedGap => "skipped/gap",
            Self::NotApplicable => "not-applicable",
        }
    }

    /// Returns true when this state can support a scoped parity claim.
    #[must_use]
    pub const fn can_support_parity_claim(self) -> bool {
        matches!(self, Self::Measured)
    }
}

/// One reproducible benchmark report row for a peer/workflow/fixture outcome.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BenchmarkReportRecord {
    schema_version: u32,
    peer: BenchmarkPeer,
    execution_mode: BenchmarkExecutionMode,
    evidence_state: BenchmarkReportEvidenceState,
    fixture_id: FormatFixtureId,
    corpus_id: FixtureCorpusId,
    tenant_id_class: String,
    command: String,
    environment: String,
    timestamp: String,
    workflow: String,
    scorecard_metric_count: usize,
    parity_claimed: bool,
}

impl BenchmarkReportRecord {
    /// Creates a reproducible report row from a Drive-bound benchmark harness case.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        case: &BenchmarkHarnessCase,
        evidence_state: BenchmarkReportEvidenceState,
        tenant_id_class: impl Into<String>,
        command: impl Into<String>,
        environment: impl Into<String>,
        timestamp: impl Into<String>,
        workflow: impl Into<String>,
        scorecard_metric_count: usize,
        parity_claimed: bool,
    ) -> Result<Self, FormatDomainError> {
        validate_benchmark_evidence_mode(case.execution_mode(), evidence_state)?;
        if matches!(evidence_state, BenchmarkReportEvidenceState::Measured)
            && scorecard_metric_count == 0
        {
            return Err(FormatDomainError::new(
                "measured benchmark reports must include scorecard metrics",
            ));
        }
        if parity_claimed
            && !(case.execution_mode().external_calls_allowed()
                && evidence_state.can_support_parity_claim())
        {
            return Err(FormatDomainError::new(
                "benchmark parity claims require credentialed measured evidence",
            ));
        }

        Ok(Self {
            schema_version: BENCHMARK_REPORT_SCHEMA_VERSION,
            peer: case.peer(),
            execution_mode: case.execution_mode(),
            evidence_state,
            fixture_id: case.job().fixture_id().clone(),
            corpus_id: case.job().corpus_id().clone(),
            tenant_id_class: validate_benchmark_report_text("tenant_id_class", tenant_id_class)?,
            command: validate_benchmark_report_text("command", command)?,
            environment: validate_benchmark_report_text("environment", environment)?,
            timestamp: validate_benchmark_report_text("timestamp", timestamp)?,
            workflow: validate_benchmark_report_text("workflow", workflow)?,
            scorecard_metric_count,
            parity_claimed,
        })
    }

    /// Returns report schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Returns benchmark peer.
    #[must_use]
    pub const fn peer(&self) -> BenchmarkPeer {
        self.peer
    }

    /// Returns execution mode.
    #[must_use]
    pub const fn execution_mode(&self) -> BenchmarkExecutionMode {
        self.execution_mode
    }

    /// Returns normalized evidence state.
    #[must_use]
    pub const fn evidence_state(&self) -> BenchmarkReportEvidenceState {
        self.evidence_state
    }

    /// Returns Drive-bound fixture id.
    #[must_use]
    pub const fn fixture_id(&self) -> &FormatFixtureId {
        &self.fixture_id
    }

    /// Returns fixture corpus id.
    #[must_use]
    pub const fn corpus_id(&self) -> &FixtureCorpusId {
        &self.corpus_id
    }

    /// Returns tenant id class instead of a raw tenant id.
    #[must_use]
    pub fn tenant_id_class(&self) -> &str {
        self.tenant_id_class.as_str()
    }

    /// Returns the exact command used to produce the row.
    #[must_use]
    pub fn command(&self) -> &str {
        self.command.as_str()
    }

    /// Returns environment label.
    #[must_use]
    pub fn environment(&self) -> &str {
        self.environment.as_str()
    }

    /// Returns evidence timestamp.
    #[must_use]
    pub fn timestamp(&self) -> &str {
        self.timestamp.as_str()
    }

    /// Returns workflow label.
    #[must_use]
    pub fn workflow(&self) -> &str {
        self.workflow.as_str()
    }

    /// Returns scorecard metric count captured for the row.
    #[must_use]
    pub const fn scorecard_metric_count(&self) -> usize {
        self.scorecard_metric_count
    }

    /// Returns true when this row explicitly claims parity.
    #[must_use]
    pub const fn parity_claimed(&self) -> bool {
        self.parity_claimed
    }

    /// Returns true when this row is allowed to claim parity.
    #[must_use]
    pub const fn parity_claim_allowed(&self) -> bool {
        self.parity_claimed
            && self.execution_mode.external_calls_allowed()
            && self.evidence_state.can_support_parity_claim()
    }
}

fn validate_benchmark_evidence_mode(
    execution_mode: BenchmarkExecutionMode,
    evidence_state: BenchmarkReportEvidenceState,
) -> Result<(), FormatDomainError> {
    match (execution_mode, evidence_state) {
        (BenchmarkExecutionMode::CredentialFreeDryRun, BenchmarkReportEvidenceState::Measured) => {
            Err(FormatDomainError::new(
                "credential-free dry-run reports cannot be measured evidence",
            ))
        }
        (
            BenchmarkExecutionMode::CredentialFreeDryRun,
            BenchmarkReportEvidenceState::CredentialedPending,
        ) => Err(FormatDomainError::new(
            "credential-free dry-run reports cannot be credentialed-pending",
        )),
        (
            BenchmarkExecutionMode::CredentialedExternal,
            BenchmarkReportEvidenceState::CredentialFreeDryRun,
        ) => Err(FormatDomainError::new(
            "credentialed benchmark reports cannot use credential-free dry-run state",
        )),
        _ => Ok(()),
    }
}

fn validate_benchmark_report_text(
    field_name: &'static str,
    value: impl Into<String>,
) -> Result<String, FormatDomainError> {
    const MAX_REPORT_FIELD_LEN: usize = 512;

    let value = value.into();
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(FormatDomainError::new(format!(
            "benchmark report {field_name} must not be empty"
        )));
    }
    if trimmed.len() > MAX_REPORT_FIELD_LEN {
        return Err(FormatDomainError::new(format!(
            "benchmark report {field_name} is too long"
        )));
    }
    Ok(trimmed.to_owned())
}

/// Credential hook expected by future benchmark executors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BenchmarkCredentialHook {
    /// OAuth access token for Google Workspace APIs.
    GoogleOAuthAccessToken,
    /// OAuth scope/consent marker for Google Drive export and file access.
    GoogleDriveExportScope,
    /// OAuth scope/consent marker for Google Sheets API access.
    GoogleSheetsApiScope,
    /// OAuth scope/consent marker for Google Slides API access.
    GoogleSlidesApiScope,
    /// ONLYOFFICE Document Server base URL.
    OnlyOfficeDocumentServerUrl,
    /// ONLYOFFICE JWT secret for signed document server calls.
    OnlyOfficeJwtSecret,
}

const GOOGLE_WORKSPACE_CREDENTIAL_HOOKS: [BenchmarkCredentialHook; 4] = [
    BenchmarkCredentialHook::GoogleOAuthAccessToken,
    BenchmarkCredentialHook::GoogleDriveExportScope,
    BenchmarkCredentialHook::GoogleSheetsApiScope,
    BenchmarkCredentialHook::GoogleSlidesApiScope,
];

const ONLYOFFICE_CREDENTIAL_HOOKS: [BenchmarkCredentialHook; 2] = [
    BenchmarkCredentialHook::OnlyOfficeDocumentServerUrl,
    BenchmarkCredentialHook::OnlyOfficeJwtSecret,
];

/// One planned peer benchmark case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BenchmarkHarnessCase {
    peer: BenchmarkPeer,
    execution_mode: BenchmarkExecutionMode,
    job: FormatJobContract,
    contract_spec: FormatContractSpec,
}

impl BenchmarkHarnessCase {
    fn new(
        peer: BenchmarkPeer,
        execution_mode: BenchmarkExecutionMode,
        job: FormatJobContract,
    ) -> Self {
        let contract_spec = FormatContractSpec::from_job(&job);
        Self {
            peer,
            execution_mode,
            job,
            contract_spec,
        }
    }

    /// Returns the peer benchmarked by this case.
    #[must_use]
    pub const fn peer(&self) -> BenchmarkPeer {
        self.peer
    }

    /// Returns execution mode.
    #[must_use]
    pub const fn execution_mode(&self) -> BenchmarkExecutionMode {
        self.execution_mode
    }

    /// Returns the Drive-bound format job.
    #[must_use]
    pub const fn job(&self) -> &FormatJobContract {
        &self.job
    }

    /// Returns the derived contract spec.
    #[must_use]
    pub const fn contract_spec(&self) -> &FormatContractSpec {
        &self.contract_spec
    }

    /// Returns true when this case may perform an external peer call.
    #[must_use]
    pub const fn external_calls_allowed(&self) -> bool {
        self.execution_mode.external_calls_allowed()
    }
}

/// Peer benchmark harness design for one fixture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BenchmarkHarnessPlan {
    peer: BenchmarkPeer,
    execution_mode: BenchmarkExecutionMode,
    fixture_id: FormatFixtureId,
    cases: Vec<BenchmarkHarnessCase>,
}

impl BenchmarkHarnessPlan {
    /// Creates a credential-free dry-run plan that never performs peer calls.
    pub fn credential_free_dry_run(
        peer: BenchmarkPeer,
        fixture: &FormatFixtureSpec,
    ) -> Result<Self, FormatDomainError> {
        Self::new(peer, BenchmarkExecutionMode::CredentialFreeDryRun, fixture)
    }

    /// Creates a credentialed plan with explicit external execution hooks.
    pub fn credentialed_execution(
        peer: BenchmarkPeer,
        fixture: &FormatFixtureSpec,
    ) -> Result<Self, FormatDomainError> {
        Self::new(peer, BenchmarkExecutionMode::CredentialedExternal, fixture)
    }

    fn new(
        peer: BenchmarkPeer,
        execution_mode: BenchmarkExecutionMode,
        fixture: &FormatFixtureSpec,
    ) -> Result<Self, FormatDomainError> {
        let mut cases = Vec::with_capacity(3);
        for direction in [
            FormatJobDirection::Import,
            FormatJobDirection::Export,
            FormatJobDirection::RoundTrip,
        ] {
            let job_id = RequestId::new(format!(
                "benchmark-{}-{}",
                peer.slug(),
                direction.benchmark_slug()
            ))
            .map_err(|error| FormatDomainError::new(error.to_string()))?;
            let job = FormatJobContract::from_fixture(job_id, direction, fixture)?;
            cases.push(BenchmarkHarnessCase::new(peer, execution_mode, job));
        }

        Ok(Self {
            peer,
            execution_mode,
            fixture_id: fixture.fixture_id().clone(),
            cases,
        })
    }

    /// Returns benchmark peer.
    #[must_use]
    pub const fn peer(&self) -> BenchmarkPeer {
        self.peer
    }

    /// Returns execution mode.
    #[must_use]
    pub const fn execution_mode(&self) -> BenchmarkExecutionMode {
        self.execution_mode
    }

    /// Returns fixture id.
    #[must_use]
    pub const fn fixture_id(&self) -> &FormatFixtureId {
        &self.fixture_id
    }

    /// Returns planned benchmark cases.
    #[must_use]
    pub fn cases(&self) -> &[BenchmarkHarnessCase] {
        self.cases.as_slice()
    }

    /// Returns true when external peer calls are permitted by this plan.
    #[must_use]
    pub const fn external_calls_allowed(&self) -> bool {
        self.execution_mode.external_calls_allowed()
    }

    /// Returns official reference URLs for this peer.
    #[must_use]
    pub const fn peer_reference_urls(&self) -> &'static [&'static str] {
        self.peer.official_reference_urls()
    }

    /// Returns credential hooks needed for later credentialed execution.
    #[must_use]
    pub const fn credential_hooks(&self) -> &'static [BenchmarkCredentialHook] {
        self.peer.credential_hooks()
    }
}

/// Google Docs / ONLYOFFICE Docs workflow axis for DOCX parity benchmarking.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DocsBenchmarkWorkflow {
    /// Open a Drive-bound document into the editor surface.
    DriveDocumentOpen,
    /// Preserve document comments across collaboration and DOCX workflows.
    CommentPreservation,
    /// Preserve suggestions/track-change semantics across collaboration and DOCX workflows.
    SuggestionTrackChanges,
    /// Preserve monotonic version history and restore/audit semantics.
    VersionHistory,
    /// Import/export/roundtrip a DOCX fixture with fidelity scorecards.
    DocxRoundTrip,
}

const DOCS_BENCHMARK_WORKFLOWS: [DocsBenchmarkWorkflow; 5] = [
    DocsBenchmarkWorkflow::DriveDocumentOpen,
    DocsBenchmarkWorkflow::CommentPreservation,
    DocsBenchmarkWorkflow::SuggestionTrackChanges,
    DocsBenchmarkWorkflow::VersionHistory,
    DocsBenchmarkWorkflow::DocxRoundTrip,
];

const DOCS_DRIVE_OPEN_METRICS: [FormatScoreMetric; 2] =
    [FormatScoreMetric::TextContent, FormatScoreMetric::Structure];
const DOCS_COMMENT_METRICS: [FormatScoreMetric; 2] =
    [FormatScoreMetric::TextContent, FormatScoreMetric::Comments];
const DOCS_SUGGESTION_METRICS: [FormatScoreMetric; 3] = [
    FormatScoreMetric::TextContent,
    FormatScoreMetric::Structure,
    FormatScoreMetric::Comments,
];
const DOCS_VERSION_METRICS: [FormatScoreMetric; 1] = [FormatScoreMetric::Structure];
const DOCS_DOCX_ROUNDTRIP_METRICS: [FormatScoreMetric; 5] = [
    FormatScoreMetric::TextContent,
    FormatScoreMetric::Structure,
    FormatScoreMetric::Styling,
    FormatScoreMetric::Layout,
    FormatScoreMetric::Comments,
];

impl DocsBenchmarkWorkflow {
    /// Returns all Docs benchmark workflows used for peer parity planning.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        DOCS_BENCHMARK_WORKFLOWS.as_slice()
    }

    /// Returns scorecard metrics required before this workflow can claim parity.
    #[must_use]
    pub const fn required_score_metrics(self) -> &'static [FormatScoreMetric] {
        match self {
            Self::DriveDocumentOpen => DOCS_DRIVE_OPEN_METRICS.as_slice(),
            Self::CommentPreservation => DOCS_COMMENT_METRICS.as_slice(),
            Self::SuggestionTrackChanges => DOCS_SUGGESTION_METRICS.as_slice(),
            Self::VersionHistory => DOCS_VERSION_METRICS.as_slice(),
            Self::DocxRoundTrip => DOCS_DOCX_ROUNDTRIP_METRICS.as_slice(),
        }
    }
}

/// Evidence state for one Docs parity benchmark case.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DocsParityEvidenceState {
    /// Credential-free planning evidence only; never enough for parity claims.
    CredentialFreePlanned,
    /// Credentialed hooks are declared, but no peer result has been recorded.
    CredentialedPending,
    /// Credentialed peer and local scorecards have been measured.
    Measured,
    /// The workflow was skipped with an honest gap state in the benchmark report.
    SkippedWithGap,
}

impl DocsParityEvidenceState {
    /// Returns true only for measured credentialed evidence.
    #[must_use]
    pub const fn permits_parity_claim(self) -> bool {
        matches!(self, Self::Measured)
    }
}

/// One Docs benchmark parity case for Google Docs or ONLYOFFICE Docs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocsParityBenchmarkCase {
    peer: BenchmarkPeer,
    execution_mode: BenchmarkExecutionMode,
    workflow: DocsBenchmarkWorkflow,
    format_kind: OfficeFormatKind,
    evidence_state: DocsParityEvidenceState,
}

impl DocsParityBenchmarkCase {
    /// Creates a Docs parity case and fail-closes on non-DOCX or invalid evidence mode.
    pub fn new(
        peer: BenchmarkPeer,
        execution_mode: BenchmarkExecutionMode,
        workflow: DocsBenchmarkWorkflow,
        format_kind: OfficeFormatKind,
        evidence_state: DocsParityEvidenceState,
    ) -> Result<Self, FormatDomainError> {
        if format_kind != OfficeFormatKind::Docx {
            return Err(FormatDomainError::new(
                "Docs parity benchmark cases must use DOCX format",
            ));
        }
        match (execution_mode, evidence_state) {
            (
                BenchmarkExecutionMode::CredentialFreeDryRun,
                DocsParityEvidenceState::CredentialFreePlanned,
            )
            | (
                BenchmarkExecutionMode::CredentialedExternal,
                DocsParityEvidenceState::CredentialedPending
                | DocsParityEvidenceState::Measured
                | DocsParityEvidenceState::SkippedWithGap,
            ) => {}
            _ => {
                return Err(FormatDomainError::new(
                    "Docs parity evidence state is incompatible with execution mode",
                ));
            }
        }

        Ok(Self {
            peer,
            execution_mode,
            workflow,
            format_kind,
            evidence_state,
        })
    }

    /// Returns the benchmark peer.
    #[must_use]
    pub const fn peer(&self) -> BenchmarkPeer {
        self.peer
    }

    /// Returns execution mode.
    #[must_use]
    pub const fn execution_mode(&self) -> BenchmarkExecutionMode {
        self.execution_mode
    }

    /// Returns the Docs workflow axis.
    #[must_use]
    pub const fn workflow(&self) -> DocsBenchmarkWorkflow {
        self.workflow
    }

    /// Returns the only allowed format kind for Docs parity.
    #[must_use]
    pub const fn format_kind(&self) -> OfficeFormatKind {
        self.format_kind
    }

    /// Returns required scorecard metric axes.
    #[must_use]
    pub const fn required_score_metrics(&self) -> &'static [FormatScoreMetric] {
        self.workflow.required_score_metrics()
    }

    /// Returns evidence state.
    #[must_use]
    pub const fn evidence_state(&self) -> DocsParityEvidenceState {
        self.evidence_state
    }

    /// Returns true when this case may call the peer service.
    #[must_use]
    pub const fn external_calls_allowed(&self) -> bool {
        self.execution_mode.external_calls_allowed()
    }

    /// Returns true only after credentialed measured evidence exists.
    #[must_use]
    pub const fn parity_claim_allowed(&self) -> bool {
        self.external_calls_allowed() && self.evidence_state.permits_parity_claim()
    }
}

/// Docs-specific parity matrix across Google Docs and ONLYOFFICE Docs workflows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocsParityBenchmarkMatrix {
    execution_mode: BenchmarkExecutionMode,
    cases: Vec<DocsParityBenchmarkCase>,
}

impl DocsParityBenchmarkMatrix {
    /// Builds a credential-free planning matrix that never permits external calls or parity claims.
    #[must_use]
    pub fn credential_free_dry_run() -> Self {
        Self::new(BenchmarkExecutionMode::CredentialFreeDryRun)
    }

    /// Builds a credentialed matrix whose cases still need measured peer/local evidence.
    #[must_use]
    pub fn credentialed_pending() -> Self {
        Self::new(BenchmarkExecutionMode::CredentialedExternal)
    }

    fn new(execution_mode: BenchmarkExecutionMode) -> Self {
        let evidence_state = match execution_mode {
            BenchmarkExecutionMode::CredentialFreeDryRun => {
                DocsParityEvidenceState::CredentialFreePlanned
            }
            BenchmarkExecutionMode::CredentialedExternal => {
                DocsParityEvidenceState::CredentialedPending
            }
        };
        let mut cases = Vec::with_capacity(10);
        for peer in [BenchmarkPeer::GoogleWorkspace, BenchmarkPeer::OnlyOffice] {
            for workflow in DocsBenchmarkWorkflow::all() {
                cases.push(DocsParityBenchmarkCase {
                    peer,
                    execution_mode,
                    workflow: *workflow,
                    format_kind: OfficeFormatKind::Docx,
                    evidence_state,
                });
            }
        }
        Self {
            execution_mode,
            cases,
        }
    }

    /// Returns execution mode.
    #[must_use]
    pub const fn execution_mode(&self) -> BenchmarkExecutionMode {
        self.execution_mode
    }

    /// Returns all matrix cases.
    #[must_use]
    pub fn cases(&self) -> &[DocsParityBenchmarkCase] {
        self.cases.as_slice()
    }

    /// Returns true when at least one case covers the peer.
    #[must_use]
    pub fn covers_peer(&self, peer: BenchmarkPeer) -> bool {
        self.cases.iter().any(|case| case.peer() == peer)
    }

    /// Returns true when at least one case covers the workflow.
    #[must_use]
    pub fn covers_workflow(&self, workflow: DocsBenchmarkWorkflow) -> bool {
        self.cases.iter().any(|case| case.workflow() == workflow)
    }

    /// Returns true when external peer calls are permitted for this matrix.
    #[must_use]
    pub const fn external_calls_allowed(&self) -> bool {
        self.execution_mode.external_calls_allowed()
    }

    /// Returns true only when every case has measured credentialed evidence.
    #[must_use]
    pub fn parity_claim_allowed(&self) -> bool {
        !self.cases.is_empty()
            && self
                .cases
                .iter()
                .all(DocsParityBenchmarkCase::parity_claim_allowed)
    }

    /// Returns official reference URLs for the selected peer.
    #[must_use]
    pub const fn peer_reference_urls_for(&self, peer: BenchmarkPeer) -> &'static [&'static str] {
        peer.official_reference_urls()
    }

    /// Returns credential hooks for the selected peer.
    #[must_use]
    pub const fn credential_hooks_for(
        &self,
        peer: BenchmarkPeer,
    ) -> &'static [BenchmarkCredentialHook] {
        peer.credential_hooks()
    }
}

/// Google Sheets / ONLYOFFICE Spreadsheet workflow axis for XLSX parity benchmarking.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SheetsBenchmarkWorkflow {
    /// Open a Drive-bound spreadsheet into the editor surface.
    DriveSpreadsheetOpen,
    /// Read and write bounded ranges.
    RangeReadWrite,
    /// Batch update values and formulas through API/SDK routes.
    BatchUpdateValuesAndFormulas,
    /// Recalculate formulas and preserve formula outputs.
    FormulaRecalculation,
    /// Preserve protected range and collaboration authorization behavior.
    ProtectedRangeCollaboration,
    /// Deliver change notifications through webhook/event seams.
    ChangeNotificationWebhook,
    /// Open a secure embedded spreadsheet session.
    EmbeddedSpreadsheet,
    /// Import/export/roundtrip an XLSX fixture with fidelity scorecards.
    XlsxRoundTrip,
}

const SHEETS_BENCHMARK_WORKFLOWS: [SheetsBenchmarkWorkflow; 8] = [
    SheetsBenchmarkWorkflow::DriveSpreadsheetOpen,
    SheetsBenchmarkWorkflow::RangeReadWrite,
    SheetsBenchmarkWorkflow::BatchUpdateValuesAndFormulas,
    SheetsBenchmarkWorkflow::FormulaRecalculation,
    SheetsBenchmarkWorkflow::ProtectedRangeCollaboration,
    SheetsBenchmarkWorkflow::ChangeNotificationWebhook,
    SheetsBenchmarkWorkflow::EmbeddedSpreadsheet,
    SheetsBenchmarkWorkflow::XlsxRoundTrip,
];

const SHEETS_DRIVE_OPEN_METRICS: [FormatScoreMetric; 2] =
    [FormatScoreMetric::Structure, FormatScoreMetric::Layout];
const SHEETS_RANGE_RW_METRICS: [FormatScoreMetric; 3] = [
    FormatScoreMetric::TextContent,
    FormatScoreMetric::Structure,
    FormatScoreMetric::Formulas,
];
const SHEETS_BATCH_UPDATE_METRICS: [FormatScoreMetric; 3] = [
    FormatScoreMetric::TextContent,
    FormatScoreMetric::Formulas,
    FormatScoreMetric::Structure,
];
const SHEETS_FORMULA_METRICS: [FormatScoreMetric; 2] =
    [FormatScoreMetric::Formulas, FormatScoreMetric::TextContent];
const SHEETS_PROTECTED_RANGE_METRICS: [FormatScoreMetric; 2] = [
    FormatScoreMetric::Structure,
    FormatScoreMetric::SecuritySanitization,
];
const SHEETS_WEBHOOK_METRICS: [FormatScoreMetric; 1] = [FormatScoreMetric::Structure];
const SHEETS_EMBED_METRICS: [FormatScoreMetric; 2] = [
    FormatScoreMetric::Layout,
    FormatScoreMetric::SecuritySanitization,
];
const SHEETS_XLSX_ROUNDTRIP_METRICS: [FormatScoreMetric; 6] = [
    FormatScoreMetric::TextContent,
    FormatScoreMetric::Structure,
    FormatScoreMetric::Styling,
    FormatScoreMetric::Layout,
    FormatScoreMetric::Formulas,
    FormatScoreMetric::Charts,
];

const SHEETS_GOOGLE_WORKSPACE_REFERENCE_URLS: [&str; 5] = [
    "https://developers.google.com/workspace/sheets/api/reference/rest/v4/spreadsheets/batchUpdate",
    "https://developers.google.com/workspace/sheets/api/reference/rest/v4/spreadsheets.values/batchGet",
    "https://developers.google.com/workspace/sheets/api/reference/rest/v4/spreadsheets.values/batchUpdate",
    "https://developers.google.com/workspace/sheets/api/guides/batchupdate",
    "https://developers.google.com/workspace/drive/api/guides/push",
];

const SHEETS_ONLYOFFICE_REFERENCE_URLS: [&str; 5] = [
    "https://api.onlyoffice.com/docs/office-api/usage-api/spreadsheet-api/Api/",
    "https://api.onlyoffice.com/docs/plugin-and-macros/interacting-with-editors/spreadsheet-api/Methods/",
    "https://api.onlyoffice.com/docs/docs-api/get-started/how-it-works/co-editing/",
    "https://api.onlyoffice.com/docs/docs-api/usage-api/methods/",
    "https://www.onlyoffice.com/images/templates/whitepapers/pdf/onlyoffice_docs_for_developers.pdf",
];

impl SheetsBenchmarkWorkflow {
    /// Returns all Sheets benchmark workflows used for peer parity planning.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        SHEETS_BENCHMARK_WORKFLOWS.as_slice()
    }

    /// Returns scorecard metrics required before this workflow can claim parity.
    #[must_use]
    pub const fn required_score_metrics(self) -> &'static [FormatScoreMetric] {
        match self {
            Self::DriveSpreadsheetOpen => SHEETS_DRIVE_OPEN_METRICS.as_slice(),
            Self::RangeReadWrite => SHEETS_RANGE_RW_METRICS.as_slice(),
            Self::BatchUpdateValuesAndFormulas => SHEETS_BATCH_UPDATE_METRICS.as_slice(),
            Self::FormulaRecalculation => SHEETS_FORMULA_METRICS.as_slice(),
            Self::ProtectedRangeCollaboration => SHEETS_PROTECTED_RANGE_METRICS.as_slice(),
            Self::ChangeNotificationWebhook => SHEETS_WEBHOOK_METRICS.as_slice(),
            Self::EmbeddedSpreadsheet => SHEETS_EMBED_METRICS.as_slice(),
            Self::XlsxRoundTrip => SHEETS_XLSX_ROUNDTRIP_METRICS.as_slice(),
        }
    }

    /// Returns the local Oya Office contract anchor that owns the workflow.
    #[must_use]
    pub const fn local_contract_anchor(self) -> &'static str {
        match self {
            Self::DriveSpreadsheetOpen => "WorkbookDriveBinding",
            Self::RangeReadWrite => "SheetsRangeRef",
            Self::BatchUpdateValuesAndFormulas => "SheetsBatchUpdateRequest",
            Self::FormulaRecalculation => "SheetsFormulaAutomationRequest",
            Self::ProtectedRangeCollaboration => "ProtectedRange",
            Self::ChangeNotificationWebhook => "SheetsWebhookSubscriptionRequest",
            Self::EmbeddedSpreadsheet => "SheetsEmbedSessionRequest",
            Self::XlsxRoundTrip => "WorkbookXlsxFormatPlan",
        }
    }
}

/// Evidence state for one Sheets parity benchmark case.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SheetsParityEvidenceState {
    /// Credential-free planning evidence only; never enough for parity claims.
    CredentialFreePlanned,
    /// Credentialed hooks are declared, but no peer result has been recorded.
    CredentialedPending,
    /// Credentialed peer and local scorecards have been measured.
    Measured,
    /// The workflow was skipped with an honest gap state in the benchmark report.
    SkippedWithGap,
}

impl SheetsParityEvidenceState {
    /// Returns true only for measured credentialed evidence.
    #[must_use]
    pub const fn permits_parity_claim(self) -> bool {
        matches!(self, Self::Measured)
    }
}

/// One Sheets benchmark parity case for Google Sheets or ONLYOFFICE Spreadsheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetsParityBenchmarkCase {
    peer: BenchmarkPeer,
    execution_mode: BenchmarkExecutionMode,
    workflow: SheetsBenchmarkWorkflow,
    format_kind: OfficeFormatKind,
    evidence_state: SheetsParityEvidenceState,
}

impl SheetsParityBenchmarkCase {
    /// Creates a Sheets parity case and fail-closes on non-XLSX or invalid evidence mode.
    pub fn new(
        peer: BenchmarkPeer,
        execution_mode: BenchmarkExecutionMode,
        workflow: SheetsBenchmarkWorkflow,
        format_kind: OfficeFormatKind,
        evidence_state: SheetsParityEvidenceState,
    ) -> Result<Self, FormatDomainError> {
        if format_kind != OfficeFormatKind::Xlsx {
            return Err(FormatDomainError::new(
                "Sheets parity benchmark cases must use XLSX format",
            ));
        }
        match (execution_mode, evidence_state) {
            (
                BenchmarkExecutionMode::CredentialFreeDryRun,
                SheetsParityEvidenceState::CredentialFreePlanned,
            )
            | (
                BenchmarkExecutionMode::CredentialedExternal,
                SheetsParityEvidenceState::CredentialedPending
                | SheetsParityEvidenceState::Measured
                | SheetsParityEvidenceState::SkippedWithGap,
            ) => {}
            _ => {
                return Err(FormatDomainError::new(
                    "Sheets parity evidence state is incompatible with execution mode",
                ));
            }
        }

        Ok(Self {
            peer,
            execution_mode,
            workflow,
            format_kind,
            evidence_state,
        })
    }

    /// Returns the benchmark peer.
    #[must_use]
    pub const fn peer(&self) -> BenchmarkPeer {
        self.peer
    }

    /// Returns execution mode.
    #[must_use]
    pub const fn execution_mode(&self) -> BenchmarkExecutionMode {
        self.execution_mode
    }

    /// Returns the Sheets workflow axis.
    #[must_use]
    pub const fn workflow(&self) -> SheetsBenchmarkWorkflow {
        self.workflow
    }

    /// Returns the only allowed format kind for Sheets parity.
    #[must_use]
    pub const fn format_kind(&self) -> OfficeFormatKind {
        self.format_kind
    }

    /// Returns required scorecard metric axes.
    #[must_use]
    pub const fn required_score_metrics(&self) -> &'static [FormatScoreMetric] {
        self.workflow.required_score_metrics()
    }

    /// Returns local contract anchor for this workflow.
    #[must_use]
    pub const fn local_contract_anchor(&self) -> &'static str {
        self.workflow.local_contract_anchor()
    }

    /// Returns evidence state.
    #[must_use]
    pub const fn evidence_state(&self) -> SheetsParityEvidenceState {
        self.evidence_state
    }

    /// Returns true when this case may call the peer service.
    #[must_use]
    pub const fn external_calls_allowed(&self) -> bool {
        self.execution_mode.external_calls_allowed()
    }

    /// Returns true only after credentialed measured evidence exists.
    #[must_use]
    pub const fn parity_claim_allowed(&self) -> bool {
        self.external_calls_allowed() && self.evidence_state.permits_parity_claim()
    }
}

/// Sheets-specific parity matrix across Google Sheets and ONLYOFFICE Spreadsheet workflows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetsParityBenchmarkMatrix {
    execution_mode: BenchmarkExecutionMode,
    cases: Vec<SheetsParityBenchmarkCase>,
}

impl SheetsParityBenchmarkMatrix {
    /// Builds a credential-free planning matrix that never permits external calls or parity claims.
    #[must_use]
    pub fn credential_free_dry_run() -> Self {
        Self::new(BenchmarkExecutionMode::CredentialFreeDryRun)
    }

    /// Builds a credentialed matrix whose cases still need measured peer/local evidence.
    #[must_use]
    pub fn credentialed_pending() -> Self {
        Self::new(BenchmarkExecutionMode::CredentialedExternal)
    }

    fn new(execution_mode: BenchmarkExecutionMode) -> Self {
        let evidence_state = match execution_mode {
            BenchmarkExecutionMode::CredentialFreeDryRun => {
                SheetsParityEvidenceState::CredentialFreePlanned
            }
            BenchmarkExecutionMode::CredentialedExternal => {
                SheetsParityEvidenceState::CredentialedPending
            }
        };
        let mut cases = Vec::with_capacity(16);
        for peer in [BenchmarkPeer::GoogleWorkspace, BenchmarkPeer::OnlyOffice] {
            for workflow in SheetsBenchmarkWorkflow::all() {
                cases.push(SheetsParityBenchmarkCase {
                    peer,
                    execution_mode,
                    workflow: *workflow,
                    format_kind: OfficeFormatKind::Xlsx,
                    evidence_state,
                });
            }
        }
        Self {
            execution_mode,
            cases,
        }
    }

    /// Returns execution mode.
    #[must_use]
    pub const fn execution_mode(&self) -> BenchmarkExecutionMode {
        self.execution_mode
    }

    /// Returns all matrix cases.
    #[must_use]
    pub fn cases(&self) -> &[SheetsParityBenchmarkCase] {
        self.cases.as_slice()
    }

    /// Returns true when at least one case covers the peer.
    #[must_use]
    pub fn covers_peer(&self, peer: BenchmarkPeer) -> bool {
        self.cases.iter().any(|case| case.peer() == peer)
    }

    /// Returns true when at least one case covers the workflow.
    #[must_use]
    pub fn covers_workflow(&self, workflow: SheetsBenchmarkWorkflow) -> bool {
        self.cases.iter().any(|case| case.workflow() == workflow)
    }

    /// Returns true when external peer calls are permitted for this matrix.
    #[must_use]
    pub const fn external_calls_allowed(&self) -> bool {
        self.execution_mode.external_calls_allowed()
    }

    /// Returns true only when every case has measured credentialed evidence.
    #[must_use]
    pub fn parity_claim_allowed(&self) -> bool {
        !self.cases.is_empty()
            && self
                .cases
                .iter()
                .all(SheetsParityBenchmarkCase::parity_claim_allowed)
    }

    /// Returns official reference URLs for the selected peer's spreadsheet surface.
    #[must_use]
    pub const fn peer_reference_urls_for(&self, peer: BenchmarkPeer) -> &'static [&'static str] {
        match peer {
            BenchmarkPeer::GoogleWorkspace => SHEETS_GOOGLE_WORKSPACE_REFERENCE_URLS.as_slice(),
            BenchmarkPeer::OnlyOffice => SHEETS_ONLYOFFICE_REFERENCE_URLS.as_slice(),
        }
    }

    /// Returns credential hooks for the selected peer.
    #[must_use]
    pub const fn credential_hooks_for(
        &self,
        peer: BenchmarkPeer,
    ) -> &'static [BenchmarkCredentialHook] {
        peer.credential_hooks()
    }
}

/// Google Slides / ONLYOFFICE Presentation workflow axis for PPTX parity benchmarking.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SlidesBenchmarkWorkflow {
    /// Open a Drive-bound presentation into the editor surface.
    DrivePresentationOpen,
    /// Create slides, apply layouts, and reorder slides.
    SlideCreateAndReorder,
    /// Edit shape text, styling, and geometry.
    ShapeTextAndLayoutEditing,
    /// Insert image/media assets and preserve alternate text.
    MediaInsertionAndAltText,
    /// Preserve presenter/speaker notes.
    SpeakerNotesPreservation,
    /// Apply realtime collaborative slide edits with target/version checks.
    CollaborativeSlideEdit,
    /// Open a secure embedded presentation session.
    EmbeddedPresentation,
    /// Import/export/roundtrip a PPTX fixture with fidelity scorecards.
    PptxRoundTrip,
}

const SLIDES_BENCHMARK_WORKFLOWS: [SlidesBenchmarkWorkflow; 8] = [
    SlidesBenchmarkWorkflow::DrivePresentationOpen,
    SlidesBenchmarkWorkflow::SlideCreateAndReorder,
    SlidesBenchmarkWorkflow::ShapeTextAndLayoutEditing,
    SlidesBenchmarkWorkflow::MediaInsertionAndAltText,
    SlidesBenchmarkWorkflow::SpeakerNotesPreservation,
    SlidesBenchmarkWorkflow::CollaborativeSlideEdit,
    SlidesBenchmarkWorkflow::EmbeddedPresentation,
    SlidesBenchmarkWorkflow::PptxRoundTrip,
];

const SLIDES_DRIVE_OPEN_METRICS: [FormatScoreMetric; 2] =
    [FormatScoreMetric::Structure, FormatScoreMetric::Layout];
const SLIDES_CREATE_REORDER_METRICS: [FormatScoreMetric; 2] =
    [FormatScoreMetric::Structure, FormatScoreMetric::Layout];
const SLIDES_SHAPE_TEXT_LAYOUT_METRICS: [FormatScoreMetric; 4] = [
    FormatScoreMetric::TextContent,
    FormatScoreMetric::Structure,
    FormatScoreMetric::Styling,
    FormatScoreMetric::Layout,
];
const SLIDES_MEDIA_ALT_TEXT_METRICS: [FormatScoreMetric; 3] = [
    FormatScoreMetric::Media,
    FormatScoreMetric::AccessibilityMetadata,
    FormatScoreMetric::Layout,
];
const SLIDES_SPEAKER_NOTES_METRICS: [FormatScoreMetric; 2] =
    [FormatScoreMetric::TextContent, FormatScoreMetric::Structure];
const SLIDES_COLLAB_EDIT_METRICS: [FormatScoreMetric; 2] = [
    FormatScoreMetric::Structure,
    FormatScoreMetric::SecuritySanitization,
];
const SLIDES_EMBED_METRICS: [FormatScoreMetric; 2] = [
    FormatScoreMetric::Layout,
    FormatScoreMetric::SecuritySanitization,
];
const SLIDES_PPTX_ROUNDTRIP_METRICS: [FormatScoreMetric; 7] = [
    FormatScoreMetric::TextContent,
    FormatScoreMetric::Structure,
    FormatScoreMetric::Styling,
    FormatScoreMetric::Layout,
    FormatScoreMetric::Media,
    FormatScoreMetric::AccessibilityMetadata,
    FormatScoreMetric::Comments,
];

const SLIDES_GOOGLE_WORKSPACE_REFERENCE_URLS: [&str; 5] = [
    "https://developers.google.com/workspace/slides/api/reference/rest/v1/presentations",
    "https://developers.google.com/workspace/slides/api/reference/rest/v1/presentations/batchUpdate",
    "https://developers.google.com/workspace/slides/api/guides/add-image",
    "https://developers.google.com/workspace/slides/api/guides/styling",
    "https://developers.google.com/workspace/drive/api/guides/ref-export-formats",
];

const SLIDES_ONLYOFFICE_REFERENCE_URLS: [&str; 5] = [
    "https://api.onlyoffice.com/docs/office-api/usage-api/presentation-api/Api/",
    "https://api.onlyoffice.com/docs/plugin-and-macros/interacting-with-editors/presentation-api/Methods/",
    "https://api.onlyoffice.com/docs/docs-api/usage-api/config/",
    "https://api.onlyoffice.com/docs/docs-api/get-started/how-it-works/co-editing/",
    "https://api.onlyoffice.com/docs/docs-api/additional-api/conversion-api/",
];

impl SlidesBenchmarkWorkflow {
    /// Returns all Slides benchmark workflows used for peer parity planning.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        SLIDES_BENCHMARK_WORKFLOWS.as_slice()
    }

    /// Returns scorecard metrics required before this workflow can claim parity.
    #[must_use]
    pub const fn required_score_metrics(self) -> &'static [FormatScoreMetric] {
        match self {
            Self::DrivePresentationOpen => SLIDES_DRIVE_OPEN_METRICS.as_slice(),
            Self::SlideCreateAndReorder => SLIDES_CREATE_REORDER_METRICS.as_slice(),
            Self::ShapeTextAndLayoutEditing => SLIDES_SHAPE_TEXT_LAYOUT_METRICS.as_slice(),
            Self::MediaInsertionAndAltText => SLIDES_MEDIA_ALT_TEXT_METRICS.as_slice(),
            Self::SpeakerNotesPreservation => SLIDES_SPEAKER_NOTES_METRICS.as_slice(),
            Self::CollaborativeSlideEdit => SLIDES_COLLAB_EDIT_METRICS.as_slice(),
            Self::EmbeddedPresentation => SLIDES_EMBED_METRICS.as_slice(),
            Self::PptxRoundTrip => SLIDES_PPTX_ROUNDTRIP_METRICS.as_slice(),
        }
    }

    /// Returns the local Oya Office contract anchor that owns the workflow.
    #[must_use]
    pub const fn local_contract_anchor(self) -> &'static str {
        match self {
            Self::DrivePresentationOpen => "DeckDriveBinding",
            Self::SlideCreateAndReorder => "DeckModel",
            Self::ShapeTextAndLayoutEditing => "SlideShape",
            Self::MediaInsertionAndAltText => "SlideMediaAsset",
            Self::SpeakerNotesPreservation => "Slide",
            Self::CollaborativeSlideEdit => "CollaborativeSlideEdit",
            Self::EmbeddedPresentation => "DriveWorkspaceNavigationItem",
            Self::PptxRoundTrip => "DeckPptxFormatPlan",
        }
    }
}

/// Evidence state for one Slides parity benchmark case.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SlidesParityEvidenceState {
    /// Credential-free planning evidence only; never enough for parity claims.
    CredentialFreePlanned,
    /// Credentialed hooks are declared, but no peer result has been recorded.
    CredentialedPending,
    /// Credentialed peer and local scorecards have been measured.
    Measured,
    /// The workflow was skipped with an honest gap state in the benchmark report.
    SkippedWithGap,
}

impl SlidesParityEvidenceState {
    /// Returns true only for measured credentialed evidence.
    #[must_use]
    pub const fn permits_parity_claim(self) -> bool {
        matches!(self, Self::Measured)
    }
}

/// One Slides benchmark parity case for Google Slides or ONLYOFFICE Presentation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlidesParityBenchmarkCase {
    peer: BenchmarkPeer,
    execution_mode: BenchmarkExecutionMode,
    workflow: SlidesBenchmarkWorkflow,
    format_kind: OfficeFormatKind,
    evidence_state: SlidesParityEvidenceState,
}

impl SlidesParityBenchmarkCase {
    /// Creates a Slides parity case and fail-closes on non-PPTX or invalid evidence mode.
    pub fn new(
        peer: BenchmarkPeer,
        execution_mode: BenchmarkExecutionMode,
        workflow: SlidesBenchmarkWorkflow,
        format_kind: OfficeFormatKind,
        evidence_state: SlidesParityEvidenceState,
    ) -> Result<Self, FormatDomainError> {
        if format_kind != OfficeFormatKind::Pptx {
            return Err(FormatDomainError::new(
                "Slides parity benchmark cases must use PPTX format",
            ));
        }
        match (execution_mode, evidence_state) {
            (
                BenchmarkExecutionMode::CredentialFreeDryRun,
                SlidesParityEvidenceState::CredentialFreePlanned,
            )
            | (
                BenchmarkExecutionMode::CredentialedExternal,
                SlidesParityEvidenceState::CredentialedPending
                | SlidesParityEvidenceState::Measured
                | SlidesParityEvidenceState::SkippedWithGap,
            ) => {}
            _ => {
                return Err(FormatDomainError::new(
                    "Slides parity evidence state is incompatible with execution mode",
                ));
            }
        }

        Ok(Self {
            peer,
            execution_mode,
            workflow,
            format_kind,
            evidence_state,
        })
    }

    /// Returns the benchmark peer.
    #[must_use]
    pub const fn peer(&self) -> BenchmarkPeer {
        self.peer
    }

    /// Returns execution mode.
    #[must_use]
    pub const fn execution_mode(&self) -> BenchmarkExecutionMode {
        self.execution_mode
    }

    /// Returns the Slides workflow axis.
    #[must_use]
    pub const fn workflow(&self) -> SlidesBenchmarkWorkflow {
        self.workflow
    }

    /// Returns the only allowed format kind for Slides parity.
    #[must_use]
    pub const fn format_kind(&self) -> OfficeFormatKind {
        self.format_kind
    }

    /// Returns required scorecard metric axes.
    #[must_use]
    pub const fn required_score_metrics(&self) -> &'static [FormatScoreMetric] {
        self.workflow.required_score_metrics()
    }

    /// Returns local contract anchor for this workflow.
    #[must_use]
    pub const fn local_contract_anchor(&self) -> &'static str {
        self.workflow.local_contract_anchor()
    }

    /// Returns evidence state.
    #[must_use]
    pub const fn evidence_state(&self) -> SlidesParityEvidenceState {
        self.evidence_state
    }

    /// Returns true when this case may call the peer service.
    #[must_use]
    pub const fn external_calls_allowed(&self) -> bool {
        self.execution_mode.external_calls_allowed()
    }

    /// Returns true only after credentialed measured evidence exists.
    #[must_use]
    pub const fn parity_claim_allowed(&self) -> bool {
        self.external_calls_allowed() && self.evidence_state.permits_parity_claim()
    }
}

/// Slides-specific parity matrix across Google Slides and ONLYOFFICE Presentation workflows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlidesParityBenchmarkMatrix {
    execution_mode: BenchmarkExecutionMode,
    cases: Vec<SlidesParityBenchmarkCase>,
}

impl SlidesParityBenchmarkMatrix {
    /// Builds a credential-free planning matrix that never permits external calls or parity claims.
    #[must_use]
    pub fn credential_free_dry_run() -> Self {
        Self::new(BenchmarkExecutionMode::CredentialFreeDryRun)
    }

    /// Builds a credentialed matrix whose cases still need measured peer/local evidence.
    #[must_use]
    pub fn credentialed_pending() -> Self {
        Self::new(BenchmarkExecutionMode::CredentialedExternal)
    }

    fn new(execution_mode: BenchmarkExecutionMode) -> Self {
        let evidence_state = match execution_mode {
            BenchmarkExecutionMode::CredentialFreeDryRun => {
                SlidesParityEvidenceState::CredentialFreePlanned
            }
            BenchmarkExecutionMode::CredentialedExternal => {
                SlidesParityEvidenceState::CredentialedPending
            }
        };
        let mut cases = Vec::with_capacity(16);
        for peer in [BenchmarkPeer::GoogleWorkspace, BenchmarkPeer::OnlyOffice] {
            for workflow in SlidesBenchmarkWorkflow::all() {
                cases.push(SlidesParityBenchmarkCase {
                    peer,
                    execution_mode,
                    workflow: *workflow,
                    format_kind: OfficeFormatKind::Pptx,
                    evidence_state,
                });
            }
        }
        Self {
            execution_mode,
            cases,
        }
    }

    /// Returns execution mode.
    #[must_use]
    pub const fn execution_mode(&self) -> BenchmarkExecutionMode {
        self.execution_mode
    }

    /// Returns all matrix cases.
    #[must_use]
    pub fn cases(&self) -> &[SlidesParityBenchmarkCase] {
        self.cases.as_slice()
    }

    /// Returns true when at least one case covers the peer.
    #[must_use]
    pub fn covers_peer(&self, peer: BenchmarkPeer) -> bool {
        self.cases.iter().any(|case| case.peer() == peer)
    }

    /// Returns true when at least one case covers the workflow.
    #[must_use]
    pub fn covers_workflow(&self, workflow: SlidesBenchmarkWorkflow) -> bool {
        self.cases.iter().any(|case| case.workflow() == workflow)
    }

    /// Returns true when external peer calls are permitted for this matrix.
    #[must_use]
    pub const fn external_calls_allowed(&self) -> bool {
        self.execution_mode.external_calls_allowed()
    }

    /// Returns true only when every case has measured credentialed evidence.
    #[must_use]
    pub fn parity_claim_allowed(&self) -> bool {
        !self.cases.is_empty()
            && self
                .cases
                .iter()
                .all(SlidesParityBenchmarkCase::parity_claim_allowed)
    }

    /// Returns official reference URLs for the selected peer's presentation surface.
    #[must_use]
    pub const fn peer_reference_urls_for(&self, peer: BenchmarkPeer) -> &'static [&'static str] {
        match peer {
            BenchmarkPeer::GoogleWorkspace => SLIDES_GOOGLE_WORKSPACE_REFERENCE_URLS.as_slice(),
            BenchmarkPeer::OnlyOffice => SLIDES_ONLYOFFICE_REFERENCE_URLS.as_slice(),
        }
    }

    /// Returns credential hooks for the selected peer.
    #[must_use]
    pub const fn credential_hooks_for(
        &self,
        peer: BenchmarkPeer,
    ) -> &'static [BenchmarkCredentialHook] {
        peer.credential_hooks()
    }
}

impl FormatJobDirection {
    fn benchmark_slug(self) -> &'static str {
        match self {
            Self::Import => "import",
            Self::Export => "export",
            Self::RoundTrip => "roundtrip",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ARCHITECTURE_LAYER, CRATE_NAME, VERTICAL_SLICE};
    use super::{
        BENCHMARK_REPORT_SCHEMA_VERSION, BenchmarkCredentialHook, BenchmarkExecutionMode,
        BenchmarkHarnessPlan, BenchmarkPeer, BenchmarkReportEvidenceState, BenchmarkReportRecord,
        DocsBenchmarkWorkflow, DocsParityBenchmarkCase, DocsParityBenchmarkMatrix,
        DocsParityEvidenceState, FidelityScore, FixtureComplexity, FixtureCorpusId, FixtureFeature,
        FormatBenchmarkGateKind, FormatContractSpec, FormatFixtureBinding, FormatFixtureId,
        FormatFixtureSpec, FormatJobContract, FormatJobDirection, FormatMetricScore,
        FormatScoreMetric, FormatScorecard, FormatWorkerIsolationTier,
        G082_FORMAT_BENCHMARK_CONTRACT_VERSION, LossinessSeverity, OfficeFormatKind,
        SheetsBenchmarkWorkflow, SheetsParityBenchmarkCase, SheetsParityBenchmarkMatrix,
        SheetsParityEvidenceState, SlidesBenchmarkWorkflow, SlidesParityBenchmarkCase,
        SlidesParityBenchmarkMatrix, SlidesParityEvidenceState, g082_benchmark_evidence_states,
        g082_benchmark_peers, g082_format_benchmark_gates, g082_required_ooxml_formats,
    };
    use oya_office_drive_domain::{DriveObjectBinding, DriveObjectKind};
    use oya_office_kernel::{DataClass, ObjectId, RequestId, TenantId};

    #[test]
    fn scaffold_identity_is_declared() {
        assert!(!ARCHITECTURE_LAYER.is_empty());
        assert!(!CRATE_NAME.is_empty());
        assert!(!VERTICAL_SLICE.is_empty());
    }

    #[test]
    fn g082_format_benchmark_contract_covers_fixture_taxonomy_and_peer_schema() {
        assert_eq!(
            G082_FORMAT_BENCHMARK_CONTRACT_VERSION,
            "g082-format-benchmark-v1"
        );

        let gates = g082_format_benchmark_gates();
        for required in [
            FormatBenchmarkGateKind::FixtureProvenance,
            FormatBenchmarkGateKind::FixtureTaxonomy,
            FormatBenchmarkGateKind::OoxmlFormatCoverage,
            FormatBenchmarkGateKind::SecurityQuarantine,
            FormatBenchmarkGateKind::ScorecardSchema,
            FormatBenchmarkGateKind::PeerBenchmarkWorkflow,
            FormatBenchmarkGateKind::BenchmarkEvidenceState,
            FormatBenchmarkGateKind::NoParityClaimWithoutMeasuredMetrics,
        ] {
            assert!(
                gates
                    .iter()
                    .any(|gate| gate.kind() == required && gate.launch_blocking()),
                "missing launch-blocking G082 gate: {required:?}"
            );
        }
        assert!(
            gates
                .iter()
                .any(|gate| gate.evidence().contains("Google Workspace")
                    && gate.evidence().contains("ONLYOFFICE"))
        );

        assert_eq!(
            g082_required_ooxml_formats(),
            [
                OfficeFormatKind::Docx,
                OfficeFormatKind::Xlsx,
                OfficeFormatKind::Pptx
            ]
        );
        assert_eq!(
            g082_benchmark_peers(),
            [BenchmarkPeer::GoogleWorkspace, BenchmarkPeer::OnlyOffice]
        );
        assert!(
            g082_benchmark_evidence_states()
                .contains(&BenchmarkReportEvidenceState::CredentialFreeDryRun)
        );
        assert!(
            g082_benchmark_evidence_states()
                .contains(&BenchmarkReportEvidenceState::CredentialedPending)
        );
        assert!(g082_benchmark_evidence_states().contains(&BenchmarkReportEvidenceState::Measured));
        assert!(
            g082_benchmark_evidence_states().contains(&BenchmarkReportEvidenceState::SkippedGap)
        );

        let dry_run_plan = BenchmarkHarnessPlan::credential_free_dry_run(
            BenchmarkPeer::GoogleWorkspace,
            &docx_fixture(),
        )
        .expect("dry-run plan");
        let dry_run_case = dry_run_plan.cases().first().expect("benchmark case");
        assert!(
            BenchmarkReportRecord::new(
                dry_run_case,
                BenchmarkReportEvidenceState::CredentialFreeDryRun,
                "synthetic-non-prod-tenant",
                "buck2 test //crates/oya-office-format-domain:test",
                "local-macos-arm64-rust-1.96.0",
                "2026-06-03T00:00:00Z",
                "G082 DOCX dry-run parity guard",
                4,
                true,
            )
            .is_err(),
            "dry-run reports must not claim parity"
        );

        let credentialed_plan = BenchmarkHarnessPlan::credentialed_execution(
            BenchmarkPeer::OnlyOffice,
            &docx_fixture(),
        )
        .expect("credentialed plan");
        let credentialed_case = credentialed_plan.cases().first().expect("benchmark case");
        assert!(
            BenchmarkReportRecord::new(
                credentialed_case,
                BenchmarkReportEvidenceState::Measured,
                "synthetic-non-prod-tenant",
                "omx benchmark peer --fixture fixture-docx-comments --peer onlyoffice",
                "isolated-staging-peer-benchmark-cell",
                "2026-06-03T00:00:00Z",
                "G082 DOCX measured peer roundtrip",
                0,
                true,
            )
            .is_err(),
            "measured parity reports must include scorecard metrics"
        );

        let quarantined_fixture = FormatFixtureSpec::new(
            FormatFixtureId::new("fixture-docx-g082-zip-bomb").expect("valid fixture id"),
            FixtureCorpusId::new("corpus-ooxml-adversarial").expect("valid corpus id"),
            FormatFixtureBinding::new(
                drive_binding("tenant-alpha", "doc-g082", DriveObjectKind::Document),
                OfficeFormatKind::Docx,
            )
            .expect("fixture binding"),
            FixtureComplexity::Adversarial,
            vec![
                FixtureFeature::ParagraphText,
                FixtureFeature::ZipContainerEdges,
            ],
        )
        .expect("fixture spec");

        assert_eq!(
            quarantined_fixture.required_isolation_tier(),
            FormatWorkerIsolationTier::Quarantine
        );
    }

    #[test]
    fn fixture_taxonomy_maps_ooxml_formats_to_drive_kinds() {
        assert_eq!(
            OfficeFormatKind::Docx.drive_object_kind(),
            DriveObjectKind::Document
        );
        assert_eq!(
            OfficeFormatKind::Xlsx.drive_object_kind(),
            DriveObjectKind::Spreadsheet
        );
        assert_eq!(
            OfficeFormatKind::Pptx.drive_object_kind(),
            DriveObjectKind::Presentation
        );
        assert_eq!(
            OfficeFormatKind::Docx.canonical_media_type(),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        );
    }

    #[test]
    fn fixture_binding_rejects_mismatched_drive_object_kind() {
        let binding = drive_binding("tenant-alpha", "sheet-1", DriveObjectKind::Spreadsheet);

        assert!(FormatFixtureBinding::new(binding, OfficeFormatKind::Docx).is_err());
    }

    #[test]
    fn fixture_import_export_and_roundtrip_jobs_keep_drive_binding() {
        let fixture = docx_fixture();

        for direction in [
            FormatJobDirection::Import,
            FormatJobDirection::Export,
            FormatJobDirection::RoundTrip,
        ] {
            let job = FormatJobContract::from_fixture(
                RequestId::new(format!("format-job-{direction:?}")).expect("valid request id"),
                direction,
                &fixture,
            )
            .expect("job contract");
            assert_eq!(job.tenant_id().as_str(), "tenant-alpha");
            assert_eq!(job.object_id().as_str(), "doc-1");
            assert_eq!(job.format_kind(), OfficeFormatKind::Docx);
        }
    }

    #[test]
    fn macro_fixture_requires_quarantine_isolation() {
        let fixture = FormatFixtureSpec::new(
            FormatFixtureId::new("fixture-docx-macro").expect("valid fixture id"),
            FixtureCorpusId::new("corpus-ooxml-adversarial").expect("valid corpus id"),
            FormatFixtureBinding::new(
                drive_binding("tenant-alpha", "doc-2", DriveObjectKind::Document),
                OfficeFormatKind::Docx,
            )
            .expect("fixture binding"),
            FixtureComplexity::Adversarial,
            vec![FixtureFeature::ParagraphText, FixtureFeature::MacroEnabled],
        )
        .expect("fixture spec");

        assert_eq!(
            fixture.required_isolation_tier(),
            FormatWorkerIsolationTier::Quarantine
        );
    }

    #[test]
    fn contract_spec_derives_io_contract_from_job_direction() {
        let job = FormatJobContract::from_fixture(
            RequestId::new("format-job-contract-export").expect("valid request id"),
            FormatJobDirection::Export,
            &docx_fixture(),
        )
        .expect("job contract");

        let spec = FormatContractSpec::from_job(&job);

        assert_eq!(spec.schema_version(), 1);
        assert_eq!(spec.direction(), FormatJobDirection::Export);
        assert_eq!(spec.format_kind(), OfficeFormatKind::Docx);
        assert_eq!(
            spec.source_media_type(),
            "application/vnd.oya-office.document+json"
        );
        assert_eq!(
            spec.target_media_type(),
            OfficeFormatKind::Docx.canonical_media_type()
        );
        assert!(spec.requires_drive_binding());
    }

    #[test]
    fn scorecard_schema_computes_overall_score_and_warning_contract() {
        let job = FormatJobContract::from_fixture(
            RequestId::new("format-job-scorecard-roundtrip").expect("valid request id"),
            FormatJobDirection::RoundTrip,
            &docx_fixture(),
        )
        .expect("job contract");

        let scorecard = FormatScorecard::new(
            job,
            vec![
                FormatMetricScore::new(
                    FormatScoreMetric::TextContent,
                    FidelityScore::new(100).expect("score"),
                    LossinessSeverity::None,
                )
                .expect("metric score"),
                FormatMetricScore::new(
                    FormatScoreMetric::Comments,
                    FidelityScore::new(72).expect("score"),
                    LossinessSeverity::Major,
                )
                .expect("metric score"),
            ],
        )
        .expect("scorecard");

        assert_eq!(scorecard.schema_version(), 1);
        assert_eq!(scorecard.overall_score().as_percent(), 86);
        assert!(scorecard.destructive_save_warning_required());
    }

    #[test]
    fn quarantine_scorecards_must_include_security_sanitization_metric() {
        let fixture = FormatFixtureSpec::new(
            FormatFixtureId::new("fixture-docx-external-links").expect("valid fixture id"),
            FixtureCorpusId::new("corpus-ooxml-adversarial").expect("valid corpus id"),
            FormatFixtureBinding::new(
                drive_binding("tenant-alpha", "doc-3", DriveObjectKind::Document),
                OfficeFormatKind::Docx,
            )
            .expect("fixture binding"),
            FixtureComplexity::Adversarial,
            vec![FixtureFeature::ParagraphText, FixtureFeature::ExternalLinks],
        )
        .expect("fixture spec");
        let job = FormatJobContract::from_fixture(
            RequestId::new("format-job-scorecard-quarantine").expect("valid request id"),
            FormatJobDirection::Import,
            &fixture,
        )
        .expect("job contract");

        assert!(
            FormatScorecard::new(
                job,
                vec![
                    FormatMetricScore::new(
                        FormatScoreMetric::TextContent,
                        FidelityScore::new(95).expect("score"),
                        LossinessSeverity::None,
                    )
                    .expect("metric score")
                ]
            )
            .is_err()
        );
    }

    #[test]
    fn credential_free_google_workspace_plan_has_no_external_calls() {
        let plan = BenchmarkHarnessPlan::credential_free_dry_run(
            BenchmarkPeer::GoogleWorkspace,
            &docx_fixture(),
        )
        .expect("dry-run plan");

        assert_eq!(
            plan.execution_mode(),
            BenchmarkExecutionMode::CredentialFreeDryRun
        );
        assert!(!plan.external_calls_allowed());
        assert_eq!(plan.cases().len(), 3);
        assert!(
            plan.peer_reference_urls()
                .iter()
                .any(|url| url.contains("developers.google.com/workspace/drive"))
        );
        assert!(
            plan.credential_hooks()
                .contains(&BenchmarkCredentialHook::GoogleOAuthAccessToken)
        );
    }

    #[test]
    fn onlyoffice_credentialed_plan_declares_document_server_and_jwt_hooks() {
        let plan = BenchmarkHarnessPlan::credentialed_execution(
            BenchmarkPeer::OnlyOffice,
            &docx_fixture(),
        )
        .expect("credentialed plan");

        assert_eq!(
            plan.execution_mode(),
            BenchmarkExecutionMode::CredentialedExternal
        );
        assert!(plan.external_calls_allowed());
        assert!(
            plan.credential_hooks()
                .contains(&BenchmarkCredentialHook::OnlyOfficeDocumentServerUrl)
        );
        assert!(
            plan.credential_hooks()
                .contains(&BenchmarkCredentialHook::OnlyOfficeJwtSecret)
        );
    }

    #[test]
    fn benchmark_report_schema_captures_reproducibility_fields() {
        let plan = BenchmarkHarnessPlan::credential_free_dry_run(
            BenchmarkPeer::GoogleWorkspace,
            &docx_fixture(),
        )
        .expect("dry-run plan");
        let case = plan.cases().first().expect("benchmark case");

        let report = BenchmarkReportRecord::new(
            case,
            BenchmarkReportEvidenceState::CredentialFreeDryRun,
            "synthetic-non-prod-tenant",
            "buck2 test //crates/oya-office-format-domain:test",
            "local-macos-arm64-rust-1.96.0",
            "2026-06-03T00:00:00Z",
            "DOCX import/export/roundtrip dry run",
            4,
            false,
        )
        .expect("benchmark report");

        assert_eq!(report.schema_version(), BENCHMARK_REPORT_SCHEMA_VERSION);
        assert_eq!(report.peer(), BenchmarkPeer::GoogleWorkspace);
        assert_eq!(
            report.execution_mode(),
            BenchmarkExecutionMode::CredentialFreeDryRun
        );
        assert_eq!(
            report.evidence_state(),
            BenchmarkReportEvidenceState::CredentialFreeDryRun
        );
        assert_eq!(report.fixture_id().as_str(), "fixture-docx-comments");
        assert_eq!(report.corpus_id().as_str(), "corpus-ooxml-parity");
        assert_eq!(report.tenant_id_class(), "synthetic-non-prod-tenant");
        assert_eq!(
            report.command(),
            "buck2 test //crates/oya-office-format-domain:test"
        );
        assert_eq!(report.environment(), "local-macos-arm64-rust-1.96.0");
        assert_eq!(report.timestamp(), "2026-06-03T00:00:00Z");
        assert_eq!(report.workflow(), "DOCX import/export/roundtrip dry run");
        assert_eq!(report.scorecard_metric_count(), 4);
        assert!(!report.parity_claimed());
        assert!(!report.parity_claim_allowed());
    }

    #[test]
    fn benchmark_report_schema_rejects_measured_without_metrics() {
        let plan = BenchmarkHarnessPlan::credentialed_execution(
            BenchmarkPeer::OnlyOffice,
            &docx_fixture(),
        )
        .expect("credentialed plan");
        let case = plan.cases().first().expect("benchmark case");

        let error = BenchmarkReportRecord::new(
            case,
            BenchmarkReportEvidenceState::Measured,
            "synthetic-non-prod-tenant",
            "buck2 test //crates/oya-office-format-domain:test",
            "staging-peer-benchmark-cell",
            "2026-06-03T00:00:00Z",
            "DOCX roundtrip measured peer run",
            0,
            false,
        )
        .expect_err("measured reports need metrics");

        assert!(error.message().contains("scorecard metrics"));
    }

    #[test]
    fn benchmark_report_schema_blocks_dry_run_parity_claims() {
        let dry_run_plan = BenchmarkHarnessPlan::credential_free_dry_run(
            BenchmarkPeer::GoogleWorkspace,
            &docx_fixture(),
        )
        .expect("dry-run plan");
        let dry_run_case = dry_run_plan.cases().first().expect("benchmark case");

        assert!(
            BenchmarkReportRecord::new(
                dry_run_case,
                BenchmarkReportEvidenceState::CredentialFreeDryRun,
                "synthetic-non-prod-tenant",
                "buck2 test //crates/oya-office-format-domain:test",
                "local-macos-arm64-rust-1.96.0",
                "2026-06-03T00:00:00Z",
                "DOCX dry-run parity guard",
                4,
                true,
            )
            .is_err()
        );
        assert!(
            BenchmarkReportRecord::new(
                dry_run_case,
                BenchmarkReportEvidenceState::Measured,
                "synthetic-non-prod-tenant",
                "buck2 test //crates/oya-office-format-domain:test",
                "local-macos-arm64-rust-1.96.0",
                "2026-06-03T00:00:00Z",
                "DOCX invalid dry-run measurement",
                4,
                false,
            )
            .is_err()
        );

        let measured_plan = BenchmarkHarnessPlan::credentialed_execution(
            BenchmarkPeer::GoogleWorkspace,
            &docx_fixture(),
        )
        .expect("credentialed plan");
        let measured_case = measured_plan.cases().first().expect("benchmark case");
        let measured = BenchmarkReportRecord::new(
            measured_case,
            BenchmarkReportEvidenceState::Measured,
            "synthetic-non-prod-tenant",
            "omx benchmark peer --fixture fixture-docx-comments --peer google-workspace",
            "isolated-staging-peer-benchmark-cell",
            "2026-06-03T00:00:00Z",
            "DOCX measured peer roundtrip",
            4,
            true,
        )
        .expect("credentialed measured parity report");

        assert!(measured.parity_claimed());
        assert!(measured.parity_claim_allowed());
    }

    #[test]
    fn docs_parity_matrix_covers_google_docs_and_onlyoffice_docs_workflows() {
        let matrix = DocsParityBenchmarkMatrix::credential_free_dry_run();

        assert!(matrix.covers_peer(BenchmarkPeer::GoogleWorkspace));
        assert!(matrix.covers_peer(BenchmarkPeer::OnlyOffice));
        assert!(matrix.covers_workflow(DocsBenchmarkWorkflow::DriveDocumentOpen));
        assert!(matrix.covers_workflow(DocsBenchmarkWorkflow::CommentPreservation));
        assert!(matrix.covers_workflow(DocsBenchmarkWorkflow::SuggestionTrackChanges));
        assert!(matrix.covers_workflow(DocsBenchmarkWorkflow::VersionHistory));
        assert!(matrix.covers_workflow(DocsBenchmarkWorkflow::DocxRoundTrip));
        assert_eq!(matrix.cases().len(), 10);
        assert!(
            matrix
                .cases()
                .iter()
                .all(|case| case.format_kind() == OfficeFormatKind::Docx)
        );
    }

    #[test]
    fn docs_parity_dry_run_never_allows_external_calls_or_parity_claims() {
        let matrix = DocsParityBenchmarkMatrix::credential_free_dry_run();

        assert_eq!(
            matrix.execution_mode(),
            BenchmarkExecutionMode::CredentialFreeDryRun
        );
        assert!(!matrix.external_calls_allowed());
        assert!(!matrix.parity_claim_allowed());
        assert!(matrix.cases().iter().all(|case| {
            !case.external_calls_allowed()
                && !case.parity_claim_allowed()
                && case.evidence_state() == DocsParityEvidenceState::CredentialFreePlanned
        }));
        assert!(
            matrix
                .credential_hooks_for(BenchmarkPeer::GoogleWorkspace)
                .contains(&BenchmarkCredentialHook::GoogleOAuthAccessToken)
        );
        assert!(
            matrix
                .credential_hooks_for(BenchmarkPeer::OnlyOffice)
                .contains(&BenchmarkCredentialHook::OnlyOfficeJwtSecret)
        );
    }

    #[test]
    fn docs_parity_workflows_require_comment_suggestion_and_docx_score_metrics() {
        assert!(
            DocsBenchmarkWorkflow::CommentPreservation
                .required_score_metrics()
                .contains(&FormatScoreMetric::Comments)
        );
        assert!(
            DocsBenchmarkWorkflow::SuggestionTrackChanges
                .required_score_metrics()
                .contains(&FormatScoreMetric::Comments)
        );
        assert!(
            DocsBenchmarkWorkflow::DocxRoundTrip
                .required_score_metrics()
                .contains(&FormatScoreMetric::TextContent)
        );
        assert!(
            DocsBenchmarkWorkflow::DocxRoundTrip
                .required_score_metrics()
                .contains(&FormatScoreMetric::Structure)
        );
        assert!(
            DocsBenchmarkWorkflow::DocxRoundTrip
                .required_score_metrics()
                .contains(&FormatScoreMetric::Styling)
        );
        assert!(
            DocsBenchmarkWorkflow::DocxRoundTrip
                .required_score_metrics()
                .contains(&FormatScoreMetric::Layout)
        );
    }

    #[test]
    fn docs_parity_case_is_docx_only_and_credentialed_measurements_can_claim() {
        assert!(
            DocsParityBenchmarkCase::new(
                BenchmarkPeer::GoogleWorkspace,
                BenchmarkExecutionMode::CredentialFreeDryRun,
                DocsBenchmarkWorkflow::DocxRoundTrip,
                OfficeFormatKind::Xlsx,
                DocsParityEvidenceState::CredentialFreePlanned,
            )
            .is_err()
        );
        assert!(
            DocsParityBenchmarkCase::new(
                BenchmarkPeer::GoogleWorkspace,
                BenchmarkExecutionMode::CredentialFreeDryRun,
                DocsBenchmarkWorkflow::DocxRoundTrip,
                OfficeFormatKind::Docx,
                DocsParityEvidenceState::Measured,
            )
            .is_err()
        );

        let measured = DocsParityBenchmarkCase::new(
            BenchmarkPeer::GoogleWorkspace,
            BenchmarkExecutionMode::CredentialedExternal,
            DocsBenchmarkWorkflow::DocxRoundTrip,
            OfficeFormatKind::Docx,
            DocsParityEvidenceState::Measured,
        )
        .expect("measured credentialed Docs parity case");

        assert!(measured.external_calls_allowed());
        assert!(measured.parity_claim_allowed());
    }

    #[test]
    fn sheets_parity_matrix_covers_google_sheets_and_onlyoffice_workflows() {
        let matrix = SheetsParityBenchmarkMatrix::credential_free_dry_run();

        assert!(matrix.covers_peer(BenchmarkPeer::GoogleWorkspace));
        assert!(matrix.covers_peer(BenchmarkPeer::OnlyOffice));
        assert!(matrix.covers_workflow(SheetsBenchmarkWorkflow::DriveSpreadsheetOpen));
        assert!(matrix.covers_workflow(SheetsBenchmarkWorkflow::RangeReadWrite));
        assert!(matrix.covers_workflow(SheetsBenchmarkWorkflow::BatchUpdateValuesAndFormulas));
        assert!(matrix.covers_workflow(SheetsBenchmarkWorkflow::FormulaRecalculation));
        assert!(matrix.covers_workflow(SheetsBenchmarkWorkflow::ProtectedRangeCollaboration));
        assert!(matrix.covers_workflow(SheetsBenchmarkWorkflow::ChangeNotificationWebhook));
        assert!(matrix.covers_workflow(SheetsBenchmarkWorkflow::EmbeddedSpreadsheet));
        assert!(matrix.covers_workflow(SheetsBenchmarkWorkflow::XlsxRoundTrip));
        assert_eq!(matrix.cases().len(), 16);
        assert!(
            matrix
                .cases()
                .iter()
                .all(|case| case.format_kind() == OfficeFormatKind::Xlsx)
        );
    }

    #[test]
    fn sheets_parity_dry_run_never_allows_external_calls_or_parity_claims() {
        let matrix = SheetsParityBenchmarkMatrix::credential_free_dry_run();

        assert_eq!(
            matrix.execution_mode(),
            BenchmarkExecutionMode::CredentialFreeDryRun
        );
        assert!(!matrix.external_calls_allowed());
        assert!(!matrix.parity_claim_allowed());
        assert!(matrix.cases().iter().all(|case| {
            !case.external_calls_allowed()
                && !case.parity_claim_allowed()
                && case.evidence_state() == SheetsParityEvidenceState::CredentialFreePlanned
        }));
    }

    #[test]
    fn sheets_parity_workflows_require_api_formula_xlsx_metrics_and_contract_anchors() {
        assert!(
            SheetsBenchmarkWorkflow::RangeReadWrite
                .required_score_metrics()
                .contains(&FormatScoreMetric::TextContent)
        );
        assert!(
            SheetsBenchmarkWorkflow::BatchUpdateValuesAndFormulas
                .required_score_metrics()
                .contains(&FormatScoreMetric::Formulas)
        );
        assert!(
            SheetsBenchmarkWorkflow::XlsxRoundTrip
                .required_score_metrics()
                .contains(&FormatScoreMetric::Styling)
        );
        assert_eq!(
            SheetsBenchmarkWorkflow::BatchUpdateValuesAndFormulas.local_contract_anchor(),
            "SheetsBatchUpdateRequest"
        );
        assert_eq!(
            SheetsBenchmarkWorkflow::ChangeNotificationWebhook.local_contract_anchor(),
            "SheetsWebhookSubscriptionRequest"
        );
        assert_eq!(
            SheetsBenchmarkWorkflow::XlsxRoundTrip.local_contract_anchor(),
            "WorkbookXlsxFormatPlan"
        );
    }

    #[test]
    fn sheets_parity_case_is_xlsx_only_and_credentialed_measurements_can_claim() {
        assert!(
            SheetsParityBenchmarkCase::new(
                BenchmarkPeer::GoogleWorkspace,
                BenchmarkExecutionMode::CredentialFreeDryRun,
                SheetsBenchmarkWorkflow::XlsxRoundTrip,
                OfficeFormatKind::Docx,
                SheetsParityEvidenceState::CredentialFreePlanned,
            )
            .is_err()
        );
        assert!(
            SheetsParityBenchmarkCase::new(
                BenchmarkPeer::GoogleWorkspace,
                BenchmarkExecutionMode::CredentialFreeDryRun,
                SheetsBenchmarkWorkflow::XlsxRoundTrip,
                OfficeFormatKind::Xlsx,
                SheetsParityEvidenceState::Measured,
            )
            .is_err()
        );

        let measured = SheetsParityBenchmarkCase::new(
            BenchmarkPeer::GoogleWorkspace,
            BenchmarkExecutionMode::CredentialedExternal,
            SheetsBenchmarkWorkflow::XlsxRoundTrip,
            OfficeFormatKind::Xlsx,
            SheetsParityEvidenceState::Measured,
        )
        .expect("measured credentialed Sheets parity case");

        assert!(measured.external_calls_allowed());
        assert!(measured.parity_claim_allowed());
    }

    #[test]
    fn sheets_parity_reference_urls_cover_google_sheets_and_onlyoffice_spreadsheet_docs() {
        let matrix = SheetsParityBenchmarkMatrix::credential_free_dry_run();

        assert!(
            matrix
                .peer_reference_urls_for(BenchmarkPeer::GoogleWorkspace)
                .iter()
                .any(|url| url.contains("workspace/sheets/api"))
        );
        assert!(
            matrix
                .peer_reference_urls_for(BenchmarkPeer::OnlyOffice)
                .iter()
                .any(|url| url.contains("spreadsheet-api"))
        );
    }

    #[test]
    fn slides_parity_matrix_covers_google_slides_and_onlyoffice_workflows() {
        let matrix = SlidesParityBenchmarkMatrix::credential_free_dry_run();

        assert!(matrix.covers_peer(BenchmarkPeer::GoogleWorkspace));
        assert!(matrix.covers_peer(BenchmarkPeer::OnlyOffice));
        assert!(matrix.covers_workflow(SlidesBenchmarkWorkflow::DrivePresentationOpen));
        assert!(matrix.covers_workflow(SlidesBenchmarkWorkflow::SlideCreateAndReorder));
        assert!(matrix.covers_workflow(SlidesBenchmarkWorkflow::ShapeTextAndLayoutEditing));
        assert!(matrix.covers_workflow(SlidesBenchmarkWorkflow::MediaInsertionAndAltText));
        assert!(matrix.covers_workflow(SlidesBenchmarkWorkflow::SpeakerNotesPreservation));
        assert!(matrix.covers_workflow(SlidesBenchmarkWorkflow::CollaborativeSlideEdit));
        assert!(matrix.covers_workflow(SlidesBenchmarkWorkflow::EmbeddedPresentation));
        assert!(matrix.covers_workflow(SlidesBenchmarkWorkflow::PptxRoundTrip));
        assert_eq!(matrix.cases().len(), 16);
        assert!(
            matrix
                .cases()
                .iter()
                .all(|case| case.format_kind() == OfficeFormatKind::Pptx)
        );
    }

    #[test]
    fn slides_parity_dry_run_never_allows_external_calls_or_parity_claims() {
        let matrix = SlidesParityBenchmarkMatrix::credential_free_dry_run();

        assert_eq!(
            matrix.execution_mode(),
            BenchmarkExecutionMode::CredentialFreeDryRun
        );
        assert!(!matrix.external_calls_allowed());
        assert!(!matrix.parity_claim_allowed());
        assert!(matrix.cases().iter().all(|case| {
            !case.external_calls_allowed()
                && !case.parity_claim_allowed()
                && case.evidence_state() == SlidesParityEvidenceState::CredentialFreePlanned
        }));
        assert!(
            matrix
                .credential_hooks_for(BenchmarkPeer::GoogleWorkspace)
                .contains(&BenchmarkCredentialHook::GoogleSlidesApiScope)
        );
        assert!(
            matrix
                .credential_hooks_for(BenchmarkPeer::OnlyOffice)
                .contains(&BenchmarkCredentialHook::OnlyOfficeJwtSecret)
        );
    }

    #[test]
    fn slides_parity_workflows_require_media_notes_pptx_metrics_and_contract_anchors() {
        assert!(
            SlidesBenchmarkWorkflow::MediaInsertionAndAltText
                .required_score_metrics()
                .contains(&FormatScoreMetric::Media)
        );
        assert!(
            SlidesBenchmarkWorkflow::MediaInsertionAndAltText
                .required_score_metrics()
                .contains(&FormatScoreMetric::AccessibilityMetadata)
        );
        assert!(
            SlidesBenchmarkWorkflow::SpeakerNotesPreservation
                .required_score_metrics()
                .contains(&FormatScoreMetric::TextContent)
        );
        assert!(
            SlidesBenchmarkWorkflow::PptxRoundTrip
                .required_score_metrics()
                .contains(&FormatScoreMetric::Layout)
        );
        assert_eq!(
            SlidesBenchmarkWorkflow::ShapeTextAndLayoutEditing.local_contract_anchor(),
            "SlideShape"
        );
        assert_eq!(
            SlidesBenchmarkWorkflow::MediaInsertionAndAltText.local_contract_anchor(),
            "SlideMediaAsset"
        );
        assert_eq!(
            SlidesBenchmarkWorkflow::PptxRoundTrip.local_contract_anchor(),
            "DeckPptxFormatPlan"
        );
    }

    #[test]
    fn slides_parity_case_is_pptx_only_and_credentialed_measurements_can_claim() {
        assert!(
            SlidesParityBenchmarkCase::new(
                BenchmarkPeer::GoogleWorkspace,
                BenchmarkExecutionMode::CredentialFreeDryRun,
                SlidesBenchmarkWorkflow::PptxRoundTrip,
                OfficeFormatKind::Docx,
                SlidesParityEvidenceState::CredentialFreePlanned,
            )
            .is_err()
        );
        assert!(
            SlidesParityBenchmarkCase::new(
                BenchmarkPeer::GoogleWorkspace,
                BenchmarkExecutionMode::CredentialFreeDryRun,
                SlidesBenchmarkWorkflow::PptxRoundTrip,
                OfficeFormatKind::Pptx,
                SlidesParityEvidenceState::Measured,
            )
            .is_err()
        );

        let measured = SlidesParityBenchmarkCase::new(
            BenchmarkPeer::GoogleWorkspace,
            BenchmarkExecutionMode::CredentialedExternal,
            SlidesBenchmarkWorkflow::PptxRoundTrip,
            OfficeFormatKind::Pptx,
            SlidesParityEvidenceState::Measured,
        )
        .expect("measured credentialed Slides parity case");

        assert!(measured.external_calls_allowed());
        assert!(measured.parity_claim_allowed());
    }

    #[test]
    fn slides_parity_reference_urls_cover_google_slides_and_onlyoffice_presentation_docs() {
        let matrix = SlidesParityBenchmarkMatrix::credential_free_dry_run();

        assert!(
            matrix
                .peer_reference_urls_for(BenchmarkPeer::GoogleWorkspace)
                .iter()
                .any(|url| url.contains("workspace/slides/api"))
        );
        assert!(
            matrix
                .peer_reference_urls_for(BenchmarkPeer::OnlyOffice)
                .iter()
                .any(|url| url.contains("presentation-api"))
        );
    }

    fn docx_fixture() -> FormatFixtureSpec {
        FormatFixtureSpec::new(
            FormatFixtureId::new("fixture-docx-comments").expect("valid fixture id"),
            FixtureCorpusId::new("corpus-ooxml-parity").expect("valid corpus id"),
            FormatFixtureBinding::new(
                drive_binding("tenant-alpha", "doc-1", DriveObjectKind::Document),
                OfficeFormatKind::Docx,
            )
            .expect("fixture binding"),
            FixtureComplexity::Representative,
            vec![FixtureFeature::ParagraphText, FixtureFeature::Comments],
        )
        .expect("fixture spec")
    }

    fn drive_binding(
        tenant_id: &str,
        object_id: &str,
        kind: DriveObjectKind,
    ) -> DriveObjectBinding {
        DriveObjectBinding::new(
            TenantId::new(tenant_id).expect("valid tenant id"),
            ObjectId::new(object_id).expect("valid object id"),
            kind,
            DataClass::Confidential,
        )
    }
}
