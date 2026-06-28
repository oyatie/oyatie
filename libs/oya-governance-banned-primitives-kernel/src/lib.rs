//! Fenced agent-instruction banned-primitives fitness kernel.
//!
//! The kernel is I/O-free. Runners enumerate `<!-- agent-instructions:start -->`
//! blocks, detect primitive usages inside the blocks, and pass typed records into
//! [`check_documented_genuine_need`].
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

pub const REQUIRED_ROOT_AGENT_SOURCES: [&str; 3] = ["AGENTS.md", "CLAUDE.md", "docs/AGENTS.md"];
const START_MARKER: &str = "<!-- agent-instructions:start -->";
const END_MARKER: &str = "<!-- agent-instructions:end -->";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentInstructionSource {
    pub path: String,           // data_class: INTERNAL_ONLY
    pub fence_count: usize,     // data_class: INTERNAL_ONLY
    pub rewrite_verified: bool, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrimitiveUsage {
    pub path: String,                  // data_class: INTERNAL_ONLY
    pub line: u32,                     // data_class: INTERNAL_ONLY
    pub primitive: PrimitiveKind,      // data_class: INTERNAL_ONLY
    pub icm_rationale: Option<String>, // data_class: INTERNAL_ONLY
    pub context: String,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentInstructionFileScan {
    pub source: AgentInstructionSource, // data_class: INTERNAL_ONLY
    pub usages: Vec<PrimitiveUsage>,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandInvocation {
    pub source: String,  // data_class: INTERNAL_ONLY
    pub line: u32,       // data_class: INTERNAL_ONLY
    pub command: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandInvocationScan {
    pub invocation: CommandInvocation, // data_class: INTERNAL_ONLY
    pub usages: Vec<PrimitiveUsage>,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PrimitiveKind {
    DirectVcs,
    DirectForge,
    TokenFilteredVcs,
    TokenFilteredForge,
    HookBypass,
    ForcePush,
    UserHomeMutation,
    ExternalFetch,
    ForgeMerge,
    ProcessKill,
    ManualBranch,
    ManualRebase,
    ManualMerge,
    ManualPush,
    ManualMutation,
}

impl PrimitiveKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DirectVcs => "direct-vcs",
            Self::DirectForge => "direct-forge",
            Self::TokenFilteredVcs => "token-filtered-vcs",
            Self::TokenFilteredForge => "token-filtered-forge",
            Self::HookBypass => "hook-bypass",
            Self::ForcePush => "force-push",
            Self::UserHomeMutation => "user-home-mutation",
            Self::ExternalFetch => "external-fetch",
            Self::ForgeMerge => "forge-merge",
            Self::ProcessKill => "process-kill",
            Self::ManualBranch => "manual-branch",
            Self::ManualRebase => "manual-rebase",
            Self::ManualMerge => "manual-merge",
            Self::ManualPush => "manual-push",
            Self::ManualMutation => "manual-mutation",
        }
    }

    pub fn is_hard_banned(self) -> bool {
        matches!(
            self,
            Self::HookBypass
                | Self::ForcePush
                | Self::UserHomeMutation
                | Self::ExternalFetch
                | Self::ForgeMerge
                | Self::ProcessKill
                | Self::ManualBranch
                | Self::ManualRebase
                | Self::ManualMerge
                | Self::ManualPush
                | Self::ManualMutation
        )
    }

    pub fn requires_documented_genuine_need(self) -> bool {
        matches!(
            self,
            Self::DirectVcs | Self::DirectForge | Self::TokenFilteredVcs | Self::TokenFilteredForge
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BannedPrimitivesFitnessReport {
    pub sources_checked: usize,       // data_class: INTERNAL_ONLY
    pub fences_checked: usize,        // data_class: INTERNAL_ONLY
    pub usages_checked: usize,        // data_class: INTERNAL_ONLY
    pub documented_exceptions: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BannedPrimitivesFitnessError {
    MissingRequiredSourceFence {
        path: String,
    },
    SourceHasNoFence {
        path: String,
    },
    HardBannedPrimitive {
        path: String,
        line: u32,
        primitive: PrimitiveKind,
    },
    UnjustifiedDirectPrimitive {
        path: String,
        line: u32,
        primitive: PrimitiveKind,
    },
    UnknownRationale {
        path: String,
        line: u32,
        rationale: String,
    },
}

impl BannedPrimitivesFitnessError {
    pub fn message(&self) -> String {
        match self {
            Self::MissingRequiredSourceFence { path } => {
                format!("required agent source '{path}' is missing a fenced instruction block")
            }
            Self::SourceHasNoFence { path } => {
                format!("agent source '{path}' has no fenced instruction block")
            }
            Self::HardBannedPrimitive {
                path,
                line,
                primitive,
            } => format!(
                "{path}:{line} uses hard-banned primitive {}",
                primitive.as_str()
            ),
            Self::UnjustifiedDirectPrimitive {
                path,
                line,
                primitive,
            } => format!(
                "{path}:{line} uses {} without an icm rationale",
                primitive.as_str()
            ),
            Self::UnknownRationale {
                path,
                line,
                rationale,
            } => {
                format!("{path}:{line} cites unknown icm rationale '{rationale}'")
            }
        }
    }
}

pub fn scan_agent_instruction_file(
    path: &str,
    contents: &str,
) -> Result<AgentInstructionFileScan, String> {
    let mut in_fence = false;
    let mut fence_count = 0usize;
    let mut usages = Vec::new();

    for (index, line) in contents.lines().enumerate() {
        let line_number = (index + 1) as u32;
        let trimmed = line.trim();
        if trimmed == START_MARKER {
            if in_fence {
                return Err(format!(
                    "{path}:{line_number}: nested agent-instructions fence"
                ));
            }
            in_fence = true;
            fence_count += 1;
            continue;
        }
        if trimmed == END_MARKER {
            if !in_fence {
                return Err(format!(
                    "{path}:{line_number}: agent-instructions end without start"
                ));
            }
            in_fence = false;
            continue;
        }
        if in_fence {
            detect_usages(path, line_number, line, &mut usages);
        }
    }

    if in_fence {
        return Err(format!("{path}: unterminated agent-instructions fence"));
    }

    Ok(AgentInstructionFileScan {
        source: AgentInstructionSource {
            path: path.to_string(),
            fence_count,
            rewrite_verified: fence_count > 0,
        },
        usages,
    })
}

pub fn scan_command_invocation(invocation: CommandInvocation) -> CommandInvocationScan {
    let mut usages = Vec::new();
    detect_usages(
        &invocation.source,
        invocation.line,
        &invocation.command,
        &mut usages,
    );

    CommandInvocationScan { invocation, usages }
}

pub fn check_documented_genuine_need(
    sources: &[AgentInstructionSource],
    usages: &[PrimitiveUsage],
    known_rationales: &[String],
) -> Result<BannedPrimitivesFitnessReport, BannedPrimitivesFitnessError> {
    for required in REQUIRED_ROOT_AGENT_SOURCES {
        let found = sources.iter().any(|source| {
            source.path == required && source.fence_count > 0 && source.rewrite_verified
        });
        if !found {
            return Err(BannedPrimitivesFitnessError::MissingRequiredSourceFence {
                path: required.to_string(),
            });
        }
    }

    for source in sources {
        if source.fence_count == 0 || !source.rewrite_verified {
            return Err(BannedPrimitivesFitnessError::SourceHasNoFence {
                path: source.path.clone(),
            });
        }
    }

    let known = known_rationales
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut documented_exceptions = 0usize;

    for usage in usages {
        if usage.primitive.is_hard_banned() {
            return Err(BannedPrimitivesFitnessError::HardBannedPrimitive {
                path: usage.path.clone(),
                line: usage.line,
                primitive: usage.primitive,
            });
        }
        if usage.primitive.requires_documented_genuine_need() {
            let rationale = usage.icm_rationale.as_deref().ok_or_else(|| {
                BannedPrimitivesFitnessError::UnjustifiedDirectPrimitive {
                    path: usage.path.clone(),
                    line: usage.line,
                    primitive: usage.primitive,
                }
            })?;
            if !known.contains(rationale) {
                return Err(BannedPrimitivesFitnessError::UnknownRationale {
                    path: usage.path.clone(),
                    line: usage.line,
                    rationale: rationale.to_string(),
                });
            }
            documented_exceptions += 1;
        }
    }

    Ok(BannedPrimitivesFitnessReport {
        sources_checked: sources.len(),
        fences_checked: sources.iter().map(|source| source.fence_count).sum(),
        usages_checked: usages.len(),
        documented_exceptions,
    })
}

fn detect_usages(path: &str, line: u32, contents: &str, usages: &mut Vec<PrimitiveUsage>) {
    let lower = contents.to_ascii_lowercase();
    let rationale = extract_rationale(contents);
    let token_segments = shellish_token_segments(&lower);

    for (needle, primitive) in [
        ("--no-verify", PrimitiveKind::HookBypass),
        ("force-with-lease", PrimitiveKind::ForcePush),
        ("push --force", PrimitiveKind::ForcePush),
        ("~/.claude/", PrimitiveKind::UserHomeMutation),
        ("~/.codex/", PrimitiveKind::UserHomeMutation),
    ] {
        if lower.contains(needle) {
            usages.push(primitive_usage(
                path,
                line,
                primitive,
                rationale.clone(),
                contents,
            ));
        }
    }

    for tokens in &token_segments {
        for (index, token) in tokens.iter().enumerate() {
            if token == "git" {
                if is_sanctioned_git_token(tokens, index) {
                    continue;
                }
                if is_token_filtered(tokens, index, "rtk") {
                    usages.push(primitive_usage(
                        path,
                        line,
                        PrimitiveKind::TokenFilteredVcs,
                        rationale.clone(),
                        contents,
                    ));
                    continue;
                }
                if let Some((subcommand_index, subcommand)) = git_subcommand(tokens, index + 1) {
                    if let Some(primitive) =
                        git_subcommand_primitive(tokens, subcommand_index, subcommand)
                    {
                        usages.push(primitive_usage(
                            path,
                            line,
                            primitive,
                            rationale.clone(),
                            contents,
                        ));
                    } else if !is_known_git_subcommand(subcommand) {
                        usages.push(primitive_usage(
                            path,
                            line,
                            PrimitiveKind::ManualMutation,
                            rationale.clone(),
                            contents,
                        ));
                    }
                }
                usages.push(primitive_usage(
                    path,
                    line,
                    PrimitiveKind::DirectVcs,
                    rationale.clone(),
                    contents,
                ));
            }
        }
    }

    if lower.contains("kill -9") && lower.contains("pgrep claude") || lower.contains("pkill claude")
    {
        usages.push(primitive_usage(
            path,
            line,
            PrimitiveKind::ProcessKill,
            rationale.clone(),
            contents,
        ));
    }

    if contains_word(&lower, "curl") || contains_word(&lower, "wget") {
        usages.push(primitive_usage(
            path,
            line,
            PrimitiveKind::ExternalFetch,
            rationale.clone(),
            contents,
        ));
    }

    for tokens in &token_segments {
        for (index, token) in tokens.iter().enumerate() {
            if token == "gh" {
                if is_token_filtered(tokens, index, "rtk") {
                    usages.push(primitive_usage(
                        path,
                        line,
                        PrimitiveKind::TokenFilteredForge,
                        rationale.clone(),
                        contents,
                    ));
                    continue;
                }
                if is_gh_pr_merge(tokens, index) {
                    usages.push(primitive_usage(
                        path,
                        line,
                        PrimitiveKind::ForgeMerge,
                        rationale.clone(),
                        contents,
                    ));
                }
                usages.push(primitive_usage(
                    path,
                    line,
                    PrimitiveKind::DirectForge,
                    rationale.clone(),
                    contents,
                ));
            }
        }
    }
}

fn shellish_token_segments(line: &str) -> Vec<Vec<String>> {
    let mut segments = Vec::new();
    let mut segment = String::new();
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        let boundary = match ch {
            ';' | '\n' | '`' => true,
            '&' => {
                if chars.peek() == Some(&'&') {
                    chars.next();
                }
                true
            }
            '|' => {
                if chars.peek() == Some(&'|') {
                    chars.next();
                }
                true
            }
            _ => false,
        };

        if boundary {
            let tokens = shellish_tokens(&segment);
            if !tokens.is_empty() {
                segments.push(tokens);
            }
            segment.clear();
        } else {
            segment.push(ch);
        }
    }

    let tokens = shellish_tokens(&segment);
    if !tokens.is_empty() {
        segments.push(tokens);
    }

    segments
}

fn shellish_tokens(line: &str) -> Vec<String> {
    line.split(|ch: char| {
        !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '/' || ch == '.')
    })
    .filter(|token| !token.is_empty())
    .map(str::to_string)
    .collect()
}

fn is_sanctioned_git_token(tokens: &[String], index: usize) -> bool {
    index
        .checked_sub(1)
        .and_then(|previous_index| {
            tokens
                .get(previous_index)
                .map(|token| (previous_index, token.as_str()))
        })
        .map(|(previous_index, token)| {
            previous_index == 0 && (token == "oya" || token.ends_with("/oya"))
        })
        .unwrap_or(false)
}

fn is_token_filtered(tokens: &[String], index: usize, previous: &str) -> bool {
    index
        .checked_sub(1)
        .and_then(|previous_index| tokens.get(previous_index))
        .map(|token| token == previous)
        .unwrap_or(false)
}

fn git_subcommand(tokens: &[String], start: usize) -> Option<(usize, &str)> {
    let mut index = start;
    while let Some(token) = tokens.get(index) {
        if token.starts_with('-') {
            index += git_option_width(token);
            continue;
        }
        return Some((index, token.as_str()));
    }
    None
}

fn git_option_width(option: &str) -> usize {
    match option {
        "-c" | "-C" | "--git-dir" | "--work-tree" | "--namespace" | "--exec-path" => 2,
        _ => 1,
    }
}

fn is_known_git_subcommand(subcommand: &str) -> bool {
    matches!(
        subcommand,
        "add"
            | "branch"
            | "checkout"
            | "clone"
            | "commit"
            | "config"
            | "diff"
            | "fetch"
            | "log"
            | "merge"
            | "pull"
            | "push"
            | "rebase"
            | "remote"
            | "reset"
            | "restore"
            | "rev-parse"
            | "show"
            | "stash"
            | "status"
            | "switch"
            | "tag"
            | "worktree"
    )
}

fn git_subcommand_primitive(
    tokens: &[String],
    subcommand_index: usize,
    subcommand: &str,
) -> Option<PrimitiveKind> {
    match subcommand {
        "push" => Some(PrimitiveKind::ManualPush),
        "merge" => Some(PrimitiveKind::ManualMerge),
        "rebase" => Some(PrimitiveKind::ManualRebase),
        "branch" => Some(PrimitiveKind::ManualBranch),
        "checkout" | "switch" => tokens[subcommand_index + 1..]
            .iter()
            .any(|token| token == "-b" || token == "-c")
            .then_some(PrimitiveKind::ManualBranch),
        "worktree" => worktree_add_creates_branch(&tokens[subcommand_index + 1..])
            .then_some(PrimitiveKind::ManualBranch),
        "update-ref" => Some(PrimitiveKind::ManualMutation),
        _ => None,
    }
}

fn worktree_add_creates_branch(tokens: &[String]) -> bool {
    let Some(add_index) = tokens.iter().position(|token| token == "add") else {
        return false;
    };
    tokens[add_index + 1..].iter().any(|token| token == "-b")
}

fn is_gh_pr_merge(tokens: &[String], index: usize) -> bool {
    tokens.get(index + 1).map(String::as_str) == Some("pr")
        && tokens.get(index + 2).map(String::as_str) == Some("merge")
}

fn primitive_usage(
    path: &str,
    line: u32,
    primitive: PrimitiveKind,
    icm_rationale: Option<String>,
    context: &str,
) -> PrimitiveUsage {
    PrimitiveUsage {
        path: path.to_string(),
        line,
        primitive,
        icm_rationale,
        context: context.trim().to_string(),
    }
}

fn extract_rationale(line: &str) -> Option<String> {
    for marker in ["icm_rationale:", "rationale_id:", "rationale:", "icm:"] {
        if let Some((_, tail)) = line.split_once(marker) {
            let value = tail
                .trim()
                .trim_matches('`')
                .trim_matches('"')
                .trim_matches('\'')
                .split(|ch: char| ch.is_whitespace() || ch == ',' || ch == ';')
                .next()
                .unwrap_or("")
                .trim_matches('`')
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

fn contains_word(haystack: &str, needle: &str) -> bool {
    let mut search_start = 0usize;
    while let Some(offset) = haystack[search_start..].find(needle) {
        let start = search_start + offset;
        let end = start + needle.len();
        let before = haystack[..start].chars().next_back();
        let after = haystack[end..].chars().next();
        if !is_word_char(before) && !is_word_char(after) {
            return true;
        }
        search_start = end;
    }
    false
}

fn is_word_char(ch: Option<char>) -> bool {
    ch.map(|value| value.is_ascii_alphanumeric() || value == '_')
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_rewritten_sources_with_no_direct_primitives() {
        let report = check_documented_genuine_need(&required_sources(), &[], &[])
            .expect("rewritten sources pass");

        assert_eq!(report.sources_checked, 3);
        assert_eq!(report.fences_checked, 3);
        assert_eq!(report.usages_checked, 0);
        assert_eq!(report.documented_exceptions, 0);
    }

    #[test]
    fn rejects_missing_required_root_fence() {
        let mut sources = required_sources();
        sources.retain(|source| source.path != "CLAUDE.md");

        assert_eq!(
            check_documented_genuine_need(&sources, &[], &[]),
            Err(BannedPrimitivesFitnessError::MissingRequiredSourceFence {
                path: "CLAUDE.md".into()
            })
        );
    }

    #[test]
    fn rejects_hard_banned_primitive_even_with_rationale() {
        let usages = [usage(PrimitiveKind::HookBypass, Some("ICM-1"))];

        assert_eq!(
            check_documented_genuine_need(&required_sources(), &usages, &["ICM-1".into()]),
            Err(BannedPrimitivesFitnessError::HardBannedPrimitive {
                path: "AGENTS.md".into(),
                line: 9,
                primitive: PrimitiveKind::HookBypass,
            })
        );
    }

    #[test]
    fn rejects_process_kill_even_with_rationale() {
        let usages = [usage(PrimitiveKind::ProcessKill, Some("ICM-1"))];

        assert_eq!(
            check_documented_genuine_need(&required_sources(), &usages, &["ICM-1".into()]),
            Err(BannedPrimitivesFitnessError::HardBannedPrimitive {
                path: "AGENTS.md".into(),
                line: 9,
                primitive: PrimitiveKind::ProcessKill,
            })
        );
    }

    #[test]
    fn scan_ignores_sanctioned_oya_git_surface() {
        let scan = scan_agent_instruction_file(
            "AGENTS.md",
            "<!-- agent-instructions:start -->\noya git status --short\n<!-- agent-instructions:end -->",
        )
        .expect("scan succeeds");

        assert!(scan.usages.is_empty());
    }

    #[test]
    fn scan_command_invocation_allows_sanctioned_bin_oya_git_surface() {
        let scan = scan_command_invocation(CommandInvocation {
            source: "registry/governance-corpora/banned-primitives/command-log.v1.jsonl".into(),
            line: 1,
            command: "bin/oya git status --short".into(),
        });

        assert!(scan.usages.is_empty());
    }
    #[test]
    fn scan_command_invocation_allows_sanctioned_oya_git_surface() {
        let scan = scan_command_invocation(CommandInvocation {
            source: "registry/governance-corpora/banned-primitives/command-log.v1.jsonl".into(),
            line: 1,
            command: "oya git status --short".into(),
        });

        assert!(scan.usages.is_empty());
    }

    #[test]
    fn scan_command_invocation_rejects_previous_segment_oya_before_git_push() {
        let scan = scan_command_invocation(CommandInvocation {
            source: "registry/governance-corpora/banned-primitives/reject-laundered-push.jsonl"
                .into(),
            line: 1,
            command: "echo oya; git push origin dev".into(),
        });

        assert_eq!(scan.usages[0].primitive, PrimitiveKind::ManualPush);
    }

    #[test]
    fn scan_command_invocation_rejects_pipe_laundered_git_push() {
        let scan = scan_command_invocation(CommandInvocation {
            source:
                "registry/governance-corpora/banned-primitives/reject-pipe-laundered-push.jsonl"
                    .into(),
            line: 1,
            command: "oya | git push origin dev".into(),
        });

        assert_eq!(scan.usages[0].primitive, PrimitiveKind::ManualPush);
    }

    #[test]
    fn scan_command_invocation_rejects_background_laundered_git_push() {
        let scan = scan_command_invocation(CommandInvocation {
            source:
                "registry/governance-corpora/banned-primitives/reject-ampersand-laundered-push.jsonl"
                    .into(),
            line: 1,
            command: "oya & git push origin dev".into(),
        });

        assert_eq!(scan.usages[0].primitive, PrimitiveKind::ManualPush);
    }

    #[test]
    fn scan_command_invocation_rejects_separator_laundered_git_push_variants() {
        for separator in ["|", "||", "&", "&&", ";"] {
            let scan = scan_command_invocation(CommandInvocation {
                source:
                    "registry/governance-corpora/banned-primitives/reject-separator-laundered-push.jsonl"
                        .into(),
                line: 1,
                command: format!("oya {separator} git push origin dev"),
            });

            assert_eq!(
                scan.usages[0].primitive,
                PrimitiveKind::ManualPush,
                "{separator} must not let prior `oya` sanitize `git push`"
            );
        }
    }

    #[test]
    fn scan_command_invocation_rejects_non_command_head_oya_before_git_push() {
        let scan = scan_command_invocation(CommandInvocation {
            source: "registry/governance-corpora/banned-primitives/reject-laundered-push.jsonl"
                .into(),
            line: 1,
            command: "echo oya git push origin dev".into(),
        });

        assert_eq!(scan.usages[0].primitive, PrimitiveKind::ManualPush);
    }

    #[test]
    fn scan_command_invocation_rejects_update_ref_as_manual_mutation() {
        let scan = scan_command_invocation(CommandInvocation {
            source: "registry/governance-corpora/banned-primitives/reject-update-ref.jsonl".into(),
            line: 1,
            command: "git update-ref refs/heads/dev HEAD".into(),
        });

        assert_eq!(scan.usages[0].primitive, PrimitiveKind::ManualMutation);
        assert_eq!(scan.usages[1].primitive, PrimitiveKind::DirectVcs);
    }

    #[test]
    fn scan_command_invocation_rejects_direct_git_status() {
        let scan = scan_command_invocation(CommandInvocation {
            source: "registry/governance-corpora/banned-primitives/reject-direct-git.jsonl".into(),
            line: 1,
            command: "git status --short".into(),
        });

        assert_eq!(scan.usages.len(), 1);
        assert_eq!(scan.usages[0].primitive, PrimitiveKind::DirectVcs);
    }

    #[test]
    fn scan_command_invocation_rejects_hard_banned_even_with_rationale() {
        let scan = scan_command_invocation(CommandInvocation {
            source: "registry/governance-corpora/banned-primitives/reject-manual-push.jsonl".into(),
            line: 1,
            command: "git push origin dev rationale: ICM-1".into(),
        });

        assert_eq!(scan.usages[0].primitive, PrimitiveKind::ManualPush);
        assert_eq!(
            check_documented_genuine_need(&required_sources(), &scan.usages, &["ICM-1".into()]),
            Err(BannedPrimitivesFitnessError::HardBannedPrimitive {
                path: "registry/governance-corpora/banned-primitives/reject-manual-push.jsonl"
                    .into(),
                line: 1,
                primitive: PrimitiveKind::ManualPush,
            })
        );
    }

    #[test]
    fn scan_detects_manual_push_inside_fence() {
        let scan = scan_agent_instruction_file(
            "AGENTS.md",
            "<!-- agent-instructions:start -->\nrun git push origin dev\n<!-- agent-instructions:end -->",
        )
        .expect("scan succeeds");

        assert_eq!(scan.usages.len(), 2);
        assert_eq!(scan.usages[0].primitive, PrimitiveKind::ManualPush);
        assert_eq!(scan.usages[1].primitive, PrimitiveKind::DirectVcs);
    }

    #[test]
    fn scan_detects_git_push_with_global_option_as_hard_banned() {
        let scan = scan_agent_instruction_file(
            "AGENTS.md",
            "<!-- agent-instructions:start -->\nrun git -C . push origin dev rationale: ICM-1\n<!-- agent-instructions:end -->",
        )
        .expect("scan succeeds");

        assert_eq!(scan.usages[0].primitive, PrimitiveKind::ManualPush);
        assert_eq!(
            check_documented_genuine_need(&required_sources(), &scan.usages, &["ICM-1".into()]),
            Err(BannedPrimitivesFitnessError::HardBannedPrimitive {
                path: "AGENTS.md".into(),
                line: 2,
                primitive: PrimitiveKind::ManualPush,
            })
        );
    }

    #[test]
    fn scan_detects_spaced_git_push_as_hard_banned() {
        let scan = scan_agent_instruction_file(
            "AGENTS.md",
            "<!-- agent-instructions:start -->\nrun git   push origin dev rationale: ICM-1\n<!-- agent-instructions:end -->",
        )
        .expect("scan succeeds");

        assert_eq!(scan.usages[0].primitive, PrimitiveKind::ManualPush);
    }

    #[test]
    fn scan_detects_spaced_gh_pr_merge_as_hard_banned() {
        let scan = scan_agent_instruction_file(
            "AGENTS.md",
            "<!-- agent-instructions:start -->\nrun gh   pr   merge 123 rationale: ICM-1\n<!-- agent-instructions:end -->",
        )
        .expect("scan succeeds");

        assert_eq!(scan.usages[0].primitive, PrimitiveKind::ForgeMerge);
        assert_eq!(
            check_documented_genuine_need(&required_sources(), &scan.usages, &["ICM-1".into()]),
            Err(BannedPrimitivesFitnessError::HardBannedPrimitive {
                path: "AGENTS.md".into(),
                line: 2,
                primitive: PrimitiveKind::ForgeMerge,
            })
        );
    }

    #[test]
    fn scan_detects_worktree_add_branch_as_hard_banned() {
        let scan = scan_agent_instruction_file(
            "AGENTS.md",
            "<!-- agent-instructions:start -->\nrun git worktree add -b feature ../feature rationale: ICM-1\n<!-- agent-instructions:end -->",
        )
        .expect("scan succeeds");

        assert_eq!(scan.usages[0].primitive, PrimitiveKind::ManualBranch);
        assert_eq!(
            check_documented_genuine_need(&required_sources(), &scan.usages, &["ICM-1".into()]),
            Err(BannedPrimitivesFitnessError::HardBannedPrimitive {
                path: "AGENTS.md".into(),
                line: 2,
                primitive: PrimitiveKind::ManualBranch,
            })
        );
    }

    #[test]
    fn scan_rejects_unterminated_fence() {
        assert_eq!(
            scan_agent_instruction_file(
                "AGENTS.md",
                "<!-- agent-instructions:start -->\nicm recall"
            )
            .unwrap_err(),
            "AGENTS.md: unterminated agent-instructions fence"
        );
    }

    #[test]
    fn rejects_undocumented_direct_primitive() {
        let usages = [usage(PrimitiveKind::DirectVcs, None)];

        assert_eq!(
            check_documented_genuine_need(&required_sources(), &usages, &[]),
            Err(BannedPrimitivesFitnessError::UnjustifiedDirectPrimitive {
                path: "AGENTS.md".into(),
                line: 9,
                primitive: PrimitiveKind::DirectVcs,
            })
        );
    }

    #[test]
    fn accepts_known_documented_genuine_need() {
        let usages = [usage(PrimitiveKind::DirectForge, Some("01ABC"))];
        let report = check_documented_genuine_need(&required_sources(), &usages, &["01ABC".into()])
            .expect("known rationale passes");

        assert_eq!(report.usages_checked, 1);
        assert_eq!(report.documented_exceptions, 1);
    }

    fn required_sources() -> Vec<AgentInstructionSource> {
        REQUIRED_ROOT_AGENT_SOURCES
            .iter()
            .map(|path| AgentInstructionSource {
                path: (*path).into(),
                fence_count: 1,
                rewrite_verified: true,
            })
            .collect()
    }

    fn usage(primitive: PrimitiveKind, rationale: Option<&str>) -> PrimitiveUsage {
        PrimitiveUsage {
            path: "AGENTS.md".into(),
            line: 9,
            primitive,
            icm_rationale: rationale.map(str::to_string),
            context: primitive.as_str().into(),
        }
    }
}
