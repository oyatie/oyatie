#![forbid(unsafe_code)]
//! Document model and Drive-bound Docs collaboration surface.
//!
//! This early slice binds every document to Oya Drive so Docs cannot drift into
//! an isolated editor store.

use oya_office_drive_domain::{DriveObjectBinding, DriveObjectKind};
use oya_office_format_domain::{FormatFixtureBinding, FormatJobDirection, OfficeFormatKind};
use oya_office_kernel::{ObjectId, PrincipalId, TenantId};

/// Stable crate identifier used by workspace and Buck2 scaffold verification.
pub const CRATE_NAME: &str = "oya-office-doc-domain";

/// Product vertical slice owned by this crate.
pub const VERTICAL_SLICE: &str = "docs";

/// Source-shaped architectural layer represented by this crate.
pub const ARCHITECTURE_LAYER: &str = "domain";

/// Document identifier inside the Docs slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentId(ObjectId);

impl DocumentId {
    /// Creates a document id from a Drive object id.
    #[must_use]
    pub const fn from_drive_object_id(object_id: ObjectId) -> Self {
        Self(object_id)
    }

    /// Returns the underlying object id.
    #[must_use]
    pub const fn as_object_id(&self) -> &ObjectId {
        &self.0
    }
}

/// Drive-bound document aggregate shell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentDriveBinding {
    document_id: DocumentId,
    binding: DriveObjectBinding,
}

impl DocumentDriveBinding {
    /// Creates a document binding and verifies it points to a Drive document object.
    pub fn new(binding: DriveObjectBinding) -> Result<Self, DocumentBindingError> {
        if binding.kind() != DriveObjectKind::Document {
            return Err(DocumentBindingError::new(
                "docs binding requires a document object",
            ));
        }
        Ok(Self {
            document_id: DocumentId::from_drive_object_id(binding.object_id().clone()),
            binding,
        })
    }

    /// Returns document id.
    #[must_use]
    pub const fn document_id(&self) -> &DocumentId {
        &self.document_id
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

/// One editable block in the Drive-bound document model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentBlock {
    block_id: String,
    text: String,
}

impl DocumentBlock {
    /// Creates a paragraph block.
    pub fn paragraph(
        block_id: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<Self, DocumentBindingError> {
        Ok(Self {
            block_id: validate_required_text("document block id", block_id)?,
            text: validate_required_text("document paragraph text", text)?,
        })
    }

    /// Returns the block identifier used by comments, suggestions, and editor operations.
    #[must_use]
    pub fn block_id(&self) -> &str {
        self.block_id.as_str()
    }

    /// Returns the block text payload for the current baseline model.
    #[must_use]
    pub fn text(&self) -> &str {
        self.text.as_str()
    }
}

/// Drive-bound document aggregate used by Docs editor, version, and format lanes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentModel {
    binding: DocumentDriveBinding,
    title: String,
    blocks: Vec<DocumentBlock>,
    version_sequence: u64,
}

impl DocumentModel {
    /// Creates a validated document aggregate.
    pub fn new(
        binding: DocumentDriveBinding,
        title: impl Into<String>,
        blocks: Vec<DocumentBlock>,
        version_sequence: u64,
    ) -> Result<Self, DocumentBindingError> {
        if blocks.is_empty() {
            return Err(DocumentBindingError::new(
                "document model requires at least one block",
            ));
        }
        if version_sequence == 0 {
            return Err(DocumentBindingError::new(
                "document model version sequence must be at least 1",
            ));
        }
        ensure_unique_block_ids(&blocks)?;
        Ok(Self {
            binding,
            title: validate_required_text("document title", title)?,
            blocks,
            version_sequence,
        })
    }

    /// Returns the document id inherited from the Drive object id.
    #[must_use]
    pub const fn document_id(&self) -> &DocumentId {
        self.binding.document_id()
    }

    /// Returns the Drive-bound document binding.
    #[must_use]
    pub const fn drive_binding(&self) -> &DocumentDriveBinding {
        &self.binding
    }

    /// Returns the owning tenant.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        self.binding.tenant_id()
    }

    /// Returns the document title.
    #[must_use]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    /// Returns editable document blocks.
    #[must_use]
    pub fn blocks(&self) -> &[DocumentBlock] {
        self.blocks.as_slice()
    }

