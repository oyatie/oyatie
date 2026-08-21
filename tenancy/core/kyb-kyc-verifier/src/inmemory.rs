//! A deterministic in-memory [`ScreeningPort`] for tests and local runs.
//!
//! It is a fixture registry, not a simulator: every answer is one you put in.
//! An unregistered subject is FAIL-CLOSED — it reports the provider as
//! unavailable rather than inventing a clear result, so a fixture you forgot to
//! register can never quietly approve a tenant.
//!
//! Every builder goes through the validated constructors on
//! [`ScreeningResult`] rather than a struct literal, so a fixture cannot
//! express a state the domain refuses to construct. That is why the builders
//! are fallible.

use std::collections::{BTreeMap, BTreeSet};

use crate::kernel::{
    ScreeningCheck, ScreeningResolution, ScreeningResult, VerificationError, VerificationKind,
};
use crate::usecase::{ScreeningError, ScreeningPort, ScreeningRequest};

/// An in-memory screening provider built from explicit fixtures.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryScreeningProvider {
    name: String,
    responses: BTreeMap<String, Vec<ScreeningResult>>,
    outages: BTreeSet<String>,
    timeouts: BTreeSet<String>,
    supported_kinds: Vec<VerificationKind>,
}

impl InMemoryScreeningProvider {
    /// Build a provider under a non-empty name.
    pub fn new(name: String) -> Result<Self, VerificationError> {
        if name.trim().is_empty() {
            return Err(VerificationError::EmptyProvider);
        }
        Ok(Self {
            name,
            responses: BTreeMap::new(),
            outages: BTreeSet::new(),
            timeouts: BTreeSet::new(),
            supported_kinds: Vec::new(),
        })
    }

    /// The provider name reported on results and failures.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Restrict which verification kinds this provider will screen. An empty
    /// list (the default) means every kind.
    #[must_use]
    pub fn supporting(mut self, kinds: Vec<VerificationKind>) -> Self {
        self.supported_kinds = kinds;
        self
    }

    /// Register an explicit, already-validated result for a subject.
    ///
    /// Several results may be registered for one subject; they are returned as
    /// ONE response, which is how a real vendor answers sanctions, PEP and
    /// adverse-media in a single call.
    #[must_use]
    pub fn with_result(mut self, subject_ref: &str, result: ScreeningResult) -> Self {
        self.responses
            .entry(subject_ref.trim().to_owned())
            .or_default()
            .push(result);
        self
    }

    /// Register an answer to one screening question for a subject.
    pub fn with_check(
        self,
        subject_ref: &str,
        check: ScreeningCheck,
        hit: bool,
        resolution: ScreeningResolution,
        details: &str,
    ) -> Result<Self, VerificationError> {
        let result = ScreeningResult::for_check(
            self.name.clone(),
            check,
            hit,
            details.to_owned(),
            resolution,
        )?;
        Ok(self.with_result(subject_ref, result))
    }

    /// Register a clear (no-hit) sanctions answer for a subject.
    pub fn with_clear(self, subject_ref: &str) -> Result<Self, VerificationError> {
        self.with_check(
            subject_ref,
            ScreeningCheck::Sanctions,
            false,
            ScreeningResolution::Unresolved,
            "",
        )
    }

    /// Register a clear (no-hit) answer to one question for a subject.
    pub fn with_clear_for_check(
        self,
        subject_ref: &str,
        check: ScreeningCheck,
    ) -> Result<Self, VerificationError> {
        self.with_check(
            subject_ref,
            check,
            false,
            ScreeningResolution::Unresolved,
            "",
        )
    }

    /// Register a sanctions hit for a subject at a given resolution.
    pub fn with_hit(
        self,
        subject_ref: &str,
        resolution: ScreeningResolution,
        details: &str,
    ) -> Result<Self, VerificationError> {
        self.with_check(
            subject_ref,
            ScreeningCheck::Sanctions,
            true,
            resolution,
            details,
        )
    }

    /// Make this subject's lookup fail as unavailable.
    #[must_use]
    pub fn with_outage(mut self, subject_ref: &str) -> Self {
        self.outages.insert(subject_ref.trim().to_owned());
        self
    }

    /// Make this subject's lookup fail as a timeout.
    #[must_use]
    pub fn with_timeout(mut self, subject_ref: &str) -> Self {
        self.timeouts.insert(subject_ref.trim().to_owned());
        self
    }
}

impl ScreeningPort for InMemoryScreeningProvider {
    fn screen(&self, request: &ScreeningRequest) -> Result<Vec<ScreeningResult>, ScreeningError> {
        let subject = request.subject_ref.trim();
        if subject.is_empty() {
            return Err(ScreeningError::EmptySubjectRef);
        }
        if !self.supported_kinds.is_empty() && !self.supported_kinds.contains(&request.kind) {
            return Err(ScreeningError::UnsupportedKind {
                provider: self.name.clone(),
                kind: request.kind,
            });
        }
        if self.timeouts.contains(subject) {
            return Err(ScreeningError::ProviderTimeout {
                provider: self.name.clone(),
            });
        }
        if self.outages.contains(subject) {
            return Err(ScreeningError::ProviderUnavailable {
                provider: self.name.clone(),
            });
        }
        match self.responses.get(subject) {
            Some(results) => Ok(results.clone()),
            // Fail closed: an unknown subject is an unanswered question, and an
            // unanswered question must never read as "clear".
            None => Err(ScreeningError::ProviderUnavailable {
                provider: self.name.clone(),
            }),
        }
    }
}
