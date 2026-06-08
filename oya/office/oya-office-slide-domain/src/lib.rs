#![forbid(unsafe_code)]
//! Deck, slide, shape, media, layout, and PPTX-bound presentation domain model.
//!
//! This early slice binds every deck to Oya Drive so Slides shares Drive ACL,
//! KMS, lifecycle, and audit semantics.

use oya_office_drive_domain::{DriveObjectBinding, DriveObjectKind};
use oya_office_format_domain::{FormatFixtureBinding, FormatJobDirection, OfficeFormatKind};
use oya_office_kernel::{DataClass, ObjectId, PrincipalId, TenantId};

/// Stable crate identifier used by workspace and Buck2 scaffold verification.
pub const CRATE_NAME: &str = "oya-office-slide-domain";

/// Product vertical slice owned by this crate.
pub const VERTICAL_SLICE: &str = "slides";

/// Source-shaped architectural layer represented by this crate.
pub const ARCHITECTURE_LAYER: &str = "domain";

/// Deck identifier inside the Slides slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeckId(ObjectId);

impl DeckId {
    /// Creates a deck id from Drive object id.
    #[must_use]
    pub const fn from_drive_object_id(object_id: ObjectId) -> Self {
        Self(object_id)
    }

    /// Returns object id.
    #[must_use]
    pub const fn as_object_id(&self) -> &ObjectId {
        &self.0
    }
}

/// Drive-bound deck aggregate shell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeckDriveBinding {
    deck_id: DeckId,
    binding: DriveObjectBinding,
}

impl DeckDriveBinding {
    /// Creates a deck binding and verifies it points to a Drive presentation object.
    pub fn new(binding: DriveObjectBinding) -> Result<Self, DeckBindingError> {
        if binding.kind() != DriveObjectKind::Presentation {
            return Err(DeckBindingError::new(
                "slides binding requires a presentation object",
            ));
        }
        Ok(Self {
            deck_id: DeckId::from_drive_object_id(binding.object_id().clone()),
            binding,
        })
    }

    /// Returns deck id.
    #[must_use]
    pub const fn deck_id(&self) -> &DeckId {
        &self.deck_id
    }

    /// Returns Drive binding.
    #[must_use]
    pub const fn drive_binding(&self) -> &DriveObjectBinding {
        &self.binding
    }

    /// Returns tenant id.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        self.binding.tenant_id()
    }
}

/// Baseline layout vocabulary for a Drive-bound deck slide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SlideLayoutKind {
    /// Empty slide where every shape is explicit.
    Blank,
    /// Title-only slide.
    TitleOnly,
    /// Title plus content body.
    TitleAndContent,
    /// Two-column content layout.
    TwoColumn,
    /// Media-first slide with caption or supporting content.
    MediaWithCaption,
}

/// Geometry for a shape on a slide canvas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SlideGeometry {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl SlideGeometry {
    /// Creates a geometry rectangle and rejects zero-area shapes.
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Result<Self, DeckBindingError> {
        if width == 0 || height == 0 {
            return Err(DeckBindingError::new(
                "slide geometry width and height must be positive",
            ));
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }

    /// Returns x offset from the slide origin.
    #[must_use]
    pub const fn x(&self) -> u32 {
        self.x
    }

    /// Returns y offset from the slide origin.
    #[must_use]
    pub const fn y(&self) -> u32 {
        self.y
    }

    /// Returns shape width.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns shape height.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }
}

/// Shape vocabulary for the first Slides domain baseline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SlideShapeKind {
    /// Editable text box.
    TextBox,
    /// Non-text rectangle or panel.
    Rectangle,
    /// Image placeholder bound to a media asset.
    ImagePlaceholder,
    /// Chart placeholder for future chart import/export lanes.
    ChartPlaceholder,
    /// Table placeholder for future table import/export lanes.
    TablePlaceholder,
}

/// One editable shape on a slide.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlideShape {
    shape_id: String,
    kind: SlideShapeKind,
    geometry: SlideGeometry,
    text: Option<String>,
    media_asset_id: Option<String>,
}

