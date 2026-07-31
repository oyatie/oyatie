// cloud-ci-affected-set decision fixtures (ADR-0554, FRIC-1781310000).
//
// Pins the pure kernel contract that closes the cf16525 false-green class: a change to code
// OUTSIDE any fixed CI scope (the PR #651 shape — oya/identity not compiling while the binding
// lane only ran //cloud/cloud-ci/...) must land in the decided target set, and every seam where
// the old advisory lane could silently under-test must either ESCALATE TO FULL or REFUSE.
//
// The live integration proof is the lane itself: it runs as a REQUIRED context on the PR that
// ships it, with this crate (and the in-PR oya/ci-webhook-gateway fix) inside its own cone.
//
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use ci_affected_target_set::{
    Change, Decision, GATE_ID, Policy, parse_name_status_z, plan_changes, resolve,
};

/// A pack mirroring the shipped oyatie policy shape (the tests stay engine-side: the kernel
/// must work against ANY pack, so fixtures carry their own).
fn policy() -> Policy {
    Policy::from_json(
        r#"{
            "gate_id": "cloud-ci-affected-set",
            "universe": "//...",
            "full_run_targets": ["//..."],
            "full_trigger_patterns": [
                ".buckconfig",
                ".buckconfig.local",
                ".buckconfig.d/**",
                "toolchains/**",
                "third-party/**",
                "**/*.bzl",
                "**/*.bxl",
                "**/PACKAGE",
                "rust-toolchain.toml"
            ],
            "require_owner_patterns": [
                "**/*.rs",
                "cloud/cloud-kernel/crates/oya-cloud-kernel-arch-aarch64-adapter/linker.ld",
                "cloud/cloud-kernel/crates/oya-cloud-kernel-arch-x86-64-adapter/linker.ld"
            ],
            "package_definition_basenames": ["BUCK.v2", "BUCK"],
            "package_sibling_basenames": ["Cargo.toml", "build.rs"],
            "cell_roots": {"": "//"},
            "synthetic_dependencies": {
                "docs/**": [],
                "**/*.md": []
            },
            "inert_selection_classes": ["docs/**", "**/*.md"],
            "default_base_ref": "origin/dev"
        }"#,
    )
    .expect("fixture pack parses")
}

fn owners(entries: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
    entries
        .iter()
        .map(|(p, ts)| (p.to_string(), ts.iter().map(|t| t.to_string()).collect()))
        .collect()
}

/// Locates the shipped policy pack from a Buck2 test working directory.
fn shipped_pack_path() -> PathBuf {
    let mut dir = std::env::current_dir().expect("cwd");
    let rel = PathBuf::from("ci/facade/affected-target-set/affected-set-policy.json");

    loop {
        let candidate = dir.join(&rel);
        if candidate.is_file() {
            return candidate;
        }
        assert!(
            dir.pop(),
            "failed to locate the shipped pack from the test cwd"
        );
    }
}

// ── The cf16525 class: out-of-scope source change MUST be in the decided set ─────────────

#[test]
fn red_class_cf16525_out_of_scope_source_lands_in_the_affected_seeds() {
    // PR #651 head cf16525: oya/identity server code did not compile (E0433 x3) yet the
    // binding lane was green because it only ran //cloud/cloud-ci/... . The kernel decision
    // for that diff shape MUST include the owning identity targets as seeds.
    let p = policy();
    let changes = [Change::Present(
        "oya/identity/crates/oya-identity-workload-app/src/server.rs".into(),
    )];
    let plan = plan_changes(&changes, &p);
    let owner_map = owners(&[(
        "oya/identity/crates/oya-identity-workload-app/src/server.rs",
        &["root//oya/identity/crates/oya-identity-workload-app:oya-identity-workload-app"],
    )]);
    let decision = resolve(&plan, &owner_map, &p);
    match decision {
        Decision::Affected { seeds } => {
            assert!(
                seeds
                    .iter()
                    .any(|s| s.contains("oya-identity-workload-app")),
                "the out-of-scope target must be a seed; got {seeds:?}"
            );
        }
        other => panic!("expected Affected, got {other:?}"),
    }
}

