//! Workspace sites kernel.
//!
//! Typed kernel records for the W-Workspace-GA Sites surface named by
//! `docs/products/workspace/PRD.md` and ADR-0029. The kernel owns site/page
//! metadata, CRDT binding to the shared collab runtime, static publication
//! snapshots, and the per-region moderation gate without owning HTTP serving,
//! object storage, or the concrete static generator.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

use oya_collab_runtime_domain::{CollabRuntime, CollabSurface};
use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const SITE_SCHEMA_VERSION: u32 = 1;
const SITE_PAGE_SCHEMA_VERSION: u32 = 1;
const SITE_PUBLISH_SCHEMA_VERSION: u32 = 1;
const MIN_PUBLISH_BYTES: u64 = 1;
const SHA256_PREFIX: &str = "sha256:";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SiteError {
    InvalidSiteId,
    InvalidTenantId,
    InvalidRegion,
    InvalidCellId,
    InvalidTitle,
    InvalidSlug,
    InvalidPageId,
    InvalidPagePath,
    EmptyPageSet,
    DuplicatePageId,
    DuplicatePagePath,
    MissingHomePage,
    MultipleHomePages,
    InvalidBlockId,
    DuplicateBlockId,
    InvalidBlockSourceRef,
    InvalidCollabRuntime,
    InvalidCollabSurface,
    InvalidModerationRegion,
    InvalidModerationPolicyId,
    MissingModerationReview,
    UnexpectedModerationReview,
    ModerationNotApproved,
    InvalidPublishTargetId,
    InvalidHost,
    InvalidCustomDomain,
    InvalidPublishAuthBoundary,
    InvalidSearchIndexBoundary,
    InvalidPublishSnapshotId,
    InvalidArtifactHash,
    InvalidStorageKey,
    EmptyPublishSnapshot,
    SnapshotSiteMismatch,
    SnapshotTenantMismatch,
    SnapshotUnknownPage,
    InvalidTimeOrder,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SiteVisibility {
    PrivateTenant,
    TenantPublic,
    PublicInternet,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SiteBlockKind {
    Text,
    Image,
    FileLink,
    DocEmbed,
    FormEmbed,
    TableOfContents,
    CustomHtml,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ModerationStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SiteCreate {
    pub id: String,                                            // data_class: INTERNAL_ONLY
    pub tenant_id: String,                                     // data_class: INTERNAL_ONLY
    pub region: String,                                        // data_class: INTERNAL_ONLY
    pub cell_id: String,                                       // data_class: INTERNAL_ONLY
    pub title: String,                                         // data_class: PII_QUASI_IDENTIFIER
    pub slug: String,                                          // data_class: PII_QUASI_IDENTIFIER
    pub data_class: Option<PrivacyDataClass>,                  // data_class: INTERNAL_ONLY
    pub collab_runtime: CollabRuntime,                         // data_class: PII_IDENTIFYING
    pub pages: Vec<SitePage>,                                  // data_class: PII_IDENTIFYING
    pub moderation_policy: Option<ContentModerationPolicyRef>, // data_class: INTERNAL_ONLY
    pub indexed_at_epoch_seconds: Option<u64>,                 // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,                         // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64,                         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Site {
    pub id: Classified<String>,                    // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,             // data_class: INTERNAL_ONLY
    pub region: Classified<String>,                // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,               // data_class: INTERNAL_ONLY
    pub title: Classified<String>,                 // data_class: PII_QUASI_IDENTIFIER
    pub slug: Classified<String>,                  // data_class: PII_QUASI_IDENTIFIER
    pub data_class: Classified<PrivacyDataClass>,  // data_class: INTERNAL_ONLY
    pub collab_runtime: Classified<CollabRuntime>, // data_class: PII_IDENTIFYING
    pub pages: Classified<Vec<SitePage>>,          // data_class: PII_IDENTIFYING
    pub moderation_policy: Classified<Option<ContentModerationPolicyRef>>, // data_class: INTERNAL_ONLY
    pub indexed_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>,         // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>,         // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,                   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SitePageCreate {
    pub page_id: String,               // data_class: INTERNAL_ONLY
    pub path: String,                  // data_class: PII_QUASI_IDENTIFIER
    pub title: String,                 // data_class: PII_QUASI_IDENTIFIER
    pub blocks: Vec<SiteBlockRef>,     // data_class: PII_IDENTIFYING
    pub is_homepage: bool,             // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SitePage {
    pub page_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub path: Classified<String>,    // data_class: PII_QUASI_IDENTIFIER
    pub title: Classified<String>,   // data_class: PII_QUASI_IDENTIFIER
    pub blocks: Classified<Vec<SiteBlockRef>>, // data_class: PII_IDENTIFYING
    pub is_homepage: Classified<bool>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SiteBlockRef {
    pub block_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub kind: Classified<SiteBlockKind>, // data_class: INTERNAL_ONLY
    pub source_ref: Classified<Option<String>>, // data_class: PII_IDENTIFYING
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentModerationPolicyRef {
    pub region: Classified<String>,           // data_class: INTERNAL_ONLY
    pub policy_id: Classified<String>,        // data_class: INTERNAL_ONLY
    pub status: Classified<ModerationStatus>, // data_class: INTERNAL_ONLY
    pub reviewed_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SitePublishTarget {
    pub target_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub visibility: Classified<SiteVisibility>, // data_class: INTERNAL_ONLY
    pub host: Classified<String>,      // data_class: PII_QUASI_IDENTIFIER
    pub custom_domain: Classified<Option<String>>, // data_class: PII_QUASI_IDENTIFIER
    pub require_auth: Classified<bool>, // data_class: INTERNAL_ONLY
    pub search_index_allowed: Classified<bool>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SitePublishSnapshotCreate {
    pub snapshot_id: String,                  // data_class: INTERNAL_ONLY
    pub site_id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub target: SitePublishTarget,            // data_class: PII_QUASI_IDENTIFIER
    pub homepage_path: String,                // data_class: PII_QUASI_IDENTIFIER
    pub page_paths: Vec<String>,              // data_class: PII_QUASI_IDENTIFIER
    pub artifact_hash: String,                // data_class: INTERNAL_ONLY
    pub storage_key: String,                  // data_class: INTERNAL_ONLY
    pub byte_len: u64,                        // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub moderation_policy: ContentModerationPolicyRef, // data_class: INTERNAL_ONLY
    pub generated_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SitePublishSnapshot {
    pub snapshot_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub site_id: Classified<String>,     // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub target: Classified<SitePublishTarget>, // data_class: PII_QUASI_IDENTIFIER
    pub homepage_path: Classified<String>, // data_class: PII_QUASI_IDENTIFIER
    pub page_paths: Classified<Vec<String>>, // data_class: PII_QUASI_IDENTIFIER
    pub artifact_hash: Classified<String>, // data_class: INTERNAL_ONLY
    pub storage_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub byte_len: Classified<u64>,       // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub moderation_policy: Classified<ContentModerationPolicyRef>, // data_class: INTERNAL_ONLY
    pub generated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

pub trait SiteReader {
    fn read_page(
        &self,
        tenant_id: &str,
        site_id: &str,
        path: &str,
    ) -> Result<Option<SitePage>, SiteError>;
}

pub trait SitePublisher {
    fn publish_snapshot(&self, snapshot: &SitePublishSnapshot) -> Result<(), SiteError>;
}

impl Site {
    pub fn new(input: SiteCreate) -> Result<Self, SiteError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_site_data_class());
        validate_non_empty(&input.id, SiteError::InvalidSiteId)?;
        validate_non_empty(&input.tenant_id, SiteError::InvalidTenantId)?;
        validate_non_empty(&input.region, SiteError::InvalidRegion)?;
        validate_non_empty(&input.cell_id, SiteError::InvalidCellId)?;
        validate_text(&input.title, SiteError::InvalidTitle)?;
        validate_slug(&input.slug)?;
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
        validate_pages(&input.pages)?;
        if let Some(policy) = &input.moderation_policy {
            policy.validate()?;
            if policy.region.value != input.region {
                return Err(SiteError::InvalidModerationRegion);
            }
        }

        Ok(Self {
            id: internal(input.id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            cell_id: internal(input.cell_id),
            title: Classified::new(input.title, site_metadata_data_class()),
            slug: Classified::new(input.slug, site_metadata_data_class()),
            data_class: internal(data_class),
            collab_runtime: Classified::new(input.collab_runtime, site_content_data_class()),
            pages: Classified::new(input.pages, site_content_data_class()),
            moderation_policy: internal(input.moderation_policy),
            indexed_at_epoch_seconds: internal(input.indexed_at_epoch_seconds),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: internal(SITE_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }

    pub fn page_paths(&self) -> BTreeSet<String> {
        self.pages
            .value
            .iter()
            .map(|page| page.path.value.clone())
            .collect()
    }
}

impl SitePage {
    pub fn new(input: SitePageCreate) -> Result<Self, SiteError> {
        validate_non_empty(&input.page_id, SiteError::InvalidPageId)?;
        validate_site_path(&input.path)?;
        validate_text(&input.title, SiteError::InvalidTitle)?;
        validate_blocks(&input.blocks)?;
        Ok(Self {
            page_id: internal(input.page_id),
            path: Classified::new(input.path, site_metadata_data_class()),
            title: Classified::new(input.title, site_metadata_data_class()),
            blocks: Classified::new(input.blocks, site_content_data_class()),
            is_homepage: internal(input.is_homepage),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: internal(SITE_PAGE_SCHEMA_VERSION),
        })
    }
}

impl SiteBlockRef {
    pub fn new(
        block_id: String,
        kind: SiteBlockKind,
        source_ref: Option<String>,
        data_class: Option<PrivacyDataClass>,
    ) -> Result<Self, SiteError> {
        validate_non_empty(&block_id, SiteError::InvalidBlockId)?;
        validate_optional_text(source_ref.as_deref(), SiteError::InvalidBlockSourceRef)?;
        Ok(Self {
            block_id: internal(block_id),
            kind: internal(kind),
            source_ref: Classified::new(source_ref, site_content_data_class()),
            data_class: internal(data_class.unwrap_or(default_workspace_site_data_class())),
        })
    }
}

impl ContentModerationPolicyRef {
    pub fn new(
        region: String,
        policy_id: String,
        status: ModerationStatus,
        reviewed_at_epoch_seconds: Option<u64>,
    ) -> Result<Self, SiteError> {
        validate_non_empty(&region, SiteError::InvalidModerationRegion)?;
        validate_non_empty(&policy_id, SiteError::InvalidModerationPolicyId)?;
        match (status, reviewed_at_epoch_seconds) {
            (ModerationStatus::Pending, Some(_)) => {
                return Err(SiteError::UnexpectedModerationReview);
            }
            (ModerationStatus::Approved | ModerationStatus::Rejected, None) => {
                return Err(SiteError::MissingModerationReview);
            }
            _ => {}
        }
        Ok(Self {
            region: internal(region),
            policy_id: internal(policy_id),
            status: internal(status),
            reviewed_at_epoch_seconds: internal(reviewed_at_epoch_seconds),
        })
    }

    fn validate(&self) -> Result<(), SiteError> {
        validate_non_empty(&self.region.value, SiteError::InvalidModerationRegion)?;
        validate_non_empty(&self.policy_id.value, SiteError::InvalidModerationPolicyId)?;
        match (self.status.value, self.reviewed_at_epoch_seconds.value) {
            (ModerationStatus::Pending, Some(_)) => Err(SiteError::UnexpectedModerationReview),
            (ModerationStatus::Approved | ModerationStatus::Rejected, None) => {
                Err(SiteError::MissingModerationReview)
            }
            _ => Ok(()),
        }
    }

    pub fn is_approved(&self) -> bool {
        self.status.value == ModerationStatus::Approved
    }
}

impl SitePublishTarget {
    pub fn new(
        target_id: String,
        visibility: SiteVisibility,
        host: String,
        custom_domain: Option<String>,
        require_auth: bool,
        search_index_allowed: bool,
    ) -> Result<Self, SiteError> {
        validate_non_empty(&target_id, SiteError::InvalidPublishTargetId)?;
        validate_host(&host, SiteError::InvalidHost)?;
        if let Some(custom_domain) = custom_domain.as_deref() {
            validate_host(custom_domain, SiteError::InvalidCustomDomain)?;
        }
        validate_publish_boundaries(visibility, require_auth, search_index_allowed)?;
        Ok(Self {
            target_id: internal(target_id),
            visibility: internal(visibility),
            host: Classified::new(host, site_metadata_data_class()),
            custom_domain: Classified::new(custom_domain, site_metadata_data_class()),
            require_auth: internal(require_auth),
            search_index_allowed: internal(search_index_allowed),
        })
    }

    fn validate(&self) -> Result<(), SiteError> {
        validate_non_empty(&self.target_id.value, SiteError::InvalidPublishTargetId)?;
        validate_host(&self.host.value, SiteError::InvalidHost)?;
        if let Some(custom_domain) = self.custom_domain.value.as_deref() {
            validate_host(custom_domain, SiteError::InvalidCustomDomain)?;
        }
        validate_publish_boundaries(
            self.visibility.value,
            self.require_auth.value,
            self.search_index_allowed.value,
        )
    }
}

impl SitePublishSnapshot {
    pub fn new(input: SitePublishSnapshotCreate, site: &Site) -> Result<Self, SiteError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_site_data_class());
        validate_non_empty(&input.snapshot_id, SiteError::InvalidPublishSnapshotId)?;
        validate_non_empty(&input.site_id, SiteError::InvalidSiteId)?;
        validate_non_empty(&input.tenant_id, SiteError::InvalidTenantId)?;
        if input.site_id != site.id.value {
            return Err(SiteError::SnapshotSiteMismatch);
        }
        if input.tenant_id != site.tenant_id.value {
            return Err(SiteError::SnapshotTenantMismatch);
        }
        input.target.validate()?;
        validate_site_path(&input.homepage_path)?;
        validate_publish_page_paths(&input.homepage_path, &input.page_paths, site)?;
        validate_artifact_hash(&input.artifact_hash)?;
        validate_storage_key(&input.storage_key)?;
        if input.byte_len < MIN_PUBLISH_BYTES {
            return Err(SiteError::EmptyPublishSnapshot);
        }
        input.moderation_policy.validate()?;
        if input.moderation_policy.region.value != site.region.value {
            return Err(SiteError::InvalidModerationRegion);
        }
        if !input.moderation_policy.is_approved() {
            return Err(SiteError::ModerationNotApproved);
        }

        Ok(Self {
            snapshot_id: internal(input.snapshot_id),
            site_id: internal(input.site_id),
            tenant_id: internal(input.tenant_id),
            target: Classified::new(input.target, site_metadata_data_class()),
            homepage_path: Classified::new(input.homepage_path, site_metadata_data_class()),
            page_paths: Classified::new(input.page_paths, site_metadata_data_class()),
            artifact_hash: internal(input.artifact_hash),
            storage_key: internal(input.storage_key),
            byte_len: internal(input.byte_len),
            data_class: internal(data_class),
            moderation_policy: internal(input.moderation_policy),
            generated_at_epoch_seconds: internal(input.generated_at_epoch_seconds),
            schema_version: internal(SITE_PUBLISH_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

pub fn default_workspace_site_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn site_content_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn site_metadata_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_quasi_identifier()
}

pub fn workspace_site_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, SiteError> {
    PrivacyDataClass::new(data_class).map_err(|_| SiteError::InvalidDataClass)
}

fn validate_collab_runtime_binding(
    runtime: &CollabRuntime,
    site_id: &str,
    tenant_id: &str,
    region: &str,
    cell_id: &str,
) -> Result<(), SiteError> {
    if runtime.surface.value != CollabSurface::Sites {
        return Err(SiteError::InvalidCollabSurface);
    }
    if runtime.document_id.value != site_id
        || runtime.tenant_id.value != tenant_id
        || runtime.region.value != region
        || runtime.cell_id.value != cell_id
    {
        return Err(SiteError::InvalidCollabRuntime);
    }
    Ok(())
}

fn validate_pages(pages: &[SitePage]) -> Result<(), SiteError> {
    if pages.is_empty() {
        return Err(SiteError::EmptyPageSet);
    }
    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut homepage_count = 0_u32;
    for page in pages {
        validate_non_empty(&page.page_id.value, SiteError::InvalidPageId)?;
        validate_site_path(&page.path.value)?;
        validate_text(&page.title.value, SiteError::InvalidTitle)?;
        validate_blocks(&page.blocks.value)?;
        if !ids.insert(page.page_id.value.clone()) {
            return Err(SiteError::DuplicatePageId);
        }
        if !paths.insert(page.path.value.clone()) {
            return Err(SiteError::DuplicatePagePath);
        }
        if page.is_homepage.value {
            homepage_count += 1;
            if page.path.value != "/" {
                return Err(SiteError::MissingHomePage);
            }
        }
    }
    match homepage_count {
        0 => Err(SiteError::MissingHomePage),
        1 => Ok(()),
        _ => Err(SiteError::MultipleHomePages),
    }
}

fn validate_blocks(blocks: &[SiteBlockRef]) -> Result<(), SiteError> {
    let mut ids = BTreeSet::new();
    for block in blocks {
        validate_non_empty(&block.block_id.value, SiteError::InvalidBlockId)?;
        validate_optional_text(
            block.source_ref.value.as_deref(),
            SiteError::InvalidBlockSourceRef,
        )?;
        if !ids.insert(block.block_id.value.clone()) {
            return Err(SiteError::DuplicateBlockId);
        }
    }
    Ok(())
}

fn validate_publish_page_paths(
    homepage_path: &str,
    page_paths: &[String],
    site: &Site,
) -> Result<(), SiteError> {
    if page_paths.is_empty() {
        return Err(SiteError::EmptyPageSet);
    }
    let site_paths = site.page_paths();
    let mut published_paths = BTreeSet::new();
    for path in page_paths {
        validate_site_path(path)?;
        if !site_paths.contains(path) {
            return Err(SiteError::SnapshotUnknownPage);
        }
        if !published_paths.insert(path.clone()) {
            return Err(SiteError::DuplicatePagePath);
        }
    }
    if !published_paths.contains(homepage_path) || homepage_path != "/" {
        return Err(SiteError::MissingHomePage);
    }
    Ok(())
}

fn validate_publish_boundaries(
    visibility: SiteVisibility,
    require_auth: bool,
    search_index_allowed: bool,
) -> Result<(), SiteError> {
    match visibility {
        SiteVisibility::PrivateTenant | SiteVisibility::TenantPublic if !require_auth => {
            Err(SiteError::InvalidPublishAuthBoundary)
        }
        SiteVisibility::PrivateTenant | SiteVisibility::TenantPublic if search_index_allowed => {
            Err(SiteError::InvalidSearchIndexBoundary)
        }
        SiteVisibility::PublicInternet if require_auth => {
            Err(SiteError::InvalidPublishAuthBoundary)
        }
        _ => Ok(()),
    }
}

fn validate_artifact_hash(hash: &str) -> Result<(), SiteError> {
    if hash.trim() != hash
        || !hash.starts_with(SHA256_PREFIX)
        || hash.len() == SHA256_PREFIX.len()
        || hash.chars().any(char::is_control)
    {
        Err(SiteError::InvalidArtifactHash)
    } else {
        Ok(())
    }
}

fn validate_storage_key(storage_key: &str) -> Result<(), SiteError> {
    if storage_key.trim() != storage_key
        || storage_key.is_empty()
        || storage_key.starts_with('/')
        || storage_key.contains("//")
        || storage_key.chars().any(char::is_control)
    {
        return Err(SiteError::InvalidStorageKey);
    }
    if storage_key
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(SiteError::InvalidStorageKey);
    }
    Ok(())
}

fn validate_host(host: &str, error: SiteError) -> Result<(), SiteError> {
    if host.trim() != host
        || host.is_empty()
        || host.contains("://")
        || host.contains('/')
        || host.contains('\\')
        || host
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(error);
    }
    if host
        .split('.')
        .any(|segment| segment.is_empty() || segment.starts_with('-') || segment.ends_with('-'))
    {
        return Err(error);
    }
    Ok(())
}

fn validate_slug(slug: &str) -> Result<(), SiteError> {
    if slug.trim() != slug || slug.is_empty() || slug.starts_with('-') || slug.ends_with('-') {
        return Err(SiteError::InvalidSlug);
    }
    if !slug.chars().all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
    }) {
        return Err(SiteError::InvalidSlug);
    }
    Ok(())
}

fn validate_site_path(path: &str) -> Result<(), SiteError> {
    if path.trim() != path
        || !path.starts_with('/')
        || path.contains("//")
        || path
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(SiteError::InvalidPagePath);
    }
    if path == "/" {
        return Ok(());
    }
    if path.ends_with('/') {
        return Err(SiteError::InvalidPagePath);
    }
    if path.split('/').skip(1).any(|segment| {
        segment.is_empty()
            || segment == "."
            || segment == ".."
            || !segment.chars().all(is_safe_path_character)
    }) {
        return Err(SiteError::InvalidPagePath);
    }
    Ok(())
}

fn is_safe_path_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | '~')
}

fn validate_time_order(created_at: u64, updated_at: u64) -> Result<(), SiteError> {
    if updated_at < created_at {
        Err(SiteError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_optional_text(value: Option<&str>, error: SiteError) -> Result<(), SiteError> {
    let Some(value) = value else {
        return Ok(());
    };
    validate_text(value, error)
}

fn validate_text(value: &str, error: SiteError) -> Result<(), SiteError> {
    if value.trim() != value || value.is_empty() || value.chars().any(char::is_control) {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_non_empty(value: &str, error: SiteError) -> Result<(), SiteError> {
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
            document_id: "site-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            surface,
            data_class: None,
            snapshot: CollabSnapshotRef::new(
                "snap-1".into(),
                "tenant-1/sites/site-1/snap-1".into(),
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

    fn block(block_id: &str) -> SiteBlockRef {
        SiteBlockRef::new(
            block_id.into(),
            SiteBlockKind::Text,
            Some(format!("content:{block_id}")),
            None,
        )
        .unwrap()
    }

    fn page(path: &str, is_homepage: bool) -> SitePage {
        SitePage::new(SitePageCreate {
            page_id: format!("page-{path}-{is_homepage}"),
            path: path.into(),
            title: format!("Page {path}"),
            blocks: vec![block("block-1")],
            is_homepage,
            updated_at_epoch_seconds: 1_700_000_001,
        })
        .unwrap()
    }

    fn site_input(surface: CollabSurface) -> SiteCreate {
        SiteCreate {
            id: "site-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            title: "Engineering Handbook".into(),
            slug: "engineering-handbook".into(),
            data_class: None,
            collab_runtime: runtime(surface),
            pages: vec![page("/", true), page("/runbooks/on-call", false)],
            moderation_policy: Some(approved_policy()),
            indexed_at_epoch_seconds: None,
            created_at_epoch_seconds: 1_700_000_000,
            updated_at_epoch_seconds: 1_700_000_010,
        }
    }

    fn site() -> Site {
        Site::new(site_input(CollabSurface::Sites)).unwrap()
    }

    fn approved_policy() -> ContentModerationPolicyRef {
        ContentModerationPolicyRef::new(
            "region-alpha1".into(),
            "content-safety-alpha1".into(),
            ModerationStatus::Approved,
            Some(1_700_000_020),
        )
        .unwrap()
    }

    fn pending_policy() -> ContentModerationPolicyRef {
        ContentModerationPolicyRef::new(
            "region-alpha1".into(),
            "content-safety-alpha1".into(),
            ModerationStatus::Pending,
            None,
        )
        .unwrap()
    }

    fn public_target() -> SitePublishTarget {
        SitePublishTarget::new(
            "target-public".into(),
            SiteVisibility::PublicInternet,
            "sites.tenant-1.oyatie.example".into(),
            Some("handbook.example.com".into()),
            false,
            true,
        )
        .unwrap()
    }

    fn snapshot_input() -> SitePublishSnapshotCreate {
        SitePublishSnapshotCreate {
            snapshot_id: "publish-1".into(),
            site_id: "site-1".into(),
            tenant_id: "tenant-1".into(),
            target: public_target(),
            homepage_path: "/".into(),
            page_paths: vec!["/".into(), "/runbooks/on-call".into()],
            artifact_hash: "sha256:abc123".into(),
            storage_key: "tenant-1/sites/site-1/publish-1/index.tar".into(),
            byte_len: 4096,
            data_class: None,
            moderation_policy: approved_policy(),
            generated_at_epoch_seconds: 1_700_000_030,
        }
    }

    #[test]
    fn site_defaults_to_identifying_and_requires_sites_collab_runtime() {
        let site = site();

        assert_eq!(
            site.privacy_data_class().data_class(),
            DataClass::PiiIdentifying
        );
        assert_eq!(
            site.title.data_class,
            DataClassification::Privacy(site_metadata_data_class())
        );
        assert_eq!(
            site.collab_runtime.data_class,
            DataClassification::Privacy(site_content_data_class())
        );

        assert_eq!(
            Site::new(site_input(CollabSurface::Docs)),
            Err(SiteError::InvalidCollabSurface)
        );
    }

    #[test]
    fn page_graph_rejects_path_traversal_duplicate_paths_and_missing_home() {
        assert_eq!(
            SitePage::new(SitePageCreate {
                page_id: "page-bad".into(),
                path: "/../admin".into(),
                title: "Bad".into(),
                blocks: vec![block("block-1")],
                is_homepage: false,
                updated_at_epoch_seconds: 1,
            }),
            Err(SiteError::InvalidPagePath)
        );

        let mut duplicate = site_input(CollabSurface::Sites);
        duplicate.pages = vec![page("/", true), page("/", false)];
        assert_eq!(Site::new(duplicate), Err(SiteError::DuplicatePagePath));

        let mut missing_home = site_input(CollabSurface::Sites);
        missing_home.pages = vec![page("/guide", false)];
        assert_eq!(Site::new(missing_home), Err(SiteError::MissingHomePage));
    }

    #[test]
    fn publish_snapshot_requires_approved_moderation_homepage_and_artifact() {
        let site = site();
        let snapshot = SitePublishSnapshot::new(snapshot_input(), &site).unwrap();
        assert_eq!(
            snapshot.privacy_data_class().data_class(),
            DataClass::PiiIdentifying
        );
        assert_eq!(snapshot.schema_version.value, 1);

        let mut pending = snapshot_input();
        pending.moderation_policy = pending_policy();
        assert_eq!(
            SitePublishSnapshot::new(pending, &site),
            Err(SiteError::ModerationNotApproved)
        );

        let mut missing_home = snapshot_input();
        missing_home.page_paths = vec!["/runbooks/on-call".into()];
        assert_eq!(
            SitePublishSnapshot::new(missing_home, &site),
            Err(SiteError::MissingHomePage)
        );

        let mut bad_hash = snapshot_input();
        bad_hash.artifact_hash = "md5:abc123".into();
        assert_eq!(
            SitePublishSnapshot::new(bad_hash, &site),
            Err(SiteError::InvalidArtifactHash)
        );
    }

    #[test]
    fn publish_target_enforces_auth_and_search_index_boundaries() {
        assert_eq!(
            SitePublishTarget::new(
                "private".into(),
                SiteVisibility::PrivateTenant,
                "private.tenant-1.oyatie.example".into(),
                None,
                false,
                false,
            ),
            Err(SiteError::InvalidPublishAuthBoundary)
        );

        assert_eq!(
            SitePublishTarget::new(
                "tenant".into(),
                SiteVisibility::TenantPublic,
                "tenant.tenant-1.oyatie.example".into(),
                None,
                true,
                true,
            ),
            Err(SiteError::InvalidSearchIndexBoundary)
        );

        assert!(public_target().search_index_allowed.value);
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_site_data_class_from_legacy(DataClass::Audit),
            Err(SiteError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
    }
}

// ---------------------------------------------------------------------------
// M03-P06-IP — workspace.sites STAGING surface markers (SPEC §4 rows).
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SitesSurfaceStaging {
    pub site_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub page_count: Classified<u64>,   // data_class: INTERNAL_ONLY
}

impl SitesSurfaceStaging {
    pub fn new(site_id: String, tenant_id: String, page_count: u64) -> Self {
        Self {
            site_id: Classified::new(site_id, DataClass::InternalOnly),
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            page_count: Classified::new(page_count, DataClass::InternalOnly),
        }
    }
}

#[cfg(test)]
mod m03_p06_tests {
    use super::*;

    fn sample() -> SitesSurfaceStaging {
        SitesSurfaceStaging::new("sites-1".into(), "sites-1".into(), 0u64)
    }

    #[test]
    fn surface_staging_constructor_sets_internal_only() {
        let s = sample();
        assert_eq!(s.site_id.data_class, DataClass::InternalOnly.into());
    }

    #[test]
    fn surface_staging_round_trip_equality() {
        assert_eq!(sample(), sample());
    }
}