    /// Returns the optimistic concurrency version sequence.
    #[must_use]
    pub const fn version_sequence(&self) -> u64 {
        self.version_sequence
    }

    fn has_block_anchor(&self, anchor_id: &str) -> bool {
        self.blocks
            .iter()
            .any(|block| block.block_id() == anchor_id)
    }
}

/// Comment lifecycle state for baseline Docs collaboration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DocumentCommentStatus {
    /// Comment is visible and unresolved.
    Open,
    /// Comment was resolved but remains in the version/audit surface.
    Resolved,
}

/// A Drive-bound comment anchored to a document block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentComment {
    comment_id: String,
    document_id: DocumentId,
    actor_id: PrincipalId,
    anchor_id: String,
    body: String,
    status: DocumentCommentStatus,
}

impl DocumentComment {
    /// Creates an open comment anchored to a document block id.
    pub fn new(
        comment_id: impl Into<String>,
        document_id: DocumentId,
        actor_id: PrincipalId,
        anchor_id: impl Into<String>,
        body: impl Into<String>,
    ) -> Result<Self, DocumentBindingError> {
        Ok(Self {
            comment_id: validate_required_text("document comment id", comment_id)?,
            document_id,
            actor_id,
            anchor_id: validate_required_text("document comment anchor id", anchor_id)?,
            body: validate_required_text("document comment body", body)?,
            status: DocumentCommentStatus::Open,
        })
    }

    /// Returns the comment id.
    #[must_use]
    pub fn comment_id(&self) -> &str {
        self.comment_id.as_str()
    }

    /// Returns the document id.
    #[must_use]
    pub const fn document_id(&self) -> &DocumentId {
        &self.document_id
    }

    /// Returns the actor id.
    #[must_use]
    pub const fn actor_id(&self) -> &PrincipalId {
        &self.actor_id
    }

    /// Returns the block anchor id.
    #[must_use]
    pub fn anchor_id(&self) -> &str {
        self.anchor_id.as_str()
    }

    /// Returns the comment body.
    #[must_use]
    pub fn body(&self) -> &str {
        self.body.as_str()
    }

    /// Returns the comment status.
    #[must_use]
    pub const fn status(&self) -> DocumentCommentStatus {
        self.status
    }

    /// Validates that the comment is bound to the same document and a known block anchor.
    pub fn validate_for_model(&self, model: &DocumentModel) -> Result<(), DocumentBindingError> {
        validate_document_anchor(
            &self.document_id,
            self.anchor_id(),
            model,
            "document comment",
        )
    }
}

/// Suggestion operation kind for baseline Docs track-change semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DocumentSuggestionKind {
    /// Insert text at the anchor.
    InsertText,
    /// Replace text at the anchor.
    ReplaceText,
    /// Delete text at the anchor.
    DeleteText,
}

/// Suggestion lifecycle state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DocumentSuggestionStatus {
    /// Suggestion awaits accept/reject.
    Pending,
    /// Suggestion was accepted.
    Accepted,
    /// Suggestion was rejected.
    Rejected,
}

/// A Drive-bound editing suggestion anchored to a document block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentSuggestion {
    suggestion_id: String,
    document_id: DocumentId,
    actor_id: PrincipalId,
    anchor_id: String,
    kind: DocumentSuggestionKind,
    body: String,
    status: DocumentSuggestionStatus,
}

impl DocumentSuggestion {
    /// Creates a pending editing suggestion.
    pub fn new(
        suggestion_id: impl Into<String>,
        document_id: DocumentId,
        actor_id: PrincipalId,
        anchor_id: impl Into<String>,
        kind: DocumentSuggestionKind,
        body: impl Into<String>,
    ) -> Result<Self, DocumentBindingError> {
        Ok(Self {
            suggestion_id: validate_required_text("document suggestion id", suggestion_id)?,
            document_id,
            actor_id,
            anchor_id: validate_required_text("document suggestion anchor id", anchor_id)?,
            kind,
            body: validate_required_text("document suggestion body", body)?,
            status: DocumentSuggestionStatus::Pending,
        })
    }

    /// Returns the suggestion id.
    #[must_use]
    pub fn suggestion_id(&self) -> &str {
        self.suggestion_id.as_str()
    }

