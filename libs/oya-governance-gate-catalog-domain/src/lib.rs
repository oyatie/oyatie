//! Foundry gate-catalog canonical domain — single source of truth for the
//! `oya gate ...` command catalog that the downstream content-validation
//! gates (quality-lane, documentation-system, supply-chain) read as their
//! input data.
//!
//! Naming justification:
//! - Crate id `oya-governance-gate-catalog-domain` — `oya-` brand prefix
//!   (ADR-0017 / MFL-0011), `foundry` axis (per ADR-0107 family table),
//!   `gate-catalog` two-word subject (the gate-validate catalog), final
//!   segment `domain` (∈ ALLOWED_ROLES per canonical 12-value layer enum at
//!   `oya-governance-predictable-naming-kernel::ALLOWED_ROLES`, post-ADR-0565).
//! - Library identifier `oya_governance_gate_catalog_domain` —
//!   snake_case mirror (ADR-0105 v4 BNF §2.2).
//! - Public constants `AGGREGATED_VALIDATE_LANES` /
//!   `AGGREGATED_NON_GATE_COMMANDS` — SCREAMING_SNAKE_CASE per Rust style;
//!   names follow predicate-naming kernel verbs `aggregated_*` (collective
//!   noun, kernel-tier).
//! - Public function `all_canonical_commands` — snake_case verb-phrase
//!   (predicate-naming kernel: descriptive returns slice/Vec are
//!   `*_commands` not `get_*`).
//!
//! Authority chain:
//! - audit `evidence/audits/shell-python-replacement-audit-2026-05-15.md`
//!   rows B-1, B-2, B-12 (the three transitional `.sh` files whose body is
//!   the content-validation input data being canonicalized here).
//! - source-of-truth lift: the catalog below mirrors
//!   `scripts/check.sh`'s ~50-command enumeration verbatim (order
//!   preserved for diff-readability) and the architecture-boundaries +
//!   pre-push contract surface; once landed, `scripts/check.sh` and the
//!   two sibling `.sh` wrappers become deletable (audit row B-12 follow-up,
//!   the `.sh-removal` chain IP-E).
//!
//! Tier 1 (kernel-tier) per ADR-0083: pure data + small validators; no
//! filesystem, no subprocess, no network, no panics outside `cfg(test)`.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fmt;

/// Catalog of `oya gate validate <name>` subcommand names invoked by the
/// pre-merge gate aggregator (`oya gate run-all`). Each name is invoked
/// with NO extra arguments (defaults). Mirrors the ~50-line block in the
/// legacy `scripts/check.sh` whose body the downstream content-validation
/// gates read as their input data.
///
/// Order is preserved from the legacy script so a human diff against the
/// shell version stays readable during the `.sh-removal` chain landing.
pub const AGGREGATED_VALIDATE_LANES: &[&str] = &[
    "codeview-read-surface",
    "active-artifact-contract",
    "authority-cohesion",
    "cedar-fragment-coverage",
    "openapi-rest-route-parity",
    "claim-ceiling",
    "codeowners-mirror",
    "cohesion",
    "data-class",
    "doc-catalog",
    "documentation-system",
    "adr-citation",
    "brand-residue",
    "no-grouping",
    "api-semver",
    "supply-chain",
    "cargo-prefix",
    "pre-push-contract",
    "freshness",
    "quality-lanes",
    "honest-claims",
    "aspirational-enforcement",
    "banned-primitives",
    "workspace-hygiene",
    "design-spec-maturity-claims",
    "hyperscaler-arch-invariants",
    "hyperscaler-maturity-claims",
    "platform-substrate-defaults",
    "loop-recovery-patterns",
    "foundation-bypass",
    "audit-chain-replay",
    "foundry-capability-schema",
    "foundry-eval",
    "cross-tenant-access-fuzz",
    "vendor-contract-recency",
    "mobile-native",
    "glossary-cross-doc-coverage",
    "glossary-vocabulary",
    "placeholder-debt",
    "retired-vocabulary",
    "protection-context-match",
    "dependency-seam",
    "license-policy",
    "http-stack",
    "workspace-topology",
    "plane-class",
    "raci-team-coverage",
    "readme-doc-coverage",
    "runbook-index-resolves",
    "runbook-freshness",
    "release-evidence-pack",
    "slo-coverage",
    "architecture-boundaries",
    "master-plan-completion",
    "product-index",
    "product-prd-json",
    "stage0-prereqs",
    // ADR-0110 changeset state machine: monotonic state progression + closed
    // status enum, validated against registry/vcs/changeset-event-log.json
    // (impl: oya-dev-cli/src/changeset_state_gates.rs).
    "changeset-state-monotonicity",
    "changeset-state-enum-closed",
    // PR #143 Fix-D strict gates.
    "high-risk-auto-decision-refusal",
    "slsa-l3-evidence-grounded",
    // ADR-0145 enforcement gates (advisory / DEFERRED mode until strict
    // parsers land per registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-*).
    "otel-trace-propagation",
    "ontology-projection-coverage",
    "audit-chain-seal-coverage",
    // ADR-0148 / ADR-0182 / ADR-0183 / ADR-0184 / ADR-0185 layered
    // architecture and native client-stack discipline.
    "layered-architecture-discipline",
    "client-stack-discipline",
    // Tier-A hyperscaler pattern remediation (Fix-Agent-I, 2026-05-18).
    // Each is strict-mode (fail-closed).
    "idempotency-key-coverage",
    "cursor-pagination-coverage",
    "rpo-rto-coverage",
    "metric-cardinality",
    "event-schema-versioning",
    "id-discipline",
    "image-signing-discipline",
    // PR #143 Fix-M/N/R/S/T/U batches — advisory lanes (most report-only).
    // Vendor-lockin is the sole BLOCKER; remainder are advisory until promotion
    // criteria in registry/quality/lanes.yaml#promotion_target trigger.
    "vendor-lockin-discipline",
    "authz-tier-discipline",
    "tenant-cost-labels-coverage",
    "backup-retention-discipline",
    "vector-store-discipline",
    "olap-tier-discipline",
    "wasm-runtime-discipline",
    "iac-tier-discipline",
    "cloud-iac-module-catalog",
    "cloud-iac-gitops-evidence",
    "cloud-iac-helm-chart-signed-image-wiring",
    "cloud-iac-kubewarden-admission-policy",
    "cloud-iac-cell-topology",
    "cloud-iac-opentofu-validation",
    "cloud-iac-module-provenance",
    "cloud-iac-module-provider-requirements",
    "cloud-iac-module-release-index",
    "cloud-iac-module-archive",
    "cloud-iac-module-registry-protocol",
    "cloud-iac-provider-readiness",
    "cloud-iac-provider-lockfile",
    "cloud-iac-provider-signature-review",
    "a11y-discipline",
    "i18n-coverage",
    "compliance-evidence-coverage",
    "realtime-transport-tier",
    // ADR-0364 generative-masterplan governance lanes.
    "adr-planning-completeness",
    "masterplan-drift",
    // #6b: ADR supersession back-link integrity — fails on any one-directional
    // supersedes/superseded_by pair (ADR-0083 Tier-3 panic-free).
    "adr-supersession-consistency",
    // ADR-0388: doc-axis convention enforcement — status casing, shadow ideas,
    // docs proliferation, catalog/manifest drift.
    "doc-axis",
    // M02b/P22 exit-gate quality lanes — check crates and `gate validate`
    // dispatch arms exist; wired here so `oya gate run-all` dispatches them.
    // ADR-0231 §"Plane 8 — Statelessness + shardability".
    "statelessness",
    "shardability",
    // ADR-0062 §"performance budgets" + §"competitive benchmark".
    "perf-budget",
    "benchmark",
];

