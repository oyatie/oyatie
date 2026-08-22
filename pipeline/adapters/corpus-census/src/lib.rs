//! Shared INDEPENDENT corpus census for the `ci/facade` gate self-tests.
//!
//! # Why this crate exists
//!
//! A gate that enumerates the repo cannot check its own enumeration. `rows.len() > 0` says "some
//! debt survived", which is not the same fact as "the corpus was enumerated" — a producer that
//! silently drops all but one eligible crate passes it. A frozen magnitude floor is worse: it
//! rots, and it is unattributable when it trips.
//!
//! The control that works is a SECOND, INDEPENDENT derivation of the corpus compared as a SET,
//! so a partial collapse surfaces as a reviewable diff of named keys. Exactly two gates in the
//! fleet were rated CENSUS for carrying it — `ci/facade/package-manifest-hygiene` (§2.5#7) and
//! `ci/facade/crate-layer-suffix` (§2.5#4) — and they carried it as two VERBATIM copies of the
//! same ~130 lines, proofs included. This crate is the single copy.
//!
//! # What "independent" means here
//!
//! Independent of the PRODUCER, deliberately, on three axes:
//!
//! * member resolution goes through the canonical `workspace_members_kernel`, not the
//!   producer's own tracked-`Cargo.toml` scan;
//! * `[package] name` is re-parsed from scratch by [`independent_parse_package_name`], not by
//!   calling the producer's parser — a bug in a shared parser must not be able to hide behind
//!   the census reusing it;
//! * the producer's config-driven `is_path_excluded` is NOT re-applied. That exclusion is itself
//!   a second silent-drop vector; a census that skipped it too would agree with a mis-scoped
//!   exclusion rule instead of catching it.
//!
//! # FAIL-CLOSED
//!
//! A resolved member directory whose `Cargo.toml` is unreadable, or carries no `[package] name`,
//! is a hard panic — never a silent skip. Absence reading as success is the bug class this whole
//! control exists to deny.
//!
//! # Panicking is the contract
//!
//! This is test-support: [`assert_census_matches`] and [`assert_census_covers`] are assertions,
//! and the census fails closed by panicking. ADR-0083 Tier-3 allows unwrap/expect/panic in that
//! role; the allow below is scoped to this crate for that reason and for no other.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ci_config_kernel::NamingConfig;

/// INDEPENDENT dynamic census of today's workspace member crates: repo-relative member directory
/// -> the `[package] name` declared in that directory's `Cargo.toml`.
///
/// This is the primitive the two named-set censuses are built on; take it directly when a gate's
/// own corpus is keyed by PATH rather than by crate name.
///
/// Resolved via the canonical `workspace_members_kernel::resolve_member_dirs` (NOT the
/// producer's own `collect_bnf_layer_suffix`/`collect_manifest_hygiene` path, and NOT applying
/// `is_path_excluded` — that config-driven exclusion is itself a SECOND silent-drop vector the
/// producer applies; a census that doesn't re-apply it will correctly MISMATCH if an exclusion
/// rule is ever mis-scoped to drop a real crate), with its OWN from-scratch `[package] name`
/// parse (see [`independent_parse_package_name`]). FAILS CLOSED: a resolved member directory
/// whose `Cargo.toml` is unreadable or carries no `[package] name` is a hard test failure
/// (panic), never a silent skip — the exact bug class (scan-root/glob/prefix/parse/truncation
/// regression, silently dropping an eligible crate) a magic-number floor, or a bare non-empty
/// check, could never catch.
pub fn independent_member_manifests(root: &Path) -> BTreeMap<String, String> {
    let mut member_dirs = workspace_members_kernel::resolve_member_dirs(root)
        .expect("resolve_member_dirs must resolve the live workspace Cargo.toml");
    member_dirs.extend(resolve_nested_workspace_member_dirs(root));
    let mut manifests = BTreeMap::new();
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
        manifests.insert(dir, name);
    }
    manifests
}

/// The repo-relative directories of today's workspace member crates, fail-closed as
/// [`independent_member_manifests`].
///
/// The key shape a gate that walks the tree for `Cargo.toml` files can be compared against.
pub fn independent_member_dirs(root: &Path) -> BTreeSet<String> {
    independent_member_manifests(root).into_keys().collect()
}