    /// Returns the document id.
    #[must_use]
    pub const fn document_id(&self) -> &DocumentId {
        &self.document_id
    }

    /// Returns the actor id.
    #[must_use]
    pub const fn actor_id(&self) -> &PrincipalId {
        &self.actor_id
    }

    /// Returns the block anchor id.
    #[must_use]
    pub fn anchor_id(&self) -> &str {
        self.anchor_id.as_str()
    }

    /// Returns the suggestion operation kind.
    #[must_use]
    pub const fn kind(&self) -> DocumentSuggestionKind {
        self.kind
    }

    /// Returns the suggestion body.
    #[must_use]
    pub fn body(&self) -> &str {
        self.body.as_str()
    }

    /// Returns the suggestion status.
    #[must_use]
    pub const fn status(&self) -> DocumentSuggestionStatus {
        self.status
    }

    /// Validates that the suggestion is bound to the same document and a known block anchor.
    pub fn validate_for_model(&self, model: &DocumentModel) -> Result<(), DocumentBindingError> {
        validate_document_anchor(
            &self.document_id,
            self.anchor_id(),
            model,
            "document suggestion",
        )
    }
}

/// Version pointer for a Drive-bound Docs revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentVersionRef {
    document_id: DocumentId,
    version_id: String,
    sequence: u64,
    actor_id: PrincipalId,
    created_at_epoch_millis: u64,
}

impl DocumentVersionRef {
    /// Creates a document version pointer.
    pub fn new(
        document_id: DocumentId,
        version_id: impl Into<String>,
        sequence: u64,
        actor_id: PrincipalId,
        created_at_epoch_millis: u64,
    ) -> Result<Self, DocumentBindingError> {
        if sequence == 0 {
            return Err(DocumentBindingError::new(
                "document version sequence must be at least 1",
            ));
        }
        if created_at_epoch_millis == 0 {
            return Err(DocumentBindingError::new(
                "document version timestamp must be present",
            ));
        }
        Ok(Self {
            document_id,
            version_id: validate_required_text("document version id", version_id)?,
            sequence,
            actor_id,
            created_at_epoch_millis,
        })
    }

    /// Returns the document id.
    #[must_use]
    pub const fn document_id(&self) -> &DocumentId {
        &self.document_id
    }

    /// Returns the version id.
    #[must_use]
    pub fn version_id(&self) -> &str {
        self.version_id.as_str()
    }

    /// Returns the monotonic version sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns the actor id that created the version.
    #[must_use]
    pub const fn actor_id(&self) -> &PrincipalId {
        &self.actor_id
    }

    /// Returns the creation time in Unix epoch milliseconds.
    #[must_use]
    pub const fn created_at_epoch_millis(&self) -> u64 {
        self.created_at_epoch_millis
    }
}

/// Sequence-aware version history for one document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentVersionHistory {
    document_id: DocumentId,
    versions: Vec<DocumentVersionRef>,
}

impl DocumentVersionHistory {
    /// Creates a version history and fail-closes on mismatched or non-monotonic versions.
    pub fn new(
        document_id: DocumentId,
        versions: Vec<DocumentVersionRef>,
    ) -> Result<Self, DocumentBindingError> {
        if versions.is_empty() {
            return Err(DocumentBindingError::new(
                "document version history requires at least one version",
            ));
        }
        let mut previous_sequence = 0;
        for version in &versions {
            if version.document_id() != &document_id {
                return Err(DocumentBindingError::new(
                    "document version history contains a different document id",
                ));
            }
            if version.sequence() <= previous_sequence {
                return Err(DocumentBindingError::new(
                    "document version history sequences must be strictly increasing",
                ));
            }
            previous_sequence = version.sequence();
        }
        Ok(Self {
            document_id,
            versions,
        })
    }

    /// Returns the document id.
    #[must_use]
    pub const fn document_id(&self) -> &DocumentId {
        &self.document_id
    }

    /// Returns the latest version.
    #[must_use]
    pub fn latest(&self) -> &DocumentVersionRef {
        self.versions
            .last()
            .expect("validated document version history is non-empty")
    }

    /// Returns the number of versions.
    #[must_use]
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }

    /// Returns all version pointers in stored sequence order.
    #[must_use]
    pub fn versions(&self) -> &[DocumentVersionRef] {
        self.versions.as_slice()
    }
}

