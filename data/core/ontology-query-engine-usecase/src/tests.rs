//! The original inline test corpus, body verbatim.

use crate::*;
use data_ontology_kernel::ObjectGraph;
use data_ontology_query_engine_domain::{
    KnowledgeGraphQueryEngine, KnowledgeGraphQueryError, KnowledgeGraphQueryRequest,
    TraversalDirection,
};

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{ObjectEntity, ObjectProperty, PropertyTier};
use data_ontology_query_engine_domain::KnowledgeGraphLinkInstance;

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
    for (entity_id, entity_type) in [("ent_root", "ety_account"), ("ent_child", "ety_contact")] {
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
            KnowledgeGraphLinkInstance::new("ten_alpha", "ent_root", "ent_child", "lty_owns", 10)
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
