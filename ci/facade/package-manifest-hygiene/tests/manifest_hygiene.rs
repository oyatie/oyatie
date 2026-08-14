// §2.5#7 cloud-ci-manifest-hygiene: born-blocking self-test over TODAY's real corpus. Runs the
// producer `--face manifest-hygiene` to resolve the per-crate manifest flags, then asserts the
// gate FIRES — some first-party oya-* crates miss a §2.5#7 field today (the frozen baseline,
// shrink-only). The count is MEASURED, not hardcoded. ADR-0083 Tier-3: integration tests assert
// via unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use oya_ci_config_kernel::NamingConfig;
use serde_json::Value;

use ci_package_manifest_hygiene::{Verdict, evaluate, evaluate_keyed};

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn producer_binary(root: &Path, producer_bin: Option<&str>) -> Result<PathBuf, String> {
    let Some(bin) = producer_bin else {
        return Err(
            "FAIL-CLOSED: missing OYA_CI_PRODUCER_BIN; Cargo fallback is forbidden".to_owned(),
        );
    };
    Ok(if Path::new(bin).is_absolute() {
        PathBuf::from(bin)
    } else {
        root.join(bin)
    })
}

#[test]
fn producer_binary_env_is_required_for_hermetic_gate() {
    let err = producer_binary(Path::new("/repo"), None)
        .expect_err("missing OYA_CI_PRODUCER_BIN must fail closed");
    assert!(err.contains("OYA_CI_PRODUCER_BIN"));
}

