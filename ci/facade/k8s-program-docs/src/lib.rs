//! Deterministic R-DOC checks for the Kubernetes port program (ADR-0637 and ADR-0638).
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

const PROGRAM_ROOT: &str = "docs/programs/k8s-port";
/// Required line-oriented registry: `version=1`, then
/// `wave=<id>;ordinal=<u32>;completed=<true|false>;operations_entries=<operations-relative paths>;no_extraction_rationale=<text>`.
const WAVE_REGISTRY: &str = "docs/programs/k8s-port/wave-registry.rdoc";
const SIX_AXIS_TOKENS: [&str; 6] = [
    "`pin`",
    "snapshot_digest",
    "engine_digest",
    "rulepack_digest",
    "toolchain_digest",
    "formatter_digest",
];

/// Root scanned for INV-3. Every Rust file below it is read; nothing below it is written.
const OS_ROOT: &str = "os";
/// INV-3, frozen on `origin/dev` @ `5e452bd70`: 15 production sites plus one test fixture at
/// `os/core/k8s-control-domain/src/manifest_controller.rs:200`. Frozen at equality — a unit that
/// retires a site re-freezes this constant in the same commit, and no unit raises it. A bare `<=`
/// would let a retirement bank silent headroom for the site to come back, so the gate reds on any
/// drift in either direction. Hand-writing upstream Kubernetes wire surface is what ADR-0704
/// charters as generated, so growth here is the defect and an unrecorded shrink is a lie.
pub const UPSTREAM_EMIT_SITE_CEILING: usize = 16;
/// INV-3 emit-site identities (`relpath\tapiVersion`) pinned by multiset equality.
/// A swap that keeps the count at 16 still changes this list.
const FROZEN_UPSTREAM_EMIT_IDENTITIES: &[&str] = &[
    "os/core/k8s-control-domain/src/admission.rs\tapiserver.config.k8s.io/v1",
    "os/core/k8s-control-domain/src/admission.rs\tapiserver.config.k8s.io/v1",
    "os/core/k8s-control-domain/src/admission.rs\taudit.k8s.io/v1",
    "os/core/k8s-control-domain/src/admission.rs\tpod-security.admission.config.k8s.io/v1",
    "os/core/k8s-control-domain/src/manifest_controller.rs\tv1",
    "os/core/k8s-control-domain/src/static_pod_controller.rs\tv1",
    "os/core/kubelet-domain/src/spec.rs\tkubelet.config.k8s.io/v1beta1",
    "os/core/kubernetes-domain/src/kubeconfig.rs\tv1",
    "os/core/kubernetes-domain/src/static_pod.rs\tv1",
    "os/core/kubernetes-domain/src/templates.rs\tapps/v1",
    "os/core/kubernetes-domain/src/templates.rs\tapps/v1",
    "os/core/kubernetes-domain/src/templates.rs\trbac.authorization.k8s.io/v1",
    "os/core/kubernetes-domain/src/templates.rs\trbac.authorization.k8s.io/v1",
    "os/core/kubernetes-domain/src/templates.rs\tv1",
    "os/core/kubernetes-domain/src/templates.rs\tv1",
    "os/core/kubernetes-domain/src/templates.rs\tv1",
];
/// W0 divergence row identities pinned by equality so a constant cumulative size
/// cannot hide a swap. Retirements and additions update this pin in the same commit.
const FROZEN_DIVERGENCE_ROW_IDS: &[&str] = &[
    "DVG-BOOTSTRAP-GO-FRONTEND",
    "DVG-CEDAR-AUTHORIZATION-SEAM",
    "DVG-OS-DUPLICATE-STATIC-POD-RENDERER",
    "DVG-OS-HANDROLLED-K8S-API-SERIALIZERS",
    "DVG-OWNED-AUDIT-CHAIN-EMISSION",
    "DVG-OWNED-OBSERVABILITY-EMISSION",
    "DVG-REMOVED-IN-TREE-PROVIDERS-VOLUME-PLUGINS",
];

/// The five upstream Kubernetes API groups that carry no `.k8s.io` suffix. Every other upstream
/// group is recognised structurally by that suffix rather than by enumeration.
const SUFFIXLESS_UPSTREAM_GROUPS: [&str; 5] =
    ["apps", "batch", "autoscaling", "policy", "extensions"];

/// INV-5 denominator source: the leaf classification every tracked crate leaf across the seam is
/// checked against. Read, never trusted — the counts in it are recomputed from the tree.
const REGENERABLE_REGIONS: &str = "specs/k8s-port/regenerable-regions.json";
/// The two capabilities the port seam runs between; the leaf census claims nothing outside them.
const SEAM_CAPABILITIES: [&str; 2] = ["k8s", "os"];
/// The divergence ledger whose `growth_policy` knobs this gate is the reader for. Before this
/// existed, `baseline_seed_count` and `max_new_rows_per_wave` each had exactly one occurrence in
/// the tree — their own definition — while the seam mapping asserted in bold that a third row
/// "is rejected by the ledger's own growth policy". Nothing evaluated it.
const DIVERGENCE_LEDGER: &str = "specs/k8s-port/divergence-ledger.json";

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum FindingCode {
    EmptyProgramCorpus,
    MissingBaselineHeader,
    CompletedWaveWithoutJournal,
    RuleWithoutJournalReference,
    DoctrineWithoutAdr,
    PrescriptionStarvation,
    UpstreamKubernetesSurfaceGrowth,
    UnclassifiableDynamicApiVersion,
    UnclassifiedCrateLeaf,
    DivergenceLedgerGrowthBudgetExceeded,
    CrossSeamDependency,
}

