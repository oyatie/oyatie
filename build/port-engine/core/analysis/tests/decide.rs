//! Deciding ownership: order, refusal, and the record that makes a decision auditable.

use port_engine_analysis::{AnalysisError, decide, receiver_form, w0_ready};
use port_engine_api::{OwnershipFacts, PointerConstruction, PointerDisposition};

fn rule(
    id: &str,
    mutated: Option<bool>,
    escapes: Option<bool>,
    effect_unknown: Option<bool>,
    receiver: Option<&str>,
) -> PointerDisposition {
    PointerDisposition {
        id: id.to_owned(),
        when_mutated: mutated,
        when_escapes: escapes,
        when_effect_unknown: effect_unknown,
        // These tests are about which rule matches on the ownership facts; rebinding is a
        // separate axis and is left uncared-about so every existing case keeps its meaning.
        when_rebound: None,
        target: format!("{id}<{{0}}>"),
        receiver: receiver.map(ToOwned::to_owned),
        // A shared borrow is the neutral fixture reference form: these tests are about
        // which rule matches, not about what a map or a slice parameter becomes under it.
        reference_target: Some("&{0}".to_owned()),
        reference_reason: Some("fixture decision".to_owned()),
        // These tests are about which rule MATCHES, not about what an argument becomes under it,
        // so the construction is the neutral borrow.
        construction: PointerConstruction::Borrow {
            mutable: mutated.unwrap_or(false),
            reason: format!("because {id}"),
        },
        reason: format!("because {id}"),
    }
}

fn facts(mutated: bool, escapes: bool, effect_unknown: bool) -> OwnershipFacts {
    OwnershipFacts {
        mutated,
        escapes,
        effect_unknown,
        rebound: false,
    }
}

#[test]
fn claims_readiness() {
    assert!(w0_ready());
}

/// First match wins, so a pack orders from most specific to least. The order is the pack's to
/// choose and the engine's to obey — reordering here would silently change every decision.
#[test]
fn the_first_matching_rule_wins() {
    let rules = vec![
        rule("escaping", None, Some(true), None, None),
        rule("catch_all", None, None, None, Some("&self")),
    ];
    let decision = decide("site", facts(false, true, false), &rules).expect("a rule matches");
    assert_eq!(decision.rule_id, "escaping");
}

/// A `None` in a rule means "do not care", so a rule can be as specific or as loose as the pack
/// needs without a separate wildcard mechanism.
#[test]
fn an_unconstrained_field_matches_either_value() {
    let rules = vec![rule(
        "any_mutation",
        None,
        Some(false),
        Some(false),
        Some("&self"),
    )];
    for mutated in [true, false] {
        assert!(decide("site", facts(mutated, false, false), &rules).is_ok());
    }
}

/// No implicit default. Facts nothing accepts refuse, because an invented default is a decision
/// nobody wrote down — and it would be invisible, since the emitted code looks the same either way.
#[test]
fn facts_no_rule_accepts_refuse() {
    let rules = vec![rule(
        "only_clean",
        Some(false),
        Some(false),
        Some(false),
        Some("&self"),
    )];
    let err = decide("Counter::Add", facts(true, false, false), &rules)
        .expect_err("nothing accepts a mutating pointer");
    match err {
        AnalysisError::NoRule { site, facts } => {
            assert_eq!(site, "Counter::Add");
            assert!(facts.mutated);
        }
        other => panic!("expected NoRule, got {other:?}"),
    }
    // And the message says what to do about it.
    let message = decide("Counter::Add", facts(true, false, false), &rules)
        .expect_err("still refuses")
        .to_string();
    assert!(message.contains("mutated=true"), "{message}");
    assert!(message.contains("the pack needs a rule"), "{message}");
}

/// A disposition with no receiver form declines the RECEIVER position specifically. That is a
/// refusal rather than a fallback: a pointer that outlives the call cannot be handed out as any
/// borrow of `self`, so there is nothing to fall back to.
#[test]
fn a_rule_without_a_receiver_form_declines_that_position() {
    let rules = vec![rule("escaping", None, Some(true), None, None)];
    let decision = decide("Node::Itself", facts(false, true, false), &rules).expect("matches");

    assert_eq!(
        decision.target, "escaping<{0}>",
        "the parameter form is still available"
    );

    let err = receiver_form("Node::Itself", &decision).expect_err("no receiver form");
    let message = err.to_string();
    assert!(message.contains("Node::Itself"), "{message}");
    assert!(
        message.contains("because escaping"),
        "the reason travels: {message}"
    );
}

/// A decision made on unproven facts is MARKED, not hidden. The difference between "safe as far as
/// anyone looked" and "safe as far as anyone looked, and nobody looked past the first call" is
/// invisible in the emitted code, so it has to be visible in the record.
#[test]
fn a_decision_on_unproven_facts_says_so() {
    let rules = vec![rule("loose", None, None, None, Some("&self"))];

    let proven = decide("a", facts(false, false, false), &rules).expect("matches");
    assert!(!proven.rests_on_unproven_facts());

    let unproven = decide("b", facts(false, false, true), &rules).expect("matches");
    assert!(unproven.rests_on_unproven_facts());
}

/// The facts travel WITH the decision. A record that omits them asks a reviewer to trust the
/// inference rather than audit it.
#[test]
fn the_decision_carries_the_facts_it_was_made_on() {
    let rules = vec![rule("loose", None, None, None, Some("&mut self"))];
    let decision = decide("site", facts(true, false, true), &rules).expect("matches");
    assert_eq!(decision.facts, facts(true, false, true));
    assert_eq!(decision.reason, "because loose");
}
