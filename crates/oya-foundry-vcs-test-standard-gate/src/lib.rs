//! Test-standard admission gate for Oya VCS ChangeBundles.
//!
//! This application-layer crate models deterministic unit/integration/e2e
//! admission. It is std-only and provider-free: adapters/controllers supply the
//! semantic diff, suite registry rows, evidence records, and rebase generation;
//! this crate returns a CI/CD admission decision plus typed fixup tasks.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use oya_foundry_vcs_kernel::{ArtifactPointer, SymbolLanguage};

const TEST_STANDARD_VERSION: u32 = 1;
const DEFAULT_MAX_EVIDENCE_AGE_SECONDS: u64 = 86_400;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TestTier {
    Unit,
    Integration,
    Contract,
    E2e,
    Property,
    Fuzz,
}

impl TestTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unit => "unit",
            Self::Integration => "integration",
            Self::Contract => "contract",
            Self::E2e => "e2e",
            Self::Property => "property",
            Self::Fuzz => "fuzz",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockingRule {
    Always,
    ProductionIfUserFacing,
    Advisory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TierDefinition {
    pub tier: TestTier,                             // data_class: INTERNAL_ONLY
    pub purpose: &'static str,                      // data_class: INTERNAL_ONLY
    pub promotion_blocking: BlockingRule,           // data_class: INTERNAL_ONLY
    pub evidence_required: &'static [&'static str], // data_class: INTERNAL_ONLY
}