impl FindingCode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::EmptyProgramCorpus => "R-DOC-EMPTY-PROGRAM-CORPUS",
            Self::MissingBaselineHeader => "R-DOC-BASELINE-HEADER-MISSING",
            Self::CompletedWaveWithoutJournal => "R-DOC-COMPLETED-WAVE-JOURNAL-MISSING",
            Self::RuleWithoutJournalReference => "R-DOC-RULE-JOURNAL-REFERENCE-MISSING",
            Self::DoctrineWithoutAdr => "R-DOC-DOCTRINE-ADR-OVERDUE",
            Self::PrescriptionStarvation => "R-DOC-PRESCRIPTION-STARVATION",
            Self::UpstreamKubernetesSurfaceGrowth => "R-DOC-OS-UPSTREAM-K8S-SURFACE-GROWTH",
            Self::UnclassifiableDynamicApiVersion => "R-DOC-OS-DYNAMIC-APIVERSION-UNCLASSIFIABLE",
            Self::UnclassifiedCrateLeaf => "R-DOC-K8S-PORT-LEAF-UNCLASSIFIED",
            Self::DivergenceLedgerGrowthBudgetExceeded => {
                "R-DOC-K8S-PORT-DIVERGENCE-GROWTH-BUDGET-EXCEEDED"
            }
            Self::CrossSeamDependency => "R-DOC-CROSS-SEAM-DEPENDENCY",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    pub code: FindingCode,
    pub path: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Counters {
    pub scanned_population: usize,
    pub finding_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evaluation {
    pub counters: Counters,
    pub findings: Vec<Finding>,
}

impl Evaluation {
    pub fn is_green(&self) -> bool {
        self.findings.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkdownDocument {
    pub path: String,
    pub contents: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JournalEntry {
    pub id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletedWave {
    pub id: String,
    pub ordinal: u32,
    pub journal_entries: Vec<JournalEntry>,
    pub no_extraction_rationale: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuleKind {
    Neutral,
    Corpus,
}

impl RuleKind {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Corpus => "corpus",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleRecord {
    pub path: String,
    pub id: String,
    pub kind: RuleKind,
    pub operations_journal_reference: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoctrineRecord {
    pub path: String,
    pub id: String,
    pub authored_wave: u32,
    pub binding_outside_lane: bool,
    pub adr_reference: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Corpus {
    pub documents: Vec<MarkdownDocument>,
    pub completed_waves: Vec<CompletedWave>,
    pub rules: Vec<RuleRecord>,
    pub doctrine: Vec<DoctrineRecord>,
    pub prescription_count: usize,
    /// INV-3 observed value: upstream-Kubernetes `apiVersion:` emit sites across `os/**/*.rs`.
    pub upstream_emit_sites: usize,
    /// Multiset of `relpath\tapiVersion` for each INV-3 emit line (sorted).
    pub upstream_emit_identities: Vec<String>,
    /// Lines across the same scan whose `apiVersion` value is BUILT AT RUNTIME. The value is not in
    /// the source, so no grammar-aware read can classify it either — the ratchet fails closed on it
    /// rather than counting it as absent, which is the exact hole the equality freeze exists to
    /// close: a new hand-written serializer landing while the census stays at 16 and the gate passes.
    pub dynamic_api_version_sites: usize,
    /// Denominator for the line above. A zero here means the scan found nothing to read, which is a
    /// false green, so the loader refuses it rather than reporting `0 <= 16`.
    pub os_rust_files: usize,
    /// Every crate leaf `<capability>/<face>/<crate>` under `k8s/` and `os/`, enumerated from the
    /// tree. This is the denominator the declared leaf census is checked against; the loader
    /// refuses an empty enumeration for the same reason it refuses a zero `os_rust_files`.
    pub crate_leaves: Vec<String>,
    /// What `specs/k8s-port/regenerable-regions.json` declares about the line above. Parsed from
    /// the file so the equality is computed rather than read back from its own literal.
    pub declared_leaves: DeclaredLeaves,
    /// The divergence ledger's own growth knobs plus its observed row count.
    pub divergence_ledger: DivergenceLedger,
    /// INV-1 / INV-2: forbidden package/target edges parsed from
    /// `os|k8s/**/{Cargo.toml,BUCK,BUCK.v2}` (inline tables including quoted keys, named
    /// dependency tables, `.workspace = true`, `package =` renames, and Buck `//os/`↔`//k8s/`
    /// deps). `BUCK.v2` is scanned because buck2's buildfile precedence lets it shadow `BUCK`.
    pub cross_seam_edges: Vec<CrossSeamEdge>,
}

/// One forbidden dependency edge across the os↔k8s seam.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossSeamEdge {
    pub path: String,
    pub dependency: String,
}

/// `specs/k8s-port/divergence-ledger.json` read as DATA: both budget knobs come from the file, so
/// what the declaration says is what the gate enforces. Hardcoding the budget would leave the knobs
/// inert, which is the defect this reader exists to fix.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DivergenceLedger {
    pub rows: usize,
    pub baseline_seed_count: usize,
    pub max_new_rows_per_wave: usize,
    /// Stable row identities. Cumulative size alone cannot see a swap (delete N, add N+k).
    pub row_ids: Vec<String>,
}

/// The leaf census `specs/k8s-port/regenerable-regions.json` declares.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DeclaredLeaves {
    /// Every `origin_classification.leaves[].path` in declaration order. Duplicates are preserved
    /// rather than collapsed, so a repeated row surfaces as a row-count mismatch instead of hiding
    /// behind a per-leaf presence check that a duplicate would still satisfy.
    pub rows: Vec<String>,
    pub k8s_leaves: usize,
    pub os_leaves: usize,
    pub total_leaves: usize,
}

pub fn evaluate(corpus: &Corpus) -> Evaluation {
    let mut findings = Vec::new();
    if corpus.documents.is_empty() {
        findings.push(finding(
            FindingCode::EmptyProgramCorpus,
            PROGRAM_ROOT,
            "no program Markdown documents were scanned",
        ));
    }

    for document in &corpus.documents {
        if !has_baseline_header(&document.contents) {
            findings.push(finding(
                FindingCode::MissingBaselineHeader,
                &document.path,
                "required Baseline version header lacks a repository baseline, upstream baseline, or six-axis tuple token",
            ));
        }
    }

    for wave in &corpus.completed_waves {
        if wave.journal_entries.is_empty() {
            findings.push(finding(
                FindingCode::CompletedWaveWithoutJournal,
                &wave.id,
                "completed wave has no non-empty operations-journal entry",
            ));
        }
    }

    for rule in &corpus.rules {
        if !non_empty(rule.operations_journal_reference.as_deref()) {
            findings.push(finding(
                FindingCode::RuleWithoutJournalReference,
                &rule.path,
                &format!(
                    "{} rule '{}' has no operations-journal reference",
                    rule.kind.as_str(),
                    rule.id
                ),
            ));
        }
    }

    let newest_completed_wave = corpus.completed_waves.iter().map(|wave| wave.ordinal).max();
    if let Some(newest_completed_wave) = newest_completed_wave {
        for doctrine in &corpus.doctrine {
            if doctrine.binding_outside_lane
                && newest_completed_wave.saturating_sub(doctrine.authored_wave) > 1
                && !non_empty(doctrine.adr_reference.as_deref())
            {
                findings.push(finding(
                    FindingCode::DoctrineWithoutAdr,
                    &doctrine.path,
                    &format!(
                        "doctrine '{}' is older than one completed wave without an ADR reference",
                        doctrine.id
                    ),
                ));
            }
        }
    }

    if corpus.prescription_count == 0 {
        let mut waves = corpus.completed_waves.iter().collect::<Vec<_>>();
        waves.sort_by_key(|wave| (wave.ordinal, &wave.id));
        for pair in waves.windows(2) {
            let earlier = pair[0];
            let later = pair[1];
            if !earlier.journal_entries.is_empty()
                && !later.journal_entries.is_empty()
                && !non_empty(later.no_extraction_rationale.as_deref())
            {
                findings.push(finding(
                    FindingCode::PrescriptionStarvation,
                    &later.id,
                    &format!(
                        "operations journal grew at completed gates '{}' and '{}' while prescriptions remain empty",
                        earlier.id, later.id
                    ),
                ));
            }
        }
    }

    if corpus.upstream_emit_sites != UPSTREAM_EMIT_SITE_CEILING {
        findings.push(finding(
            FindingCode::UpstreamKubernetesSurfaceGrowth,
            OS_ROOT,
            &format!(
                "{} upstream-Kubernetes apiVersion emit sites across {} Rust files does not equal the frozen census of {}; the surface is chartered as generated, so growth means consume the seam instead of hand-writing it, and a retirement must lower this constant in the same commit",
                corpus.upstream_emit_sites, corpus.os_rust_files, UPSTREAM_EMIT_SITE_CEILING
            ),
        ));
    }
    let frozen_identities: Vec<&str> = FROZEN_UPSTREAM_EMIT_IDENTITIES.to_vec();
    let observed_identities: Vec<&str> = corpus
        .upstream_emit_identities
        .iter()
        .map(String::as_str)
        .collect();
    if observed_identities != frozen_identities {
        findings.push(finding(
            FindingCode::UpstreamKubernetesSurfaceGrowth,
            OS_ROOT,
            &format!(
                "upstream emit-site identity multiset does not equal the frozen pin (observed {:?}, frozen {:?}); a swap that preserves the count of {} still changes membership — update the pin in the same commit",
                observed_identities, frozen_identities, UPSTREAM_EMIT_SITE_CEILING
            ),
        ));
    }

    if corpus.dynamic_api_version_sites != 0 {
        findings.push(finding(
            FindingCode::UnclassifiableDynamicApiVersion,
            OS_ROOT,
            &format!(
                "{} line(s) build an apiVersion value at runtime (a format placeholder), so the emitted API group is not in the source and cannot be classified as upstream or Talos by any read of it; make the value static or consume it through the k8s/ports seam, because an unclassifiable site would otherwise pass the frozen census of {} while hand-writing upstream wire surface",
                corpus.dynamic_api_version_sites, UPSTREAM_EMIT_SITE_CEILING
            ),
        ));
    }

    for edge in &corpus.cross_seam_edges {
        findings.push(finding(
            FindingCode::CrossSeamDependency,
            &edge.path,
            &format!(
                "manifest declares forbidden cross-seam dependency `{}`; INV-1/INV-2 forbid os↔k8s Cargo/Buck edges in any TOML spelling (inline `{{ path }}`, quoted keys, named `[*.dependencies.<pkg>]` tables, `.workspace = true`, `package =` renames) or Buck `//os/`↔`//k8s/` target edges in `BUCK`/`BUCK.v2`",
                edge.dependency
            ),
        ));
    }

    // Both knobs are read from the ledger, so the declaration is what is enforced.
    // CEILING: `rows` is CUMULATIVE while `max_new_rows_per_wave` is a per-wave DELTA, so this
    // comparison is exact only while exactly one wave has elapsed (W0). Re-derive it at the next
    // wave gate — after W1 the budget needs the per-wave delta, not the cumulative total.
    let ledger = &corpus.divergence_ledger;
    let budget = ledger.baseline_seed_count + ledger.max_new_rows_per_wave;
    if ledger.rows > budget {
        findings.push(finding(
            FindingCode::DivergenceLedgerGrowthBudgetExceeded,
            DIVERGENCE_LEDGER,
            &format!(
                "{} divergence rows exceed the ledger's own declared budget of {} (baseline_seed_count {} + max_new_rows_per_wave {}); the growth policy is DATA in this file and this is its reader — a further divergence is recorded in the operations journal and waits for the next wave gate",
                ledger.rows, budget, ledger.baseline_seed_count, ledger.max_new_rows_per_wave
            ),
        ));
    }
    let observed_ids: BTreeSet<&str> = ledger.row_ids.iter().map(String::as_str).collect();
    let frozen_ids: BTreeSet<&str> = FROZEN_DIVERGENCE_ROW_IDS.iter().copied().collect();
    if observed_ids != frozen_ids {
        findings.push(finding(
            FindingCode::DivergenceLedgerGrowthBudgetExceeded,
            DIVERGENCE_LEDGER,
            &format!(
                "divergence row identity set does not equal the frozen W0 pin (observed {:?}, frozen {:?}); a swap that preserves cumulative size still changes membership — update the pin in the same commit as any retirement or addition",
                observed_ids, frozen_ids
            ),
        ));
    }

    // The leaf census is a denominator, so it is checked against the tree rather than against
    // itself. `crate_leaves` comes from `read_dir`; every number below comes from the declaration.
    let declared = &corpus.declared_leaves;
    let declared_paths = declared.rows.iter().collect::<BTreeSet<_>>();
    for leaf in &corpus.crate_leaves {
        if !declared_paths.contains(leaf) {
            findings.push(finding(
                FindingCode::UnclassifiedCrateLeaf,
                leaf,
                "tracked crate leaf carries no origin_classification.leaves row; an unclassified leaf is the hole that makes a leaf-by-leaf partition report a clean result over an unseen corpus",
            ));
        }
    }
    // Presence alone cannot see a deletion or a rename: the stale row it leaves behind still
    // satisfies every remaining leaf. The row count is what catches that direction.
    if declared.rows.len() != corpus.crate_leaves.len() {
        findings.push(finding(
            FindingCode::UnclassifiedCrateLeaf,
            REGENERABLE_REGIONS,
            &format!(
                "{} declared leaf rows against {} crate leaves enumerated from the tree; a deleted or renamed leaf leaves a stale row that a per-leaf presence check alone cannot see",
                declared.rows.len(),
                corpus.crate_leaves.len()
            ),
        ));
    }
    let k8s_leaves = corpus
        .crate_leaves
        .iter()
        .filter(|leaf| leaf.starts_with("k8s/"))
        .count();
    let os_leaves = corpus
        .crate_leaves
        .iter()
        .filter(|leaf| leaf.starts_with("os/"))
        .count();
    if (declared.k8s_leaves, declared.os_leaves, declared.total_leaves)
        != (k8s_leaves, os_leaves, corpus.crate_leaves.len())
    {
        findings.push(finding(
            FindingCode::UnclassifiedCrateLeaf,
            REGENERABLE_REGIONS,
            &format!(
                "declared census k8s_leaves={} os_leaves={} total_leaves={} does not equal the tree at HEAD ({} / {} / {}); a stale census silently converts every count above it into a claim about a tree that no longer exists",
                declared.k8s_leaves,
                declared.os_leaves,
                declared.total_leaves,
                k8s_leaves,
                os_leaves,
                corpus.crate_leaves.len()
            ),
        ));
    }

    findings.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.message.cmp(&right.message))
    });
    let counters = Counters {
        scanned_population: corpus.documents.len()
            + corpus.completed_waves.len()
            + corpus.rules.len()
            + corpus.doctrine.len()
            + corpus.prescription_count
            + corpus.os_rust_files
            + corpus.crate_leaves.len(),
        finding_count: findings.len(),
    };
    Evaluation { counters, findings }
}

pub fn load_repository(repo_root: &Path) -> Result<Corpus, LoadError> {
    load_repository_inner(repo_root).map_err(|mut error| {
        error.path = display_path(repo_root, Path::new(&error.path));
        error
    })
}

fn load_repository_inner(repo_root: &Path) -> Result<Corpus, LoadError> {
    let program_root = repo_root.join(PROGRAM_ROOT);
    let documents = load_markdown_documents(repo_root, &program_root)?;
    if documents.is_empty() {
        return Ok(Corpus::default());
    }
    let completed_waves = load_wave_registry(repo_root, &program_root)?;
    let rules = load_rule_records(repo_root)?;
    let doctrine = load_doctrine_records(repo_root, &program_root)?;
    let prescription_count = count_lane_entries(repo_root, &program_root.join("prescriptions"))?;
    let scan = scan_os_upstream_emit_sites(repo_root)?;
    let crate_leaves = scan_crate_leaves(repo_root)?;
    let declared_leaves = load_declared_leaves(repo_root)?;
    let divergence_ledger = load_divergence_ledger(repo_root)?;
    let cross_seam_edges = scan_cross_seam_edges(repo_root)?;
    Ok(Corpus {
        documents,
        completed_waves,
        rules,
        doctrine,
        prescription_count,
        upstream_emit_sites: scan.upstream_emit_sites,
        upstream_emit_identities: scan.upstream_emit_identities,
        dynamic_api_version_sites: scan.dynamic_api_version_sites,
        os_rust_files: scan.os_rust_files,
        crate_leaves,
        declared_leaves,
        divergence_ledger,
        cross_seam_edges,
    })
}

/// Parse a JSON artifact. A malformed or unnavigable declaration is a `LoadError`, never a silent
/// default: an empty declaration would report a clean partition over a corpus it never opened.
fn read_json(path: &Path, code: &'static str) -> Result<Value, LoadError> {
    serde_json::from_str(&read_utf8(path, code)?)
        .map_err(|error| load_error(code, path, &format!("file must be valid JSON: {error}")))
}

fn json_usize(
    document: &Value,
    pointer: &str,
    path: &Path,
    code: &'static str,
) -> Result<usize, LoadError> {
    document
        .pointer(pointer)
        .and_then(Value::as_u64)
        .and_then(|number| usize::try_from(number).ok())
        .ok_or_else(|| {
            load_error(
                code,
                path,
                &format!("`{pointer}` must be present and a non-negative integer"),
            )
        })
}

/// Reads the divergence ledger's row count and its own declared growth budget.
fn load_divergence_ledger(repo_root: &Path) -> Result<DivergenceLedger, LoadError> {
    const CODE: &str = "R-DOC-K8S-PORT-DIVERGENCE-LEDGER-MALFORMED";
    let path = repo_root.join(DIVERGENCE_LEDGER);
    if !path.is_file() {
        return Err(load_error(
            "R-DOC-K8S-PORT-DIVERGENCE-LEDGER-MISSING",
            &path,
            "the divergence ledger is required; without it the growth policy is data nothing evaluates",
        ));
    }
    let document = read_json(&path, CODE)?;
    let rows_value = document
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| load_error(CODE, &path, "`rows` must be an array"))?;
    let mut row_ids = Vec::with_capacity(rows_value.len());
    for row in rows_value {
        let id = row.get("id").and_then(Value::as_str).ok_or_else(|| {
            load_error(CODE, &path, "every divergence row must carry a string `id`")
        })?;
        row_ids.push(id.to_owned());
    }
    row_ids.sort();
    Ok(DivergenceLedger {
        rows: rows_value.len(),
        baseline_seed_count: json_usize(
            &document,
            "/growth_policy/baseline_seed_count",
            &path,
            CODE,
        )?,
        max_new_rows_per_wave: json_usize(
            &document,
            "/growth_policy/max_new_rows_per_wave",
            &path,
            CODE,
        )?,
        row_ids,
    })
}

/// Enumerates every `<capability>/<face>/<crate>` directory holding a `Cargo.toml` under `k8s/` and
/// `os/`, which is the census reproducer `git ls-files 'k8s/*/*/Cargo.toml' 'os/*/*/Cargo.toml'`
/// evaluated at fixed depth.
fn scan_crate_leaves(repo_root: &Path) -> Result<Vec<String>, LoadError> {
    let mut leaves = Vec::new();
    for capability in SEAM_CAPABILITIES {
        let root = repo_root.join(capability);
        for face in read_dir_sorted(&root, "R-DOC-SEAM-LEAF-TREE-UNREADABLE")? {
            if !face.is_dir() {
                continue;
            }
            for leaf in read_dir_sorted(&face, "R-DOC-SEAM-LEAF-TREE-UNREADABLE")? {
                if leaf.join("Cargo.toml").is_file() {
                    leaves.push(display_path(repo_root, &leaf));
                }
            }
        }
    }
    leaves.sort();
    if leaves.is_empty() {
        return Err(load_error(
            "R-DOC-SEAM-LEAF-TREE-EMPTY",
            repo_root,
            "no crate leaves were enumerated under k8s/ or os/; a zero denominator would report every declared leaf row as classified without reading the tree",
        ));
    }
    Ok(leaves)
}

/// INV-1 / INV-2: parse Cargo manifests and Buck buildfiles under `os/` and `k8s/` for
/// forbidden cross-seam edges.
///
/// A formatting-specific `git grep` for `pkg = { path` misses named dependency tables
/// (`[dependencies.k8s-foo]`), quoted keys (`"k8s-foo" = { path = … }`), workspace
/// inheritance (`k8s-foo.workspace = true`), and `package = "k8s-…"` renames. Buck target
/// edges (`//k8s/…` from `os/`, `//os/…` from `k8s/`) are the same seam crossed through the
/// Buck graph — including `BUCK.v2`, which buck2 prefers over `BUCK` when both exist.
///
/// Only recognized manifest/buildfile basenames are read as UTF-8. Binary fixtures under
/// `os/` or `k8s/` (e.g. DER certs in `testdata/`) must not trip `R-DOC-SEAM-MANIFEST-UNREADABLE`.
fn scan_cross_seam_edges(repo_root: &Path) -> Result<Vec<CrossSeamEdge>, LoadError> {
    let mut edges = Vec::new();
    for (capability, forbidden_prefix, forbidden_buck_root) in [
        ("os", "k8s-", "//k8s/"),
        ("k8s", "os-", "//os/"),
    ] {
        let root = repo_root.join(capability);
        if !root.is_dir() {
            continue;
        }
        for path in collect_files(&root, "R-DOC-SEAM-MANIFEST-TREE-UNREADABLE")? {
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let rel = display_path(repo_root, &path);
            match name {
                "Cargo.toml" => {
                    let contents = read_utf8(&path, "R-DOC-SEAM-MANIFEST-UNREADABLE")?;
                    for dependency in forbidden_package_deps(&contents, forbidden_prefix) {
                        edges.push(CrossSeamEdge {
                            path: rel.clone(),
                            dependency,
                        });
                    }
                }
                "BUCK" | "BUCK.v2" => {
                    let contents = read_utf8(&path, "R-DOC-SEAM-MANIFEST-UNREADABLE")?;
                    for dependency in forbidden_buck_deps(&contents, forbidden_buck_root) {
                        edges.push(CrossSeamEdge {
                            path: rel.clone(),
                            dependency,
                        });
                    }
                }
                _ => {}
            }
        }
    }
    edges.sort_by(|left, right| {
        (&left.path, &left.dependency).cmp(&(&right.path, &right.dependency))
    });
    Ok(edges)
}

#[derive(Clone, Copy)]
enum DepSection {
    Inactive,
    /// Inside `[dependencies]` / `[dev-dependencies]` / `[target.*.dependencies]`.
    Table,
    /// Inside `[dependencies.<pkg>]` — scan for `package = "…"` renames too.
    NamedPackage,
}

/// Package names under dependency tables that start with `forbidden_prefix`.
pub fn forbidden_package_deps(contents: &str, forbidden_prefix: &str) -> Vec<String> {
    let mut section = DepSection::Inactive;
    let mut hits = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let inner = &trimmed[1..trimmed.len() - 1];
            match classify_dependency_header(inner) {
                Some(None) => section = DepSection::Table,
                Some(Some(package)) => {
                    section = DepSection::NamedPackage;
                    if package.starts_with(forbidden_prefix) {
                        hits.push(package);
                    }
                }
                None => section = DepSection::Inactive,
            }
            continue;
        }
        match section {
            DepSection::Table => {
                if let Some(package) = dependency_key(trimmed) {
                    if package.starts_with(forbidden_prefix) {
                        hits.push(package);
                    }
                }
                // Rename evasion: `local = { package = "k8s-foo", path = "…" }`.
                if let Some(package) = package_rename_value(trimmed) {
                    if package.starts_with(forbidden_prefix) {
                        hits.push(package);
                    }
                }
            }
            DepSection::NamedPackage => {
                if let Some(package) = package_rename_value(trimmed) {
                    if package.starts_with(forbidden_prefix) {
                        hits.push(package);
                    }
                }
            }
            DepSection::Inactive => {}
        }
    }
    hits.sort();
    hits.dedup();
    hits
}

