// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_vcs_cli_ratchet_kernel::{
    CliRatchetError, CloseoutMode, ControllerAction, EvidenceCommand, ForbiddenPrimitive,
    OyaVcsCommandKind, RatchetPolicy, detect_forbidden_primitives, evaluate_command, parse_command,
    plan_sequence,
};

#[test]
fn grit_alias_claim_maps_to_controller_lock_claim() {
    let plan = parse_command([
        "grit",
        "claim",
        "--agent",
        "agent-ip005",
        "--intent",
        "M01-P07-IP-005",
        "crates/oya-vcs-cli-ratchet-kernel",
    ])
    .unwrap();

    assert_eq!(plan.kind, OyaVcsCommandKind::Claim);
    assert_eq!(plan.action, ControllerAction::ClaimLock);
    assert_eq!(plan.compatibility_alias.as_deref(), Some("grit"));
}

#[test]
fn direct_provider_primitives_are_rejected_after_ratchet_arm() {
    let plan = parse_command([
        "done",
        "--agent",
        "agent-ip005",
        "--evidence",
        "evidence/gitops-vcs/ip-005-cli-ratchet.json",
    ])
    .unwrap();
    let decision = evaluate_command(
        plan,
        &[
            EvidenceCommand::new("git status --short").unwrap(),
            EvidenceCommand::new("gh pr create --fill").unwrap(),
        ],
        &RatchetPolicy::enforce(),
    );

    assert!(!decision.accepted);
    assert!(
        decision
            .blocking_errors
            .contains(&CliRatchetError::ForbiddenPrimitive(
                ForbiddenPrimitive::Git
            ))
    );
    assert!(
        decision
            .blocking_errors
            .contains(&CliRatchetError::ForbiddenPrimitive(ForbiddenPrimitive::Gh))
    );
    assert_eq!(
        detect_forbidden_primitives(
            &decision
                .forbidden_uses
                .iter()
                .map(|usage| EvidenceCommand::new(usage.command.clone()).unwrap())
                .collect::<Vec<_>>()
        )
        .len(),
        2
    );
}

#[test]
fn local_only_closeout_is_blocked_after_ratchet_arm() {
    let plan = parse_command([
        "done",
        "--agent",
        "agent-ip005",
        "--local-only",
        "--evidence",
        "evidence/gitops-vcs/ip-005-cli-ratchet.json",
    ])
    .unwrap();

    assert_eq!(plan.closeout_mode, Some(CloseoutMode::LocalOnly));
    let decision = evaluate_command(plan, &[], &RatchetPolicy::enforce());
    assert!(!decision.accepted);
    assert!(
        decision
            .blocking_errors
            .contains(&CliRatchetError::LocalOnlyCloseoutBlocked)
    );
}

#[test]
fn malformed_option_value_and_incomplete_lifecycle_are_rejected() {
    assert_eq!(
        parse_command(["done", "--agent", "--local-only", "--evidence", "ev"]),
        Err(CliRatchetError::MissingOptionValue("--local-only".into()))
    );

    let plans = vec![
        parse_command(["work", "--agent", "agent-ip005"]).unwrap(),
        parse_command([
            "done",
            "--agent",
            "agent-ip005",
            "--evidence",
            "evidence/gitops-vcs/ip-005-cli-ratchet.json",
        ])
        .unwrap(),
    ];
    assert_eq!(
        plan_sequence(&plans),
        Err(CliRatchetError::MissingLifecycleCommand(
            OyaVcsCommandKind::Claim
        ))
    );
}

#[test]
fn claim_work_verify_done_promote_flow_is_controller_ordered() {
    let plans = vec![
        parse_command([
            "claim",
            "--agent",
            "agent-ip005",
            "--intent",
            "M01-P07-IP-005",
            "crates/oya-vcs-cli-ratchet-kernel",
        ])
        .unwrap(),
        parse_command(["work", "--agent", "agent-ip005"]).unwrap(),
        parse_command([
            "verify",
            "--agent",
            "agent-ip005",
            "--evidence",
            "evidence/gitops-vcs/ip-005-cli-ratchet.json#verify",
        ])
        .unwrap(),
        parse_command([
            "done",
            "--agent",
            "agent-ip005",
            "--controller-promote",
            "--evidence",
            "evidence/gitops-vcs/ip-005-cli-ratchet.json",
        ])
        .unwrap(),
        parse_command([
            "promote",
            "--agent",
            "agent-ip005",
            "--bundle",
            "cb_ip005",
            "--env",
            "production",
        ])
        .unwrap(),
    ];

    assert_eq!(
        plan_sequence(&plans).unwrap(),
        vec![
            ControllerAction::ClaimLock,
            ControllerAction::StartWork,
            ControllerAction::VerifyEvidence,
            ControllerAction::EmitChangeBundle,
            ControllerAction::PromoteBundle,
        ]
    );
}
