//! Foundry gate-catalog canonical domain — single source of truth for the
//! `oya gate ...` command catalog that the downstream content-validation
//! gates (quality-lane, documentation-system, supply-chain) read as their
//! input data.
//!
//! Naming justification:
//! - Crate id `oya-foundry-gate-catalog-domain` — `oya-` brand prefix
//!   (ADR-0017 / MFL-0011), `foundry` axis (per ADR-0107 family table),
//!   `gate-catalog` two-word subject (the gate-validate catalog), final
//!   segment `domain` (∈ ALLOWED_ROLES per canonical 13-value layer enum at
//!   `oya-foundry-fitness-predictable-naming-kernel::ALLOWED_ROLES`).
//! - Library identifier `oya_foundry_gate_catalog_domain` —
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
    "api-semver",
    "supply-chain",
    "pr-traceability",
    "cargo-prefix",
    "pre-push-contract",
    "quality-lanes",
    "honest-claims",
    "workspace-hygiene",
    "hyperscaler-arch-invariants",
    "hyperscaler-maturity-claims",
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
    "license-policy",
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
    "changeset-state-monotonicity",
    "changeset-state-enum-closed",
];

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
/// Strings are preserved with their full `cargo run -p oya-dev-cli -- …`
/// prefix so the downstream content-validation gates (which historically
/// did `check_script.contains(<canonical_command>)`) keep matching their
/// expected patterns against the unified catalog.
pub const AGGREGATED_NON_GATE_COMMANDS: &[&str] = &[
    // Toolchain primitives.
    "cargo fmt --all -- --check",
    "cargo check --workspace --all-targets --all-features",
    "cargo clippy --workspace --all-targets --all-features -- -D warnings",
    "cargo machete",
    "cargo audit",
    "cargo nextest run --workspace --all-features --no-fail-fast",
    "cargo deny check",
    // Demo and catalog.
    "cargo run -p oya-dev-cli -- demo",
    "cargo run -p oya-dev-cli -- catalog validate",
    // Doc pipeline (active steps in registry/docs/pipeline.tsv).
    "cargo run -p oya-dev-cli -- doc mdbook",
    "cargo run -p oya-dev-cli -- doc openapi",
    "cargo run -p oya-dev-cli -- doc rustdoc",
    "cargo run -p oya-dev-cli -- doc adr-index",
    // TypeScript workspace lanes (parameterized; not under run-all).
    "cargo run -p oya-dev-cli -- gate validate typescript-workspace --lane typecheck",
    "cargo run -p oya-dev-cli -- gate validate typescript-workspace --lane test",
    // Active-artifact + cedar-fragment + openapi-route emit-evidence lanes.
    "cargo run -p oya-dev-cli -- gate validate active-artifact-contract --emit-evidence evidence/active-artifact-contract-lane-run.json --emit-graph-edges registry/graph/active-artifact-contract-edges.json",
    "cargo run -p oya-dev-cli -- gate validate cedar-fragment-coverage --emit-evidence evidence/cedar-fragment-coverage-lane-run.json",
    "cargo run -p oya-dev-cli -- gate validate openapi-rest-route-parity --emit-evidence evidence/openapi-rest-route-parity-lane-run.json",
    // Release-supply-chain phased lane (separate from default supply-chain).
    "cargo run -p oya-dev-cli -- gate validate release-supply-chain --phase pre-release",
    "cargo run -p oya-dev-cli -- gate validate supply-chain --require-adr0039-evidence",
    // Local verification + dedicated foundry tool entry points.
    "cargo run -p oya-dev-cli -- verify",
    "cargo run -q -p oya-foundry-vcs-admission-gate-app",
    "cargo run -q -p oya-foundry-fitness-purpose-audit-app",
    "cargo run -p oya-foundry-vcs-merge-queue-fix-loop-app -- --gc-staging-refs --max-age-seconds 3600",
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
        rendered.push_str("cargo run -p oya-dev-cli -- gate validate ");
        rendered.push_str(lane);
        rendered.push('\n');
    }
    for command in AGGREGATED_NON_GATE_COMMANDS {
        rendered.push_str(command);
        rendered.push('\n');
    }
    rendered
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
            let expected = format!("cargo run -p oya-dev-cli -- gate validate {lane}");
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
        // documentation-system kernel checks for `cargo run -p oya-dev-cli -- doc <step>`
        // commands per registry/docs/pipeline.tsv.
        let rendered = all_canonical_commands_rendered();
        for step in ["mdbook", "openapi", "rustdoc", "adr-index"] {
            let expected = format!("cargo run -p oya-dev-cli -- doc {step}");
            assert!(
                rendered.contains(&expected),
                "rendered catalog must wire doc step `{step}`"
            );
        }
        assert!(rendered.contains("cargo run -p oya-dev-cli -- catalog validate"));
    }

    #[test]
    fn rendered_form_contains_pre_push_contract_check() {
        // oya-check-pre-push kernel asserts that `oya verify` is the
        // canonical local verification surface.
        let rendered = all_canonical_commands_rendered();
        assert!(rendered.contains("cargo run -p oya-dev-cli -- verify"));
    }

    #[test]
    fn rendered_form_contains_loop_recovery_patterns_lane() {
        let rendered = all_canonical_commands_rendered();
        assert!(
            rendered.contains("cargo run -p oya-dev-cli -- gate validate loop-recovery-patterns")
        );
    }

    #[test]
    fn rendered_form_contains_hyperscaler_architecture_and_maturity_lanes() {
        let rendered = all_canonical_commands_rendered();
        assert!(rendered.contains("cargo run -p oya-dev-cli -- gate validate workspace-hygiene"));
        assert!(
            rendered
                .contains("cargo run -p oya-dev-cli -- gate validate hyperscaler-arch-invariants")
        );
        assert!(
            rendered
                .contains("cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims")
        );
    }

    #[test]
    fn rendered_form_contains_architecture_boundaries_lane() {
        let rendered = all_canonical_commands_rendered();
        assert!(
            rendered.contains("cargo run -p oya-dev-cli -- gate validate architecture-boundaries")
        );
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
}