#[test]
fn docs_only_diff_is_no_graph_targets_via_explicit_inert_declaration() {
    let p = policy();
    let changes = [Change::Present("docs/decisions/ADR-0001-x.md".into())];
    let plan = plan_changes(&changes, &p);
    // owner() ran and found nothing. Post-round-6 (defect 2), an unowned non-owner-required path
    // is NO LONGER silently ignored — it must map to an EXPLICIT synthetic-dependency declaration
    // or it escalates to FULL. Docs are declared inert (`docs/**`/`**/*.md` -> []) in the pack, so
    // this stays NoGraphTargets — now by an auditable pack rule, not a silent code default.
    let decision = resolve(&plan, &owners(&[("docs/decisions/ADR-0001-x.md", &[])]), &p);
    assert_eq!(decision, Decision::NoGraphTargets);
}

// ── PREDICATE (1): a non-empty diff that selects NOTHING is RED ──────────────────────────

#[test]
fn red_predicate_1_nonempty_diff_with_empty_selection_refuses() {
    // THE RED. This is PR #1389 replayed through the real kernel: `.github/**` declared `[]` in
    // synthetic_dependencies. Every escalation above fires on paths with NO declaration; a `[]`
    // declaration matches, contributes nothing, and used to sail straight to NoGraphTargets ->
    // exit 0. The lane then reported SUCCESS having built and tested zero targets, and a
    // workflow-only PR walked past the no-new-shell ratchet and every whole-tree gate.
    //
    // The pack below is the exact regression shape: `.github/**` is declared inert-by-seed but is
    // NOT in `inert_selection_classes`, so it holds no licence to be the entire selection.
    let p = Policy::from_json(
        r#"{
            "gate_id": "cloud-ci-affected-set",
            "universe": "//...",
            "full_run_targets": ["//..."],
            "full_trigger_patterns": ["**/*.bzl"],
            "require_owner_patterns": ["**/*.rs"],
            "package_definition_basenames": ["BUCK.v2", "BUCK"],
            "package_sibling_basenames": ["Cargo.toml"],
            "cell_roots": {"": "//"},
            "synthetic_dependencies": {
                ".github/**": []
            },
            "inert_selection_classes": ["docs/**"],
            "default_base_ref": "origin/dev"
        }"#,
    )
    .expect("pack parses");
    let path = ".github/workflows/oya-ci-required.yml";
    let changes = [Change::Present(path.into())];
    let plan = plan_changes(&changes, &p);
    // Production reality: `owner()` is empty for `.github/**` (it is nobody's declared src).
    let decision = resolve(&plan, &owners(&[(path, &[])]), &p);
    assert_eq!(
        decision,
        Decision::RefuseEmptySelection {
            paths: vec![path.into()]
        },
        "a non-empty diff selecting NO targets must be RED — it is green precisely because it \
         tested nothing"
    );
}

#[test]
fn red_predicate_1_absent_licence_field_is_fail_closed() {
    // The pack field is optional. ABSENT must mean "nothing may be the entire selection", not
    // "everything may" — an unwrap_or_default that silently disables a gate is the standard way
    // this class of assertion rots back into a no-op.
    let p = Policy::from_json(
        r#"{
            "gate_id": "cloud-ci-affected-set",
            "universe": "//...",
            "full_run_targets": ["//..."],
            "full_trigger_patterns": ["**/*.bzl"],
            "require_owner_patterns": ["**/*.rs"],
            "package_definition_basenames": ["BUCK.v2", "BUCK"],
            "package_sibling_basenames": ["Cargo.toml"],
            "cell_roots": {"": "//"},
            "synthetic_dependencies": {"docs/**": []},
            "default_base_ref": "origin/dev"
        }"#,
    )
    .expect("pack parses");
    assert!(p.inert_selection_classes.is_empty());
    let changes = [Change::Present("docs/note.md".into())];
    let plan = plan_changes(&changes, &p);
    assert_eq!(
        resolve(&plan, &owners(&[("docs/note.md", &[])]), &p),
        Decision::RefuseEmptySelection {
            paths: vec!["docs/note.md".into()]
        },
        "an absent `inert_selection_classes` must license NOTHING"
    );
}