pub const BANNED_PRIMITIVES_COMMAND_LOG_CORPUS_ROOT: &str =
    "registry/governance-corpora/banned-primitives";

/// Multispectrum evidence bundle that promotes the dependency-seam lane from
/// advisory/default-offline to CI-required, fail-closed governance.
pub const DEPENDENCY_SEAM_EVIDENCE: &str =
    "evidence/multispectrum/cs-p13-dependency-seam-1779166052.json";

/// Required non-cargo hosted-status preflight commands that are not pure
/// `oya gate validate <name>` lanes.
///
/// `oya verify --ci-required` owns the cargo required checks (fmt/check/clippy/
/// nextest) before invoking `gate run-all`, so this list deliberately contains
/// only the remaining non-cargo protection proof. That avoids replaying the
/// expensive workspace cargo mirror twice in one local CI-required run.
///
/// ADR-0363 retired the oya-vcs admission/provider-execution checks from the
/// required merge substrate; governance now rides plain git plus oya gate/verify
/// and the `oya-pr-review` context. Keep this list aligned with the live dev
/// branch-protection contexts to avoid local CI replaying retired checks.
pub const CI_REQUIRED_PREFLIGHT_COMMANDS: &[&str] =
    &["bash scripts/github-actions-required-secrets-check.sh"];

/// Catalog of non-`gate validate` commands the legacy `scripts/check.sh`
/// wired into the pre-merge gate sequence. These cover:
/// 1. Cargo toolchain commands (fmt/check/clippy/audit/deny/machete/nextest).
/// 2. Specialty `cargo run -p <tool>` invocations that aren't `gate validate`.
/// 3. The architecture-boundaries Rust port (now `gate validate
///    architecture-boundaries`, mirrored here for downstream lookups).
/// 4. The `repoctl pre-push --verify-contract` contract check.
/// 5. The doc-pipeline subcommands the documentation-system gate must
///    confirm are wired.
/// 6. The catalog-validate subcommand the doc-pipeline lint step needs.
/// 7. The typescript-workspace lanes (typecheck + test) the master gate
///    expects to be wired even though they're not under `gate run-all`.
/// 8. The retired-but-canonical-in-body `cargo audit` / `cargo deny check`
///    tokens required by `oya-check-supply-chain` evidence detection.
///
/// Naming-justification: `_non_gate_` describes the negative axis
/// against `AGGREGATED_VALIDATE_LANES` — every entry that the legacy
/// script body emitted but that is NOT a `gate validate` subcommand.
///
/// Strings are preserved with their full `buck2 run //marketplace/facade/dev-cli:oya -- …`
/// prefix so the downstream content-validation gates (which historically
/// did `check_script.contains(<canonical_command>)`) keep matching their
/// expected patterns against the unified catalog.
pub const AGGREGATED_NON_GATE_COMMANDS: &[&str] = &[
    // Toolchain primitives.
    "cargo fmt --all -- --check",
    "cargo check --workspace --all-targets --keep-going",
    "cargo clippy --workspace --all-targets --keep-going -- -D warnings",
    "cargo machete",
    "cargo audit",
    "cargo nextest run --workspace --no-fail-fast",
    "cargo deny check",
    // Demo and catalog.
    "buck2 run //marketplace/facade/dev-cli:oya -- demo",
    "buck2 run //marketplace/facade/dev-cli:oya -- catalog validate",
    // Doc pipeline (active steps in registry/docs/pipeline.tsv).
    "buck2 run //marketplace/facade/dev-cli:oya -- doc mdbook",
    "buck2 run //marketplace/facade/dev-cli:oya -- doc openapi",
    "buck2 run //marketplace/facade/dev-cli:oya -- doc rustdoc",
    "buck2 run //marketplace/facade/dev-cli:oya -- doc adr-index",
    // TypeScript workspace lanes (parameterized; not under run-all).
    "buck2 run //marketplace/facade/dev-cli:oya -- gate validate typescript-workspace --lane typecheck",
    "buck2 run //marketplace/facade/dev-cli:oya -- gate validate typescript-workspace --lane test",
    // Active-artifact + cedar-fragment + openapi-route emit-evidence lanes.
    "buck2 run //marketplace/facade/dev-cli:oya -- gate validate active-artifact-contract --emit-evidence evidence/active-artifact-contract-lane-run.json --emit-graph-edges registry/graph/active-artifact-contract-edges.json",
    "buck2 run //marketplace/facade/dev-cli:oya -- gate validate cedar-fragment-coverage --emit-evidence evidence/cedar-fragment-coverage-lane-run.json",
    "buck2 run //marketplace/facade/dev-cli:oya -- gate validate openapi-rest-route-parity --emit-evidence evidence/openapi-rest-route-parity-lane-run.json",
    // Release-supply-chain phased lane (separate from default supply-chain).
    "buck2 run //marketplace/facade/dev-cli:oya -- gate validate release-supply-chain --phase pre-release",
    "buck2 run //marketplace/facade/dev-cli:oya -- gate validate supply-chain --require-adr0039-evidence",
    // ADR-0221 governance hook-efficacy CI contexts.
    "bash tools/governance/adr-0221-governance-gates.sh vacuous-green",
    "bash tools/governance/adr-0221-governance-gates.sh orphan-citation",
    "bash tools/governance/adr-0221-governance-gates.sh version-pin",
    "bash tools/governance/adr-0221-governance-gates.sh buildability-line-count",
    // Local verification + dedicated tool entry points.
    "buck2 run //marketplace/facade/dev-cli:oya -- verify --ci-required",
    "cargo run -q -p oya-governance-purpose-audit-app",
    "cargo run -p oya-vcs-merge-queue-fix-loop-app -- --gc-staging-refs --max-age-seconds 3600",
    "scripts/check-sequential-pr-merge-conflicts.sh --base-branch dev --start-pr 111",
    "scripts/repair-sequential-pr-queue.sh --base-branch dev --start-pr 111 --target-pr 111",
    "scripts/trigger-next-queue-automerge.sh --base-branch dev --start-pr 111 --dry-run",
];

