//! Which source file is a face leaf's root. Provenance: ADR-0719 D-30/D-41.

use super::expected_manifest_identity;
use crate::layout::path_parts;

/// The entry points a face leaf may present, most-preferred first.
///
/// A `core`, `ports` or `adapters` leaf is a library and roots at `src/lib.rs`.
///
/// A `facade` leaf is a service *surface*, which is not the same as a running
/// service. 31 of 54 have no `src/main.rs`, and every one of those has
/// `src/lib.rs`. The `iam/facade/tenant-rbac-*` family records why in its own
/// source — `deployed_listener_attached: false`, "does not start a listener":
/// the surface lands with its routes and handlers, and a later composition
/// attaches the listener.
///
/// Not all 31 are that deliberate. Some are plainly libraries filed under the
/// wrong face — `compute/facade/k8s` is package `compute-k8s-api`;
/// `iam/facade/identity-workload-app` calls itself a usecase ring — and those
/// belong in `core/`, which this rule does not decide. What the rule decides is
/// narrower: whether a crate can be EDITED while its face is unresolved.
///
/// Requiring `src/main.rs` unconditionally said no. Every one of the 31 was
/// untouchable, because any edit to such a manifest demanded inventing a binary
/// — one the tenant-rbac family explicitly defers, and one a misfiled library
/// should never have. Since a moved crate must repoint its consumers, a single
/// staged facade anywhere in a dependency chain blocked the extraction. A
/// facade therefore roots at `src/main.rs` once it runs, and at `src/lib.rs`
/// until then; the deletion of an existing `src/main.rs` is still refused, so
/// the door only opens one way.
///
/// What the check is for is unchanged: a touched leaf must present a reachable
/// root, and that root must be a regular blob.
pub fn cargo_entrypoints(path: &str) -> Vec<String> {
    let Some((_, face, _)) = expected_manifest_identity(path) else {
        return Vec::new();
    };
    let Some(directory) = path.strip_suffix("/Cargo.toml") else {
        return Vec::new();
    };
    if face == "facade" {
        vec![
            format!("{directory}/src/main.rs"),
            format!("{directory}/src/lib.rs"),
        ]
    } else {
        vec![format!("{directory}/src/lib.rs")]
    }
}

/// The preferred entry point, or `None` when `path` is not a face-leaf manifest.
pub fn cargo_entrypoint(path: &str) -> Option<String> {
    cargo_entrypoints(path).into_iter().next()
}

pub fn cargo_manifest_for_entrypoint(path: &str) -> Option<String> {
    let directory = path
        .strip_suffix("/src/lib.rs")
        .or_else(|| path.strip_suffix("/src/main.rs"))?;
    let manifest = format!("{directory}/Cargo.toml");
    cargo_entrypoints(&manifest)
        .iter()
        .any(|candidate| candidate == path)
        .then_some(manifest)
}

/// Map any path below a canonical face leaf back to that crate's manifest.
pub fn cargo_manifest_for_crate_path(path: &str) -> Option<String> {
    let parts = path_parts(path);
    (1..=parts.len()).find_map(|end| {
        let manifest = format!("{}/Cargo.toml", parts[..end].join("/"));
        if expected_manifest_identity(&manifest).is_some() {
            Some(manifest)
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_library_face_roots_only_at_a_library() {
        assert_eq!(
            cargo_entrypoints("network/core/route/Cargo.toml"),
            vec!["network/core/route/src/lib.rs".to_owned()]
        );
        // A core leaf that shipped a binary instead of a library is still wrong.
        assert!(cargo_manifest_for_entrypoint("network/core/route/src/main.rs").is_none());
    }

    #[test]
    fn a_facade_may_root_at_a_library_until_its_listener_is_attached() {
        assert_eq!(
            cargo_entrypoints("network/facade/edge-app/Cargo.toml"),
            vec![
                "network/facade/edge-app/src/main.rs".to_owned(),
                "network/facade/edge-app/src/lib.rs".to_owned(),
            ]
        );
        for root in [
            "network/facade/edge-app/src/main.rs",
            "network/facade/edge-app/src/lib.rs",
        ] {
            assert_eq!(
                cargo_manifest_for_entrypoint(root).as_deref(),
                Some("network/facade/edge-app/Cargo.toml"),
                "{root} must map back to its manifest"
            );
        }
    }

    #[test]
    fn a_path_that_is_not_a_face_leaf_has_no_entry_point() {
        assert!(cargo_entrypoints("docs/Cargo.toml").is_empty());
        assert!(cargo_entrypoint("docs/Cargo.toml").is_none());
    }
}
