//! Ontology query execution usecase foundation.
//!
//! This crate is the source-level orchestration seam between future REST/gRPC
//! adapters and the ontology query domain. It accepts a precomputed policy
//! decision, enforces idempotent execution semantics, emits metadata-only audit
//! events, and never carries raw property values or provider/runtime credentials.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

use data_ontology_kernel::ObjectGraph;
use data_ontology_query_engine_domain::{
    KnowledgeGraphQueryEngine, KnowledgeGraphQueryError, KnowledgeGraphQueryRequest,
    KnowledgeGraphQueryResponse, TraversalDirection,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OntologyQueryPolicyDecision {
    pub decision_id: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub principal_id: String,                // data_class: INTERNAL_ONLY
    pub allowed_query_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
    pub max_depth_ceiling: u32,              // data_class: INTERNAL_ONLY
    pub evidence_ref: String,                // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OntologyQueryExecutionInput {
    pub idempotency_key: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,                // data_class: INTERNAL_ONLY
    pub query_surface: String,               // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,        // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,           // data_class: INTERNAL_ONLY
    pub request: KnowledgeGraphQueryRequest, // data_class: INTERNAL_ONLY
    pub policy_decision: OntologyQueryPolicyDecision, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OntologyQueryExecutionStatus {
    Completed,
    Denied,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OntologyQueryDenialKind {
    InvalidInput,
    IdempotencyConflict,
    TenantMismatch,
    PrincipalMismatch,
    SurfaceDenied,
    DepthCeilingExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OntologyQueryExecutionReceipt {
    pub idempotency_key: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub principal_id: String,                 // data_class: INTERNAL_ONLY
    pub status: OntologyQueryExecutionStatus, // data_class: PUBLIC
    pub denial_kind: Option<OntologyQueryDenialKind>, // data_class: INTERNAL_ONLY
    pub failure: Option<KnowledgeGraphQueryError>, // data_class: INTERNAL_ONLY
    pub response: Option<KnowledgeGraphQueryResponse>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OntologyQueryAuditEventKind {
    QueryRequestReceived,
    QueryDenied,
    QueryFailed,
    QueryCompleted,
    IdempotencyConflict,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OntologyQueryAuditEvent {
    pub kind: OntologyQueryAuditEventKind, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub principal_id: String,              // data_class: INTERNAL_ONLY
    pub idempotency_key: String,           // data_class: INTERNAL_ONLY
    pub query_id: String,                  // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OntologyQueryExecutionUsecase {
    receipts_by_idempotency_key: BTreeMap<String, OntologyQueryExecutionReceipt>,
    fingerprints_by_idempotency_key: BTreeMap<String, OntologyQueryIntentFingerprint>,
    audit_events: Vec<OntologyQueryAuditEvent>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OntologyQueryIntentFingerprint {
    tenant_id: String,
    principal_id: String,
    query_surface: String,
    root_entity_id: String,
    edge_type_ids: Vec<String>,
    max_depth: u32,
    freshness_floor_epoch_seconds: u64,
    policy_decision_id: String,
}

impl OntologyQueryExecutionUsecase {
    pub fn execute(
        &mut self,
        graph: &ObjectGraph,
        engine: &KnowledgeGraphQueryEngine,
        input: OntologyQueryExecutionInput,
    ) -> OntologyQueryExecutionReceipt {
        if input.idempotency_key.trim().is_empty() {
            return invalid_input_receipt(input, Vec::new());
        }

        let fingerprint = OntologyQueryIntentFingerprint::from(&input);
        if let Some(existing) = self.receipts_by_idempotency_key.get(&input.idempotency_key) {
            if self
                .fingerprints_by_idempotency_key
                .get(&input.idempotency_key)
                == Some(&fingerprint)
            {
                return existing.clone();
            }
            return self.conflict_receipt(input);
        }

        let evidence_refs = canonical_evidence_refs(&input);
        if input.principal_id.trim().is_empty()
            || input.query_surface.trim().is_empty()
            || input.request_evidence_ref.trim().is_empty()
            || input.trace_context_ref.trim().is_empty()
            || input.policy_decision.decision_id.trim().is_empty()
            || input.policy_decision.evidence_ref.trim().is_empty()
        {
            return invalid_input_receipt(input, evidence_refs);
        }

        self.record_event(
            OntologyQueryAuditEventKind::QueryRequestReceived,
            &input,
            evidence_refs.clone(),
        );

        if let Some(receipt) = self.policy_denial(&input, &evidence_refs) {
            self.cache_receipt(&input.idempotency_key, fingerprint, &receipt);
            return receipt;
        }

        match engine.query_graph_slice(graph, input.request.clone()) {
            Ok(response) => {
                let receipt = OntologyQueryExecutionReceipt {
                    idempotency_key: input.idempotency_key.clone(),
                    tenant_id: input.request.tenant_id.clone(),
                    principal_id: input.principal_id.clone(),
                    status: OntologyQueryExecutionStatus::Completed,
                    denial_kind: None,
                    failure: None,
                    response: Some(response),
                    evidence_refs: evidence_refs.clone(),
                };
                self.record_event(
                    OntologyQueryAuditEventKind::QueryCompleted,
                    &input,
                    evidence_refs,
                );
                self.cache_receipt(&input.idempotency_key, fingerprint, &receipt);
                receipt
            }
            Err(error) => {
                let receipt = OntologyQueryExecutionReceipt {
                    idempotency_key: input.idempotency_key.clone(),
                    tenant_id: input.request.tenant_id.clone(),
                    principal_id: input.principal_id.clone(),
                    status: OntologyQueryExecutionStatus::Failed,
                    denial_kind: None,
                    failure: Some(error),
                    response: None,
                    evidence_refs: evidence_refs.clone(),
                };
                self.record_event(
                    OntologyQueryAuditEventKind::QueryFailed,
                    &input,
                    evidence_refs,
                );
                self.cache_receipt(&input.idempotency_key, fingerprint, &receipt);
                receipt
            }
        }
    }

    pub fn audit_events(&self) -> &[OntologyQueryAuditEvent] {
        &self.audit_events
    }

    pub fn receipt_count(&self) -> usize {
        self.receipts_by_idempotency_key.len()
    }

    fn policy_denial(
        &mut self,
        input: &OntologyQueryExecutionInput,
        evidence_refs: &[String],
    ) -> Option<OntologyQueryExecutionReceipt> {
        let denial_kind = if input.policy_decision.tenant_id != input.request.tenant_id {
            Some(OntologyQueryDenialKind::TenantMismatch)
        } else if input.policy_decision.principal_id != input.principal_id {
            Some(OntologyQueryDenialKind::PrincipalMismatch)
        } else if !input
            .policy_decision
            .allowed_query_surfaces
            .iter()
            .any(|surface| surface == &input.query_surface)
        {
            Some(OntologyQueryDenialKind::SurfaceDenied)
        } else if input.request.max_depth > input.policy_decision.max_depth_ceiling {
            Some(OntologyQueryDenialKind::DepthCeilingExceeded)
        } else {
            None
        }?;

        let receipt = OntologyQueryExecutionReceipt {
            idempotency_key: input.idempotency_key.clone(),
            tenant_id: input.request.tenant_id.clone(),
            principal_id: input.principal_id.clone(),
            status: OntologyQueryExecutionStatus::Denied,
            denial_kind: Some(denial_kind),
            failure: None,
            response: None,
            evidence_refs: evidence_refs.to_vec(),
        };
        self.record_event(
            OntologyQueryAuditEventKind::QueryDenied,
            input,
            evidence_refs.to_vec(),
        );
        Some(receipt)
    }

    fn conflict_receipt(
        &mut self,
        input: OntologyQueryExecutionInput,
    ) -> OntologyQueryExecutionReceipt {
        let evidence_refs = canonical_evidence_refs(&input);
        let receipt = OntologyQueryExecutionReceipt {
            idempotency_key: input.idempotency_key.clone(),
            tenant_id: input.request.tenant_id.clone(),
            principal_id: input.principal_id.clone(),
            status: OntologyQueryExecutionStatus::Denied,
            denial_kind: Some(OntologyQueryDenialKind::IdempotencyConflict),
            failure: None,
            response: None,
            evidence_refs: evidence_refs.clone(),
        };
        self.record_event(
            OntologyQueryAuditEventKind::IdempotencyConflict,
            &input,
            evidence_refs,
        );
        receipt
    }

    fn record_event(
        &mut self,
        kind: OntologyQueryAuditEventKind,
        input: &OntologyQueryExecutionInput,
        evidence_refs: Vec<String>,
    ) {
        self.audit_events.push(OntologyQueryAuditEvent {
            kind,
            tenant_id: input.request.tenant_id.clone(),
            principal_id: input.principal_id.clone(),
            idempotency_key: input.idempotency_key.clone(),
            query_id: input.request.query_id.clone(),
            evidence_refs,
        });
    }

    fn cache_receipt(
        &mut self,
        idempotency_key: &str,
        fingerprint: OntologyQueryIntentFingerprint,
        receipt: &OntologyQueryExecutionReceipt,
    ) {
        self.fingerprints_by_idempotency_key
            .insert(idempotency_key.to_string(), fingerprint);
        self.receipts_by_idempotency_key
            .insert(idempotency_key.to_string(), receipt.clone());
    }
}

impl OntologyQueryIntentFingerprint {
    fn from(input: &OntologyQueryExecutionInput) -> Self {
        let mut edge_type_ids = input.request.edge_type_ids.clone();
        edge_type_ids.sort();
        edge_type_ids.dedup();
        Self {
            tenant_id: input.request.tenant_id.clone(),
            principal_id: input.principal_id.clone(),
            query_surface: input.query_surface.clone(),
            root_entity_id: input.request.root_entity_id.clone(),
            edge_type_ids,
            max_depth: input.request.max_depth,
            freshness_floor_epoch_seconds: input.request.freshness_floor_epoch_seconds,
            policy_decision_id: input.policy_decision.decision_id.clone(),
        }
    }
}

fn invalid_input_receipt(
    input: OntologyQueryExecutionInput,
    evidence_refs: Vec<String>,
) -> OntologyQueryExecutionReceipt {
    OntologyQueryExecutionReceipt {
        idempotency_key: input.idempotency_key,
        tenant_id: input.request.tenant_id,
        principal_id: input.principal_id,
        status: OntologyQueryExecutionStatus::Denied,
        denial_kind: Some(OntologyQueryDenialKind::InvalidInput),
        failure: None,
        response: None,
        evidence_refs,
    }
}

fn canonical_evidence_refs(input: &OntologyQueryExecutionInput) -> Vec<String> {
    let mut refs = vec![
        input.policy_decision.evidence_ref.clone(),
        input.request_evidence_ref.clone(),
        input.trace_context_ref.clone(),
    ];
    refs.retain(|value| !value.trim().is_empty());
    refs.sort();
    refs.dedup();
    refs
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_ontology_kernel::{ObjectEntity, ObjectProperty, PropertyTier};
    use data_ontology_query_engine_domain::KnowledgeGraphLinkInstance;
    use data_boundary_kernel::{DataClass, PrivacyDataClass};

    fn property(name: &str) -> ObjectProperty {
        ObjectProperty::new(
            name.to_string(),
            "value".to_string(),
            PropertyTier::Scalar,
            PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap(),
        )
    }

    fn graph_and_engine() -> (ObjectGraph, KnowledgeGraphQueryEngine) {
        let mut graph = ObjectGraph::default();
        for (entity_id, entity_type) in [("ent_root", "ety_account"), ("ent_child", "ety_contact")]
        {
            graph
                .upsert_entity(
                    ObjectEntity::new(
                        "ten_alpha".to_string(),
                        entity_id.to_string(),
                        entity_type.to_string(),
                        vec![property("name")],
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        let mut engine = KnowledgeGraphQueryEngine::default();
        engine
            .upsert_link(
                &graph,
                KnowledgeGraphLinkInstance::new(
                    "ten_alpha",
                    "ent_root",
                    "ent_child",
                    "lty_owns",
                    10,
                )
                .unwrap(),
            )
            .unwrap();
        (graph, engine)
    }

    fn input(idempotency_key: &str) -> OntologyQueryExecutionInput {
        OntologyQueryExecutionInput {
            idempotency_key: idempotency_key.to_string(),
            principal_id: "usr_alice".to_string(),
            query_surface: "ontology.query.graph_slice".to_string(),
            request_evidence_ref: "evidence://query/request/kgq_alpha".to_string(),
            trace_context_ref: "trace://kgq_alpha".to_string(),
            request: KnowledgeGraphQueryRequest::new(
                "ten_alpha",
                "kgq_alpha",
                "ent_root",
                vec!["lty_owns"],
                1,
                0,
                12,
                vec!["lty_owns"],
                TraversalDirection::Outbound,
            )
            .unwrap(),
            policy_decision: OntologyQueryPolicyDecision {
                decision_id: "dec_allow_query".to_string(),
                tenant_id: "ten_alpha".to_string(),
                principal_id: "usr_alice".to_string(),
                allowed_query_surfaces: vec!["ontology.query.graph_slice".to_string()],
                max_depth_ceiling: 4,
                evidence_ref: "evidence://policy/dec_allow_query".to_string(),
            },
        }
    }

    #[test]
    fn executes_authorized_query_with_audit_metadata_and_receipt() {
        let (graph, engine) = graph_and_engine();
        let mut usecase = OntologyQueryExecutionUsecase::default();

        let receipt = usecase.execute(&graph, &engine, input("idem-query-001"));

        assert_eq!(receipt.status, OntologyQueryExecutionStatus::Completed);
        assert_eq!(receipt.tenant_id, "ten_alpha");
        assert_eq!(receipt.response.unwrap().nodes.len(), 2);
        assert_eq!(
            receipt.evidence_refs,
            vec![
                "evidence://policy/dec_allow_query".to_string(),
                "evidence://query/request/kgq_alpha".to_string(),
                "trace://kgq_alpha".to_string(),
            ]
        );
        assert_eq!(
            usecase
                .audit_events()
                .iter()
                .map(|event| event.kind)
                .collect::<Vec<_>>(),
            vec![
                OntologyQueryAuditEventKind::QueryRequestReceived,
                OntologyQueryAuditEventKind::QueryCompleted,
            ]
        );
    }

    #[test]
    fn duplicate_idempotency_returns_cached_receipt_without_new_audit_event() {
        let (graph, engine) = graph_and_engine();
        let mut usecase = OntologyQueryExecutionUsecase::default();

        let first = usecase.execute(&graph, &engine, input("idem-query-dup"));
        let duplicate = usecase.execute(&graph, &engine, input("idem-query-dup"));

        assert_eq!(duplicate, first);
        assert_eq!(usecase.audit_events().len(), 2);
        assert_eq!(usecase.receipt_count(), 1);
    }

    #[test]
    fn same_idempotency_key_with_different_intent_is_denied_without_replacing_original() {
        let (graph, engine) = graph_and_engine();
        let mut usecase = OntologyQueryExecutionUsecase::default();
        let first = usecase.execute(&graph, &engine, input("idem-query-conflict"));
        let mut conflicting = input("idem-query-conflict");
        conflicting.request = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_alpha",
            "ent_child",
            vec!["lty_owns"],
            1,
            0,
            12,
            vec!["lty_owns"],
            TraversalDirection::Outbound,
        )
        .unwrap();

        let conflict = usecase.execute(&graph, &engine, conflicting);
        let duplicate_original = usecase.execute(&graph, &engine, input("idem-query-conflict"));

        assert_eq!(first.status, OntologyQueryExecutionStatus::Completed);
        assert_eq!(
            conflict.denial_kind,
            Some(OntologyQueryDenialKind::IdempotencyConflict)
        );
        assert_eq!(duplicate_original, first);
        assert_eq!(usecase.receipt_count(), 1);
        assert_eq!(
            usecase.audit_events().last().unwrap().kind,
            OntologyQueryAuditEventKind::IdempotencyConflict
        );
    }

    #[test]
    fn policy_denials_block_query_before_domain_execution() {
        let (graph, engine) = graph_and_engine();
        let mut usecase = OntologyQueryExecutionUsecase::default();
        let mut denied = input("idem-query-denied");
        denied.policy_decision.allowed_query_surfaces = vec!["ontology.query.other".to_string()];

        let receipt = usecase.execute(&graph, &engine, denied);

        assert_eq!(receipt.status, OntologyQueryExecutionStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(OntologyQueryDenialKind::SurfaceDenied)
        );
        assert!(receipt.response.is_none());
        assert_eq!(
            usecase
                .audit_events()
                .iter()
                .map(|event| event.kind)
                .collect::<Vec<_>>(),
            vec![
                OntologyQueryAuditEventKind::QueryRequestReceived,
                OntologyQueryAuditEventKind::QueryDenied,
            ]
        );
    }

    #[test]
    fn depth_ceiling_denial_blocks_overbroad_query() {
        let (graph, engine) = graph_and_engine();
        let mut usecase = OntologyQueryExecutionUsecase::default();
        let mut denied = input("idem-query-depth-denied");
        denied.policy_decision.max_depth_ceiling = 0;

        let receipt = usecase.execute(&graph, &engine, denied);

        assert_eq!(
            receipt.denial_kind,
            Some(OntologyQueryDenialKind::DepthCeilingExceeded)
        );
        assert_eq!(usecase.receipt_count(), 1);
    }

    #[test]
    fn query_domain_failure_returns_failed_receipt_and_audit_event() {
        let (graph, engine) = graph_and_engine();
        let mut usecase = OntologyQueryExecutionUsecase::default();
        let mut missing_root = input("idem-query-missing-root");
        missing_root.request = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_alpha",
            "ent_missing",
            vec!["lty_owns"],
            1,
            0,
            12,
            vec!["lty_owns"],
            TraversalDirection::Outbound,
        )
        .unwrap();

        let receipt = usecase.execute(&graph, &engine, missing_root);

        assert_eq!(receipt.status, OntologyQueryExecutionStatus::Failed);
        assert_eq!(
            receipt.failure,
            Some(KnowledgeGraphQueryError::MissingRootEntity)
        );
        assert_eq!(
            usecase.audit_events().last().unwrap().kind,
            OntologyQueryAuditEventKind::QueryFailed
        );
    }

    #[test]
    fn invalid_input_rejects_before_audit_or_cache_mutation() {
        let (graph, engine) = graph_and_engine();
        let mut usecase = OntologyQueryExecutionUsecase::default();
        let mut invalid = input("");
        invalid.request_evidence_ref.clear();

        let receipt = usecase.execute(&graph, &engine, invalid);

        assert_eq!(receipt.status, OntologyQueryExecutionStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(OntologyQueryDenialKind::InvalidInput)
        );
        assert!(usecase.audit_events().is_empty());
        assert_eq!(usecase.receipt_count(), 0);
    }

    #[test]
    fn missing_policy_decision_or_trace_refs_are_invalid_before_audit() {
        let (graph, engine) = graph_and_engine();
        let mut usecase = OntologyQueryExecutionUsecase::default();
        let mut invalid = input("idem-query-missing-trace");
        invalid.trace_context_ref.clear();
        invalid.policy_decision.decision_id.clear();

        let receipt = usecase.execute(&graph, &engine, invalid);

        assert_eq!(receipt.status, OntologyQueryExecutionStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(OntologyQueryDenialKind::InvalidInput)
        );
        assert!(usecase.audit_events().is_empty());
        assert_eq!(usecase.receipt_count(), 0);
    }
}
