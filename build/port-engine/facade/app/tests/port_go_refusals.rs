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

/// A failing return DISCARDS the value carried beside the failure, because the pack says so.
///
/// The source returns both; the target returns one or the other. The source documents that a
/// result beside a non-nil error is not guaranteed to be meaningful, so discarding is faithful to
/// the CONVENTION — and the engine used to be faithful only to the cases inspection could confirm,
/// admitting a literal and refusing anything computed, which left most real fallible code
/// unported.
///
/// This asserts the decision as the pack currently declares it. The stricter half — refusing
/// unless the value can be seen to be inert — is what the pack buys back by setting
/// `discards_companion` to false, and is fenced by the transform's own tests, which declare it
/// that way.
#[test]
fn a_failing_return_discards_its_companion() {
    let report = driver::port_go_failure_pipeline()
        .expect("the pack grants the trust, so a carried value no longer refuses");
    let source = driver::assemble_modules(&report);

    assert!(
        source.contains("return Err("),
        "a failing return must carry the failure alone:\n{source}"
    );
    assert!(
        !source.contains("Err((") ,
        "the companion must be gone, not tupled into the failure:\n{source}"
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
