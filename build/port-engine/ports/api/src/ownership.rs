//! Ownership: the facts a front end observes, and the dispositions a pack declares.
//!
//! Go is garbage-collected, so a pointer says nothing about ownership. The same `*T` may be a
//! borrow that does not outlive the call, an owned value passed by pointer for efficiency, or a
//! shared structure with live aliases — and Rust needs that decided. Nobody can decide it from the
//! type, so it is decided from FACTS, by RULES, and the pairing is recorded.

/// What a front end observed about one pointer receiver or parameter.
///
/// Intra-procedural, and [`OwnershipFacts::effect_unknown`] is what keeps that honest.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct OwnershipFacts {
    /// The body provably assigns through it.
    pub mutated: bool,
    /// It provably outlives the call — returned, or captured by a closure.
    pub escapes: bool,
    /// It was passed to a call the front end did not analyse, so `mutated` and `escapes` being
    /// false means UNPROVEN rather than false.
    ///
    /// The distinction is the whole reason this field exists. Without it a function that hands its
    /// pointer to an unanalysed callee is indistinguishable from one that does nothing with it,
    /// and the second answer is safe while the first is a guess.
    pub effect_unknown: bool,
}

impl OwnershipFacts {
    /// `true` when every fact is proven absent — nothing mutates, nothing escapes, nothing is
    /// unaccounted for.
    #[must_use]
    pub const fn is_clean(self) -> bool {
        !self.mutated && !self.escapes && !self.effect_unknown
    }
}

/// A rule mapping observed facts onto a target ownership form.
///
/// Rules are DATA and are evaluated in declared order, first match winning, because which
/// ownership form a set of facts deserves is a translation decision rather than a fact — and one
/// with a cost either way, which is what [`PointerDisposition::reason`] has to name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PointerDisposition {
    /// Stable identity, so a decision can be cited.
    pub id: String, // data_class: INTERNAL_ONLY
    /// Required fact values. `None` means "do not care".
    pub when_mutated: Option<bool>,
    /// Required `escapes` value, or `None` for "do not care".
    pub when_escapes: Option<bool>,
    /// Required `effect_unknown` value, or `None` for "do not care".
    pub when_effect_unknown: Option<bool>,
    /// Target type template for a PARAMETER position, with `{0}` for the pointee.
    pub target: String, // data_class: INTERNAL_ONLY
    /// Target form for a RECEIVER position: `&self`, `&mut self`, `self`.
    ///
    /// `None` means this disposition has no receiver form, which is a refusal rather than a
    /// fallback — a pointer that escapes cannot be handed out as any borrow of `self`.
    pub receiver: Option<String>, // data_class: INTERNAL_ONLY
    /// Why these facts deserve this form, and what it costs.
    pub reason: String, // data_class: INTERNAL_ONLY
}

impl PointerDisposition {
    /// Whether this rule accepts `facts`.
    #[must_use]
    pub fn accepts(&self, facts: OwnershipFacts) -> bool {
        matches(self.when_mutated, facts.mutated)
            && matches(self.when_escapes, facts.escapes)
            && matches(self.when_effect_unknown, facts.effect_unknown)
    }
}

const fn matches(required: Option<bool>, observed: bool) -> bool {
    match required {
        None => true,
        Some(value) => value == observed,
    }
}
