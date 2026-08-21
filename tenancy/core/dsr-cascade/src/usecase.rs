//! The cascade runner: plan the fan-out, execute it step by step, and seal
//! the proof of erasure.
//!
//! The runner is synchronous and clock-free. `now` is threaded in as a
//! parameter on every call that can reach an SLA decision, so a cascade
//! replays identically in a test and in production.
//!
//! Idempotency has two layers, and both are exercised by the tests:
//!
//! 1. Before invoking a handler, the runner reads back the receipt already
//!    recorded for that microservice. If one exists the handler is NOT
//!    called again — re-running an erasure handler is real work against a
//!    real datastore, and a second invocation would produce a second
//!    evidence digest for the same deletion.
//! 2. The repository still rejects a second append for the same
//!    (tenant, request, microservice) triple with
//!    [`DsrKernelError::DuplicateReceipt`], so a racing runner cannot widen
//!    the tree behind the first one's back. A runner that LOSES that race
//!    re-reads the receipt and reports [`StepStatus::AlreadyDone`]; it does
//!    not abort the pass, because the microservices after it in plan order
//!    are still owed their erasure.
//!
//! # The plan is bound to its request
//!
//! [`CascadePlan`] has public fields and is freely constructible, so a
//! caller can hand [`CascadeRunner::run_pass`] a plan for one request and a
//! [`DsrRequest`] for another. Without a check, the handlers would erase one
//! subject while the certificate was sealed over the other's receipts — a
//! certificate asserting an erasure that pass never performed. Every entry
//! point therefore refuses a plan whose (tenant, request) identity is not
//! the request's own, BEFORE any handler runs.

use crate::domain::{compute_proof_of_erasure, sla_at_risk, sla_breached, sla_deadline};
use crate::kernel::{
    DpoOverride, DsrKernelError, DsrRequest, DsrRequestId, DsrRequestKey, DsrRequestRepository,
    ErasureReceipt, MicroserviceRegistry, ProofOfErasure, Timestamp,
};

/// The ordered fan-out a request must complete, plus its legal deadline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CascadePlan {
    pub tenant: String,             // data_class: INTERNAL_ONLY
    pub request: DsrRequestId,      // data_class: INTERNAL_ONLY
    pub microservices: Vec<String>, // data_class: INTERNAL_ONLY
    pub requested_at: Timestamp,    // data_class: INTERNAL_ONLY
    pub deadline: Timestamp,        // data_class: INTERNAL_ONLY
}

impl CascadePlan {
    /// How many microservices must report before the cascade is complete.
    #[must_use]
    pub fn expected_microservices(&self) -> usize {
        self.microservices.len()
    }

    /// The request identity this plan belongs to.
    #[must_use]
    pub fn key(&self) -> DsrRequestKey {
        DsrRequestKey::new(&self.tenant, &self.request)
    }

    /// Whether this plan was derived for `request`.
    #[must_use]
    pub fn belongs_to(&self, request: &DsrRequest) -> bool {
        self.tenant == request.tenant_id && self.request == request.id
    }
}

/// What one microservice's step of the cascade did.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StepStatus {
    /// A receipt already existed; the handler was deliberately not re-run.
    AlreadyDone { leaf: [u8; 32] },
    /// The handler ran in this pass and its receipt was appended.
    Completed { leaf: [u8; 32] },
    /// No handler is registered for a microservice the registry lists.
    HandlerMissing,
    /// The handler ran and refused; the step is retryable next pass.
    Failed { detail: String },
    /// The step could not be ATTEMPTED or recorded this pass — the store was
    /// unreachable, or an append raced and left no readable receipt. Nothing
    /// is known about whether the microservice holds data; retry next pass.
    Deferred { detail: String },
}

impl StepStatus {
    /// Whether this step contributed a receipt to the tree.
    #[must_use]
    pub const fn has_receipt(&self) -> bool {
        matches!(self, Self::AlreadyDone { .. } | Self::Completed { .. })
    }
}

/// One microservice's outcome in a cascade pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CascadeStep {
    pub microservice: String, // data_class: INTERNAL_ONLY
    pub status: StepStatus,   // data_class: INTERNAL_ONLY
}

/// The per-step picture of one cascade pass. A partially completed cascade
/// is a first-class value here, never an all-or-nothing failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CascadeState {
    pub tenant: String,          // data_class: INTERNAL_ONLY
    pub request: DsrRequestId,   // data_class: INTERNAL_ONLY
    pub steps: Vec<CascadeStep>, // data_class: INTERNAL_ONLY
    pub evaluated_at: Timestamp, // data_class: INTERNAL_ONLY
}

impl CascadeState {
    /// Microservices that still owe a receipt, in plan order.
    #[must_use]
    pub fn pending(&self) -> Vec<String> {
        self.steps
            .iter()
            .filter(|step| !step.status.has_receipt())
            .map(|step| step.microservice.clone())
            .collect()
    }

