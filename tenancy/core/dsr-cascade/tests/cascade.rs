//! End-to-end cascade behaviour: deterministic planning, tenant isolation,
//! plan/request binding, partial fan-out, idempotent replay, a shrinking
//! registry, SLA breach, and DPO-waived sealing.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::{Arc, Mutex};

use tenancy_dsr_cascade::domain::{sla_deadline, verify_proof_of_erasure};
use tenancy_dsr_cascade::inmemory::{InMemoryDsrRepository, InMemoryRegistry, RecordingHandler};
use tenancy_dsr_cascade::kernel::{
    DpoOverride, DsrKernelError, DsrKind, DsrRequest, DsrRequestId, DsrRequestKey,
    DsrRequestRepository, ErasureHandler, ErasureReceipt, HandlerFailure, MAX_HANDLER_DETAIL_BYTES,
    MicroserviceRegistry, ProofOfErasure, RegulatoryPack, Timestamp,
};
use tenancy_dsr_cascade::usecase::{CascadeOutcome, CascadeRunner, StepStatus};

const DAY: i64 = 86_400;
const REQUESTED_AT: Timestamp = Timestamp(1_700_000_000);

fn request(pack: RegulatoryPack) -> DsrRequest {
    DsrRequest {
        id: DsrRequestId("dsr_alpha".to_owned()),
        tenant_id: "ten_alpha".to_owned(),
        subject_id: "subject-7".to_owned(),
        kind: DsrKind::Erasure,
        pack,
        requested_at: REQUESTED_AT,
    }
}

/// A registry whose `list_active` deliberately returns an unsorted list
/// with a duplicate, to prove the planner normalises it.
struct NoisyRegistry {
    inner: InMemoryRegistry,
    noise: Vec<String>,
}

impl MicroserviceRegistry for NoisyRegistry {
    fn list_active(&self) -> Result<Vec<String>, DsrKernelError> {
        Ok(self.noise.clone())
    }

    fn handler(&self, microservice: &str) -> Option<&dyn ErasureHandler> {
        self.inner.handler(microservice)
    }
}

/// A repository that is unreachable for ONE microservice's receipt read,
/// and healthy for every other. Models a partially degraded store.
struct PartlyBrokenRepository {
    inner: InMemoryDsrRepository,
    broken: String,
}

impl DsrRequestRepository for PartlyBrokenRepository {
    fn open(&self, request: &DsrRequest) -> Result<(), DsrKernelError> {
        self.inner.open(request)
    }

    fn append_receipt(&self, receipt: &ErasureReceipt) -> Result<(), DsrKernelError> {
        self.inner.append_receipt(receipt)
    }

    fn finalize(&self, proof: &ProofOfErasure) -> Result<(), DsrKernelError> {
        self.inner.finalize(proof)
    }

    fn receipt(
        &self,
        key: &DsrRequestKey,
        microservice: &str,
    ) -> Result<Option<ErasureReceipt>, DsrKernelError> {
        if microservice == self.broken {
            return Err(DsrKernelError::RepositoryUnavailable);
        }
        self.inner.receipt(key, microservice)
    }

    fn receipts(&self, key: &DsrRequestKey) -> Result<Vec<ErasureReceipt>, DsrKernelError> {
        self.inner.receipts(key)
    }

    fn proof(&self, key: &DsrRequestKey) -> Result<Option<ProofOfErasure>, DsrKernelError> {
        self.inner.proof(key)
    }
}

/// A repository that hides one microservice's receipt on the FIRST read,
/// reproducing the runner-versus-runner race the usecase module documents:
/// this runner reads "no receipt", calls the handler, and only then learns
/// another runner already appended one.
struct RacingRepository {
    inner: InMemoryDsrRepository,
    raced: String,
    reads: Mutex<usize>,
}

impl DsrRequestRepository for RacingRepository {
    fn open(&self, request: &DsrRequest) -> Result<(), DsrKernelError> {
        self.inner.open(request)
    }

    fn append_receipt(&self, receipt: &ErasureReceipt) -> Result<(), DsrKernelError> {
        self.inner.append_receipt(receipt)
    }

    fn finalize(&self, proof: &ProofOfErasure) -> Result<(), DsrKernelError> {
        self.inner.finalize(proof)
    }