/// BUCK target literals that cross the seam (`"//k8s/…"` from `os/`, `"//os/…"` from `k8s/`).
pub fn forbidden_buck_deps(contents: &str, forbidden_root: &str) -> Vec<String> {
    let mut hits = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        for (index, _) in trimmed.match_indices(forbidden_root) {
            let rest = &trimmed[index..];
            let end = rest
                .find(|character: char| {
                    character == '"'
                        || character == '\''
                        || character == ','
                        || character.is_whitespace()
                })
                .unwrap_or(rest.len());
            let target = &rest[..end];
            if !target.is_empty() {
                hits.push(target.to_owned());
            }
        }
    }
    hits.sort();
    hits.dedup();
    hits
}

/// `Some(None)` = open dependency table; `Some(Some(pkg))` = named package table; `None` = other.
fn classify_dependency_header(inner: &str) -> Option<Option<String>> {
    let lower = inner.to_ascii_lowercase();
    let Some(index) = lower.rfind("dependencies") else {
        return None;
    };
    let after = &inner[index + "dependencies".len()..];
    if after.is_empty() {
        return Some(None);
    }
    after
        .strip_prefix('.')
        .filter(|package| !package.is_empty())
        .and_then(toml_bare_or_quoted_key)
        .map(Some)
}

fn dependency_key(line: &str) -> Option<String> {
    let key = line.split_once('=')?.0.trim();
    let name = key.split('.').next()?.trim();
    toml_bare_or_quoted_key(name)
}

/// Bare TOML keys, or single-/double-quoted keys with an otherwise bare package name.
///
/// Cargo accepts `"k8s-foo" = { path = "…" }` and `[dependencies."k8s-foo"]`; rejecting the
/// surrounding quotes would leave INV-1 green over a real cross-seam edge.
fn toml_bare_or_quoted_key(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let unquoted = match raw.as_bytes() {
        [b'"', .., b'"'] | [b'\'', .., b'\''] if raw.len() >= 2 => &raw[1..raw.len() - 1],
        _ => raw,
    };
    if unquoted.is_empty()
        || !unquoted
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return None;
    }
    Some(unquoted.to_owned())
}

/// `package = "k8s-foo"` / `package = 'k8s-foo'` inside an inline table or named dep section.
fn package_rename_value(line: &str) -> Option<String> {
    for (index, _) in line.match_indices("package") {
        let before_ok = index == 0
            || !line.as_bytes()[index - 1].is_ascii_alphanumeric() && line.as_bytes()[index - 1] != b'_' && line.as_bytes()[index - 1] != b'-';
        let after = &line[index + "package".len()..];
        if !before_ok || !after.starts_with(|c: char| c == '=' || c.is_whitespace()) {
            continue;
        }
        let value = after.trim_start().strip_prefix('=')?.trim_start();
        let quote = value.chars().next().filter(|c| *c == '"' || *c == '\'')?;
        let rest = &value[quote.len_utf8()..];
        let end = rest.find(quote)?;
        let name = &rest[..end];
        if !name.is_empty() {
            return Some(name.to_owned());
        }
    }
    None
}

/// Reads the declared leaf census. Absence is a load error rather than a finding: an evaluation run
/// against a missing declaration would report a clean partition over a corpus it never opened.
fn load_declared_leaves(repo_root: &Path) -> Result<DeclaredLeaves, LoadError> {
    let path = repo_root.join(REGENERABLE_REGIONS);
    if !path.is_file() {
        return Err(load_error(
            "R-DOC-K8S-PORT-REGIONS-MISSING",
            &path,
            "the regenerable-region declaration is required; without it the leaf census is a literal that nothing recomputes",
        ));
    }
    // NAVIGATED, not line-scanned. The previous reader matched `"path": "` against every line of
    // the file, so a leaf path relocated into ANY other object still landed in `rows`: the row
    // count and every per-leaf presence check stayed green over a leaf that was no longer
    // classified. `declared_count` had the same shape one level worse — `split_once` took the
    // FIRST match anywhere in the file, so a future block mentioning `k8s_leaves` earlier would
    // have silently pinned the comparison to the wrong number.
    const CODE: &str = "R-DOC-K8S-PORT-REGIONS-MALFORMED";
    let document = read_json(&path, CODE)?;
    // Validate `/regions` unconditionally — before leaf origin checks — so a region covering only
    // first-party leaves, missing a producer/receipt, or overlapping another region cannot hide
    // behind an all-`first-party` census (fail_closed_conditions in regenerable-regions.json).
    let regions = load_validated_regions(&document, &path, CODE)?;
    let leaves = document
        .pointer("/origin_classification/leaves")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            load_error(CODE, &path, "`origin_classification.leaves` must be an array")
        })?;
    // A `Vec`, never a `BTreeSet`: duplicates must survive, because the row-count comparison in
    // `evaluate` is what catches a deletion or a rename that a per-leaf presence check cannot see.
    let mut rows = Vec::with_capacity(leaves.len());
    const CLOSED_ORIGINS: &[&str] = &["first-party", "generated"];
    for leaf in leaves {
        let value = leaf.get("path").and_then(Value::as_str).ok_or_else(|| {
            load_error(
                CODE,
                &path,
                "every `origin_classification.leaves` row must carry a string `path`",
            )
        })?;
        let origin = leaf.get("origin").and_then(Value::as_str).ok_or_else(|| {
            load_error(
                CODE,
                &path,
                "every `origin_classification.leaves` row must carry a closed `origin` classification",
            )
        })?;
        if !CLOSED_ORIGINS.contains(&origin) {
            return Err(load_error(
                CODE,
                &path,
                &format!(
                    "leaf origin `{origin}` is outside the closed set {:?}",
                    CLOSED_ORIGINS
                ),
            ));
        }
        let covered = regions.iter().any(|prefix| region_covers(prefix, value));
        // Vocabulary: every generated leaf MUST resolve to exactly one registered region.
        // With `regions` empty (W0: no producer), `generated` is refuse. A first-party leaf
        // MUST NOT fall inside a registered region path_prefix.
        match origin {
            "generated" if !covered => {
                return Err(load_error(
                    CODE,
                    &path,
                    &format!(
                        "generated leaf `{value}` resolves to no registered region; register the producer region in the same change or keep the leaf first-party"
                    ),
                ));
            }
            "first-party" if covered => {
                return Err(load_error(
                    CODE,
                    &path,
                    &format!(
                        "first-party leaf `{value}` falls inside a registered region path_prefix; generated ownership and hand-edit ownership cannot share a path"
                    ),
                ));
            }
            _ => {}
        }
        rows.push(value.to_owned());
    }
    Ok(DeclaredLeaves {
        rows,
        k8s_leaves: json_usize(
            &document,
            "/origin_classification/census/k8s_leaves",
            &path,
            CODE,
        )?,
        os_leaves: json_usize(
            &document,
            "/origin_classification/census/os_leaves",
            &path,
            CODE,
        )?,
        total_leaves: json_usize(
            &document,
            "/origin_classification/census/total_leaves",
            &path,
            CODE,
        )?,
    })
}

