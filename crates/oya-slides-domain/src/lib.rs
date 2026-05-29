//! Workspace slides kernel.
//!
//! Typed kernel records for the W-Workspace-Stable Slides surface named by
//! `docs/products/workspace/PRD.md` and ADR-0029. The kernel owns deck metadata,
//! CRDT binding to the shared collab runtime, template references, slide graph
//! validation, and the export-format guardrail for PPTX/PDF/HTML.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

use oya_collab_runtime_domain::{CollabRuntime, CollabSurface};
use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use oya_document_format_domain::{DocumentKind, ExportFormat};

const SLIDE_DECK_SCHEMA_VERSION: u32 = 1;
const SLIDE_GRAPH_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlideError {
    InvalidDeckId,
    InvalidTenantId,
    InvalidRegion,
    InvalidCellId,
    InvalidDrivePath,
    InvalidTitle,
    InvalidCollabRuntime,
    InvalidCollabSurface,
    InvalidTemplateId,
    InvalidTemplateVersion,
    EmptySlideGraph,
    InvalidSlideId,
    InvalidSlideOrdinal,
    DuplicateSlideId,
    DuplicateSlideOrdinal,
    NonContiguousSlideOrdinal,
    InvalidBlockId,
    DuplicateBlockId,
    InvalidBlockSourceRef,
    MissingTransitionEndpoint,
    SelfTransition,
    DuplicateTransition,
    InvalidTimeOrder,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SlideLayout {
    Blank,
    Title,
    TitleContent,
    Section,
    TwoColumn,
    Media,
    Custom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SlideBlockKind {
    Text,
    Image,
    Chart,
    Table,
    EmbeddedDoc,
    Shape,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SlideTransitionKind {
    Cut,
    Fade,
    Push,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlideDeckCreate {
    pub id: String,                             // data_class: INTERNAL_ONLY
    pub title: String,                          // data_class: PII_QUASI_IDENTIFIER
    pub drive_path: String,                     // data_class: PII_QUASI_IDENTIFIER
    pub tenant_id: String,                      // data_class: INTERNAL_ONLY
    pub region: String,                         // data_class: INTERNAL_ONLY
    pub cell_id: String,                        // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>,   // data_class: INTERNAL_ONLY
    pub collab_runtime: CollabRuntime,          // data_class: PII_IDENTIFYING
    pub slide_graph: SlideGraph,                // data_class: PII_IDENTIFYING
    pub template_ref: Option<SlideTemplateRef>, // data_class: INTERNAL_ONLY
    pub indexed_at_epoch_seconds: Option<u64>,  // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,          // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlideDeck {
    pub id: Classified<String>,                    // data_class: INTERNAL_ONLY
    pub title: Classified<String>,                 // data_class: PII_QUASI_IDENTIFIER
    pub drive_path: Classified<String>,            // data_class: PII_QUASI_IDENTIFIER
    pub tenant_id: Classified<String>,             // data_class: INTERNAL_ONLY
    pub region: Classified<String>,                // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,               // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>,  // data_class: INTERNAL_ONLY
    pub collab_runtime: Classified<CollabRuntime>, // data_class: PII_IDENTIFYING
    pub slide_graph: Classified<SlideGraph>,       // data_class: PII_IDENTIFYING
    pub template_ref: Classified<Option<SlideTemplateRef>>, // data_class: INTERNAL_ONLY
    pub indexed_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlideTemplateRef {
    pub template_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub version: Classified<u32>,        // data_class: INTERNAL_ONLY
    pub owner_tenant_id: Classified<Option<String>>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlideCreate {
    pub slide_id: String,              // data_class: INTERNAL_ONLY
    pub ordinal: u32,                  // data_class: INTERNAL_ONLY
    pub title: Option<String>,         // data_class: PII_QUASI_IDENTIFIER
    pub layout: SlideLayout,           // data_class: INTERNAL_ONLY
    pub blocks: Vec<SlideBlockRef>,    // data_class: PII_IDENTIFYING
    pub speaker_notes: Option<String>, // data_class: PII_IDENTIFYING
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Slide {
    pub slide_id: Classified<String>,      // data_class: INTERNAL_ONLY
    pub ordinal: Classified<u32>,          // data_class: INTERNAL_ONLY
    pub title: Classified<Option<String>>, // data_class: PII_QUASI_IDENTIFIER
    pub layout: Classified<SlideLayout>,   // data_class: INTERNAL_ONLY
    pub blocks: Classified<Vec<SlideBlockRef>>, // data_class: PII_IDENTIFYING
    pub speaker_notes: Classified<Option<String>>, // data_class: PII_IDENTIFYING
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SlideBlockRef {
    pub block_id: Classified<String>,     // data_class: INTERNAL_ONLY
    pub kind: Classified<SlideBlockKind>, // data_class: INTERNAL_ONLY
    pub source_ref: Classified<Option<String>>, // data_class: PII_IDENTIFYING
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SlideTransition {
    pub from_slide_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub to_slide_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub kind: Classified<SlideTransitionKind>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlideGraph {
    pub slides: Vec<Slide>,                // data_class: PII_IDENTIFYING
    pub transitions: Vec<SlideTransition>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,   // data_class: INTERNAL_ONLY
}

pub trait SlideDeckReader {
    fn read_slide(
        &self,
        tenant_id: &str,
        deck_id: &str,
        slide_id: &str,
    ) -> Result<Option<Slide>, SlideError>;
}

impl SlideDeck {
    pub fn new(input: SlideDeckCreate) -> Result<Self, SlideError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_slide_data_class());
        validate_non_empty(&input.id, SlideError::InvalidDeckId)?;
        validate_non_empty(&input.tenant_id, SlideError::InvalidTenantId)?;
        validate_non_empty(&input.region, SlideError::InvalidRegion)?;
        validate_non_empty(&input.cell_id, SlideError::InvalidCellId)?;
        validate_non_empty(&input.title, SlideError::InvalidTitle)?;
        validate_drive_path(&input.drive_path)?;
        validate_time_order(
            input.created_at_epoch_seconds,
            input.updated_at_epoch_seconds,
        )?;
        validate_collab_runtime_binding(
            &input.collab_runtime,
            &input.id,
            &input.tenant_id,
            &input.region,
            &input.cell_id,
        )?;
        if let Some(template_ref) = &input.template_ref {
            template_ref.validate()?;
        }
        input.slide_graph.validate()?;

        Ok(Self {
            id: internal(input.id),
            title: Classified::new(input.title, title_data_class()),
            drive_path: Classified::new(input.drive_path, title_data_class()),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            cell_id: internal(input.cell_id),
            data_class: internal(data_class),
            collab_runtime: Classified::new(input.collab_runtime, slide_content_data_class()),
            slide_graph: Classified::new(input.slide_graph, slide_content_data_class()),
            template_ref: internal(input.template_ref),
            indexed_at_epoch_seconds: internal(input.indexed_at_epoch_seconds),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: internal(SLIDE_DECK_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl SlideTemplateRef {
    pub fn new(
        template_id: String,
        version: u32,
        owner_tenant_id: Option<String>,
    ) -> Result<Self, SlideError> {
        validate_non_empty(&template_id, SlideError::InvalidTemplateId)?;
        if version == 0 {
            return Err(SlideError::InvalidTemplateVersion);
        }
        if let Some(owner_tenant_id) = owner_tenant_id.as_deref() {
            validate_non_empty(owner_tenant_id, SlideError::InvalidTenantId)?;
        }
        Ok(Self {
            template_id: internal(template_id),
            version: internal(version),
            owner_tenant_id: internal(owner_tenant_id),
        })
    }

    fn validate(&self) -> Result<(), SlideError> {
        validate_non_empty(&self.template_id.value, SlideError::InvalidTemplateId)?;
        if self.version.value == 0 {
            return Err(SlideError::InvalidTemplateVersion);
        }
        if let Some(owner_tenant_id) = self.owner_tenant_id.value.as_deref() {
            validate_non_empty(owner_tenant_id, SlideError::InvalidTenantId)?;
        }
        Ok(())
    }
}

impl Slide {
    pub fn new(input: SlideCreate) -> Result<Self, SlideError> {
        validate_non_empty(&input.slide_id, SlideError::InvalidSlideId)?;
        if input.ordinal == 0 {
            return Err(SlideError::InvalidSlideOrdinal);
        }
        validate_optional_text(input.title.as_deref(), SlideError::InvalidTitle)?;
        validate_optional_text(
            input.speaker_notes.as_deref(),
            SlideError::InvalidBlockSourceRef,
        )?;
        validate_blocks(&input.blocks)?;

        Ok(Self {
            slide_id: internal(input.slide_id),
            ordinal: internal(input.ordinal),
            title: Classified::new(input.title, title_data_class()),
            layout: internal(input.layout),
            blocks: Classified::new(input.blocks, slide_content_data_class()),
            speaker_notes: Classified::new(input.speaker_notes, slide_content_data_class()),
        })
    }
}

impl SlideBlockRef {
    pub fn new(
        block_id: String,
        kind: SlideBlockKind,
        source_ref: Option<String>,
        data_class: Option<PrivacyDataClass>,
    ) -> Result<Self, SlideError> {
        validate_non_empty(&block_id, SlideError::InvalidBlockId)?;
        validate_optional_text(source_ref.as_deref(), SlideError::InvalidBlockSourceRef)?;
        Ok(Self {
            block_id: internal(block_id),
            kind: internal(kind),
            source_ref: Classified::new(source_ref, slide_content_data_class()),
            data_class: internal(data_class.unwrap_or(default_workspace_slide_data_class())),
        })
    }
}

impl SlideTransition {
    pub fn new(
        from_slide_id: String,
        to_slide_id: String,
        kind: SlideTransitionKind,
    ) -> Result<Self, SlideError> {
        validate_non_empty(&from_slide_id, SlideError::InvalidSlideId)?;
        validate_non_empty(&to_slide_id, SlideError::InvalidSlideId)?;
        if from_slide_id == to_slide_id {
            return Err(SlideError::SelfTransition);
        }
        Ok(Self {
            from_slide_id: internal(from_slide_id),
            to_slide_id: internal(to_slide_id),
            kind: internal(kind),
        })
    }
}

impl SlideGraph {
    pub fn new(slides: Vec<Slide>, transitions: Vec<SlideTransition>) -> Result<Self, SlideError> {
        let graph = Self {
            slides,
            transitions,
            schema_version: internal(SLIDE_GRAPH_SCHEMA_VERSION),
        };
        graph.validate()?;
        Ok(graph)
    }

    pub fn validate(&self) -> Result<(), SlideError> {
        if self.slides.is_empty() {
            return Err(SlideError::EmptySlideGraph);
        }
        let mut ids = BTreeSet::new();
        let mut ordinals = BTreeSet::new();
        for slide in &self.slides {
            validate_non_empty(&slide.slide_id.value, SlideError::InvalidSlideId)?;
            if slide.ordinal.value == 0 {
                return Err(SlideError::InvalidSlideOrdinal);
            }
            if !ids.insert(slide.slide_id.value.clone()) {
                return Err(SlideError::DuplicateSlideId);
            }
            if !ordinals.insert(slide.ordinal.value) {
                return Err(SlideError::DuplicateSlideOrdinal);
            }
        }
        validate_contiguous_ordinals(&ordinals)?;
        validate_transitions(&ids, &self.transitions)
    }
}

pub fn supports_slide_export(format: ExportFormat) -> bool {
    format.is_supported_for(DocumentKind::Presentation)
}

pub fn default_workspace_slide_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn slide_content_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn title_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_quasi_identifier()
}

pub fn workspace_slide_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, SlideError> {
    PrivacyDataClass::new(data_class).map_err(|_| SlideError::InvalidDataClass)
}

fn validate_collab_runtime_binding(
    runtime: &CollabRuntime,
    deck_id: &str,
    tenant_id: &str,
    region: &str,
    cell_id: &str,
) -> Result<(), SlideError> {
    if runtime.surface.value != CollabSurface::Slides {
        return Err(SlideError::InvalidCollabSurface);
    }
    if runtime.document_id.value != deck_id
        || runtime.tenant_id.value != tenant_id
        || runtime.region.value != region
        || runtime.cell_id.value != cell_id
    {
        return Err(SlideError::InvalidCollabRuntime);
    }
    Ok(())
}

fn validate_blocks(blocks: &[SlideBlockRef]) -> Result<(), SlideError> {
    let mut ids = BTreeSet::new();
    for block in blocks {
        validate_non_empty(&block.block_id.value, SlideError::InvalidBlockId)?;
        if !ids.insert(block.block_id.value.clone()) {
            return Err(SlideError::DuplicateBlockId);
        }
        validate_optional_text(
            block.source_ref.value.as_deref(),
            SlideError::InvalidBlockSourceRef,
        )?;
    }
    Ok(())
}

fn validate_contiguous_ordinals(ordinals: &BTreeSet<u32>) -> Result<(), SlideError> {
    for (index, ordinal) in ordinals.iter().enumerate() {
        let expected = u32::try_from(index + 1).map_err(|_| SlideError::InvalidSlideOrdinal)?;
        if *ordinal != expected {
            return Err(SlideError::NonContiguousSlideOrdinal);
        }
    }
    Ok(())
}

fn validate_transitions(
    slide_ids: &BTreeSet<String>,
    transitions: &[SlideTransition],
) -> Result<(), SlideError> {
    let mut seen = BTreeSet::new();
    for transition in transitions {
        let from = &transition.from_slide_id.value;
        let to = &transition.to_slide_id.value;
        if from == to {
            return Err(SlideError::SelfTransition);
        }
        if !slide_ids.contains(from) || !slide_ids.contains(to) {
            return Err(SlideError::MissingTransitionEndpoint);
        }
        if !seen.insert((from.clone(), to.clone())) {
            return Err(SlideError::DuplicateTransition);
        }
    }
    Ok(())
}

fn validate_optional_text(value: Option<&str>, error: SlideError) -> Result<(), SlideError> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.trim() != value || value.is_empty() || value.chars().any(char::is_control) {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_drive_path(path: &str) -> Result<(), SlideError> {
    if path.trim() != path
        || !path.starts_with('/')
        || path == "/"
        || path.ends_with('/')
        || path.contains("//")
        || path.chars().any(char::is_control)
    {
        return Err(SlideError::InvalidDrivePath);
    }
    if path
        .split('/')
        .skip(1)
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(SlideError::InvalidDrivePath);
    }
    Ok(())
}

fn validate_time_order(created_at: u64, updated_at: u64) -> Result<(), SlideError> {
    if updated_at < created_at {
        Err(SlideError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_non_empty(value: &str, error: SlideError) -> Result<(), SlideError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_collab_runtime_domain::{CollabRuntimeCreate, CollabSnapshotRef, CollabStateVectorRef};
    use oya_data_boundary_kernel::{DataClassification, OperationalDataClass};

    fn runtime(surface: CollabSurface) -> CollabRuntime {
        CollabRuntime::new(CollabRuntimeCreate {
            document_id: "deck-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            surface,
            data_class: None,
            snapshot: CollabSnapshotRef::new(
                "snap-1".into(),
                "tenant-1/slides/deck-1/snap-1".into(),
                "sha256:snapshot".into(),
                "sv:1".into(),
                1,
                1,
                128,
            )
            .unwrap(),
            state_vector: CollabStateVectorRef::new("sv:1".into(), 1, 1, 32).unwrap(),
            active_awareness: Vec::new(),
            created_at_epoch_millis: 1_700_000_000_000,
            updated_at_epoch_millis: 1_700_000_010_000,
        })
        .unwrap()
    }

    fn block(block_id: &str) -> SlideBlockRef {
        SlideBlockRef::new(
            block_id.into(),
            SlideBlockKind::Text,
            Some(format!("block:{block_id}")),
            None,
        )
        .unwrap()
    }

    fn slide(slide_id: &str, ordinal: u32) -> Slide {
        Slide::new(SlideCreate {
            slide_id: slide_id.into(),
            ordinal,
            title: Some(format!("Slide {ordinal}")),
            layout: SlideLayout::TitleContent,
            blocks: vec![block(&format!("block-{ordinal}"))],
            speaker_notes: Some("speaker note".into()),
        })
        .unwrap()
    }

    fn graph() -> SlideGraph {
        SlideGraph::new(
            vec![slide("slide-1", 1), slide("slide-2", 2)],
            vec![
                SlideTransition::new(
                    "slide-1".into(),
                    "slide-2".into(),
                    SlideTransitionKind::Fade,
                )
                .unwrap(),
            ],
        )
        .unwrap()
    }

    fn deck_input() -> SlideDeckCreate {
        SlideDeckCreate {
            id: "deck-1".into(),
            title: "Board update".into(),
            drive_path: "/team/board-update.oyaslide".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            data_class: None,
            collab_runtime: runtime(CollabSurface::Slides),
            slide_graph: graph(),
            template_ref: Some(SlideTemplateRef::new("template-1".into(), 1, None).unwrap()),
            indexed_at_epoch_seconds: None,
            created_at_epoch_seconds: 1_700_000_000,
            updated_at_epoch_seconds: 1_700_000_010,
        }
    }

    #[test]
    fn deck_defaults_to_identifying_and_supports_required_exports() {
        let deck = SlideDeck::new(deck_input()).unwrap();

        assert_eq!(
            deck.privacy_data_class().data_class(),
            DataClass::PiiIdentifying
        );
        assert_eq!(
            deck.title.data_class,
            DataClassification::Privacy(title_data_class())
        );
        assert_eq!(
            deck.slide_graph.data_class,
            DataClassification::Privacy(slide_content_data_class())
        );
        assert!(supports_slide_export(ExportFormat::Pptx));
        assert!(supports_slide_export(ExportFormat::Pdf));
        assert!(supports_slide_export(ExportFormat::Html));
        assert!(!supports_slide_export(ExportFormat::Markdown));
        assert_eq!(deck.schema_version.value, 1);
    }

    #[test]
    fn slide_graph_rejects_duplicate_or_noncontiguous_ordinals() {
        let duplicate = SlideGraph::new(vec![slide("slide-1", 1), slide("slide-2", 1)], vec![]);
        assert_eq!(duplicate, Err(SlideError::DuplicateSlideOrdinal));

        let noncontiguous = SlideGraph::new(vec![slide("slide-1", 1), slide("slide-3", 3)], vec![]);
        assert_eq!(noncontiguous, Err(SlideError::NonContiguousSlideOrdinal));
    }

    #[test]
    fn slide_graph_rejects_dangling_self_and_duplicate_transitions() {
        let dangling = SlideGraph::new(
            vec![slide("slide-1", 1)],
            vec![
                SlideTransition::new("slide-1".into(), "slide-2".into(), SlideTransitionKind::Cut)
                    .unwrap(),
            ],
        );
        assert_eq!(dangling, Err(SlideError::MissingTransitionEndpoint));

        assert_eq!(
            SlideTransition::new("slide-1".into(), "slide-1".into(), SlideTransitionKind::Cut),
            Err(SlideError::SelfTransition)
        );

        let duplicate = SlideGraph::new(
            vec![slide("slide-1", 1), slide("slide-2", 2)],
            vec![
                SlideTransition::new("slide-1".into(), "slide-2".into(), SlideTransitionKind::Cut)
                    .unwrap(),
                SlideTransition::new(
                    "slide-1".into(),
                    "slide-2".into(),
                    SlideTransitionKind::Fade,
                )
                .unwrap(),
            ],
        );
        assert_eq!(duplicate, Err(SlideError::DuplicateTransition));
    }

    #[test]
    fn template_and_collab_surface_are_validated() {
        assert_eq!(
            SlideTemplateRef::new("template-1".into(), 0, None),
            Err(SlideError::InvalidTemplateVersion)
        );

        let mut invalid = deck_input();
        invalid.collab_runtime = runtime(CollabSurface::Docs);
        assert_eq!(
            SlideDeck::new(invalid),
            Err(SlideError::InvalidCollabSurface)
        );
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_slide_data_class_from_legacy(DataClass::Audit),
            Err(SlideError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
    }
}

// ---------------------------------------------------------------------------
// M03-P06-IP — workspace.slides STAGING surface markers (SPEC §4 rows).
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlidesSurfaceStaging {
    pub deck_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub yrs_state_vector: Classified<Vec<u8>>, // data_class: INTERNAL_ONLY
}

impl SlidesSurfaceStaging {
    pub fn new(deck_id: String, tenant_id: String, yrs_state_vector: Vec<u8>) -> Self {
        Self {
            deck_id: Classified::new(deck_id, DataClass::InternalOnly),
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            yrs_state_vector: Classified::new(yrs_state_vector, DataClass::InternalOnly),
        }
    }
}

#[cfg(test)]
mod m03_p06_tests {
    use super::*;

    fn sample() -> SlidesSurfaceStaging {
        SlidesSurfaceStaging::new("slides-1".into(), "slides-1".into(), vec![])
    }

    #[test]
    fn surface_staging_constructor_sets_internal_only() {
        let s = sample();
        assert_eq!(s.deck_id.data_class, DataClass::InternalOnly.into());
    }

    #[test]
    fn surface_staging_round_trip_equality() {
        assert_eq!(sample(), sample());
    }
}
