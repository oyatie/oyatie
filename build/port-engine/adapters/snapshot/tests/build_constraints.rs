//! A Go package is the file set a build CONFIGURATION selects — not every `.go` file in a
//! directory.
//!
//! The front end used to glob the directory, which is not a bigger package but a file set no
//! `go build` ever emits. That failed two ways, and only one of them was loud.
//!
//! LOUD: two files declaring one symbol under mutually exclusive constraints are a redeclaration,
//! so the type check fails and the package yields no measurement at all. Three of eight surveyed
//! third-party corpora fail exactly there.
//!
//! QUIET, and the reason this fence exists: a constrained file that collides with nothing is
//! admitted, and its declarations enter the snapshot as unconditional members of the package.
//! `pkg/errors` ships `Is`, `As` and `Unwrap` behind `//go:build go1.13` and they arrived that
//! way — correct for a recent toolchain, wrong for a configuration that excludes them, and
//! recorded nowhere as conditional at all.
//!
//! The fixture corpus is built to fail this test loudly if selection is ever removed: `tagged`
//! declares `Platform` in both `platform_linux.go` and `platform_darwin.go`, so a globbing front
//! end cannot even regenerate the artifact these assertions read.

use std::collections::BTreeMap;

use port_engine_api::SourceModel;
use port_engine_snapshot::admit_embedded_fixture_buildtags_v1;

/// Declaration names of the one package in the build-constraint fixture.
fn declaration_names() -> Vec<String> {
    let admitted = admit_embedded_fixture_buildtags_v1().expect("build-tag fixture admits");
    let units = admitted.units();
    assert_eq!(
        units.len(),
        1,
        "the fixture corpus holds exactly one package"
    );
    admitted
        .declarations(&units[0])
        .expect("the admitted unit has declarations")
        .into_iter()
        .map(|declaration| declaration.name)
        .collect()
}

/// The filename constraint decides, and it decides ONCE.
///
/// `platform_linux.go` and `platform_darwin.go` both declare `Platform`. Under the declared
/// configuration exactly one file is in, so exactly one declaration exists. Two would mean the
/// front end merged two build configurations into a package that cannot be built.
#[test]
fn a_filename_constrained_symbol_is_declared_exactly_once() {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for name in declaration_names() {
        *counts.entry(name).or_default() += 1;
    }

    assert_eq!(
        counts.get("Platform").copied(),
        Some(1),
        "`Platform` is declared in two mutually exclusive files; the configuration selects one"
    );
}

/// A release-gated declaration the configuration INCLUDES is present.
///
/// The complement of the test below: a front end that answered every constraint with "no" would
/// pass the exclusion fence while silently emptying the package.
#[test]
fn a_satisfied_release_constraint_admits_its_declarations() {
    assert!(
        declaration_names().iter().any(|name| name == "Recent"),
        "`Recent` sits behind `//go:build go1.13`, which the declared release satisfies"
    );
}

/// A release-gated declaration the configuration EXCLUDES is absent.
///
/// `Unreleased` sits behind `//go:build go1.99`. No declared release selects it, so it must not be
/// in the model. Left to the host toolchain's own release tags, whether this declaration exists
/// would become a property of the machine that ran the extractor — and the snapshot digest is the
/// engine's identity for the source, so a host-dependent digest makes every re-extraction look
/// like upstream drift.
#[test]
fn an_unsatisfied_release_constraint_excludes_its_declarations() {
    assert!(
        !declaration_names().iter().any(|name| name == "Unreleased"),
        "`Unreleased` sits behind `//go:build go1.99` and no declared release selects it"
    );
}
