//! The refusal proofs: real Go the engine declines to translate, and what it says about it.
//!
//! Each class has its OWN corpus, and that is the finding rather than the arrangement. One corpus
//! carrying several classes proves whichever package the transform reaches first and leaves the
//! rest untested while looking tested — which is exactly what happened when the statement and
//! ownership refusals shared a fixture.
//!
//! What each assertion checks is that the message NAMES the thing: the construct, the site, the
//! position, and where the missing analysis belongs. A refusal that only says no is a refusal
//! nobody can act on.

use port_engine_app::driver;

/// A failing return that carries a COMPUTED value beside the failure.
///
/// The source returns both; the target returns one or the other. Discarding the companion is sound
/// exactly when it is the zero value — the convention says a caller may not read it after a failure
/// — and is a silent loss of work when it is not. So the engine admits literals and the absent
/// value, and refuses anything computed rather than deciding that some expression is "obviously"
/// zero.
#[test]
fn a_failing_return_that_carries_a_value_is_refused_with_its_reason() {
    let err = driver::port_go_refused_failure()
        .expect_err("a computed value beside a failure has no target shape");

    let message = err.to_string();
    assert!(
        message.contains("Sized"),
        "the refusal must name the declaration: {message}"
    );
    assert!(
        message.contains("carries only the failure"),
        "the refusal must say what the target's shape IS: {message}"
    );
    assert!(
        message.contains("lose work"),
        "the refusal must say what would be lost: {message}"
    );
}

/// and none for a result. Emitting a box there would be choosing an owner on the source's behalf.
#[test]
fn an_interface_in_an_undeclared_position_is_refused_with_its_reason() {
    let err = driver::port_go_refused_interface()
        .expect_err("an interface result has no declared target form");

    let message = err.to_string();
    assert!(
        message.contains("Speaker"),
        "the refusal must name the trait: {message}"
    );
    assert!(
        message.contains("result"),
        "the refusal must name the position: {message}"
    );
    assert!(
        message.contains("owns the value"),
        "the refusal must say what the missing decision IS: {message}"
    );
}

/// refuse anything a front end would actually produce.
#[test]
fn the_refusal_corpus_is_refused_by_name() {
    let err = driver::port_go_refused().expect_err("the refusal corpus must not translate");

    let message = err.to_string();
    // The corpus holds several untranslatable things and the pipeline reports the first it
    // reaches, so the fence asserts the PROPERTY every refusal must have rather than pinning one
    // message. Which one comes first is a function of declaration order, and a fence that depends
    // on that breaks every time a corpus package lands.
    let named = ["ForStmt", "DeferStmt", "&^=", "the pack defers", "is variadic"];
    assert!(
        named.iter().any(|subject| message.contains(subject)),
        "the refusal must name the construct it refused, got: {message}"
    );
    // A refusal of a source CONSTRUCT points at the census that will size it. An operator with no
    // target form has no census to point at — the reason is a property of the two languages and
    // is stated in full where the operator is mapped — so requiring the word everywhere would buy
    // a citation that does not exist.
    if message.contains("source construct") {
        assert!(
            message.contains("census"),
            "a construct refusal must point at where the analysis lives, got: {message}"
        );
    }
}

/// hold.
#[test]
fn an_escaping_receiver_is_refused_with_its_reason() {
    let err =
        driver::port_go_refused_ownership().expect_err("an escaping receiver has no borrow form");

    let message = err.to_string();
    assert!(
        message.contains("Itself"),
        "the refusal must name the site: {message}"
    );
    assert!(
        message.contains("escaping_owned"),
        "the refusal must name the disposition that declined: {message}"
    );
    assert!(
        message.contains("outlives the call"),
        "the refusal must carry the pack's recorded reason: {message}"
    );
}
