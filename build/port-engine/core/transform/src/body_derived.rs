//! The same body, told one more thing.
//!
//! Split from `body.rs` because the two are different subjects: that file says what a body IS and
//! what a translation needs from it, and this says how one is DERIVED from another — the same body
//! inside a loop that walks a sequence, or with one more counter proven to be an index.
//!
//! Every one of these is scoped to what proved it. A fact learned inside a loop belongs to that
//! loop, and a body that carried it outward would translate a name by evidence that no longer
//! applies to it.

use std::collections::BTreeSet;

use crate::body::{Body, Walked};

impl<'a> Body<'a> {
    /// The same body, with the parameters the signature already made the target's index type.
    pub(crate) fn with_usize_parameters(mut self, names: BTreeSet<String>) -> Self {
        self.usize_counters.extend(names);
        self
    }

    /// The same body, told which of its parameters are newtypes the target wraps.
    pub(crate) fn with_newtype_parameters(mut self, names: BTreeSet<String>) -> Self {
        self.newtype_parameters.extend(names);
        self
    }

    /// The same body, told its signature dropped a failure result the returns still carry.
    pub(crate) fn with_dropped_failure(mut self, dropped: bool) -> Self {
        self.drops_absent_failure = dropped;
        self
    }

    /// The same body, told the NAMED results a bare return hands back.
    pub(crate) fn with_named_results(mut self, names: Vec<String>) -> Self {
        self.named_results = names;
        self
    }

    /// The same body, told which type it is a method OF.
    pub(crate) fn with_receiver_type(mut self, name: Option<&'a str>) -> Self {
        self.receiver_type = name;
        self
    }

    /// The same body, inside a loop that WALKS a sequence rather than counting into it.
    pub(crate) fn with_element(&self, counter: &str, sequence: &str, element: &str) -> Self {
        Self {
            owner: self.owner,
            newtype_parameters: self.newtype_parameters.clone(),
            resolver: self.resolver,
            fallible: self.fallible,
            result_is_owned_string: self.result_is_owned_string,
            result_is_owned_sequence: self.result_is_owned_sequence.clone(),
            drops_absent_failure: self.drops_absent_failure,
            named_results: self.named_results.clone(),
            borrowed: self.borrowed.clone(),
            results: self.results.clone(),
            usize_counters: self.usize_counters.clone(),
            walked: Some(Walked {
                counter: counter.to_owned(),
                sequence: sequence.to_owned(),
                element: element.to_owned(),
            }),
            receiver_type: self.receiver_type,
        }
    }

    /// The same body, translating one more counter as a `usize`.
    ///
    /// Scoped to the loop that proved it: a name shadowed by an inner loop with different uses gets
    /// its own answer, and nothing outside the loop is affected by what happens inside it.
    pub(crate) fn with_usize_counter(&self, counter: &str) -> Self {
        let mut usize_counters = self.usize_counters.clone();
        usize_counters.insert(counter.to_owned());
        Self {
            owner: self.owner,
            newtype_parameters: self.newtype_parameters.clone(),
            resolver: self.resolver,
            fallible: self.fallible,
            result_is_owned_string: self.result_is_owned_string,
            result_is_owned_sequence: self.result_is_owned_sequence.clone(),
            drops_absent_failure: self.drops_absent_failure,
            named_results: self.named_results.clone(),
            borrowed: self.borrowed.clone(),
            results: self.results.clone(),
            usize_counters,
            walked: self.walked.clone(),
            receiver_type: self.receiver_type,
        }
    }
}