    fn receipt(
        &self,
        key: &DsrRequestKey,
        microservice: &str,
    ) -> Result<Option<ErasureReceipt>, DsrKernelError> {
        if microservice == self.raced {
            let mut reads = self.reads.lock().unwrap();
            *reads += 1;
            if *reads == 1 {
                return Ok(None);
            }
        }
        self.inner.receipt(key, microservice)
    }

    fn receipts(&self, key: &DsrRequestKey) -> Result<Vec<ErasureReceipt>, DsrKernelError> {
        self.inner.receipts(key)
    }

    fn proof(&self, key: &DsrRequestKey) -> Result<Option<ProofOfErasure>, DsrKernelError> {
        self.inner.proof(key)
    }
}

fn registry_with(names: &[&str]) -> InMemoryRegistry {
    let mut registry = InMemoryRegistry::new();
    for name in names {
        registry.register(name, Box::new(RecordingHandler::succeeding(name)));
    }
    registry
}

fn waiver() -> DpoOverride {
    DpoOverride {
        first_approver: "dpo-a".to_owned(),
        second_approver: "dpo-b".to_owned(),
        reason: "legacy-dw decommissioned 2026-01".to_owned(),
    }
}

#[test]
fn plan_is_deterministic_deduplicated_and_sorted() {
    let registry = NoisyRegistry {
        inner: registry_with(&["mail", "billing", "crm"]),
        noise: vec![
            "mail".to_owned(),
            "crm".to_owned(),
            "billing".to_owned(),
            "mail".to_owned(),
        ],
    };
    let runner = CascadeRunner::new(InMemoryDsrRepository::new(), registry);
    let request = request(RegulatoryPack::Eu);

    let plan = runner.plan(&request).unwrap();
    assert_eq!(plan.microservices, vec!["billing", "crm", "mail"]);
    assert_eq!(plan.expected_microservices(), 3);
    assert_eq!(plan.tenant, "ten_alpha");
    assert!(plan.belongs_to(&request));
    assert_eq!(
        plan.deadline,
        sla_deadline(RegulatoryPack::Eu, REQUESTED_AT).unwrap()
    );
    // Same registry, same plan — every time.
    assert_eq!(plan, runner.plan(&request).unwrap());
}

#[test]
fn an_empty_registry_cannot_produce_a_plan() {
    let runner = CascadeRunner::new(InMemoryDsrRepository::new(), InMemoryRegistry::new());
    assert_eq!(
        runner.plan(&request(RegulatoryPack::Eu)).unwrap_err(),
        DsrKernelError::EmptyCascadePlan
    );
}

#[test]
fn a_complete_pass_seals_a_verifiable_proof() {
    let runner = CascadeRunner::new(
        InMemoryDsrRepository::new(),
        registry_with(&["billing", "crm", "mail"]),
    );
    let request = request(RegulatoryPack::Eu);
    let plan = runner.submit(&request).unwrap();

    let outcome = runner
        .run_pass(&request, &plan, Timestamp(REQUESTED_AT.0 + DAY))
        .unwrap();
    let CascadeOutcome::Sealed { proof, state } = outcome else {
        panic!("a fully handled cascade must seal");
    };
    assert!(state.is_complete());
    assert_eq!(proof.receipts.len(), 3);
    assert_eq!(proof.expected_microservices, 3);
    assert_eq!(proof.covered_microservices, plan.microservices);
    assert_eq!(proof.tenant, "ten_alpha");
    assert!(proof.dpo_override.is_none());
    assert_eq!(proof.sealed_at, Timestamp(REQUESTED_AT.0 + DAY));
    verify_proof_of_erasure(&proof).unwrap();
    // Receipts are stored in canonical microservice order.
    let names: Vec<&str> = proof
        .receipts
        .iter()
        .map(|receipt| receipt.microservice.as_str())
        .collect();
    assert_eq!(names, vec!["billing", "crm", "mail"]);
    assert_eq!(
        runner.repository().proof(&plan.key()).unwrap().as_ref(),
        Some(&proof)
    );
}