impl SlideShape {
    /// Creates a slide shape with validation that matches its kind.
    pub fn new(
        shape_id: impl Into<String>,
        kind: SlideShapeKind,
        geometry: SlideGeometry,
        text: Option<String>,
        media_asset_id: Option<String>,
    ) -> Result<Self, DeckBindingError> {
        let text = validate_optional_text("slide shape text", text)?;
        let media_asset_id = validate_optional_text("slide shape media asset id", media_asset_id)?;

        match kind {
            SlideShapeKind::TextBox if text.is_none() => {
                return Err(DeckBindingError::new(
                    "text box shapes require non-empty text",
                ));
            }
            SlideShapeKind::ImagePlaceholder if media_asset_id.is_none() => {
                return Err(DeckBindingError::new(
                    "image placeholder shapes require a media asset id",
                ));
            }
            _ => {}
        }

        Ok(Self {
            shape_id: validate_required_text("slide shape id", shape_id)?,
            kind,
            geometry,
            text,
            media_asset_id,
        })
    }

    /// Returns shape id.
    #[must_use]
    pub fn shape_id(&self) -> &str {
        self.shape_id.as_str()
    }

    /// Returns shape kind.
    #[must_use]
    pub const fn kind(&self) -> SlideShapeKind {
        self.kind
    }

    /// Returns shape geometry.
    #[must_use]
    pub const fn geometry(&self) -> SlideGeometry {
        self.geometry
    }

    /// Returns optional text content.
    #[must_use]
    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    /// Returns optional media asset id.
    #[must_use]
    pub fn media_asset_id(&self) -> Option<&str> {
        self.media_asset_id.as_deref()
    }
}

/// Media types supported by the first Slides baseline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SlideMediaKind {
    /// Raster or vector image.
    Image,
    /// Audio clip.
    Audio,
    /// Video clip.
    Video,
}

/// A Drive-bound media asset referenced by slide shapes and PPTX contracts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlideMediaAsset {
    media_id: String,
    kind: SlideMediaKind,
    content_type: String,
    alt_text: String,
    data_class: DataClass,
}

impl SlideMediaAsset {
    /// Creates a media asset with accessibility and data-class metadata.
    pub fn new(
        media_id: impl Into<String>,
        kind: SlideMediaKind,
        content_type: impl Into<String>,
        alt_text: impl Into<String>,
        data_class: DataClass,
    ) -> Result<Self, DeckBindingError> {
        let content_type = validate_required_text("slide media content type", content_type)?;
        if !content_type.contains('/') {
            return Err(DeckBindingError::new(
                "slide media content type must be a media type",
            ));
        }
        Ok(Self {
            media_id: validate_required_text("slide media id", media_id)?,
            kind,
            content_type,
            alt_text: validate_required_text("slide media alt text", alt_text)?,
            data_class,
        })
    }

    /// Returns media id.
    #[must_use]
    pub fn media_id(&self) -> &str {
        self.media_id.as_str()
    }

    /// Returns media kind.
    #[must_use]
    pub const fn kind(&self) -> SlideMediaKind {
        self.kind
    }

    /// Returns media content type.
    #[must_use]
    pub fn content_type(&self) -> &str {
        self.content_type.as_str()
    }

    /// Returns required accessibility alternate text.
    #[must_use]
    pub fn alt_text(&self) -> &str {
        self.alt_text.as_str()
    }

    /// Returns data classification inherited by media processing lanes.
    #[must_use]
    pub const fn data_class(&self) -> DataClass {
        self.data_class
    }
}

/// One slide in a Drive-bound presentation deck.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slide {
    slide_id: String,
    layout: SlideLayoutKind,
    shapes: Vec<SlideShape>,
    media: Vec<SlideMediaAsset>,
    speaker_notes: Option<String>,
}

impl Slide {
    /// Creates a slide with unique shapes and internally resolved media references.
    pub fn new(
        slide_id: impl Into<String>,
        layout: SlideLayoutKind,
        shapes: Vec<SlideShape>,
        media: Vec<SlideMediaAsset>,
        speaker_notes: Option<String>,
    ) -> Result<Self, DeckBindingError> {
        if shapes.is_empty() {
            return Err(DeckBindingError::new("slide requires at least one shape"));
        }
        ensure_unique_shape_ids(&shapes)?;
        ensure_unique_media_ids(&media)?;
        ensure_shape_media_refs(&shapes, &media)?;

        Ok(Self {
            slide_id: validate_required_text("slide id", slide_id)?,
            layout,
            shapes,
            media,
            speaker_notes: validate_optional_text("slide speaker notes", speaker_notes)?,
        })
    }

