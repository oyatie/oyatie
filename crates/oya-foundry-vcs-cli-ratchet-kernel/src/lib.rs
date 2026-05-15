//! Grit-compatible CLI and migration ratchet for Oya VCS.
//!
//! This crate is deliberately std-only and provider-free. It parses the agent
//! ergonomics surface (`claim`, `work`, `verify`, `done`, `status`, `symbols`,
//! `queue`, `watch`, `promote`), maps compatibility aliases onto controller actions, and
//! enforces the migration ratchet that moves agents away from direct `git`/`gh`
//! and local-only closeout.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fmt;

pub const CLI_RATCHET_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RatchetStage {
    Observe,
    Warn,
    Enforce,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RatchetPolicy {
    pub stage: RatchetStage,                // data_class: INTERNAL_ONLY
    pub forbid_direct_git_gh: bool,         // data_class: INTERNAL_ONLY
    pub block_local_only_closeout: bool,    // data_class: INTERNAL_ONLY
    pub require_evidence_on_done: bool,     // data_class: INTERNAL_ONLY
    pub require_controller_promotion: bool, // data_class: INTERNAL_ONLY
}

impl RatchetPolicy {
    pub fn observe() -> Self {
        Self {
            stage: RatchetStage::Observe,
            forbid_direct_git_gh: false,
            block_local_only_closeout: false,
            require_evidence_on_done: true,
            require_controller_promotion: false,
        }
    }

    pub fn warn() -> Self {
        Self {
            stage: RatchetStage::Warn,
            forbid_direct_git_gh: false,
            block_local_only_closeout: false,
            require_evidence_on_done: true,
            require_controller_promotion: true,
        }
    }

    pub fn enforce() -> Self {
        Self {
            stage: RatchetStage::Enforce,
            forbid_direct_git_gh: true,
            block_local_only_closeout: true,
            require_evidence_on_done: true,
            require_controller_promotion: true,
        }
    }

    fn blocks_forbidden_primitives(&self) -> bool {
        self.forbid_direct_git_gh || self.stage == RatchetStage::Enforce
    }

    fn blocks_local_closeout(&self) -> bool {
        self.block_local_only_closeout || self.stage == RatchetStage::Enforce
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum OyaVcsCommandKind {
    Claim,
    Work,
    Verify,
    Done,
    Status,
    Symbols,
    Queue,
    Watch,
    Promote,
}

impl OyaVcsCommandKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Claim => "claim",
            Self::Work => "work",
            Self::Verify => "verify",
            Self::Done => "done",
            Self::Status => "status",
            Self::Symbols => "symbols",
            Self::Queue => "queue",
            Self::Watch => "watch",
            Self::Promote => "promote",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControllerAction {
    ClaimLock,
    StartWork,
    VerifyEvidence,
    EmitChangeBundle,
    ReadStatus,
    ListSymbols,
    QueueProjection,
    WatchEvents,
    PromoteBundle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloseoutMode {
    GritDone,
    ControllerPromote,
    LocalOnly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandPlan {
    pub schema_version: u32,                 // data_class: INTERNAL_ONLY
    pub kind: OyaVcsCommandKind,             // data_class: INTERNAL_ONLY
    pub action: ControllerAction,            // data_class: INTERNAL_ONLY
    pub agent_id: Option<String>,            // data_class: INTERNAL_ONLY
    pub intent: Option<String>,              // data_class: INTERNAL_ONLY
    pub scopes: Vec<String>,                 // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,          // data_class: INTERNAL_ONLY
    pub bundle_id: Option<String>,           // data_class: INTERNAL_ONLY
    pub environment: Option<String>,         // data_class: INTERNAL_ONLY
    pub closeout_mode: Option<CloseoutMode>, // data_class: INTERNAL_ONLY
    pub compatibility_alias: Option<String>, // data_class: INTERNAL_ONLY
}

impl CommandPlan {
    fn new(kind: OyaVcsCommandKind, action: ControllerAction) -> Self {
        Self {
            schema_version: CLI_RATCHET_SCHEMA_VERSION,
            kind,
            action,
            agent_id: None,
            intent: None,
            scopes: Vec::new(),
            evidence_refs: Vec::new(),
            bundle_id: None,
            environment: None,
            closeout_mode: None,
            compatibility_alias: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceCommand {
    pub command: String,              // data_class: INTERNAL_ONLY
    pub evidence_ref: Option<String>, // data_class: INTERNAL_ONLY
}

impl EvidenceCommand {
    pub fn new(command: impl Into<String>) -> Result<Self, CliRatchetError> {
        let command = normalize_non_empty(command.into(), CliRatchetError::InvalidArgument)?;
        Ok(Self {
            command,
            evidence_ref: None,
        })
    }

    pub fn with_ref(mut self, evidence_ref: impl Into<String>) -> Result<Self, CliRatchetError> {
        self.evidence_ref = Some(normalize_non_empty(
            evidence_ref.into(),
            CliRatchetError::InvalidEvidence,
        )?);
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ForbiddenPrimitive {
    Git,
    Gh,
}

impl ForbiddenPrimitive {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Git => "git",
            Self::Gh => "gh",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForbiddenPrimitiveUse {
    pub primitive: ForbiddenPrimitive, // data_class: INTERNAL_ONLY
    pub command: String,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RatchetDecision {
    pub schema_version: u32,                        // data_class: INTERNAL_ONLY
    pub accepted: bool,                             // data_class: INTERNAL_ONLY
    pub plan: CommandPlan,                          // data_class: INTERNAL_ONLY
    pub warnings: Vec<String>,                      // data_class: INTERNAL_ONLY
    pub blocking_errors: Vec<CliRatchetError>,      // data_class: INTERNAL_ONLY
    pub forbidden_uses: Vec<ForbiddenPrimitiveUse>, // data_class: INTERNAL_ONLY
}

pub fn parse_command<I, S>(args: I) -> Result<CommandPlan, CliRatchetError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    if args.first().map(String::as_str) == Some("grit") {
        args.remove(0);
        let mut plan = parse_command(args)?;
        plan.compatibility_alias = Some("grit".into());
        return Ok(plan);
    }
    let Some(command) = args.first().cloned() else {
        return Err(CliRatchetError::MissingCommand);
    };
    let rest = args.into_iter().skip(1).collect::<Vec<_>>();
    match command.as_str() {
        "claim" => parse_claim(rest),
        "work" => parse_work(rest),
        "verify" => parse_verify(rest),
        "done" => parse_done(rest),
        "status" => parse_status(rest),
        "symbols" => parse_symbols(rest),
        "queue" => parse_queue(rest),
        "watch" => parse_watch(rest),
        "promote" => parse_promote(rest),
        other => Err(CliRatchetError::UnknownCommand(other.to_string())),
    }
}

pub fn evaluate_command(
    plan: CommandPlan,
    evidence: &[EvidenceCommand],
    policy: &RatchetPolicy,
) -> RatchetDecision {
    let forbidden_uses = detect_forbidden_primitives(evidence);
    let mut warnings = Vec::new();
    let mut blocking_errors = Vec::new();

    if !forbidden_uses.is_empty() {
        if policy.blocks_forbidden_primitives() {
            for usage in &forbidden_uses {
                blocking_errors.push(CliRatchetError::ForbiddenPrimitive(usage.primitive));
            }
        } else {
            warnings.push(format!(
                "direct provider primitives observed but not yet enforced: {}",
                forbidden_uses
                    .iter()
                    .map(|usage| usage.primitive.as_str())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
    }

    if plan.kind == OyaVcsCommandKind::Done
        && policy.require_evidence_on_done
        && plan.evidence_refs.is_empty()
    {
        blocking_errors.push(CliRatchetError::MissingEvidence);
    }

    if plan.closeout_mode == Some(CloseoutMode::LocalOnly) && policy.blocks_local_closeout() {
        blocking_errors.push(CliRatchetError::LocalOnlyCloseoutBlocked);
    }

    if plan.kind == OyaVcsCommandKind::Promote
        && policy.require_controller_promotion
        && plan.action != ControllerAction::PromoteBundle
    {
        blocking_errors.push(CliRatchetError::ControllerPromotionRequired);
    }

    RatchetDecision {
        schema_version: CLI_RATCHET_SCHEMA_VERSION,
        accepted: blocking_errors.is_empty(),
        plan,
        warnings,
        blocking_errors,
        forbidden_uses,
    }
}

pub fn detect_forbidden_primitives(evidence: &[EvidenceCommand]) -> Vec<ForbiddenPrimitiveUse> {
    let mut uses = Vec::new();
    for record in evidence {
        let mut found = BTreeSet::new();
        for token in shellish_tokens(&record.command) {
            if let Some(primitive) = forbidden_primitive_from_token(&token) {
                found.insert(primitive);
            }
        }
        for primitive in found {
            uses.push(ForbiddenPrimitiveUse {
                primitive,
                command: record.command.clone(),
            });
        }
    }
    uses
}

pub fn plan_sequence(plans: &[CommandPlan]) -> Result<Vec<ControllerAction>, CliRatchetError> {
    let kinds = plans.iter().map(|plan| plan.kind).collect::<Vec<_>>();
    let has_lifecycle = kinds.iter().any(|kind| {
        matches!(
            kind,
            OyaVcsCommandKind::Claim
                | OyaVcsCommandKind::Work
                | OyaVcsCommandKind::Verify
                | OyaVcsCommandKind::Done
                | OyaVcsCommandKind::Promote
        )
    });
    if has_lifecycle {
        let claim = position_of(&kinds, OyaVcsCommandKind::Claim)?;
        let work = position_of(&kinds, OyaVcsCommandKind::Work)?;
        let verify = position_of(&kinds, OyaVcsCommandKind::Verify)?;
        let done = position_of(&kinds, OyaVcsCommandKind::Done)?;
        let promote = position_of(&kinds, OyaVcsCommandKind::Promote)?;
        if !(claim < work && work < verify && verify < done && done < promote) {
            return Err(CliRatchetError::InvalidCommandOrder);
        }
    }
    Ok(plans.iter().map(|plan| plan.action).collect())
}

fn position_of(
    kinds: &[OyaVcsCommandKind],
    required: OyaVcsCommandKind,
) -> Result<usize, CliRatchetError> {
    kinds
        .iter()
        .position(|kind| *kind == required)
        .ok_or(CliRatchetError::MissingLifecycleCommand(required))
}

fn parse_claim(args: Vec<String>) -> Result<CommandPlan, CliRatchetError> {
    let mut plan = CommandPlan::new(OyaVcsCommandKind::Claim, ControllerAction::ClaimLock);
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--agent" => {
                plan.agent_id = Some(next_value(&mut iter, CliRatchetError::MissingAgent)?)
            }
            "--intent" => {
                plan.intent = Some(next_value(&mut iter, CliRatchetError::MissingIntent)?)
            }
            "--evidence" => plan
                .evidence_refs
                .push(next_value(&mut iter, CliRatchetError::MissingEvidence)?),
            value if value.starts_with('-') => return Err(CliRatchetError::InvalidArgument),
            value => plan.scopes.push(normalize_non_empty(
                value.to_string(),
                CliRatchetError::MissingScope,
            )?),
        }
    }
    require_agent(&plan)?;
    if plan.intent.is_none() {
        return Err(CliRatchetError::MissingIntent);
    }
    if plan.scopes.is_empty() {
        return Err(CliRatchetError::MissingScope);
    }
    Ok(plan)
}

fn parse_work(args: Vec<String>) -> Result<CommandPlan, CliRatchetError> {
    let mut plan = CommandPlan::new(OyaVcsCommandKind::Work, ControllerAction::StartWork);
    parse_agent_and_evidence(args, &mut plan)?;
    require_agent(&plan)?;
    Ok(plan)
}

fn parse_verify(args: Vec<String>) -> Result<CommandPlan, CliRatchetError> {
    let mut plan = CommandPlan::new(OyaVcsCommandKind::Verify, ControllerAction::VerifyEvidence);
    parse_agent_and_evidence(args, &mut plan)?;
    require_agent(&plan)?;
    if plan.evidence_refs.is_empty() {
        return Err(CliRatchetError::MissingEvidence);
    }
    Ok(plan)
}

fn parse_done(args: Vec<String>) -> Result<CommandPlan, CliRatchetError> {
    let mut plan = CommandPlan::new(OyaVcsCommandKind::Done, ControllerAction::EmitChangeBundle);
    plan.closeout_mode = Some(CloseoutMode::GritDone);
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--agent" => {
                plan.agent_id = Some(next_value(&mut iter, CliRatchetError::MissingAgent)?)
            }
            "--evidence" => plan
                .evidence_refs
                .push(next_value(&mut iter, CliRatchetError::MissingEvidence)?),
            "--local-only" => plan.closeout_mode = Some(CloseoutMode::LocalOnly),
            "--controller-promote" => plan.closeout_mode = Some(CloseoutMode::ControllerPromote),
            value if value.starts_with('-') => return Err(CliRatchetError::InvalidArgument),
            value => plan.scopes.push(normalize_non_empty(
                value.to_string(),
                CliRatchetError::MissingScope,
            )?),
        }
    }
    require_agent(&plan)?;
    Ok(plan)
}

fn parse_status(args: Vec<String>) -> Result<CommandPlan, CliRatchetError> {
    let mut plan = CommandPlan::new(OyaVcsCommandKind::Status, ControllerAction::ReadStatus);
    parse_agent_and_evidence(args, &mut plan)?;
    Ok(plan)
}

fn parse_symbols(args: Vec<String>) -> Result<CommandPlan, CliRatchetError> {
    let mut plan = CommandPlan::new(OyaVcsCommandKind::Symbols, ControllerAction::ListSymbols);
    parse_scope_only(args, &mut plan)?;
    Ok(plan)
}

fn parse_queue(args: Vec<String>) -> Result<CommandPlan, CliRatchetError> {
    let mut plan = CommandPlan::new(OyaVcsCommandKind::Queue, ControllerAction::QueueProjection);
    parse_agent_and_evidence(args, &mut plan)?;
    Ok(plan)
}

fn parse_watch(args: Vec<String>) -> Result<CommandPlan, CliRatchetError> {
    let mut plan = CommandPlan::new(OyaVcsCommandKind::Watch, ControllerAction::WatchEvents);
    parse_agent_and_evidence(args, &mut plan)?;
    Ok(plan)
}

fn parse_promote(args: Vec<String>) -> Result<CommandPlan, CliRatchetError> {
    let mut plan = CommandPlan::new(OyaVcsCommandKind::Promote, ControllerAction::PromoteBundle);
    plan.closeout_mode = Some(CloseoutMode::ControllerPromote);
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--agent" => {
                plan.agent_id = Some(next_value(&mut iter, CliRatchetError::MissingAgent)?)
            }
            "--bundle" => {
                plan.bundle_id = Some(next_value(&mut iter, CliRatchetError::MissingBundle)?)
            }
            "--env" | "--environment" => {
                plan.environment = Some(next_value(&mut iter, CliRatchetError::MissingEnvironment)?)
            }
            "--evidence" => plan
                .evidence_refs
                .push(next_value(&mut iter, CliRatchetError::MissingEvidence)?),
            value if value.starts_with('-') => return Err(CliRatchetError::InvalidArgument),
            value => plan.scopes.push(normalize_non_empty(
                value.to_string(),
                CliRatchetError::MissingScope,
            )?),
        }
    }
    require_agent(&plan)?;
    if plan.bundle_id.is_none() {
        return Err(CliRatchetError::MissingBundle);
    }
    if plan.environment.is_none() {
        return Err(CliRatchetError::MissingEnvironment);
    }
    Ok(plan)
}

fn parse_agent_and_evidence(
    args: Vec<String>,
    plan: &mut CommandPlan,
) -> Result<(), CliRatchetError> {
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--agent" => {
                plan.agent_id = Some(next_value(&mut iter, CliRatchetError::MissingAgent)?)
            }
            "--evidence" => plan
                .evidence_refs
                .push(next_value(&mut iter, CliRatchetError::MissingEvidence)?),
            value if value.starts_with('-') => return Err(CliRatchetError::InvalidArgument),
            value => plan.scopes.push(normalize_non_empty(
                value.to_string(),
                CliRatchetError::MissingScope,
            )?),
        }
    }
    Ok(())
}

fn parse_scope_only(args: Vec<String>, plan: &mut CommandPlan) -> Result<(), CliRatchetError> {
    for arg in args {
        if arg.starts_with('-') {
            return Err(CliRatchetError::InvalidArgument);
        }
        plan.scopes
            .push(normalize_non_empty(arg, CliRatchetError::MissingScope)?);
    }
    Ok(())
}

fn require_agent(plan: &CommandPlan) -> Result<(), CliRatchetError> {
    if plan
        .agent_id
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        Err(CliRatchetError::MissingAgent)
    } else {
        Ok(())
    }
}

fn next_value(
    iter: &mut impl Iterator<Item = String>,
    error: CliRatchetError,
) -> Result<String, CliRatchetError> {
    let value = iter.next().ok_or(error.clone())?;
    if value.starts_with('-') {
        return Err(CliRatchetError::MissingOptionValue(value));
    }
    normalize_non_empty(value, error)
}

fn forbidden_primitive_from_token(token: &str) -> Option<ForbiddenPrimitive> {
    let basename = token
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(token)
        .trim_end_matches(".exe");
    match basename {
        "git" => Some(ForbiddenPrimitive::Git),
        "gh" => Some(ForbiddenPrimitive::Gh),
        _ => None,
    }
}

fn shellish_tokens(command: &str) -> Vec<String> {
    command
        .split(|ch: char| {
            ch.is_whitespace() || matches!(ch, ';' | '|' | '&' | '(' | ')' | '<' | '>')
        })
        .filter_map(|token| {
            let token = token.trim_matches(|ch: char| matches!(ch, '"' | '\'' | '`' | ','));
            if token.is_empty() {
                None
            } else {
                Some(token.to_string())
            }
        })
        .collect()
}

fn normalize_non_empty(value: String, error: CliRatchetError) -> Result<String, CliRatchetError> {
    let value = value.trim().to_string();
    if value.is_empty() || value.contains('\0') {
        Err(error)
    } else {
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliRatchetError {
    MissingCommand,
    UnknownCommand(String),
    MissingAgent,
    MissingIntent,
    MissingScope,
    MissingEvidence,
    MissingBundle,
    MissingEnvironment,
    InvalidArgument,
    InvalidEvidence,
    MissingLifecycleCommand(OyaVcsCommandKind),
    MissingOptionValue(String),
    ForbiddenPrimitive(ForbiddenPrimitive),
    LocalOnlyCloseoutBlocked,
    ControllerPromotionRequired,
    InvalidCommandOrder,
}

impl fmt::Display for CliRatchetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for CliRatchetError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(args: &[&str]) -> CommandPlan {
        parse_command(args.iter().copied()).expect("valid plan")
    }

    #[test]
    fn parses_grit_compatible_claim_work_verify_done_surface() {
        let claim = plan(&[
            "grit",
            "claim",
            "--agent",
            "agent-a",
            "--intent",
            "M-CC-P00-IP-005",
            "crates/oya-foundry-vcs-cli-ratchet-kernel",
        ]);
        assert_eq!(claim.kind, OyaVcsCommandKind::Claim);
        assert_eq!(claim.action, ControllerAction::ClaimLock);
        assert_eq!(claim.compatibility_alias.as_deref(), Some("grit"));
        assert_eq!(
            claim.scopes,
            vec!["crates/oya-foundry-vcs-cli-ratchet-kernel"]
        );

        let work = plan(&["work", "--agent", "agent-a"]);
        assert_eq!(work.action, ControllerAction::StartWork);

        let verify = plan(&[
            "verify",
            "--agent",
            "agent-a",
            "--evidence",
            "evidence/gitops-vcs/ip-005-cli-ratchet.json#verify",
        ]);
        assert_eq!(verify.action, ControllerAction::VerifyEvidence);

        let done = plan(&[
            "done",
            "--agent",
            "agent-a",
            "--evidence",
            "evidence/gitops-vcs/ip-005-cli-ratchet.json",
        ]);
        assert_eq!(done.action, ControllerAction::EmitChangeBundle);
        assert_eq!(done.closeout_mode, Some(CloseoutMode::GritDone));
    }

    #[test]
    fn parses_status_symbols_queue_watch_and_promote() {
        assert_eq!(plan(&["status"]).action, ControllerAction::ReadStatus);
        assert_eq!(
            plan(&["symbols", "crates/demo"]).action,
            ControllerAction::ListSymbols
        );
        assert_eq!(
            plan(&[
                "verify",
                "--agent",
                "agent-a",
                "--evidence",
                "evidence/gitops-vcs/ip-005-cli-ratchet.json#verify",
            ])
            .action,
            ControllerAction::VerifyEvidence
        );
        assert_eq!(plan(&["queue"]).action, ControllerAction::QueueProjection);
        assert_eq!(plan(&["watch"]).action, ControllerAction::WatchEvents);
        let promote = plan(&[
            "promote",
            "--agent",
            "agent-a",
            "--bundle",
            "cb_ip005",
            "--env",
            "staging",
            "--evidence",
            "evidence/gitops-vcs/ip-005-cli-ratchet.json#promote",
        ]);
        assert_eq!(promote.action, ControllerAction::PromoteBundle);
        assert_eq!(promote.bundle_id.as_deref(), Some("cb_ip005"));
        assert_eq!(promote.environment.as_deref(), Some("staging"));
    }

    #[test]
    fn detects_forbidden_git_and_gh_without_false_gitops_match() {
        let evidence = vec![
            EvidenceCommand::new("git status").unwrap(),
            EvidenceCommand::new("/usr/bin/gh pr create --title x").unwrap(),
            EvidenceCommand::new("oya vcs promote gitops bundle").unwrap(),
        ];
        let found = detect_forbidden_primitives(&evidence);
        assert_eq!(found.len(), 2);
        assert!(
            found
                .iter()
                .any(|usage| usage.primitive == ForbiddenPrimitive::Git)
        );
        assert!(
            found
                .iter()
                .any(|usage| usage.primitive == ForbiddenPrimitive::Gh)
        );
    }

    #[test]
    fn enforcement_rejects_direct_git_and_gh_evidence() {
        let done = plan(&[
            "done",
            "--agent",
            "agent-a",
            "--evidence",
            "evidence/gitops-vcs/ip-005-cli-ratchet.json",
        ]);
        let decision = evaluate_command(
            done,
            &[
                EvidenceCommand::new("git diff --check").unwrap(),
                EvidenceCommand::new("gh pr merge 1").unwrap(),
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
    }

    #[test]
    fn warn_stage_warns_but_does_not_block_forbidden_primitives() {
        let status = plan(&["status"]);
        let decision = evaluate_command(
            status,
            &[EvidenceCommand::new("git status").unwrap()],
            &RatchetPolicy::warn(),
        );
        assert!(decision.accepted);
        assert_eq!(decision.warnings.len(), 1);
    }

    #[test]
    fn done_requires_evidence_and_blocks_local_only_closeout_after_ratchet() {
        let no_evidence = plan(&["done", "--agent", "agent-a"]);
        let decision = evaluate_command(no_evidence, &[], &RatchetPolicy::enforce());
        assert!(!decision.accepted);
        assert!(
            decision
                .blocking_errors
                .contains(&CliRatchetError::MissingEvidence)
        );

        let local = plan(&[
            "done",
            "--agent",
            "agent-a",
            "--local-only",
            "--evidence",
            "evidence/gitops-vcs/ip-005-cli-ratchet.json",
        ]);
        let decision = evaluate_command(local, &[], &RatchetPolicy::enforce());
        assert!(!decision.accepted);
        assert!(
            decision
                .blocking_errors
                .contains(&CliRatchetError::LocalOnlyCloseoutBlocked)
        );
    }

    #[test]
    fn agent_flow_claim_work_verify_done_promote_uses_controller_actions_in_order() {
        let plans = vec![
            plan(&[
                "claim",
                "--agent",
                "agent-a",
                "--intent",
                "ship IP-005",
                "crates/oya-foundry-vcs-cli-ratchet-kernel",
            ]),
            plan(&["work", "--agent", "agent-a"]),
            plan(&[
                "verify",
                "--agent",
                "agent-a",
                "--evidence",
                "evidence/gitops-vcs/ip-005-cli-ratchet.json#verify",
            ]),
            plan(&[
                "done",
                "--agent",
                "agent-a",
                "--controller-promote",
                "--evidence",
                "evidence/gitops-vcs/ip-005-cli-ratchet.json",
            ]),
            plan(&[
                "promote",
                "--agent",
                "agent-a",
                "--bundle",
                "cb_ip005",
                "--environment",
                "production",
            ]),
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
        let done_decision = evaluate_command(plans[2].clone(), &[], &RatchetPolicy::enforce());
        assert!(
            done_decision.accepted,
            "{:?}",
            done_decision.blocking_errors
        );
    }

    #[test]
    fn command_sequence_rejects_out_of_order_done() {
        let plans = vec![
            plan(&[
                "claim",
                "--agent",
                "agent-a",
                "--intent",
                "ship IP-005",
                "crates/oya-foundry-vcs-cli-ratchet-kernel",
            ]),
            plan(&[
                "promote",
                "--agent",
                "agent-a",
                "--bundle",
                "cb_ip005",
                "--environment",
                "production",
            ]),
            plan(&["work", "--agent", "agent-a"]),
            plan(&[
                "verify",
                "--agent",
                "agent-a",
                "--evidence",
                "evidence/gitops-vcs/ip-005-cli-ratchet.json#verify",
            ]),
            plan(&[
                "done",
                "--agent",
                "agent-a",
                "--evidence",
                "evidence/gitops-vcs/ip-005-cli-ratchet.json",
            ]),
        ];
        assert_eq!(
            plan_sequence(&plans),
            Err(CliRatchetError::InvalidCommandOrder)
        );
    }

    #[test]
    fn lifecycle_sequence_requires_claim_before_work_verify_done_promote() {
        let plans = vec![
            plan(&["work", "--agent", "agent-a"]),
            plan(&[
                "verify",
                "--agent",
                "agent-a",
                "--evidence",
                "evidence/gitops-vcs/ip-005-cli-ratchet.json#verify",
            ]),
            plan(&[
                "done",
                "--agent",
                "agent-a",
                "--evidence",
                "evidence/gitops-vcs/ip-005-cli-ratchet.json",
            ]),
        ];
        assert_eq!(
            plan_sequence(&plans),
            Err(CliRatchetError::MissingLifecycleCommand(
                OyaVcsCommandKind::Claim
            ))
        );
    }

    #[test]
    fn option_values_cannot_be_other_flags() {
        assert_eq!(
            parse_command(["done", "--agent", "--local-only", "--evidence", "ev"]),
            Err(CliRatchetError::MissingOptionValue("--local-only".into()))
        );
        assert_eq!(
            parse_command(["promote", "--agent", "agent-a", "--bundle", "--env"]),
            Err(CliRatchetError::MissingOptionValue("--env".into()))
        );
    }
}