/// INDEPENDENT dynamic census of today's `oya-*`-prefixed workspace member crate NAMES.
///
/// Self-adjusts through future de-brands: the census shrinks in lockstep with the face as crates
/// lose the `oya-` prefix, so this stays valid without ever needing a bump. The prefix is read
/// from `NamingConfig`, not hardcoded here.
pub fn independent_prefix_census(root: &Path) -> BTreeSet<String> {
    let prefix = NamingConfig::default().required_prefix;
    independent_member_manifests(root)
        .into_values()
        .filter(|name| name.starts_with(&prefix))
        .collect()
}

/// Nested-workspace roots this repo carves out of the root workspace (`[workspace].exclude`
/// entries that are THEMSELVES a `[workspace]` root — e.g. `kernel/`'s ADR-0512 rung-0
/// carve-out): their first-party crates are real, tracked manifests the PRODUCER's
/// tracked-Cargo.toml scan legitimately includes (it doesn't care which cargo workspace a
/// Cargo.toml belongs to), so the census must resolve them too or it silently under-counts
/// relative to the face. Discovered from the root manifest's OWN `exclude` list (not a hardcoded
/// dir list) — self-adjusts if a future carve-out is added or removed; an excluded entry with no
/// `[workspace]` Cargo.toml (e.g. a buck2-only gate with no Cargo.toml at all) is simply skipped,
/// not a nested workspace.
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
        let members =
            workspace_members_kernel::resolve_member_dirs_from_str(&nested_text, &nested_root)
                .expect("resolve nested workspace members");
        dirs.extend(members.into_iter().map(|m| format!("{excluded}/{m}")));
    }
    dirs
}