/// Six-axis receipt keys required on every registered regenerable region.
const REGION_RECEIPT_AXES: &[&str] = &[
    "pin",
    "snapshot_digest",
    "engine_digest",
    "rulepack_digest",
    "toolchain_digest",
    "formatter_digest",
];

/// Load `/regions`, validate producer + six receipt axes + disjoint prefixes, return prefixes.
fn load_validated_regions(
    document: &Value,
    path: &Path,
    code: &'static str,
) -> Result<Vec<String>, LoadError> {
    let Some(regions_value) = document.get("regions") else {
        return Err(load_error(
            code,
            path,
            "`regions` must be present (use `[]` while no producer exists)",
        ));
    };
    let regions = regions_value.as_array().ok_or_else(|| {
        load_error(code, path, "`regions` must be an array")
    })?;
    let mut prefixes: Vec<String> = Vec::with_capacity(regions.len());
    for (index, region) in regions.iter().enumerate() {
        let region_id = region
            .get("region_id")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .ok_or_else(|| {
                load_error(
                    code,
                    path,
                    &format!("regions[{index}] must carry a non-empty string `region_id`"),
                )
            })?;
        let prefix = region
            .get("path_prefix")
            .and_then(Value::as_str)
            .filter(|prefix| !prefix.is_empty())
            .ok_or_else(|| {
                load_error(
                    code,
                    path,
                    &format!("region `{region_id}` must carry a non-empty string `path_prefix`"),
                )
            })?;
        if region.get("origin").and_then(Value::as_str).is_none() {
            return Err(load_error(
                code,
                path,
                &format!("region `{region_id}` must carry a string `origin`"),
            ));
        }
        match region.get("producer") {
            Some(Value::String(producer)) if !producer.is_empty() => {}
            Some(Value::Object(producer)) if !producer.is_empty() => {}
            Some(Value::Null) | None => {
                return Err(load_error(
                    code,
                    path,
                    &format!(
                        "region `{region_id}` is registered without a producer; a no-hand-edit region with no regeneration owner is unmaintainable"
                    ),
                ));
            }
            Some(_) => {
                return Err(load_error(
                    code,
                    path,
                    &format!(
                        "region `{region_id}` producer must be a non-empty string or object"
                    ),
                ));
            }
        }
        let receipt = region.get("receipt").and_then(Value::as_object).ok_or_else(|| {
            load_error(
                code,
                path,
                &format!("region `{region_id}` must carry an object `receipt` with all six axes"),
            )
        })?;
        for axis in REGION_RECEIPT_AXES {
            if !receipt.contains_key(*axis) || receipt.get(*axis).is_some_and(Value::is_null) {
                return Err(load_error(
                    code,
                    path,
                    &format!(
                        "region `{region_id}` receipt is missing axis `{axis}`; all six receipt axes are required"
                    ),
                ));
            }
        }
        for prior in &prefixes {
            if path_prefixes_overlap(prior, prefix) {
                return Err(load_error(
                    code,
                    path,
                    &format!(
                        "region path prefixes `{prior}` and `{prefix}` overlap; two regions claiming the same path make the no-hand-edit owner ambiguous"
                    ),
                ));
            }
        }
        prefixes.push(prefix.to_owned());
    }
    Ok(prefixes)
}

fn region_covers(prefix: &str, leaf: &str) -> bool {
    leaf == prefix || leaf.starts_with(&format!("{prefix}/"))
}

fn path_prefixes_overlap(left: &str, right: &str) -> bool {
    left == right
        || left.starts_with(&format!("{right}/"))
        || right.starts_with(&format!("{left}/"))
}

/// What one pass over `os/**/*.rs` observed.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct OsScan {
    upstream_emit_sites: usize,
    upstream_emit_identities: Vec<String>,
    dynamic_api_version_sites: usize,
    os_rust_files: usize,
}

/// Counts INV-3 emit sites over `os/**/*.rs`.
///
/// One line counts at most once, matching the section-8 `git grep -cE` reproducer, which counts
/// matching lines rather than matches.
fn scan_os_upstream_emit_sites(repo_root: &Path) -> Result<OsScan, LoadError> {
    let root = repo_root.join(OS_ROOT);
    let mut scan = OsScan::default();
    for path in collect_files(&root, "R-DOC-OS-TREE-UNREADABLE")? {
        if path.extension().is_some_and(|extension| extension == "rs") {
            let contents = read_utf8(&path, "R-DOC-OS-SOURCE-UNREADABLE")?;
            let rel = path
                .strip_prefix(repo_root)
                .unwrap_or(path.as_path())
                .to_string_lossy()
                .into_owned();
            for value in upstream_emit_identity_values(&contents) {
                scan.upstream_emit_identities
                    .push(format!("{rel}\t{value}"));
            }
            scan.upstream_emit_sites += upstream_emit_sites(&contents);
            scan.dynamic_api_version_sites += dynamic_api_version_sites(&contents);
            scan.os_rust_files += 1;
        }
    }
    scan.upstream_emit_identities.sort();
    if scan.os_rust_files == 0 {
        return Err(load_error(
            "R-DOC-OS-TREE-EMPTY",
            &root,
            "no Rust sources were scanned; a zero denominator would report the INV-3 ceiling as satisfied without reading anything",
        ));
    }
    Ok(scan)
}

/// A line the census reads at all. A `//`-leading line is DOCUMENTATION, not emission: counting
/// `// example: apiVersion: apps/v1` would red the ratchet with a diagnostic ("consume the seam
/// instead of hand-writing it") that is wholly wrong for someone who wrote a comment. The mechanism
/// was live and only accidentally quiet — `os/core/config-docs-domain/src/lib.rs:12` is extracted
/// and classified today and merely happens to be Talos.
///
/// ponytail: known ceiling — a TRAILING comment on a code line is still scanned, and a `//`-leading
/// line inside a raw string literal is now skipped. Both are strictly narrower than the doc-comment
/// case this closes; closing them needs a Rust tokenizer, which no line census is worth.
fn is_scannable_line(line: &str) -> bool {
    !line.trim_start().starts_with("//")
}


/// One apiVersion value per INV-3 emit line (same filter as [`upstream_emit_sites`]).
fn upstream_emit_identity_values(contents: &str) -> Vec<String> {
    contents
        .lines()
        .filter(|line| is_scannable_line(line) && emits_upstream_group(line))
        .filter_map(|line| {
            api_version_values(line)
                .into_iter()
                .find(|value| is_upstream_api_version(value))
                .map(str::to_owned)
        })
        .collect()
}

/// Number of lines emitting an upstream-Kubernetes `apiVersion:`.
pub fn upstream_emit_sites(contents: &str) -> usize {
    contents
        .lines()
        .filter(|line| is_scannable_line(line) && emits_upstream_group(line))
        .count()
}

/// Number of lines whose `apiVersion` value is BUILT AT RUNTIME (`"apiVersion: {}"`,
/// `format!("apiVersion: {group}/v1")`). A tokenizer does not help here: the value is not in the
/// source at all, so it is unclassifiable however well the line is lexed. The gate fails closed on
/// it rather than counting it as absent.
pub fn dynamic_api_version_sites(contents: &str) -> usize {
    contents
        .lines()
        .filter(|line| {
            if !is_scannable_line(line) {
                return false;
            }
            // Format-placeholder emissions (`apiVersion: {}`) are unclassifiable.
            if api_version_values(line)
                .iter()
                .any(|value| value.contains('{'))
            {
                return true;
            }
            // Split/computed emissions: key written with an empty same-line value
            // (e.g. `out.push_str("apiVersion: ");` then a later push of the group).
            // Bare identifier mentions (`let field = "apiVersion";`) are not emissions.
            empty_api_version_emit(line)
        })
        .count()
}

/// True when the line emits an `apiVersion` key with no resolvable static value.
fn empty_api_version_emit(line: &str) -> bool {
    for (index, _) in line.match_indices("apiVersion") {
        let mut rest = &line[index + "apiVersion".len()..];
        rest = strip_quote(rest);
        // Structured insert: `map.insert("apiVersion", Value::String(group))`
        // — key closes, then a comma introduces a separately-supplied value.
        if rest.starts_with(',') || rest.starts_with(", ") {
            return true;
        }
        let Some(mut rest) = rest.strip_prefix(':') else {
            continue;
        };
        rest = rest.trim_start_matches([' ', '\t']);
        rest = strip_quote(rest);
        if rest.is_empty()
            || rest.starts_with("\\n")
            || rest.starts_with('"')
            || rest.starts_with('\'')
            || rest.starts_with(')')
            || rest.starts_with(';')
            || rest.starts_with(',')
        {
            return true;
        }
    }
    false
}

/// Every `apiVersion` value on the line, **normalized once before classification**. Classifying raw
/// text is what made the previous predicate under-inclusive: it enumerated spellings, so a quoted
/// value or a quoted key fell out silently and understated the census. This tolerates a quoted key
/// (`"apiVersion":`), leading whitespace, and a quoted value (`"apps/v1"`, `\"apps/v1\"` inside a
/// Rust literal, `'apps/v1'`). The value terminates at the first quote, backslash escape or
/// whitespace, so a trailing comment cannot be read as part of it.
fn api_version_values(line: &str) -> Vec<&str> {
    let mut values = Vec::new();
    for (index, _) in line.match_indices("apiVersion") {
        let mut rest = &line[index + "apiVersion".len()..];
        rest = strip_quote(rest);
        // No colon means this was a bare mention such as `let field = "apiVersion";`, not an emit.
        let Some(mut rest) = rest.strip_prefix(':') else {
            continue;
        };
        rest = rest.trim_start_matches([' ', '\t']);
        rest = strip_quote(rest);
        let end = rest
            .find(|character: char| {
                character == '"' || character == '\'' || character == '\\' || character.is_whitespace()
            })
            .unwrap_or(rest.len());
        let value = &rest[..end];
        if !value.is_empty() {
            values.push(value);
        }
    }
    values
}