    /// Whether every planned microservice has reported.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.steps.iter().all(|step| step.status.has_receipt())
    }
}

/// The result of one cascade pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CascadeOutcome {
    /// Every microservice reported; the proof is sealed and stored.
    Sealed {
        proof: ProofOfErasure,
        state: CascadeState,
    },
    /// Some microservices still owe receipts, and the deadline has not
    /// passed. Run another pass.
    Incomplete {
        state: CascadeState,
        pending: Vec<String>,
        /// 80% or more of the statutory window is spent: alert the DPO.
        sla_at_risk: bool,
    },
}

/// Orchestrates DSR erasure cascades over a repository and a registry.
#[derive(Clone, Debug)]
pub struct CascadeRunner<R, M> {
    repository: R,
    registry: M,
}

impl<R: DsrRequestRepository, M: MicroserviceRegistry> CascadeRunner<R, M> {
    /// Build a runner over its two ports.
    pub const fn new(repository: R, registry: M) -> Self {
        Self {
            repository,
            registry,
        }
    }

    /// Borrow the repository (for read-back and assertions).
    pub const fn repository(&self) -> &R {
        &self.repository
    }

    /// Borrow the microservice registry.
    pub const fn registry(&self) -> &M {
        &self.registry
    }

    /// Borrow the microservice registry mutably.
    ///
    /// A registry changes underneath a live request — services are
    /// decommissioned and handlers are repaired inside the statutory window
    /// — and the runner must survive that, so mutating it is part of the
    /// contract rather than something only a fresh runner can do.
    pub const fn registry_mut(&mut self) -> &mut M {
        &mut self.registry
    }

    /// Derive the deterministic cascade plan for a request.
    ///
    /// The registry may return microservices in any order and may repeat
    /// them; the plan is deduplicated and sorted ascending, so the same
    /// registry contents always yield the same plan and therefore the same
    /// canonical tree.
    ///
    /// # Errors
    /// - [`DsrKernelError::InvalidRequest`] / [`DsrKernelError::UnsupportedKind`]
    ///   from [`DsrRequest::validate_for_erasure`].
    /// - [`DsrKernelError::EmptyCascadePlan`] when no microservice is active.
    /// - [`DsrKernelError::TimestampOverflow`] from the SLA deadline.
    pub fn plan(&self, request: &DsrRequest) -> Result<CascadePlan, DsrKernelError> {
        request.validate_for_erasure()?;
        let mut microservices = self.registry.list_active()?;
        microservices.sort();
        microservices.dedup();
        if microservices.is_empty() {
            return Err(DsrKernelError::EmptyCascadePlan);
        }
        Ok(CascadePlan {
            tenant: request.tenant_id.clone(),
            request: request.id.clone(),
            microservices,
            requested_at: request.requested_at,
            deadline: sla_deadline(request.pack, request.requested_at)?,
        })
    }

    /// Validate, open and plan a request in one step.
    ///
    /// # Errors
    /// Propagates [`Self::plan`] and the repository's `open`.
    pub fn submit(&self, request: &DsrRequest) -> Result<CascadePlan, DsrKernelError> {
        let plan = self.plan(request)?;
        self.repository.open(request)?;
        Ok(plan)
    }

    /// Refuse a plan that was not derived for `request`.
    fn check_binding(request: &DsrRequest, plan: &CascadePlan) -> Result<(), DsrKernelError> {
        request.validate_for_erasure()?;
        if plan.belongs_to(request) {
            Ok(())
        } else {
            Err(DsrKernelError::PlanRequestMismatch)
        }
    }

    /// Run one pass of the cascade over every planned microservice.
    ///
    /// Steps that already hold a receipt are skipped without re-invoking
    /// their handler. Failed, handler-less and deferred steps do not abort
    /// the pass: they are recorded and the remaining microservices still
    /// run, so one broken service cannot starve the rest of the erasure.
    /// Only a REQUEST-level error (unknown, invalid, already sealed — see
    /// [`DsrKernelError::is_request_terminal`]) ends the pass, because it is
    /// true of every remaining step anyway.
    ///
    /// # Errors
    /// - [`DsrKernelError::PlanRequestMismatch`] if `plan` was not derived
    ///   for `request`. Checked BEFORE any handler runs.
    /// - [`DsrKernelError::SlaBreached`] when the pass ends incomplete and
    ///   the statutory deadline has passed; it carries the tenant, the
    ///   request, the still-pending microservices and both instants.
    /// - [`DsrKernelError::UnknownRequest`] if the request was never opened.
    /// - Aggregation errors from sealing the proof.
    pub fn run_pass(
        &self,
        request: &DsrRequest,
        plan: &CascadePlan,
        now: Timestamp,
    ) -> Result<CascadeOutcome, DsrKernelError> {
        Self::check_binding(request, plan)?;
        let key = request.key();
        let mut steps = Vec::with_capacity(plan.microservices.len());
        for microservice in &plan.microservices {
            let status = self.step_status(request, &key, microservice)?;
            steps.push(CascadeStep {
                microservice: microservice.clone(),
                status,
            });
        }
        let state = CascadeState {
            tenant: key.tenant.clone(),
            request: key.request.clone(),
            steps,
            evaluated_at: now,
        };

        if state.is_complete() {
            let receipts = self.repository.receipts(&key)?;
            let proof = compute_proof_of_erasure(&key, &receipts, &plan.microservices, now, None)?;
            self.repository.finalize(&proof)?;
            return Ok(CascadeOutcome::Sealed { proof, state });
        }

        let pending = state.pending();
        if sla_breached(plan.deadline, now) {
            return Err(DsrKernelError::SlaBreached {
                tenant: key.tenant,
                request: key.request,
                pending,
                deadline: plan.deadline,
                now,
            });
        }
        Ok(CascadeOutcome::Incomplete {
            sla_at_risk: sla_at_risk(plan.requested_at, plan.deadline, now)?,
            state,
            pending,
        })
    }