/// Concatenated canonical command catalog. Provides downstream gates a
/// single authoritative input vector that replaces the file-body read of
/// `scripts/check.sh`.
///
/// Lookup semantics: each downstream content-validation gate
/// (`oya-check-supply-chain`, `oya-check-documentation-system`,
/// `oya-check-quality-lane`) previously did
/// `check_script_contents.contains(<command>)`. With this catalog, the
/// canonical lookup becomes `wired_commands.iter().any(|wired|
/// wired.contains(<command>))` — preserving the substring-tolerant
/// matching the original gates use so partial / parameterized commands
/// (e.g. trailing `--require-adr0039-evidence`) still match.
#[must_use]
pub fn all_canonical_commands() -> Vec<&'static str> {
    let mut commands =
        Vec::with_capacity(AGGREGATED_VALIDATE_LANES.len() + AGGREGATED_NON_GATE_COMMANDS.len());
    for lane in AGGREGATED_VALIDATE_LANES {
        commands.push(*lane);
    }
    for command in AGGREGATED_NON_GATE_COMMANDS {
        commands.push(*command);
    }
    commands
}

/// Render the canonical catalog as a single newline-joined string
/// suitable for substring-tolerant lookups by downstream gates that
/// historically did `check_script.contains(<command>)`. The first line
/// is each `gate validate <name>` invocation in canonical form so
/// `contains("gate validate <name>")` matches; subsequent lines are the
/// non-gate commands verbatim.
#[must_use]
pub fn all_canonical_commands_rendered() -> String {
    let mut rendered = String::new();
    for lane in AGGREGATED_VALIDATE_LANES {
        rendered.push_str(&canonical_gate_validate_command(lane));
        rendered.push('\n');
    }
    for command in AGGREGATED_NON_GATE_COMMANDS {
        rendered.push_str(command);
        rendered.push('\n');
    }
    rendered
}

#[must_use]
pub fn canonical_gate_validate_command(lane: &str) -> String {
    let mut command = format!("buck2 run //marketplace/facade/dev-cli:oya -- gate validate {lane}");
    if lane == "banned-primitives" {
        command.push_str(" --require-command-log-corpus --command-log-root ");
        command.push_str(BANNED_PRIMITIVES_COMMAND_LOG_CORPUS_ROOT);
    } else if lane == "dependency-seam" {
        command.push_str(" --repo-root . --evidence ");
        command.push_str(DEPENDENCY_SEAM_EVIDENCE);
        command.push_str(" --online-audit --severity error");
    }
    command
}

// -----------------------------------------------------------------------
// Lane input-path globs — affected-scope selection data table
// -----------------------------------------------------------------------

/// Declares which paths in the repository trigger re-evaluation of a lane.
///
/// # Variants
///
/// * `Global` — the lane must always run regardless of what changed (used for
///   cross-cutting concerns such as `supply-chain` or `license-policy`).
/// * `Globs` — a non-empty list of path-glob patterns; the lane is selected
///   when at least one changed file matches at least one glob.  An empty
///   `Globs(&[])` is treated conservatively as `Global` by `lanes_for_changed`.
#[derive(Debug)]
pub enum LaneInputs {
    /// Always selected; runs on every changeset.
    Global,
    /// Selected when at least one changed file matches at least one glob.
    Globs(&'static [&'static str]),
}

/// Pure glob matcher covering the three canonical shapes used in
/// `LANE_INPUT_GLOBS`.
///
/// Supported shapes (evaluated in order):
/// 1. `dir/**`     — the path starts with `dir/` (directory subtree).
/// 2. `dir/`       — trailing slash; same prefix check as `dir/**`.
/// 3. `**/*.ext`   — any path whose file-name component ends with `.ext`.
/// 4. `*.ext`      — file in the root (no `/`) ending with `.ext`.
/// 5. Exact string — the path equals the glob literally.
///
/// Any other shape (bracket expressions, mid-path `?`, etc.) returns `false`
/// to stay conservative — the lane falls back to the unmapped-always-selected
/// path via `LaneInputs::Global`.
///
/// Leading `./` on `path` is normalised away before matching.
#[must_use]
pub(crate) fn path_glob_matches(path: &str, glob: &str) -> bool {
    // Normalise leading "./" from VCS-supplied relative paths.
    let path = path.strip_prefix("./").unwrap_or(path);

    // Shape 1: "prefix/**"
    if let Some(dir) = glob.strip_suffix("/**") {
        let prefix = format!("{dir}/");
        return path.starts_with(prefix.as_str()) || path == dir;
    }

    // Shape 2: trailing "/" — directory prefix
    if glob.ends_with('/') && !glob.starts_with("**") {
        return path.starts_with(glob);
    }

    // Shape 3: "**/*.ext" — any depth, specific extension
    if let Some(pattern) = glob.strip_prefix("**/") {
        // pattern is now "*.ext" or similar; must have no further wildcards
        if let Some(ext) = pattern.strip_prefix("*.")
            && !ext.contains(['*', '?', '[', ']'])
        {
            let suffix = format!(".{ext}");
            return path.ends_with(suffix.as_str());
        }
        return false;
    }

    // Shape 4: "*.ext" — root-level file with extension
    if let Some(ext) = glob.strip_prefix("*.") {
        if !ext.contains(['*', '?', '[', ']']) && !path.contains('/') {
            let suffix = format!(".{ext}");
            return path.ends_with(suffix.as_str());
        }
        return false;
    }

    // Shape 5: exact match (no wildcards)
    if !glob.contains(['*', '?', '[', ']']) {
        return path == glob;
    }

    // Unknown shape: conservative false.
    false
}

