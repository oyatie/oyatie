use std::process::ExitCode;

use oya_foundry_vcs_cli_ratchet_kernel::{
    CliRatchetError, CloseoutMode, ControllerAction, EvidenceCommand, ForbiddenPrimitiveUse,
    RatchetDecision, RatchetPolicy, evaluate_command, parse_command,
};

use crate::command_output::{OutputFormat, json_escape};

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let parsed = match parse_vcs_args(args, usage) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    let output_format = parsed.output_format;
    let decision = match evaluate_vcs_args(&parsed) {
        Ok(decision) => decision,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    render_decision(&decision, output_format)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VcsArgs {
    output_format: OutputFormat,
    policy: RatchetPolicy,
    evidence_commands: Vec<String>,
    command_args: Vec<String>,
}

fn parse_vcs_args(args: Vec<String>, usage: &str) -> Result<VcsArgs, String> {
    let mut parsed = VcsArgs {
        output_format: OutputFormat::Text,
        policy: RatchetPolicy::enforce(),
        evidence_commands: Vec::new(),
        command_args: Vec::new(),
    };
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--format" => {
                let value = iter.next().ok_or_else(|| usage.to_string())?;
                parsed.output_format =
                    OutputFormat::parse(&value).ok_or_else(|| usage.to_string())?;
            }
            "--policy" => {
                let value = iter.next().ok_or_else(|| usage.to_string())?;
                parsed.policy = parse_policy(&value).ok_or_else(|| usage.to_string())?;
            }
            "--evidence-command" => {
                let command = iter.next().ok_or_else(|| {
                    "oya vcs failed: --evidence-command needs a value".to_string()
                })?;
                parsed.evidence_commands.push(command);
            }
            value if value.starts_with("--") => return Err(usage.to_string()),
            command => {
                parsed.command_args.push(command.to_string());
                parsed.command_args.extend(iter);
                break;
            }
        }
    }
    if parsed.command_args.is_empty() {
        return Err(usage.to_string());
    }
    Ok(parsed)
}

fn parse_policy(value: &str) -> Option<RatchetPolicy> {
    match value {
        "observe" => Some(RatchetPolicy::observe()),
        "warn" => Some(RatchetPolicy::warn()),
        "enforce" => Some(RatchetPolicy::enforce()),
        _ => None,
    }
}