#[test]
fn empty_diff_selects_nothing_legitimately() {
    // The GREEN half of predicate (1): the predicate is `changed_files non-empty AND selection
    // empty`. An empty diff has nothing to select and must stay a PASS, not become a refusal.
    let p = policy();
    let plan = plan_changes(&[], &p);
    assert_eq!(resolve(&plan, &BTreeMap::new(), &p), Decision::NoGraphTargets);
}

#[test]
fn docs_only_diff_keeps_its_licence_to_select_nothing_in_the_shipped_pack() {
    // The other GREEN half, against the SHIPPED pack rather than a fixture: docs-only changes may
    // legitimately select nothing, so the shipped `inert_selection_classes` must actually cover
    // them. Without this the assertion would RED every docs PR — a gate that fires on everything
    // is as useless as one that fires on nothing.
    let p = Policy::from_json(
        &fs::read_to_string(shipped_pack_path()).expect("read shipped pack"),
    )
    .expect("shipped pack parses");
    for path in [
        "docs/decisions/ADR-0554-binding-buck2-workspace-coverage.md",
        "README.md",
        "docs/guide/x.mdx",
    ] {
        let changes = [Change::Present(path.into())];
        let plan = plan_changes(&changes, &p);
        assert_eq!(
            resolve(&plan, &owners(&[(path, &[])]), &p),
            Decision::NoGraphTargets,
            "`{path}` is a docs-class change and must keep passing with an empty selection"
        );
    }
}

#[test]
fn shipped_pack_does_not_license_the_github_class_to_select_nothing() {
    // Belt to the github-consumer-coverage gate's braces, from the other direction: even if the
    // `.github/**` seed list were ever emptied, the class holds no licence to be the entire
    // selection, so the PR #1389 outcome is RED rather than green.
    let p = Policy::from_json(
        &fs::read_to_string(shipped_pack_path()).expect("read shipped pack"),
    )
    .expect("shipped pack parses");
    assert!(
        !p.inert_selection_classes
            .iter()
            .any(|c| c.starts_with(".github")),
        "`.github/**` must never hold an empty-selection licence; got {:?}",
        p.inert_selection_classes
    );
}

#[test]
fn unowned_unmapped_path_escalates_to_full() {
    // Defect 2 RED (pre-round-6: silently ignored -> NoGraphTargets/Affected-without-it). An
    // unowned Present path that is NOT owner-required and matches NO synthetic-dependency
    // declaration is derivation uncertainty -> FULL. Here `config/app.yaml` has no owner, is not
    // `.rs`, and is not declared inert -> FULL (a yaml could carry runtime config the cone
    // cannot model; the pack must explicitly declare it inert to skip it).
    let p = policy();
    let changes = [Change::Present("config/app.yaml".into())];
    let plan = plan_changes(&changes, &p);
    let decision = resolve(&plan, &owners(&[("config/app.yaml", &[])]), &p);
    match decision {
        Decision::Full { reasons } => assert!(
            reasons.iter().any(|r| r.contains("config/app.yaml")
                && r.contains("no synthetic-dependency declaration")),
            "FULL reason must name the unmapped path; got {reasons:?}"
        ),
        other => panic!("an unowned unmapped path must escalate to FULL, got {other:?}"),
    }
}