/// Maps each governance lane (from `AGGREGATED_VALIDATE_LANES`) to the
/// repository paths that should trigger it.  Lanes absent from this table
/// are treated as `Global` by `lanes_for_changed` (conservative fallback).
///
/// Keys must be a subset of `AGGREGATED_VALIDATE_LANES`; duplicates are
/// rejected by the unit tests.
pub const LANE_INPUT_GLOBS: &[(&str, LaneInputs)] = &[
    // ── Architecture / ADR surface ──────────────────────────────────────────
    (
        "architecture-boundaries",
        LaneInputs::Globs(&[
            "docs/decisions/**",
            "crates/**",
            "microservices/**",
            "specs/**",
        ]),
    ),
    (
        "adr-citation",
        LaneInputs::Globs(&["docs/decisions/**", "crates/**", "microservices/**"]),
    ),
    (
        "adr-supersession-consistency",
        LaneInputs::Globs(&["docs/decisions/**"]),
    ),
    (
        "adr-planning-completeness",
        LaneInputs::Globs(&["docs/decisions/**", "specs/**"]),
    ),
    (
        "masterplan-drift",
        LaneInputs::Globs(&["docs/decisions/**", "specs/**"]),
    ),
    // ── Supply-chain / licensing ─────────────────────────────────────────────
    ("supply-chain", LaneInputs::Global),
    ("license-policy", LaneInputs::Global),
    (
        "dependency-seam",
        LaneInputs::Globs(&["Cargo.toml", "Cargo.lock", "crates/**"]),
    ),
    // ── Cargo / workspace hygiene ────────────────────────────────────────────
    (
        "cargo-prefix",
        LaneInputs::Globs(&["crates/**", "microservices/**", "Cargo.toml"]),
    ),
    (
        "workspace-hygiene",
        LaneInputs::Globs(&["Cargo.toml", "Cargo.lock", "crates/**", "microservices/**"]),
    ),
    ("banned-primitives", LaneInputs::Global),
    // ── Documentation ────────────────────────────────────────────────────────
    (
        "documentation-system",
        LaneInputs::Globs(&["docs/**", "registry/docs/**", "*.md"]),
    ),
    (
        "doc-catalog",
        LaneInputs::Globs(&["docs/**", "registry/docs/**"]),
    ),
    ("doc-axis", LaneInputs::Globs(&["docs/**", "*.md"])),
    (
        "readme-doc-coverage",
        LaneInputs::Globs(&["*.md", "crates/**", "microservices/**"]),
    ),
    (
        "runbook-index-resolves",
        LaneInputs::Globs(&["docs/**", "registry/**"]),
    ),
    ("runbook-freshness", LaneInputs::Globs(&["docs/**"])),
    (
        "glossary-cross-doc-coverage",
        LaneInputs::Globs(&["docs/**", "*.md"]),
    ),
    (
        "glossary-vocabulary",
        LaneInputs::Globs(&["docs/**", "*.md"]),
    ),
    // ── OpenAPI / API surface ────────────────────────────────────────────────
    (
        "openapi-rest-route-parity",
        LaneInputs::Globs(&["contracts/**", "crates/**", "microservices/**"]),
    ),
    (
        "api-semver",
        LaneInputs::Globs(&["contracts/**", "crates/**"]),
    ),
    (
        "active-artifact-contract",
        LaneInputs::Globs(&["contracts/**", "registry/**"]),
    ),
    // ── Cloud / IaC ──────────────────────────────────────────────────────────
    (
        "cloud-iac-module-catalog",
        LaneInputs::Globs(&["infra/**", "microservices/observability/iac/**"]),
    ),
    (
        "cloud-iac-gitops-evidence",
        LaneInputs::Globs(&["infra/**", "microservices/observability/iac/**"]),
    ),
    (
        "cloud-iac-helm-chart-signed-image-wiring",
        LaneInputs::Globs(&["infra/**", "microservices/observability/iac/**"]),
    ),
    (
        "cloud-iac-kubewarden-admission-policy",
        LaneInputs::Globs(&["infra/**", "microservices/observability/iac/**"]),
    ),
    ("cloud-iac-cell-topology", LaneInputs::Globs(&["infra/**"])),
    (
        "cloud-iac-opentofu-validation",
        LaneInputs::Globs(&["infra/**"]),
    ),
    (
        "cloud-iac-module-provenance",
        LaneInputs::Globs(&["infra/**"]),
    ),
    (
        "cloud-iac-module-provider-requirements",
        LaneInputs::Globs(&["infra/**"]),
    ),
    (
        "cloud-iac-module-release-index",
        LaneInputs::Globs(&["infra/**", "registry/**"]),
    ),
    ("cloud-iac-module-archive", LaneInputs::Globs(&["infra/**"])),
    (
        "cloud-iac-module-registry-protocol",
        LaneInputs::Globs(&["infra/**", "registry/**"]),
    ),
    (
        "cloud-iac-provider-readiness",
        LaneInputs::Globs(&["infra/**"]),
    ),
    (
        "cloud-iac-provider-lockfile",
        LaneInputs::Globs(&["infra/**"]),
    ),
    (
        "cloud-iac-provider-signature-review",
        LaneInputs::Globs(&["infra/**"]),
    ),
    (
        "iac-tier-discipline",
        LaneInputs::Globs(&["infra/**", "microservices/observability/iac/**"]),
    ),
    // ── SLO / observability ──────────────────────────────────────────────────
    (
        "slo-coverage",
        LaneInputs::Globs(&["**/*.openslo.yaml", "microservices/**"]),
    ),
    // ── Security / Cedar / authz ─────────────────────────────────────────────
    (
        "cedar-fragment-coverage",
        LaneInputs::Globs(&["registry/cedar/**", "crates/**", "microservices/**"]),
    ),
    (
        "authz-tier-discipline",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    ("cross-tenant-access-fuzz", LaneInputs::Global),
    ("high-risk-auto-decision-refusal", LaneInputs::Global),
    (
        "slsa-l3-evidence-grounded",
        LaneInputs::Globs(&["evidence/**", "registry/**"]),
    ),
    (
        "image-signing-discipline",
        LaneInputs::Globs(&["infra/**", "microservices/**"]),
    ),
    // ── Microservice / hyperscaler patterns ──────────────────────────────────
    (
        "hyperscaler-arch-invariants",
        LaneInputs::Globs(&["crates/**", "microservices/**", "docs/decisions/**"]),
    ),
    (
        "hyperscaler-maturity-claims",
        LaneInputs::Globs(&["crates/**", "microservices/**", "docs/decisions/**"]),
    ),
    (
        "platform-substrate-defaults",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "layered-architecture-discipline",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "client-stack-discipline",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "idempotency-key-coverage",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "cursor-pagination-coverage",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "rpo-rto-coverage",
        LaneInputs::Globs(&["crates/**", "microservices/**", "infra/**"]),
    ),
    (
        "metric-cardinality",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "event-schema-versioning",
        LaneInputs::Globs(&["crates/**", "microservices/**", "contracts/**"]),
    ),
    (
        "id-discipline",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "loop-recovery-patterns",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "http-stack",
        LaneInputs::Globs(&["crates/**", "microservices/**", "Cargo.toml"]),
    ),
    // ── Data / governance policy ─────────────────────────────────────────────
    (
        "data-class",
        LaneInputs::Globs(&["crates/**", "microservices/**", "registry/**"]),
    ),
    (
        "plane-class",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "cohesion",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "authority-cohesion",
        LaneInputs::Globs(&["docs/decisions/**", "crates/**", "microservices/**"]),
    ),
    (
        "claim-ceiling",
        LaneInputs::Globs(&["crates/**", "microservices/**", "docs/decisions/**"]),
    ),
    (
        "codeview-read-surface",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "honest-claims",
        LaneInputs::Globs(&["crates/**", "microservices/**", "docs/**"]),
    ),
    (
        "aspirational-enforcement",
        LaneInputs::Globs(&["docs/**", "specs/**", "crates/**"]),
    ),
    (
        "design-spec-maturity-claims",
        LaneInputs::Globs(&["docs/**", "specs/**"]),
    ),
    ("no-grouping", LaneInputs::Global),
    ("brand-residue", LaneInputs::Global),
    ("retired-vocabulary", LaneInputs::Global),
    ("placeholder-debt", LaneInputs::Global),
    // ── PR / changeset / release ─────────────────────────────────────────────
    (
        "changeset-state-monotonicity",
        LaneInputs::Globs(&["registry/vcs/changeset-event-log.json"]),
    ),
    (
        "changeset-state-enum-closed",
        LaneInputs::Globs(&["registry/vcs/changeset-event-log.json"]),
    ),
    (
        "release-evidence-pack",
        LaneInputs::Globs(&["evidence/**", "registry/**"]),
    ),
    (
        "vendor-contract-recency",
        LaneInputs::Globs(&["registry/**", "Cargo.lock"]),
    ),
    // ── Quality / CI ─────────────────────────────────────────────────────────
    (
        "quality-lanes",
        LaneInputs::Globs(&["registry/quality/**", "crates/**"]),
    ),
    ("pre-push-contract", LaneInputs::Global),
    (
        "codeowners-mirror",
        LaneInputs::Globs(&["CODEOWNERS", ".github/**", "crates/**", "microservices/**"]),
    ),
    ("stage0-prereqs", LaneInputs::Global),
    (
        "master-plan-completion",
        LaneInputs::Globs(&["docs/decisions/**", "specs/**"]),
    ),
    (
        "product-index",
        LaneInputs::Globs(&["docs/**", "specs/**", "registry/**"]),
    ),
    (
        "product-prd-json",
        LaneInputs::Globs(&["docs/**", "specs/**"]),
    ),
    (
        "raci-team-coverage",
        LaneInputs::Globs(&["docs/**", "registry/**"]),
    ),
    // ── OTel / audit ─────────────────────────────────────────────────────────
    (
        "otel-trace-propagation",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "ontology-projection-coverage",
        LaneInputs::Globs(&["registry/**", "docs/**"]),
    ),
    (
        "audit-chain-replay",
        LaneInputs::Globs(&["evidence/**", "registry/**"]),
    ),
    (
        "audit-chain-seal-coverage",
        LaneInputs::Globs(&["evidence/**", "registry/**"]),
    ),
    (
        "foundry-capability-schema",
        LaneInputs::Globs(&["registry/**", "specs/**"]),
    ),
    (
        "foundry-eval",
        LaneInputs::Globs(&["registry/**", "specs/**"]),
    ),
    // ── Specialty discipline ─────────────────────────────────────────────────
    ("vendor-lockin-discipline", LaneInputs::Global),
    (
        "tenant-cost-labels-coverage",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "backup-retention-discipline",
        LaneInputs::Globs(&["infra/**", "docs/**"]),
    ),
    (
        "vector-store-discipline",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "olap-tier-discipline",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "wasm-runtime-discipline",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "a11y-discipline",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "i18n-coverage",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "compliance-evidence-coverage",
        LaneInputs::Globs(&["evidence/**", "registry/**"]),
    ),
    (
        "realtime-transport-tier",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "mobile-native",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "protection-context-match",
        LaneInputs::Globs(&["crates/**", "microservices/**", "registry/cedar/**"]),
    ),
    ("foundation-bypass", LaneInputs::Global),
    // ── M02b/P22 quality lanes (statelessness / shardability / perf-budget / benchmark) ──
    (
        "statelessness",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "shardability",
        LaneInputs::Globs(&["crates/**", "microservices/**"]),
    ),
    (
        "perf-budget",
        LaneInputs::Globs(&["docs/**", "specs/**", "crates/**", "microservices/**"]),
    ),
    (
        "benchmark",
        LaneInputs::Globs(&["docs/**", "specs/**", "crates/**", "microservices/**"]),
    ),
];

/// Returns the subset of `AGGREGATED_VALIDATE_LANES` that should be run given
/// the set of changed file paths.
///
/// # Selection algorithm
///
/// For each lane in `AGGREGATED_VALIDATE_LANES` (in catalog order):
/// - If the lane has no entry in `LANE_INPUT_GLOBS`, it is treated as
///   `Global` (always selected).
/// - If the lane is mapped to `LaneInputs::Global`, it is always selected.
/// - If the lane is mapped to `LaneInputs::Globs(&[])` (empty list), it is
///   treated conservatively as `Global` (always selected).
/// - If the lane is mapped to `LaneInputs::Globs(globs)` with at least one
///   glob, it is selected when at least one path in `changed` matches at
///   least one glob.
///
/// When `changed` is empty the full catalog is returned (conservative).
///
/// The output is deduplicated and preserves catalog order.
#[must_use]
pub fn lanes_for_changed(changed: &[&str]) -> Vec<&'static str> {
    // Empty input → return full catalog.
    if changed.is_empty() {
        return AGGREGATED_VALIDATE_LANES.to_vec();
    }

    // Build lookup: lane name → LaneInputs reference.
    use std::collections::HashMap;
    let glob_map: HashMap<&str, &LaneInputs> =
        LANE_INPUT_GLOBS.iter().map(|(k, v)| (*k, v)).collect();

    let mut result = Vec::with_capacity(AGGREGATED_VALIDATE_LANES.len());
    for lane in AGGREGATED_VALIDATE_LANES {
        let selected = match glob_map.get(lane) {
            // Not in table → conservative Global.
            None => true,
            Some(LaneInputs::Global) => true,
            // Empty globs list → conservative Global; otherwise selected
            // when any changed file matches any glob.
            Some(LaneInputs::Globs(globs)) => {
                globs.is_empty()
                    || changed
                        .iter()
                        .any(|path| globs.iter().any(|g| path_glob_matches(path, g)))
            }
        };
        if selected {
            result.push(*lane);
        }
    }
    result
}