#[test]
fn two_tenants_sharing_a_request_id_run_independent_cascades() {
    // Request ids are caller-supplied and tenant-local, so per-tenant
    // numbering collides by accident. If the store keyed on the id alone,
    // beta's cascade would read alpha's receipts as its own: one certificate
    // would discharge two tenants' Art. 17 obligations while each tenant's
    // data survived in the microservice the other had already "covered".
    let billing = Arc::new(RecordingHandler::succeeding("billing"));
    let mail = Arc::new(RecordingHandler::succeeding("mail"));
    let mut registry = InMemoryRegistry::new();
    registry.register("billing", Box::new(Arc::clone(&billing)));
    registry.register("mail", Box::new(Arc::clone(&mail)));
    let runner = CascadeRunner::new(InMemoryDsrRepository::new(), registry);

    let alpha = request(RegulatoryPack::Eu);
    let mut beta = request(RegulatoryPack::Eu);
    beta.tenant_id = "ten_beta".to_owned();
    beta.subject_id = "bob@example.com".to_owned();
    assert_eq!(alpha.id, beta.id, "the ids collide on purpose");

    let alpha_plan = runner.submit(&alpha).unwrap();
    let beta_plan = runner.submit(&beta).unwrap();
    let now = Timestamp(REQUESTED_AT.0 + DAY);

    let CascadeOutcome::Sealed {
        proof: alpha_proof, ..
    } = runner.run_pass(&alpha, &alpha_plan, now).unwrap()
    else {
        panic!("alpha should seal");
    };
    let CascadeOutcome::Sealed {
        proof: beta_proof, ..
    } = runner.run_pass(&beta, &beta_plan, now).unwrap()
    else {
        panic!("beta should seal");
    };

    // Every handler ran for BOTH tenants: nothing was skipped as "already
    // done" on the strength of the other tenant's receipt.
    assert_eq!(billing.invocations().unwrap(), 2);
    assert_eq!(mail.invocations().unwrap(), 2);
    assert_eq!(alpha_proof.tenant, "ten_alpha");
    assert_eq!(beta_proof.tenant, "ten_beta");
    assert_ne!(
        alpha_proof.merkle_root, beta_proof.merkle_root,
        "two tenants must not share one certificate root"
    );
    verify_proof_of_erasure(&alpha_proof).unwrap();
    verify_proof_of_erasure(&beta_proof).unwrap();
    assert_eq!(runner.repository().open_request_count().unwrap(), 2);
}

#[test]
fn a_pass_refuses_a_plan_belonging_to_another_request() {
    // The plan and the request are independent parameters, and CascadePlan
    // is freely constructible. Without the binding check, the handlers would
    // erase THIS request's subject while the certificate was sealed over the
    // OTHER request's receipts.
    let billing = Arc::new(RecordingHandler::succeeding("billing"));
    let mut registry = InMemoryRegistry::new();
    registry.register("billing", Box::new(Arc::clone(&billing)));
    let runner = CascadeRunner::new(InMemoryDsrRepository::new(), registry);

    let alpha = request(RegulatoryPack::Eu);
    let mut beta = request(RegulatoryPack::Eu);
    beta.id = DsrRequestId("dsr_beta".to_owned());
    let alpha_plan = runner.submit(&alpha).unwrap();
    runner.submit(&beta).unwrap();
    let now = Timestamp(REQUESTED_AT.0 + DAY);

    assert_eq!(
        runner.run_pass(&beta, &alpha_plan, now).unwrap_err(),
        DsrKernelError::PlanRequestMismatch
    );
    assert_eq!(
        billing.invocations().unwrap(),
        0,
        "the refusal must land BEFORE any erasure handler runs"
    );
    assert_eq!(
        runner.repository().receipt_count(&beta.key()).unwrap(),
        0,
        "no receipt may be attributed to a request the caller was not running"
    );
    assert!(runner.repository().proof(&alpha.key()).unwrap().is_none());
}

#[test]
fn a_pass_refuses_a_plan_belonging_to_another_tenant() {
    let runner = CascadeRunner::new(InMemoryDsrRepository::new(), registry_with(&["billing"]));
    let alpha = request(RegulatoryPack::Eu);
    let mut beta = request(RegulatoryPack::Eu);
    beta.tenant_id = "ten_beta".to_owned();
    let alpha_plan = runner.submit(&alpha).unwrap();
    runner.submit(&beta).unwrap();
    assert_eq!(alpha_plan.request, beta.id, "same id, different tenant");
    assert_eq!(
        runner
            .run_pass(&beta, &alpha_plan, Timestamp(REQUESTED_AT.0 + DAY))
            .unwrap_err(),
        DsrKernelError::PlanRequestMismatch
    );
}