    /// Returns slide id.
    #[must_use]
    pub fn slide_id(&self) -> &str {
        self.slide_id.as_str()
    }

    /// Returns layout kind.
    #[must_use]
    pub const fn layout(&self) -> SlideLayoutKind {
        self.layout
    }

    /// Returns shapes.
    #[must_use]
    pub fn shapes(&self) -> &[SlideShape] {
        self.shapes.as_slice()
    }

    /// Returns media assets.
    #[must_use]
    pub fn media(&self) -> &[SlideMediaAsset] {
        self.media.as_slice()
    }

    /// Returns optional speaker notes.
    #[must_use]
    pub fn speaker_notes(&self) -> Option<&str> {
        self.speaker_notes.as_deref()
    }

    /// Returns shape count.
    #[must_use]
    pub fn shape_count(&self) -> usize {
        self.shapes.len()
    }

    /// Returns media count.
    #[must_use]
    pub fn media_count(&self) -> usize {
        self.media.len()
    }

    fn has_shape(&self, shape_id: &str) -> bool {
        self.shapes.iter().any(|shape| shape.shape_id() == shape_id)
    }

    fn has_media(&self, media_id: &str) -> bool {
        self.media.iter().any(|media| media.media_id() == media_id)
    }
}

/// Drive-bound deck aggregate used by Slides editor, collab, and PPTX lanes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeckModel {
    binding: DeckDriveBinding,
    title: String,
    slides: Vec<Slide>,
    version_sequence: u64,
}

impl DeckModel {
    /// Creates a validated deck aggregate.
    pub fn new(
        binding: DeckDriveBinding,
        title: impl Into<String>,
        slides: Vec<Slide>,
        version_sequence: u64,
    ) -> Result<Self, DeckBindingError> {
        if slides.is_empty() {
            return Err(DeckBindingError::new(
                "deck model requires at least one slide",
            ));
        }
        if version_sequence == 0 {
            return Err(DeckBindingError::new(
                "deck model version sequence must be at least 1",
            ));
        }
        ensure_unique_slide_ids(&slides)?;
        Ok(Self {
            binding,
            title: validate_required_text("deck title", title)?,
            slides,
            version_sequence,
        })
    }

    /// Returns deck id inherited from the Drive object id.
    #[must_use]
    pub const fn deck_id(&self) -> &DeckId {
        self.binding.deck_id()
    }

    /// Returns Drive-bound deck binding.
    #[must_use]
    pub const fn drive_binding(&self) -> &DeckDriveBinding {
        &self.binding
    }

    /// Returns tenant id.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        self.binding.tenant_id()
    }

    /// Returns deck title.
    #[must_use]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    /// Returns slides.
    #[must_use]
    pub fn slides(&self) -> &[Slide] {
        self.slides.as_slice()
    }

    /// Returns optimistic concurrency deck version sequence.
    #[must_use]
    pub const fn version_sequence(&self) -> u64 {
        self.version_sequence
    }

    fn slide(&self, slide_id: &str) -> Option<&Slide> {
        self.slides
            .iter()
            .find(|slide| slide.slide_id() == slide_id)
    }
}

/// Target of a collaborative slide edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SlideEditTarget {
    /// Edit targets the whole slide.
    Slide,
    /// Edit targets a shape id.
    Shape(String),
    /// Edit targets a media asset id.
    Media(String),
}

impl SlideEditTarget {
    /// Creates a whole-slide target.
    #[must_use]
    pub const fn slide() -> Self {
        Self::Slide
    }

    /// Creates a shape target.
    pub fn shape(shape_id: impl Into<String>) -> Result<Self, DeckBindingError> {
        validate_required_text("slide edit shape id", shape_id).map(Self::Shape)
    }

