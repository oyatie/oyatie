// TDD red-phase tests for the OpenTofu plan-changeset domain model.
//
// These tests specify all behavioral contracts of `ResourceChange`,
// `PlanChangeset`, `PlanChangesetSummary`, and `ResourceChangeAction`.
//
// The plan-changeset model is a DISTINCT axis from the existing IaC
// plan-diff surface (`compute_iac_plan_diff`):
//   - plan-diff:       desired CellTopologyPlan vs observed CellTopologyPlan
//   - plan-changeset:  models OpenTofu plan output (tofu plan JSON), keyed
//                      by fully-qualified resource address
//
// ADR-0130 observability note:
//   `PlanChangesetSummary` fields are flat so telemetry adapters can emit
//   per-action histograms/counters without re-iterating the changeset.
//   `has_destructive_changes()` feeds `iac-plan-destructive-gate-pass-rate`.
//
// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use iac_domain::{
    CloudIacError, PlanChangeset, PlanChangesetSummary, ResourceChange, ResourceChangeAction,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn change(address: &str, action: ResourceChangeAction) -> ResourceChange {
    ResourceChange::new(address, action).expect("valid resource change")
}

fn changeset(plan_id: &str, changes: Vec<ResourceChange>) -> PlanChangeset {
    PlanChangeset::new(plan_id, changes).expect("valid plan changeset")
}

// ---------------------------------------------------------------------------
// ResourceChangeAction: all five variants accepted, Copy, Clone, Debug, Eq
// ---------------------------------------------------------------------------

#[test]
fn resource_change_action_all_variants_accepted() {
    let create = change("module.vpc.aws_vpc.main", ResourceChangeAction::Create);
    let update = change("module.vpc.aws_subnet.public", ResourceChangeAction::Update);
    let delete = change("module.vpc.aws_subnet.old", ResourceChangeAction::Delete);
    let replace = change(
        "module.vpc.aws_security_group.legacy",
        ResourceChangeAction::Replace,
    );
    let no_op = change("module.vpc.aws_vpc.existing", ResourceChangeAction::NoOp);

    assert_eq!(create.action(), ResourceChangeAction::Create);
    assert_eq!(update.action(), ResourceChangeAction::Update);
    assert_eq!(delete.action(), ResourceChangeAction::Delete);
    assert_eq!(replace.action(), ResourceChangeAction::Replace);
    assert_eq!(no_op.action(), ResourceChangeAction::NoOp);
}

#[test]
fn resource_change_action_derives_copy_clone_debug_eq() {
    let a = ResourceChangeAction::Replace;
    let b = a; // Copy
    assert_eq!(a, b);
    let c = a; // Clone via Copy
    assert_eq!(a, c);
    let debug = format!("{a:?}");
    assert!(!debug.is_empty());
    assert_ne!(ResourceChangeAction::Create, ResourceChangeAction::Delete);
}

#[test]
fn resource_change_action_ord_matches_declaration_order() {
    // Natural Ord: Create < Update < Delete < Replace < NoOp
    assert!(ResourceChangeAction::Create < ResourceChangeAction::Update);
    assert!(ResourceChangeAction::Update < ResourceChangeAction::Delete);
    assert!(ResourceChangeAction::Delete < ResourceChangeAction::Replace);
    assert!(ResourceChangeAction::Replace < ResourceChangeAction::NoOp);
}

// ---------------------------------------------------------------------------
// ResourceChange: validation
// ---------------------------------------------------------------------------

#[test]
fn resource_change_rejects_empty_address() {
    assert_eq!(
        ResourceChange::new("", ResourceChangeAction::Create).unwrap_err(),
        CloudIacError::InvalidResourceAddress
    );
}

#[test]
fn resource_change_rejects_whitespace_only_address() {
    assert_eq!(
        ResourceChange::new("   ", ResourceChangeAction::Create).unwrap_err(),
        CloudIacError::InvalidResourceAddress
    );
}

#[test]
fn resource_change_rejects_address_with_embedded_newline() {
    assert_eq!(
        ResourceChange::new(
            "module.vpc.aws_vpc.main\ninjected",
            ResourceChangeAction::Create
        )
        .unwrap_err(),
        CloudIacError::InvalidResourceAddress
    );
}

#[test]
fn resource_change_rejects_address_with_embedded_tab() {
    assert_eq!(
        ResourceChange::new("module.vpc\taws_vpc.main", ResourceChangeAction::Create).unwrap_err(),
        CloudIacError::InvalidResourceAddress
    );
}