#[test]
fn a_waiver_holder_cannot_seal_over_another_requests_receipts() {
    let runner = CascadeRunner::new(InMemoryDsrRepository::new(), registry_with(&["billing"]));
    let alpha = request(RegulatoryPack::Eu);
    let mut beta = request(RegulatoryPack::Eu);
    beta.id = DsrRequestId("dsr_beta".to_owned());
    let alpha_plan = runner.submit(&alpha).unwrap();
    runner.submit(&beta).unwrap();

    assert_eq!(
        runner
            .seal_with_dpo_override(
                &beta,
                &alpha_plan,
                Timestamp(REQUESTED_AT.0 + DAY),
                waiver()
            )
            .unwrap_err(),
        DsrKernelError::PlanRequestMismatch
    );
    assert!(runner.repository().proof(&alpha.key()).unwrap().is_none());
    assert!(runner.repository().proof(&beta.key()).unwrap().is_none());
}

#[test]
fn a_partial_cascade_is_representable_and_keeps_running() {
    let mut registry = InMemoryRegistry::new();
    registry.register("billing", Box::new(RecordingHandler::succeeding("billing")));
    registry.register(
        "crm",
        Box::new(RecordingHandler::failing("crm", "shard unreachable")),
    );
    registry.register_without_handler("legacy-dw");
    registry.register("mail", Box::new(RecordingHandler::succeeding("mail")));
    let runner = CascadeRunner::new(InMemoryDsrRepository::new(), registry);
    let request = request(RegulatoryPack::Eu);
    let plan = runner.submit(&request).unwrap();

    let outcome = runner
        .run_pass(&request, &plan, Timestamp(REQUESTED_AT.0 + DAY))
        .unwrap();
    let CascadeOutcome::Incomplete {
        state,
        pending,
        sla_at_risk,
    } = outcome
    else {
        panic!("a cascade with a failed step must not seal");
    };
    assert!(!sla_at_risk);
    assert_eq!(pending, vec!["crm".to_owned(), "legacy-dw".to_owned()]);
    // The failure did not stop the microservices after it in plan order.
    assert!(matches!(
        state.steps[3].status,
        StepStatus::Completed { .. }
    ));
    assert_eq!(state.steps[2].status, StepStatus::HandlerMissing);
    assert_eq!(
        state.steps[1].status,
        StepStatus::Failed {
            detail: "shard unreachable".to_owned()
        }
    );
    assert!(runner.repository().proof(&plan.key()).unwrap().is_none());
}

#[test]
fn an_unreachable_store_defers_one_step_without_starving_the_rest() {
    // A repository error inside the loop must not abort the pass: the
    // microservices after the broken one in plan order are still owed their
    // erasure, and the caller still needs the per-step diagnosis.
    let mail = Arc::new(RecordingHandler::succeeding("mail"));
    let mut registry = InMemoryRegistry::new();
    registry.register("billing", Box::new(RecordingHandler::succeeding("billing")));
    registry.register("mail", Box::new(Arc::clone(&mail)));
    let repository = PartlyBrokenRepository {
        inner: InMemoryDsrRepository::new(),
        broken: "billing".to_owned(),
    };
    let runner = CascadeRunner::new(repository, registry);
    let request = request(RegulatoryPack::Eu);
    let plan = runner.submit(&request).unwrap();

    let CascadeOutcome::Incomplete { state, pending, .. } = runner
        .run_pass(&request, &plan, Timestamp(REQUESTED_AT.0 + DAY))
        .unwrap()
    else {
        panic!("the pass must report state, not abort");
    };
    assert_eq!(
        state.steps[0].status,
        StepStatus::Deferred {
            detail: DsrKernelError::RepositoryUnavailable.to_string()
        }
    );
    assert!(matches!(
        state.steps[1].status,
        StepStatus::Completed { .. }
    ));
    assert_eq!(mail.invocations().unwrap(), 1, "mail was not starved");
    assert_eq!(pending, vec!["billing".to_owned()]);
}

