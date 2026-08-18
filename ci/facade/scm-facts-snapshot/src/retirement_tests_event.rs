//! Event-identity and historical-dev-push tests.

use super::test_fixtures::*;
use super::test_receipts::*;
use super::*;

#[test]
fn historical_dev_push_context_fails_closed_on_head_and_parent_topology() {
    let source = fixture();
    assert_eq!(
        historical_dev_push_context_from_source(&source, CANDIDATE).expect("tuple"),
        Some((CANDIDATE.to_owned(), PROTECTED.to_owned()))
    );
    assert!(historical_dev_push_context_from_source(&source, "HEAD").is_err());

    let mut zero_parent = fixture();
    zero_parent.parents.clear();
    assert!(historical_dev_push_context_from_source(&zero_parent, CANDIDATE).is_err());

    let mut multiple_parent = fixture();
    multiple_parent.parents.push(PREDECESSOR.to_owned());
    assert!(historical_dev_push_context_from_source(&multiple_parent, CANDIDATE).is_err());

    let mut first_parent_drift = fixture();
    first_parent_drift.first_parent = PREDECESSOR.to_owned();
    assert!(historical_dev_push_context_from_source(&first_parent_drift, CANDIDATE).is_err());
}

#[test]
fn historical_dev_push_context_allows_control_plane_absent_bootstrap() {
    let mut source = fixture();
    source.trees.insert(CANDIDATE.to_owned(), Vec::new());
    assert_eq!(
        historical_dev_push_context_from_source(&source, CANDIDATE).expect("bootstrap"),
        None
    );
}


#[test]
fn event_identity_rejects_pr_parent_order_extra_parent_and_subject_aliases() {
    let mut source = fixture();
    let mut context = context();
    context.scm_event_name = "pull_request";
    context.scm_event_ref = "refs/pull/123/merge";
    context.scm_event_base_ref = "dev";
    context.subject_commit = PREDECESSOR;
    source.parents = vec![PROTECTED.to_owned(), PREDECESSOR.to_owned()];
    assert!(materialize_history_only_retirement_facts(&source, &context).is_ok());

    for parents in [
        vec![PREDECESSOR.to_owned(), PROTECTED.to_owned()],
        vec![
            PROTECTED.to_owned(),
            PREDECESSOR.to_owned(),
            CANDIDATE.to_owned(),
        ],
        vec![PROTECTED.to_owned(), CANDIDATE.to_owned()],
    ] {
        source.parents = parents;
        assert!(materialize_history_only_retirement_facts(&source, &context).is_err());
    }
}

#[test]
fn event_identity_rejects_nonself_push_and_merge_group_subjects() {
    for event in ["push", "merge_group"] {
        let source = fixture();
        let mut context = context();
        context.scm_event_name = event;
        context.scm_event_ref = if event == "push" {
            "refs/heads/dev"
        } else {
            "refs/heads/gh-readonly-queue/dev/pr-123"
        };
        context.subject_commit = PREDECESSOR;
        assert!(materialize_history_only_retirement_facts(&source, &context).is_err());
    }
}

#[test]
fn event_identity_rejects_push_merge_topology() {
    let mut source = fixture();
    source.parents = vec![PROTECTED.to_owned(), PREDECESSOR.to_owned()];

    let error = materialize_history_only_retirement_facts(&source, &context())
        .expect_err("push must not accept a direct merge topology");
    assert!(
        error.contains("push evaluated commit parents"),
        "unexpected error: {error}"
    );
}

#[test]
fn event_identity_rejects_non_dev_push_ref() {
    let mut context = context();
    context.scm_event_ref = "refs/heads/contributor";

    let error = materialize_history_only_retirement_facts(&fixture(), &context)
        .expect_err("pushes outside dev must fail closed");
    assert!(
        error.contains("refs/heads/dev"),
        "unexpected error: {error}"
    );
}

#[test]
fn event_identity_rejects_non_dev_provider_base_refs() {
    let mut push_context = context();
    push_context.scm_event_base_ref = "refs/heads/release";
    let push_error = materialize_history_only_retirement_facts(&fixture(), &push_context)
        .expect_err("push provider base ref must bind exactly to dev");
    assert!(
        push_error.contains("protected base ref"),
        "unexpected error: {push_error}"
    );

    let mut source = fixture();
    source.parents = vec![PROTECTED.to_owned(), PREDECESSOR.to_owned()];
    let mut pull_request_context = context();
    pull_request_context.scm_event_name = "pull_request";
    pull_request_context.scm_event_ref = "refs/pull/123/merge";
    pull_request_context.scm_event_base_ref = "release";
    pull_request_context.subject_commit = PREDECESSOR;
    let pull_request_error =
        materialize_history_only_retirement_facts(&source, &pull_request_context)
            .expect_err("pull-request provider base ref must bind exactly to dev");
    assert!(
        pull_request_error.contains("protected base ref"),
        "unexpected error: {pull_request_error}"
    );
}