/// Minimal, from-scratch extraction of the root workspace's `[workspace] exclude = [...]` array
/// — a plain independent text scan, not a TOML-library parse, so a bug in that shared parser
/// cannot hide behind the census reusing it. `#`-comments are stripped PER LINE before any
/// bracket detection, so a comment mentioning a bracket (e.g. "...its own [workspace]...")
/// can never be mistaken for the array's own structural delimiter.
pub fn root_workspace_excludes(manifest_text: &str) -> Vec<String> {
    let mut in_exclude = false;
    let mut body = String::new();
    for raw in manifest_text.lines() {
        let line = raw.split('#').next().unwrap_or("");
        if !in_exclude {
            if line.trim_start().starts_with("exclude") && line.contains('[') {
                in_exclude = true;
                if let Some((_, after)) = line.split_once('[') {
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
        .filter(|(i, _)| i % 2 == 1)
        .map(|(_, seg)| seg.to_owned())
        .collect()
}

/// Minimal, deliberately SEPARATE `[package] name` parse — NOT a call into the producer's own
/// `parse_package_name`. The point of "independent" is that a bug in the shared parser cannot
/// hide behind the census reusing the same buggy code.
pub fn independent_parse_package_name(contents: &str) -> Option<String> {
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

/// Assert the face's enumerated key set EXACTLY equals the independent census, with a diagnostic
/// naming exactly which keys are missing/extra on mismatch.
///
/// Use when the gate's corpus and the census are the same population — the face is wrong if it
/// holds a key the census does not, and wrong if it drops one the census has.
pub fn assert_census_matches(face_names: &BTreeSet<String>, census: &BTreeSet<String>) {
    let missing_from_face: Vec<&String> = census.difference(face_names).collect();
    let extra_in_face: Vec<&String> = face_names.difference(census).collect();
    assert!(
        missing_from_face.is_empty() && extra_in_face.is_empty(),
        "face/census SET MISMATCH — missing_from_face={missing_from_face:?} \
         extra_in_face={extra_in_face:?}"
    );
}

/// Assert every census key is present in an observed corpus that is legitimately a SUPERSET,
/// naming exactly which keys went missing.
///
/// The containment direction is not a weakened equality — it is the true relation when a gate
/// scans a population that strictly contains the workspace members (e.g. every first-party
/// `Cargo.toml` in the tree, which also picks up nested-workspace roots and manifests no member
/// glob matches). Asserting equality there would red on a fact that is correct.
///
/// The unchecked direction — a manifest in the tree that is NOT a workspace member — is a real
/// defect, and it is `ci/facade/workspace-member-coverage`'s subject, not this helper's: a crate
/// outside the member set is not compiled by `cargo nextest run --workspace` and is therefore
/// invisible to every gate at once. Do not re-litigate it here; that gate owns it.
///
/// `observed_label` names the observed set in the failure message so a reader knows which scan
/// collapsed.
pub fn assert_census_covers(
    observed: &BTreeSet<String>,
    census: &BTreeSet<String>,
    observed_label: &str,
) {
    let missing_from_observed: Vec<&String> = census.difference(observed).collect();
    assert!(
        missing_from_observed.is_empty(),
        "face/census SET MISMATCH — {} workspace member(s) the independent census resolved are \
         absent from {observed_label} (observed={} census={}): {missing_from_observed:?}",
        missing_from_observed.len(),
        observed.len(),
        census.len(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn set(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|n| (*n).to_owned()).collect()
    }

    // --- assert_census_matches: RED-test the vacuous cases a bare `rows.len() > 0` check is
    // blind to ("some debt survived" is not the same fact as "the corpus was enumerated"). These
    // three proofs are the reason to trust the helper; they moved with it. ---

    #[test]
    #[should_panic(expected = "SET MISMATCH")]
    fn census_mismatch_is_caught_when_face_is_empty_but_census_is_not() {
        assert_census_matches(&BTreeSet::new(), &set(&["a-domain"]));
    }

    #[test]
    #[should_panic(expected = "SET MISMATCH")]
    fn census_mismatch_is_caught_when_face_is_near_empty() {
        // The exact vacuous-case bug class a bare `rows.len() > 0` check is blind to: one
        // surviving row ("some debt survived") is not the same fact as "the corpus was
        // enumerated."
        assert_census_matches(
            &set(&["a-domain"]),
            &set(&["a-domain", "b-domain", "c-domain"]),
        );
    }

    #[test]
    #[should_panic(expected = "SET MISMATCH")]
    fn census_mismatch_is_caught_when_exactly_one_crate_is_missing() {
        assert_census_matches(
            &set(&["a-domain", "b-domain"]),
            &set(&["a-domain", "b-domain", "c-domain"]),
        );
    }

    #[test]
    #[should_panic(expected = "SET MISMATCH")]
    fn census_mismatch_is_caught_when_the_face_carries_a_key_the_census_does_not() {
        assert_census_matches(
            &set(&["a-domain", "ghost-domain"]),
            &set(&["a-domain"]),
        );
    }

    #[test]
    fn census_match_is_green_when_sets_are_equal() {
        let names = set(&["a-domain", "b-domain"]);
        assert_census_matches(&names, &names.clone());
    }

    // --- assert_census_covers: the containment form, for a corpus that is legitimately a
    // superset. It must still catch a partial collapse, and must NOT red on the extra keys that
    // make the observed set a superset in the first place. ---

    #[test]
    #[should_panic(expected = "SET MISMATCH")]
    fn covers_is_caught_when_observed_is_empty_but_census_is_not() {
        assert_census_covers(&BTreeSet::new(), &set(&["a/Cargo.toml"]), "observed");
    }

    #[test]
    #[should_panic(expected = "SET MISMATCH")]
    fn covers_is_caught_when_exactly_one_member_is_missing() {
        assert_census_covers(&set(&["a", "b"]), &set(&["a", "b", "c"]), "observed");
    }

    #[test]
    #[should_panic(expected = "\"c\"")]
    fn covers_names_the_missing_key_rather_than_reporting_a_number() {
        assert_census_covers(&set(&["a", "b"]), &set(&["a", "b", "c"]), "observed");
    }

    #[test]
    fn covers_is_green_when_observed_is_a_strict_superset() {
        // The whole point of the containment form: extra observed keys are CORRECT here (a
        // nested-workspace root, a manifest no member glob matches), so they must not red.
        assert_census_covers(&set(&["a", "b", "c"]), &set(&["a", "b"]), "observed");
    }

    #[test]
    fn covers_is_green_when_the_sets_are_equal() {
        assert_census_covers(&set(&["a", "b"]), &set(&["a", "b"]), "observed");
    }

    // --- independent_member_manifests: fixture-driven fail-closed proof (unreadable/unparseable
    // eligible manifest) + a happy-path resolve+filter proof. ---

    fn census_tmp_root(tag: &str) -> PathBuf {
        let unique = format!("ci-corpus-census-{tag}-{}", std::process::id());
        let root = std::env::temp_dir().join(unique);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");
        root
    }

    #[test]
    #[should_panic(expected = "unreadable manifest")]
    fn independent_census_fails_closed_on_unreadable_member_manifest() {
        // `resolve_member_dirs` fails when a matched directory has no Cargo.toml. This fixture
        // tests the next boundary: the manifest exists for membership resolution but is
        // unreadable when the census parses it. A deliberately unreadable oya-* manifest must RED
        // the census.
        let root = census_tmp_root("unreadable");
        let manifest_path = root.join("crates/ghost-domain/Cargo.toml");
        std::fs::create_dir_all(manifest_path.parent().unwrap()).expect("mkdir");
        std::fs::write(&manifest_path, "[package]\nname = \"ghost-domain\"\n")
            .expect("write member manifest");
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&manifest_path, std::fs::Permissions::from_mode(0o000))
            .expect("chmod 000");
        std::fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]\n",
        )
        .expect("write root manifest");
        let _ = independent_prefix_census(&root);
    }

    #[test]
    #[should_panic(expected = "no [package] name")]
    fn independent_census_fails_closed_on_unparseable_member_manifest() {
        let root = census_tmp_root("unparseable");
        std::fs::create_dir_all(root.join("crates/broken-domain")).expect("mkdir");
        std::fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]\n",
        )
        .expect("write root manifest");
        std::fs::write(
            root.join("crates/broken-domain/Cargo.toml"),
            "[dependencies]\n",
        )
        .expect("write member manifest");
        let _ = independent_prefix_census(&root);
    }

    #[test]
    fn independent_census_resolves_and_filters_a_small_fixture() {
        let root = census_tmp_root("happy");
        std::fs::create_dir_all(root.join("crates/a-domain")).expect("mkdir");
        std::fs::create_dir_all(root.join("crates/other-b-domain")).expect("mkdir");
        std::fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]\n",
        )
        .expect("write root manifest");
        std::fs::write(
            root.join("crates/a-domain/Cargo.toml"),
            "[package]\nname = \"a-domain\"\n",
        )
        .expect("write member manifest");
        std::fs::write(
            root.join("crates/other-b-domain/Cargo.toml"),
            "[package]\nname = \"other-b-domain\"\n",
        )
        .expect("write member manifest");
        assert_eq!(
            independent_prefix_census(&root),
            set(&["a-domain"]),
            "the prefix filter must drop the non-oya member"
        );
        // The dir-keyed primitive keeps BOTH members: the prefix filter belongs to the name
        // census, not to the enumeration, so a path-keyed gate sees the whole member set.
        assert_eq!(
            independent_member_dirs(&root),
            set(&["crates/a-domain", "crates/other-b-domain"])
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn root_workspace_excludes_ignores_brackets_inside_comments() {
        let manifest = "[workspace]\n\
             # a comment mentioning [workspace] and a ] bracket\n\
             exclude = [\n\
             \x20 \"kernel\", # its own [workspace] root\n\
             \x20 \"ci/facade/harness\",\n\
             ]\n";
        assert_eq!(
            root_workspace_excludes(manifest),
            vec!["kernel".to_owned(), "ci/facade/harness".to_owned()]
        );
    }

    #[test]
    fn independent_parse_package_name_reads_only_the_package_table() {
        assert_eq!(
            independent_parse_package_name("[dependencies]\nname = \"wrong\"\n"),
            None,
            "a `name` key outside [package] must not be mistaken for the package name"
        );
        assert_eq!(
            independent_parse_package_name("[package]\nname = \"a-domain\"\n").as_deref(),
            Some("a-domain")
        );
    }
}
