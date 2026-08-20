//! Methods a type keeps DESPITE, recorded so nothing is lost quietly.
//!
//! A type and its methods are separable in both languages: the source declares a method outside the
//! struct it is on, and the target puts it in an `impl` block of its own. So a method the engine
//! cannot translate does not oblige it to refuse the type — the type's SHAPE is what every other
//! declaration needs in order to mention it, and no method body contributes to that shape.
//!
//! Refusing the type instead is what the engine used to do, by way of a single `?`, and it is the
//! largest structural cause of low coverage in the corpus: one untranslatable method refuses the
//! whole type, the type is then not emitted, and every declaration mentioning it refuses in turn.
//! Six central types accounted for thirty-five refusals that way, each one a package's principal
//! export. A package's coverage was capped by its single hardest method.
//!
//! Dropping the method instead is safe by machinery that already exists: what is emitted has to be
//! self-contained, so a CALL to a method the crate does not contain refuses by name on its own.
//!
//! What must not happen is the drop going unmentioned. Trading a loud cascade for a silent hole is
//! the worse failure of the two and the one this engine exists to prevent — a reader of the output
//! would have no way to learn that a method of a type they can see is missing. So every drop is
//! recorded here, with the refusal that caused it, and the survey reports each as its own refusal
//! against the method's own name.

use std::cell::RefCell;

/// One method dropped from a type that was otherwise emitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DroppedMethod {
    /// The type the method was declared on.
    pub owner: String,
    /// The method's own name, as the source spells it.
    pub name: String,
    /// Why it could not be translated — the refusal, unmodified.
    pub reason: String,
}

/// A per-run record of every method dropped from a type that was emitted.
///
/// Interior mutability for the same reason [`crate::ownership::DispositionLog`] has it: the decision
/// is made deep inside a build that hands back one value, and the alternative is threading an
/// out-parameter through every construction that can contain a method.
#[derive(Debug, Default)]
pub struct DropLog {
    records: RefCell<Vec<DroppedMethod>>,
}

impl DropLog {
    /// An empty log.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn record(&self, record: DroppedMethod) {
        self.records.borrow_mut().push(record);
    }

    /// Every method dropped, in the order they were dropped.
    #[must_use]
    pub fn records(&self) -> Vec<DroppedMethod> {
        self.records.borrow().clone()
    }

    /// Whether anything was dropped at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.borrow().is_empty()
    }
}