#[test]
fn synthetic_dependency_maps_an_unowned_path_to_declared_seeds() {
    // The other half of defect 2: an unowned path CAN be accounted for by an explicit synthetic
    // dependency that seeds specific targets (e.g. a whole-tree-scanner input declared as a graph
    // edge). Such a path contributes its declared seeds instead of escalating to FULL.
    let p = Policy::from_json(
        r#"{
            "gate_id": "cloud-ci-affected-set",
            "universe": "//...",
            "full_run_targets": ["//..."],
            "full_trigger_patterns": ["**/*.bzl"],
            "require_owner_patterns": ["**/*.rs"],
            "package_definition_basenames": ["BUCK.v2", "BUCK"],
            "package_sibling_basenames": ["Cargo.toml"],
            "cell_roots": {"": "//"},
            "synthetic_dependencies": {
                "policies/**": ["root//ci/facade/policy-lint:policy-lint"]
            },
            "default_base_ref": "origin/dev"
        }"#,
    )
    .expect("pack parses");
    let changes = [Change::Present("policies/tenancy.cedar".into())];
    let plan = plan_changes(&changes, &p);
    let decision = resolve(&plan, &owners(&[("policies/tenancy.cedar", &[])]), &p);
    assert_eq!(
        decision,
        Decision::Affected {
            seeds: vec!["root//ci/facade/policy-lint:policy-lint".into()]
        }
    );
}

#[test]
fn renamed_non_source_file_escalates_to_full_end_to_end() {
    // Defect 1 RED (pre-round-6: a rename was split into Deleted(old)+Present(new); a rename of a
    // non-owner-required file whose destination is owned resolved to AFFECTED, not FULL). Parsed
    // from `git diff --name-status -z`, a rename is a single Structural change -> FULL.
    let p = policy();
    let changes =
        parse_name_status_z("R100\0old/fixtures/data.bin\0new/fixtures/data.bin\0").unwrap();
    let plan = plan_changes(&changes, &p);
    assert!(matches!(
        resolve(&plan, &BTreeMap::new(), &p),
        Decision::Full { .. }
    ));
}

#[test]
fn owned_non_source_asset_closes_the_include_str_seam() {
    // A .md/.json file CAN be a declared src of a target (include_str! assets). Extension
    // pre-filtering was the old shell driver's false-negative hole; the kernel sends EVERY
    // existing file to owner(), so an owned asset becomes a seed.
    let p = policy();
    let changes = [Change::Present("oya/svc/asset/template.md".into())];
    let plan = plan_changes(&changes, &p);
    let decision = resolve(
        &plan,
        &owners(&[("oya/svc/asset/template.md", &["root//oya/svc:oya-svc"])]),
        &p,
    );
    assert_eq!(
        decision,
        Decision::Affected {
            seeds: vec!["root//oya/svc:oya-svc".into()]
        }
    );
}

// ── Escape classes: the rdeps cone cannot model these -> FULL, mechanically ──────────────

#[test]
fn buckconfig_toolchains_third_party_bzl_and_toolchain_pin_escalate_to_full() {
    let p = policy();
    for path in [
        ".buckconfig",
        ".buckconfig.d/extra.bcfg",
        "toolchains/BUCK",
        "toolchains/rust.bzl",
        "third-party/BUCK",
        "third-party/reindeer.toml",
        "third-party/fixups/ring/fixups.toml",
        "infra/macros/defs.bzl",
        "rust-toolchain.toml",
    ] {
        let changes = [Change::Present(path.into())];
        let plan = plan_changes(&changes, &p);
        let decision = resolve(&plan, &BTreeMap::new(), &p);
        match decision {
            Decision::Full { ref reasons } => {
                assert!(
                    reasons.iter().any(|r| r.contains(path)),
                    "FULL reason must name the trigger `{path}`; got {reasons:?}"
                );
            }
            ref other => panic!("`{path}` must escalate to FULL, got {other:?}"),
        }
    }
}

#[test]
fn red_f2_buildfile_and_config_classes_escalate_to_full() {
    // F2 (reviewer-reproduced silent PASS): buck2 honors more buildfile/config names than a
    // single hand-set "BUCK". A NEW BUCK.v2 SHADOWS the BUCK dependents load; a NEW PACKAGE
    // file evaluates to [] (looks like a plain no-owner file); .buckconfig.local is read by
    // buck2 and committable. Each is added by an empty/valid file -> owner() empty -> would be
    // a silent no-op without these classes. ALL must escalate to FULL.
    let p = policy();
    for path in [
        "libs/oya-buck-syntax-kernel/BUCK.v2",
        "ci/facade/affected-target-set/BUCK.v2",
        "libs/oya-thing/PACKAGE",
        "PACKAGE",
        ".buckconfig.local",
    ] {
        let changes = [Change::Present(path.into())];
        let plan = plan_changes(&changes, &p);
        // owner() is empty for buildfile/config files (BY DESIGN) — pass an empty owner map to
        // prove the FULL escalation comes from classification, not from any owner result.
        let decision = resolve(&plan, &owners(&[(path, &[])]), &p);
        match decision {
            Decision::Full { ref reasons } => assert!(
                reasons.iter().any(|r| r.contains(path)),
                "FULL reason must name `{path}`; got {reasons:?}"
            ),
            ref other => {
                panic!("`{path}` (buildfile/config class) must escalate to FULL, got {other:?}")
            }
        }
    }
}