#[test]
fn losing_the_append_race_reports_already_done_and_continues() {
    let mut registry = InMemoryRegistry::new();
    registry.register("billing", Box::new(RecordingHandler::succeeding("billing")));
    registry.register("mail", Box::new(RecordingHandler::succeeding("mail")));
    let inner = InMemoryDsrRepository::new();
    let request = request(RegulatoryPack::Eu);
    inner.open(&request).unwrap();
    // Another runner got there first.
    inner
        .append_receipt(&ErasureReceipt {
            tenant: request.tenant_id.clone(),
            request: request.id.clone(),
            microservice: "billing".to_owned(),
            merkle_leaf: [0x5a_u8; 32],
        })
        .unwrap();
    let repository = RacingRepository {
        inner,
        raced: "billing".to_owned(),
        reads: Mutex::new(0),
    };
    let runner = CascadeRunner::new(repository, registry);
    let plan = runner.plan(&request).unwrap();

    let CascadeOutcome::Sealed { proof, state } = runner
        .run_pass(&request, &plan, Timestamp(REQUESTED_AT.0 + DAY))
        .unwrap()
    else {
        panic!("the lost race must not stop the pass from completing");
    };
    assert_eq!(
        state.steps[0].status,
        StepStatus::AlreadyDone {
            leaf: [0x5a_u8; 32]
        }
    );
    assert!(matches!(
        state.steps[1].status,
        StepStatus::Completed { .. }
    ));
    assert_eq!(proof.receipts.len(), 2);
    verify_proof_of_erasure(&proof).unwrap();
}

#[test]
fn replaying_a_pass_does_not_re_invoke_a_handler_or_duplicate_a_receipt() {
    let billing = Arc::new(RecordingHandler::succeeding("billing"));
    let mut registry = InMemoryRegistry::new();
    registry.register("billing", Box::new(Arc::clone(&billing)));
    registry.register(
        "crm",
        Box::new(RecordingHandler::failing("crm", "shard unreachable")),
    );
    let runner = CascadeRunner::new(InMemoryDsrRepository::new(), registry);
    let request = request(RegulatoryPack::Eu);
    let plan = runner.submit(&request).unwrap();

    let now = Timestamp(REQUESTED_AT.0 + DAY);
    let first = runner.run_pass(&request, &plan, now).unwrap();
    assert!(matches!(first, CascadeOutcome::Incomplete { .. }));

    // Replay: billing already has a receipt, so its handler must NOT run
    // again and no second receipt may be appended.
    let second = runner.run_pass(&request, &plan, now).unwrap();
    let CascadeOutcome::Incomplete { state, .. } = second else {
        panic!("still incomplete");
    };
    assert!(matches!(
        state.steps[0].status,
        StepStatus::AlreadyDone { .. }
    ));
    assert_eq!(
        billing.invocations().unwrap(),
        1,
        "an erasure handler that already produced a receipt must never run twice"
    );
    assert_eq!(runner.repository().receipt_count(&plan.key()).unwrap(), 1);
    assert_eq!(
        runner
            .repository()
            .receipts(&plan.key())
            .unwrap()
            .iter()
            .filter(|receipt| receipt.microservice == "billing")
            .count(),
        1
    );
}

#[test]
fn a_replayed_step_leaves_the_sealed_root_unchanged() {
    let runner = CascadeRunner::new(
        InMemoryDsrRepository::new(),
        registry_with(&["billing", "mail"]),
    );
    let request = request(RegulatoryPack::Eu);
    let plan = runner.submit(&request).unwrap();
    let now = Timestamp(REQUESTED_AT.0 + DAY);

    // Run every step twice before sealing.
    for microservice in &plan.microservices {
        runner.run_step(&request, microservice).unwrap();
        let replay = runner.run_step(&request, microservice).unwrap();
        assert!(matches!(replay, StepStatus::AlreadyDone { .. }));
    }
    let CascadeOutcome::Sealed { proof, .. } = runner.run_pass(&request, &plan, now).unwrap()
    else {
        panic!("cascade should seal");
    };
    assert_eq!(proof.receipts.len(), 2);

    // A fresh runner over a fresh repository, run once, must agree.
    let clean = CascadeRunner::new(
        InMemoryDsrRepository::new(),
        registry_with(&["billing", "mail"]),
    );
    let clean_plan = clean.submit(&request).unwrap();
    let CascadeOutcome::Sealed {
        proof: clean_proof, ..
    } = clean.run_pass(&request, &clean_plan, now).unwrap()
    else {
        panic!("cascade should seal");
    };
    assert_eq!(proof.merkle_root, clean_proof.merkle_root);
}