/// Run the producer to emit a single face to stdout, HERMETICALLY. The producer binary must be
/// provided by `OYA_CI_PRODUCER_BIN`; missing env fails closed so tests cannot silently fall back to
/// Cargo. The producer reads the materialized scm-facts face (a declared input); it never calls git.
fn run_producer_face(root: &Path, face: &str) -> Value {
    let scm_facts = root
        .join("ci/facade/artifact-inventory-registry/scm-facts.generated.json");
    let producer_bin = std::env::var("OYA_CI_PRODUCER_BIN").ok();
    let bin = producer_binary(root, producer_bin.as_deref()).unwrap_or_else(|e| panic!("{e}"));
    let output = Command::new(bin)
        .arg("--repo-root")
        .arg(root)
        .arg("--scm-facts")
        .arg(&scm_facts)
        .arg("--stdout")
        .arg("--face")
        .arg(face)
        .current_dir(root)
        .output()
        .expect("run producer binary");
    assert!(
        output.status.success(),
        "producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("producer face stdout is valid JSON")
}

/// INDEPENDENT dynamic census of today's oya-*-prefixed workspace member crates: resolved via
/// the canonical `oya_workspace_members_kernel::resolve_member_dirs` (NOT the producer's own
/// `collect_bnf_layer_suffix`/`collect_manifest_hygiene` path, and NOT applying
/// `is_path_excluded` — that config-driven exclusion is itself a SECOND silent-drop vector the
/// producer applies; a census that doesn't re-apply it will correctly MISMATCH if an exclusion
/// rule is ever mis-scoped to drop a real crate), with its OWN from-scratch `[package] name`
/// parse (see `independent_parse_package_name`). FAILS CLOSED: a resolved member directory whose
/// Cargo.toml is unreadable or carries no `[package] name` is a hard test failure (panic), never
/// a silent skip — the exact bug class (scan-root/glob/prefix/parse/truncation regression,
/// silently dropping an eligible crate) a magic-number floor, or a bare non-empty check, could
/// never catch. Self-adjusts through future de-brands: the census shrinks in lockstep with the
/// face as crates lose the oya- prefix, so this stays valid without ever needing a bump.
///
/// KEYED BY MANIFEST PATH, not package name. A rehomed destination crate can share its package
/// name with the retained legacy source (integ/procurement absorb, PR #1672); a name-keyed
/// census would collapse both into one key and let the face's sorted-later legacy row mask the
/// destination manifest, so the live-corpus test could pass without checking the newly tracked
/// manifest. Path keys preserve identity: the census asserts the face carries EVERY manifest
/// (root members + nested-workspace members + `app/<product>/crates/*` destinations + excluded
/// retained-source crate dirs) exactly once, with its own flags.
fn independent_oya_prefix_census(root: &Path) -> BTreeSet<String> {
    let prefix = NamingConfig::default().required_prefix;
    let mut member_dirs = oya_workspace_members_kernel::resolve_member_dirs(root)
        .expect("resolve_member_dirs must resolve the live workspace Cargo.toml");
    member_dirs.extend(resolve_nested_workspace_member_dirs(root));
    member_dirs.extend(resolve_app_destination_crate_dirs(root));
    member_dirs.extend(resolve_excluded_source_crate_dirs(root));
    let mut manifests = BTreeSet::new();
    for dir in member_dirs {
        let manifest_path = root.join(&dir).join("Cargo.toml");
        let contents = std::fs::read_to_string(&manifest_path).unwrap_or_else(|e| {
            panic!(
                "independent census FAIL-CLOSED: unreadable manifest {} ({e}) — a workspace \
                 member MUST have a readable Cargo.toml",
                manifest_path.display()
            )
        });
        let name = independent_parse_package_name(&contents).unwrap_or_else(|| {
            panic!(
                "independent census FAIL-CLOSED: no [package] name in {}",
                manifest_path.display()
            )
        });
        if name.starts_with(&prefix) {
            manifests.insert(format!("{dir}/Cargo.toml"));
        }
    }
    manifests
}

/// Nested-workspace roots this repo carves out of the root workspace (`[workspace].exclude`
/// entries that are THEMSELVES a `[workspace]` root — e.g. `cloud/cloud-kernel`'s bare-metal
/// no_std workspace, `kernel/`'s ADR-0512 rung-0 carve-out): their first-party oya-* crates are
/// real, tracked, prefix-matching manifests the PRODUCER's tracked-Cargo.toml scan legitimately
/// includes (it doesn't care which cargo workspace a Cargo.toml belongs to), so the census must
/// resolve them too or it silently under-counts relative to the face. Discovered from the root
/// manifest's OWN `exclude` list (not a hardcoded dir list) — self-adjusts if a future carve-out
/// is added or removed; an excluded entry with no `[workspace]` Cargo.toml (e.g. a buck2-only
/// gate with no Cargo.toml at all) is simply skipped, not a nested workspace.
fn resolve_nested_workspace_member_dirs(root: &Path) -> Vec<String> {
    let root_manifest =
        std::fs::read_to_string(root.join("Cargo.toml")).expect("read root Cargo.toml");
    let mut dirs = Vec::new();
    for excluded in root_workspace_excludes(&root_manifest) {
        let nested_root = root.join(&excluded);
        let Ok(nested_text) = std::fs::read_to_string(nested_root.join("Cargo.toml")) else {
            continue;
        };
        if !nested_text.contains("[workspace]") {
            continue; // excluded for a different reason (no Cargo.toml / not a workspace root).
        }
        let members = oya_workspace_members_kernel::resolve_member_dirs_from_str(
            &nested_text,
            &nested_root,
        )
        .expect("resolve nested workspace members");
        dirs.extend(members.into_iter().map(|m| format!("{excluded}/{m}")));
    }
    dirs
}

/// ADR-0562 app-product destination crate dirs (`app/<product>/crates/<leaf>`, the forever home
/// for absorbed products while the legacy oya/ source stays live or the root workspace keeps them
/// excluded until the drain). These are REAL first-party oya-* manifests the PRODUCER's
/// tracked-Cargo.toml scan legitimately enumerates, but they are NOT root-workspace members
/// (ADR-0538: app/*/crates membership flips only on the integ/oya drain), so the census must
/// resolve them explicitly or it silently under-counts relative to the face (the exact
/// extra_in_face mismatch an absorb lands when the legacy source was already evicted). Discovered
/// by a flat `app/*/crates/*` walk, not a hardcoded product list — self-adjusts as products
/// absorb; a missing `app/` root is simply skipped (repo-portable). A crate dir whose Cargo.toml
/// is unreadable or carries no `[package] name` is left to the census's own FAIL-CLOSED loop
/// below (never a silent skip).
fn resolve_app_destination_crate_dirs(root: &Path) -> Vec<String> {
    let mut dirs = Vec::new();
    let Ok(app_root) = std::fs::read_dir(root.join("app")) else {
        return dirs; // no app/ destination tree in this checkout.
    };
    for product in app_root.filter_map(Result::ok) {
        if !product.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let Ok(crates_root) = std::fs::read_dir(product.path().join("crates")) else {
            continue; // an app product with no destination crates yet.
        };
        for crate_dir in crates_root.filter_map(Result::ok) {
            if !crate_dir.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            if !crate_dir.path().join("Cargo.toml").is_file() {
                continue; // not a crate dir (no manifest) — skipped, like the nested-workspace rule.
            }
            let name = crate_dir.file_name().to_string_lossy().into_owned();
            let product_name = product.file_name().to_string_lossy().into_owned();
            dirs.push(format!("app/{product_name}/crates/{name}"));
        }
    }
    dirs
}

/// Excluded tracked crate dirs that are NOT nested-workspace roots — retained legacy source
/// copies parked under `[workspace].exclude` until their shrink-only drain (e.g.
/// `oya/crm/crates/oya-procurement-source-to-pay-domain`, the integ/procurement source kept
/// until integ/oya deletes it). They are real first-party oya-* manifests the PRODUCER's
/// tracked-Cargo.toml scan includes (path_excludes does not cover them), but they are neither
/// root members nor nested-workspace members, so the census must resolve them explicitly or it
/// silently under-counts relative to the face — and with the face now keyed by manifest path the
/// census MUST carry each such copy as its own key (a name-keyed census would collapse it into
/// its same-named destination and mask the destination's flags). Discovered from the root
/// manifest's OWN `exclude` list: an excluded entry whose directory holds a Cargo.toml that
/// declares a `[package]` is a candidate crate dir; virtual workspace roots (`kernel/`,
/// `cloud/cloud-kernel` — workspace tables but no `[package]`) are skipped here rather than
/// failing the census's own parse, exactly as they are not crate manifests.
fn resolve_excluded_source_crate_dirs(root: &Path) -> Vec<String> {
    let root_manifest =
        std::fs::read_to_string(root.join("Cargo.toml")).expect("read root Cargo.toml");
    let mut dirs = Vec::new();
    for excluded in root_workspace_excludes(&root_manifest) {
        let manifest_path = root.join(&excluded).join("Cargo.toml");
        let Ok(contents) = std::fs::read_to_string(&manifest_path) else {
            continue; // not a crate dir (no manifest) — skipped, like the nested-workspace rule.
        };
        if independent_parse_package_name(&contents).is_some() {
            dirs.push(excluded);
        }
    }
    dirs
}

/// Minimal, from-scratch extraction of the root workspace's `[workspace] exclude = [...]` array
/// — a plain independent text scan, not a TOML-library parse, so a bug in that shared parser
/// cannot hide behind the census reusing it. `#`-comments are stripped PER LINE before any
/// bracket detection, so a comment mentioning a bracket (e.g. "...its own [workspace]...")
/// can never be mistaken for the array's own structural delimiter.
fn root_workspace_excludes(manifest_text: &str) -> Vec<String> {
    let mut in_exclude = false;
    let mut body = String::new();
    for raw in manifest_text.lines() {
        let line = raw.split('#').next().unwrap_or("");
        if !in_exclude {
            if line.trim_start().starts_with("exclude") && line.contains('[') {
                in_exclude = true;
                if let Some(after) = line.splitn(2, '[').nth(1) {
                    body.push_str(after);
                    body.push('\n');
                    if after.contains(']') {
                        in_exclude = false;
                    }
                }
            }
            continue;
        }
        body.push_str(line);
        body.push('\n');
        if line.contains(']') {
            break;
        }
    }
    body.split('"')
        .enumerate()
        .filter_map(|(i, seg)| (i % 2 == 1).then(|| seg.to_owned()))
        .collect()
}

/// Minimal, deliberately SEPARATE `[package] name` parse — NOT a call into the producer's own
/// `parse_package_name`. The point of "independent" is that a bug in the shared parser cannot
/// hide behind the census reusing the same buggy code.
fn independent_parse_package_name(contents: &str) -> Option<String> {
    let mut in_package = false;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package
            && let Some(rest) = trimmed.strip_prefix("name")
            && let Some(rest) = rest.trim_start().strip_prefix('=')
        {
            let value = rest.trim().trim_matches('"');
            if !value.is_empty() {
                return Some(value.to_owned());
            }
        }
    }
    None
}