#[test]
fn red_f2_buck_v2_precedence_first_in_basenames() {
    // The basename list is PRECEDENCE-ordered: BUCK.v2 before BUCK (BUCK.v2 shadows BUCK in
    // buck2). The engine treats either as a buildfile -> FULL; this pins the ground-truth order
    // in the shipped pack so a future editor cannot silently drop BUCK.v2.
    let p = policy();
    assert_eq!(
        p.package_definition_basenames.first().map(String::as_str),
        Some("BUCK.v2"),
        "BUCK.v2 must be first (it shadows BUCK)"
    );
    assert!(p.package_definition_basenames.iter().any(|b| b == "BUCK"));
}

#[test]
fn deleted_source_file_escalates_to_full() {
    // owner() cannot resolve a path that no longer exists at HEAD, but deleting a source can
    // break every dependent of its former target — mechanical escalation, never skip.
    let p = policy();
    let changes = [Change::Deleted("libs/oya-thing/src/gone.rs".into())];
    let plan = plan_changes(&changes, &p);
    assert!(matches!(
        resolve(&plan, &BTreeMap::new(), &p),
        Decision::Full { .. }
    ));
}

#[test]
fn deleted_package_definition_escalates_to_full() {
    let p = policy();
    let changes = [Change::Deleted("libs/oya-thing/BUCK".into())];
    let plan = plan_changes(&changes, &p);
    assert!(matches!(
        resolve(&plan, &BTreeMap::new(), &p),
        Decision::Full { .. }
    ));
}

#[test]
fn deleted_doc_escalates_to_full_safe_version() {
    // Defect 1 (safe version): EVERY deletion escalates to FULL, including a doc — a deletion's
    // blast radius is not bounded by the head-only rdeps cone (owner() cannot resolve a gone
    // path). The pre-round-6 doc-deletion optimization is deliberately traded for soundness now;
    // a later base+head owner-graph union may restore it.
    let p = policy();
    let changes = [Change::Deleted("docs/old-note.md".into())];
    let plan = plan_changes(&changes, &p);
    assert!(matches!(
        resolve(&plan, &BTreeMap::new(), &p),
        Decision::Full { .. }
    ));
}

// ── Buildfile changes escalate to FULL (blast radius exceeds the package's own rdeps) ────

#[test]
fn modified_buck_file_escalates_to_full() {
    // A BUCK edit can add/remove targets or rewire deps that arbitrary OTHER packages resolve;
    // seeding only "its own package" (the previous, wrong behavior) missed those dependents.
    let p = policy();
    let changes = [Change::Present(
        "cloud/cloud-iam/crates/oya-iam/BUCK".into(),
    )];
    let plan = plan_changes(&changes, &p);
    match resolve(&plan, &BTreeMap::new(), &p) {
        Decision::Full { reasons } => assert!(
            reasons.iter().any(|r| r.contains("oya-iam/BUCK")),
            "FULL reason must name the buildfile; got {reasons:?}"
        ),
        other => panic!("a BUCK change must escalate to FULL, got {other:?}"),
    }
}

// ── Graph-invisible code REFUSES (running more targets cannot make it safe) ──────────────