#[test]
fn a_second_seal_is_refused() {
    let runner = CascadeRunner::new(InMemoryDsrRepository::new(), registry_with(&["billing"]));
    let request = request(RegulatoryPack::Eu);
    let plan = runner.submit(&request).unwrap();
    let now = Timestamp(REQUESTED_AT.0 + DAY);
    runner.run_pass(&request, &plan, now).unwrap();
    assert_eq!(
        runner.run_pass(&request, &plan, now).unwrap_err(),
        DsrKernelError::AlreadyFinalized
    );
}

#[test]
fn a_registry_that_shrinks_mid_window_can_still_be_sealed() {
    // Pass 1: a, b and c are active; a and b report, c has no handler.
    // Between passes b is decommissioned and c gains a handler. The plan is
    // re-derived (nothing persists it), so the accumulated receipts now
    // OUTNUMBER the plan. Comparing counts would refuse both seal paths for
    // ever, leaving a fully erased subject with no obtainable certificate.
    let mut registry = InMemoryRegistry::new();
    registry.register("a", Box::new(RecordingHandler::succeeding("a")));
    registry.register("b", Box::new(RecordingHandler::succeeding("b")));
    registry.register_without_handler("c");
    let mut runner = CascadeRunner::new(InMemoryDsrRepository::new(), registry);
    let request = request(RegulatoryPack::Eu);
    let plan = runner.submit(&request).unwrap();
    let first = runner
        .run_pass(&request, &plan, Timestamp(REQUESTED_AT.0 + DAY))
        .unwrap();
    assert!(matches!(first, CascadeOutcome::Incomplete { .. }));
    assert_eq!(runner.repository().receipt_count(&plan.key()).unwrap(), 2);

    // The registry changes underneath the request.
    {
        let registry = runner.registry_mut();
        registry.decommission("b");
        registry.register("c", Box::new(RecordingHandler::succeeding("c")));
    }
    let plan = runner.plan(&request).unwrap();
    assert_eq!(plan.microservices, vec!["a", "c"]);

    let CascadeOutcome::Sealed { proof, .. } = runner
        .run_pass(&request, &plan, Timestamp(REQUESTED_AT.0 + 2 * DAY))
        .unwrap()
    else {
        panic!("every planned microservice reported; this must seal");
    };
    assert_eq!(
        proof.receipts.len(),
        3,
        "b's receipt is evidence, not noise"
    );
    assert_eq!(proof.covered_microservices, vec!["a", "c"]);
    assert_eq!(proof.expected_microservices, 2);
    assert!(
        proof.dpo_override.is_none(),
        "nothing is missing, so no waiver is owed"
    );
    verify_proof_of_erasure(&proof).unwrap();
}

#[test]
fn a_complete_cascade_still_seals_on_the_deadline_day() {
    let runner = CascadeRunner::new(
        InMemoryDsrRepository::new(),
        registry_with(&["billing", "mail"]),
    );
    let request = request(RegulatoryPack::UsHc);
    let plan = runner.submit(&request).unwrap();
    assert_eq!(plan.deadline, Timestamp(REQUESTED_AT.0 + 7 * DAY));
    let outcome = runner
        .run_pass(&request, &plan, Timestamp(REQUESTED_AT.0 + 7 * DAY))
        .unwrap();
    assert!(matches!(outcome, CascadeOutcome::Sealed { .. }));
}