/// Assert the face's enumerated manifest-path set exactly equals the independent census, with a
/// diagnostic naming exactly which keys are missing/extra on mismatch.
fn assert_census_matches(face_names: &BTreeSet<String>, census: &BTreeSet<String>) {
    let missing_from_face: Vec<&String> = census.difference(face_names).collect();
    let extra_in_face: Vec<&String> = face_names.difference(census).collect();
    assert!(
        missing_from_face.is_empty() && extra_in_face.is_empty(),
        "face/census SET MISMATCH — missing_from_face={missing_from_face:?} \
         extra_in_face={extra_in_face:?}"
    );
}

#[test]
fn manifest_hygiene_is_born_blocking_on_the_live_corpus() {
    let root = repo_root();
    let face = run_producer_face(&root, "manifest-hygiene");
    let rows = face["rows"].as_array().expect("manifest-hygiene face rows");
    // Face is keyed by MANIFEST PATH (not package name): a rehomed destination crate shares its
    // name with the retained legacy source, and a name-keyed comparison would collapse both and
    // let the sorted-later legacy row mask the destination's flags (review thread 3783908720).
    // Each row must carry its manifest_path; a missing one fails closed.
    let face_names: BTreeSet<String> = rows
        .iter()
        .map(|r| {
            r["manifest_path"]
                .as_str()
                .expect("manifest-hygiene row manifest_path")
                .to_owned()
        })
        .collect();

    // INDEPENDENT DYNAMIC CENSUS (not a hardcoded magnitude floor, and not a bare non-empty
    // check): re-derive the live oya-* crate manifest set via the canonical workspace-member
    // resolver + a from-scratch parse, then assert EXACT set equality against the face.
    // Self-adjusts through future de-brands (the census shrinks in lockstep with the face) while
    // catching scan-root/glob/prefix/parse/truncation/exclusion regressions a magnitude floor
    // cannot: a producer that silently drops even ONE eligible manifest is caught as a set
    // difference, not masked by "some debt survived."
    let census = independent_oya_prefix_census(&root);
    assert_census_matches(&face_names, &census);

    let findings = evaluate_keyed(&face);
    eprintln!(
        "BORN-BLOCKING manifest-hygiene: oya-* crates={} total_findings={}",
        rows.len(),
        findings.len()
    );

    assert_eq!(
        evaluate(&face).verdict,
        Verdict::Red,
        "GATE must go RED on today's corpus (some crates miss a §2.5#7 field)"
    );
    assert!(
        !findings.is_empty(),
        "the live corpus must surface at least one manifest-hygiene violation"
    );
}