    /// One step, with step-level errors turned into a recorded status so
    /// the pass continues. Request-level errors still propagate.
    fn step_status(
        &self,
        request: &DsrRequest,
        key: &DsrRequestKey,
        microservice: &str,
    ) -> Result<StepStatus, DsrKernelError> {
        match self.run_step(request, microservice) {
            Ok(status) => Ok(status),
            Err(error) if error.is_request_terminal() => Err(error),
            Err(DsrKernelError::DuplicateReceipt) => {
                // Lost the append race with a concurrent runner: the receipt
                // exists, this pass simply did not write it.
                match self.repository.receipt(key, microservice) {
                    Ok(Some(existing)) => Ok(StepStatus::AlreadyDone {
                        leaf: existing.merkle_leaf,
                    }),
                    Ok(None) => Ok(StepStatus::Deferred {
                        detail: DsrKernelError::DuplicateReceipt.to_string(),
                    }),
                    Err(error) if error.is_request_terminal() => Err(error),
                    Err(error) => Ok(StepStatus::Deferred {
                        detail: error.to_string(),
                    }),
                }
            }
            Err(error) => Ok(StepStatus::Deferred {
                detail: error.to_string(),
            }),
        }
    }

    /// Execute (or skip) one microservice's erasure.
    ///
    /// # Errors
    /// Repository failures, and [`DsrKernelError::DuplicateReceipt`] if a
    /// concurrent runner appended between this runner's read and write.
    pub fn run_step(
        &self,
        request: &DsrRequest,
        microservice: &str,
    ) -> Result<StepStatus, DsrKernelError> {
        let key = request.key();
        if let Some(existing) = self.repository.receipt(&key, microservice)? {
            // Layer 1 idempotency: the handler is NOT re-invoked.
            return Ok(StepStatus::AlreadyDone {
                leaf: existing.merkle_leaf,
            });
        }
        let Some(handler) = self.registry.handler(microservice) else {
            return Ok(StepStatus::HandlerMissing);
        };
        match handler.erase(request) {
            Ok(merkle_leaf) => {
                let receipt = ErasureReceipt {
                    tenant: key.tenant,
                    request: key.request,
                    microservice: microservice.to_owned(),
                    merkle_leaf,
                };
                self.repository.append_receipt(&receipt)?;
                Ok(StepStatus::Completed { leaf: merkle_leaf })
            }
            // Handler text is third-party controlled: bound it here rather
            // than carrying an arbitrary string into cascade state.
            Err(failure) => Ok(StepStatus::Failed {
                detail: failure.bounded_detail(),
            }),
        }
    }

    /// Seal an incomplete cascade under a two-person DPO waiver.
    ///
    /// The only sanctioned path to a proof whose coverage is short of the
    /// plan. The waiver is recorded inside the certificate, so a regulator
    /// can always tell a complete proof from a waived one.
    ///
    /// `request` is taken (and checked against `plan`) for the same reason
    /// [`Self::run_pass`] takes it: a waiver holder must not be able to seal
    /// a certificate over some other request's receipts.
    ///
    /// # Errors
    /// [`DsrKernelError::PlanRequestMismatch`] for a plan that is not this
    /// request's, [`DsrKernelError::InvalidDpoOverride`] for a waiver that
    /// is not dual control, plus the aggregation and repository errors.
    pub fn seal_with_dpo_override(
        &self,
        request: &DsrRequest,
        plan: &CascadePlan,
        now: Timestamp,
        waiver: DpoOverride,
    ) -> Result<ProofOfErasure, DsrKernelError> {
        Self::check_binding(request, plan)?;
        waiver.validate()?;
        let key = request.key();
        let receipts = self.repository.receipts(&key)?;
        let proof =
            compute_proof_of_erasure(&key, &receipts, &plan.microservices, now, Some(waiver))?;
        self.repository.finalize(&proof)?;
        Ok(proof)
    }
}