#[test]
fn an_incomplete_cascade_past_its_deadline_breaches_with_a_full_diagnosis() {
    let mut registry = InMemoryRegistry::new();
    registry.register("billing", Box::new(RecordingHandler::succeeding("billing")));
    registry.register(
        "crm",
        Box::new(RecordingHandler::failing("crm", "shard down")),
    );
    registry.register_without_handler("legacy-dw");
    let runner = CascadeRunner::new(InMemoryDsrRepository::new(), registry);
    // Brazil: a 15-day window, so day 16 is a breach and day 15 is not.
    let request = request(RegulatoryPack::Br);
    let plan = runner.submit(&request).unwrap();
    assert_eq!(plan.deadline, Timestamp(REQUESTED_AT.0 + 15 * DAY));

    let inside = runner
        .run_pass(&request, &plan, Timestamp(REQUESTED_AT.0 + 15 * DAY))
        .unwrap();
    assert!(matches!(inside, CascadeOutcome::Incomplete { .. }));

    let now = Timestamp(REQUESTED_AT.0 + 15 * DAY + 1);
    let breach = runner.run_pass(&request, &plan, now).unwrap_err();
    // The responder handling an Art. 17 breach notification must not have to
    // re-derive who still owes a receipt: the error carries it.
    assert_eq!(
        breach,
        DsrKernelError::SlaBreached {
            tenant: "ten_alpha".to_owned(),
            request: DsrRequestId("dsr_alpha".to_owned()),
            pending: vec!["crm".to_owned(), "legacy-dw".to_owned()],
            deadline: plan.deadline,
            now,
        }
    );
    let rendered = breach.to_string();
    for expected in ["ten_alpha", "dsr_alpha", "crm", "legacy-dw"] {
        assert!(
            rendered.contains(expected),
            "missing {expected}: {rendered}"
        );
    }
}

#[test]
fn the_dpo_is_alerted_once_eighty_percent_of_the_window_is_spent() {
    let mut registry = InMemoryRegistry::new();
    registry.register("billing", Box::new(RecordingHandler::succeeding("billing")));
    registry.register_without_handler("legacy-dw");
    let runner = CascadeRunner::new(InMemoryDsrRepository::new(), registry);
    let request = request(RegulatoryPack::Eu);
    let plan = runner.submit(&request).unwrap();

    let CascadeOutcome::Incomplete { sla_at_risk, .. } = runner
        .run_pass(&request, &plan, Timestamp(REQUESTED_AT.0 + 23 * DAY))
        .unwrap()
    else {
        panic!("incomplete");
    };
    assert!(!sla_at_risk, "23/30 days is under the 80% alert line");

    let CascadeOutcome::Incomplete { sla_at_risk, .. } = runner
        .run_pass(&request, &plan, Timestamp(REQUESTED_AT.0 + 24 * DAY))
        .unwrap()
    else {
        panic!("incomplete");
    };
    assert!(sla_at_risk, "24/30 days is the 80% alert line");
}

#[test]
fn a_missing_receipt_can_only_be_waived_by_dual_control() {
    let mut registry = InMemoryRegistry::new();
    registry.register("billing", Box::new(RecordingHandler::succeeding("billing")));
    registry.register_without_handler("legacy-dw");
    let runner = CascadeRunner::new(InMemoryDsrRepository::new(), registry);
    let request = request(RegulatoryPack::Eu);
    let plan = runner.submit(&request).unwrap();
    let now = Timestamp(REQUESTED_AT.0 + DAY);
    runner.run_pass(&request, &plan, now).unwrap();

    // Same person twice is not dual control.
    assert_eq!(
        runner
            .seal_with_dpo_override(
                &request,
                &plan,
                now,
                DpoOverride {
                    first_approver: "dpo-a".to_owned(),
                    second_approver: "dpo-a".to_owned(),
                    reason: "legacy-dw decommissioned 2026-01".to_owned(),
                },
            )
            .unwrap_err(),
        DsrKernelError::InvalidDpoOverride
    );

    let proof = runner
        .seal_with_dpo_override(&request, &plan, now, waiver())
        .unwrap();
    assert_eq!(proof.receipts.len(), 1);
    assert_eq!(proof.expected_microservices, 2);
    assert_eq!(proof.covered_microservices, vec!["billing", "legacy-dw"]);
    assert!(proof.dpo_override.is_some());
    verify_proof_of_erasure(&proof).unwrap();
}

#[test]
fn a_waiver_stripped_from_a_short_proof_fails_verification() {
    let mut registry = InMemoryRegistry::new();
    registry.register("billing", Box::new(RecordingHandler::succeeding("billing")));
    registry.register_without_handler("legacy-dw");
    let runner = CascadeRunner::new(InMemoryDsrRepository::new(), registry);
    let request = request(RegulatoryPack::Eu);
    let plan = runner.submit(&request).unwrap();
    let now = Timestamp(REQUESTED_AT.0 + DAY);
    runner.run_pass(&request, &plan, now).unwrap();

    let mut proof = runner
        .seal_with_dpo_override(&request, &plan, now, waiver())
        .unwrap();
    proof.dpo_override = None;
    assert_eq!(
        verify_proof_of_erasure(&proof).unwrap_err(),
        DsrKernelError::DpoOverrideRequired
    );
}

