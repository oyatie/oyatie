//! RED/GREEN multi-layer tests for the four registrar writers.
//!
//! Each writer is covered by PURE-content tests (no filesystem — exercise the `compute_*` fn) plus
//! a tmpfile round-trip test (exercise `apply_*` against a real on-disk file, asserting the
//! idempotent no-op on re-apply). The matrix per writer: fresh-apply produces correct content;
//! re-apply is byte-identical; the fail-closed refusal (unknown capability / brace-glob /
//! missing-slo / uncovered-member); and the #66 verbatim-append correctness (sorted + literal +
//! idempotent). The fourth (`workspace_member_glob`) is a coverage VERIFIER — its only outcomes are
//! covered (byte-identical no-op) or fail-closed (no glob is ever synthesized), so its matrix is
//! coverage-confirm / exclude-honoring / fail-closed-uncovered / malformed-manifest.

use super::*;
use std::path::PathBuf;

// ───────────────────────────── tmpdir helper ─────────────────────────────

/// A unique scratch directory under the OS temp, removed on drop. Deterministic-enough for tests:
/// keyed by a monotonically-increasing counter + the test thread name so parallel tests never
/// collide. No external crate (std-only) to keep the writer dep tree minimal.
struct TmpDir {
    path: PathBuf,
}

impl TmpDir {
    fn new(tag: &str) -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let path = std::env::temp_dir().join(format!("oya-registrar-writers-{tag}-{pid}-{n}"));
        std::fs::create_dir_all(&path).expect("create tmpdir");
        TmpDir { path }
    }

    fn write(&self, rel: &str, content: &str) {
        let abs = self.path.join(rel);
        if let Some(parent) = abs.parent() {
            std::fs::create_dir_all(parent).expect("create parent");
        }
        std::fs::write(&abs, content).expect("write file");
    }

    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.path.join(rel)).expect("read file")
    }
}

impl Drop for TmpDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

// ───────────────────────────── canonical-json fixture ─────────────────────────────

/// A minimal capability-registry fixture with the two group key shapes the writer matches: a
/// `meta_dir`-keyed group (`build/`) and a `capability`-keyed group (`data`). Already canonical.
fn registry_fixture() -> String {
    to_canonical_json(&serde_json::json!({
        "closed": true,
        "membership_lint_coverage": {
            "absorbs_current_crate_globs": [
                {
                    "meta_dir": "build/",
                    "globs": [
                        "libs/oya-ci-config",
                        "libs/oya-crate-registrar-kernel"
                    ]
                },
                {
                    "capability": "data",
                    "globs": [
                        "libs/oya-data-boundary-kernel"
                    ]
                }
            ]
        }
    }))
    .expect("fixture canonicalizes")
}

// ───────────────────────────── 1. capability_mapping (pure) ─────────────────────────────

#[test]
fn capability_fresh_apply_upserts_into_meta_dir_group() {
    let current = registry_fixture();
    let next =
        capability_mapping::compute(&current, "libs/oya-crate-registrar-app", "build/").unwrap();
    // The new dir is present, in the build/ group, sorted.
    assert!(next.contains("libs/oya-crate-registrar-app"));
    // Parse back and assert the group membership precisely.
    let value: Value = serde_json::from_str(&next).unwrap();
    let globs = value["membership_lint_coverage"]["absorbs_current_crate_globs"][0]["globs"]
        .as_array()
        .unwrap();
    let strings: Vec<&str> = globs.iter().filter_map(Value::as_str).collect();
    assert_eq!(
        strings,
        vec![
            "libs/oya-ci-config",
            "libs/oya-crate-registrar-app",
            "libs/oya-crate-registrar-kernel"
        ],
        "the new dir is upserted sorted into the build/ group"
    );
    // The other (capability-keyed) group is untouched.
    assert!(next.contains("libs/oya-data-boundary-kernel"));
}

#[test]
fn capability_matches_capability_keyed_group() {
    let current = registry_fixture();
    let next =
        capability_mapping::compute(&current, "libs/oya-data-pagination-kernel", "data").unwrap();
    let value: Value = serde_json::from_str(&next).unwrap();
    let globs = value["membership_lint_coverage"]["absorbs_current_crate_globs"][1]["globs"]
        .as_array()
        .unwrap();
    let strings: Vec<&str> = globs.iter().filter_map(Value::as_str).collect();
    assert_eq!(
        strings,
        vec!["libs/oya-data-boundary-kernel", "libs/oya-data-pagination-kernel"]
    );
}