/// Editor interaction operation kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DocumentEditorInteractionKind {
    /// Insert text at the anchor.
    InsertText,
    /// Replace text at the anchor.
    ReplaceText,
    /// Delete text at the anchor.
    DeleteText,
    /// Attach a comment to the anchor.
    AddComment,
    /// Attach a suggestion to the anchor.
    AddSuggestion,
}

/// Optimistic-concurrency editor operation bound to one document block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentEditorInteraction {
    document_id: DocumentId,
    actor_id: PrincipalId,
    kind: DocumentEditorInteractionKind,
    anchor_id: String,
    expected_version_sequence: u64,
}

impl DocumentEditorInteraction {
    /// Creates an editor interaction with an expected document version sequence.
    pub fn new(
        document_id: DocumentId,
        actor_id: PrincipalId,
        kind: DocumentEditorInteractionKind,
        anchor_id: impl Into<String>,
        expected_version_sequence: u64,
    ) -> Result<Self, DocumentBindingError> {
        if expected_version_sequence == 0 {
            return Err(DocumentBindingError::new(
                "document editor interaction expected version must be at least 1",
            ));
        }
        Ok(Self {
            document_id,
            actor_id,
            kind,
            anchor_id: validate_required_text("document editor interaction anchor id", anchor_id)?,
            expected_version_sequence,
        })
    }

    /// Returns the document id.
    #[must_use]
    pub const fn document_id(&self) -> &DocumentId {
        &self.document_id
    }

    /// Returns the actor id.
    #[must_use]
    pub const fn actor_id(&self) -> &PrincipalId {
        &self.actor_id
    }

    /// Returns the interaction kind.
    #[must_use]
    pub const fn kind(&self) -> DocumentEditorInteractionKind {
        self.kind
    }

    /// Returns the block anchor id.
    #[must_use]
    pub fn anchor_id(&self) -> &str {
        self.anchor_id.as_str()
    }

    /// Returns the expected document version sequence.
    #[must_use]
    pub const fn expected_version_sequence(&self) -> u64 {
        self.expected_version_sequence
    }

    /// Validates document id, block anchor, and optimistic concurrency sequence.
    pub fn validate_for_model(&self, model: &DocumentModel) -> Result<(), DocumentBindingError> {
        validate_document_anchor(
            &self.document_id,
            self.anchor_id(),
            model,
            "document editor interaction",
        )?;
        if self.expected_version_sequence != model.version_sequence() {
            return Err(DocumentBindingError::new(
                "document editor interaction expected version does not match model",
            ));
        }
        Ok(())
    }
}

/// Drive-bound DOCX import/export contract for the Docs slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentDocxFormatPlan {
    document_binding: DocumentDriveBinding,
    format_binding: FormatFixtureBinding,
    direction: FormatJobDirection,
    preserve_comments: bool,
    preserve_suggestions: bool,
    requires_drive_binding: bool,
}

impl DocumentDocxFormatPlan {
    /// Creates a DOCX format plan from an already validated document binding.
    pub fn new(
        document_binding: DocumentDriveBinding,
        direction: FormatJobDirection,
    ) -> Result<Self, DocumentBindingError> {
        let format_binding = FormatFixtureBinding::new(
            document_binding.drive_binding().clone(),
            OfficeFormatKind::Docx,
        )
        .map_err(|error| {
            DocumentBindingError::new(format!(
                "document DOCX format binding failed: {}",
                error.message()
            ))
        })?;

        Ok(Self {
            document_binding,
            format_binding,
            direction,
            preserve_comments: true,
            preserve_suggestions: true,
            requires_drive_binding: true,
        })
    }

    /// Creates a DOCX format plan from a raw Drive object binding.
    pub fn from_drive_binding(
        binding: DriveObjectBinding,
        direction: FormatJobDirection,
    ) -> Result<Self, DocumentBindingError> {
        Self::new(DocumentDriveBinding::new(binding)?, direction)
    }

    /// Returns the document binding.
    #[must_use]
    pub const fn document_binding(&self) -> &DocumentDriveBinding {
        &self.document_binding
    }

    /// Returns the format binding used by format workers.
    #[must_use]
    pub const fn format_binding(&self) -> &FormatFixtureBinding {
        &self.format_binding
    }

