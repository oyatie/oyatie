//! Typed refusals from rulepack decode and validation.
//!
//! Several of these exist because a pack must not be able to DECLARE semantics the engine
//! silently drops. A field that parses and does nothing is worse than a field that cannot be
//! written: it reads as a promise nothing keeps.

use std::fmt;

use crate::CONFLICT_REFUSE;

/// Typed refusal from rulepack decode / validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RulepackError {
    /// JSON could not be parsed.
    Parse {
        /// Parser detail.
        detail: String,
    },
    /// Required field missing / empty / invalid.
    Schema {
        /// Which field failed.
        field: &'static str,
    },
    /// A declared rule has zero selecting fixtures (W0-B hard stop).
    MissingSelectingFixture {
        /// Rule id that failed the fixture gate.
        rule: String,
    },
    /// A rule declares fixtures but none is a positive selection example.
    NoPositiveFixture {
        /// Rule id that failed the positive-fixture gate.
        rule: String,
        /// Number of declared negative fixtures.
        fixture_count: usize,
    },
    /// A fixture's expected selection disagrees with the loaded `applies` policy.
    FixtureExpectationMismatch {
        /// Rule whose fixture failed.
        rule: String,
        /// Stable fixture identity.
        fixture: String,
        /// Unit exercised by the fixture.
        unit: String,
        /// Selection recorded by the fixture.
        expected: bool,
        /// Selection derived from `applies`.
        actual: bool,
    },
    /// `applies` referenced a rule absent from `rules`.
    UndeclaredApply {
        /// Unit that referenced the rule.
        unit: String,
        /// Undeclared rule id.
        rule: String,
    },
    /// The pack declares semantics the engine does not implement.
    UnimplementedSemantics {
        /// Rule that declares it.
        rule: String,
        /// Which declared field has no implementation behind it.
        field: &'static str,
    },
    /// Declared precedence disagrees with declaration order.
    PrecedenceDisagreesWithOrder {
        /// Rule whose precedence is out of order.
        rule: String,
        /// Precedence it declares.
        precedence: i64,
        /// Precedence of the rule declared before it.
        previous: i64,
    },
    /// A conflict policy the engine has no implementation for.
    UnknownConflictPolicy {
        /// Rule that declares it.
        rule: String,
        /// The policy string found.
        policy: String,
    },
    /// A deferred kind is also captured by a rule — the pack says both things at once.
    DeferredKindAlsoCaptured {
        /// The contradictory kind.
        kind: String,
        /// A rule that captures it.
        rule: String,
    },
    /// Language pair cannot address a rule namespace.
    Pair(port_engine_api::PortError),
}

impl fmt::Display for RulepackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse { detail } => write!(f, "rulepack JSON parse failed: {detail}"),
            Self::Schema { field } => write!(f, "rulepack schema missing or invalid: {field}"),
            Self::MissingSelectingFixture { rule } => write!(
                f,
                "rulepack rule `{rule}` cannot load without ≥1 selecting fixture"
            ),
            Self::NoPositiveFixture {
                rule,
                fixture_count,
            } => write!(
                f,
                "rulepack rule `{rule}` declares {fixture_count} fixture(s) but none selects it"
            ),
            Self::FixtureExpectationMismatch {
                rule,
                fixture,
                unit,
                expected,
                actual,
            } => write!(
                f,
                "rulepack fixture `{fixture}` for rule `{rule}` and unit `{unit}` expected \
                 selects={expected}, derived selects={actual}"
            ),
            Self::UndeclaredApply { unit, rule } => write!(
                f,
                "rulepack applies rule `{rule}` to unit `{unit}` but rules[] does not declare it"
            ),
            Self::UnimplementedSemantics { rule, field } => write!(
                f,
                "rulepack rule `{rule}` declares `{field}`, which the engine does not implement —                  a pack may not declare semantics that are silently dropped"
            ),
            Self::PrecedenceDisagreesWithOrder {
                rule,
                precedence,
                previous,
            } => write!(
                f,
                "rulepack rule `{rule}` declares precedence {precedence} after {previous}:                  declaration order is the transform order, so a precedence that disagrees with it                  is a second ordering nothing obeys"
            ),
            Self::UnknownConflictPolicy { rule, policy } => write!(
                f,
                "rulepack rule `{rule}` declares conflict policy `{policy}`; only                  `{CONFLICT_REFUSE}` is implemented"
            ),
            Self::DeferredKindAlsoCaptured { kind, rule } => write!(
                f,
                "rulepack defers kind `{kind}` and also captures it in rule `{rule}`: the pack                  cannot both translate it and record it as untranslated"
            ),
            Self::Pair(err) => write!(f, "rulepack language pair refused: {err}"),
        }
    }
}

impl std::error::Error for RulepackError {}
