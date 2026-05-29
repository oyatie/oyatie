//! Cloud IaC domain foundation.
//!
//! This crate intentionally starts as a pure domain crate: no filesystem,
//! network, provider SDK, OpenTofu CLI, Argo CD API, or Kubernetes client I/O.
//!
//! # GitOps drift reconciliation
//!
//! [`reconcile_gitops_drift`] compares a _desired_ [`GitOpsEvidence`] (what the
//! declarative pipeline intends) against an _observed_ [`GitOpsEvidence`] (what
//! the GitOps controller last reported) and returns a [`GitOpsDriftReport`].
//!
//! ## Identity contract
//!
//! Both arguments must describe the same
//! `(controller, tenant_id, cell_id, application_name)` tuple.  If they differ
//! the report verdict is [`GitOpsDriftVerdict::IdentityMismatch`] regardless of
//! any other field values.
//!
//! ## Drift rank order (applied only when identities match)
//!
//! 1. [`GitOpsDriftVerdict::DriftedCommit`] — observed `commit_sha` ≠ desired
//! 2. [`GitOpsDriftVerdict::DriftedSyncStatus`] — observed `sync_status` ≠ `Synced`
//! 3. [`GitOpsDriftVerdict::DegradedHealth`] — observed `health_status` ≠ `Healthy`
//! 4. [`GitOpsDriftVerdict::InSync`] — all fields aligned

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudIacError {
    InvalidNamespace,
    InvalidModuleName,
    InvalidModuleSystem,
    InvalidSemanticVersion,
    MissingSourceVersionPin,
    SourceLooksSecretLike,
    InvalidDigest,
    EvidenceRefMissing,
    EvidenceRefLooksSecretLike,
    DuplicateModuleVersion,
    ModuleVersionNotFound,
    InvalidTenantId,
    RegionEmpty,
    CellIdEmpty,
    MissingModuleRefs,
    DefaultCrossCellTrafficForbidden,
    DuplicateCellId,
    CellRegionMismatch,
    TopologyIdEmpty,
    ApplicationNameEmpty,
    RepositoryUrlInvalid,
    InvalidCommitSha,
    CatalogIdEmpty,
    CatalogEmpty,
    CatalogPathInvalid,
    CatalogPathOutsideRoot,
    CatalogMainFileInvalid,
    CatalogSkeletonOverclaim,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OpenTofuModuleRef {
    namespace: String,
    name: String,
    system: String,
    version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenTofuModuleRelease {
    module_ref: OpenTofuModuleRef,
    source: String,
    digest: String,
    evidence_ref: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ModuleRegistry {
    releases: BTreeMap<OpenTofuModuleRef, OpenTofuModuleRelease>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalOpenTofuModuleCatalog {
    catalog_id: String,
    source_path_root: String,
    entries: Vec<LocalOpenTofuModuleCatalogEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalOpenTofuModuleCatalogEntry {
    module_ref: OpenTofuModuleRef,
    source_path: String,
    main_file: String,
    release_status: LocalModuleReleaseStatus,
    provider_resources_implemented: bool,
    outputs_materialized: bool,
    tests_present: bool,
    evidence_ref: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LocalModuleReleaseStatus {
    LocalFoundationSkeleton,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CellIsolationTier {
    Foundation,
    Substrate,
    Capability,
    Application,
    Edge,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellDefinition {
    tenant_id: String,
    region: String,
    cell_id: String,
    isolation_tier: CellIsolationTier,
    module_refs: Vec<OpenTofuModuleRef>,
    default_cross_cell_traffic_allowed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellTopologyPlan {
    topology_id: String,
    region: String,
    evidence_ref: String,
    cells: Vec<CellDefinition>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GitOpsController {
    ArgoCd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GitOpsSyncStatus {
    Synced,
    OutOfSync,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GitOpsHealthStatus {
    Healthy,
    Progressing,
    Degraded,
    Missing,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitOpsEvidence {
    controller: GitOpsController,
    tenant_id: String,
    cell_id: String,
    application_name: String,
    repository_url: String,
    commit_sha: String,
    sync_status: GitOpsSyncStatus,
    health_status: GitOpsHealthStatus,
    evidence_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitOpsEvidenceInput {
    pub controller: GitOpsController,
    pub tenant_id: String,
    pub cell_id: String,
    pub application_name: String,
    pub repository_url: String,
    pub commit_sha: String,
    pub sync_status: GitOpsSyncStatus,
    pub health_status: GitOpsHealthStatus,
    pub evidence_ref: String,
}

impl OpenTofuModuleRelease {
    pub fn new(
        namespace: impl Into<String>,
        name: impl Into<String>,
        system: impl Into<String>,
        version: impl Into<String>,
        source: impl Into<String>,
        digest: impl Into<String>,
        evidence_ref: impl Into<String>,
    ) -> Result<Self, CloudIacError> {
        let namespace = namespace.into();
        let name = name.into();
        let system = system.into();
        let version = version.into();
        let source = source.into();
        let digest = digest.into();
        let evidence_ref = evidence_ref.into();

        validate_slug(&namespace).map_err(|()| CloudIacError::InvalidNamespace)?;
        validate_slug(&name).map_err(|()| CloudIacError::InvalidModuleName)?;
        validate_slug(&system).map_err(|()| CloudIacError::InvalidModuleSystem)?;
        validate_exact_semver(&version)?;
        validate_source(&source, &version)?;
        validate_digest(&digest)?;
        validate_evidence_ref(&evidence_ref)?;

        Ok(Self {
            module_ref: OpenTofuModuleRef {
                namespace,
                name,
                system,
                version,
            },
            source,
            digest,
            evidence_ref,
        })
    }

    pub fn namespace(&self) -> &str {
        &self.module_ref.namespace
    }

    pub fn name(&self) -> &str {
        &self.module_ref.name
    }

    pub fn system(&self) -> &str {
        &self.module_ref.system
    }

    pub fn version(&self) -> &str {
        &self.module_ref.version
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn evidence_ref(&self) -> &str {
        &self.evidence_ref
    }

    pub fn module_ref(&self) -> OpenTofuModuleRef {
        self.module_ref.clone()
    }
}

impl OpenTofuModuleRef {
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn system(&self) -> &str {
        &self.system
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

impl ModuleRegistry {
    pub fn publish(&mut self, release: OpenTofuModuleRelease) -> Result<(), CloudIacError> {
        let module_ref = release.module_ref();
        if self.releases.contains_key(&module_ref) {
            return Err(CloudIacError::DuplicateModuleVersion);
        }
        self.releases.insert(module_ref, release);
        Ok(())
    }

    pub fn versions(
        &self,
        namespace: &str,
        name: &str,
        system: &str,
    ) -> Result<Vec<&OpenTofuModuleRelease>, CloudIacError> {
        validate_slug(namespace).map_err(|()| CloudIacError::InvalidNamespace)?;
        validate_slug(name).map_err(|()| CloudIacError::InvalidModuleName)?;
        validate_slug(system).map_err(|()| CloudIacError::InvalidModuleSystem)?;

        let mut releases = self
            .releases
            .iter()
            .filter_map(|(module_ref, release)| {
                (module_ref.namespace() == namespace
                    && module_ref.name() == name
                    && module_ref.system() == system)
                    .then_some(release)
            })
            .collect::<Vec<_>>();
        releases.sort_by_key(|release| semver_sort_key(release.version()));
        if releases.is_empty() {
            Err(CloudIacError::ModuleVersionNotFound)
        } else {
            Ok(releases)
        }
    }

    pub fn resolve(
        &self,
        namespace: &str,
        name: &str,
        system: &str,
        version: &str,
    ) -> Result<&OpenTofuModuleRelease, CloudIacError> {
        validate_slug(namespace).map_err(|()| CloudIacError::InvalidNamespace)?;
        validate_slug(name).map_err(|()| CloudIacError::InvalidModuleName)?;
        validate_slug(system).map_err(|()| CloudIacError::InvalidModuleSystem)?;
        validate_exact_semver(version)?;

        let module_ref = OpenTofuModuleRef {
            namespace: namespace.to_string(),
            name: name.to_string(),
            system: system.to_string(),
            version: version.to_string(),
        };
        self.releases
            .get(&module_ref)
            .ok_or(CloudIacError::ModuleVersionNotFound)
    }

    pub fn len(&self) -> usize {
        self.releases.len()
    }

    pub fn is_empty(&self) -> bool {
        self.releases.is_empty()
    }
}

fn semver_sort_key(version: &str) -> (u64, u64, u64) {
    let mut parts = version.split('.');
    let major = parts.next().and_then(|part| part.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|part| part.parse().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|part| part.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

impl LocalModuleReleaseStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LocalFoundationSkeleton => "local-foundation-skeleton",
        }
    }
}

impl LocalOpenTofuModuleCatalogEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        namespace: impl Into<String>,
        name: impl Into<String>,
        system: impl Into<String>,
        version: impl Into<String>,
        source_path: impl Into<String>,
        main_file: impl Into<String>,
        release_status: LocalModuleReleaseStatus,
        provider_resources_implemented: bool,
        outputs_materialized: bool,
        tests_present: bool,
        evidence_ref: impl Into<String>,
    ) -> Result<Self, CloudIacError> {
        let namespace = namespace.into();
        let name = name.into();
        let system = system.into();
        let version = version.into();
        let source_path = source_path.into();
        let main_file = main_file.into();
        let evidence_ref = evidence_ref.into();

        validate_slug(&namespace).map_err(|()| CloudIacError::InvalidNamespace)?;
        validate_slug(&name).map_err(|()| CloudIacError::InvalidModuleName)?;
        validate_slug(&system).map_err(|()| CloudIacError::InvalidModuleSystem)?;
        validate_exact_semver(&version)?;
        validate_repo_relative_path(&source_path, false)?;
        validate_repo_relative_path(&main_file, true)?;
        validate_evidence_ref(&evidence_ref)?;

        let expected_main_file = format!("{source_path}/main.tofu");
        if main_file != expected_main_file {
            return Err(CloudIacError::CatalogMainFileInvalid);
        }

        if matches!(
            release_status,
            LocalModuleReleaseStatus::LocalFoundationSkeleton
        ) && (provider_resources_implemented || outputs_materialized || tests_present)
        {
            return Err(CloudIacError::CatalogSkeletonOverclaim);
        }

        Ok(Self {
            module_ref: OpenTofuModuleRef {
                namespace,
                name,
                system,
                version,
            },
            source_path,
            main_file,
            release_status,
            provider_resources_implemented,
            outputs_materialized,
            tests_present,
            evidence_ref,
        })
    }

    pub fn namespace(&self) -> &str {
        &self.module_ref.namespace
    }

    pub fn name(&self) -> &str {
        &self.module_ref.name
    }

    pub fn system(&self) -> &str {
        &self.module_ref.system
    }

    pub fn version(&self) -> &str {
        &self.module_ref.version
    }

    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    pub fn main_file(&self) -> &str {
        &self.main_file
    }

    pub const fn release_status(&self) -> LocalModuleReleaseStatus {
        self.release_status
    }

    pub const fn provider_resources_implemented(&self) -> bool {
        self.provider_resources_implemented
    }

    pub const fn outputs_materialized(&self) -> bool {
        self.outputs_materialized
    }

    pub const fn tests_present(&self) -> bool {
        self.tests_present
    }

    pub fn evidence_ref(&self) -> &str {
        &self.evidence_ref
    }

    pub fn module_ref(&self) -> OpenTofuModuleRef {
        self.module_ref.clone()
    }
}

impl LocalOpenTofuModuleCatalog {
    pub fn new(
        catalog_id: impl Into<String>,
        source_path_root: impl Into<String>,
        entries: Vec<LocalOpenTofuModuleCatalogEntry>,
    ) -> Result<Self, CloudIacError> {
        let catalog_id = catalog_id.into();
        let source_path_root = source_path_root.into();

        if catalog_id.trim().is_empty() {
            return Err(CloudIacError::CatalogIdEmpty);
        }
        validate_slug(&catalog_id).map_err(|()| CloudIacError::CatalogIdEmpty)?;
        validate_repo_relative_path(&source_path_root, false)?;

        if entries.is_empty() {
            return Err(CloudIacError::CatalogEmpty);
        }

        let root_prefix = format!("{source_path_root}/");
        let mut seen: BTreeMap<OpenTofuModuleRef, ()> = BTreeMap::new();
        for entry in &entries {
            if !entry.source_path().starts_with(&root_prefix) {
                return Err(CloudIacError::CatalogPathOutsideRoot);
            }
            if entry.source_path().rsplit('/').next() != Some(entry.name()) {
                return Err(CloudIacError::CatalogPathInvalid);
            }
            let expected_main_file = format!("{}/main.tofu", entry.source_path());
            if entry.main_file() != expected_main_file {
                return Err(CloudIacError::CatalogMainFileInvalid);
            }
            if matches!(
                entry.release_status(),
                LocalModuleReleaseStatus::LocalFoundationSkeleton
            ) && (entry.provider_resources_implemented()
                || entry.outputs_materialized()
                || entry.tests_present())
            {
                return Err(CloudIacError::CatalogSkeletonOverclaim);
            }
            if seen.insert(entry.module_ref(), ()).is_some() {
                return Err(CloudIacError::DuplicateModuleVersion);
            }
        }

        Ok(Self {
            catalog_id,
            source_path_root,
            entries,
        })
    }

    pub fn catalog_id(&self) -> &str {
        &self.catalog_id
    }

    pub fn source_path_root(&self) -> &str {
        &self.source_path_root
    }

    pub fn entries(&self) -> &[LocalOpenTofuModuleCatalogEntry] {
        &self.entries
    }

    pub fn module_count(&self) -> usize {
        self.entries.len()
    }
}

impl CellDefinition {
    pub fn new(
        tenant_id: impl Into<String>,
        region: impl Into<String>,
        cell_id: impl Into<String>,
        isolation_tier: CellIsolationTier,
        module_refs: Vec<OpenTofuModuleRef>,
        default_cross_cell_traffic_allowed: bool,
    ) -> Result<Self, CloudIacError> {
        let tenant_id = tenant_id.into();
        let region = region.into();
        let cell_id = cell_id.into();

        validate_tenant_id(&tenant_id)?;
        validate_region(&region)?;
        validate_cell_id(&cell_id)?;
        if module_refs.is_empty() {
            return Err(CloudIacError::MissingModuleRefs);
        }
        if default_cross_cell_traffic_allowed {
            return Err(CloudIacError::DefaultCrossCellTrafficForbidden);
        }

        Ok(Self {
            tenant_id,
            region,
            cell_id,
            isolation_tier,
            module_refs,
            default_cross_cell_traffic_allowed,
        })
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn region(&self) -> &str {
        &self.region
    }

    pub fn cell_id(&self) -> &str {
        &self.cell_id
    }

    pub const fn isolation_tier(&self) -> CellIsolationTier {
        self.isolation_tier
    }

    pub fn module_refs(&self) -> &[OpenTofuModuleRef] {
        &self.module_refs
    }

    pub const fn default_cross_cell_traffic_allowed(&self) -> bool {
        self.default_cross_cell_traffic_allowed
    }
}

impl CellTopologyPlan {
    pub fn new(
        topology_id: impl Into<String>,
        region: impl Into<String>,
        evidence_ref: impl Into<String>,
    ) -> Result<Self, CloudIacError> {
        let topology_id = topology_id.into();
        let region = region.into();
        let evidence_ref = evidence_ref.into();

        if topology_id.trim().is_empty() {
            return Err(CloudIacError::TopologyIdEmpty);
        }
        validate_region(&region)?;
        validate_evidence_ref(&evidence_ref)?;

        Ok(Self {
            topology_id,
            region,
            evidence_ref,
            cells: Vec::new(),
        })
    }

    pub fn add_cell(mut self, cell: CellDefinition) -> Result<Self, CloudIacError> {
        if cell.region() != self.region {
            return Err(CloudIacError::CellRegionMismatch);
        }
        if self
            .cells
            .iter()
            .any(|existing| existing.cell_id() == cell.cell_id())
        {
            return Err(CloudIacError::DuplicateCellId);
        }
        self.cells.push(cell);
        Ok(self)
    }

    pub fn topology_id(&self) -> &str {
        &self.topology_id
    }

    pub fn region(&self) -> &str {
        &self.region
    }

    pub fn evidence_ref(&self) -> &str {
        &self.evidence_ref
    }

    pub fn cells(&self) -> &[CellDefinition] {
        &self.cells
    }
}

impl GitOpsEvidence {
    pub fn new(input: GitOpsEvidenceInput) -> Result<Self, CloudIacError> {
        validate_tenant_id(&input.tenant_id)?;
        validate_cell_id(&input.cell_id)?;
        if input.application_name.trim().is_empty() {
            return Err(CloudIacError::ApplicationNameEmpty);
        }
        validate_repository_url(&input.repository_url)?;
        validate_commit_sha(&input.commit_sha)?;
        validate_evidence_ref(&input.evidence_ref)?;

        Ok(Self {
            controller: input.controller,
            tenant_id: input.tenant_id,
            cell_id: input.cell_id,
            application_name: input.application_name,
            repository_url: input.repository_url,
            commit_sha: input.commit_sha,
            sync_status: input.sync_status,
            health_status: input.health_status,
            evidence_ref: input.evidence_ref,
        })
    }

    pub const fn controller(&self) -> GitOpsController {
        self.controller
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn cell_id(&self) -> &str {
        &self.cell_id
    }

    pub fn application_name(&self) -> &str {
        &self.application_name
    }

    pub fn repository_url(&self) -> &str {
        &self.repository_url
    }

    pub fn commit_sha(&self) -> &str {
        &self.commit_sha
    }

    pub const fn sync_status(&self) -> GitOpsSyncStatus {
        self.sync_status
    }

    pub const fn health_status(&self) -> GitOpsHealthStatus {
        self.health_status
    }

    pub fn evidence_ref(&self) -> &str {
        &self.evidence_ref
    }

    pub const fn is_converged(&self) -> bool {
        matches!(self.sync_status, GitOpsSyncStatus::Synced)
            && matches!(self.health_status, GitOpsHealthStatus::Healthy)
    }
}

// ---------------------------------------------------------------------------
// GitOps drift reconciliation
// ---------------------------------------------------------------------------

/// Verdict of a desired-vs-observed GitOps drift comparison.
///
/// Variants are ranked: `IdentityMismatch` is checked first; within matching
/// identities the order is `DriftedCommit` → `DriftedSyncStatus` →
/// `DegradedHealth` → `InSync`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GitOpsDriftVerdict {
    /// All observed fields match the desired state.
    InSync,
    /// The observed `commit_sha` differs from the desired `commit_sha`.
    DriftedCommit,
    /// The observed `sync_status` is not `Synced`.
    DriftedSyncStatus,
    /// The observed `health_status` is not `Healthy`.
    DegradedHealth,
    /// The identity tuple `(controller, tenant_id, cell_id, application_name)`
    /// differs between desired and observed.
    IdentityMismatch,
}

/// Report returned by [`reconcile_gitops_drift`].
///
/// Always carries the identity tuple and the _observed_ (not desired) status
/// fields so callers can emit structured telemetry without re-reading either
/// evidence object.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitOpsDriftReport {
    /// The drift verdict.
    pub verdict: GitOpsDriftVerdict,
    /// The GitOps controller observed (taken from the observed evidence).
    pub controller: GitOpsController,
    /// Tenant identifier (taken from the observed evidence).
    pub tenant_id: String,
    /// Cell identifier (taken from the observed evidence).
    pub cell_id: String,
    /// Application name (taken from the observed evidence).
    pub application_name: String,
    /// Commit SHA as last reported by the controller.
    pub observed_commit_sha: String,
    /// Sync status as last reported by the controller.
    pub observed_sync_status: GitOpsSyncStatus,
    /// Health status as last reported by the controller.
    pub observed_health_status: GitOpsHealthStatus,
}

/// Compare a desired GitOps state against an observed GitOps state and return
/// a typed drift report.
///
/// # Identity contract
/// `desired` and `observed` must describe the same
/// `(controller, tenant_id, cell_id, application_name)` tuple.
/// If they differ, the report verdict is
/// [`GitOpsDriftVerdict::IdentityMismatch`] regardless of any other field
/// values.
///
/// # Drift rank order (applied only when identities match)
/// 1. `DriftedCommit`     — observed `commit_sha` != desired `commit_sha`
/// 2. `DriftedSyncStatus` — observed `sync_status` != `GitOpsSyncStatus::Synced`
/// 3. `DegradedHealth`    — observed `health_status` != `GitOpsHealthStatus::Healthy`
/// 4. `InSync`            — all fields aligned
///
/// This function performs no I/O.
pub fn reconcile_gitops_drift(
    desired: &GitOpsEvidence,
    observed: &GitOpsEvidence,
) -> GitOpsDriftReport {
    let verdict = if desired.controller() != observed.controller()
        || desired.tenant_id() != observed.tenant_id()
        || desired.cell_id() != observed.cell_id()
        || desired.application_name() != observed.application_name()
    {
        GitOpsDriftVerdict::IdentityMismatch
    } else if desired.commit_sha() != observed.commit_sha() {
        GitOpsDriftVerdict::DriftedCommit
    } else if observed.sync_status() != GitOpsSyncStatus::Synced {
        GitOpsDriftVerdict::DriftedSyncStatus
    } else if observed.health_status() != GitOpsHealthStatus::Healthy {
        GitOpsDriftVerdict::DegradedHealth
    } else {
        GitOpsDriftVerdict::InSync
    };

    GitOpsDriftReport {
        verdict,
        controller: observed.controller(),
        tenant_id: observed.tenant_id().to_string(),
        cell_id: observed.cell_id().to_string(),
        application_name: observed.application_name().to_string(),
        observed_commit_sha: observed.commit_sha().to_string(),
        observed_sync_status: observed.sync_status(),
        observed_health_status: observed.health_status(),
    }
}

fn validate_slug(value: &str) -> Result<(), ()> {
    if value.trim().is_empty() {
        return Err(());
    }
    if value
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        Ok(())
    } else {
        Err(())
    }
}

fn validate_exact_semver(version: &str) -> Result<(), CloudIacError> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err(CloudIacError::InvalidSemanticVersion);
    }
    for part in parts {
        if part.is_empty() || part.starts_with('+') || part.starts_with('-') {
            return Err(CloudIacError::InvalidSemanticVersion);
        }
        if part.len() > 1 && part.starts_with('0') {
            return Err(CloudIacError::InvalidSemanticVersion);
        }
        if part.parse::<u64>().is_err() {
            return Err(CloudIacError::InvalidSemanticVersion);
        }
    }
    Ok(())
}

fn validate_source(source: &str, version: &str) -> Result<(), CloudIacError> {
    if looks_secret_like(source) {
        return Err(CloudIacError::SourceLooksSecretLike);
    }
    if is_relative_archive_source(source, version) {
        return Ok(());
    }
    if is_object_archive_source(source, version) {
        return Ok(());
    }
    let expected_ref = format!("v{version}");
    if has_exactly_one_source_ref(source, &expected_ref) {
        Ok(())
    } else {
        Err(CloudIacError::MissingSourceVersionPin)
    }
}

fn has_exactly_one_source_ref(source: &str, expected_ref: &str) -> bool {
    let Some((_, query)) = source.split_once('?') else {
        return false;
    };
    let mut matching_ref_count = 0_u8;
    for pair in query.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        if key == "ref" {
            if value != expected_ref {
                return false;
            }
            matching_ref_count = matching_ref_count.saturating_add(1);
        }
    }
    matching_ref_count == 1
}

fn is_relative_archive_source(source: &str, version: &str) -> bool {
    const ARTIFACT_PREFIX: &str = "/artifacts/modules/";
    let Some(file_name) = source.strip_prefix(ARTIFACT_PREFIX) else {
        return false;
    };
    if file_name.is_empty()
        || file_name.contains('/')
        || file_name.contains('\\')
        || file_name.contains('?')
        || file_name.contains('#')
        || file_name == "."
        || file_name == ".."
        || !file_name.ends_with(".zip")
        || !file_name.ends_with(&format!("-{version}.zip"))
    {
        return false;
    }
    file_name
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '.'))
}

fn is_object_archive_source(source: &str, version: &str) -> bool {
    let Some(location) = source
        .strip_prefix("s3::https://")
        .or_else(|| source.strip_prefix("gcs::https://"))
    else {
        return false;
    };
    if source.contains('@')
        || source.contains('?')
        || source.contains('#')
        || source.contains('\\')
        || source
            .chars()
            .any(|ch| ch.is_ascii_whitespace() || ch.is_control())
    {
        return false;
    }
    let Some((host, object_path)) = location.split_once('/') else {
        return false;
    };
    if host.is_empty() || object_path.is_empty() || object_path.contains("//") {
        return false;
    }
    let Some(file_name) = object_path.rsplit('/').next() else {
        return false;
    };
    if file_name.is_empty()
        || file_name == "."
        || file_name == ".."
        || !file_name.ends_with(".zip")
        || !file_name.ends_with(&format!("-{version}.zip"))
    {
        return false;
    }
    file_name
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '.'))
}

fn validate_digest(digest: &str) -> Result<(), CloudIacError> {
    let Some(hex) = digest.strip_prefix("sha256:") else {
        return Err(CloudIacError::InvalidDigest);
    };
    if hex.len() == 64 && hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(CloudIacError::InvalidDigest)
    }
}

fn validate_evidence_ref(evidence_ref: &str) -> Result<(), CloudIacError> {
    if evidence_ref.trim().is_empty() {
        return Err(CloudIacError::EvidenceRefMissing);
    }
    if looks_secret_like(evidence_ref) {
        return Err(CloudIacError::EvidenceRefLooksSecretLike);
    }
    if evidence_ref.starts_with("evidence://") {
        Ok(())
    } else {
        Err(CloudIacError::EvidenceRefMissing)
    }
}

fn validate_repo_relative_path(path: &str, allow_dot: bool) -> Result<(), CloudIacError> {
    if path.trim().is_empty() || path.starts_with('/') || path.contains('\\') {
        return Err(CloudIacError::CatalogPathInvalid);
    }
    for segment in path.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(CloudIacError::CatalogPathInvalid);
        }
        let valid_segment = segment.chars().all(|ch| {
            ch.is_ascii_lowercase()
                || ch.is_ascii_digit()
                || ch == '-'
                || ch == '_'
                || (allow_dot && ch == '.')
        });
        if !valid_segment {
            return Err(CloudIacError::CatalogPathInvalid);
        }
    }
    Ok(())
}

fn validate_tenant_id(tenant_id: &str) -> Result<(), CloudIacError> {
    let Some(suffix) = tenant_id.strip_prefix("ten_") else {
        return Err(CloudIacError::InvalidTenantId);
    };
    if !suffix.is_empty()
        && suffix
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-')
    {
        Ok(())
    } else {
        Err(CloudIacError::InvalidTenantId)
    }
}

fn validate_region(region: &str) -> Result<(), CloudIacError> {
    if region.trim().is_empty()
        || !region
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
        || !region.contains('-')
    {
        Err(CloudIacError::RegionEmpty)
    } else {
        Ok(())
    }
}

fn validate_cell_id(cell_id: &str) -> Result<(), CloudIacError> {
    let Some(suffix) = cell_id.strip_prefix("cell-") else {
        return Err(CloudIacError::CellIdEmpty);
    };
    if !suffix.is_empty()
        && suffix
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
        && suffix.contains('-')
    {
        Ok(())
    } else {
        Err(CloudIacError::CellIdEmpty)
    }
}

fn validate_repository_url(repository_url: &str) -> Result<(), CloudIacError> {
    if looks_secret_like(repository_url) {
        return Err(CloudIacError::RepositoryUrlInvalid);
    }
    if repository_url.starts_with("https://") || repository_url.starts_with("ssh://") {
        Ok(())
    } else {
        Err(CloudIacError::RepositoryUrlInvalid)
    }
}

fn validate_commit_sha(commit_sha: &str) -> Result<(), CloudIacError> {
    if commit_sha.len() == 40 && commit_sha.chars().all(|ch| ch.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(CloudIacError::InvalidCommitSha)
    }
}

fn looks_secret_like(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("token=")
        || lower.contains("password=")
        || lower.contains("secret=")
        || lower.contains("kubeconfig")
        || lower.contains("-----begin")
        || lower.contains("sk-live")
        || lower.contains("sk-")
}
