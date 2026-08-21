//! DSR cascade runner and proof of erasure — IP-009
//! (`tenancy/IP-009-dsr-cascade-runner.md`).
//!
//! A data-subject erasure request (GDPR Art. 17, LGPD Art. 18, DPDPA §12,
//! CCPA §1798.105) does not end at one datastore: every microservice
//! holding tenant data must erase and prove it. This crate plans that
//! fan-out deterministically, executes it idempotently, aggregates the
//! per-microservice receipts into a Merkle root, and seals a
//! [`ProofOfErasure`] a regulator can re-verify offline.
//!
//! # Module tree
//!
//! IP-009 specifies seven crates
//! (`...-{kernel,domain,usecase,adapter,rest,worker,app}`); the capability
//! is capped at 12 crates, so they are collapsed here into one module tree:
//!
//! - [`kernel`] — entities, value objects, ports, error taxonomy.
//! - [`digest`] — SHA-256, pinned to the NIST vectors.
//! - [`domain`] — Merkle encoding/aggregation/verification, per-pack SLA.
//! - [`usecase`] — the cascade runner (plan / pass / step / DPO seal).
//! - [`inmemory`] — in-memory repository, registry and handlers.
//!
//! Everything in [`kernel`] is re-exported at the crate root, so the paths
//! the Wave-15 scaffold published (`tenancy_dsr_cascade::DsrRequest`, …)
//! are unchanged.
//!
//! # What is enforced here
//!
//! - **Tenant scoping.** A [`DsrRequestId`] is caller-supplied and
//!   tenant-local, so per-tenant numbering collides across tenants by
//!   accident. Requests are addressed by [`DsrRequestKey`] (tenant + id)
//!   everywhere, the tenant is bound into every Merkle leaf, and the
//!   repository keys its records on the pair. One certificate can never
//!   discharge two tenants' erasure obligations.
//! - **The plan is bound to its request.** [`usecase::CascadeRunner::run_pass`]
//!   and [`usecase::CascadeRunner::seal_with_dpo_override`] refuse a plan
//!   that was not derived for the request they were handed, before any
//!   handler runs: erasing one subject and sealing over another's receipts
//!   would produce a certificate asserting an erasure that never happened.
//! - **Determinism.** The plan is deduplicated and sorted; leaves are
//!   ordered by microservice name, not by arrival, so the root does not
//!   depend on fan-out timing. No clock is read anywhere in `domain` or
//!   `usecase` — `now` is a parameter on every SLA-bearing call.
//! - **Idempotent retry.** A step whose receipt already exists does not
//!   re-invoke its handler, and the repository refuses a second receipt for
//!   the same (tenant, request, microservice) triple with
//!   [`DsrKernelError::DuplicateReceipt`].
//! - **Partial cascades are representable.** A broken microservice yields a
//!   failed or deferred step, never an aborted pass: the services after it
//!   in plan order are still owed their erasure.
//! - **Coverage is a set, not a count.** A certificate names the plan it
//!   covers. Receipts from a microservice decommissioned mid-window are
//!   surplus evidence, kept in the tree and never a reason to refuse — a
//!   count comparison would leave a fully erased subject with no obtainable
//!   certificate by any path, waiver included.
//! - **No silently short proof.** Sealing without a receipt from a covered
//!   microservice requires a validated two-person [`DpoOverride`], recorded
//!   inside the certificate.
//! - **Re-verifiability.** [`domain::verify_proof_of_erasure`] recomputes
//!   the root from the certificate's own receipts, so a removed, added or
//!   altered receipt is detectable after the fact.
//! - **Bounded third-party text.** [`HandlerFailure::detail`] is the only
//!   wholly downstream-controlled field; it is bounded at
//!   [`MAX_HANDLER_DETAIL_BYTES`] on every path that reads it.
//!
//! # Gaps (deliberately deferred)
//!
//! This capability's `Cargo.lock` is frozen and owned by another lane, so
//! this crate has ZERO dependencies. The consequences, stated plainly:
//!
//! - **Hand-rolled SHA-256.** [`digest`] is ~200 lines of plain `std`
//!   Rust, pinned to the published FIPS 180-4 / NIST CAVP vectors (empty,
//!   `"abc"`, the 448-bit and 896-bit multi-block vectors, and one million
//!   `'a'`). Passing known-answer vectors is correctness evidence only —
//!   it says nothing about side-channel resistance or review pedigree.
//!   It is NOT audited and SHOULD be replaced by a vetted crate (`sha2`,
//!   or `blake3` as IP-009 originally specified, with a matching change to
//!   the documented leaf encoding) the moment the lockfile opens.
//! - **No signature, so the certificate ENVELOPE is unauthenticated.** The
//!   Merkle root binds the receipts and nothing else: a certificate whose
//!   coverage list, waiver and count were rewritten together still verifies.
//!   [`domain::verify_proof_of_erasure`] detects tampering WITH THE
//!   RECEIPTS, not a wholesale forgery of the envelope. An adapter must sign
//!   [`ProofOfErasure`] before it leaves the trust boundary; until it does,
//!   the root is an integrity digest, not an authenticity proof.
//! - **The plan is trusted input.** The runner checks that a plan belongs to
//!   its request, but a caller that derives a plan from a registry missing a
//!   microservice gets a certificate covering only what it named. Whether
//!   the registry is complete is the registry's guarantee, not this crate's.
//! - **Synchronous ports.** IP-009's ports are `async` over Workflow,
//!   Postgres and an alerter. These are sync traits: the cascade runs one
//!   pass per call and the caller owns scheduling and retry. No
//!   `tokio::time::sleep` loop lives here, and that is on purpose — the
//!   loop is the adapter's business, the decision is this crate's.
//! - **No REST / worker / event surface.** `POST /dsr-requests`, the
//!   `TenantDeletionRequested` fan-out event, the SLA timer worker and the
//!   DPO alerter are not implemented; [`usecase::CascadeOutcome`] carries
//!   the `sla_at_risk` flag those surfaces would act on, and
//!   [`DsrKernelError::SlaBreached`] carries the breach diagnosis they would
//!   escalate.
//! - **In-memory persistence only.** [`inmemory`] is the only adapter; the
//!   Postgres repository IP-009 names would add a dependency.
//! - **Timestamps are `i64` epoch seconds**, not `chrono::DateTime<Utc>`,
//!   and the per-pack windows are fixed day counts, so no calendar or
//!   business-day rule (a real DPDPA/LGPD implementation needs one).
//! - **No receipt deletion.** No port removes a receipt, so a receipt
//!   written in error can only be superseded by a new request, never
//!   retracted. That is deliberate for evidence, and it is why surplus
//!   receipts had to become legal rather than fatal.
//!
//! ADR-0083 Tier 3: production code here carries no unwrap/expect/panic;
//! tests use them to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod digest;
pub mod domain;
pub mod inmemory;
pub mod kernel;
pub mod usecase;