#[test]
fn unowned_source_file_refuses_instead_of_passing_or_fulling() {
    let p = policy();
    let changes = [
        Change::Present("oya/new-svc/src/lib.rs".into()),
        // Even with a co-present full trigger, refusal must dominate: a full run would not
        // compile the unowned file, so FULL would still be a false-green for it.
        Change::Present("third-party/reindeer.toml".into()),
    ];
    let plan = plan_changes(&changes, &p);
    let decision = resolve(&plan, &owners(&[("oya/new-svc/src/lib.rs", &[])]), &p);
    assert_eq!(
        decision,
        Decision::RefuseUnowned {
            paths: vec!["oya/new-svc/src/lib.rs".into()]
        }
    );
}

#[test]
fn cargo_manifest_seeds_its_enclosing_package() {
    // Manifests are not buck2 graph inputs (no owner() BY DESIGN — proven by this lane's own
    // first dogfood run, which refused on its own crate's Cargo.toml under an owner-required
    // pack). They are semantically bound to their crate: seed the enclosing package pattern,
    // exactly like a BUCK edit. A manifest in a package-less dir makes the seed query fail
    // downstream -> the adapter escalates to FULL.
    let p = policy();
    let changes = [Change::Present(
        "oya/svc/crates/oya-svc-app/Cargo.toml".into(),
    )];
    let plan = plan_changes(&changes, &p);
    let decision = resolve(&plan, &BTreeMap::new(), &p);
    assert_eq!(
        decision,
        Decision::Affected {
            seeds: vec!["//oya/svc/crates/oya-svc-app:".into()]
        }
    );
}

#[test]
fn deleted_cargo_manifest_escalates_to_full() {
    let p = policy();
    let changes = [Change::Deleted(
        "oya/svc/crates/oya-svc-app/Cargo.toml".into(),
    )];
    let plan = plan_changes(&changes, &p);
    assert!(matches!(
        resolve(&plan, &BTreeMap::new(), &p),
        Decision::Full { .. }
    ));
}

#[test]
fn build_script_is_a_package_sibling_not_a_refusal() {
    // build.rs is cargo-side crate metadata (live-repo audit 2026-06-12: the only first-party
    // build.rs has no buck2 owner) — owner-requiring it would be a refusal landmine; it seeds
    // the enclosing package like Cargo.toml.
    let p = policy();
    let changes = [Change::Present(
        "oya/svc/crates/oya-svc-app/build.rs".into(),
    )];
    let plan = plan_changes(&changes, &p);
    assert_eq!(
        resolve(&plan, &BTreeMap::new(), &p),
        Decision::Affected {
            seeds: vec!["//oya/svc/crates/oya-svc-app:".into()]
        }
    );
}

#[test]
fn red_f1_owned_kernel_source_lands_in_its_cone_no_exemption() {
    // F1 (reviewer-reproduced, the bad one): the prior pack out-of-graph-exempted
    // cloud/cloud-kernel/** — FACTUALLY FALSE (the cited
    // oya-cloud-kernel-user-layout-kernel/src/lib.rs is owned by two host-buildable Buck2
    // targets). The exemption made an OWNED .rs break PASS as NO-GRAPH-TARGETS — the exact
    // cf16525 class, reintroduced by my own pack. The exemption is DELETED: an owned kernel
    // source is an ordinary owner-required file and lands in its cone as a seed.
    let p = policy();
    let path = "cloud/cloud-kernel/crates/oya-cloud-kernel-user-layout-kernel/src/lib.rs";
    let changes = [Change::Present(path.into())];
    let plan = plan_changes(&changes, &p);
    let decision = resolve(
        &plan,
        &owners(&[(
            path,
            &[
                "root//cloud/cloud-kernel/crates/oya-cloud-kernel-user-layout-kernel:oya-cloud-kernel-user-layout-kernel",
                "root//cloud/cloud-kernel/crates/oya-cloud-kernel-user-layout-kernel:host-tests",
            ],
        )]),
        &p,
    );
    match decision {
        Decision::Affected { seeds } => assert!(
            seeds
                .iter()
                .any(|s| s.contains("oya-cloud-kernel-user-layout-kernel")),
            "owned kernel source must seed its target; got {seeds:?}"
        ),
        other => panic!("owned kernel source must be Affected (not exempted), got {other:?}"),
    }
}