fn evaluate_vcs_args(args: &VcsArgs) -> Result<RatchetDecision, String> {
    let plan = parse_command(args.command_args.iter().cloned())
        .map_err(|error| format!("oya vcs parse failed: {error}"))?;
    let evidence = args
        .evidence_commands
        .iter()
        .map(|command| EvidenceCommand::new(command.clone()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("oya vcs evidence failed: {error}"))?;
    Ok(evaluate_command(plan, &evidence, &args.policy))
}

fn render_decision(decision: &RatchetDecision, output_format: OutputFormat) -> ExitCode {
    match output_format {
        OutputFormat::Text => render_text(decision),
        OutputFormat::Json => render_json(decision),
    }
}

fn render_text(decision: &RatchetDecision) -> ExitCode {
    let plan = &decision.plan;
    let status = if decision.accepted {
        "accepted"
    } else {
        "rejected"
    };
    println!(
        "oya vcs {} {status}: action={} agent={} scopes={} evidence={}",
        plan.kind.as_str(),
        action_label(plan.action),
        plan.agent_id.as_deref().unwrap_or("-"),
        plan.scopes.len(),
        plan.evidence_refs.len()
    );
    for warning in &decision.warnings {
        eprintln!("warning: {warning}");
    }
    for error in &decision.blocking_errors {
        eprintln!("blocked: {error}");
    }
    for usage in &decision.forbidden_uses {
        eprintln!(
            "forbidden primitive: {} in {}",
            usage.primitive.as_str(),
            usage.command
        );
    }
    if decision.accepted {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn render_json(decision: &RatchetDecision) -> ExitCode {
    let plan = &decision.plan;
    println!(
        "{{\"command\":\"oya vcs\",\"status\":\"{}\",\"schema_version\":{},\"ratchet\":{{\"accepted\":{},\"warnings\":{},\"blocking_errors\":{},\"forbidden_uses\":{}}},\"plan\":{{\"kind\":\"{}\",\"action\":\"{}\",\"agent_id\":{},\"intent\":{},\"scopes\":{},\"evidence_refs\":{},\"bundle_id\":{},\"environment\":{},\"closeout_mode\":{},\"compatibility_alias\":{}}}}}",
        if decision.accepted { "accepted" } else { "rejected" },
        decision.schema_version,
        decision.accepted,
        json_string_array(&decision.warnings),
        json_error_array(&decision.blocking_errors),
        json_forbidden_uses(&decision.forbidden_uses),
        plan.kind.as_str(),
        action_label(plan.action),
        json_optional_string(plan.agent_id.as_deref()),
        json_optional_string(plan.intent.as_deref()),
        json_string_array(&plan.scopes),
        json_string_array(&plan.evidence_refs),
        json_optional_string(plan.bundle_id.as_deref()),
        json_optional_string(plan.environment.as_deref()),
        json_closeout(plan.closeout_mode),
        json_optional_string(plan.compatibility_alias.as_deref())
    );
    if decision.accepted {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn action_label(action: ControllerAction) -> &'static str {
    match action {
        ControllerAction::ClaimLock => "claim-lock",
        ControllerAction::StartWork => "start-work",
        ControllerAction::VerifyEvidence => "verify-evidence",
        ControllerAction::EmitChangeBundle => "emit-change-bundle",
        ControllerAction::ReadStatus => "read-status",
        ControllerAction::ListSymbols => "list-symbols",
        ControllerAction::QueueProjection => "queue-projection",
        ControllerAction::WatchEvents => "watch-events",
        ControllerAction::PromoteBundle => "promote-bundle",
    }
}

fn json_closeout(closeout: Option<CloseoutMode>) -> String {
    let label = closeout.map(|mode| match mode {
        CloseoutMode::GritDone => "legacy-grit-compatible-done",
        CloseoutMode::ControllerPromote => "controller-promote",
        CloseoutMode::LocalOnly => "local-only",
    });
    json_optional_string(label)
}

fn json_optional_string(value: Option<&str>) -> String {
    match value {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_string(),
    }
}

fn json_string_array(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>();
    format!("[{}]", values.join(","))
}

fn json_error_array(values: &[CliRatchetError]) -> String {
    let values = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(&value.to_string())))
        .collect::<Vec<_>>();
    format!("[{}]", values.join(","))
}

fn json_forbidden_uses(values: &[ForbiddenPrimitiveUse]) -> String {
    let values = values
        .iter()
        .map(|usage| {
            format!(
                "{{\"primitive\":\"{}\",\"command\":\"{}\"}}",
                usage.primitive.as_str(),
                json_escape(&usage.command)
            )
        })
        .collect::<Vec<_>>();
    format!("[{}]", values.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_foundry_vcs_cli_ratchet_kernel::RatchetStage;

    #[test]
    fn parse_vcs_args_accepts_policy_format_and_evidence_command() {
        let args = parse_vcs_args(
            vec![
                "--format".into(),
                "json".into(),
                "--policy".into(),
                "warn".into(),
                "--evidence-command".into(),
                "oya test-standard gate".into(),
                "claim".into(),
                "--agent".into(),
                "agent-a".into(),
                "--intent".into(),
                "slice".into(),
                "crates/demo::Symbol".into(),
            ],
            "usage",
        )
        .expect("vcs args parse");

        assert_eq!(args.output_format, OutputFormat::Json);
        assert_eq!(args.policy.stage, RatchetStage::Warn);
        assert_eq!(args.evidence_commands, vec!["oya test-standard gate"]);
        assert_eq!(args.command_args.first().map(String::as_str), Some("claim"));
    }

    #[test]
    fn enforce_policy_rejects_done_without_evidence() {
        let args = parse_vcs_args(
            vec!["done".into(), "--agent".into(), "agent-a".into()],
            "usage",
        )
        .expect("vcs args parse");

        let decision = evaluate_vcs_args(&args).expect("ratchet decision");

        assert!(!decision.accepted);
        assert!(
            decision
                .blocking_errors
                .contains(&CliRatchetError::MissingEvidence)
        );
    }

    #[test]
    fn enforce_policy_rejects_forbidden_provider_command_evidence() {
        let args = parse_vcs_args(
            vec![
                "--evidence-command".into(),
                "git status".into(),
                "status".into(),
            ],
            "usage",
        )
        .expect("vcs args parse");

        let decision = evaluate_vcs_args(&args).expect("ratchet decision");

        assert!(!decision.accepted);
        assert_eq!(decision.forbidden_uses.len(), 1);
    }

    #[test]
    fn verify_command_accepts_evidence_and_maps_to_controller_action() {
        let args = parse_vcs_args(
            vec![
                "verify".into(),
                "--agent".into(),
                "agent-a".into(),
                "--evidence".into(),
                "evidence/gitops-vcs/verify.json".into(),
            ],
            "usage",
        )
        .expect("vcs args parse");
        let decision = evaluate_vcs_args(&args).expect("ratchet decision");

        assert!(decision.accepted);
        assert_eq!(decision.plan.action, ControllerAction::VerifyEvidence);
    }

    #[test]
    fn json_rendering_includes_controller_plan() {
        let args = parse_vcs_args(
            vec![
                "--format".into(),
                "json".into(),
                "promote".into(),
                "--agent".into(),
                "agent-a".into(),
                "--bundle".into(),
                "bundle-1".into(),
                "--environment".into(),
                "ci-preview".into(),
            ],
            "usage",
        )
        .expect("vcs args parse");
        let decision = evaluate_vcs_args(&args).expect("ratchet decision");

        assert!(decision.accepted);
        assert_eq!(decision.plan.action, ControllerAction::PromoteBundle);
    }
}