#[test]
fn capability_reapply_is_byte_identical_noop() {
    let current = registry_fixture();
    let once =
        capability_mapping::compute(&current, "libs/oya-crate-registrar-app", "build/").unwrap();
    let twice =
        capability_mapping::compute(&once, "libs/oya-crate-registrar-app", "build/").unwrap();
    assert_eq!(once, twice, "re-applying the capability mapping is a no-op (byte-identical)");
}

#[test]
fn capability_already_present_is_byte_identical_noop() {
    let current = registry_fixture();
    // libs/oya-crate-registrar-kernel is ALREADY in the build/ group — re-mapping it must produce
    // bytes identical to the canonical re-serialization of the input.
    let next =
        capability_mapping::compute(&current, "libs/oya-crate-registrar-kernel", "build/").unwrap();
    assert_eq!(next, current, "mapping an already-present dir is a byte-identical no-op");
}

#[test]
fn capability_unknown_slug_rejected() {
    let current = registry_fixture();
    let err =
        capability_mapping::compute(&current, "libs/oya-whatever-kernel", "totally-made-up")
            .unwrap_err();
    assert_eq!(
        err,
        WriterError::UnknownCapability("totally-made-up".to_owned()),
        "a slug with no existing group is fail-closed rejected"
    );
}

#[test]
fn capability_output_is_canonical_json() {
    let current = registry_fixture();
    let next =
        capability_mapping::compute(&current, "libs/oya-crate-registrar-app", "build/").unwrap();
    // Canonical: re-canonicalizing the output yields the same bytes.
    let value: Value = serde_json::from_str(&next).unwrap();
    let recanon = to_canonical_json(&value).unwrap();
    assert_eq!(next, recanon, "writer output is already canonical JSON (byte-stable)");
}

// ───────────────────────────── 1. capability_mapping (tmpfile round-trip) ─────────────────────────────

#[test]
fn capability_apply_roundtrip_and_idempotent() {
    let tmp = TmpDir::new("cap");
    tmp.write(capability_mapping::REGISTRY_PATH, &registry_fixture());

    let wrote =
        capability_mapping::apply(&tmp.path, "libs/oya-crate-registrar-app", "build/").unwrap();
    assert!(wrote, "first apply writes the file");
    let after_first = tmp.read(capability_mapping::REGISTRY_PATH);
    assert!(after_first.contains("libs/oya-crate-registrar-app"));

    // Re-apply: no write, file byte-identical.
    let wrote_again =
        capability_mapping::apply(&tmp.path, "libs/oya-crate-registrar-app", "build/").unwrap();
    assert!(!wrote_again, "re-apply is a no-op (no write)");
    assert_eq!(after_first, tmp.read(capability_mapping::REGISTRY_PATH));
}

/// The real `governance/capability-registry.json` is HAND-AUTHORED with a deliberate key order
/// (`_comment`, `schema_version`, `doctrine_adr`, …) — nothing close to sorted. The
/// canonical-json policy pins `sort_keys: false` precisely because "sorting would churn 1452 repo
/// files and destroy intentional order on the agent entry surface". A writer that re-sorts on
/// write turns one crate registration into a wholesale reordering of a governed spec.
///
/// This fixture reproduces that shape: keys in descending-ish authored order, so ANY sort is
/// visible as a reordering.
fn hand_authored_registry() -> String {
    concat!(
        "{\n",
        "  \"_comment\": \"authored first, sorts near-first\",\n",
        "  \"schema_version\": \"1.1.0\",\n",
        "  \"doctrine_adr\": \"ADR-0562\",\n",
        "  \"closed\": true,\n",
        "  \"membership_lint_coverage\": {\n",
        "    \"_comment\": \"nested authored order matters too\",\n",
        "    \"absorbs_current_crate_globs\": [\n",
        "      {\n",
        "        \"meta_dir\": \"build/\",\n",
        "        \"globs\": [\n",
        "          \"libs/oya-ci-config\"\n",
        "        ]\n",
        "      }\n",
        "    ]\n",
        "  }\n",
        "}\n"
    )
    .to_owned()
}

/// The authored key order of every object in `text`, outermost object first, as they appear on
/// disk. Cheap textual read (keys are one-per-line in 2-space pretty form) — enough to assert
/// "the writer did not reorder", which a parsed comparison could not do under a sorting map.
fn authored_key_order(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let rest = trimmed.strip_prefix('"')?;
            let (key, after) = rest.split_once('"')?;
            after.trim_start().starts_with(':').then(|| key.to_owned())
        })
        .collect()
}