#[test]
fn bare_metal_kernel_backend_without_buck2_platform_refuses_until_wired() {
    // The bare-metal arch backends intentionally have no placeholder Buck2 target until the
    // repo has a real bare-metal Rust platform/toolchain. That must fail closed: touching their
    // owner-required Rust source REFUSES as unowned instead of silently passing or exposing an
    // incompatible target that wildcard builds skip.
    let p = policy();
    let path = "cloud/cloud-kernel/crates/oya-cloud-kernel-arch-x86-64-adapter/src/lib.rs";
    let changes = [Change::Present(path.into())];
    let plan = plan_changes(&changes, &p);
    let decision = resolve(&plan, &owners(&[(path, &[])]), &p);
    assert_eq!(
        decision,
        Decision::RefuseUnowned {
            paths: vec![path.into()]
        }
    );
}

#[test]
fn bare_metal_kernel_linker_script_without_buck2_platform_refuses_until_wired() {
    // The arch linker scripts are part of the same bare-metal surface as the backend Rust code.
    // Without an exact owner-required pattern they would be unowned but not owner-required,
    // reopening the no-graph-targets false-green seam for linker-script edits.
    let p = policy();
    let path = "cloud/cloud-kernel/crates/oya-cloud-kernel-arch-x86-64-adapter/linker.ld";
    let changes = [Change::Present(path.into())];
    let plan = plan_changes(&changes, &p);
    let decision = resolve(&plan, &owners(&[(path, &[])]), &p);
    assert_eq!(
        decision,
        Decision::RefuseUnowned {
            paths: vec![path.into()]
        }
    );
}

#[test]
fn genuinely_unowned_kernel_userspace_source_refuses() {
    // The real residual after deleting the F1 exemption: the per-arch userspace sub-crates
    // (own Cargo.toml + own rust-toolchain.toml, not globbed by the parent BUCK) are genuinely
    // unowned. They REFUSE on touch (engine's existing handling) — never a silent PASS, never a
    // hand exemption that also swallows owned siblings.
    let p = policy();
    let path =
        "cloud/cloud-kernel/crates/oya-cloud-kernel-arch-x86-64-adapter/user-src/src/main.rs";
    let changes = [Change::Present(path.into())];
    let plan = plan_changes(&changes, &p);
    let decision = resolve(&plan, &owners(&[(path, &[])]), &p);
    assert_eq!(
        decision,
        Decision::RefuseUnowned {
            paths: vec![path.into()]
        }
    );
}

#[test]
fn unwired_test_file_refusal_is_the_fix_on_touch_forcing_function() {
    // Live-repo audit (2026-06-12, dev tip): 100 tracked tests/examples/benches .rs files
    // have NO owning rust_test target (the ADR-0540 member_test_code_without_rust_test_target
    // debt class) — those tests run NOWHERE. A green REQUIRED context over an edit to a test
    // that never executes is the silent variant of the cf16525 class, so touching one REFUSES
    // until the file is wired into a BUCK target (fix-on-touch ratchet).
    let p = policy();
    let changes = [Change::Present(
        "cloud/svc/crates/oya-svc-sdk/tests/contract.rs".into(),
    )];
    let plan = plan_changes(&changes, &p);
    let decision = resolve(
        &plan,
        &owners(&[("cloud/svc/crates/oya-svc-sdk/tests/contract.rs", &[])]),
        &p,
    );
    assert_eq!(
        decision,
        Decision::RefuseUnowned {
            paths: vec!["cloud/svc/crates/oya-svc-sdk/tests/contract.rs".into()]
        }
    );
}

// ── Determinism + dedup (same inputs -> same decided set, stable order) ──────────────────