#[test]
fn event_identity_rejects_evaluated_commit_away_from_head() {
    let mut source = fixture();
    source
        .commits
        .insert(OTHER_COMMIT.to_owned(), OTHER_COMMIT.to_owned());
    let mut context = context();
    context.evaluated_commit = OTHER_COMMIT;

    let error = materialize_history_only_retirement_facts(&source, &context)
        .expect_err("evaluated commit must resolve to HEAD");
    assert!(
        error.contains("not exact HEAD"),
        "unexpected error: {error}"
    );
}

#[test]
fn event_identity_rejects_push_with_provider_first_parent_mismatch() {
    let mut source = fixture();
    source.first_parent = PREDECESSOR.to_owned();

    let error = materialize_history_only_retirement_facts(&source, &context())
        .expect_err("push first parent must equal provider protected SHA");
    assert!(
        error.contains("not candidate first parent"),
        "unexpected error: {error}"
    );
}

#[test]
fn merge_group_keeps_evaluated_self_without_contributor_identity() {
    let source = fixture();
    let mut context = context();
    context.scm_event_name = "merge_group";
    context.scm_event_ref = "refs/heads/gh-readonly-queue/dev/pr-123";

    let facts = materialize_history_only_retirement_facts(&source, &context)
        .expect("merge-group evaluated-self topology remains valid");
    assert!(
        !facts.to_string().contains("contributor"),
        "merge-group facts must not invent a contributor field"
    );
}

#[test]
fn event_identity_rejects_merge_group_for_non_dev_target() {
    let source = fixture();
    let mut context = context();
    context.scm_event_name = "merge_group";
    context.scm_event_ref = "refs/heads/gh-readonly-queue/release/pr-123";

    let error = materialize_history_only_retirement_facts(&source, &context)
        .expect_err("merge groups targeting a branch other than dev must fail closed");
    assert!(
        error.contains("refs/heads/gh-readonly-queue/dev/"),
        "unexpected error: {error}"
    );
}

#[test]
fn event_identity_rejects_merge_group_dev_prefix_collision() {
    let source = fixture();
    let mut context = context();
    context.scm_event_name = "merge_group";
    context.scm_event_ref = "refs/heads/gh-readonly-queue/devil/pr-123";

    let error = materialize_history_only_retirement_facts(&source, &context)
        .expect_err("merge-group target matching must preserve the dev path separator");
    assert!(
        error.contains("refs/heads/gh-readonly-queue/dev/"),
        "unexpected error: {error}"
    );
}

#[test]
fn event_identity_rejects_merge_group_nested_dev_branch_collision() {
    let source = fixture();
    let mut context = context();
    context.scm_event_name = "merge_group";
    context.scm_event_ref = "refs/heads/gh-readonly-queue/dev/release/pr-123";
    context.scm_event_base_ref = "refs/heads/dev/release";

    let error = materialize_history_only_retirement_facts(&source, &context)
        .expect_err("a merge group for dev/release must not be labeled origin/dev");
    assert!(
        error.contains("protected base ref"),
        "unexpected error: {error}"
    );
}

#[test]
fn event_identity_rejects_revision_aliases_and_noncanonical_oids() {
    let mut source = fixture();
    source
        .commits
        .insert("alias".to_owned(), CANDIDATE.to_owned());
    let mut alias_context = context();
    alias_context.evaluated_commit = "alias";
    assert!(materialize_history_only_retirement_facts(&source, &alias_context).is_err());

    let mut noncanonical_context = context();
    noncanonical_context.protected_base_commit = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    assert!(materialize_history_only_retirement_facts(&source, &noncanonical_context).is_err());
}

#[test]
fn closure_and_closed_carried_identity_mutations_fail_closed() {
    let control = control_plane().entries.remove(0);
    for stage in [ReceiptStage::ClosureNew, ReceiptStage::ClosedCarried] {
        for mutation in [
            "artifact_id",
            "scope_ref",
            "planning_state",
            "dispatch_authorized",
        ] {
            let mut receipt = receipt_value(&control, true, Some(PREDECESSOR));
            match mutation {
                "artifact_id" => receipt["artifact_id"] = json!("wrong"),
                "scope_ref" => receipt["scope_ref"] = json!("wrong"),
                "planning_state" => receipt["authority"]["planning_state"] = json!("ACTIVE"),
                "dispatch_authorized" => {
                    receipt["authority"]["dispatch_authorized"] = json!(true)
                }
                _ => unreachable!(),
            }
            assert!(
                validate_receipt_identity(stage, &control, "receipt.json", &receipt).is_err(),
                "{stage:?} must reject mutated {mutation}"
            );
        }
    }
}