#[test]
fn capability_write_preserves_hand_authored_key_order() {
    let current = hand_authored_registry();
    let next = capability_mapping::compute(&current, "libs/oya-crate-registrar-app", "build/")
        .expect("compute");

    assert_eq!(
        authored_key_order(&next),
        authored_key_order(&current),
        "canonical-json policy pins sort_keys:false — registering a crate must upsert one glob, \
         NOT reorder the governed registry's hand-authored keys"
    );
    // The edit itself still landed (the assertion above must not pass by doing nothing).
    assert!(next.contains("libs/oya-crate-registrar-app"), "the upsert still happened");
}

#[test]
fn to_canonical_json_does_not_sort_keys() {
    // Direct cover of the writer primitive: sort_keys:false is a property of the serializer,
    // not an accident of one caller's input.
    let authored: Value = serde_json::from_str(r#"{"zeta":1,"alpha":2,"mid":{"z":1,"a":2}}"#)
        .expect("parse");
    let out = to_canonical_json(&authored).expect("serialize");
    assert_eq!(
        authored_key_order(&out),
        vec!["zeta", "alpha", "mid", "z", "a"],
        "to_canonical_json must preserve authored key order (canonical-json-policy sort_keys:false)"
    );
}

// ───────────────────────────── 2. adr_governed_paths (pure) ─────────────────────────────

const ADR_NO_BLOCK: &str = "---\nid: ADR-0568\n---\n\n# ADR-0568: scaffold\n\n## Context\n\nSome body.\n";

#[test]
fn adr_creates_block_when_absent() {
    let paths = vec![
        "libs/oya-crate-registrar-app/src/lib.rs".to_owned(),
        "libs/oya-crate-registrar-app/Cargo.toml".to_owned(),
        "libs/oya-crate-registrar-app/BUCK".to_owned(),
    ];
    let next = adr_governed_paths::compute(ADR_NO_BLOCK, &paths).unwrap();
    assert!(next.contains("## Governed surfaces"));
    // The block lists each path verbatim, sorted (BUCK < Cargo.toml < src/lib.rs).
    let block_start = next.find("## Governed surfaces").unwrap();
    let block = &next[block_start..];
    let buck = block.find("BUCK").unwrap();
    let cargo = block.find("Cargo.toml").unwrap();
    let lib = block.find("src/lib.rs").unwrap();
    assert!(buck < cargo && cargo < lib, "paths are sorted in the block");
    // The original body is preserved.
    assert!(next.contains("## Context"));
    assert!(next.contains("Some body."));
}

#[test]
fn adr_upserts_into_existing_block_sorted_and_deduped() {
    // An ADR that already governs two paths; append two more (one duplicate).
    let current = "# ADR-0568\n\n## Governed surfaces\n\n```\nlibs/oya-crate-registrar-app/BUCK\nlibs/oya-crate-registrar-app/Cargo.toml\n```\n\n## Consequences\n\nbody\n";
    let paths = vec![
        "libs/oya-crate-registrar-app/Cargo.toml".to_owned(), // duplicate
        "libs/oya-crate-registrar-app/OWNERS".to_owned(),
        "libs/oya-crate-registrar-app/src/lib.rs".to_owned(),
    ];
    let next = adr_governed_paths::compute(current, &paths).unwrap();
    let block_start = next.find("## Governed surfaces").unwrap();
    // Slice from the heading to the trailing "## Consequences".
    let consequences = next.find("## Consequences").unwrap();
    let block = &next[block_start..consequences];
    // Every expected path appears exactly once, sorted.
    let expected = [
        "libs/oya-crate-registrar-app/BUCK",
        "libs/oya-crate-registrar-app/Cargo.toml",
        "libs/oya-crate-registrar-app/OWNERS",
        "libs/oya-crate-registrar-app/src/lib.rs",
    ];
    let mut last = 0;
    for path in expected {
        let at = block.find(path).unwrap_or_else(|| panic!("{path} present"));
        assert!(at >= last, "{path} is in sorted order");
        last = at;
        // Exactly once (no duplicate Cargo.toml from the upsert).
        assert_eq!(block.matches(path).count(), 1, "{path} appears exactly once");
    }
    // The trailing section survives.
    assert!(next.contains("## Consequences"));
    assert!(next.contains("body"));
}

#[test]
fn adr_reapply_is_byte_identical_noop() {
    let paths = vec![
        "libs/oya-crate-registrar-app/src/lib.rs".to_owned(),
        "libs/oya-crate-registrar-app/Cargo.toml".to_owned(),
    ];
    let once = adr_governed_paths::compute(ADR_NO_BLOCK, &paths).unwrap();
    let twice = adr_governed_paths::compute(&once, &paths).unwrap();
    assert_eq!(once, twice, "re-applying the governed-path append is a no-op (byte-identical)");
}

#[test]
fn adr_verbatim_paths_are_literal_no_globs() {
    // The #66 fix in the affirmative: each path is emitted as a literal line — no brace-glob token.
    let paths = vec![
        "libs/oya-crate-registrar-app/src/lib.rs".to_owned(),
        "libs/oya-crate-registrar-app/src/tests.rs".to_owned(),
    ];
    let next = adr_governed_paths::compute(ADR_NO_BLOCK, &paths).unwrap();
    assert!(next.contains("libs/oya-crate-registrar-app/src/lib.rs"));
    assert!(next.contains("libs/oya-crate-registrar-app/src/tests.rs"));
    // No brace-glob collapse anywhere in the block.
    let block = &next[next.find("## Governed surfaces").unwrap()..];
    assert!(!block.contains('{') && !block.contains('}'), "no brace-glob in the block: {block}");
}

#[test]
fn adr_brace_glob_path_rejected() {
    let paths = vec!["libs/oya-crate-registrar-app/src/{lib,tests}.rs".to_owned()];
    let err = adr_governed_paths::compute(ADR_NO_BLOCK, &paths).unwrap_err();
    assert_eq!(
        err,
        WriterError::BraceGlobInGovernedPath(
            "libs/oya-crate-registrar-app/src/{lib,tests}.rs".to_owned()
        ),
        "a brace-glob governed path is fail-closed rejected (#66 defense)"
    );
}

#[test]
fn adr_block_body_preserved_through_roundtrip_parse() {
    // Verify the block we emit is exactly the shape resolve_justifications credits: a heading,
    // a fenced block, one verbatim path per line. Re-parsing our own block recovers the paths.
    let paths = vec![
        "libs/oya-crate-registrar-app/BUCK".to_owned(),
        "libs/oya-crate-registrar-app/Cargo.toml".to_owned(),
    ];
    let next = adr_governed_paths::compute(ADR_NO_BLOCK, &paths).unwrap();
    // Append the same paths again — the re-parse-merge must keep exactly those two.
    let next2 = adr_governed_paths::compute(&next, &paths).unwrap();
    assert_eq!(next, next2);
    // The fenced block contains exactly the two lines (plus fences).
    let start = next.find("```").unwrap() + 3;
    let rest = &next[start..];
    let close = rest.find("```").unwrap();
    let body = rest[..close].trim();
    let lines: Vec<&str> = body.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(
        lines,
        vec!["libs/oya-crate-registrar-app/BUCK", "libs/oya-crate-registrar-app/Cargo.toml"]
    );
}

// ───────────────────────────── 2. adr_governed_paths (defect-driven RED tests) ─────────────────────────────

#[test]
fn adr_heading_with_no_fence_followed_by_foreign_code_block_is_not_hijacked() {
    // DEFECT 1: `## Governed surfaces` has NO fence in its OWN section; a LATER `## Consequences`
    // section owns a ```code``` block. The locator must NOT span across into that foreign block.
    // After compute: the Consequences heading + its code block survive verbatim, the governed
    // block is created in the Governed-surfaces section, and the foreign code lines are NOT
    // credited as governed paths.
    let current = "# ADR-0568\n\n## Governed surfaces\n\nNo paths yet.\n\n## Consequences\n\n```rust\nlet hijacked = true;\nfn foreign() {}\n```\n\nMore prose.\n";
    let paths = vec!["libs/oya-crate-registrar-app/src/lib.rs".to_owned()];
    let next = adr_governed_paths::compute(current, &paths).unwrap();

    // The Consequences section + its foreign code block survive verbatim.
    assert!(
        next.contains("## Consequences\n\n```rust\nlet hijacked = true;\nfn foreign() {}\n```\n\nMore prose.\n"),
        "the foreign Consequences code block must survive verbatim:\n{next}"
    );
    // Exactly one `## Governed surfaces` heading (no duplicate appended at EOF).
    assert_eq!(
        next.matches("## Governed surfaces").count(),
        1,
        "exactly one Governed surfaces heading:\n{next}"
    );
    // The governed path is credited.
    assert!(next.contains("libs/oya-crate-registrar-app/src/lib.rs"));
    // The foreign code lines are NOT credited as governed paths.
    assert!(
        !next.contains("\nlet hijacked = true;\nlibs")
            && !next.contains("fn foreign() {}\nlibs"),
        "foreign code lines must not be slurped as governed paths:\n{next}"
    );
    // Idempotent re-apply.
    let again = adr_governed_paths::compute(&next, &paths).unwrap();
    assert_eq!(next, again, "re-apply is byte-identical");
    // The block lives in the Governed-surfaces section, BEFORE Consequences.
    let gov = next.find("## Governed surfaces").unwrap();
    let cons = next.find("## Consequences").unwrap();
    let gov_block = &next[gov..cons];
    assert!(
        gov_block.contains("libs/oya-crate-registrar-app/src/lib.rs"),
        "governed path is inside the Governed-surfaces section:\n{gov_block}"
    );
}

#[test]
fn adr_info_string_fence_existing_path_is_preserved() {
    // DEFECT 2: the Governed-surfaces block opens with an info-string fence ```text. The existing
    // path inside must be preserved (not dropped) on the next apply.
    let current = "# ADR-0568\n\n## Governed surfaces\n\n```text\nlibs/oya-crate-registrar-app/BUCK\n```\n";
    let paths = vec!["libs/oya-crate-registrar-app/Cargo.toml".to_owned()];
    let next = adr_governed_paths::compute(current, &paths).unwrap();
    assert!(
        next.contains("libs/oya-crate-registrar-app/BUCK"),
        "the existing path under an info-string fence is preserved:\n{next}"
    );
    assert!(next.contains("libs/oya-crate-registrar-app/Cargo.toml"));
    // Re-emitted opening fence is bare (no info string) and idempotent.
    let again = adr_governed_paths::compute(&next, &paths).unwrap();
    assert_eq!(next, again, "re-apply is byte-identical");
}

#[test]
fn adr_malformed_governed_path_whitespace_rejected() {
    // DEFECT 4/6: leading/trailing whitespace is non-idempotent → fail-closed.
    let paths = vec!["  libs/spaced  ".to_owned()];
    let err = adr_governed_paths::compute(ADR_NO_BLOCK, &paths).unwrap_err();
    assert_eq!(err, WriterError::MalformedGovernedPath("  libs/spaced  ".to_owned()));
}

#[test]
fn adr_malformed_governed_path_newline_rejected() {
    let paths = vec!["libs/a\nlibs/b".to_owned()];
    let err = adr_governed_paths::compute(ADR_NO_BLOCK, &paths).unwrap_err();
    assert_eq!(err, WriterError::MalformedGovernedPath("libs/a\nlibs/b".to_owned()));
}

#[test]
fn adr_malformed_governed_path_fence_sequence_rejected() {
    let paths = vec!["libs/a```evil".to_owned()];
    let err = adr_governed_paths::compute(ADR_NO_BLOCK, &paths).unwrap_err();
    assert_eq!(err, WriterError::MalformedGovernedPath("libs/a```evil".to_owned()));
}

#[test]
fn adr_legacy_suffix_heading_is_not_canonical_block() {
    // DEFECT 5: `## Governed surfaces (legacy)` must NOT match the canonical heading. The heading
    // is treated as absent → a real `## Governed surfaces` block is created and the `(legacy)` line
    // is untouched.
    let current = "# ADR-0568\n\n## Governed surfaces (legacy)\n\nold notes.\n";
    let paths = vec!["libs/oya-crate-registrar-app/src/lib.rs".to_owned()];
    let next = adr_governed_paths::compute(current, &paths).unwrap();
    // The legacy line survives untouched.
    assert!(next.contains("## Governed surfaces (legacy)"));
    assert!(next.contains("old notes."));
    // A real canonical `## Governed surfaces` block (with a trailing newline boundary) is created.
    assert!(
        next.contains("## Governed surfaces\n"),
        "a canonical Governed surfaces heading line was created:\n{next}"
    );
    assert!(next.contains("libs/oya-crate-registrar-app/src/lib.rs"));
    // Idempotent.
    let again = adr_governed_paths::compute(&next, &paths).unwrap();
    assert_eq!(next, again, "re-apply is byte-identical");
}

// ───────────────────────────── 2. adr_governed_paths (tmpfile round-trip) ─────────────────────────────

#[test]
fn adr_apply_roundtrip_and_idempotent() {
    let tmp = TmpDir::new("adr");
    let rel = "docs/adr-archive/ADR-0568-born-accounting-register-crate-registrar-kernel.md";
    tmp.write(rel, ADR_NO_BLOCK);

    let paths = vec![
        "libs/oya-crate-registrar-app/BUCK".to_owned(),
        "libs/oya-crate-registrar-app/Cargo.toml".to_owned(),
    ];
    let wrote = adr_governed_paths::apply(&tmp.path, rel, &paths).unwrap();
    assert!(wrote, "first apply writes the ADR");
    let after_first = tmp.read(rel);
    assert!(after_first.contains("## Governed surfaces"));

    let wrote_again = adr_governed_paths::apply(&tmp.path, rel, &paths).unwrap();
    assert!(!wrote_again, "re-apply is a no-op (no write)");
    assert_eq!(after_first, tmp.read(rel));
}

// ───────────────────────────── 3. catalog_yaml (pure) ─────────────────────────────

#[test]
fn catalog_fresh_render_has_required_fields() {
    let yaml = catalog_yaml::compute("iam/core/identity-domain", "control", "ga-control-plane")
        .unwrap();
    assert!(yaml.contains("plane: control"));
    assert!(yaml.contains("slo: ga-control-plane"));
    assert!(yaml.contains("capability: identity-domain"));
    assert!(yaml.ends_with('\n'), "trailing newline");
}

#[test]
fn catalog_fresh_render_declares_the_api_stability_tier() {
    // A row rendered WITHOUT `api_stability:` is born-blocking: the ci/facade/lifecycle-status
    // api-stability-tier lane is rooted on registry/catalog/*.yaml with stage_field
    // `api_stability` and has NO frozen violation row, so one undeclared row is an unbaselined
    // `stage_not_declared`. This test is the thing that fails if the key is ever dropped again.
    let yaml = catalog_yaml::compute("iam/core/identity-domain", "control", "ga-control-plane")
        .unwrap();
    assert!(
        yaml.lines().any(|l| l == "api_stability: preview"),
        "fresh catalog row must declare the canonical first tier as a TOP-LEVEL scalar, got:\n{yaml}"
    );
}

#[test]
fn catalog_reapply_is_byte_identical_noop() {
    let once = catalog_yaml::compute("iam/core/identity-domain", "control", "ga-control-plane")
        .unwrap();
    let twice = catalog_yaml::compute("iam/core/identity-domain", "control", "ga-control-plane")
        .unwrap();
    assert_eq!(once, twice, "re-rendering the catalog is deterministic (byte-identical)");
}

#[test]
fn catalog_missing_slo_rejected() {
    let err = catalog_yaml::compute("iam/core/identity-domain", "control", "   ").unwrap_err();
    assert_eq!(
        err,
        WriterError::MissingCatalogField("slo".to_owned()),
        "an empty slo is fail-closed rejected (never defaulted)"
    );
}

#[test]
fn catalog_missing_plane_rejected() {
    let err = catalog_yaml::compute("iam/core/identity-domain", "", "ga-control-plane").unwrap_err();
    assert_eq!(err, WriterError::MissingCatalogField("plane".to_owned()));
}

#[test]
fn catalog_path_derives_from_leaf() {
    assert_eq!(
        catalog_yaml::catalog_path("iam/core/identity-domain"),
        "registry/catalog/identity-domain.yaml"
    );
}

// ───────────────────────────── 3. catalog_yaml (defect-driven RED tests) ─────────────────────────────

#[test]
fn catalog_slo_with_newline_forging_keys_rejected() {
    // DEFECT 3: a newline in `slo` forges a top-level YAML key → fail-closed InvalidCatalogField.
    let err = catalog_yaml::compute(
        "iam/core/identity-domain",
        "control",
        "ga\nmalicious: true",
    )
    .unwrap_err();
    assert_eq!(err, WriterError::InvalidCatalogField("slo".to_owned()));
}

#[test]
fn catalog_plane_with_yaml_map_metachars_rejected() {
    // DEFECT 3: a value like `{flow: x}` changes the scalar into a YAML map → fail-closed.
    let err = catalog_yaml::compute("iam/core/identity-domain", "{flow: x}", "ga-control-plane")
        .unwrap_err();
    assert_eq!(err, WriterError::InvalidCatalogField("plane".to_owned()));
}

#[test]
fn catalog_normal_identifier_values_still_render() {
    // DEFECT 3 positive: legit identifier-shaped values render fine.
    let yaml = catalog_yaml::compute("iam/core/identity-domain", "control", "ga-control-plane")
        .unwrap();
    assert!(yaml.contains("plane: control"));
    assert!(yaml.contains("slo: ga-control-plane"));
    assert!(yaml.contains("capability: identity-domain"));
}

// ───────────────────────────── 3. catalog_yaml (tmpfile round-trip) ─────────────────────────────

#[test]
fn catalog_apply_roundtrip_and_idempotent() {
    let tmp = TmpDir::new("cat");
    let dir = "iam/core/identity-domain";
    let rel = catalog_yaml::catalog_path(dir);

    let wrote = catalog_yaml::apply(&tmp.path, dir, "control", "ga-control-plane").unwrap();
    assert!(wrote, "first apply writes the catalog yaml");
    let after_first = tmp.read(&rel);
    assert!(after_first.contains("slo: ga-control-plane"));

    let wrote_again = catalog_yaml::apply(&tmp.path, dir, "control", "ga-control-plane").unwrap();
    assert!(!wrote_again, "re-apply is a no-op (no write)");
    assert_eq!(after_first, tmp.read(&rel));
}

#[test]
fn catalog_apply_missing_slo_rejected_before_write() {
    let tmp = TmpDir::new("cat-bad");
    let dir = "iam/core/identity-domain";
    let err = catalog_yaml::apply(&tmp.path, dir, "control", "").unwrap_err();
    assert_eq!(err, WriterError::MissingCatalogField("slo".to_owned()));
    // No file was written.
    assert!(!tmp.path.join(catalog_yaml::catalog_path(dir)).exists());
}

// ───────────────────────────── 4. workspace_member_glob (pure) ─────────────────────────────

/// A minimal root-workspace manifest fixture whose `members` are glob-only (ADR-0538) with a
/// narrowed leaf glob (`libs/oya-*`), a bare-leaf glob (`libs/*`), a capability glob
/// (`messaging/*/*`), and an `exclude` subtree. The comment is load-bearing — a re-serializing
/// writer would lose it, which is exactly why the verifier returns `current` verbatim on no-op.
const MANIFEST_FIXTURE: &str = "[workspace]\nmembers = [\n  # glob-only members (ADR-0538): zero edit to add a crate under an existing root.\n  \"libs/oya-*\",\n  \"cloud/cloud-ci/gates/*\",\n  \"messaging/*/*\",\n]\nexclude = [\n  \"messaging/observability\",\n]\nresolver = \"2\"\n";

#[test]
fn member_glob_covered_dir_is_byte_identical_noop() {
    // `libs/oya-crate-registrar-app` is covered by `libs/oya-*` → return the manifest verbatim.
    let next =
        workspace_member_glob::compute(MANIFEST_FIXTURE, "libs/oya-crate-registrar-app").unwrap();
    assert_eq!(
        next, MANIFEST_FIXTURE,
        "a covered dir yields the manifest byte-unchanged (comments preserved)"
    );
}

#[test]
fn member_glob_covered_by_bare_leaf_glob_noop() {
    // A bare-`*` leaf glob (`cloud/cloud-ci/gates/*`) covers a non-oya-prefixed leaf.
    let next =
        workspace_member_glob::compute(MANIFEST_FIXTURE, "cloud/cloud-ci/gates/registry-drift")
            .unwrap();
    assert_eq!(next, MANIFEST_FIXTURE, "the bare-leaf glob covers the dir → no-op");
}

#[test]
fn member_glob_covered_by_capability_glob_noop() {
    // A 3-segment capability glob (`messaging/*/*`) covers a face/leaf dir.
    let next =
        workspace_member_glob::compute(MANIFEST_FIXTURE, "messaging/core/domain").unwrap();
    assert_eq!(next, MANIFEST_FIXTURE);
}

#[test]
fn member_glob_reapply_is_byte_identical_noop() {
    let once =
        workspace_member_glob::compute(MANIFEST_FIXTURE, "libs/oya-foo-kernel").unwrap();
    let twice = workspace_member_glob::compute(&once, "libs/oya-foo-kernel").unwrap();
    assert_eq!(once, twice, "re-applying the covered-dir check is a no-op (byte-identical)");
    assert_eq!(once, MANIFEST_FIXTURE);
}

#[test]
fn member_glob_uncovered_dir_is_fail_closed() {
    // No members glob covers `apps/oya-thing-app` → the writer REFUSES (never synthesizes a glob).
    let err = workspace_member_glob::compute(MANIFEST_FIXTURE, "apps/oya-thing-app").unwrap_err();
    assert_eq!(
        err,
        WriterError::WorkspaceMemberUncovered("apps/oya-thing-app".to_owned()),
        "an uncovered dir is fail-closed — a covering glob is a human ADR decision, never invented"
    );
}

#[test]
fn member_glob_excluded_subtree_is_uncovered() {
    // `messaging/observability/...` matches `messaging/*/*` but is removed by the exclude subtree →
    // not covered → fail-closed (the writer honors excludes via the workspace-members kernel).
    let err =
        workspace_member_glob::compute(MANIFEST_FIXTURE, "messaging/observability/tracing")
            .unwrap_err();
    assert_eq!(
        err,
        WriterError::WorkspaceMemberUncovered("messaging/observability/tracing".to_owned())
    );
}

#[test]
fn member_glob_oya_prefix_narrowing_rejects_non_oya_leaf() {
    // `libs/oya-*` does NOT cover a non-oya-prefixed leaf — the writer must not pretend it does
    // (that crate would fail the cargo-prefix gate; widening the glob is not the registrar's job).
    let err = workspace_member_glob::compute(MANIFEST_FIXTURE, "libs/not-prefixed").unwrap_err();
    assert_eq!(
        err,
        WriterError::WorkspaceMemberUncovered("libs/not-prefixed".to_owned())
    );
}

#[test]
fn member_glob_malformed_manifest_is_fail_closed() {
    // A manifest with no `[workspace]` table is a typed fail-closed error (never a partial write).
    let err = workspace_member_glob::compute("[package]\nname = \"x\"\n", "libs/oya-foo-kernel")
        .unwrap_err();
    assert!(
        matches!(err, WriterError::WorkspaceManifest(_)),
        "a malformed manifest is fail-closed, got {err:?}"
    );
}

#[test]
fn member_glob_unparseable_toml_is_fail_closed() {
    let err =
        workspace_member_glob::compute("this is = = not toml", "libs/oya-foo-kernel").unwrap_err();
    assert!(matches!(err, WriterError::WorkspaceManifest(_)), "got {err:?}");
}

#[test]
fn member_glob_trailing_slash_dir_is_normalized() {
    // A trailing-slash dir is normalized before the coverage check (idempotent).
    let next =
        workspace_member_glob::compute(MANIFEST_FIXTURE, "libs/oya-foo-kernel/").unwrap();
    assert_eq!(next, MANIFEST_FIXTURE);
}

// ───────────────────────────── 4. workspace_member_glob (tmpfile round-trip) ─────────────────────────────

#[test]
fn member_glob_apply_covered_is_noop_no_write() {
    let tmp = TmpDir::new("wsm");
    tmp.write(workspace_member_glob::MANIFEST_PATH, MANIFEST_FIXTURE);

    // Covered dir → apply returns false (no write) and the file is byte-identical.
    let wrote = workspace_member_glob::apply(&tmp.path, "libs/oya-crate-registrar-app").unwrap();
    assert!(!wrote, "a covered dir is a no-op (no write)");
    assert_eq!(
        tmp.read(workspace_member_glob::MANIFEST_PATH),
        MANIFEST_FIXTURE,
        "the manifest is left byte-identical (comments intact)"
    );

    // Re-apply: still a no-op.
    let wrote_again =
        workspace_member_glob::apply(&tmp.path, "libs/oya-crate-registrar-app").unwrap();
    assert!(!wrote_again, "re-apply is a no-op (no write)");
    assert_eq!(tmp.read(workspace_member_glob::MANIFEST_PATH), MANIFEST_FIXTURE);
}

#[test]
fn member_glob_apply_uncovered_is_fail_closed_no_write() {
    let tmp = TmpDir::new("wsm-bad");
    tmp.write(workspace_member_glob::MANIFEST_PATH, MANIFEST_FIXTURE);

    let err = workspace_member_glob::apply(&tmp.path, "apps/oya-thing-app").unwrap_err();
    assert_eq!(
        err,
        WriterError::WorkspaceMemberUncovered("apps/oya-thing-app".to_owned())
    );
    // The manifest is untouched (fail-closed, never a partial write).
    assert_eq!(tmp.read(workspace_member_glob::MANIFEST_PATH), MANIFEST_FIXTURE);
}
