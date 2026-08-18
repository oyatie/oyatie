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
    assert!(
        message.contains("ForStmt") || message.contains("DeferStmt"),
        "the refusal must name the construct it refused, got: {message}"
    );
    assert!(
        message.contains("census"),
        "the refusal must point at where the analysis lives, got: {message}"
    );
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