    /// Returns the Office format kind.
    #[must_use]
    pub const fn format_kind(&self) -> OfficeFormatKind {
        self.format_binding.format_kind()
    }

    /// Returns the import/export/round-trip direction.
    #[must_use]
    pub const fn direction(&self) -> FormatJobDirection {
        self.direction
    }

    /// Returns true because DOCX format work must remain Drive-bound.
    #[must_use]
    pub const fn requires_drive_binding(&self) -> bool {
        self.requires_drive_binding
    }

    /// Returns true when DOCX comments are part of the preservation contract.
    #[must_use]
    pub const fn preserve_comments(&self) -> bool {
        self.preserve_comments
    }

    /// Returns true when DOCX track-change/suggestion data is part of the preservation contract.
    #[must_use]
    pub const fn preserve_suggestions(&self) -> bool {
        self.preserve_suggestions
    }
}

/// Document binding validation error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentBindingError {
    message: String,
}

impl DocumentBindingError {
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

impl core::fmt::Display for DocumentBindingError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for DocumentBindingError {}

fn validate_required_text(
    field: &'static str,
    value: impl Into<String>,
) -> Result<String, DocumentBindingError> {
    let value = value.into();
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(DocumentBindingError::new(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

fn ensure_unique_block_ids(blocks: &[DocumentBlock]) -> Result<(), DocumentBindingError> {
    let mut seen = std::collections::BTreeSet::new();
    for block in blocks {
        if !seen.insert(block.block_id()) {
            return Err(DocumentBindingError::new(
                "document model block ids must be unique",
            ));
        }
    }
    Ok(())
}

fn validate_document_anchor(
    document_id: &DocumentId,
    anchor_id: &str,
    model: &DocumentModel,
    label: &'static str,
) -> Result<(), DocumentBindingError> {
    if document_id != model.document_id() {
        return Err(DocumentBindingError::new(format!(
            "{label} document id does not match model"
        )));
    }
    if !model.has_block_anchor(anchor_id) {
        return Err(DocumentBindingError::new(format!(
            "{label} anchor does not exist in model"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use oya_office_drive_domain::{DriveObjectBinding, DriveObjectKind};
    use oya_office_format_domain::{FormatJobDirection, OfficeFormatKind};
    use oya_office_kernel::{DataClass, ObjectId, PrincipalId, TenantId};

    use super::{
        ARCHITECTURE_LAYER, CRATE_NAME, DocumentBlock, DocumentComment, DocumentDocxFormatPlan,
        DocumentDriveBinding, DocumentEditorInteraction, DocumentEditorInteractionKind,
        DocumentModel, DocumentSuggestion, DocumentSuggestionKind, DocumentVersionHistory,
        DocumentVersionRef, VERTICAL_SLICE,
    };

    #[test]
    fn scaffold_identity_is_declared() {
        assert!(!ARCHITECTURE_LAYER.is_empty());
        assert!(!CRATE_NAME.is_empty());
        assert!(!VERTICAL_SLICE.is_empty());
    }

    #[test]
    fn docs_bind_only_to_drive_document_objects() {
        let binding = DriveObjectBinding::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("doc-1").expect("valid object id"),
            DriveObjectKind::Document,
            DataClass::Confidential,
        );
        let doc = DocumentDriveBinding::new(binding).expect("document binding");
        assert_eq!(doc.document_id().as_object_id().as_str(), "doc-1");

        let wrong_kind = DriveObjectBinding::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("sheet-1").expect("valid object id"),
            DriveObjectKind::Spreadsheet,
            DataClass::Internal,
        );
        assert!(DocumentDriveBinding::new(wrong_kind).is_err());
    }

    fn actor() -> PrincipalId {
        PrincipalId::new("user-alpha").expect("valid principal")
    }

    fn document_binding() -> DocumentDriveBinding {
        DocumentDriveBinding::new(DriveObjectBinding::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("doc-1").expect("valid object id"),
            DriveObjectKind::Document,
            DataClass::Confidential,
        ))
        .expect("document binding")
    }

    fn document_model() -> DocumentModel {
        DocumentModel::new(
            document_binding(),
            "Launch plan".to_owned(),
            vec![
                DocumentBlock::paragraph("paragraph-1".to_owned(), "Executive summary".to_owned())
                    .expect("paragraph"),
            ],
            1,
        )
        .expect("document model")
    }

    #[test]
    fn document_model_requires_title_blocks_and_drive_document_binding() {
        let model = document_model();

        assert_eq!(model.title(), "Launch plan");
        assert_eq!(model.blocks().len(), 1);
        assert_eq!(model.version_sequence(), 1);
        assert_eq!(
            model.drive_binding().drive_binding().kind(),
            DriveObjectKind::Document
        );

        assert!(
            DocumentModel::new(
                document_binding(),
                " ".to_owned(),
                vec![model.blocks()[0].clone()],
                1
            )
            .is_err()
        );
        assert!(DocumentModel::new(document_binding(), "Untitled".to_owned(), vec![], 1).is_err());
    }

    #[test]
    fn comments_and_suggestions_bind_to_same_document_and_actor() {
        let model = document_model();
        let comment = DocumentComment::new(
            "comment-1".to_owned(),
            model.document_id().clone(),
            actor(),
            "paragraph-1".to_owned(),
            "Clarify launch date".to_owned(),
        )
        .expect("comment");
        let suggestion = DocumentSuggestion::new(
            "suggestion-1".to_owned(),
            model.document_id().clone(),
            actor(),
            "paragraph-1".to_owned(),
            DocumentSuggestionKind::ReplaceText,
            "Replace vague wording".to_owned(),
        )
        .expect("suggestion");

        assert!(comment.validate_for_model(&model).is_ok());
        assert!(suggestion.validate_for_model(&model).is_ok());
        assert_eq!(comment.anchor_id(), "paragraph-1");
        assert_eq!(suggestion.kind(), DocumentSuggestionKind::ReplaceText);
    }

    #[test]
    fn version_history_requires_matching_document_and_monotonic_sequence() {
        let model = document_model();
        let v1 = DocumentVersionRef::new(
            model.document_id().clone(),
            "version-1".to_owned(),
            1,
            actor(),
            1_700_000_000_000,
        )
        .expect("version");
        let v2 = DocumentVersionRef::new(
            model.document_id().clone(),
            "version-2".to_owned(),
            2,
            actor(),
            1_700_000_010_000,
        )
        .expect("version");
        let history =
            DocumentVersionHistory::new(model.document_id().clone(), vec![v1.clone(), v2])
                .expect("history");

        assert_eq!(history.latest().version_id(), "version-2");
        assert_eq!(history.version_count(), 2);
        assert!(
            DocumentVersionHistory::new(model.document_id().clone(), vec![v1.clone(), v1]).is_err()
        );
    }

    #[test]
    fn editor_interaction_requires_anchor_and_expected_version() {
        let model = document_model();
        let interaction = DocumentEditorInteraction::new(
            model.document_id().clone(),
            actor(),
            DocumentEditorInteractionKind::InsertText,
            "paragraph-1".to_owned(),
            model.version_sequence(),
        )
        .expect("interaction");

        assert_eq!(
            interaction.kind(),
            DocumentEditorInteractionKind::InsertText
        );
        assert!(interaction.validate_for_model(&model).is_ok());
        assert!(
            DocumentEditorInteraction::new(
                model.document_id().clone(),
                actor(),
                DocumentEditorInteractionKind::InsertText,
                " ".to_owned(),
                model.version_sequence(),
            )
            .is_err()
        );
    }

    #[test]
    fn docx_import_export_plan_is_drive_bound_and_docx_only() {
        let plan = DocumentDocxFormatPlan::new(document_binding(), FormatJobDirection::Import)
            .expect("docx plan");

        assert_eq!(plan.format_kind(), OfficeFormatKind::Docx);
        assert_eq!(plan.direction(), FormatJobDirection::Import);
        assert!(plan.requires_drive_binding());
        assert!(plan.preserve_comments());
        assert!(plan.preserve_suggestions());

        let wrong_kind = DriveObjectBinding::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("sheet-1").expect("valid object id"),
            DriveObjectKind::Spreadsheet,
            DataClass::Internal,
        );
        assert!(
            DocumentDocxFormatPlan::from_drive_binding(wrong_kind, FormatJobDirection::Export)
                .is_err()
        );
    }
}