#[test]
fn a_forged_receipt_appended_to_a_sealed_proof_is_detected() {
    let runner = CascadeRunner::new(
        InMemoryDsrRepository::new(),
        registry_with(&["billing", "mail"]),
    );
    let request = request(RegulatoryPack::Eu);
    let plan = runner.submit(&request).unwrap();
    let CascadeOutcome::Sealed { mut proof, .. } = runner
        .run_pass(&request, &plan, Timestamp(REQUESTED_AT.0 + DAY))
        .unwrap()
    else {
        panic!("cascade should seal");
    };

    proof.receipts.push(ErasureReceipt {
        tenant: request.tenant_id.clone(),
        request: plan.request.clone(),
        microservice: "ghost".to_owned(),
        merkle_leaf: [0x42_u8; 32],
    });
    assert_eq!(
        verify_proof_of_erasure(&proof).unwrap_err(),
        DsrKernelError::RootMismatch
    );
}

#[test]
fn a_receipt_moved_to_another_tenants_certificate_is_detected() {
    let runner = CascadeRunner::new(InMemoryDsrRepository::new(), registry_with(&["billing"]));
    let request = request(RegulatoryPack::Eu);
    let plan = runner.submit(&request).unwrap();
    let CascadeOutcome::Sealed { mut proof, .. } = runner
        .run_pass(&request, &plan, Timestamp(REQUESTED_AT.0 + DAY))
        .unwrap()
    else {
        panic!("cascade should seal");
    };

    // Re-badge the whole certificate for another tenant: the leaves bind the
    // tenant, so the receipts no longer belong to the key they are under.
    proof.tenant = "ten_beta".to_owned();
    assert_eq!(
        verify_proof_of_erasure(&proof).unwrap_err(),
        DsrKernelError::ForeignReceipt
    );
}

#[test]
fn a_handler_failure_is_typed_bounded_and_appends_no_receipt() {
    // The port returns a typed failure rather than unwinding, so a broken
    // microservice degrades the cascade instead of poisoning the runner —
    // and its text, which is wholly third-party controlled, is bounded
    // before it enters cascade state.
    struct Refusing(String);
    impl ErasureHandler for Refusing {
        fn erase(&self, _request: &DsrRequest) -> Result<[u8; 32], HandlerFailure> {
            Err(HandlerFailure {
                detail: self.0.clone(),
            })
        }
    }
    let mut registry = InMemoryRegistry::new();
    registry.register(
        "billing",
        Box::new(Refusing("tombstone write rejected".to_owned())),
    );
    registry.register("mail", Box::new(Refusing("z".repeat(4096))));
    let runner = CascadeRunner::new(InMemoryDsrRepository::new(), registry);
    let request = request(RegulatoryPack::Eu);
    let plan = runner.submit(&request).unwrap();

    assert_eq!(
        runner.run_step(&request, "billing").unwrap(),
        StepStatus::Failed {
            detail: "tombstone write rejected".to_owned()
        }
    );
    let StepStatus::Failed { detail } = runner.run_step(&request, "mail").unwrap() else {
        panic!("mail refuses");
    };
    assert!(
        detail.len() <= MAX_HANDLER_DETAIL_BYTES + "…[truncated]".len(),
        "unbounded third-party text must not ride the error channel: {} bytes",
        detail.len()
    );
    assert_eq!(runner.repository().receipt_count(&plan.key()).unwrap(), 0);
}

#[test]
fn running_a_pass_for_a_request_that_was_never_opened_is_refused() {
    let runner = CascadeRunner::new(InMemoryDsrRepository::new(), registry_with(&["billing"]));
    let request = request(RegulatoryPack::Eu);
    let plan = runner.plan(&request).unwrap(); // planned, never submitted
    assert_eq!(
        runner
            .run_pass(&request, &plan, Timestamp(REQUESTED_AT.0 + DAY))
            .unwrap_err(),
        DsrKernelError::UnknownRequest
    );
}