    /// Creates a media target.
    pub fn media(media_id: impl Into<String>) -> Result<Self, DeckBindingError> {
        validate_required_text("slide edit media id", media_id).map(Self::Media)
    }
}

/// Collaborative slide edit operation kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CollaborativeSlideEditKind {
    /// Add a new shape to a slide.
    AddShape,
    /// Update existing shape text.
    UpdateShapeText,
    /// Move or resize an existing shape.
    MoveShape,
    /// Add a new media asset to a slide.
    AddMedia,
    /// Replace an existing media asset while preserving shape references.
    ReplaceMedia,
    /// Update slide speaker notes.
    UpdateSpeakerNotes,
    /// Reorder a slide within the deck.
    ReorderSlide,
}

/// One optimistic-concurrency collaborative slide edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollaborativeSlideEdit {
    deck_id: DeckId,
    actor_id: PrincipalId,
    slide_id: String,
    target: SlideEditTarget,
    kind: CollaborativeSlideEditKind,
    expected_version_sequence: u64,
}

impl CollaborativeSlideEdit {
    /// Creates a collaborative slide edit with an expected deck version.
    pub fn new(
        deck_id: DeckId,
        actor_id: PrincipalId,
        slide_id: impl Into<String>,
        target: SlideEditTarget,
        kind: CollaborativeSlideEditKind,
        expected_version_sequence: u64,
    ) -> Result<Self, DeckBindingError> {
        if expected_version_sequence == 0 {
            return Err(DeckBindingError::new(
                "collaborative slide edit expected version must be at least 1",
            ));
        }
        Ok(Self {
            deck_id,
            actor_id,
            slide_id: validate_required_text("collaborative slide edit slide id", slide_id)?,
            target,
            kind,
            expected_version_sequence,
        })
    }

    /// Returns deck id.
    #[must_use]
    pub const fn deck_id(&self) -> &DeckId {
        &self.deck_id
    }

    /// Returns actor id.
    #[must_use]
    pub const fn actor_id(&self) -> &PrincipalId {
        &self.actor_id
    }

    /// Returns slide id.
    #[must_use]
    pub fn slide_id(&self) -> &str {
        self.slide_id.as_str()
    }

    /// Returns edit target.
    #[must_use]
    pub const fn target(&self) -> &SlideEditTarget {
        &self.target
    }

    /// Returns edit kind.
    #[must_use]
    pub const fn kind(&self) -> CollaborativeSlideEditKind {
        self.kind
    }

    /// Returns expected deck version sequence.
    #[must_use]
    pub const fn expected_version_sequence(&self) -> u64 {
        self.expected_version_sequence
    }

    /// Validates deck/slide/target binding and optimistic concurrency sequence.
    pub fn validate_for_deck(&self, model: &DeckModel) -> Result<(), DeckBindingError> {
        if self.deck_id() != model.deck_id() {
            return Err(DeckBindingError::new(
                "collaborative slide edit deck id does not match model",
            ));
        }
        let slide = model.slide(self.slide_id()).ok_or_else(|| {
            DeckBindingError::new("collaborative slide edit slide does not exist in model")
        })?;
        if self.expected_version_sequence() != model.version_sequence() {
            return Err(DeckBindingError::new(
                "collaborative slide edit expected version does not match model",
            ));
        }
        validate_slide_edit_target(self.kind(), self.target(), slide)
    }
}

/// Drive-bound PPTX import/export contract for the Slides slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeckPptxFormatPlan {
    deck_binding: DeckDriveBinding,
    format_binding: FormatFixtureBinding,
    direction: FormatJobDirection,
    preserve_slide_layouts: bool,
    preserve_media: bool,
    preserve_speaker_notes: bool,
    requires_drive_binding: bool,
}

