//! Interfaces, end to end: impls from observed satisfaction, and what embedding becomes.
//!
//! Over the emitted crate rather than over hand-built nodes, because both properties are about
//! whole declarations relating to each other — which interfaces a type was SEEN satisfying, and
//! what a type gains from what it embeds.

use port_engine_app::driver;

/// inferred one is not, and the two produce identical Rust.
#[test]
fn trait_impls_are_emitted_from_observed_satisfaction() {
    let report = driver::port_go_pipeline().expect("the Go corpus must port");
    let source = driver::assemble_modules(&report);

    // NO PROVENANCE in the doc comment. A reviewer reading the emitted crate found "Ported from an
    // implicit interface: the source was observed satisfying `X` at <site>." in the public rustdoc
    // and named it as a translator's working note shipped as API documentation. Which satisfactions
    // were observed and where is what the plan and the receipt record; the emitted crate is the
    // product, not the record of how it was made.
    assert!(
        !source.contains("observed satisfying"),
        "provenance must not ship in the emitted crate's documentation:\n{source}"
    );

    for expected in [
        "impl crate::shapes::Named for Label",
        "impl crate::shapes::Named for Tag",
        // The BODY is here, not in an inherent twin. A type carrying both an inherent `name` and
        // a trait `name` compiles only because inherent wins path resolution, and deleting the
        // inherent one turns a forwarding trait impl into infinite recursion.
        "fn name(&self) -> String {\n            self.text.clone()",
    ] {
        assert!(
            source.contains(expected),
            "emitted source must carry `{expected}`:\n{source}"
        );
    }
}

/// mutates.
#[test]
fn a_trait_method_binds_the_receiver_its_implementors_need() {
    let report = driver::port_go_pipeline().expect("the Go corpus must port");
    let source = driver::assemble_modules(&report);

    assert!(
        source.contains("fn name(&self) -> String;"),
        "a read-only trait method must bind shared:\n{source}"
    );
    assert!(
        source.contains("fn refresh(&mut self);"),
        "a mutating trait method must bind exclusive:\n{source}"
    );
}

/// naming a method nothing implements — and nothing short of compiling it would notice.
#[test]
fn embedding_becomes_supertraits_and_forwarding_methods() {
    let report = driver::port_go_pipeline().expect("the Go corpus must port");
    let source = driver::assemble_modules(&report);

    for expected in [
        // An embedded interface is a REQUIREMENT, not a copy of its method set.
        "pub trait Job: Runner + Describer {}",
        // A promoted method is forwarded through the field it was promoted from.
        "self.engine.run()",
        // A pure BUNDLE is implemented once, for everything that qualifies. The source satisfies
        // such an interface STRUCTURALLY — a type with both method sets has it, with nothing to
        // declare — and a blanket impl is what says that in a nominal system. One empty impl per
        // observed type would be both more code and strictly weaker: a type the engine never saw
        // asserted would not have the trait the source says it has.
        "impl<T: Runner + Describer> Job for T {}",
        "impl Runner for Driver",
    ] {
        assert!(
            source.contains(expected),
            "emitted source must carry `{expected}`:\n{source}"
        );
    }

    // And no per-type impl beside it, which the target rejects outright as a coherence conflict.
    assert!(
        !source.contains("impl Job for Driver"),
        "a bundle's blanket impl and a per-type impl cannot both be emitted:\n{source}"
    );
}

/// method's own ownership facts precisely so this is a decision rather than a default.
#[test]
fn a_forwarding_method_inherits_its_receiver_from_what_it_forwards_to() {
    let report = driver::port_go_pipeline().expect("the Go corpus must port");
    let source = driver::assemble_modules(&report);

    assert!(
        source.contains("fn run(&mut self) -> i64 {\n            self.engine.run()"),
        "a forwarding method for a mutating method must bind exclusively:\n{source}"
    );
}