fn strip_quote(rest: &str) -> &str {
    rest.strip_prefix("\\\"")
        .or_else(|| rest.strip_prefix('"'))
        .or_else(|| rest.strip_prefix('\''))
        .unwrap_or(rest)
}

/// The discriminator is the **API group SHAPE**, never the token `apiVersion` (trap T-1) and never a
/// closed allowlist of the groups that happened to exist at census time. A grouped value is upstream
/// Kubernetes when its group segment — `<group>/<version>` — either ends in `.k8s.io` or is one of
/// the five suffix-less upstream groups; an ungrouped value is upstream only when it is exactly
/// `v1`. Talos is excluded structurally: it emits bare `v1alpha1`/`v1beta1` with no group segment at
/// all, so nothing slash-shaped can swallow it and the bare arm cannot either.
fn is_upstream_api_version(value: &str) -> bool {
    match value.split_once('/') {
        Some((group, _)) => {
            // A group is a DNS-subdomain-shaped token. Bounding the charset stops a trailing
            // comment such as `apiVersion: v1alpha1 // see rbac.authorization.k8s.io/v1` from
            // being read as a group segment.
            !group.is_empty()
                && group.chars().all(|character| {
                    character.is_ascii_alphanumeric() || character == '.' || character == '-'
                })
                && (group.ends_with(".k8s.io") || SUFFIXLESS_UPSTREAM_GROUPS.contains(&group))
        }
        None => value == "v1",
    }
}

fn emits_upstream_group(line: &str) -> bool {
    api_version_values(line)
        .iter()
        .any(|value| is_upstream_api_version(value))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadError {
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

impl fmt::Display for LoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}: {}", self.code, self.path, self.message)
    }
}

impl std::error::Error for LoadError {}

fn has_baseline_header(contents: &str) -> bool {
    let mut lines = contents.lines();
    while let Some(line) = lines.next() {
        if line.trim() == "## Baseline version header" {
            let section = lines
                .by_ref()
                .take_while(|next| !next.trim_start().starts_with("## "))
                .collect::<Vec<_>>()
                .join("\n");
            return section.contains("Repository baseline")
                && (section.contains("Kubernetes upstream")
                    || section.contains("Upstream Kubernetes pin"))
                && SIX_AXIS_TOKENS.iter().all(|token| section.contains(token));
        }
    }
    false
}

fn finding(code: FindingCode, path: &str, message: &str) -> Finding {
    Finding {
        code,
        path: path.to_owned(),
        message: message.to_owned(),
    }
}

fn non_empty(value: Option<&str>) -> bool {
    value.is_some_and(|value| !value.trim().is_empty())
}

fn load_markdown_documents(
    repo_root: &Path,
    program_root: &Path,
) -> Result<Vec<MarkdownDocument>, LoadError> {
    let paths = collect_files(program_root, "R-DOC-PROGRAM-DOCUMENTS-UNREADABLE")?;
    let mut documents = Vec::new();
    for path in paths {
        if path.extension().is_some_and(|extension| extension == "md") {
            documents.push(MarkdownDocument {
                path: display_path(repo_root, &path),
                contents: read_utf8(&path, "R-DOC-PROGRAM-DOCUMENT-UNREADABLE")?,
            });
        }
    }
    for relative in [
        "docs/adr-archive/ADR-0637-owned-deterministic-go-to-rust-port-engine.md",
        "docs/adr-archive/ADR-0638-mechanically-maintained-kubernetes-rust-port.md",
    ] {
        let path = repo_root.join(relative);
        documents.push(MarkdownDocument {
            path: relative.to_owned(),
            contents: read_utf8(&path, "R-DOC-PROGRAM-DOCUMENT-UNREADABLE")?,
        });
    }
    documents.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(documents)
}

fn load_wave_registry(
    repo_root: &Path,
    program_root: &Path,
) -> Result<Vec<CompletedWave>, LoadError> {
    let path = repo_root.join(WAVE_REGISTRY);
    if !path.exists() {
        return Err(load_error(
            "R-DOC-WAVE-REGISTRY-MISSING",
            &path,
            "wave registry is required; prose cannot attest that no wave completed",
        ));
    }

    let contents = read_utf8(&path, "R-DOC-WAVE-REGISTRY-MALFORMED")?;
    let mut lines = contents
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'));
    if lines.next().map(str::trim) != Some("version=1") {
        return Err(load_error(
            "R-DOC-WAVE-REGISTRY-MALFORMED",
            &path,
            "first non-comment line must be version=1",
        ));
    }

    let mut seen_ids = BTreeSet::new();
    let mut seen_ordinals = BTreeSet::new();
    let mut waves = Vec::new();
    for line in lines {
        let fields = parse_fields(line, &path, "R-DOC-WAVE-REGISTRY-MALFORMED")?;
        ensure_only_fields(
            &fields,
            &[
                "wave",
                "ordinal",
                "completed",
                "operations_entries",
                "no_extraction_rationale",
            ],
            &path,
            "R-DOC-WAVE-REGISTRY-MALFORMED",
        )?;
        let id = required_field(&fields, "wave", &path, "R-DOC-WAVE-REGISTRY-MALFORMED")?;
        let ordinal = required_field(&fields, "ordinal", &path, "R-DOC-WAVE-REGISTRY-MALFORMED")?
            .parse::<u32>()
            .map_err(|_| {
                load_error(
                    "R-DOC-WAVE-REGISTRY-MALFORMED",
                    &path,
                    "ordinal must be an unsigned integer",
                )
            })?;
        let completed =
            required_field(&fields, "completed", &path, "R-DOC-WAVE-REGISTRY-MALFORMED")?;
        if !seen_ids.insert(id.to_owned()) || !seen_ordinals.insert(ordinal) {
            return Err(load_error(
                "R-DOC-WAVE-REGISTRY-MALFORMED",
                &path,
                "wave identifiers and ordinals must be unique",
            ));
        }
        // Resolved for EVERY row, not only the completed ones. Keying validation to consumption
        // rather than to presence left any journal path on an in-flight wave an unverified string:
        // a typo, rename or deletion stayed green until the wave eventually completed, and the
        // backlink dangled in the meantime.
        let entries = fields
            .get("operations_entries")
            .map_or(Ok(Vec::new()), |value| {
                load_journal_entries(program_root, value)
            })?;
        if completed == "true" {
            waves.push(CompletedWave {
                id: id.to_owned(),
                ordinal,
                journal_entries: entries,
                no_extraction_rationale: fields.get("no_extraction_rationale").cloned(),
            });
        } else if completed != "false" {
            return Err(load_error(
                "R-DOC-WAVE-REGISTRY-MALFORMED",
                &path,
                "completed must be true or false",
            ));
        }
    }
    Ok(waves)
}

fn load_journal_entries(program_root: &Path, value: &str) -> Result<Vec<JournalEntry>, LoadError> {
    let mut entries = Vec::new();
    for entry in value.split(',').filter(|entry| !entry.trim().is_empty()) {
        let entry = entry.trim();
        let relative = Path::new(entry);
        if relative.is_absolute()
            || relative
                .components()
                .any(|component| component == Component::ParentDir)
        {
            return Err(load_error(
                "R-DOC-WAVE-REGISTRY-MALFORMED",
                program_root,
                "operations_entries paths must be relative to operations/ and cannot escape it",
            ));
        }
        let path = program_root.join("operations").join(relative);
        let metadata = fs::symlink_metadata(&path).map_err(|_| {
            load_error(
                "R-DOC-JOURNAL-ENTRY-UNREADABLE",
                &path,
                "operations journal entry metadata must be readable",
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(load_error(
                "R-DOC-JOURNAL-ENTRY-UNREADABLE",
                &path,
                "operations journal entries must be regular files, not symlinks",
            ));
        }
        let contents = read_utf8(&path, "R-DOC-JOURNAL-ENTRY-UNREADABLE")?;
        if contents.trim().is_empty() {
            return Err(load_error(
                "R-DOC-JOURNAL-ENTRY-EMPTY",
                &path,
                "operations journal entry is empty",
            ));
        }
        entries.push(JournalEntry {
            id: entry.to_owned(),
        });
    }
    Ok(entries)
}

/// Every rule file under `specs/port-rules` or `specs/k8s-port/rules` is a record with
/// `rule_id`, `rule_kind: neutral|corpus`, and `operations_journal_ref` front-matter fields.
fn load_rule_records(repo_root: &Path) -> Result<Vec<RuleRecord>, LoadError> {
    let mut records = Vec::new();
    for root in [
        repo_root.join("specs/port-rules"),
        repo_root.join("specs/k8s-port/rules"),
    ] {
        if !root.exists() {
            continue;
        }
        for path in collect_files(&root, "R-DOC-RULES-UNREADABLE")? {
            if path.extension().is_none_or(|extension| extension != "md") {
                return Err(load_error(
                    "R-DOC-RULE-METADATA-MALFORMED",
                    &path,
                    "rule records must be Markdown with YAML-style front matter",
                ));
            }
            let fields = parse_front_matter(
                &read_utf8(&path, "R-DOC-RULE-UNREADABLE")?,
                &path,
                "R-DOC-RULE-METADATA-MALFORMED",
            )?;
            ensure_only_fields(
                &fields,
                &["rule_id", "rule_kind", "operations_journal_ref"],
                &path,
                "R-DOC-RULE-METADATA-MALFORMED",
            )?;
            let rule_kind =
                match required_field(&fields, "rule_kind", &path, "R-DOC-RULE-METADATA-MALFORMED")?
                {
                    "neutral" => RuleKind::Neutral,
                    "corpus" => RuleKind::Corpus,
                    _ => {
                        return Err(load_error(
                            "R-DOC-RULE-METADATA-MALFORMED",
                            &path,
                            "rule_kind must be neutral or corpus",
                        ));
                    }
                };
            records.push(RuleRecord {
                path: display_path(repo_root, &path),
                id: required_field(&fields, "rule_id", &path, "R-DOC-RULE-METADATA-MALFORMED")?
                    .to_owned(),
                kind: rule_kind,
                operations_journal_reference: fields.get("operations_journal_ref").cloned(),
            });
        }
    }
    Ok(records)
}

fn load_doctrine_records(
    repo_root: &Path,
    program_root: &Path,
) -> Result<Vec<DoctrineRecord>, LoadError> {
    let root = program_root.join("doctrine");
    let mut records = Vec::new();
    for path in collect_files(&root, "R-DOC-DOCTRINE-UNREADABLE")? {
        if path.file_name().is_some_and(|name| name == "INDEX.md") {
            continue;
        }
        if path.extension().is_none_or(|extension| extension != "md") {
            return Err(load_error(
                "R-DOC-DOCTRINE-METADATA-MALFORMED",
                &path,
                "doctrine records must be Markdown with YAML-style front matter",
            ));
        }
        let fields = parse_front_matter(
            &read_utf8(&path, "R-DOC-DOCTRINE-UNREADABLE")?,
            &path,
            "R-DOC-DOCTRINE-METADATA-MALFORMED",
        )?;
        ensure_only_fields(
            &fields,
            &[
                "doctrine_id",
                "authored_wave",
                "binding_outside_lane",
                "adr_reference",
            ],
            &path,
            "R-DOC-DOCTRINE-METADATA-MALFORMED",
        )?;
        let authored_wave = required_field(
            &fields,
            "authored_wave",
            &path,
            "R-DOC-DOCTRINE-METADATA-MALFORMED",
        )?
        .parse::<u32>()
        .map_err(|_| {
            load_error(
                "R-DOC-DOCTRINE-METADATA-MALFORMED",
                &path,
                "authored_wave must be an unsigned integer",
            )
        })?;
        let binding_outside_lane = match required_field(
            &fields,
            "binding_outside_lane",
            &path,
            "R-DOC-DOCTRINE-METADATA-MALFORMED",
        )? {
            "true" => true,
            "false" => false,
            _ => {
                return Err(load_error(
                    "R-DOC-DOCTRINE-METADATA-MALFORMED",
                    &path,
                    "binding_outside_lane must be true or false",
                ));
            }
        };
        records.push(DoctrineRecord {
            path: display_path(repo_root, &path),
            id: required_field(
                &fields,
                "doctrine_id",
                &path,
                "R-DOC-DOCTRINE-METADATA-MALFORMED",
            )?
            .to_owned(),
            authored_wave,
            binding_outside_lane,
            adr_reference: fields.get("adr_reference").cloned(),
        });
    }
    Ok(records)
}

fn count_lane_entries(_repo_root: &Path, root: &Path) -> Result<usize, LoadError> {
    Ok(collect_files(root, "R-DOC-PRESCRIPTIONS-UNREADABLE")?
        .into_iter()
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .filter(|path| path.file_name().is_none_or(|name| name != "INDEX.md"))
        .count())
}

fn parse_front_matter(
    contents: &str,
    path: &Path,
    code: &'static str,
) -> Result<BTreeMap<String, String>, LoadError> {
    let mut lines = contents.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Err(load_error(code, path, "front matter must start with ---"));
    }
    let mut fields = BTreeMap::new();
    for line in lines {
        if line.trim() == "---" {
            return Ok(fields);
        }
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| load_error(code, path, "front-matter fields must be key: value"))?;
        if key.trim().is_empty()
            || fields
                .insert(key.trim().to_owned(), value.trim().to_owned())
                .is_some()
        {
            return Err(load_error(
                code,
                path,
                "front-matter fields must have unique non-empty keys",
            ));
        }
    }
    Err(load_error(
        code,
        path,
        "front matter is missing its closing ---",
    ))
}

