//! Oya VCS kernel.
//!
//! Pure value objects and invariants for the agent-native ChangeSet queue.
//! This crate deliberately has no Git, GitHub, CI, filesystem watcher, network,
//! or provider dependency. Grit-compatible adapters own repo transitions; the
//! kernel only models advisory/admission invariants for claims, semantic locks,
//! queue-aware leases, virtual-head projections, ChangeSets, and promotion state.
//! It is not a parallel lock authority or repo-state store.

use std::collections::BTreeSet;
use std::fmt;

const CHANGESET_SCHEMA_VERSION: u32 = 1;
const SYMBOL_ID_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ClaimMode {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SymbolLanguage {
    Rust,
    TypeScript,
    JavaScript,
    Swift,
    Kotlin,
    CSharp,
    Cpp,
    Xaml,
    Json,
    Yaml,
    Toml,
    OpenApi,
    AsyncApi,
    Protobuf,
    Cedar,
    Sql,
    Config,
    Unknown,
}

impl SymbolLanguage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::JavaScript => "javascript",
            Self::Swift => "swift",
            Self::Kotlin => "kotlin",
            Self::CSharp => "csharp",
            Self::Cpp => "cpp",
            Self::Xaml => "xaml",
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::OpenApi => "openapi",
            Self::AsyncApi => "asyncapi",
            Self::Protobuf => "protobuf",
            Self::Cedar => "cedar",
            Self::Sql => "sql",
            Self::Config => "config",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ArtifactSelectorKind {
    WholeFile,
    JsonPointer,
    YamlPointer,
    TomlTable,
    OpenApiOperation,
    AsyncApiChannel,
    ProtobufSymbol,
    CedarPolicy,
    SqlMigration,
    TerraformResource,
    CargoPackage,
    PackageManifest,
    XamlBinding,
}

impl ArtifactSelectorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WholeFile => "file",
            Self::JsonPointer => "json-pointer",
            Self::YamlPointer => "yaml-pointer",
            Self::TomlTable => "toml-table",
            Self::OpenApiOperation => "openapi-operation",
            Self::AsyncApiChannel => "asyncapi-channel",
            Self::ProtobufSymbol => "protobuf-symbol",
            Self::CedarPolicy => "cedar-policy",
            Self::SqlMigration => "sql-migration",
            Self::TerraformResource => "terraform-resource",
            Self::CargoPackage => "cargo-package",
            Self::PackageManifest => "package-manifest",
            Self::XamlBinding => "xaml-binding",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ArtifactPointer {
    pub path: String,                        // data_class: INTERNAL_ONLY
    pub selector_kind: ArtifactSelectorKind, // data_class: INTERNAL_ONLY
    pub selector: Option<String>,            // data_class: INTERNAL_ONLY
}

impl ArtifactPointer {
    pub fn file(path: impl Into<String>) -> Result<Self, VcsKernelError> {
        Self::new(path, ArtifactSelectorKind::WholeFile, None)
    }

    pub fn new(
        path: impl Into<String>,
        selector_kind: ArtifactSelectorKind,
        selector: Option<String>,
    ) -> Result<Self, VcsKernelError> {
        let path = normalize_non_empty(path.into(), VcsKernelError::InvalidArtifactPointer)?;
        let selector = selector
            .map(|value| normalize_non_empty(value, VcsKernelError::InvalidArtifactPointer))
            .transpose()?;
        if selector_kind != ArtifactSelectorKind::WholeFile && selector.is_none() {
            return Err(VcsKernelError::InvalidArtifactPointer);
        }
        Ok(Self {
            path,
            selector_kind,
            selector,
        })
    }

    /// Stable collision-resistant field encoding for symbol lock keys.
    ///
    /// It is human-readable but length-prefixed, so a whole-file path that
    /// contains selector delimiters cannot collide with a pointer-scoped artifact.
    pub fn stable_fragment(&self) -> String {
        [
            encode_field("path", &self.path),
            encode_field("selector_kind", self.selector_kind.as_str()),
            encode_field("selector", self.selector.as_deref().unwrap_or("")),
        ]
        .join("|")
    }

    pub fn covers(&self, touched: &ArtifactPointer) -> bool {
        self == touched
            || (self.path == touched.path && self.selector_kind == ArtifactSelectorKind::WholeFile)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SymbolId {
    pub value: String,             // data_class: INTERNAL_ONLY
    pub language: SymbolLanguage,  // data_class: INTERNAL_ONLY
    pub artifact: ArtifactPointer, // data_class: INTERNAL_ONLY
    pub symbol_path: String,       // data_class: INTERNAL_ONLY
    pub schema_version: u32,       // data_class: INTERNAL_ONLY
}

impl SymbolId {
    pub fn new(
        language: SymbolLanguage,
        artifact: ArtifactPointer,
        symbol_path: impl Into<String>,
    ) -> Result<Self, VcsKernelError> {
        let symbol_path = normalize_symbol_path(symbol_path.into())?;
        let value = [
            format!("sym:v{SYMBOL_ID_SCHEMA_VERSION}"),
            encode_field("language", language.as_str()),
            artifact.stable_fragment(),
            encode_field("symbol", &symbol_path),
        ]
        .join("|");
        Ok(Self {
            value,
            language,
            artifact,
            symbol_path,
            schema_version: SYMBOL_ID_SCHEMA_VERSION,
        })
    }

    pub fn file_scope(
        language: SymbolLanguage,
        path: impl Into<String>,
    ) -> Result<Self, VcsKernelError> {
        Self::new(language, ArtifactPointer::file(path)?, "<file>")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SymbolLock {
    pub symbol_id: SymbolId, // data_class: INTERNAL_ONLY
    pub mode: ClaimMode,     // data_class: INTERNAL_ONLY
}

impl SymbolLock {
    pub fn read(symbol_id: SymbolId) -> Self {
        Self {
            symbol_id,
            mode: ClaimMode::Read,
        }
    }

    pub fn write(symbol_id: SymbolId) -> Self {
        Self {
            symbol_id,
            mode: ClaimMode::Write,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimState {
    Requested,
    Granted,
    Working,
    DoneSubmitted,
    PromotionSubmitted,
    TerminalReleased,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Claim {
    pub id: String,                   // data_class: INTERNAL_ONLY
    pub agent_id: String,             // data_class: INTERNAL_ONLY
    pub intent: String,               // data_class: INTERNAL_ONLY
    pub write_symbols: Vec<SymbolId>, // data_class: INTERNAL_ONLY
    pub read_symbols: Vec<SymbolId>,  // data_class: INTERNAL_ONLY
    pub state: ClaimState,            // data_class: INTERNAL_ONLY
    pub ttl_seconds: u64,             // data_class: INTERNAL_ONLY
}

impl Claim {
    pub fn new(
        id: impl Into<String>,
        agent_id: impl Into<String>,
        intent: impl Into<String>,
        write_symbols: Vec<SymbolId>,
        read_symbols: Vec<SymbolId>,
        ttl_seconds: u64,
    ) -> Result<Self, VcsKernelError> {
        let id = validate_prefixed(id.into(), "claim_", VcsKernelError::InvalidClaimId)?;
        let agent_id = normalize_non_empty(agent_id.into(), VcsKernelError::InvalidAgentId)?;
        let intent = normalize_non_empty(intent.into(), VcsKernelError::EmptyIntent)?;
        if write_symbols.is_empty() && read_symbols.is_empty() {
            return Err(VcsKernelError::MissingClaimSymbols);
        }
        validate_unique_symbols(&write_symbols)?;
        validate_unique_symbols(&read_symbols)?;
        if ttl_seconds == 0 {
            return Err(VcsKernelError::InvalidLeaseTtl);
        }
        Ok(Self {
            id,
            agent_id,
            intent,
            write_symbols,
            read_symbols,
            state: ClaimState::Requested,
            ttl_seconds,
        })
    }

    pub fn grant(mut self) -> Self {
        self.state = ClaimState::Granted;
        self
    }

    pub fn start_work(mut self) -> Result<Self, VcsKernelError> {
        if self.state != ClaimState::Granted {
            return Err(VcsKernelError::InvalidClaimTransition);
        }
        self.state = ClaimState::Working;
        Ok(self)
    }

    pub fn all_locks(&self) -> Vec<SymbolLock> {
        self.write_symbols
            .iter()
            .cloned()
            .map(SymbolLock::write)
            .chain(self.read_symbols.iter().cloned().map(SymbolLock::read))
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewState {
    NotSubmitted,
    Pending,
    Approved,
    ChangesRequested,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CiState {
    NotRun,
    Pending,
    Passed,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueueState {
    Draft,
    Ready,
    QueueStable,
    VirtualMerged,
    PhysicallyMerged,
    Superseded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromotionState {
    NotRequested,
    Requested,
    Admitted,
    PromotedDev,
    PromotedStaging,
    PromotedProduction,
    Rejected,
    RolledBack,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LeaseState {
    Working,
    Submitted,
    QueueStable,
    VirtualMerged,
    MergedDev,
    PromotedStaging,
    PromotedProduction,
    Superseded,
    Released,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueAwareLease {
    pub claim_id: String,                    // data_class: INTERNAL_ONLY
    pub symbols: Vec<SymbolId>,              // data_class: INTERNAL_ONLY
    pub state: LeaseState,                   // data_class: INTERNAL_ONLY
    pub virtual_predecessor: Option<String>, // data_class: INTERNAL_ONLY
}

impl QueueAwareLease {
    pub fn from_claim(claim: &Claim) -> Result<Self, VcsKernelError> {
        if claim.state != ClaimState::Working {
            return Err(VcsKernelError::InvalidClaimTransition);
        }
        Ok(Self {
            claim_id: claim.id.clone(),
            symbols: claim.write_symbols.clone(),
            state: LeaseState::Working,
            virtual_predecessor: None,
        })
    }

    pub fn submit(&mut self) -> Result<(), VcsKernelError> {
        self.transition(LeaseState::Working, LeaseState::Submitted)
    }

    pub fn mark_queue_stable(
        &mut self,
        virtual_predecessor: impl Into<String>,
    ) -> Result<(), VcsKernelError> {
        self.transition(LeaseState::Submitted, LeaseState::QueueStable)?;
        self.virtual_predecessor = Some(normalize_non_empty(
            virtual_predecessor.into(),
            VcsKernelError::InvalidVirtualHead,
        )?);
        Ok(())
    }

    pub fn mark_virtual_merged(&mut self) -> Result<(), VcsKernelError> {
        self.transition(LeaseState::QueueStable, LeaseState::VirtualMerged)
    }

    pub fn mark_merged_dev(&mut self) -> Result<(), VcsKernelError> {
        self.transition(LeaseState::VirtualMerged, LeaseState::MergedDev)
    }

    pub fn mark_promoted_staging(&mut self) -> Result<(), VcsKernelError> {
        self.transition(LeaseState::MergedDev, LeaseState::PromotedStaging)
    }

    pub fn mark_promoted_production(&mut self) -> Result<(), VcsKernelError> {
        self.transition(LeaseState::PromotedStaging, LeaseState::PromotedProduction)
    }

    pub fn release_after_terminal_promotion(&mut self) -> Result<(), VcsKernelError> {
        match self.state {
            LeaseState::PromotedProduction | LeaseState::Superseded => {
                self.state = LeaseState::Released;
                Ok(())
            }
            _ => Err(VcsKernelError::LeaseCannotReleaseBeforeTerminalPromotion),
        }
    }

    pub fn release_due_to_superseded(&mut self) -> Result<(), VcsKernelError> {
        self.state = LeaseState::Superseded;
        self.release_after_terminal_promotion()
    }

    pub fn allows_next_agent_with_virtual_predecessor(&self, predecessor: &str) -> bool {
        self.state == LeaseState::QueueStable
            && self.virtual_predecessor.as_deref() == Some(predecessor)
    }

    fn transition(&mut self, expected: LeaseState, next: LeaseState) -> Result<(), VcsKernelError> {
        if self.state == expected {
            self.state = next;
            Ok(())
        } else {
            Err(VcsKernelError::InvalidLeaseTransition)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeSetLineage {
    pub work_item_id: String,           // data_class: INTERNAL_ONLY
    pub issue_plan_id: String,          // data_class: INTERNAL_ONLY
    pub parent_changesets: Vec<String>, // data_class: INTERNAL_ONLY
}

impl ChangeSetLineage {
    pub fn new(
        work_item_id: impl Into<String>,
        issue_plan_id: impl Into<String>,
        parent_changesets: Vec<String>,
    ) -> Result<Self, VcsKernelError> {
        Ok(Self {
            work_item_id: validate_prefixed(
                work_item_id.into(),
                "wi_",
                VcsKernelError::InvalidLineage,
            )?,
            issue_plan_id: validate_prefixed(
                issue_plan_id.into(),
                "ip_",
                VcsKernelError::InvalidLineage,
            )?,
            parent_changesets,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeSet {
    pub id: String,                          // data_class: INTERNAL_ONLY
    pub agent_id: String,                    // data_class: INTERNAL_ONLY
    pub target_branch: String,               // data_class: INTERNAL_ONLY
    pub base_sha: String,                    // data_class: INTERNAL_ONLY
    pub branch_or_workspace_ref: String,     // data_class: INTERNAL_ONLY
    pub patch_id: String,                    // data_class: INTERNAL_ONLY
    pub write_symbols: Vec<SymbolId>,        // data_class: INTERNAL_ONLY
    pub read_symbols: Vec<SymbolId>,         // data_class: INTERNAL_ONLY
    pub touched_files: Vec<ArtifactPointer>, // data_class: INTERNAL_ONLY
    pub dependencies: Vec<String>,           // data_class: INTERNAL_ONLY
    pub review_state: ReviewState,           // data_class: INTERNAL_ONLY
    pub ci_state: CiState,                   // data_class: INTERNAL_ONLY
    pub queue_state: QueueState,             // data_class: INTERNAL_ONLY
    pub promotion_state: PromotionState,     // data_class: INTERNAL_ONLY
    pub lineage: ChangeSetLineage,           // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,          // data_class: INTERNAL_ONLY
    pub schema_version: u32,                 // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeSetDraft {
    pub id: String,                          // data_class: INTERNAL_ONLY
    pub agent_id: String,                    // data_class: INTERNAL_ONLY
    pub target_branch: String,               // data_class: INTERNAL_ONLY
    pub base_sha: String,                    // data_class: INTERNAL_ONLY
    pub branch_or_workspace_ref: String,     // data_class: INTERNAL_ONLY
    pub patch_id: String,                    // data_class: INTERNAL_ONLY
    pub write_symbols: Vec<SymbolId>,        // data_class: INTERNAL_ONLY
    pub read_symbols: Vec<SymbolId>,         // data_class: INTERNAL_ONLY
    pub touched_files: Vec<ArtifactPointer>, // data_class: INTERNAL_ONLY
    pub dependencies: Vec<String>,           // data_class: INTERNAL_ONLY
    pub lineage: ChangeSetLineage,           // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,          // data_class: INTERNAL_ONLY
}

impl ChangeSet {
    pub fn new(draft: ChangeSetDraft) -> Result<Self, VcsKernelError> {
        let id = validate_prefixed(draft.id, "cs_", VcsKernelError::InvalidChangeSetId)?;
        let agent_id = normalize_non_empty(draft.agent_id, VcsKernelError::InvalidAgentId)?;
        let target_branch = normalize_non_empty(draft.target_branch, VcsKernelError::EmptyTarget)?;
        let base_sha = validate_base_sha(draft.base_sha)?;
        let branch_or_workspace_ref = normalize_non_empty(
            draft.branch_or_workspace_ref,
            VcsKernelError::EmptyWorkspace,
        )?;
        let patch_id = validate_prefixed(draft.patch_id, "patch_", VcsKernelError::InvalidPatchId)?;
        if draft.write_symbols.is_empty() {
            return Err(VcsKernelError::MissingClaimSymbols);
        }
        validate_unique_symbols(&draft.write_symbols)?;
        validate_unique_symbols(&draft.read_symbols)?;
        if draft.touched_files.is_empty() {
            return Err(VcsKernelError::MissingTouchedFiles);
        }
        if draft.evidence_refs.is_empty() {
            return Err(VcsKernelError::MissingEvidence);
        }
        Ok(Self {
            id,
            agent_id,
            target_branch,
            base_sha,
            branch_or_workspace_ref,
            patch_id,
            write_symbols: draft.write_symbols,
            read_symbols: draft.read_symbols,
            touched_files: draft.touched_files,
            dependencies: draft.dependencies,
            review_state: ReviewState::NotSubmitted,
            ci_state: CiState::NotRun,
            queue_state: QueueState::Draft,
            promotion_state: PromotionState::NotRequested,
            lineage: draft.lineage,
            evidence_refs: draft.evidence_refs,
            schema_version: CHANGESET_SCHEMA_VERSION,
        })
    }

    pub fn mark_ready_for_queue(&mut self) -> Result<(), VcsKernelError> {
        if self.review_state != ReviewState::Approved || self.ci_state != CiState::Passed {
            return Err(VcsKernelError::ChangeSetNotAdmissible);
        }
        self.queue_state = QueueState::Ready;
        Ok(())
    }

    pub fn attach_review(&mut self, state: ReviewState) {
        self.review_state = state;
    }

    pub fn attach_ci(&mut self, state: CiState) {
        self.ci_state = state;
    }

    pub fn request_promotion(&mut self) -> Result<(), VcsKernelError> {
        if self.queue_state != QueueState::QueueStable
            && self.queue_state != QueueState::VirtualMerged
        {
            return Err(VcsKernelError::ChangeSetNotAdmissible);
        }
        if self.evidence_refs.is_empty() {
            return Err(VcsKernelError::MissingEvidence);
        }
        self.promotion_state = PromotionState::Requested;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VirtualHeadStatus {
    Pending,
    Building,
    Stable,
    Invalidated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VirtualHead {
    pub queue_name: String,              // data_class: INTERNAL_ONLY
    pub ordered_changesets: Vec<String>, // data_class: INTERNAL_ONLY
    pub base_ref: String,                // data_class: INTERNAL_ONLY
    pub projection_ref: String,          // data_class: INTERNAL_ONLY
    pub required_checks: Vec<String>,    // data_class: INTERNAL_ONLY
    pub status: VirtualHeadStatus,       // data_class: INTERNAL_ONLY
    pub invalidated_by: Vec<String>,     // data_class: INTERNAL_ONLY
}

impl VirtualHead {
    pub fn new(
        queue_name: impl Into<String>,
        ordered_changesets: Vec<String>,
        base_ref: impl Into<String>,
        projection_ref: impl Into<String>,
        required_checks: Vec<String>,
    ) -> Result<Self, VcsKernelError> {
        if ordered_changesets.is_empty() || required_checks.is_empty() {
            return Err(VcsKernelError::InvalidVirtualHead);
        }
        Ok(Self {
            queue_name: normalize_non_empty(queue_name.into(), VcsKernelError::InvalidVirtualHead)?,
            ordered_changesets,
            base_ref: normalize_non_empty(base_ref.into(), VcsKernelError::InvalidVirtualHead)?,
            projection_ref: normalize_non_empty(
                projection_ref.into(),
                VcsKernelError::InvalidVirtualHead,
            )?,
            required_checks,
            status: VirtualHeadStatus::Pending,
            invalidated_by: Vec::new(),
        })
    }

    pub fn is_projection_only(&self) -> bool {
        true
    }

    pub fn invalidate_for(&mut self, reason: impl Into<String>) -> Result<(), VcsKernelError> {
        self.invalidated_by.push(normalize_non_empty(
            reason.into(),
            VcsKernelError::InvalidVirtualHead,
        )?);
        self.status = VirtualHeadStatus::Invalidated;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimCompatibility {
    Compatible,
    Conflict,
}

pub fn claim_compatibility(left: &Claim, right: &Claim) -> ClaimCompatibility {
    let left_writes = symbol_values(&left.write_symbols);
    let right_writes = symbol_values(&right.write_symbols);
    let left_reads = symbol_values(&left.read_symbols);
    let right_reads = symbol_values(&right.read_symbols);

    if !left_writes.is_disjoint(&right_writes)
        || !left_writes.is_disjoint(&right_reads)
        || !right_writes.is_disjoint(&left_reads)
    {
        ClaimCompatibility::Conflict
    } else {
        ClaimCompatibility::Compatible
    }
}

pub fn required_claim_coverage(
    changeset: &ChangeSet,
    active_claim: &Claim,
) -> Result<(), VcsKernelError> {
    if active_claim.state != ClaimState::Working {
        return Err(VcsKernelError::InvalidClaimTransition);
    }
    let claim_writes = symbol_values(&active_claim.write_symbols);
    let changeset_writes = symbol_values(&changeset.write_symbols);
    if !changeset_writes.is_subset(&claim_writes) {
        return Err(VcsKernelError::UnclaimedWriteSymbol);
    }
    for touched in &changeset.touched_files {
        if !active_claim
            .write_symbols
            .iter()
            .any(|symbol| symbol.artifact.covers(touched))
        {
            return Err(VcsKernelError::UnclaimedTouchedArtifact);
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VcsKernelError {
    InvalidArtifactPointer,
    InvalidSymbolPath,
    InvalidClaimId,
    InvalidAgentId,
    EmptyIntent,
    MissingClaimSymbols,
    DuplicateSymbol,
    InvalidLeaseTtl,
    InvalidClaimTransition,
    InvalidLeaseTransition,
    LeaseCannotReleaseBeforeTerminalPromotion,
    InvalidLineage,
    InvalidChangeSetId,
    EmptyTarget,
    EmptyBaseRef,
    EmptyWorkspace,
    InvalidPatchId,
    MissingTouchedFiles,
    MissingEvidence,
    ChangeSetNotAdmissible,
    InvalidVirtualHead,
    UnclaimedWriteSymbol,
    UnclaimedTouchedArtifact,
}

impl fmt::Display for VcsKernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for VcsKernelError {}

fn encode_field(label: &str, value: &str) -> String {
    format!("{label}:{}:{value}", value.len())
}

fn normalize_non_empty(value: String, error: VcsKernelError) -> Result<String, VcsKernelError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        Err(error)
    } else {
        Ok(value)
    }
}

fn validate_prefixed(
    value: String,
    prefix: &str,
    error: VcsKernelError,
) -> Result<String, VcsKernelError> {
    let value = normalize_non_empty(value, error.clone())?;
    if value.starts_with(prefix) && value.len() > prefix.len() {
        Ok(value)
    } else {
        Err(error)
    }
}

fn validate_base_sha(value: String) -> Result<String, VcsKernelError> {
    let value = normalize_non_empty(value, VcsKernelError::EmptyBaseRef)?;
    let valid_len = value.len() == 40 || value.len() == 64;
    if valid_len && value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        Ok(value)
    } else {
        Err(VcsKernelError::EmptyBaseRef)
    }
}

fn normalize_symbol_path(value: String) -> Result<String, VcsKernelError> {
    let value = normalize_non_empty(value, VcsKernelError::InvalidSymbolPath)?;
    if value.contains('\n') || value.contains('\r') || value.contains("//") {
        return Err(VcsKernelError::InvalidSymbolPath);
    }
    Ok(value)
}

fn validate_unique_symbols(symbols: &[SymbolId]) -> Result<(), VcsKernelError> {
    let mut seen = BTreeSet::new();
    for symbol in symbols {
        if !seen.insert(symbol.value.clone()) {
            return Err(VcsKernelError::DuplicateSymbol);
        }
    }
    Ok(())
}

fn symbol_values(symbols: &[SymbolId]) -> BTreeSet<String> {
    symbols.iter().map(|symbol| symbol.value.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE_SHA: &str = "0123456789012345678901234567890123456789";

    fn rust_symbol(path: &str, symbol: &str) -> SymbolId {
        SymbolId::new(
            SymbolLanguage::Rust,
            ArtifactPointer::file(path).expect("valid path"),
            symbol,
        )
        .expect("valid symbol")
    }

    fn claim(id: &str, write_symbols: Vec<SymbolId>, read_symbols: Vec<SymbolId>) -> Claim {
        Claim::new(
            id,
            "codex-autopilot",
            "test claim",
            write_symbols,
            read_symbols,
            600,
        )
        .expect("valid claim")
        .grant()
        .start_work()
        .expect("claim can start")
    }

    #[test]
    fn same_file_different_symbol_write_claims_are_compatible() {
        let left = claim(
            "claim_left",
            vec![rust_symbol("src/lib.rs", "module::alpha")],
            vec![],
        );
        let right = claim(
            "claim_right",
            vec![rust_symbol("src/lib.rs", "module::beta")],
            vec![],
        );

        assert_eq!(
            claim_compatibility(&left, &right),
            ClaimCompatibility::Compatible
        );
    }

    #[test]
    fn same_symbol_write_claims_conflict() {
        let symbol = rust_symbol("src/lib.rs", "module::alpha");
        let left = claim("claim_left", vec![symbol.clone()], vec![]);
        let right = claim("claim_right", vec![symbol], vec![]);

        assert_eq!(
            claim_compatibility(&left, &right),
            ClaimCompatibility::Conflict
        );
    }

    #[test]
    fn symbol_encoding_separates_whole_file_from_pointer_scope() {
        let whole_file = SymbolId::new(
            SymbolLanguage::OpenApi,
            ArtifactPointer::file("contracts/vcs.yaml#openapi-operation:POST /claim").unwrap(),
            "operation::claim",
        )
        .unwrap();
        let pointer = SymbolId::new(
            SymbolLanguage::OpenApi,
            ArtifactPointer::new(
                "contracts/vcs.yaml",
                ArtifactSelectorKind::OpenApiOperation,
                Some("POST /claim".into()),
            )
            .unwrap(),
            "operation::claim",
        )
        .unwrap();

        assert_ne!(whole_file.value, pointer.value);
    }

    #[test]
    fn queue_aware_lease_releases_only_after_terminal_promotion() {
        let active_claim = claim(
            "claim_main",
            vec![rust_symbol("src/lib.rs", "module::alpha")],
            vec![],
        );
        let mut lease = QueueAwareLease::from_claim(&active_claim).expect("working claim");

        assert_eq!(
            lease.release_after_terminal_promotion(),
            Err(VcsKernelError::LeaseCannotReleaseBeforeTerminalPromotion)
        );

        lease.submit().expect("submitted");
        lease.mark_queue_stable("virtual/head/1").expect("stable");
        assert!(lease.allows_next_agent_with_virtual_predecessor("virtual/head/1"));
        lease.mark_virtual_merged().expect("virtual merged");
        lease.mark_merged_dev().expect("merged dev");
        lease.mark_promoted_staging().expect("staging");
        lease.mark_promoted_production().expect("prod");
        lease.release_after_terminal_promotion().expect("released");

        assert_eq!(lease.state, LeaseState::Released);
    }

    #[test]
    fn changeset_requires_claim_coverage_and_evidence() {
        let symbol = rust_symbol("src/lib.rs", "module::alpha");
        let active_claim = claim("claim_main", vec![symbol.clone()], vec![]);
        let lineage = ChangeSetLineage::new("wi_001", "ip_001", vec![]).unwrap();
        let changeset = ChangeSet::new(ChangeSetDraft {
            id: "cs_001".into(),
            agent_id: "codex-autopilot".into(),
            target_branch: "main".into(),
            base_sha: BASE_SHA.into(),
            branch_or_workspace_ref: "workspace/codex".into(),
            patch_id: "patch_001".into(),
            write_symbols: vec![symbol],
            read_symbols: vec![],
            touched_files: vec![ArtifactPointer::file("src/lib.rs").unwrap()],
            dependencies: vec![],
            lineage,
            evidence_refs: vec![".omc/evidence/gitops-vcs/ip-001-claim-kernel.json".into()],
        })
        .expect("valid changeset");

        required_claim_coverage(&changeset, &active_claim).expect("covered by claim");
    }

    #[test]
    fn pointer_claim_does_not_cover_whole_file_touched_artifact() {
        let pointer = ArtifactPointer::new(
            "contracts/vcs.yaml",
            ArtifactSelectorKind::OpenApiOperation,
            Some("POST /claim".into()),
        )
        .unwrap();
        let symbol = SymbolId::new(SymbolLanguage::OpenApi, pointer, "operation::claim").unwrap();
        let active_claim = claim("claim_main", vec![symbol.clone()], vec![]);
        let changeset = ChangeSet::new(ChangeSetDraft {
            id: "cs_001".into(),
            agent_id: "codex-autopilot".into(),
            target_branch: "main".into(),
            base_sha: BASE_SHA.into(),
            branch_or_workspace_ref: "workspace/codex".into(),
            patch_id: "patch_001".into(),
            write_symbols: vec![symbol],
            read_symbols: vec![],
            touched_files: vec![ArtifactPointer::file("contracts/vcs.yaml").unwrap()],
            dependencies: vec![],
            lineage: ChangeSetLineage::new("wi_001", "ip_001", vec![]).unwrap(),
            evidence_refs: vec![".omc/evidence/gitops-vcs/ip-001-claim-kernel.json".into()],
        })
        .expect("valid changeset shape");

        assert_eq!(
            required_claim_coverage(&changeset, &active_claim),
            Err(VcsKernelError::UnclaimedTouchedArtifact)
        );
    }

    #[test]
    fn changeset_rejects_unclaimed_touched_artifact() {
        let symbol = rust_symbol("src/lib.rs", "module::alpha");
        let active_claim = claim("claim_main", vec![symbol.clone()], vec![]);
        let changeset = ChangeSet::new(ChangeSetDraft {
            id: "cs_001".into(),
            agent_id: "codex-autopilot".into(),
            target_branch: "main".into(),
            base_sha: BASE_SHA.into(),
            branch_or_workspace_ref: "workspace/codex".into(),
            patch_id: "patch_001".into(),
            write_symbols: vec![symbol],
            read_symbols: vec![],
            touched_files: vec![ArtifactPointer::file("other.rs").unwrap()],
            dependencies: vec![],
            lineage: ChangeSetLineage::new("wi_001", "ip_001", vec![]).unwrap(),
            evidence_refs: vec![".omc/evidence/gitops-vcs/ip-001-claim-kernel.json".into()],
        })
        .expect("valid changeset shape");

        assert_eq!(
            required_claim_coverage(&changeset, &active_claim),
            Err(VcsKernelError::UnclaimedTouchedArtifact)
        );
    }

    #[test]
    fn changeset_rejects_non_sha_base() {
        let symbol = rust_symbol("src/lib.rs", "module::alpha");
        let error = ChangeSet::new(ChangeSetDraft {
            id: "cs_001".into(),
            agent_id: "codex-autopilot".into(),
            target_branch: "main".into(),
            base_sha: "main@base".into(),
            branch_or_workspace_ref: "workspace/codex".into(),
            patch_id: "patch_001".into(),
            write_symbols: vec![symbol],
            read_symbols: vec![],
            touched_files: vec![ArtifactPointer::file("src/lib.rs").unwrap()],
            dependencies: vec![],
            lineage: ChangeSetLineage::new("wi_001", "ip_001", vec![]).unwrap(),
            evidence_refs: vec![".omc/evidence/gitops-vcs/ip-001-claim-kernel.json".into()],
        })
        .expect_err("base_sha must be a digest-like hex value");

        assert_eq!(error, VcsKernelError::EmptyBaseRef);
    }

    #[test]
    fn virtual_head_is_projection_only_and_has_status() {
        let mut head = VirtualHead::new(
            "main",
            vec!["cs_001".into()],
            "main@base",
            "virtual/main/1",
            vec!["unit".into()],
        )
        .expect("valid virtual head");

        assert!(head.is_projection_only());
        assert_eq!(head.status, VirtualHeadStatus::Pending);
        head.invalidate_for("base advanced").unwrap();
        assert_eq!(head.status, VirtualHeadStatus::Invalidated);
    }
}
