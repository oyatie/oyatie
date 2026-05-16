// ADR-0083 Tier 3: integration tests use .unwrap() / .expect() / panic! to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_tooling_agent_read as lib;

use lib::audit::MemoryAuditor;
use lib::runner::{CommandOutput, CommandSpec, MemoryRunner};

#[test]
fn pr_comments_uses_comments_json_shape() {
    let runner = MemoryRunner::new(vec![(
        CommandSpec::new(
            "gh",
            vec![
                "pr",
                "view",
                "42",
                "--comments",
                "--json",
                "number,comments,reviews",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        ),
        CommandOutput {
            status: 0,
            stdout: "{\"number\":42}\n".to_string(),
            stderr: String::new(),
        },
    )]);
    let auditor = MemoryAuditor::default();
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = lib::run_cli(
        vec!["pr-comments", "42"].into_iter().map(String::from),
        &runner,
        &auditor,
        &mut out,
        &mut err,
    )
    .unwrap();
    assert_eq!(code, 0);
    assert_eq!(String::from_utf8(out).unwrap(), "{\"number\":42}\n");
    assert_eq!(auditor.records()[0].event, "EVT-AGENT-READ-PR_COMMENTS");
}
