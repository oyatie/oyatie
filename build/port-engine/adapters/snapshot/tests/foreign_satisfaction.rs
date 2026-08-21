//! A foreign satisfaction is an OBSERVATION about a type, not a declaration of one.
//!
//! Go's package scope is a namespace: one name means one thing. That rule is correct and stays
//! exactly as strict for everything that enters the scope. A foreign satisfaction does not enter
//! it — its name is a type ANOTHER package declares, and the entry records something true about
//! that type rather than introducing it here.
//!
//! Reading those entries as declarations rejected the whole snapshot the moment one foreign type
//! satisfied two interfaces, which is ordinary Go: `os.File` is a reader and a writer, and so is
//! `bytes.Buffer`. Two of eight surveyed third-party corpora died on exactly that, and the failure
//! was total — no measurement at all from a package whose only sin was passing a buffer to two
//! functions.
//!
//! The second half is why the entries are worth keeping. The interface being satisfied used to
//! live only inside an English sentence in `go_node`, so the two facts differed nowhere a rule
//! could read them: anything keyed on "what satisfies `io.Reader`" would have had to parse prose.
//! It is a structured attribute now, and these tests fence both halves.

use port_engine_api::{Declaration, SourceModel};
use port_engine_frontend_go::ATTR_INTERFACE;
use port_engine_snapshot::admit_embedded_fixture_foreign_v1;

/// The foreign satisfactions recorded by the fixture's single package.
fn foreign_satisfactions() -> Vec<Declaration> {
    let admitted = admit_embedded_fixture_foreign_v1().expect("foreign fixture admits");
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
        .filter(|declaration| declaration.kind == "foreign_satisfaction")
        .collect()
}

/// One foreign type satisfying two interfaces admits as two facts.
///
/// Admission is the whole assertion: before the namespace rule was qualified, this fixture could
/// not be decoded at all and the error named a duplicate declaration of `bytes.Buffer`.
#[test]
fn one_foreign_type_may_satisfy_two_interfaces() {
    let facts = foreign_satisfactions();

    assert_eq!(
        facts.len(),
        2,
        "`bytes.Buffer` satisfies both `io.Reader` and `io.Writer` in this corpus"
    );
    assert!(
        facts.iter().all(|fact| fact.name == "bytes.Buffer"),
        "both facts are about the same foreign type, which is why they collided"
    );
}

/// The interface is a FIELD, so the two facts are distinguishable without reading prose.
#[test]
fn each_fact_names_the_interface_it_satisfies() {
    let mut interfaces: Vec<String> = foreign_satisfactions()
        .into_iter()
        .map(|fact| {
            fact.attrs
                .get(ATTR_INTERFACE)
                .cloned()
                .unwrap_or_else(|| panic!("a foreign satisfaction must name its interface"))
        })
        .collect();
    interfaces.sort();

    assert_eq!(
        interfaces,
        vec!["io.Reader".to_owned(), "io.Writer".to_owned()],
        "the interface identity must be structured, not folded into the refusal sentence"
    );
}

/// A foreign satisfaction never displaces a real declaration.
///
/// The qualification is narrow on purpose: it says these entries do not ENTER the namespace, not
/// that the namespace is looser. `Sink`, `Source` and `Drive` are still bound exactly once each.
#[test]
fn real_declarations_still_bind_exactly_once() {
    let admitted = admit_embedded_fixture_foreign_v1().expect("foreign fixture admits");
    let units = admitted.units();
    let declarations = admitted.declarations(&units[0]).expect("declarations");

    for name in ["Sink", "Source", "Drive"] {
        let bound = declarations
            .iter()
            .filter(|declaration| declaration.name == name)
            .count();
        assert_eq!(bound, 1, "`{name}` binds exactly one package-scope name");
    }
}