impl DeckPptxFormatPlan {
    /// Creates a PPTX format plan from an already validated deck binding.
    pub fn new(
        deck_binding: DeckDriveBinding,
        direction: FormatJobDirection,
    ) -> Result<Self, DeckBindingError> {
        let format_binding =
            FormatFixtureBinding::new(deck_binding.drive_binding().clone(), OfficeFormatKind::Pptx)
                .map_err(|error| {
                    DeckBindingError::new(format!(
                        "deck PPTX format binding failed: {}",
                        error.message()
                    ))
                })?;

        Ok(Self {
            deck_binding,
            format_binding,
            direction,
            preserve_slide_layouts: true,
            preserve_media: true,
            preserve_speaker_notes: true,
            requires_drive_binding: true,
        })
    }

    /// Creates a PPTX format plan from a raw Drive object binding.
    pub fn from_drive_binding(
        binding: DriveObjectBinding,
        direction: FormatJobDirection,
    ) -> Result<Self, DeckBindingError> {
        Self::new(DeckDriveBinding::new(binding)?, direction)
    }

    /// Returns deck binding.
    #[must_use]
    pub const fn deck_binding(&self) -> &DeckDriveBinding {
        &self.deck_binding
    }

    /// Returns format binding.
    #[must_use]
    pub const fn format_binding(&self) -> &FormatFixtureBinding {
        &self.format_binding
    }

    /// Returns Office format kind.
    #[must_use]
    pub const fn format_kind(&self) -> OfficeFormatKind {
        self.format_binding.format_kind()
    }

    /// Returns format job direction.
    #[must_use]
    pub const fn direction(&self) -> FormatJobDirection {
        self.direction
    }

    /// Returns true because PPTX format work must remain Drive-bound.
    #[must_use]
    pub const fn requires_drive_binding(&self) -> bool {
        self.requires_drive_binding
    }

    /// Returns true when slide layouts are part of the preservation contract.
    #[must_use]
    pub const fn preserve_slide_layouts(&self) -> bool {
        self.preserve_slide_layouts
    }

    /// Returns true when media assets are part of the preservation contract.
    #[must_use]
    pub const fn preserve_media(&self) -> bool {
        self.preserve_media
    }

    /// Returns true when speaker notes are part of the preservation contract.
    #[must_use]
    pub const fn preserve_speaker_notes(&self) -> bool {
        self.preserve_speaker_notes
    }
}

/// Deck binding validation error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeckBindingError {
    message: String,
}

impl DeckBindingError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

impl core::fmt::Display for DeckBindingError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for DeckBindingError {}

fn validate_required_text(
    field: &'static str,
    value: impl Into<String>,
) -> Result<String, DeckBindingError> {
    let value = value.into();
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(DeckBindingError::new(format!("{field} must not be empty")));
    }
    Ok(trimmed.to_owned())
}

fn validate_optional_text(
    field: &'static str,
    value: Option<String>,
) -> Result<Option<String>, DeckBindingError> {
    value
        .map(|text| validate_required_text(field, text))
        .transpose()
}

fn ensure_unique_slide_ids(slides: &[Slide]) -> Result<(), DeckBindingError> {
    let mut seen = std::collections::BTreeSet::new();
    for slide in slides {
        if !seen.insert(slide.slide_id()) {
            return Err(DeckBindingError::new("deck slide ids must be unique"));
        }
    }
    Ok(())
}

fn ensure_unique_shape_ids(shapes: &[SlideShape]) -> Result<(), DeckBindingError> {
    let mut seen = std::collections::BTreeSet::new();
    for shape in shapes {
        if !seen.insert(shape.shape_id()) {
            return Err(DeckBindingError::new("slide shape ids must be unique"));
        }
    }
    Ok(())
}

fn ensure_unique_media_ids(media_assets: &[SlideMediaAsset]) -> Result<(), DeckBindingError> {
    let mut seen = std::collections::BTreeSet::new();
    for media in media_assets {
        if !seen.insert(media.media_id()) {
            return Err(DeckBindingError::new("slide media ids must be unique"));
        }
    }
    Ok(())
}

fn ensure_shape_media_refs(
    shapes: &[SlideShape],
    media_assets: &[SlideMediaAsset],
) -> Result<(), DeckBindingError> {
    for shape in shapes {
        if let Some(media_asset_id) = shape.media_asset_id() {
            let media = media_assets
                .iter()
                .find(|media| media.media_id() == media_asset_id)
                .ok_or_else(|| {
                    DeckBindingError::new(
                        "slide shape media asset id does not exist in slide media",
                    )
                })?;
            if shape.kind() == SlideShapeKind::ImagePlaceholder
                && media.kind() != SlideMediaKind::Image
            {
                return Err(DeckBindingError::new(
                    "image placeholder shapes require image media assets",
                ));
            }
        }
    }
    Ok(())
}