#[test]
fn seeds_are_sorted_and_deduplicated() {
    let p = policy();
    let changes = [
        Change::Present("oya/b/src/lib.rs".into()),
        Change::Present("oya/a/src/lib.rs".into()),
        Change::Present("oya/a/src/other.rs".into()),
    ];
    let plan = plan_changes(&changes, &p);
    let owner_map = owners(&[
        ("oya/b/src/lib.rs", &["root//oya/b:b"]),
        ("oya/a/src/lib.rs", &["root//oya/a:a"]),
        ("oya/a/src/other.rs", &["root//oya/a:a"]),
    ]);
    let decision = resolve(&plan, &owner_map, &p);
    assert_eq!(
        decision,
        Decision::Affected {
            seeds: vec!["root//oya/a:a".into(), "root//oya/b:b".into()]
        }
    );
}

// ── Pack/engine pairing + the shipped oyatie pack stays loadable ─────────────────────────

#[test]
fn shipped_pack_parses_and_matches_the_engine() {
    // Locate the shipped pack relative to the repo root (walk-up mirrors the firewall gate's
    // resolver; buck2 runs tests inside the repo).
    let pack = shipped_pack_path();
    let bytes = fs::read_to_string(&pack).expect("read shipped pack");
    let p = Policy::from_json(&bytes).expect("shipped pack must satisfy the engine schema");
    assert_eq!(p.gate_id, GATE_ID);
    assert_eq!(p.universe, "//...");
    assert!(
        p.full_trigger_patterns
            .iter()
            .any(|t| t == "third-party/**")
    );
    assert!(p.require_owner_patterns.iter().any(|t| t == "**/*.rs"));
    assert!(p.require_owner_patterns.iter().any(|t| {
        t == "cloud/cloud-kernel/crates/oya-cloud-kernel-arch-aarch64-adapter/linker.ld"
    }));
    assert!(p.require_owner_patterns.iter().any(|t| {
        t == "cloud/cloud-kernel/crates/oya-cloud-kernel-arch-x86-64-adapter/linker.ld"
    }));
    assert!(
        p.package_sibling_basenames
            .iter()
            .any(|t| t == "Cargo.toml")
    );
    assert!(p.package_sibling_basenames.iter().any(|t| t == "build.rs"));
    // F2: ground-truth buildfile names, BUCK.v2 first (it shadows BUCK).
    assert_eq!(
        p.package_definition_basenames.first().map(String::as_str),
        Some("BUCK.v2")
    );
    assert!(p.package_definition_basenames.iter().any(|t| t == "BUCK"));
    // F2: PACKAGE + .buckconfig.local are escape triggers in the shipped pack.
    assert!(p.full_trigger_patterns.iter().any(|t| t == "**/PACKAGE"));
    assert!(
        p.full_trigger_patterns
            .iter()
            .any(|t| t == ".buckconfig.local")
    );
    // F1: there is NO out-of-graph path exemption (the cloud-kernel exemption was unsound).
    assert!(
        !bytes.contains("out_of_graph_roots"),
        "the unsound out_of_graph_roots exemption must not reappear in the pack"
    );
}

#[test]
fn archive_epoch_e4_inputs_seed_cross_artifact_agreement_gate_despite_docs_inert_mapping() {
    // E4 archive inputs are cross-artifact-agreement inputs, even though the generic docs
    // declaration is inert. Synthetic declarations union, so the narrow non-empty archive
    // mapping contributes this gate while `docs/**` contributes no seed.
    let pack = shipped_pack_path();
    let policy = Policy::from_json(&fs::read_to_string(pack).expect("read shipped pack"))
        .expect("shipped pack parses");
    let expected =
        vec!["root//ci/facade/cross-artifact-agreement:ci-cross-artifact-agreement-gate".into()];

    for path in [
        "docs/ideas/archive/e4-transition.md",
        "specs/markdown-retirement-policy.json",
    ] {
        let changes = [Change::Present(path.into())];
        let decision = resolve(
            &plan_changes(&changes, &policy),
            &owners(&[(path, &[])]),
            &policy,
        );
        assert_eq!(
            decision,
            Decision::Affected {
                seeds: expected.clone()
            }
        );
    }
}

#[test]
fn wrong_pack_for_engine_is_rejected() {
    let err = Policy::from_json(r#"{"gate_id": "cloud-ci-something-else"}"#).unwrap_err();
    assert!(format!("{err}").contains("wrong pack"));
}
