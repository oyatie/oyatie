//! What is OBSERVED about a package-scope variable.
//!
//! `var` is deferred by the pack because Rust's `static` is immutable, `static mut` is unsafe, and
//! `OnceLock`/`Mutex` each pick a synchronization policy the source never stated. That argument is
//! sound and it only bites for a variable the program actually ASSIGNS TO. Across the seven
//! surveyed third-party corpora, 45 of 67 package variables are never written anywhere in their
//! own package — so the hardest case's reason was being applied to two thirds of the variables
//! that do not have the problem.
//!
//! Two facts had to exist before that could even be said, and neither did. A `const` recorded its
//! value and a `var` recorded nothing, so every package variable reached the engine as a name with
//! no content; and nothing computed whether anything wrote it. These tests fence both.
//!
//! Neither fact decides the emitted form. That decision stays with the pack, which is the point:
//! it can now be made on evidence instead of on the worst case.

use port_engine_api::{Declaration, SourceModel};
use port_engine_snapshot::{admit_embedded_fixture_refused_v1, admit_embedded_fixture_v1};

const FLAG_REBOUND: &str = "rebound";

/// The package variables of the fixture's `globals` package, by name.
fn package_variables() -> Vec<Declaration> {
    let admitted = admit_embedded_fixture_v1().expect("v1 fixture admits");
    let unit = admitted
        .units()
        .into_iter()
        .find(|unit| unit.0.ends_with("/globals"))
        .expect("the fixture corpus holds a `globals` package");
    admitted
        .declarations(&unit)
        .expect("the admitted unit has declarations")
        .into_iter()
        .filter(|declaration| declaration.kind == "var")
        .collect()
}

fn named(name: &str) -> Declaration {
    package_variables()
        .into_iter()
        .find(|declaration| declaration.name == name)
        .unwrap_or_else(|| panic!("`{name}` must be a package variable of the fixture"))
}

/// An initialised variable carries WHAT it is initialised to.
///
/// As a child expression rather than as source text, unlike a constant's value: a constant's is a
/// literal the target can re-parse, and a variable's is arbitrary code — `errors.New("..")`, a
/// call into another package — that a rule has to be able to inspect and a resolver to qualify.
#[test]
fn an_initialised_variable_carries_its_initialiser() {
    assert_eq!(
        named("prefix").children.len(),
        1,
        "`Prefix = \"id-\"` must record what it is initialised to"
    );
}

/// A variable with no initialiser carries none.
///
/// ABSENT means the source wrote none and the zero value applies. That has to stay distinguishable
/// from an initialiser the front end could not attribute, which is recorded as an `unsupported`
/// child instead of as silence — `var a, b = f()` gives two names one value, and dropping it would
/// make the pair look exactly like `var a, b T`.
#[test]
fn an_uninitialised_variable_carries_no_initialiser() {
    assert!(
        named("limit").children.is_empty(),
        "`var Limit int64` has no initialiser, and absence is what says so"
    );
}

/// A variable nothing writes is not marked as written.
///
/// The direction that matters: this is the fact that lets a package variable stop being deferred on
/// an argument about synchronizing writes that never happen.
#[test]
fn an_unwritten_variable_is_not_marked_written() {
    for name in ["prefix", "limit"] {
        assert!(
            !named(name).flags.contains(FLAG_REBOUND),
            "nothing in the package assigns to `{name}`"
        );
    }
}

/// A variable some function writes IS marked, whether or not it was initialised.
///
/// Read from the REFUSAL corpus, and not by preference: a function that touches a deferred
/// variable cannot be emitted — what the engine emits has to be self-contained — so the writer and
/// the variables it writes are only expressible where refusals are proven.
#[test]
fn a_written_variable_is_marked_written() {
    let admitted = admit_embedded_fixture_refused_v1().expect("refusal fixture admits");
    let unit = admitted
        .units()
        .into_iter()
        .find(|unit| unit.0.ends_with("/hard"))
        .expect("the refusal corpus holds a `hard` package");
    let declarations = admitted.declarations(&unit).expect("declarations");

    for name in ["counter", "pooled"] {
        let variable = declarations
            .iter()
            .find(|declaration| declaration.name == name)
            .unwrap_or_else(|| panic!("`{name}` must be a package variable of the refusal corpus"));
        assert!(
            variable.flags.contains(FLAG_REBOUND),
            "`Bump` assigns to `{name}`, so the synchronization question is real for it"
        );
    }
}