fn validate_slide_edit_target(
    kind: CollaborativeSlideEditKind,
    target: &SlideEditTarget,
    slide: &Slide,
) -> Result<(), DeckBindingError> {
    match (kind, target) {
        (
            CollaborativeSlideEditKind::AddShape
            | CollaborativeSlideEditKind::AddMedia
            | CollaborativeSlideEditKind::UpdateSpeakerNotes
            | CollaborativeSlideEditKind::ReorderSlide,
            SlideEditTarget::Slide,
        ) => Ok(()),
        (
            CollaborativeSlideEditKind::UpdateShapeText | CollaborativeSlideEditKind::MoveShape,
            SlideEditTarget::Shape(shape_id),
        ) => {
            if slide.has_shape(shape_id) {
                Ok(())
            } else {
                Err(DeckBindingError::new(
                    "collaborative slide edit shape does not exist in slide",
                ))
            }
        }
        (CollaborativeSlideEditKind::ReplaceMedia, SlideEditTarget::Media(media_id)) => {
            if slide.has_media(media_id) {
                Ok(())
            } else {
                Err(DeckBindingError::new(
                    "collaborative slide edit media does not exist in slide",
                ))
            }
        }
        _ => Err(DeckBindingError::new(
            "collaborative slide edit target is incompatible with edit kind",
        )),
    }
}

#[cfg(test)]
mod tests {
    use oya_office_drive_domain::{DriveObjectBinding, DriveObjectKind};
    use oya_office_format_domain::{FormatJobDirection, OfficeFormatKind};
    use oya_office_kernel::{DataClass, ObjectId, PrincipalId, TenantId};

    use super::{
        ARCHITECTURE_LAYER, CRATE_NAME, CollaborativeSlideEdit, CollaborativeSlideEditKind,
        DeckDriveBinding, DeckModel, DeckPptxFormatPlan, Slide, SlideEditTarget, SlideGeometry,
        SlideLayoutKind, SlideMediaAsset, SlideMediaKind, SlideShape, SlideShapeKind,
        VERTICAL_SLICE,
    };

    #[test]
    fn scaffold_identity_is_declared() {
        assert!(!ARCHITECTURE_LAYER.is_empty());
        assert!(!CRATE_NAME.is_empty());
        assert!(!VERTICAL_SLICE.is_empty());
    }

    #[test]
    fn slides_bind_only_to_drive_presentation_objects() {
        let binding = DriveObjectBinding::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("deck-1").expect("valid object id"),
            DriveObjectKind::Presentation,
            DataClass::Confidential,
        );
        let deck = DeckDriveBinding::new(binding).expect("deck binding");
        assert_eq!(deck.deck_id().as_object_id().as_str(), "deck-1");