#[test]
fn resource_change_rejects_address_with_control_char() {
    assert_eq!(
        ResourceChange::new("module.vpc\x00.main", ResourceChangeAction::Create).unwrap_err(),
        CloudIacError::InvalidResourceAddress
    );
}

#[test]
fn resource_change_address_accessor() {
    let c = change("module.cell-vpc.aws_vpc.main", ResourceChangeAction::NoOp);
    assert_eq!(c.resource_address(), "module.cell-vpc.aws_vpc.main");
}

#[test]
fn resource_change_derives_clone_debug_eq() {
    let c = change("module.vpc.aws_vpc.main", ResourceChangeAction::Create);
    let cloned = c.clone();
    assert_eq!(c, cloned);
    let debug = format!("{c:?}");
    assert!(!debug.is_empty());
}

// ---------------------------------------------------------------------------
// PlanChangeset: valid construction
// ---------------------------------------------------------------------------

#[test]
fn plan_changeset_empty_is_valid() {
    let cs = changeset("plan-empty-001", vec![]);
    assert_eq!(cs.plan_id(), "plan-empty-001");
    assert!(cs.changes().is_empty());
    assert!(!cs.has_destructive_changes());
    let summary = cs.summarize();
    assert_eq!(summary.total, 0);
    assert_eq!(summary.create_count, 0);
    assert_eq!(summary.update_count, 0);
    assert_eq!(summary.delete_count, 0);
    assert_eq!(summary.replace_count, 0);
    assert_eq!(summary.no_op_count, 0);
}

#[test]
fn plan_changeset_plan_id_accessor() {
    let cs = changeset("plan-abc-123", vec![]);
    assert_eq!(cs.plan_id(), "plan-abc-123");
}

#[test]
fn plan_changeset_changes_accessor_returns_slice() {
    let changes = vec![
        change("module.vpc.aws_vpc.main", ResourceChangeAction::Create),
        change("module.vpc.aws_subnet.public", ResourceChangeAction::Update),
    ];
    let cs = changeset("plan-002", changes);
    assert_eq!(cs.changes().len(), 2);
}

// ---------------------------------------------------------------------------
// PlanChangeset: validation errors
// ---------------------------------------------------------------------------

#[test]
fn plan_changeset_rejects_empty_plan_id() {
    assert_eq!(
        PlanChangeset::new("", vec![]).unwrap_err(),
        CloudIacError::InvalidPlanId
    );
}

#[test]
fn plan_changeset_rejects_whitespace_plan_id() {
    assert_eq!(
        PlanChangeset::new("   ", vec![]).unwrap_err(),
        CloudIacError::InvalidPlanId
    );
}

#[test]
fn plan_changeset_rejects_uppercase_plan_id() {
    assert_eq!(
        PlanChangeset::new("Plan-001", vec![]).unwrap_err(),
        CloudIacError::InvalidPlanId
    );
}

#[test]
fn plan_changeset_rejects_plan_id_with_underscore() {
    assert_eq!(
        PlanChangeset::new("plan_001", vec![]).unwrap_err(),
        CloudIacError::InvalidPlanId
    );
}

#[test]
fn plan_changeset_rejects_duplicate_resource_address() {
    let changes = vec![
        change("module.vpc.aws_vpc.main", ResourceChangeAction::Create),
        change("module.vpc.aws_vpc.main", ResourceChangeAction::Update),
    ];
    assert_eq!(
        PlanChangeset::new("plan-dup-001", changes).unwrap_err(),
        CloudIacError::DuplicateResourceAddress
    );
}

// ---------------------------------------------------------------------------
// has_destructive_changes
// ---------------------------------------------------------------------------

#[test]
fn has_destructive_changes_false_when_only_create_update_no_op() {
    let cs = changeset(
        "plan-safe-001",
        vec![
            change("module.vpc.aws_vpc.main", ResourceChangeAction::Create),
            change("module.vpc.aws_subnet.public", ResourceChangeAction::Update),
            change(
                "module.vpc.aws_route_table.main",
                ResourceChangeAction::NoOp,
            ),
        ],
    );
    assert!(!cs.has_destructive_changes());
}

#[test]
fn has_destructive_changes_true_when_delete_present() {
    let cs = changeset(
        "plan-delete-001",
        vec![
            change("module.vpc.aws_vpc.main", ResourceChangeAction::Create),
            change("module.vpc.aws_subnet.old", ResourceChangeAction::Delete),
        ],
    );
    assert!(cs.has_destructive_changes());
}