/// Tier 1 error surface. Currently empty (the catalog is static-data;
/// integrity is guaranteed by the unit tests below). The variant
/// `EmptyCatalog` is reserved for future runtime-callers that may want
/// to refuse an empty catalog without re-implementing the check.
///
/// `non_exhaustive` enables additive evolution without breaking the
/// Tier 1 surface contract (ADR-0083 §"Public-error stability").
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum GateCatalogError {
    /// Returned by `assert_non_empty` if either constant list is empty.
    EmptyCatalog { list_name: &'static str },
    /// Returned by `assert_unique` if a duplicate entry is detected
    /// (defensive: the unit tests guarantee uniqueness at build time).
    DuplicateEntry { entry: String },
}

impl fmt::Display for GateCatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCatalog { list_name } => {
                write!(formatter, "gate catalog list `{list_name}` is empty")
            }
            Self::DuplicateEntry { entry } => {
                write!(formatter, "gate catalog has duplicate entry `{entry}`")
            }
        }
    }
}

impl std::error::Error for GateCatalogError {}

/// Defensive runtime guard: errors out if a downstream caller hands an
/// empty `&[&str]` where the catalog should be non-empty. Intended to be
/// the canonical replacement for the old downstream-gate `if
/// commands.is_empty()` defensive checks.
///
/// # Errors
///
/// Returns [`GateCatalogError::EmptyCatalog`] when `commands` is empty.
pub fn assert_non_empty(
    commands: &[&str],
    list_name: &'static str,
) -> Result<(), GateCatalogError> {
    if commands.is_empty() {
        Err(GateCatalogError::EmptyCatalog { list_name })
    } else {
        Ok(())
    }
}