pub const TIER_DEFINITIONS: &[TierDefinition] = &[
    TierDefinition {
        tier: TestTier::Unit,
        purpose: "Fast proof for pure logic and component behavior.",
        promotion_blocking: BlockingRule::Always,
        evidence_required: &[
            "command",
            "runner",
            "changed symbols",
            "result",
            "freshness",
        ],
    },
    TierDefinition {
        tier: TestTier::Integration,
        purpose: "Boundary proof across ports/adapters/storage/contracts.",
        promotion_blocking: BlockingRule::Always,
        evidence_required: &[
            "command",
            "runner",
            "fixtures/dependencies",
            "artifact",
            "result",
            "freshness",
        ],
    },
    TierDefinition {
        tier: TestTier::Contract,
        purpose: "Provider/consumer schema parity for contracts.",
        promotion_blocking: BlockingRule::Always,
        evidence_required: &[
            "contract id",
            "provider",
            "consumer",
            "schema digest",
            "result",
        ],
    },
    TierDefinition {
        tier: TestTier::E2e,
        purpose: "Workflow-level proof across preview/deployed composition.",
        promotion_blocking: BlockingRule::ProductionIfUserFacing,
        evidence_required: &[
            "scenario",
            "environment",
            "platform",
            "artifact/ref",
            "result",
            "freshness",
        ],
    },
    TierDefinition {
        tier: TestTier::Property,
        purpose: "Invariant proof for parsers, codecs, and state machines.",
        promotion_blocking: BlockingRule::Always,
        evidence_required: &["case budget", "counterexample handling", "result"],
    },
    TierDefinition {
        tier: TestTier::Fuzz,
        purpose: "Boundary probing for unsafe, FFI, and untrusted parsers.",
        promotion_blocking: BlockingRule::Always,
        evidence_required: &["target", "duration/cases", "crash artifacts", "result"],
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SurfaceKind {
    Kernel,
    Adapter,
    ContractSchema,
    GeneratedClient,
    UiRoute,
    Workflow,
    Policy,
    Persistence,
    Config,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DeployEdge {
    None,
    Preview,
    Staging,
    Production,
    GitOpsPromotion,
}

impl DeployEdge {
    fn requires_e2e(self) -> bool {
        matches!(self, Self::Production | Self::GitOpsPromotion)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticChange {
    pub artifact: ArtifactPointer,            // data_class: INTERNAL_ONLY
    pub language: SymbolLanguage,             // data_class: INTERNAL_ONLY
    pub surface: SurfaceKind,                 // data_class: INTERNAL_ONLY
    pub package: String,                      // data_class: INTERNAL_ONLY
    pub symbol: String,                       // data_class: INTERNAL_ONLY
    pub deploy_edge: DeployEdge,              // data_class: INTERNAL_ONLY
    pub tenant_visible: bool,                 // data_class: INTERNAL_ONLY
    pub generated_client_for: Option<String>, // data_class: INTERNAL_ONLY
    pub contract_id: Option<String>,          // data_class: INTERNAL_ONLY
    pub touches_state_machine: bool,          // data_class: INTERNAL_ONLY
    pub touches_parser_or_serializer: bool,   // data_class: INTERNAL_ONLY
    pub touches_unsafe_or_ffi: bool,          // data_class: INTERNAL_ONLY
}

impl SemanticChange {
    pub fn new(
        artifact: ArtifactPointer,
        language: SymbolLanguage,
        surface: SurfaceKind,
        package: impl Into<String>,
        symbol: impl Into<String>,
    ) -> Result<Self, TestStandardError> {
        let package = normalize_non_empty(package.into(), TestStandardError::InvalidPackage)?;
        let symbol = normalize_non_empty(symbol.into(), TestStandardError::InvalidSymbol)?;
        Ok(Self {
            artifact,
            language,
            surface,
            package,
            symbol,
            deploy_edge: DeployEdge::None,
            tenant_visible: false,
            generated_client_for: None,
            contract_id: None,
            touches_state_machine: false,
            touches_parser_or_serializer: false,
            touches_unsafe_or_ffi: false,
        })
    }

    pub fn with_deploy_edge(mut self, deploy_edge: DeployEdge) -> Self {
        self.deploy_edge = deploy_edge;
        self
    }

    pub fn tenant_visible(mut self) -> Self {
        self.tenant_visible = true;
        self
    }

    pub fn generated_client_for(mut self, contract_id: impl Into<String>) -> Self {
        self.generated_client_for = Some(contract_id.into());
        self
    }

    pub fn contract(mut self, contract_id: impl Into<String>) -> Self {
        self.contract_id = Some(contract_id.into());
        self
    }

    pub fn state_machine(mut self) -> Self {
        self.touches_state_machine = true;
        self
    }

    pub fn parser_or_serializer(mut self) -> Self {
        self.touches_parser_or_serializer = true;
        self
    }

    pub fn unsafe_or_ffi(mut self) -> Self {
        self.touches_unsafe_or_ffi = true;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestSuiteRow {
    pub suite_id: String,                   // data_class: INTERNAL_ONLY
    pub language_or_surface: SuiteSelector, // data_class: INTERNAL_ONLY
    pub tiers: BTreeSet<TestTier>,          // data_class: INTERNAL_ONLY
    pub command: String,                    // data_class: INTERNAL_ONLY
    pub runner: String,                     // data_class: INTERNAL_ONLY
    pub required_when: Vec<String>,         // data_class: INTERNAL_ONLY
    pub promotion_blocking: BlockingRule,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SuiteSelector {
    Language(SymbolLanguage),
    Surface(SurfaceKind),
    Package(String),
}

impl TestSuiteRow {
    pub fn new(
        suite_id: impl Into<String>,
        language_or_surface: SuiteSelector,
        tiers: impl IntoIterator<Item = TestTier>,
        command: impl Into<String>,
        runner: impl Into<String>,
        promotion_blocking: BlockingRule,
    ) -> Result<Self, TestStandardError> {
        let suite_id = normalize_non_empty(suite_id.into(), TestStandardError::InvalidSuite)?;
        let command = normalize_non_empty(command.into(), TestStandardError::InvalidSuite)?;
        let runner = normalize_non_empty(runner.into(), TestStandardError::InvalidSuite)?;
        let tiers = tiers.into_iter().collect::<BTreeSet<_>>();
        if tiers.is_empty() {
            return Err(TestStandardError::InvalidSuite);
        }
        Ok(Self {
            suite_id,
            language_or_surface,
            tiers,
            command,
            runner,
            required_when: Vec::new(),
            promotion_blocking,
        })
    }

    fn matches(&self, change: &SemanticChange) -> bool {
        match &self.language_or_surface {
            SuiteSelector::Language(language) => *language == change.language,
            SuiteSelector::Surface(surface) => *surface == change.surface,
            SuiteSelector::Package(package) => package == &change.package,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestSuiteRegistry {
    pub rows: Vec<TestSuiteRow>, // data_class: INTERNAL_ONLY
}

impl TestSuiteRegistry {
    pub fn new(rows: Vec<TestSuiteRow>) -> Result<Self, TestStandardError> {
        if rows.is_empty() {
            return Err(TestStandardError::EmptyRegistry);
        }
        let mut ids = BTreeSet::new();
        for row in &rows {
            if !ids.insert(row.suite_id.clone()) {
                return Err(TestStandardError::DuplicateSuite(row.suite_id.clone()));
            }
        }
        Ok(Self { rows })
    }

    pub fn oyatie_default() -> Self {
        Self::new(vec![
            TestSuiteRow::new(
                "rust-nextest-workspace",
                SuiteSelector::Language(SymbolLanguage::Rust),
                [TestTier::Unit, TestTier::Integration, TestTier::Contract, TestTier::Property],
                "cargo nextest run --workspace --all-features --no-fail-fast",
                "cargo-nextest",
                BlockingRule::Always,
            )
            .expect("valid default registry row"),
            TestSuiteRow::new(
                "rust-fuzz",
                SuiteSelector::Language(SymbolLanguage::Rust),
                [TestTier::Fuzz],
                "cargo fuzz run <target>",
                "cargo-fuzz",
                BlockingRule::Always,
            )
            .expect("valid default registry row"),
            TestSuiteRow::new(
                "contract-parity",
                SuiteSelector::Surface(SurfaceKind::ContractSchema),
                [TestTier::Contract, TestTier::Integration],
                "cargo run -p oya-dev-cli -- gate validate openapi-rest-route-parity && cargo run -p oya-dev-cli -- gate validate api-semver",
                "oya-dev-cli",
                BlockingRule::Always,
            )
            .expect("valid default registry row"),
            TestSuiteRow::new(
                "generated-client-contract-parity",
                SuiteSelector::Surface(SurfaceKind::GeneratedClient),
                [TestTier::Contract, TestTier::Integration],
                "oya-dev-cli gate validate generated-client-contract-parity",
                "oya-dev-cli",
                BlockingRule::Always,
            )
            .expect("valid default registry row"),
            TestSuiteRow::new(
                "web-e2e",
                SuiteSelector::Surface(SurfaceKind::UiRoute),
                [TestTier::E2e],
                "playwright test (via owning package script)",
                "Playwright",
                BlockingRule::ProductionIfUserFacing,
            )
            .expect("valid default registry row"),
            TestSuiteRow::new(
                "workflow-e2e",
                SuiteSelector::Surface(SurfaceKind::Workflow),
                [TestTier::E2e],
                "workflow e2e smoke (owning package)",
                "workflow-runner",
                BlockingRule::ProductionIfUserFacing,
            )
            .expect("valid default registry row"),
        ])
        .expect("valid default registry")
    }

    fn select_for(
        &self,
        changes: &[SemanticChange],
        required: &BTreeSet<TestTier>,
    ) -> Vec<TestSuiteRow> {
        let mut selected = BTreeMap::<String, TestSuiteRow>::new();
        for change in changes {
            for row in &self.rows {
                if row.matches(change) && row.tiers.iter().any(|tier| required.contains(tier)) {
                    selected.insert(row.suite_id.clone(), row.clone());
                }
            }
        }
        selected.into_values().collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceDisposition {
    Required,
    Advisory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceResult {
    Pass,
    Fail,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceRecord {
    pub suite_id: String,                 // data_class: INTERNAL_ONLY
    pub tier: TestTier,                   // data_class: INTERNAL_ONLY
    pub disposition: EvidenceDisposition, // data_class: INTERNAL_ONLY
    pub result: EvidenceResult,           // data_class: INTERNAL_ONLY
    pub observed_rebase_generation: u64,  // data_class: INTERNAL_ONLY
    pub recorded_at_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
    pub command: String,                  // data_class: INTERNAL_ONLY
    pub artifact_ref: Option<String>,     // data_class: INTERNAL_ONLY
}

impl EvidenceRecord {
    pub fn required_pass(
        suite_id: impl Into<String>,
        tier: TestTier,
        observed_rebase_generation: u64,
        recorded_at_epoch_seconds: u64,
        command: impl Into<String>,
    ) -> Result<Self, TestStandardError> {
        Self::new(
            suite_id,
            tier,
            EvidenceDisposition::Required,
            EvidenceResult::Pass,
            observed_rebase_generation,
            recorded_at_epoch_seconds,
            command,
        )
    }

    pub fn new(
        suite_id: impl Into<String>,
        tier: TestTier,
        disposition: EvidenceDisposition,
        result: EvidenceResult,
        observed_rebase_generation: u64,
        recorded_at_epoch_seconds: u64,
        command: impl Into<String>,
    ) -> Result<Self, TestStandardError> {
        Ok(Self {
            suite_id: normalize_non_empty(suite_id.into(), TestStandardError::InvalidEvidence)?,
            tier,
            disposition,
            result,
            observed_rebase_generation,
            recorded_at_epoch_seconds,
            command: normalize_non_empty(command.into(), TestStandardError::InvalidEvidence)?,
            artifact_ref: None,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountingRecord {
    pub id: String,                       // data_class: INTERNAL_ONLY
    pub kind: AccountingKind,             // data_class: INTERNAL_ONLY
    pub source_artifact: ArtifactPointer, // data_class: INTERNAL_ONLY
    pub target_artifact: ArtifactPointer, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountingKind {
    GeneratedClient,
    ContractProviderConsumer,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreshnessPolicy {
    pub current_rebase_generation: u64, // data_class: INTERNAL_ONLY
    pub now_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
    pub max_age_seconds: u64,           // data_class: INTERNAL_ONLY
}

impl FreshnessPolicy {
    pub fn new(current_rebase_generation: u64, now_epoch_seconds: u64) -> Self {
        Self {
            current_rebase_generation,
            now_epoch_seconds,
            max_age_seconds: DEFAULT_MAX_EVIDENCE_AGE_SECONDS,
        }
    }

    fn stale_reason(&self, evidence: &EvidenceRecord) -> Option<String> {
        if evidence.observed_rebase_generation != self.current_rebase_generation {
            return Some(format!(
                "evidence observed rebase generation {}, current is {}",
                evidence.observed_rebase_generation, self.current_rebase_generation
            ));
        }
        if evidence.recorded_at_epoch_seconds > self.now_epoch_seconds {
            return Some("evidence timestamp is in the future".to_string());
        }
        let age = self.now_epoch_seconds - evidence.recorded_at_epoch_seconds;
        if age > self.max_age_seconds {
            return Some(format!(
                "evidence age {}s exceeds freshness limit {}s",
                age, self.max_age_seconds
            ));
        }
        None
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionInput {
    pub changes: Vec<SemanticChange>,      // data_class: INTERNAL_ONLY
    pub registry: TestSuiteRegistry,       // data_class: INTERNAL_ONLY
    pub evidence: Vec<EvidenceRecord>,     // data_class: INTERNAL_ONLY
    pub accounting: Vec<AccountingRecord>, // data_class: INTERNAL_ONLY
    pub freshness_policy: FreshnessPolicy, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionDecision {
    pub schema_version: u32,                    // data_class: INTERNAL_ONLY
    pub accepted: bool,                         // data_class: INTERNAL_ONLY
    pub required_tiers: BTreeSet<TestTier>,     // data_class: INTERNAL_ONLY
    pub required_suites: Vec<TestSuiteRow>,     // data_class: INTERNAL_ONLY
    pub advisory_evidence: Vec<EvidenceRecord>, // data_class: INTERNAL_ONLY
    pub fixup_tasks: Vec<FixupTask>,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixupTask {
    pub task_id: String,            // data_class: INTERNAL_ONLY
    pub reason: FixupReason,        // data_class: INTERNAL_ONLY
    pub tier: Option<TestTier>,     // data_class: INTERNAL_ONLY
    pub suite_id: Option<String>,   // data_class: INTERNAL_ONLY
    pub affected_refs: Vec<String>, // data_class: INTERNAL_ONLY
    pub blocking: bool,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FixupReason {
    MissingRequiredEvidence,
    FailingRequiredEvidence,
    StaleRequiredEvidence,
    AdvisoryEvidenceCannotSatisfyRequiredTier,
    MissingRegistrySuite,
    UnaccountedGeneratedClient,
    UnaccountedContract,
}

pub fn resolve_required_tiers(changes: &[SemanticChange]) -> BTreeSet<TestTier> {
    let mut required = BTreeSet::new();
    for change in changes {
        match change.surface {
            SurfaceKind::Kernel => {
                required.insert(TestTier::Unit);
            }
            SurfaceKind::Adapter | SurfaceKind::Persistence => {
                required.insert(TestTier::Unit);
                required.insert(TestTier::Integration);
            }
            SurfaceKind::ContractSchema => {
                required.insert(TestTier::Integration);
                required.insert(TestTier::Contract);
            }
            SurfaceKind::GeneratedClient => {
                required.insert(TestTier::Integration);
                required.insert(TestTier::Contract);
            }
            SurfaceKind::UiRoute | SurfaceKind::Workflow => {
                required.insert(TestTier::Unit);
                required.insert(TestTier::E2e);
            }
            SurfaceKind::Policy => {
                required.insert(TestTier::Unit);
                required.insert(TestTier::Contract);
            }
            SurfaceKind::Config => {
                required.insert(TestTier::Unit);
            }
        }
        if change.tenant_visible || change.deploy_edge.requires_e2e() {
            required.insert(TestTier::E2e);
        }
        if change.touches_state_machine || change.touches_parser_or_serializer {
            required.insert(TestTier::Property);
        }
        if change.touches_unsafe_or_ffi {
            required.insert(TestTier::Fuzz);
        }
    }
    required
}

pub fn evaluate_admission(input: AdmissionInput) -> AdmissionDecision {
    let required_tiers = resolve_required_tiers(&input.changes);
    let required_suites = input.registry.select_for(&input.changes, &required_tiers);
    let mut fixup_tasks = Vec::new();

    for tier in &required_tiers {
        if !required_suites
            .iter()
            .any(|suite| suite.tiers.contains(tier))
        {
            fixup_tasks.push(fixup(
                FixupReason::MissingRegistrySuite,
                Some(*tier),
                None,
                changed_refs(&input.changes),
                true,
            ));
        }
    }

    for suite in &required_suites {
        for tier in suite
            .tiers
            .iter()
            .filter(|tier| required_tiers.contains(tier))
        {
            let records = input
                .evidence
                .iter()
                .filter(|record| record.suite_id == suite.suite_id && record.tier == *tier)
                .collect::<Vec<_>>();
            if records.is_empty() {
                fixup_tasks.push(fixup(
                    FixupReason::MissingRequiredEvidence,
                    Some(*tier),
                    Some(suite.suite_id.clone()),
                    changed_refs(&input.changes),
                    true,
                ));
                continue;
            }
            let mut satisfied = false;
            for record in records {
                match (
                    record.disposition,
                    record.result,
                    input.freshness_policy.stale_reason(record),
                ) {
                    (EvidenceDisposition::Advisory, _, _) => fixup_tasks.push(fixup(
                        FixupReason::AdvisoryEvidenceCannotSatisfyRequiredTier,
                        Some(*tier),
                        Some(suite.suite_id.clone()),
                        vec![record.command.clone()],
                        true,
                    )),
                    (EvidenceDisposition::Required, EvidenceResult::Fail, _) => {
                        fixup_tasks.push(fixup(
                            FixupReason::FailingRequiredEvidence,
                            Some(*tier),
                            Some(suite.suite_id.clone()),
                            vec![record.command.clone()],
                            true,
                        ))
                    }
                    (EvidenceDisposition::Required, EvidenceResult::Pass, Some(reason)) => {
                        fixup_tasks.push(fixup(
                            FixupReason::StaleRequiredEvidence,
                            Some(*tier),
                            Some(suite.suite_id.clone()),
                            vec![reason],
                            true,
                        ));
                    }
                    (EvidenceDisposition::Required, EvidenceResult::Pass, None) => {
                        satisfied = true;
                    }
                }
            }
            if !satisfied {
                fixup_tasks.push(fixup(
                    FixupReason::MissingRequiredEvidence,
                    Some(*tier),
                    Some(suite.suite_id.clone()),
                    changed_refs(&input.changes),
                    true,
                ));
            }
        }
    }

    check_accounting(&input, &mut fixup_tasks);

    AdmissionDecision {
        schema_version: TEST_STANDARD_VERSION,
        accepted: fixup_tasks.iter().all(|task| !task.blocking),
        required_tiers,
        required_suites,
        advisory_evidence: input
            .evidence
            .into_iter()
            .filter(|record| record.disposition == EvidenceDisposition::Advisory)
            .collect(),
        fixup_tasks: renumber_fixups(fixup_tasks),
    }
}

fn check_accounting(input: &AdmissionInput, fixup_tasks: &mut Vec<FixupTask>) {
    for change in &input.changes {
        if let Some(contract_id) = &change.generated_client_for {
            let accounted = input.accounting.iter().any(|record| {
                record.kind == AccountingKind::GeneratedClient
                    && record.id == *contract_id
                    && record.target_artifact == change.artifact
            });
            if !accounted {
                fixup_tasks.push(fixup(
                    FixupReason::UnaccountedGeneratedClient,
                    Some(TestTier::Contract),
                    None,
                    vec![change_ref(change), contract_id.clone()],
                    true,
                ));
            }
        }
        if let Some(contract_id) = &change.contract_id {
            let accounted = input.accounting.iter().any(|record| {
                record.kind == AccountingKind::ContractProviderConsumer
                    && record.id == *contract_id
                    && (record.source_artifact == change.artifact
                        || record.target_artifact == change.artifact)
            });
            if !accounted {
                fixup_tasks.push(fixup(
                    FixupReason::UnaccountedContract,
                    Some(TestTier::Contract),
                    None,
                    vec![change_ref(change), contract_id.clone()],
                    true,
                ));
            }
        }
    }
}

fn fixup(
    reason: FixupReason,
    tier: Option<TestTier>,
    suite_id: Option<String>,
    affected_refs: Vec<String>,
    blocking: bool,
) -> FixupTask {
    FixupTask {
        task_id: String::new(),
        reason,
        tier,
        suite_id,
        affected_refs,
        blocking,
    }
}

fn renumber_fixups(tasks: Vec<FixupTask>) -> Vec<FixupTask> {
    tasks
        .into_iter()
        .enumerate()
        .map(|(index, mut task)| {
            task.task_id = format!("fixup-test-standard-{:04}", index + 1);
            task
        })
        .collect()
}

fn changed_refs(changes: &[SemanticChange]) -> Vec<String> {
    changes.iter().map(change_ref).collect()
}

fn change_ref(change: &SemanticChange) -> String {
    format!(
        "{}::{}::{}",
        change.package, change.artifact.path, change.symbol
    )
}

fn normalize_non_empty(
    value: String,
    error: TestStandardError,
) -> Result<String, TestStandardError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.contains('\0') {
        return Err(error);
    }
    Ok(trimmed.to_string())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TestStandardError {
    EmptyRegistry,
    DuplicateSuite(String),
    InvalidSuite,
    InvalidPackage,
    InvalidSymbol,
    InvalidEvidence,
}

impl fmt::Display for TestStandardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRegistry => write!(formatter, "test suite registry must not be empty"),
            Self::DuplicateSuite(suite_id) => {
                write!(formatter, "duplicate test suite id: {suite_id}")
            }
            Self::InvalidSuite => write!(formatter, "invalid test suite row"),
            Self::InvalidPackage => write!(formatter, "invalid package"),
            Self::InvalidSymbol => write!(formatter, "invalid symbol"),
            Self::InvalidEvidence => write!(formatter, "invalid evidence"),
        }
    }
}

impl std::error::Error for TestStandardError {}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_foundry_vcs_kernel::ArtifactPointer;

    fn artifact(path: &str) -> ArtifactPointer {
        ArtifactPointer::file(path).expect("valid artifact")
    }

    fn rust_change(surface: SurfaceKind) -> SemanticChange {
        SemanticChange::new(
            artifact("crates/demo/src/lib.rs"),
            SymbolLanguage::Rust,
            surface,
            "demo",
            "demo::symbol",
        )
        .expect("valid change")
    }

    fn policy() -> FreshnessPolicy {
        FreshnessPolicy::new(7, 1_000)
    }

    fn evidence(suite: &str, tier: TestTier) -> EvidenceRecord {
        EvidenceRecord::required_pass(
            suite,
            tier,
            7,
            999,
            format!("run {suite} {}", tier.as_str()),
        )
        .expect("valid evidence")
    }

    fn input(changes: Vec<SemanticChange>, evidence: Vec<EvidenceRecord>) -> AdmissionInput {
        AdmissionInput {
            changes,
            registry: TestSuiteRegistry::oyatie_default(),
            evidence,
            accounting: Vec::new(),
            freshness_policy: policy(),
        }
    }

    #[test]
    fn diff_to_required_tier_resolver_is_semantic() {
        let changes = vec![
            rust_change(SurfaceKind::Kernel).state_machine(),
            rust_change(SurfaceKind::Adapter),
            rust_change(SurfaceKind::Workflow).tenant_visible(),
        ];

        let required = resolve_required_tiers(&changes);

        assert!(required.contains(&TestTier::Unit));
        assert!(required.contains(&TestTier::Integration));
        assert!(required.contains(&TestTier::Property));
        assert!(required.contains(&TestTier::E2e));
        assert!(!required.contains(&TestTier::Fuzz));
    }

    #[test]
    fn registry_driven_selection_uses_surface_language_and_package() {
        let registry = TestSuiteRegistry::new(vec![
            TestSuiteRow::new(
                "rust-unit",
                SuiteSelector::Language(SymbolLanguage::Rust),
                [TestTier::Unit],
                "cargo test -p demo",
                "cargo",
                BlockingRule::Always,
            )
            .unwrap(),
            TestSuiteRow::new(
                "package-integration",
                SuiteSelector::Package("demo".to_string()),
                [TestTier::Integration],
                "demo integration",
                "custom",
                BlockingRule::Always,
            )
            .unwrap(),
            TestSuiteRow::new(
                "workflow-e2e",
                SuiteSelector::Surface(SurfaceKind::Workflow),
                [TestTier::E2e],
                "workflow e2e",
                "custom",
                BlockingRule::ProductionIfUserFacing,
            )
            .unwrap(),
        ])
        .unwrap();
        let changes = vec![rust_change(SurfaceKind::Workflow).tenant_visible()];
        let required = resolve_required_tiers(&changes);

        let suites = registry.select_for(&changes, &required);
        let ids = suites
            .iter()
            .map(|suite| suite.suite_id.as_str())
            .collect::<BTreeSet<_>>();

        assert_eq!(ids, BTreeSet::from(["rust-unit", "workflow-e2e"]));
    }

    #[test]
    fn failing_required_integration_and_e2e_block_and_emit_fixup_tasks() {
        let mut failed_integration = EvidenceRecord::new(
            "rust-nextest-workspace",
            TestTier::Integration,
            EvidenceDisposition::Required,
            EvidenceResult::Fail,
            7,
            999,
            "cargo nextest run --workspace --all-features --no-fail-fast",
        )
        .unwrap();
        failed_integration.artifact_ref = Some("target/nextest/junit.xml".into());
        let failed_e2e = EvidenceRecord::new(
            "workflow-e2e",
            TestTier::E2e,
            EvidenceDisposition::Required,
            EvidenceResult::Fail,
            7,
            999,
            "workflow e2e smoke",
        )
        .unwrap();

        let decision = evaluate_admission(input(
            vec![
                rust_change(SurfaceKind::Adapter),
                rust_change(SurfaceKind::Workflow).tenant_visible(),
            ],
            vec![
                evidence("rust-nextest-workspace", TestTier::Unit),
                failed_integration,
                failed_e2e,
            ],
        ));

        assert!(!decision.accepted);
        assert!(decision.fixup_tasks.iter().any(|task| {
            task.reason == FixupReason::FailingRequiredEvidence
                && task.tier == Some(TestTier::Integration)
                && task.blocking
        }));
        assert!(decision.fixup_tasks.iter().any(|task| {
            task.reason == FixupReason::FailingRequiredEvidence
                && task.tier == Some(TestTier::E2e)
                && task.blocking
        }));
    }

    #[test]
    fn stale_evidence_blocks_promotion() {
        let stale = EvidenceRecord::required_pass(
            "rust-nextest-workspace",
            TestTier::Unit,
            6,
            999,
            "cargo nextest run --workspace --all-features --no-fail-fast",
        )
        .unwrap();

        let decision =
            evaluate_admission(input(vec![rust_change(SurfaceKind::Kernel)], vec![stale]));

        assert!(!decision.accepted);
        assert!(
            decision
                .fixup_tasks
                .iter()
                .any(|task| task.reason == FixupReason::StaleRequiredEvidence)
        );
    }

    #[test]
    fn advisory_only_evidence_cannot_satisfy_required_tier() {
        let advisory = EvidenceRecord::new(
            "rust-nextest-workspace",
            TestTier::Unit,
            EvidenceDisposition::Advisory,
            EvidenceResult::Pass,
            7,
            999,
            "cargo test -p demo",
        )
        .unwrap();

        let decision = evaluate_admission(input(
            vec![rust_change(SurfaceKind::Kernel)],
            vec![advisory],
        ));

        assert!(!decision.accepted);
        assert!(decision.fixup_tasks.iter().any(|task| {
            task.reason == FixupReason::AdvisoryEvidenceCannotSatisfyRequiredTier && task.blocking
        }));
    }

    #[test]
    fn unaccounted_generated_client_or_contract_blocks() {
        let generated =
            rust_change(SurfaceKind::GeneratedClient).generated_client_for("contract:v1");
        let contract = rust_change(SurfaceKind::ContractSchema).contract("contract:v2");
        let decision = evaluate_admission(input(
            vec![generated, contract],
            vec![
                evidence("rust-nextest-workspace", TestTier::Integration),
                evidence("rust-nextest-workspace", TestTier::Contract),
                evidence("contract-parity", TestTier::Integration),
                evidence("contract-parity", TestTier::Contract),
                evidence("generated-client-contract-parity", TestTier::Integration),
                evidence("generated-client-contract-parity", TestTier::Contract),
            ],
        ));

        assert!(!decision.accepted);
        assert!(
            decision
                .fixup_tasks
                .iter()
                .any(|task| task.reason == FixupReason::UnaccountedGeneratedClient)
        );
        assert!(
            decision
                .fixup_tasks
                .iter()
                .any(|task| task.reason == FixupReason::UnaccountedContract)
        );
    }

    #[test]
    fn fresh_required_evidence_with_accounting_admits_bundle() {
        let source = artifact("contracts/openapi/demo.yaml");
        let target = artifact("crates/demo-client/src/generated.rs");
        let generated = SemanticChange::new(
            target.clone(),
            SymbolLanguage::Rust,
            SurfaceKind::GeneratedClient,
            "demo-client",
            "demo_client::generated",
        )
        .unwrap()
        .generated_client_for("contract:v1");
        let mut admission = input(
            vec![generated],
            vec![
                evidence("rust-nextest-workspace", TestTier::Integration),
                evidence("rust-nextest-workspace", TestTier::Contract),
                evidence("generated-client-contract-parity", TestTier::Integration),
                evidence("generated-client-contract-parity", TestTier::Contract),
            ],
        );
        admission.accounting.push(AccountingRecord {
            id: "contract:v1".into(),
            kind: AccountingKind::GeneratedClient,
            source_artifact: source,
            target_artifact: target,
        });

        let decision = evaluate_admission(admission);

        assert!(decision.accepted, "fixups: {:?}", decision.fixup_tasks);
        assert!(decision.fixup_tasks.is_empty());
    }

    #[test]
    fn accounting_must_match_changed_artifact() {
        let generated_artifact = artifact("crates/demo-client/src/generated.rs");
        let contract_artifact = artifact("contracts/openapi/demo.yaml");
        let generated = SemanticChange::new(
            generated_artifact.clone(),
            SymbolLanguage::Rust,
            SurfaceKind::GeneratedClient,
            "demo-client",
            "demo_client::generated",
        )
        .unwrap()
        .generated_client_for("contract:v1");
        let contract = SemanticChange::new(
            contract_artifact.clone(),
            SymbolLanguage::OpenApi,
            SurfaceKind::ContractSchema,
            "demo-api",
            "POST /demo",
        )
        .unwrap()
        .contract("contract:v2");
        let mut admission = input(
            vec![generated, contract],
            vec![
                evidence("rust-nextest-workspace", TestTier::Integration),
                evidence("rust-nextest-workspace", TestTier::Contract),
                evidence("generated-client-contract-parity", TestTier::Integration),
                evidence("generated-client-contract-parity", TestTier::Contract),
                evidence("contract-parity", TestTier::Integration),
                evidence("contract-parity", TestTier::Contract),
            ],
        );
        admission.accounting.push(AccountingRecord {
            id: "contract:v1".into(),
            kind: AccountingKind::GeneratedClient,
            source_artifact: artifact("contracts/openapi/demo.yaml"),
            target_artifact: artifact("crates/other-client/src/generated.rs"),
        });
        admission.accounting.push(AccountingRecord {
            id: "contract:v2".into(),
            kind: AccountingKind::ContractProviderConsumer,
            source_artifact: artifact("contracts/openapi/other.yaml"),
            target_artifact: artifact("crates/demo-api/src/routes.rs"),
        });

        let decision = evaluate_admission(admission);

        assert!(!decision.accepted);
        assert!(
            decision
                .fixup_tasks
                .iter()
                .any(|task| task.reason == FixupReason::UnaccountedGeneratedClient)
        );
        assert!(
            decision
                .fixup_tasks
                .iter()
                .any(|task| task.reason == FixupReason::UnaccountedContract)
        );
    }
}