        let wrong_kind = DriveObjectBinding::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("sheet-1").expect("valid object id"),
            DriveObjectKind::Spreadsheet,
            DataClass::Internal,
        );
        assert!(DeckDriveBinding::new(wrong_kind).is_err());
    }

    fn actor() -> PrincipalId {
        PrincipalId::new("user-alpha").expect("valid principal")
    }

    fn deck_binding() -> DeckDriveBinding {
        DeckDriveBinding::new(DriveObjectBinding::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("deck-1").expect("valid object id"),
            DriveObjectKind::Presentation,
            DataClass::Confidential,
        ))
        .expect("deck binding")
    }

    fn deck_model() -> DeckModel {
        let title_shape = SlideShape::new(
            "shape-title".to_owned(),
            SlideShapeKind::TextBox,
            SlideGeometry::new(0, 0, 960, 120).expect("geometry"),
            Some("Quarterly results".to_owned()),
            None,
        )
        .expect("title shape");
        let media = SlideMediaAsset::new(
            "media-logo".to_owned(),
            SlideMediaKind::Image,
            "image/png".to_owned(),
            "Company logo".to_owned(),
            DataClass::Internal,
        )
        .expect("media asset");
        let image_shape = SlideShape::new(
            "shape-logo".to_owned(),
            SlideShapeKind::ImagePlaceholder,
            SlideGeometry::new(800, 20, 120, 80).expect("geometry"),
            None,
            Some("media-logo".to_owned()),
        )
        .expect("image shape");
        let slide = Slide::new(
            "slide-1".to_owned(),
            SlideLayoutKind::TitleAndContent,
            vec![title_shape, image_shape],
            vec![media],
            Some("Mention revised FY guide.".to_owned()),
        )
        .expect("slide");

        DeckModel::new(deck_binding(), "Board update".to_owned(), vec![slide], 7)
            .expect("deck model")
    }

    #[test]
    fn deck_model_requires_title_slides_unique_layout_and_drive_presentation_binding() {
        let model = deck_model();

        assert_eq!(model.title(), "Board update");
        assert_eq!(model.slides().len(), 1);
        assert_eq!(model.version_sequence(), 7);
        assert_eq!(model.slides()[0].layout(), SlideLayoutKind::TitleAndContent);
        assert_eq!(
            model.drive_binding().drive_binding().kind(),
            DriveObjectKind::Presentation
        );

        assert!(
            DeckModel::new(deck_binding(), " ".to_owned(), model.slides().to_vec(), 7).is_err()
        );
        assert!(DeckModel::new(deck_binding(), "Deck".to_owned(), vec![], 1).is_err());
        assert!(
            DeckModel::new(
                deck_binding(),
                "Deck".to_owned(),
                vec![model.slides()[0].clone(), model.slides()[0].clone()],
                1
            )
            .is_err()
        );
    }

    #[test]
    fn slide_shapes_and_media_validate_layout_and_anchor_contracts() {
        let geometry = SlideGeometry::new(10, 20, 300, 160).expect("geometry");
        let text_shape = SlideShape::new(
            "shape-copy".to_owned(),
            SlideShapeKind::TextBox,
            geometry,
            Some("Launch plan".to_owned()),
            None,
        )
        .expect("text shape");
        let media = SlideMediaAsset::new(
            "media-hero".to_owned(),
            SlideMediaKind::Image,
            "image/jpeg".to_owned(),
            "Hero image".to_owned(),
            DataClass::Confidential,
        )
        .expect("media");
        let image_shape = SlideShape::new(
            "shape-hero".to_owned(),
            SlideShapeKind::ImagePlaceholder,
            SlideGeometry::new(400, 20, 300, 160).expect("geometry"),
            None,
            Some("media-hero".to_owned()),
        )
        .expect("image shape");

        let slide = Slide::new(
            "slide-media".to_owned(),
            SlideLayoutKind::Blank,
            vec![text_shape, image_shape],
            vec![media],
            None,
        )
        .expect("slide");

        assert_eq!(slide.shape_count(), 2);
        assert_eq!(slide.media_count(), 1);
        assert!(SlideGeometry::new(0, 0, 0, 100).is_err());
        assert!(
            SlideShape::new(
                "bad-text".to_owned(),
                SlideShapeKind::TextBox,
                SlideGeometry::new(0, 0, 10, 10).expect("geometry"),
                Some(" ".to_owned()),
                None,
            )
            .is_err()
        );
        assert!(
            SlideShape::new(
                "bad-image".to_owned(),
                SlideShapeKind::ImagePlaceholder,
                SlideGeometry::new(0, 0, 10, 10).expect("geometry"),
                None,
                None,
            )
            .is_err()
        );
        assert!(
            Slide::new(
                "slide-missing-media".to_owned(),
                SlideLayoutKind::Blank,
                vec![
                    SlideShape::new(
                        "shape-unknown-media".to_owned(),
                        SlideShapeKind::ImagePlaceholder,
                        SlideGeometry::new(0, 0, 10, 10).expect("geometry"),
                        None,
                        Some("missing-media".to_owned()),
                    )
                    .expect("shape")
                ],
                vec![],
                None,
            )
            .is_err()
        );
        let audio_media = SlideMediaAsset::new(
            "media-audio".to_owned(),
            SlideMediaKind::Audio,
            "audio/mpeg".to_owned(),
            "Narration".to_owned(),
            DataClass::Internal,
        )
        .expect("audio media");
        let image_shape_with_audio = SlideShape::new(
            "shape-audio".to_owned(),
            SlideShapeKind::ImagePlaceholder,
            SlideGeometry::new(0, 0, 10, 10).expect("geometry"),
            None,
            Some("media-audio".to_owned()),
        )
        .expect("image placeholder");
        assert!(
            Slide::new(
                "slide-wrong-media-kind".to_owned(),
                SlideLayoutKind::Blank,
                vec![image_shape_with_audio],
                vec![audio_media],
                None,
            )
            .is_err()
        );
    }

    #[test]
    fn collaborative_slide_edit_requires_expected_version_and_known_targets() {
        let model = deck_model();
        let shape_edit = CollaborativeSlideEdit::new(
            model.deck_id().clone(),
            actor(),
            "slide-1".to_owned(),
            SlideEditTarget::shape("shape-title".to_owned()).expect("target"),
            CollaborativeSlideEditKind::UpdateShapeText,
            model.version_sequence(),
        )
        .expect("edit");
        let stale_edit = CollaborativeSlideEdit::new(
            model.deck_id().clone(),
            actor(),
            "slide-1".to_owned(),
            SlideEditTarget::slide(),
            CollaborativeSlideEditKind::UpdateSpeakerNotes,
            6,
        )
        .expect("edit");
        let unknown_shape_edit = CollaborativeSlideEdit::new(
            model.deck_id().clone(),
            actor(),
            "slide-1".to_owned(),
            SlideEditTarget::shape("shape-missing".to_owned()).expect("target"),
            CollaborativeSlideEditKind::MoveShape,
            model.version_sequence(),
        )
        .expect("edit");
        let media_edit = CollaborativeSlideEdit::new(
            model.deck_id().clone(),
            actor(),
            "slide-1".to_owned(),
            SlideEditTarget::media("media-logo".to_owned()).expect("target"),
            CollaborativeSlideEditKind::ReplaceMedia,
            model.version_sequence(),
        )
        .expect("edit");
        let wrong_target_edit = CollaborativeSlideEdit::new(
            model.deck_id().clone(),
            actor(),
            "slide-1".to_owned(),
            SlideEditTarget::slide(),
            CollaborativeSlideEditKind::UpdateShapeText,
            model.version_sequence(),
        )
        .expect("edit");

        assert!(shape_edit.validate_for_deck(&model).is_ok());
        assert!(media_edit.validate_for_deck(&model).is_ok());
        assert!(stale_edit.validate_for_deck(&model).is_err());
        assert!(unknown_shape_edit.validate_for_deck(&model).is_err());
        assert!(wrong_target_edit.validate_for_deck(&model).is_err());
        assert!(
            CollaborativeSlideEdit::new(
                model.deck_id().clone(),
                actor(),
                "slide-1".to_owned(),
                SlideEditTarget::shape("shape-title".to_owned()).expect("target"),
                CollaborativeSlideEditKind::MoveShape,
                0,
            )
            .is_err()
        );
    }

    #[test]
    fn pptx_import_export_plan_is_drive_bound_and_pptx_only() {
        let plan = DeckPptxFormatPlan::new(deck_binding(), FormatJobDirection::RoundTrip)
            .expect("pptx plan");

        assert_eq!(plan.format_kind(), OfficeFormatKind::Pptx);
        assert_eq!(plan.direction(), FormatJobDirection::RoundTrip);
        assert!(plan.requires_drive_binding());
        assert!(plan.preserve_slide_layouts());
        assert!(plan.preserve_media());
        assert!(plan.preserve_speaker_notes());

        let wrong_kind = DriveObjectBinding::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("sheet-1").expect("valid object id"),
            DriveObjectKind::Spreadsheet,
            DataClass::Internal,
        );
        assert!(
            DeckPptxFormatPlan::from_drive_binding(wrong_kind, FormatJobDirection::Export).is_err()
        );
    }
}