fn parse_fields(
    line: &str,
    path: &Path,
    code: &'static str,
) -> Result<BTreeMap<String, String>, LoadError> {
    let mut fields = BTreeMap::new();
    for field in line.split(';') {
        let (key, value) = field.split_once('=').ok_or_else(|| {
            load_error(
                code,
                path,
                "registry fields must be key=value separated by semicolons",
            )
        })?;
        if key.trim().is_empty()
            || fields
                .insert(key.trim().to_owned(), value.trim().to_owned())
                .is_some()
        {
            return Err(load_error(
                code,
                path,
                "registry fields must have unique non-empty keys",
            ));
        }
    }
    Ok(fields)
}

fn ensure_only_fields(
    fields: &BTreeMap<String, String>,
    allowed: &[&str],
    path: &Path,
    code: &'static str,
) -> Result<(), LoadError> {
    if fields
        .keys()
        .any(|field| !allowed.contains(&field.as_str()))
    {
        return Err(load_error(code, path, "metadata contains an unknown field"));
    }
    Ok(())
}

fn required_field<'a>(
    fields: &'a BTreeMap<String, String>,
    name: &str,
    path: &Path,
    code: &'static str,
) -> Result<&'a str, LoadError> {
    fields
        .get(name)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| load_error(code, path, &format!("metadata field '{name}' is required")))
}

fn collect_files(root: &Path, code: &'static str) -> Result<Vec<PathBuf>, LoadError> {
    let metadata = fs::metadata(root)
        .map_err(|_| load_error(code, root, "directory is missing or unreadable"))?;
    if !metadata.is_dir() {
        return Err(load_error(code, root, "expected a directory"));
    }
    let mut paths = Vec::new();
    collect_files_inner(root, code, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_files_inner(
    root: &Path,
    code: &'static str,
    paths: &mut Vec<PathBuf>,
) -> Result<(), LoadError> {
    let mut entries = fs::read_dir(root)
        .map_err(|_| load_error(code, root, "directory is unreadable"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| load_error(code, root, "directory entry is unreadable"))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|_| load_error(code, &path, "file metadata is unreadable"))?;
        if metadata.file_type().is_symlink() {
            return Err(load_error(
                code,
                &path,
                "symbolic links are not allowed in scanned roots",
            ));
        }
        if metadata.is_dir() {
            collect_files_inner(&path, code, paths)?;
        } else if metadata.is_file() {
            paths.push(path);
        } else {
            return Err(load_error(code, &path, "only regular files are allowed"));
        }
    }
    Ok(())
}

/// One directory level, sorted. `collect_files` recurses and rejects symlinks; the leaf census only
/// needs fixed depth, and a symlinked face is not a crate leaf either way.
fn read_dir_sorted(root: &Path, code: &'static str) -> Result<Vec<PathBuf>, LoadError> {
    let mut paths = fs::read_dir(root)
        .map_err(|_| load_error(code, root, "directory is missing or unreadable"))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| load_error(code, root, "directory entry is unreadable"))?;
    paths.sort();
    Ok(paths)
}

fn read_utf8(path: &Path, code: &'static str) -> Result<String, LoadError> {
    fs::read_to_string(path).map_err(|_| load_error(code, path, "file must be readable UTF-8"))
}

