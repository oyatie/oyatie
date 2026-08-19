//! The failure convention, end to end over real source.
//!
//! Not one construct but a CONVENTION, which is why it blocked every real package: the source
//! returns failure as an extra result that nothing requires a caller to check, and the target says
//! it in the return type, where the compiler does. These proofs are over the emitted crate rather
//! than over hand-built nodes, because a convention only exists across whole declarations.

use port_engine_app::driver;

/// equivalent to it.
#[test]
fn a_fallible_signature_becomes_a_result() {
    let report = driver::port_go_pipeline().expect("the Go corpus must port");
    let source = driver::assemble_modules(&report);

    for expected in [
        // A value and a failure.
        // The unit NAMES the failure type once and its signatures use that name. `Send + Sync` is
        // still part of the type — without them a ported error cannot cross a thread boundary,
        // which the source's error had no trouble doing — and the alias is where it is now said.
        "pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;",
        "pub fn length(s: &str) -> Result<i64>",
        // A failure alone still has to say it succeeded.
        "pub fn check(s: &str) -> Result<()>",
        // The trailing operand decides the constructor, and a failing return drops its zero-value
        // companion because the target's failing return carries only the failure. Built by the
        // INFERRING form, because inside `Err(..)` of a function whose return type names the
        // failure the type is already known and spelling it again says nothing new.
        "return Err(\"empty\".into());",
        "Ok(s.len() as i64)",
    ] {
        assert!(
            source.contains(expected),
            "emitted source must carry `{expected}`:\n{source}"
        );
    }
}

/// system.
#[test]
fn the_propagation_idiom_becomes_the_try_operator() {
    let report = driver::port_go_pipeline().expect("the Go corpus must port");
    let source = driver::assemble_modules(&report);

    assert!(
        source.contains("let n = length(s)?;"),
        "a bind-and-check pair must collapse into one operator:\n{source}"
    );
    // The failure-only shape binds nothing, so the call stays a statement.
    assert!(
        source.contains("check(s)?;"),
        "a check with no value bound must stay a statement:\n{source}"
    );
    // Over CODE lines only. The corpus's own prose describes the idiom it exercises, and a check
    // that read documentation as output would be asserting about the comment rather than the port.
    let code: String = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("///"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !code.contains("!= nil"),
        "no comparison against the source's absent value may survive:\n{code}"
    );
}

/// library is exactly the part that does not come along.
#[test]
fn a_mapped_call_is_answered_by_the_pack() {
    let report = driver::port_go_pipeline().expect("the Go corpus must port");
    let source = driver::assemble_modules(&report);

    assert!(
        source.contains("s.len() as i64"),
        "a builtin must be answered by the pack rather than emitted by name:\n{source}"
    );
    // The pack answers the standard-library call, and where the destination already fixes the
    // type it answers with the shorter of its two forms. Both are the pack's; neither is a spelling
    // this code chose.
    assert!(
        source.contains("Err(\"empty\".into())"),
        "a standard-library call must be answered by the pack:\n{source}"
    );
}