/// Defensive runtime guard: errors out if any entry in `commands`
/// repeats. The static catalog uniqueness is unit-tested at compile-time
/// of the test target, but this function exists for callers that build
/// dynamic catalogs (e.g. layered packs) on top of this domain.
///
/// # Errors
///
/// Returns [`GateCatalogError::DuplicateEntry`] when any entry repeats.
pub fn assert_unique(commands: &[&str]) -> Result<(), GateCatalogError> {
    let mut seen = BTreeSet::new();
    for command in commands {
        if !seen.insert(*command) {
            return Err(GateCatalogError::DuplicateEntry {
                entry: (*command).into(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregated_validate_lanes_is_non_empty() {
        assert!(!AGGREGATED_VALIDATE_LANES.is_empty());
        assert!(AGGREGATED_VALIDATE_LANES.len() >= 30);
    }

    #[test]
    fn aggregated_non_gate_commands_is_non_empty() {
        assert!(!AGGREGATED_NON_GATE_COMMANDS.is_empty());
        assert!(AGGREGATED_NON_GATE_COMMANDS.len() >= 15);
    }

    #[test]
    fn aggregated_validate_lanes_entries_unique() {
        assert_unique(AGGREGATED_VALIDATE_LANES).expect("validate lanes must be unique");
    }

    #[test]
    fn aggregated_validate_lanes_contains_freshness_gate() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"freshness"));
    }

    #[test]
    fn aggregated_non_gate_commands_entries_unique() {
        assert_unique(AGGREGATED_NON_GATE_COMMANDS).expect("non-gate commands must be unique");
    }

    #[test]
    fn all_canonical_commands_concatenates_both_lists() {
        let commands = all_canonical_commands();
        assert_eq!(
            commands.len(),
            AGGREGATED_VALIDATE_LANES.len() + AGGREGATED_NON_GATE_COMMANDS.len()
        );
        for lane in AGGREGATED_VALIDATE_LANES {
            assert!(
                commands.contains(lane),
                "validate lane `{lane}` missing from concatenated catalog"
            );
        }
        for command in AGGREGATED_NON_GATE_COMMANDS {
            assert!(
                commands.contains(command),
                "non-gate command `{command}` missing from concatenated catalog"
            );
        }
    }

    #[test]
    fn rendered_form_contains_each_validate_lane_canonical_invocation() {
        let rendered = all_canonical_commands_rendered();
        for lane in AGGREGATED_VALIDATE_LANES {
            let expected =
                format!("buck2 run //marketplace/facade/dev-cli:oya -- gate validate {lane}");
            assert!(
                rendered.contains(&expected),
                "rendered catalog must contain `{expected}`"
            );
        }
        for command in AGGREGATED_NON_GATE_COMMANDS {
            assert!(
                rendered.contains(command),
                "rendered catalog must contain `{command}`"
            );
        }
    }

    #[test]
    fn rendered_form_contains_cargo_deny_check_token() {
        // supply-chain kernel checks for the literal "cargo deny check" token.
        let rendered = all_canonical_commands_rendered();
        assert!(rendered.contains("cargo deny check"));
        assert!(rendered.contains("cargo audit"));
    }

    #[test]
    fn rendered_form_contains_doc_pipeline_canonical_commands() {
        // documentation-system kernel checks for `buck2 run //marketplace/facade/dev-cli:oya -- doc <step>`
        // commands per registry/docs/pipeline.tsv.
        let rendered = all_canonical_commands_rendered();
        for step in ["mdbook", "openapi", "rustdoc", "adr-index"] {
            let expected = format!("buck2 run //marketplace/facade/dev-cli:oya -- doc {step}");
            assert!(
                rendered.contains(&expected),
                "rendered catalog must wire doc step `{step}`"
            );
        }
        assert!(
            rendered.contains("buck2 run //marketplace/facade/dev-cli:oya -- catalog validate")
        );
    }

    #[test]
    fn rendered_form_contains_pre_push_contract_check() {
        // oya-check-pre-push kernel asserts that `oya verify` is the
        // canonical local verification surface.
        let rendered = all_canonical_commands_rendered();
        assert!(
            rendered.contains("buck2 run //marketplace/facade/dev-cli:oya -- verify --ci-required")
        );
    }

    #[test]
    fn rendered_dependency_seam_lane_is_ci_strict() {
        let rendered = all_canonical_commands_rendered();
        assert!(rendered.contains(
            "buck2 run //marketplace/facade/dev-cli:oya -- gate validate dependency-seam --repo-root . --evidence evidence/multispectrum/cs-p13-dependency-seam-1779166052.json --online-audit --severity error"
        ));
    }

    #[test]
    fn ci_required_preflight_commands_include_only_non_cargo_extra_proofs() {
        assert_eq!(
            CI_REQUIRED_PREFLIGHT_COMMANDS,
            &["bash scripts/github-actions-required-secrets-check.sh"]
        );
        assert!(
            CI_REQUIRED_PREFLIGHT_COMMANDS
                .iter()
                .all(|command| { !command.starts_with("cargo ") && !command.contains("oya-vcs-") })
        );
    }

    #[test]
    fn rendered_form_contains_loop_recovery_patterns_lane() {
        let rendered = all_canonical_commands_rendered();
        assert!(rendered.contains(
            "buck2 run //marketplace/facade/dev-cli:oya -- gate validate loop-recovery-patterns"
        ));
    }

    #[test]
    fn rendered_form_contains_adr_0221_governance_gates() {
        let rendered = all_canonical_commands_rendered();
        for gate in [
            "vacuous-green",
            "orphan-citation",
            "version-pin",
            "buildability-line-count",
        ] {
            let expected = format!("bash tools/governance/adr-0221-governance-gates.sh {gate}");
            assert!(
                rendered.contains(&expected),
                "rendered catalog must wire ADR-0221 governance gate `{gate}`"
            );
        }
    }

    #[test]
    fn rendered_form_contains_hyperscaler_architecture_and_maturity_lanes() {
        let rendered = all_canonical_commands_rendered();
        assert!(rendered.contains(
            "buck2 run //marketplace/facade/dev-cli:oya -- gate validate workspace-hygiene"
        ));
        assert!(
            rendered
                .contains("buck2 run //marketplace/facade/dev-cli:oya -- gate validate design-spec-maturity-claims")
        );
        assert!(
            rendered
                .contains("buck2 run //marketplace/facade/dev-cli:oya -- gate validate hyperscaler-arch-invariants")
        );
        assert!(
            rendered
                .contains("buck2 run //marketplace/facade/dev-cli:oya -- gate validate hyperscaler-maturity-claims")
        );
    }

    #[test]
    fn rendered_form_contains_architecture_boundaries_lane() {
        let rendered = all_canonical_commands_rendered();
        assert!(rendered.contains(
            "buck2 run //marketplace/facade/dev-cli:oya -- gate validate architecture-boundaries"
        ));
    }

    #[test]
    fn rendered_form_contains_cloud_iac_kubewarden_admission_policy_lane() {
        let rendered = all_canonical_commands_rendered();
        assert!(rendered.contains(
            "buck2 run //marketplace/facade/dev-cli:oya -- gate validate cloud-iac-kubewarden-admission-policy"
        ));
    }

    #[test]
    fn assert_non_empty_accepts_non_empty_slice() {
        assert_eq!(assert_non_empty(&["x"], "test"), Ok(()));
    }

    #[test]
    fn assert_non_empty_rejects_empty_slice() {
        assert_eq!(
            assert_non_empty(&[], "demo"),
            Err(GateCatalogError::EmptyCatalog { list_name: "demo" })
        );
    }

    #[test]
    fn assert_unique_detects_duplicate_entry() {
        assert_eq!(
            assert_unique(&["a", "b", "a"]),
            Err(GateCatalogError::DuplicateEntry { entry: "a".into() })
        );
    }

    #[test]
    fn error_display_renders_human_message() {
        assert_eq!(
            format!("{}", GateCatalogError::EmptyCatalog { list_name: "lanes" }),
            "gate catalog list `lanes` is empty"
        );
        assert_eq!(
            format!(
                "{}",
                GateCatalogError::DuplicateEntry {
                    entry: "supply-chain".into()
                }
            ),
            "gate catalog has duplicate entry `supply-chain`"
        );
    }

    // -----------------------------------------------------------------------
    // Group 1: path_glob_matches — three-shape glob matcher
    // -----------------------------------------------------------------------

    #[test]
    fn path_glob_matches_exact_returns_true_for_identical_path() {
        assert!(path_glob_matches("Cargo.lock", "Cargo.lock"));
    }

    #[test]
    fn path_glob_matches_exact_returns_false_for_different_path() {
        assert!(!path_glob_matches("Cargo.toml", "Cargo.lock"));
    }

    #[test]
    fn path_glob_matches_dir_double_star_matches_file_under_directory() {
        assert!(path_glob_matches(
            "microservices/foo/src/lib.rs",
            "microservices/**"
        ));
    }

    #[test]
    fn path_glob_matches_dir_double_star_does_not_match_sibling_directory() {
        assert!(!path_glob_matches(
            "crates/foo/src/lib.rs",
            "microservices/**"
        ));
    }

    #[test]
    fn path_glob_matches_dir_trailing_slash_matches_file_under_directory() {
        assert!(path_glob_matches(
            "crates/oya-foo/src/lib.rs",
            "crates/oya-foo/"
        ));
    }

    #[test]
    fn path_glob_matches_dir_trailing_slash_does_not_match_peer_directory() {
        assert!(!path_glob_matches(
            "crates/oya-bar/src/lib.rs",
            "crates/oya-foo/"
        ));
    }

    #[test]
    fn path_glob_matches_double_star_ext_matches_deep_file_with_extension() {
        assert!(path_glob_matches(
            "microservices/obs/slos/latency.openslo.yaml",
            "**/*.openslo.yaml"
        ));
    }

    #[test]
    fn path_glob_matches_double_star_ext_does_not_match_different_extension() {
        assert!(!path_glob_matches(
            "microservices/obs/slos/latency.yaml",
            "**/*.openslo.yaml"
        ));
    }

    #[test]
    fn path_glob_matches_star_ext_matches_root_level_file() {
        assert!(path_glob_matches("README.md", "*.md"));
    }

    #[test]
    fn path_glob_matches_star_ext_does_not_match_non_matching_extension() {
        assert!(!path_glob_matches("README.txt", "*.md"));
    }

    #[test]
    fn path_glob_matches_normalises_leading_dot_slash() {
        // A changed path supplied with a leading "./" must match the same
        // glob as the normalised form.
        assert!(path_glob_matches("./Cargo.lock", "Cargo.lock"));
        assert!(path_glob_matches(
            "./microservices/foo/bar.rs",
            "microservices/**"
        ));
    }

    #[test]
    fn path_glob_matches_unknown_glob_shape_returns_false() {
        // A glob that is not one of the three supported shapes must return
        // false (non-matching), preserving safety at the lane level via the
        // unmapped-fallback rule.
        assert!(!path_glob_matches(
            "crates/foo/src/lib.rs",
            "crates/[a-z]*/src/*.rs"
        ));
    }

    // -----------------------------------------------------------------------
    // Group 2: LANE_INPUT_GLOBS table key validity
    // -----------------------------------------------------------------------

    #[test]
    fn lane_input_globs_every_key_is_a_member_of_aggregated_validate_lanes() {
        for (lane, _inputs) in LANE_INPUT_GLOBS {
            assert!(
                AGGREGATED_VALIDATE_LANES.contains(lane),
                "LANE_INPUT_GLOBS key `{lane}` is not in AGGREGATED_VALIDATE_LANES"
            );
        }
    }

    #[test]
    fn lane_input_globs_no_key_is_listed_twice() {
        let mut seen = std::collections::BTreeSet::new();
        for (lane, _inputs) in LANE_INPUT_GLOBS {
            assert!(
                seen.insert(*lane),
                "LANE_INPUT_GLOBS has duplicate key `{lane}`"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Group 3: lanes_for_changed base cases
    // -----------------------------------------------------------------------

    #[test]
    fn lanes_for_changed_empty_input_returns_full_catalog_in_order() {
        let result = lanes_for_changed(&[]);
        assert_eq!(
            result,
            AGGREGATED_VALIDATE_LANES.to_vec(),
            "empty changed list must return the full catalog"
        );
    }

    #[test]
    fn lanes_for_changed_matching_path_includes_matched_lane_and_all_unmapped_global_lanes() {
        // Spec criterion 3 (positive sub-case): a path that matches exactly one
        // explicitly-mapped lane's globs must yield that lane PLUS every
        // Global/unmapped lane, while excluding explicitly-mapped lanes the path
        // does NOT hit.
        //
        // `docs/decisions/ADR-9999-test.md` matches `adr-supersession-consistency`
        // (mapped solely to `docs/decisions/**`) but does NOT match `slo-coverage`
        // (mapped to `**/*.openslo.yaml` + `microservices/**`).
        let changed = ["docs/decisions/ADR-9999-test.md"];
        let result = lanes_for_changed(&changed);

        // (a) The matched lane is present.
        assert!(
            result.contains(&"adr-supersession-consistency"),
            "matched lane `adr-supersession-consistency` must be selected for {changed:?}"
        );

        // (b) An explicitly-mapped lane whose globs do NOT match is absent —
        //     proving the positive path still narrows (sanity check the chosen
        //     path truly does not hit slo-coverage's globs before asserting).
        let slo_globs: &[&str] = &["**/*.openslo.yaml", "microservices/**"];
        assert!(
            !slo_globs.iter().any(|g| path_glob_matches(changed[0], g)),
            "fixture invariant: chosen path must not match slo-coverage globs"
        );
        assert!(
            !result.contains(&"slo-coverage"),
            "non-matching explicitly-mapped lane `slo-coverage` must be excluded"
        );

        // (c) Every Global-marked and every unmapped lane is still present.
        let mapped_keys: std::collections::BTreeSet<&str> =
            LANE_INPUT_GLOBS.iter().map(|(k, _)| *k).collect();
        let global_keys: std::collections::BTreeSet<&str> = LANE_INPUT_GLOBS
            .iter()
            .filter_map(|(k, v)| matches!(v, LaneInputs::Global).then_some(*k))
            .collect();
        for lane in AGGREGATED_VALIDATE_LANES {
            let is_unmapped = !mapped_keys.contains(*lane);
            let is_global = global_keys.contains(*lane);
            if is_unmapped || is_global {
                assert!(
                    result.contains(lane),
                    "Global/unmapped lane `{lane}` must always be present in {changed:?}"
                );
            }
        }
    }

    #[test]
    fn lanes_for_changed_path_matching_no_mapped_lane_still_includes_unmapped_lanes() {
        // A path that matches no declared glob must still produce all
        // unmapped/Global lanes — it must never return an empty result.
        let result = lanes_for_changed(&["some/completely/unknown/path.xyz"]);
        assert!(
            !result.is_empty(),
            "result must be non-empty even when no mapped lane matches"
        );
        // Every unmapped lane must appear in the result.
        let mapped_keys: std::collections::BTreeSet<&str> =
            LANE_INPUT_GLOBS.iter().map(|(k, _)| *k).collect();
        for lane in AGGREGATED_VALIDATE_LANES {
            if !mapped_keys.contains(*lane) {
                assert!(
                    result.contains(lane),
                    "unmapped lane `{lane}` must be in result for any changed input"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // Group 4: conservative invariants
    // -----------------------------------------------------------------------

    #[test]
    fn lanes_for_changed_every_unmapped_lane_appears_for_any_input() {
        // Regression guard: iterate AGGREGATED_VALIDATE_LANES; every lane
        // absent from LANE_INPUT_GLOBS must always appear in the result.
        let mapped_keys: std::collections::BTreeSet<&str> =
            LANE_INPUT_GLOBS.iter().map(|(k, _)| *k).collect();
        let unmapped: Vec<&str> = AGGREGATED_VALIDATE_LANES
            .iter()
            .copied()
            .filter(|l| !mapped_keys.contains(*l))
            .collect();

        // Test with several representative changed-file lists.
        for changed in [
            vec!["Cargo.lock"],
            vec!["microservices/obs/src/main.rs"],
            vec!["docs/decisions/ADR-9999-test.md"],
            vec!["some/unknown/path.xyz"],
        ] {
            let result = lanes_for_changed(&changed);
            for lane in &unmapped {
                assert!(
                    result.contains(lane),
                    "unmapped lane `{lane}` missing from lanes_for_changed({changed:?})"
                );
            }
        }
    }

    #[test]
    fn lanes_for_changed_output_is_duplicate_free() {
        let result =
            lanes_for_changed(&["Cargo.lock", "microservices/obs/slos/latency.openslo.yaml"]);
        let mut seen = std::collections::BTreeSet::new();
        for lane in &result {
            assert!(
                seen.insert(*lane),
                "lanes_for_changed returned duplicate lane `{lane}`"
            );
        }
    }

    #[test]
    fn lanes_for_changed_output_preserves_catalog_order() {
        let result = lanes_for_changed(&["Cargo.lock"]);
        // Every element of result must appear in AGGREGATED_VALIDATE_LANES
        // and in the same relative order.
        let catalog_positions: std::collections::HashMap<&str, usize> = AGGREGATED_VALIDATE_LANES
            .iter()
            .enumerate()
            .map(|(i, l)| (*l, i))
            .collect();
        let mut prev_pos = 0usize;
        for lane in &result {
            let pos = catalog_positions[*lane];
            assert!(
                pos >= prev_pos,
                "output order violation: `{lane}` at catalog pos {pos} came after pos {prev_pos}"
            );
            prev_pos = pos;
        }
    }

    // -----------------------------------------------------------------------
    // Group 0: new quality-lane acceptance tests
    // -----------------------------------------------------------------------

    #[test]
    fn aggregated_validate_lanes_contains_all_four_quality_lanes() {
        for lane in ["statelessness", "shardability", "perf-budget", "benchmark"] {
            assert!(
                AGGREGATED_VALIDATE_LANES.contains(&lane),
                "quality lane `{lane}` must be present in AGGREGATED_VALIDATE_LANES"
            );
        }
    }

    #[test]
    fn lane_input_globs_contains_entries_for_all_four_quality_lanes() {
        let keys: std::collections::BTreeSet<&str> =
            LANE_INPUT_GLOBS.iter().map(|(k, _)| *k).collect();
        for lane in ["statelessness", "shardability", "perf-budget", "benchmark"] {
            assert!(
                keys.contains(lane),
                "LANE_INPUT_GLOBS must have an entry for quality lane `{lane}`"
            );
        }
    }

    #[test]
    fn lanes_for_changed_empty_globs_list_treated_as_global_always_selected() {
        // A lane mapped with LaneInputs::Globs(&[]) (empty list) must behave
        // like Global — always selected — not "matches nothing" (conservative
        // signal per spec §Edge cases).
        //
        // We verify this by constructing a result for a very specific path
        // and checking that any lane with an empty globs list appears.
        let result = lanes_for_changed(&["some/path/that/matches/nothing.xyz"]);
        for (lane, inputs) in LANE_INPUT_GLOBS {
            if let LaneInputs::Globs(globs) = inputs
                && globs.is_empty()
            {
                assert!(
                    result.contains(lane),
                    "lane `{lane}` with empty globs list must always be selected"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // Group 5: exclusion proof — narrowing actually works
    // -----------------------------------------------------------------------

    #[test]
    fn lanes_for_changed_explicitly_mapped_lane_absent_when_path_does_not_match_its_globs() {
        // Find at least one lane that is mapped with explicit Globs (not
        // Global, not empty). Then construct a changed-file path that
        // definitely does NOT match any of that lane's globs and verify that
        // the lane is absent from the result (proving narrowing works).
        //
        // We look for the first Globs-mapped lane with at least one glob and
        // use a path guaranteed not to match (a UUID-like sentinel under an
        // unrelated prefix).
        let sentinel = "zzz-no-lane-will-ever-match-this-unique-sentinel-path-xq7r9w2t.never";
        let result = lanes_for_changed(&[sentinel]);

        for (lane, inputs) in LANE_INPUT_GLOBS {
            if let LaneInputs::Globs(globs) = inputs
                && !globs.is_empty()
            {
                // Verify none of the lane's globs match the sentinel.
                let any_match = globs.iter().any(|g| path_glob_matches(sentinel, g));
                if !any_match {
                    assert!(
                        !result.contains(lane),
                        "explicitly-mapped lane `{lane}` must be absent when its globs \
                         do not match the changed file set"
                    );
                    // One proven exclusion is sufficient for this test.
                    return;
                }
            }
        }
        // If every mapped lane is Global or has empty globs, the test is
        // vacuously satisfied (the table has not yet grown to include any
        // narrowing entry). This is acceptable for the starter table.
    }
}