pub use kernel::{
    DpoOverride, DsrKernelError, DsrKind, DsrRequest, DsrRequestId, DsrRequestKey,
    DsrRequestRepository, ErasureHandler, ErasureReceipt, HandlerFailure, MAX_HANDLER_DETAIL_BYTES,
    MicroserviceRegistry, ProofOfErasure, RegulatoryPack, Timestamp,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digest::to_hex;
    use crate::domain::{
        canonical_receipt_bytes, leaf_hash, merkle_root, node_hash, receipts_merkle_root,
        sla_at_risk, sla_breached, sla_deadline, verify_proof_of_erasure,
    };
    use crate::inmemory::InMemoryDsrRepository;

    const DAY: i64 = 86_400;
    const TENANT: &str = "ten_alpha";

    fn request() -> DsrRequest {
        DsrRequest {
            id: DsrRequestId("dsr_001".to_owned()),
            tenant_id: TENANT.to_owned(),
            subject_id: "subject-7".to_owned(),
            kind: DsrKind::Erasure,
            pack: RegulatoryPack::Eu,
            requested_at: Timestamp(1_000_000),
        }
    }

    fn key() -> DsrRequestKey {
        request().key()
    }

    fn receipt(microservice: &str, byte: u8) -> ErasureReceipt {
        ErasureReceipt {
            tenant: TENANT.to_owned(),
            request: DsrRequestId("dsr_001".to_owned()),
            microservice: microservice.to_owned(),
            merkle_leaf: [byte; 32],
        }
    }

    fn plan_of(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn merkle_root_is_independent_of_receipt_order() {
        let forward = [receipt("mail", 1), receipt("billing", 2), receipt("crm", 3)];
        let reversed = [receipt("crm", 3), receipt("mail", 1), receipt("billing", 2)];
        assert_eq!(
            receipts_merkle_root(&key(), &forward).unwrap(),
            receipts_merkle_root(&key(), &reversed).unwrap()
        );
    }

    #[test]
    fn altering_one_receipt_changes_the_root() {
        let base = [receipt("mail", 1), receipt("billing", 2)];
        let tampered = [receipt("mail", 1), receipt("billing", 0x99)];
        assert_ne!(
            receipts_merkle_root(&key(), &base).unwrap(),
            receipts_merkle_root(&key(), &tampered).unwrap()
        );
    }

    #[test]
    fn moving_a_receipt_to_another_microservice_changes_the_root() {
        // The leaf binds the microservice name, so the same evidence digest
        // reported by a different service is a different leaf.
        let base = [receipt("mail", 1), receipt("billing", 2)];
        let swapped = [receipt("mail", 2), receipt("billing", 1)];
        assert_ne!(
            receipts_merkle_root(&key(), &base).unwrap(),
            receipts_merkle_root(&key(), &swapped).unwrap()
        );
    }

    #[test]
    fn the_same_receipt_under_another_tenant_is_a_different_leaf() {
        // Without tenant binding, tenant beta could present tenant alpha's
        // receipt as evidence for its own identically-numbered request.
        let mine = receipt("mail", 1);
        let mut theirs = mine.clone();
        theirs.tenant = "ten_beta".to_owned();
        assert_ne!(leaf_hash(&mine).unwrap(), leaf_hash(&theirs).unwrap());
    }

    #[test]
    fn the_leaf_encoding_is_pinned_to_a_stated_digest() {
        // A third party reimplementing the verifier from the domain module
        // header must be able to check their implementation against a value
        // this crate states, not merely against itself. The preimage is
        //   0x00 || be64(9)"ten_alpha" || be64(7)"dsr_001"
        //        || be64(4)"mail"      || be64(32)(0x01 * 32)
        let subject = receipt("mail", 1);
        let bytes = canonical_receipt_bytes(&subject).unwrap();
        assert_eq!(bytes.len(), 85, "1 tag + 4 length-prefixed fields");
        assert_eq!(bytes.first(), Some(&0x00), "the leaf tag is applied once");
        assert_eq!(
            to_hex(&leaf_hash(&subject).unwrap()),
            "5fba3fd82df08ab4fd14c9defda54365acdb8a027872771a03eadadb4531a779"
        );
    }

    #[test]
    fn leaf_and_node_hashes_are_domain_separated() {
        let leaf = leaf_hash(&receipt("mail", 1)).unwrap();
        let node = node_hash(&leaf, &leaf);
        assert_ne!(
            leaf, node,
            "a node hash must never collide with a leaf hash"
        );
    }

    #[test]
    fn odd_leaf_count_promotes_instead_of_duplicating() {
        // The Bitcoin CVE-2012-2459 shape: [a,b,c] must NOT equal [a,b,c,c].
        let a = [0x0a_u8; 32];
        let b = [0x0b_u8; 32];
        let c = [0x0c_u8; 32];
        let three = merkle_root(&[a, b, c]).unwrap();
        let four = merkle_root(&[a, b, c, c]).unwrap();
        assert_ne!(three, four);
        // And promotion is exactly node(node(a,b), c).
        assert_eq!(three, node_hash(&node_hash(&a, &b), &c));
    }

    #[test]
    fn single_leaf_root_is_that_leaf() {
        let a = [0x0a_u8; 32];
        assert_eq!(merkle_root(&[a]).unwrap(), a);
    }

    #[test]
    fn empty_receipt_set_cannot_be_aggregated() {
        assert_eq!(
            merkle_root(&[]).unwrap_err(),
            DsrKernelError::MerkleAggregationFailed
        );
        assert_eq!(
            receipts_merkle_root(&key(), &[]).unwrap_err(),
            DsrKernelError::EmptyReceiptSet
        );
    }

    #[test]
    fn duplicate_microservice_in_a_receipt_set_is_refused() {
        let doubled = [receipt("mail", 1), receipt("mail", 2)];
        assert_eq!(
            receipts_merkle_root(&key(), &doubled).unwrap_err(),
            DsrKernelError::DuplicateMicroserviceReceipt
        );
    }

    #[test]
    fn a_receipt_from_another_request_is_refused() {
        let mut foreign = receipt("mail", 7);
        foreign.request = DsrRequestId("dsr_999".to_owned());
        assert_eq!(
            receipts_merkle_root(&key(), &[receipt("billing", 1), foreign]).unwrap_err(),
            DsrKernelError::ForeignReceipt
        );
    }

    #[test]
    fn a_receipt_from_another_tenant_is_refused() {
        let mut foreign = receipt("mail", 7);
        foreign.tenant = "ten_beta".to_owned();
        assert_eq!(
            receipts_merkle_root(&key(), &[receipt("billing", 1), foreign]).unwrap_err(),
            DsrKernelError::ForeignReceipt
        );
    }

    #[test]
    fn per_pack_sla_windows_match_the_statutes() {
        let requested_at = Timestamp(0);
        for (pack, days) in [
            (RegulatoryPack::Eu, 30),
            (RegulatoryPack::Kr, 30),
            (RegulatoryPack::In, 30),
            (RegulatoryPack::Br, 15),
            (RegulatoryPack::UsHc, 7),
            (RegulatoryPack::Default, 30),
        ] {
            assert_eq!(
                sla_deadline(pack, requested_at).unwrap(),
                Timestamp(days * DAY),
                "wrong window for {pack:?}"
            );
        }
    }

    #[test]
    fn sla_deadline_overflow_is_an_error_not_a_wrap() {
        assert_eq!(
            sla_deadline(RegulatoryPack::Eu, Timestamp(i64::MAX)).unwrap_err(),
            DsrKernelError::TimestampOverflow
        );
    }

    #[test]
    fn sla_at_risk_trips_at_eighty_percent_of_the_window() {
        let requested_at = Timestamp(0);
        let deadline = Timestamp(30 * DAY);
        assert!(!sla_at_risk(requested_at, deadline, Timestamp(23 * DAY)).unwrap());
        assert!(sla_at_risk(requested_at, deadline, Timestamp(24 * DAY)).unwrap());
        assert!(sla_at_risk(requested_at, deadline, Timestamp(31 * DAY)).unwrap());
        assert!(!sla_at_risk(requested_at, deadline, Timestamp(0)).unwrap());
    }

    #[test]
    fn sla_breach_is_strictly_after_the_deadline() {
        let deadline = Timestamp(30 * DAY);
        assert!(!sla_breached(deadline, Timestamp(30 * DAY)));
        assert!(sla_breached(deadline, Timestamp(30 * DAY + 1)));
    }

    #[test]
    fn verify_detects_a_removed_receipt() {
        let receipts = vec![receipt("billing", 1), receipt("crm", 2), receipt("mail", 3)];
        let mut proof = domain::compute_proof_of_erasure(
            &key(),
            &receipts,
            &plan_of(&["billing", "crm", "mail"]),
            Timestamp(10),
            None,
        )
        .unwrap();
        verify_proof_of_erasure(&proof).unwrap();

        proof.receipts.pop();
        assert_eq!(
            verify_proof_of_erasure(&proof).unwrap_err(),
            DsrKernelError::RootMismatch
        );
    }

    #[test]
    fn verify_detects_an_altered_receipt() {
        let receipts = vec![receipt("billing", 1), receipt("mail", 3)];
        let mut proof = domain::compute_proof_of_erasure(
            &key(),
            &receipts,
            &plan_of(&["billing", "mail"]),
            Timestamp(10),
            None,
        )
        .unwrap();
        if let Some(first) = proof.receipts.first_mut() {
            first.merkle_leaf = [0xff_u8; 32];
        }
        assert_eq!(
            verify_proof_of_erasure(&proof).unwrap_err(),
            DsrKernelError::RootMismatch
        );
    }

    #[test]
    fn sealing_short_of_the_plan_needs_a_dpo_override() {
        let receipts = vec![receipt("billing", 1)];
        assert_eq!(
            domain::compute_proof_of_erasure(
                &key(),
                &receipts,
                &plan_of(&["billing", "crm", "mail"]),
                Timestamp(10),
                None,
            )
            .unwrap_err(),
            DsrKernelError::DpoOverrideRequired
        );
    }

    #[test]
    fn a_dpo_override_must_be_two_distinct_named_approvers() {
        for waiver in [
            DpoOverride {
                first_approver: "dpo-a".to_owned(),
                second_approver: "dpo-a".to_owned(),
                reason: "service decommissioned".to_owned(),
            },
            DpoOverride {
                first_approver: "  ".to_owned(),
                second_approver: "dpo-b".to_owned(),
                reason: "service decommissioned".to_owned(),
            },
            DpoOverride {
                first_approver: "dpo-a".to_owned(),
                second_approver: "dpo-b".to_owned(),
                reason: String::new(),
            },
        ] {
            assert_eq!(
                waiver.validate().unwrap_err(),
                DsrKernelError::InvalidDpoOverride
            );
        }
    }

    #[test]
    fn surplus_receipts_from_a_decommissioned_service_still_seal() {
        // The registry shrank inside the statutory window: `crm` reported and
        // was then decommissioned, so the current plan no longer names it.
        // Its receipt is evidence, not an obstruction — refusing here would
        // leave a fully erased subject with no obtainable certificate.
        let receipts = vec![receipt("billing", 1), receipt("crm", 2), receipt("mail", 3)];
        let proof = domain::compute_proof_of_erasure(
            &key(),
            &receipts,
            &plan_of(&["billing", "mail"]),
            Timestamp(10),
            None,
        )
        .unwrap();
        assert_eq!(proof.receipts.len(), 3, "surplus evidence is kept");
        assert_eq!(proof.covered_microservices, plan_of(&["billing", "mail"]));
        assert_eq!(proof.expected_microservices, 2);
        assert!(proof.dpo_override.is_none(), "no waiver is needed");
        verify_proof_of_erasure(&proof).unwrap();
    }

    #[test]
    fn a_certificate_whose_stated_count_was_edited_is_refused() {
        let receipts = vec![receipt("billing", 1), receipt("mail", 3)];
        let mut proof = domain::compute_proof_of_erasure(
            &key(),
            &receipts,
            &plan_of(&["billing", "mail"]),
            Timestamp(10),
            None,
        )
        .unwrap();
        proof.expected_microservices = 5;
        assert_eq!(
            verify_proof_of_erasure(&proof).unwrap_err(),
            DsrKernelError::InconsistentProof
        );
    }

    #[test]
    fn a_certificate_with_a_non_canonical_coverage_list_is_refused() {
        let receipts = vec![receipt("billing", 1), receipt("mail", 3)];
        let mut proof = domain::compute_proof_of_erasure(
            &key(),
            &receipts,
            &plan_of(&["billing", "mail"]),
            Timestamp(10),
            None,
        )
        .unwrap();
        proof.covered_microservices = plan_of(&["mail", "billing"]);
        assert_eq!(
            verify_proof_of_erasure(&proof).unwrap_err(),
            DsrKernelError::InconsistentProof
        );
    }

    #[test]
    fn repository_rejects_a_duplicate_receipt() {
        let repository = InMemoryDsrRepository::new();
        let request = request();
        repository.open(&request).unwrap();
        repository.append_receipt(&receipt("mail", 1)).unwrap();
        assert_eq!(
            repository.append_receipt(&receipt("mail", 9)).unwrap_err(),
            DsrKernelError::DuplicateReceipt
        );
        assert_eq!(repository.receipt_count(&request.key()).unwrap(), 1);
    }

    #[test]
    fn repository_rejects_receipts_for_an_unopened_request() {
        let repository = InMemoryDsrRepository::new();
        assert_eq!(
            repository.append_receipt(&receipt("mail", 1)).unwrap_err(),
            DsrKernelError::UnknownRequest
        );
    }

    #[test]
    fn reopening_a_request_keeps_its_receipts() {
        let repository = InMemoryDsrRepository::new();
        let request = request();
        repository.open(&request).unwrap();
        repository.append_receipt(&receipt("mail", 1)).unwrap();
        repository.open(&request).unwrap();
        assert_eq!(repository.receipt_count(&request.key()).unwrap(), 1);
    }

    #[test]
    fn two_tenants_using_the_same_request_id_get_separate_records() {
        let repository = InMemoryDsrRepository::new();
        let alpha = request();
        let mut beta = request();
        beta.tenant_id = "ten_beta".to_owned();
        beta.subject_id = "bob".to_owned();

        repository.open(&alpha).unwrap();
        repository.open(&beta).unwrap();
        assert_eq!(repository.open_request_count().unwrap(), 2);

        repository.append_receipt(&receipt("mail", 1)).unwrap();
        // Beta's store is untouched by alpha's receipt: its own append for
        // the same microservice is NOT a duplicate.
        let mut beta_receipt = receipt("mail", 2);
        beta_receipt.tenant = "ten_beta".to_owned();
        repository.append_receipt(&beta_receipt).unwrap();

        assert_eq!(repository.receipt_count(&alpha.key()).unwrap(), 1);
        assert_eq!(repository.receipt_count(&beta.key()).unwrap(), 1);
        assert_eq!(
            repository.receipts(&alpha.key()).unwrap()[0].merkle_leaf,
            [1_u8; 32]
        );
        assert_eq!(
            repository.receipts(&beta.key()).unwrap()[0].merkle_leaf,
            [2_u8; 32]
        );
    }

    #[test]
    fn non_erasure_kinds_are_refused_before_anything_is_opened() {
        let mut request = request();
        request.kind = DsrKind::Access;
        assert_eq!(
            request.validate_for_erasure().unwrap_err(),
            DsrKernelError::UnsupportedKind
        );
        let repository = InMemoryDsrRepository::new();
        assert_eq!(
            repository.open(&request).unwrap_err(),
            DsrKernelError::UnsupportedKind
        );
    }

    #[test]
    fn blank_identity_fields_are_refused() {
        for mutate in [
            |request: &mut DsrRequest| request.id = DsrRequestId("  ".to_owned()),
            |request: &mut DsrRequest| request.tenant_id = String::new(),
            |request: &mut DsrRequest| request.subject_id = String::new(),
        ] {
            let mut request = request();
            mutate(&mut request);
            assert_eq!(
                request.validate_for_erasure().unwrap_err(),
                DsrKernelError::InvalidRequest
            );
        }
    }

    #[test]
    fn a_breached_sla_error_names_the_request_and_what_is_still_owed() {
        let breach = DsrKernelError::SlaBreached {
            tenant: TENANT.to_owned(),
            request: DsrRequestId("dsr_001".to_owned()),
            pending: plan_of(&["crm", "legacy-dw"]),
            deadline: Timestamp(30 * DAY),
            now: Timestamp(31 * DAY),
        };
        let rendered = breach.to_string();
        for expected in ["ten_alpha", "dsr_001", "crm", "legacy-dw", "2592000s"] {
            assert!(
                rendered.contains(expected),
                "breach diagnosis must name {expected}: {rendered}"
            );
        }
    }

    #[test]
    fn handler_detail_is_bounded_on_the_way_in_and_out() {
        let long = "x".repeat(MAX_HANDLER_DETAIL_BYTES * 4);
        let bounded = HandlerFailure::new(&long);
        assert!(bounded.detail.len() < long.len());
        assert!(bounded.detail.ends_with("[truncated]"));

        // A handler bypassing the constructor is still bounded on read.
        let raw = HandlerFailure { detail: long };
        assert!(raw.bounded_detail().len() < raw.detail.len());
        assert!(raw.to_string().len() < raw.detail.len());
    }

    #[test]
    fn bounding_a_detail_never_splits_a_utf8_character() {
        // A multi-byte character straddling the byte limit must not be cut
        // in half: a panicking or lossy truncation in the ERROR path is how
        // a diagnostic becomes a second incident.
        for padding in 0..8 {
            let detail = format!("{}{}", "a".repeat(padding), "é".repeat(512));
            let bounded = HandlerFailure::new(&detail).detail;
            assert!(bounded.ends_with("[truncated]"));
            assert!(detail.starts_with(bounded.trim_end_matches("…[truncated]")));
        }
    }

    #[test]
    fn request_level_errors_are_distinguished_from_step_level_ones() {
        for terminal in [
            DsrKernelError::UnknownRequest,
            DsrKernelError::AlreadyFinalized,
            DsrKernelError::InvalidRequest,
            DsrKernelError::UnsupportedKind,
            DsrKernelError::PlanRequestMismatch,
        ] {
            assert!(terminal.is_request_terminal(), "{terminal} ends the pass");
        }
        for step_level in [
            DsrKernelError::RepositoryUnavailable,
            DsrKernelError::DuplicateReceipt,
        ] {
            assert!(
                !step_level.is_request_terminal(),
                "{step_level} must not starve the rest of the plan"
            );
        }
    }

    #[test]
    fn kernel_errors_render_distinct_messages() {
        let rendered = [
            DsrKernelError::UnknownRequest,
            DsrKernelError::DuplicateReceipt,
            DsrKernelError::MerkleAggregationFailed,
            DsrKernelError::EmptyReceiptSet,
            DsrKernelError::DuplicateMicroserviceReceipt,
            DsrKernelError::ForeignReceipt,
            DsrKernelError::ReceiptEncodingTooLarge,
            DsrKernelError::RootMismatch,
            DsrKernelError::InconsistentProof,
            DsrKernelError::PlanRequestMismatch,
        ]
        .map(|error| error.to_string());
        for (index, message) in rendered.iter().enumerate() {
            assert!(!message.is_empty());
            assert!(
                !rendered
                    .iter()
                    .enumerate()
                    .any(|(other, text)| other != index && text == message),
                "message {index} is not distinct: {message}"
            );
        }
    }
}