fn display_path(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn load_error(code: &'static str, path: &Path, message: &str) -> LoadError {
    LoadError {
        code,
        path: path.to_string_lossy().replace('\\', "/"),
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header_document(path: &str) -> MarkdownDocument {
        MarkdownDocument {
            path: path.to_owned(),
            contents: "# fixture\n## Baseline version header\nRepository baseline\nKubernetes upstream\n`pin` snapshot_digest engine_digest rulepack_digest toolchain_digest formatter_digest\n".to_owned(),
        }
    }

    fn live_fixture() -> Corpus {
        Corpus {
            documents: vec![header_document("docs/programs/k8s-port/README.md")],
            completed_waves: vec![CompletedWave {
                id: "W0-A".to_owned(),
                ordinal: 0,
                journal_entries: vec![JournalEntry {
                    id: "W0-A/run-1.md".to_owned(),
                }],
                no_extraction_rationale: None,
            }],
            rules: vec![RuleRecord {
                path: "specs/port-rules/fixture.md".to_owned(),
                id: "fixture".to_owned(),
                kind: RuleKind::Neutral,
                operations_journal_reference: Some("W0-A/run-1".to_owned()),
            }],
            doctrine: Vec::new(),
            prescription_count: 1,
            upstream_emit_identities: FROZEN_UPSTREAM_EMIT_IDENTITIES
                .iter()
                .map(|identity| (*identity).to_owned())
                .collect(),
            upstream_emit_sites: UPSTREAM_EMIT_SITE_CEILING,
            dynamic_api_version_sites: 0,
            os_rust_files: 373,
            crate_leaves: vec![
                "k8s/ports/fixture-api".to_owned(),
                "os/core/fixture-domain".to_owned(),
            ],
            declared_leaves: DeclaredLeaves {
                rows: vec![
                    "k8s/ports/fixture-api".to_owned(),
                    "os/core/fixture-domain".to_owned(),
                ],
                k8s_leaves: 1,
                os_leaves: 1,
                total_leaves: 2,
            },
            // The live shape at W0: five baseline seeds plus the two rows this wave adds.
            divergence_ledger: DivergenceLedger {
                rows: 7,
                baseline_seed_count: 5,
                max_new_rows_per_wave: 2,
                row_ids: FROZEN_DIVERGENCE_ROW_IDS
                    .iter()
                    .map(|id| (*id).to_owned())
                    .collect(),
            },
            cross_seam_edges: Vec::new(),
        }
    }

    fn has_code(evaluation: &Evaluation, code: FindingCode) -> bool {
        evaluation
            .findings
            .iter()
            .any(|finding| finding.code == code)
    }

    #[test]
    fn baseline_header_requires_every_axis_and_both_baselines() {
        let mut corpus = live_fixture();
        corpus.documents[0].contents = "## Baseline version header\nRepository baseline\nKubernetes upstream\n`pin` snapshot_digest engine_digest rulepack_digest toolchain_digest\n".to_owned();
        let evaluation = evaluate(&corpus);
        assert!(has_code(&evaluation, FindingCode::MissingBaselineHeader));
    }

    #[test]
    fn completed_wave_without_journal_is_red_even_with_zero_findings() {
        let mut corpus = live_fixture();
        corpus.completed_waves[0].journal_entries.clear();
        let evaluation = evaluate(&corpus);
        assert!(has_code(
            &evaluation,
            FindingCode::CompletedWaveWithoutJournal
        ));
    }
    #[test]
    fn malformed_rule_metadata_is_rejected_instead_of_skipped() {
        let error = parse_front_matter(
            "---\nrule_id fixture\n---\n",
            Path::new("specs/port-rules/fixture.md"),
            "R-DOC-RULE-METADATA-MALFORMED",
        )
        .expect_err("malformed metadata must fail closed");
        assert_eq!(error.code, "R-DOC-RULE-METADATA-MALFORMED");
    }
    #[test]
    fn missing_wave_registry_is_unconditionally_rejected() {
        let root = Path::new("/definitely-missing-r-doc-repository");
        let error = load_wave_registry(root, &root.join(PROGRAM_ROOT))
            .expect_err("missing wave registry must fail closed without a prose escape");
        assert_eq!(error.code, "R-DOC-WAVE-REGISTRY-MISSING");
    }

    #[test]
    fn incomplete_wave_with_unreadable_journal_path_is_red() {
        // The regression test for resolving journal paths on EVERY row. Before the hoist this
        // returned Ok(vec![]): nothing had completed, so nothing was resolved, and the dangling
        // backlink stayed invisible until the wave eventually completed.
        let root = std::env::temp_dir().join("r-doc-incomplete-wave-journal-fixture");
        let registry = root.join(WAVE_REGISTRY);
        fs::create_dir_all(registry.parent().expect("the registry has a parent directory"))
            .expect("the fixture tree is creatable");
        fs::write(
            &registry,
            "version=1\nwave=W0-A;ordinal=0;completed=false;operations_entries=absent.md;no_extraction_rationale=\n",
        )
        .expect("the fixture registry is writable");
        let result = load_wave_registry(&root, &root.join(PROGRAM_ROOT));
        fs::remove_dir_all(&root).ok();
        let error = result.expect_err("an in-flight wave's journal path is resolved, not trusted");
        assert_eq!(error.code, "R-DOC-JOURNAL-ENTRY-UNREADABLE");
    }

    #[test]
    fn rule_without_journal_reference_is_red() {
        let mut corpus = live_fixture();
        corpus.rules[0].operations_journal_reference = Some("   ".to_owned());
        let evaluation = evaluate(&corpus);
        assert!(has_code(
            &evaluation,
            FindingCode::RuleWithoutJournalReference
        ));
    }

    #[test]
    fn overdue_cross_lane_doctrine_without_adr_is_red() {
        let mut corpus = live_fixture();
        corpus.completed_waves.extend([
            CompletedWave {
                id: "W0-B".to_owned(),
                ordinal: 1,
                journal_entries: vec![JournalEntry { id: "b".to_owned() }],
                no_extraction_rationale: Some("fixture".to_owned()),
            },
            CompletedWave {
                id: "W0-C".to_owned(),
                ordinal: 2,
                journal_entries: vec![JournalEntry { id: "c".to_owned() }],
                no_extraction_rationale: Some("fixture".to_owned()),
            },
        ]);
        corpus.doctrine.push(DoctrineRecord {
            path: "docs/programs/k8s-port/doctrine/D-1.md".to_owned(),
            id: "D-1".to_owned(),
            authored_wave: 0,
            binding_outside_lane: true,
            adr_reference: None,
        });
        let evaluation = evaluate(&corpus);
        assert!(has_code(&evaluation, FindingCode::DoctrineWithoutAdr));
    }

    #[test]
    fn empty_prescriptions_after_two_live_completed_gates_is_red_unless_later_gate_explains_it() {
        let mut corpus = live_fixture();
        corpus.prescription_count = 0;
        corpus.completed_waves.push(CompletedWave {
            id: "W0-B".to_owned(),
            ordinal: 1,
            journal_entries: vec![JournalEntry {
                id: "W0-B/run-1.md".to_owned(),
            }],
            no_extraction_rationale: None,
        });
        assert!(has_code(
            &evaluate(&corpus),
            FindingCode::PrescriptionStarvation
        ));
        corpus.completed_waves[1].no_extraction_rationale =
            Some("no incident class repeated".to_owned());
        assert!(!has_code(
            &evaluate(&corpus),
            FindingCode::PrescriptionStarvation
        ));
    }

    #[test]
    fn true_zero_document_scan_is_red_and_counters_are_distinct() {
        let evaluation = evaluate(&Corpus::default());
        assert_eq!(evaluation.counters.scanned_population, 0);
        // Default corpus is red on four independent equality pins: empty documents, emit-site
        // count, emit-site identity multiset, and divergence row-id pin.
        assert_eq!(evaluation.counters.finding_count, 4);
        assert!(has_code(&evaluation, FindingCode::EmptyProgramCorpus));
        assert!(has_code(
            &evaluation,
            FindingCode::UpstreamKubernetesSurfaceGrowth
        ));
        assert!(has_code(
            &evaluation,
            FindingCode::DivergenceLedgerGrowthBudgetExceeded
        ));
    }

    #[test]
    fn live_scan_with_zero_findings_is_green() {
        let evaluation = evaluate(&live_fixture());
        assert!(evaluation.is_green());
        assert!(evaluation.counters.scanned_population > 0);
        assert_eq!(evaluation.counters.finding_count, 0);
    }

    #[test]
    fn upstream_emit_site_census_is_frozen_at_equality() {
        let mut corpus = live_fixture();
        assert!(!has_code(
            &evaluate(&corpus),
            FindingCode::UpstreamKubernetesSurfaceGrowth
        ));
        // An unrecorded reduction is red too: a retirement that does not re-freeze the constant
        // would otherwise bank silent headroom for the site to come back.
        corpus.upstream_emit_sites = UPSTREAM_EMIT_SITE_CEILING - 1;
        assert!(has_code(
            &evaluate(&corpus),
            FindingCode::UpstreamKubernetesSurfaceGrowth
        ));
        corpus.upstream_emit_sites = UPSTREAM_EMIT_SITE_CEILING + 1;
        assert!(has_code(
            &evaluate(&corpus),
            FindingCode::UpstreamKubernetesSurfaceGrowth
        ));
    }

    #[test]
    fn the_api_group_is_the_discriminator_not_the_token_api_version() {
        // Every group in the section-3.2 table, one per line, plus the two bare-`v1` spellings the
        // reproducer admits: end of line and the `\n` escape inside a Rust string literal.
        let upstream = concat!(
            "apiVersion: v1\n",
            "    \"apiVersion: v1\\n\",\n",
            "        writeln!(out, \"apiVersion: v1\")?;\n",
            "apiVersion: apps/v1\n",
            "apiVersion: rbac.authorization.k8s.io/v1\n",
            "apiVersion: kubelet.config.k8s.io/v1beta1\n",
            "apiVersion: apiserver.config.k8s.io/v1\n",
            "apiVersion: audit.k8s.io/v1\n",
            "apiVersion: pod-security.admission.config.k8s.io/v1\n",
            // The exact regression the closed six-entry allowlist missed: upstream groups nobody
            // enumerated during the census. Both are counted because the group SHAPE decides.
            "apiVersion: batch/v1\n",
            "apiVersion: networking.k8s.io/v1\n",
            // The spelling class the previous predicate enumerated its way past. Every one of these
            // is a valid emission of upstream wire surface; every one returned false while the
            // predicate classified raw text instead of a normalized value. The quoted-value idiom
            // is already live in this corpus at os/core/machine-config-domain/src/encoder.rs:162.
            "apiVersion: \"apps/v1\"\n",
            "apiVersion: 'apps/v1'\n",
            "        \"apiVersion\": \"apps/v1\",\n",
            "apiVersion:  apps/v1\n",
            "out.push_str(\"apiVersion: \\\"v1\\\"\\n\");\n",
            "apiVersion: \"rbac.authorization.k8s.io/v1\"\n",
        );
        assert_eq!(upstream_emit_sites(upstream), 17);

        // Talos's own machine-config surface. These are correct Talos output, chartered by the port
        // of siderolabs/talos, and must never enter the count (trap T-1).
        let talos = concat!(
            "apiVersion: v1alpha1\n",
            "    \"apiVersion: v1alpha1\\n\",\n",
            "apiVersion: v1beta1\n",
            "apiVersion:\n",
            "let field = \"apiVersion\";\n",
            // A trailing reference to an upstream group is not a group segment: the token between
            // the marker and the first `/` is not DNS-subdomain-shaped, so the charset bound holds.
            "apiVersion: v1alpha1 // see rbac.authorization.k8s.io/v1\n",
            // Widening the spellings must not widen the group test: the live quoted-value idiom in
            // os/core/machine-config-domain/src/encoder.rs is Talos, and it stays out.
            "apiVersion: \"v1alpha1\"\n",
            "apiVersion: 'v1beta1'\n",
        );
        assert_eq!(upstream_emit_sites(talos), 0);
    }

    #[test]
    fn the_leaf_census_is_computed_from_the_tree_not_read_back_from_its_own_literal() {
        // Green baseline: the declaration matches the enumeration on every axis.
        assert!(!has_code(
            &evaluate(&live_fixture()),
            FindingCode::UnclassifiedCrateLeaf
        ));

        // A new leaf lands with no row. Presence catches it.
        let mut corpus = live_fixture();
        corpus.crate_leaves.push("k8s/core/newborn".to_owned());
        corpus.declared_leaves.k8s_leaves = 2;
        corpus.declared_leaves.total_leaves = 3;
        let evaluation = evaluate(&corpus);
        assert!(has_code(&evaluation, FindingCode::UnclassifiedCrateLeaf));
        assert!(evaluation
            .findings
            .iter()
            .any(|finding| finding.path == "k8s/core/newborn"));

        // A leaf is deleted and its row is left behind. Presence alone is blind here — every
        // surviving leaf still has a row — so the row count is what reds.
        let mut corpus = live_fixture();
        corpus.crate_leaves.pop();
        corpus.declared_leaves.os_leaves = 0;
        corpus.declared_leaves.total_leaves = 1;
        assert!(has_code(
            &evaluate(&corpus),
            FindingCode::UnclassifiedCrateLeaf
        ));

        // The three declared totals are pinned to the enumeration, not to each other.
        let mut corpus = live_fixture();
        corpus.declared_leaves.total_leaves = 59;
        assert!(has_code(
            &evaluate(&corpus),
            FindingCode::UnclassifiedCrateLeaf
        ));
    }

    #[test]
    fn a_comment_naming_an_upstream_api_version_is_documentation_not_emission() {
        // The false-RED direction. Every spelling here is prose; none is wire surface, and the
        // diagnostic the census would print ("consume the seam instead of hand-writing it") is
        // wholly wrong advice for someone who wrote a comment.
        let comments = concat!(
            "// example: apiVersion: apps/v1\n",
            "    /// see apiVersion: rbac.authorization.k8s.io/v1 for the shape\n",
            "//! apiVersion: v1\n",
        );
        assert_eq!(upstream_emit_sites(comments), 0);
        // ...and the skip is narrow: the same text as code still counts.
        assert_eq!(upstream_emit_sites("writeln!(out, \"apiVersion: apps/v1\")?;\n"), 1);
    }

    #[test]
    fn a_runtime_built_api_version_is_unclassifiable_and_fails_closed() {
        // NEGATIVE-OF-THE-NEGATIVE. A tokenizer cannot rescue either of these: the emitted group is
        // not in the source. Counted as absent, they are exactly how a new hand-written upstream
        // serializer lands while the census stays frozen at 16 and the gate passes.
        let dynamic = concat!(
            "        writeln!(out, \"apiVersion: {}\", \"apps/v1\")?;\n",
            "        out.push_str(&format!(\"apiVersion: {group}/v1\\n\"));\n",
        );
        assert_eq!(dynamic_api_version_sites(dynamic), 2);
        // Neither reaches the upstream census, which is why silence there is not evidence.
        assert_eq!(upstream_emit_sites(dynamic), 0);
        // A static value is not dynamic, and a comment about one is neither.
        assert_eq!(
            dynamic_api_version_sites("        writeln!(out, \"apiVersion: apps/v1\")?;\n"),
            0
        );
        assert_eq!(dynamic_api_version_sites("// apiVersion: {group}/v1\n"), 0);
        // Split emission: key without same-line value.
        assert_eq!(
            dynamic_api_version_sites("        out.push_str(\"apiVersion: \");\n"),
            1
        );
        // Structured insert: key closes, value arrives as a separate argument.
        assert_eq!(
            dynamic_api_version_sites(
                "        map.insert(\"apiVersion\", Value::String(group));\n"
            ),
            1
        );
        // Bare identifier is not an emission.
        assert_eq!(dynamic_api_version_sites("let field = \"apiVersion\";\n"), 0);

        let mut corpus = live_fixture();
        assert!(!has_code(
            &evaluate(&corpus),
            FindingCode::UnclassifiableDynamicApiVersion
        ));
        corpus.dynamic_api_version_sites = 1;
        assert!(has_code(
            &evaluate(&corpus),
            FindingCode::UnclassifiableDynamicApiVersion
        ));
    }

    #[test]
    fn the_divergence_ledger_growth_budget_has_a_reader() {
        // Green at the live shape: 7 rows == baseline 5 + budget 2, fully consumed.
        let mut corpus = live_fixture();
        assert!(!has_code(
            &evaluate(&corpus),
            FindingCode::DivergenceLedgerGrowthBudgetExceeded
        ));
        // The eighth row is the one the seam mapping promises in bold will be rejected.
        corpus.divergence_ledger.rows += 1;
        assert!(has_code(
            &evaluate(&corpus),
            FindingCode::DivergenceLedgerGrowthBudgetExceeded
        ));
        // Both knobs are read from the file, so raising the declared budget is what admits it —
        // not a constant in this crate.
        corpus.divergence_ledger.max_new_rows_per_wave += 1;
        assert!(!has_code(
            &evaluate(&corpus),
            FindingCode::DivergenceLedgerGrowthBudgetExceeded
        ));
    }

    #[test]
    fn a_leaf_path_outside_origin_classification_leaves_is_not_a_declared_row() {
        // The line-scanning reader matched `"path": "` anywhere in the file, so this document
        // declared one row. Navigation reports zero, which is what makes the row-count comparison
        // in `evaluate` able to see the relocation.
        let root = std::env::temp_dir().join("r-doc-regions-navigation-fixture");
        let path = root.join(REGENERABLE_REGIONS);
        fs::create_dir_all(path.parent().expect("the declaration has a parent directory"))
            .expect("the fixture tree is creatable");
        fs::write(
            &path,
            r#"{"regions": [],
                "somewhere_else": {"path": "os/core/relocated"},
                "origin_classification": {"leaves": [],
                  "census": {"k8s_leaves": 0, "os_leaves": 0, "total_leaves": 0}}}"#,
        )
        .expect("the fixture declaration is writable");
        let declared = load_declared_leaves(&root);
        fs::remove_dir_all(&root).ok();
        assert_eq!(
            declared.expect("the fixture declaration parses").rows,
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_malformed_declaration_is_a_load_error_not_an_empty_census() {
        let root = std::env::temp_dir().join("r-doc-regions-malformed-fixture");
        let path = root.join(REGENERABLE_REGIONS);
        fs::create_dir_all(path.parent().expect("the declaration has a parent directory"))
            .expect("the fixture tree is creatable");
        fs::write(&path, r#"{"origin_classification": {"leaves": []}}"#)
            .expect("the fixture declaration is writable");
        let result = load_declared_leaves(&root);
        fs::remove_dir_all(&root).ok();
        // An absent census must NOT degrade to zero: a clean partition reported over a corpus the
        // gate never opened is the false green this loader exists to refuse.
        assert_eq!(
            result.expect_err("a missing census is malformed").code,
            "R-DOC-K8S-PORT-REGIONS-MALFORMED"
        );
    }

    #[test]
    fn one_line_counts_once_like_the_line_oriented_reproducer() {
        assert_eq!(
            upstream_emit_sites("apiVersion: apps/v1 apiVersion: audit.k8s.io/v1"),
            1
        );
    }

    #[test]
    fn live_tree_style_fixture_is_evaluated_without_filesystem_mutation() {
        let evaluation = evaluate(&live_fixture());
        assert_eq!(evaluation.findings, Vec::<Finding>::new());
        assert_eq!(evaluation.counters.finding_count, 0);
    }

    #[test]
    fn cross_seam_deps_catch_inline_named_table_and_workspace_spellings() {
        let inline = "[dependencies]\nk8s-foo = { path = \"../../k8s/ports/foo\" }\n";
        assert_eq!(forbidden_package_deps(inline, "k8s-"), vec!["k8s-foo".to_owned()]);

        let named = "[dependencies.k8s-foo]\npath = \"../../k8s/ports/foo\"\n";
        assert_eq!(forbidden_package_deps(named, "k8s-"), vec!["k8s-foo".to_owned()]);

        let workspace = "[dependencies]\nk8s-foo.workspace = true\n";
        assert_eq!(
            forbidden_package_deps(workspace, "k8s-"),
            vec!["k8s-foo".to_owned()]
        );

        let renamed = "[dependencies]\nlocal = { package = \"k8s-foo\", path = \"../../k8s/ports/foo\" }\n";
        assert_eq!(
            forbidden_package_deps(renamed, "k8s-"),
            vec!["k8s-foo".to_owned()]
        );

        // Quoted keys are valid TOML; INV-1 must not treat the quotes as an illegal key charset.
        let quoted = "[dependencies]\n\"k8s-foo\" = { path = \"../../k8s/ports/foo\" }\n";
        assert_eq!(
            forbidden_package_deps(quoted, "k8s-"),
            vec!["k8s-foo".to_owned()]
        );
        let quoted_named = "[dependencies.\"k8s-foo\"]\npath = \"../../k8s/ports/foo\"\n";
        assert_eq!(
            forbidden_package_deps(quoted_named, "k8s-"),
            vec!["k8s-foo".to_owned()]
        );
        let quoted_workspace = "[dependencies]\n\"k8s-foo\".workspace = true\n";
        assert_eq!(
            forbidden_package_deps(quoted_workspace, "k8s-"),
            vec!["k8s-foo".to_owned()]
        );

        let clean = "[dependencies]\nserde = { workspace = true }\nos-kernel = { path = \"../kernel\" }\n";
        assert!(forbidden_package_deps(clean, "k8s-").is_empty());

        assert_eq!(
            forbidden_buck_deps(
                "deps = [\n    \"//k8s/ports/foo:k8s-foo\",\n    \"//os/core/bar:os-bar\",\n]\n",
                "//k8s/"
            ),
            vec!["//k8s/ports/foo:k8s-foo".to_owned()]
        );

        let mut corpus = live_fixture();
        corpus.cross_seam_edges = vec![CrossSeamEdge {
            path: "os/core/fixture-domain/Cargo.toml".to_owned(),
            dependency: "k8s-foo".to_owned(),
        }];
        assert!(has_code(
            &evaluate(&corpus),
            FindingCode::CrossSeamDependency
        ));
    }

    #[test]
    fn scan_cross_seam_edges_reads_buck_v2_and_skips_binary_fixtures() {
        let root = std::env::temp_dir().join(format!(
            "r-doc-seam-buck-v2-binary-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let crate_dir = root.join("os/core/fixture-domain");
        fs::create_dir_all(crate_dir.join("testdata")).expect("fixture dirs");
        // Legitimate binary fixture must not force UTF-8 decode of the whole tree.
        fs::write(crate_dir.join("testdata/cert.der"), [0xff, 0xfe, 0x00, 0x80])
            .expect("binary fixture");
        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"fixture-domain\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
        )
        .expect("cargo");
        // Clean BUCK shadowed by a BUCK.v2 that crosses the seam — INV-1 must see BUCK.v2.
        fs::write(crate_dir.join("BUCK"), "rust_library(name = \"ok\", deps = [])\n")
            .expect("buck");
        fs::write(
            crate_dir.join("BUCK.v2"),
            "rust_library(\n    name = \"fixture\",\n    deps = [\"//k8s/ports/foo:k8s-foo\"],\n)\n",
        )
        .expect("buck.v2");
        let edges = scan_cross_seam_edges(&root).expect("scan");
        let _ = fs::remove_dir_all(&root);
        assert_eq!(
            edges,
            vec![CrossSeamEdge {
                path: "os/core/fixture-domain/BUCK.v2".to_owned(),
                dependency: "//k8s/ports/foo:k8s-foo".to_owned(),
            }]
        );
    }

    #[test]
    fn a_generated_leaf_without_a_registered_region_fails_closed() {
        let root = std::env::temp_dir().join("r-doc-generated-without-region-fixture");
        let path = root.join(REGENERABLE_REGIONS);
        fs::create_dir_all(path.parent().expect("the declaration has a parent directory"))
            .expect("the fixture tree is creatable");
        fs::write(
            &path,
            r#"{
              "regions": [],
              "origin_classification": {
                "leaves": [{"path": "k8s/ports/upstream-api", "origin": "generated"}],
                "census": {"k8s_leaves": 1, "os_leaves": 0, "total_leaves": 1}
              }
            }"#,
        )
        .expect("the fixture declaration is writable");
        let result = load_declared_leaves(&root);
        fs::remove_dir_all(&root).ok();
        assert_eq!(
            result.expect_err("generated without a region is malformed").code,
            "R-DOC-K8S-PORT-REGIONS-MALFORMED"
        );
    }

    #[test]
    fn regions_are_validated_even_when_every_leaf_is_first_party() {
        let root = std::env::temp_dir().join(format!(
            "r-doc-regions-independent-{}",
            std::process::id()
        ));
        let path = root.join(REGENERABLE_REGIONS);
        fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
        // Region covers a first-party leaf, has no producer, incomplete receipt, and would
        // overlap a second region — all fail_closed conditions independent of `generated`.
        fs::write(
            &path,
            r#"{
              "regions": [
                {
                  "region_id": "ports",
                  "path_prefix": "k8s/ports",
                  "origin": "generated",
                  "receipt": {"pin": "x"}
                },
                {
                  "region_id": "ports-nested",
                  "path_prefix": "k8s/ports/upstream-api",
                  "origin": "generated",
                  "producer": "build/port-engine",
                  "receipt": {
                    "pin": "x",
                    "snapshot_digest": "a",
                    "engine_digest": "b",
                    "rulepack_digest": "c",
                    "toolchain_digest": "d",
                    "formatter_digest": "e"
                  }
                }
              ],
              "origin_classification": {
                "leaves": [{"path": "k8s/ports/tenant-quota-api", "origin": "first-party"}],
                "census": {"k8s_leaves": 1, "os_leaves": 0, "total_leaves": 1}
              }
            }"#,
        )
        .expect("write");
        let result = load_declared_leaves(&root);
        fs::remove_dir_all(&root).ok();
        let err = result.expect_err("invalid region table must fail closed");
        assert_eq!(err.code, "R-DOC-K8S-PORT-REGIONS-MALFORMED");
        assert!(
            err.message.contains("producer") || err.message.contains("receipt"),
            "expected producer/receipt validation, got: {}",
            err.message
        );
    }

    #[test]
    fn overlapping_region_prefixes_fail_closed() {
        let root = std::env::temp_dir().join(format!(
            "r-doc-regions-overlap-{}",
            std::process::id()
        ));
        let path = root.join(REGENERABLE_REGIONS);
        fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
        fs::write(
            &path,
            r#"{
              "regions": [
                {
                  "region_id": "ports",
                  "path_prefix": "k8s/ports",
                  "origin": "generated",
                  "producer": "build/port-engine",
                  "receipt": {
                    "pin": "x",
                    "snapshot_digest": "a",
                    "engine_digest": "b",
                    "rulepack_digest": "c",
                    "toolchain_digest": "d",
                    "formatter_digest": "e"
                  }
                },
                {
                  "region_id": "ports-nested",
                  "path_prefix": "k8s/ports/upstream-api",
                  "origin": "generated",
                  "producer": "build/port-engine",
                  "receipt": {
                    "pin": "x",
                    "snapshot_digest": "a",
                    "engine_digest": "b",
                    "rulepack_digest": "c",
                    "toolchain_digest": "d",
                    "formatter_digest": "e"
                  }
                }
              ],
              "origin_classification": {
                "leaves": [],
                "census": {"k8s_leaves": 0, "os_leaves": 0, "total_leaves": 0}
              }
            }"#,
        )
        .expect("write");
        let result = load_declared_leaves(&root);
        fs::remove_dir_all(&root).ok();
        let err = result.expect_err("overlapping prefixes are malformed");
        assert_eq!(err.code, "R-DOC-K8S-PORT-REGIONS-MALFORMED");
        assert!(
            err.message.contains("overlap"),
            "expected overlap message, got: {}",
            err.message
        );
    }

    #[test]
    fn first_party_leaf_inside_registered_region_fails_closed() {
        let root = std::env::temp_dir().join(format!(
            "r-doc-first-party-in-region-{}",
            std::process::id()
        ));
        let path = root.join(REGENERABLE_REGIONS);
        fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
        fs::write(
            &path,
            r#"{
              "regions": [
                {
                  "region_id": "ports",
                  "path_prefix": "k8s/ports",
                  "origin": "generated",
                  "producer": "build/port-engine",
                  "receipt": {
                    "pin": "x",
                    "snapshot_digest": "a",
                    "engine_digest": "b",
                    "rulepack_digest": "c",
                    "toolchain_digest": "d",
                    "formatter_digest": "e"
                  }
                }
              ],
              "origin_classification": {
                "leaves": [{"path": "k8s/ports/tenant-quota-api", "origin": "first-party"}],
                "census": {"k8s_leaves": 1, "os_leaves": 0, "total_leaves": 1}
              }
            }"#,
        )
        .expect("write");
        let result = load_declared_leaves(&root);
        fs::remove_dir_all(&root).ok();
        let err = result.expect_err("first-party under a region is malformed");
        assert_eq!(err.code, "R-DOC-K8S-PORT-REGIONS-MALFORMED");
        assert!(
            err.message.contains("first-party"),
            "expected first-party coverage refusal, got: {}",
            err.message
        );
    }
}