#[test]
fn has_destructive_changes_true_when_replace_present() {
    let cs = changeset(
        "plan-replace-001",
        vec![
            change("module.vpc.aws_vpc.main", ResourceChangeAction::NoOp),
            change(
                "module.vpc.aws_security_group.legacy",
                ResourceChangeAction::Replace,
            ),
        ],
    );
    assert!(cs.has_destructive_changes());
}

#[test]
fn has_destructive_changes_true_when_both_delete_and_replace_present() {
    let cs = changeset(
        "plan-destruct-001",
        vec![
            change("module.vpc.aws_subnet.old", ResourceChangeAction::Delete),
            change(
                "module.vpc.aws_security_group.legacy",
                ResourceChangeAction::Replace,
            ),
        ],
    );
    assert!(cs.has_destructive_changes());
}

#[test]
fn has_destructive_changes_false_on_empty_changeset() {
    let cs = changeset("plan-empty-002", vec![]);
    assert!(!cs.has_destructive_changes());
}

// ---------------------------------------------------------------------------
// summarize: exact counts
// ---------------------------------------------------------------------------

#[test]
fn summarize_counts_are_exact_for_mixed_changeset() {
    let cs = changeset(
        "plan-mixed-001",
        vec![
            change("r.create1", ResourceChangeAction::Create),
            change("r.create2", ResourceChangeAction::Create),
            change("r.update1", ResourceChangeAction::Update),
            change("r.delete1", ResourceChangeAction::Delete),
            change("r.replace1", ResourceChangeAction::Replace),
            change("r.replace2", ResourceChangeAction::Replace),
            change("r.noop1", ResourceChangeAction::NoOp),
            change("r.noop2", ResourceChangeAction::NoOp),
            change("r.noop3", ResourceChangeAction::NoOp),
        ],
    );
    let summary = cs.summarize();
    assert_eq!(summary.create_count, 2);
    assert_eq!(summary.update_count, 1);
    assert_eq!(summary.delete_count, 1);
    assert_eq!(summary.replace_count, 2);
    assert_eq!(summary.no_op_count, 3);
    assert_eq!(summary.total, 9);
}

#[test]
fn summarize_total_equals_sum_of_per_action_counts() {
    let cs = changeset(
        "plan-total-check",
        vec![
            change("r.a", ResourceChangeAction::Create),
            change("r.b", ResourceChangeAction::Replace),
            change("r.c", ResourceChangeAction::NoOp),
        ],
    );
    let s = cs.summarize();
    assert_eq!(
        s.total,
        s.create_count + s.update_count + s.delete_count + s.replace_count + s.no_op_count
    );
}

#[test]
fn summarize_derives_clone_copy_debug_eq() {
    let cs = changeset(
        "plan-derive-check",
        vec![change("r.a", ResourceChangeAction::Update)],
    );
    let s: PlanChangesetSummary = cs.summarize();
    let cloned = s; // Copy
    assert_eq!(s, cloned);
    let debug = format!("{s:?}");
    assert!(!debug.is_empty());
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn plan_changeset_summarize_is_deterministic() {
    let changes = vec![
        change("module.vpc.aws_vpc.main", ResourceChangeAction::Create),
        change(
            "module.dns.aws_route53_zone.primary",
            ResourceChangeAction::Update,
        ),
        change("module.kms.aws_kms_key.old", ResourceChangeAction::Delete),
        change(
            "module.iam.aws_iam_role.compute",
            ResourceChangeAction::Replace,
        ),
        change("module.s3.aws_s3_bucket.logs", ResourceChangeAction::NoOp),
    ];
    let cs1 = PlanChangeset::new("plan-determinism-001", changes.clone())
        .expect("valid changeset for determinism check 1");
    let cs2 = PlanChangeset::new("plan-determinism-001", changes)
        .expect("valid changeset for determinism check 2");

    assert_eq!(cs1.summarize(), cs2.summarize());
    assert_eq!(cs1.has_destructive_changes(), cs2.has_destructive_changes());
}

// ---------------------------------------------------------------------------
// PlanChangeset derives Clone, Debug, Eq, PartialEq
// ---------------------------------------------------------------------------

#[test]
fn plan_changeset_derives_clone_debug_eq() {
    let cs = changeset(
        "plan-trait-check",
        vec![change("r.x", ResourceChangeAction::Create)],
    );
    let cloned = cs.clone();
    assert_eq!(cs, cloned);
    let debug = format!("{cs:?}");
    assert!(!debug.is_empty());

    let cs2 = changeset(
        "plan-trait-check",
        vec![change("r.x", ResourceChangeAction::Delete)],
    );
    assert_ne!(cs, cs2);
}