// --- assert_census_matches: RED-test the vacuous cases a bare `rows.len() > 0` check is blind
// to ("some debt survived" is not the same fact as "the corpus was enumerated"). ---

#[test]
#[should_panic(expected = "SET MISMATCH")]
fn census_mismatch_is_caught_when_face_is_empty_but_census_is_not() {
    let face_names: BTreeSet<String> = BTreeSet::new();
    let census: BTreeSet<String> = ["oya-a-domain".to_string()].into_iter().collect();
    assert_census_matches(&face_names, &census);
}

#[test]
#[should_panic(expected = "SET MISMATCH")]
fn census_mismatch_is_caught_when_face_is_near_empty() {
    // The exact vacuous-case bug class a bare `rows.len() > 0` check is blind to: one surviving
    // row ("some debt survived") is not the same fact as "the corpus was enumerated."
    let face_names: BTreeSet<String> = ["oya-a-domain".to_string()].into_iter().collect();
    let census: BTreeSet<String> = ["oya-a-domain", "oya-b-domain", "oya-c-domain"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    assert_census_matches(&face_names, &census);
}

#[test]
#[should_panic(expected = "SET MISMATCH")]
fn census_mismatch_is_caught_when_exactly_one_crate_is_missing() {
    let face_names: BTreeSet<String> = ["oya-a-domain", "oya-b-domain"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    let census: BTreeSet<String> = ["oya-a-domain", "oya-b-domain", "oya-c-domain"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    assert_census_matches(&face_names, &census);
}

#[test]
fn census_match_is_green_when_sets_are_equal() {
    let names: BTreeSet<String> = ["oya-a-domain", "oya-b-domain"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    assert_census_matches(&names, &names.clone());
}

// --- independent_oya_prefix_census: fixture-driven fail-closed proof (unreadable/unparseable
// eligible manifest) + a happy-path resolve+filter proof. ---

fn census_tmp_root(tag: &str) -> PathBuf {
    let unique = format!("manifest-hygiene-census-{tag}-{}", std::process::id());
    let root = std::env::temp_dir().join(unique);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");
    root
}

#[test]
#[should_panic(expected = "unreadable manifest")]
fn independent_census_fails_closed_on_unreadable_member_manifest() {
    // `resolve_member_dirs` fails when a matched directory has no Cargo.toml. This fixture tests
    // the next boundary: the manifest exists for membership resolution but is unreadable when the
    // census parses it. A deliberately unreadable oya-* manifest must RED the census.
    let root = census_tmp_root("unreadable");
    let manifest_path = root.join("crates/oya-ghost-domain/Cargo.toml");
    std::fs::create_dir_all(manifest_path.parent().unwrap()).expect("mkdir");
    std::fs::write(&manifest_path, "[package]\nname = \"oya-ghost-domain\"\n")
        .expect("write member manifest");
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&manifest_path, std::fs::Permissions::from_mode(0o000))
        .expect("chmod 000");
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/*\"]\n",
    )
    .expect("write root manifest");
    let _ = independent_oya_prefix_census(&root);
}

#[test]
#[should_panic(expected = "no [package] name")]
fn independent_census_fails_closed_on_unparseable_member_manifest() {
    let root = census_tmp_root("unparseable");
    std::fs::create_dir_all(root.join("crates/oya-broken-domain")).expect("mkdir");
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/*\"]\n",
    )
    .expect("write root manifest");
    std::fs::write(
        root.join("crates/oya-broken-domain/Cargo.toml"),
        "[dependencies]\n",
    )
    .expect("write member manifest");
    let _ = independent_oya_prefix_census(&root);
}

#[test]
fn independent_census_resolves_and_filters_a_small_fixture() {
    let root = census_tmp_root("happy");
    std::fs::create_dir_all(root.join("crates/oya-a-domain")).expect("mkdir");
    std::fs::create_dir_all(root.join("crates/other-b-domain")).expect("mkdir");
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/*\"]\n",
    )
    .expect("write root manifest");
    std::fs::write(
        root.join("crates/oya-a-domain/Cargo.toml"),
        "[package]\nname = \"oya-a-domain\"\n",
    )
    .expect("write member manifest");
    std::fs::write(
        root.join("crates/other-b-domain/Cargo.toml"),
        "[package]\nname = \"other-b-domain\"\n",
    )
    .expect("write member manifest");
    let census = independent_oya_prefix_census(&root);
    assert_eq!(
        census,
        ["crates/oya-a-domain/Cargo.toml".to_string()]
            .into_iter()
            .collect()
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Codex P1 on PR #1672 (r3783908720): a destination `app/<product>/crates/<pkg>` copy that
/// retains the same package name as its excluded legacy source must occupy TWO census keys.
/// A name-keyed `BTreeSet` would collapse them into one and let the live-corpus equality
/// pass without enumerating the newly tracked destination manifest.
#[test]
fn independent_census_keeps_same_named_destination_and_excluded_source() {
    let root = census_tmp_root("dup-name");
    std::fs::create_dir_all(root.join("app/prod/crates/oya-dup-domain")).expect("mkdir dest");
    std::fs::create_dir_all(root.join("oya/crm/crates/oya-dup-domain")).expect("mkdir source");
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = []\nexclude = [\"oya/crm/crates/oya-dup-domain\"]\n",
    )
    .expect("write root manifest");
    let package = "[package]\nname = \"oya-dup-domain\"\n";
    std::fs::write(
        root.join("app/prod/crates/oya-dup-domain/Cargo.toml"),
        package,
    )
    .expect("write destination manifest");
    std::fs::write(
        root.join("oya/crm/crates/oya-dup-domain/Cargo.toml"),
        package,
    )
    .expect("write retained source manifest");
    let census = independent_oya_prefix_census(&root);
    assert_eq!(
        census,
        [
            "app/prod/crates/oya-dup-domain/Cargo.toml".to_string(),
            "oya/crm/crates/oya-dup-domain/Cargo.toml".to_string(),
        ]
        .into_iter()
        .collect()
    );
    let _ = std::fs::remove_dir_all(&root);
}
