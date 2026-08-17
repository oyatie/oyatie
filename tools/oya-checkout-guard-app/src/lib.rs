#![forbid(unsafe_code)]

use std::path::{Component, Path, PathBuf};

use serde_json::Value;

const DENY_REASON: &str = "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867";
const MAX_NESTED_COMMAND_DEPTH: usize = 32;
/// Subcommand placeholder used when a `-C`/`--git-dir`/`--work-tree` target is a
/// dynamic expansion that could swallow the real subcommand (review #685 r10).
/// Carries a `${…}` sigil so the subcommand fail-closed check denies it.
const UNRESOLVED_TARGET_SENTINEL: &str = "${__unresolved__}";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionInput {
    pub command: String,
    pub session_cwd: PathBuf,
    pub canonical_checkout: Option<PathBuf>,
    pub process_env: Vec<(String, String)>,
}

pub fn decide(input: DecisionInput) -> Decision {
    let Some(canonical_checkout) = input.canonical_checkout else {
        return Decision::Allow;
    };
    let canonical_checkout = normalize_path(&canonical_checkout);
    let session_cwd = normalize_path(&input.session_cwd);

    let initial_cwd = CwdState::Known(session_cwd);
    let base_git_env = GitEnv::from_assignments(&input.process_env, &initial_cwd);

    decide_with_context(
        &input.command,
        initial_cwd,
        &canonical_checkout,
        &base_git_env,
        0,
    )
}

fn decide_with_context(
    command: &str,
    initial_cwd: CwdState,
    canonical_checkout: &Path,
    base_git_env: &GitEnv,
    depth: usize,
) -> Decision {
    // Fail CLOSED on recursion-depth exhaustion (review #685 r4). Reaching this
    // depth means a wrapper/shell/substitution nest deeper than any legitimate
    // command (real nesting is <= 4; the budget is 32) — refuse rather than let
    // the unevaluated deeper command fall through to ALLOW. Zero false-positive
    // risk: no legitimate invocation nests 32 layers.
    if depth >= MAX_NESTED_COMMAND_DEPTH {
        return Decision::Deny {
            reason: DENY_REASON.to_owned(),
        };
    }
    // Resolve statically-determinable shell expansions BEFORE walking (review
    // #685 r5, founder directive): ANSI-C `$'…'`, `${x:-default}`, same-line
    // `var=value` references, and `echo`/`printf` command substitutions all
    // synthesise a command word/subcommand the literal tokenizer would miss
    // (e.g. `git -C <canon> $'reset' --hard`, `eval $(echo git … reset)`).
    // Only unambiguous forms are rewritten; unknown `$VAR`/`$(prog)` are left
    // byte-identical so legitimate commands never mis-expand (fail-safe).
    let normalized = normalize_static_expansions(command);
    let command: &str = &normalized;
    {
        for nested_command in extract_command_substitutions(command) {
            let nested_decision = decide_with_context(
                &nested_command,
                initial_cwd.clone(),
                canonical_checkout,
                base_git_env,
                depth.saturating_add(1),
            );
            if matches!(nested_decision, Decision::Deny { .. }) {
                return nested_decision;
            }
        }
    }

    let mut current_cwd = initial_cwd;
    let mut previous_cwd = None;
    let mut directory_stack = Vec::new();
    let mut subshell_stack = Vec::new();
    let mut command_start_cwd = current_cwd.clone();
    let mut command_in_pipeline = false;
    let mut command_cwd_scoped = false;
    let tokens = shell_tokens(command);
    let mut command_position = true;
    let mut env_prefix = false;
    let mut pending_git_env = base_git_env.clone();
    let mut skip_env_option_value = false;
    let mut env_chdir_next = false;
    let mut skip_redirection_value = false;
    let mut skip_function_name = false;
    let mut command_prefix = None;
    let mut skip_command_prefix_option_value = false;

    for (index, token) in tokens.iter().enumerate() {
        let ShellToken::Word(word) = token else {
            if let ShellToken::Separator(kind) = token {
                match kind {
                    SeparatorKind::SubshellStart => {
                        if command_cwd_scoped || command_in_pipeline {
                            current_cwd = command_start_cwd.clone();
                        }
                        subshell_stack.push((
                            current_cwd.clone(),
                            previous_cwd.clone(),
                            directory_stack.clone(),
                        ));
                    }
                    SeparatorKind::SubshellEnd => {
                        if let Some((parent_cwd, parent_previous_cwd, parent_directory_stack)) =
                            subshell_stack.pop()
                        {
                            current_cwd = parent_cwd;
                            previous_cwd = parent_previous_cwd;
                            directory_stack = parent_directory_stack;
                        } else {
                            current_cwd = CwdState::Unknown;
                            previous_cwd = None;
                            directory_stack.clear();
                        }
                    }
                    _ => {
                        if command_cwd_scoped || command_in_pipeline || kind.resets_parent_cwd() {
                            current_cwd = command_start_cwd.clone();
                        }
                        command_in_pipeline = *kind == SeparatorKind::Pipe;
                    }
                }
                command_position = true;
                command_start_cwd = current_cwd.clone();
                if !matches!(kind, SeparatorKind::Pipe) {
                    command_in_pipeline = false;
                }
                command_cwd_scoped = false;
            }
            env_prefix = false;
            pending_git_env = base_git_env.clone();
            skip_env_option_value = false;
            env_chdir_next = false;
            skip_redirection_value = false;
            skip_function_name = false;
            command_prefix = None;
            skip_command_prefix_option_value = false;
            continue;
        };

        if !command_position {
            continue;
        }

        if skip_redirection_value {
            skip_redirection_value = false;
            continue;
        }

        if skip_function_name {
            skip_function_name = false;
            continue;
        }

        if skip_command_prefix_option_value {
            skip_command_prefix_option_value = false;
            continue;
        }

        if is_shell_redirection(word) {
            skip_redirection_value = redirection_consumes_next(word);
            continue;
        }

        if skip_env_option_value {
            if env_chdir_next {
                current_cwd = resolve_target_path(&current_cwd, word);
                command_cwd_scoped = true;
                env_chdir_next = false;
            }
            skip_env_option_value = false;
            continue;
        }

        if env_prefix {
            if let Some((name, value)) = parse_env_assignment(word) {
                pending_git_env.record_assignment(name, value, &current_cwd);
                continue;
            }
            if let Some(script) = env_split_inline_command(&tokens, index, word) {
                if depth < MAX_NESTED_COMMAND_DEPTH {
                    let nested_decision = decide_with_context(
                        &script,
                        current_cwd.clone(),
                        canonical_checkout,
                        &pending_git_env,
                        depth.saturating_add(1),
                    );
                    if matches!(nested_decision, Decision::Deny { .. }) {
                        return nested_decision;
                    }
                }
                command_position = false;
                continue;
            }
            if env_split_string_consumes_next(word) {
                let Some(script) = env_split_next_command(&tokens, index) else {
                    command_position = false;
                    continue;
                };
                if depth < MAX_NESTED_COMMAND_DEPTH {
                    let nested_decision = decide_with_context(
                        &script,
                        current_cwd.clone(),
                        canonical_checkout,
                        &pending_git_env,
                        depth.saturating_add(1),
                    );
                    if matches!(nested_decision, Decision::Deny { .. }) {
                        return nested_decision;
                    }
                }
                command_position = false;
                continue;
            }
            if let Some(raw_path) = env_chdir_inline_value(word) {
                current_cwd = resolve_target_path(&current_cwd, raw_path);
                command_cwd_scoped = true;
                continue;
            }
            if is_env_attached_value_option(word) {
                continue;
            }
            if env_option_consumes_next(word) {
                env_chdir_next = env_chdir_option_consumes_next(word);
                skip_env_option_value = true;
                continue;
            }
            if is_env_option_flag(word) {
                continue;
            }
            env_prefix = false;
        }

        if let Some(prefix) = command_prefix {
            if shell_builtin_prefix_option_consumes_next(prefix, word) {
                skip_command_prefix_option_value = true;
                continue;
            }
            if is_shell_builtin_prefix_option(prefix, word) {
                continue;
            }
            command_prefix = None;
        }

        if command_basename_is(word, "env") {
            env_prefix = true;
            continue;
        }
        if word == "command" {
            command_prefix = Some(CommandPrefix::Command);
            continue;
        }
        if word == "exec" {
            command_prefix = Some(CommandPrefix::Exec);
            continue;
        }
        if word == "builtin" {
            command_prefix = Some(CommandPrefix::Builtin);
            continue;
        }
        if word == "function" {
            skip_function_name = true;
            continue;
        }
        if is_shell_reserved_word(word) {
            continue;
        }
        if word == "eval" {
            if depth < MAX_NESTED_COMMAND_DEPTH {
                let script = words_until_separator(&tokens, index.saturating_add(1)).join(" ");
                let nested_decision = decide_with_context(
                    &script,
                    current_cwd.clone(),
                    canonical_checkout,
                    base_git_env,
                    depth.saturating_add(1),
                );
                if matches!(nested_decision, Decision::Deny { .. }) {
                    return nested_decision;
                }
            }
            command_position = false;
            continue;
        }
        if word == "cd" {
            apply_cd_target(
                parse_cd_target(&tokens, index, &current_cwd),
                &mut current_cwd,
                &mut previous_cwd,
            );
            command_position = false;
            continue;
        }
        if word == "pushd" {
            if let Some(cwd) = known_cwd(&current_cwd) {
                directory_stack.push(cwd.to_path_buf());
            }
            apply_cd_target(
                parse_pushd_target(&tokens, index, &current_cwd),
                &mut current_cwd,
                &mut previous_cwd,
            );
            command_position = false;
            continue;
        }
        if word == "popd" {
            previous_cwd = known_cwd(&current_cwd).map(Path::to_path_buf);
            current_cwd = directory_stack
                .pop()
                .map_or(CwdState::Unknown, CwdState::Known);
            command_position = false;
            continue;
        }
        if let Some(script) = shell_wrapper_script(&tokens, index, word) {
            if depth < MAX_NESTED_COMMAND_DEPTH {
                let nested_decision = decide_with_context(
                    &script,
                    current_cwd.clone(),
                    canonical_checkout,
                    base_git_env,
                    depth.saturating_add(1),
                );
                if matches!(nested_decision, Decision::Deny { .. }) {
                    return nested_decision;
                }
            }
            command_position = false;
            continue;
        }
        // A literal `git`, OR a command-name that still carries an unresolved
        // expansion after static normalization (`$g`, `${x:+…}`, …) which COULD
        // be git — evaluate the following tokens as a git invocation. The
        // residual-expansion case fails closed below if it forms a canonical
        // mutation (review #685 r6 F2 command-name case).
        if command_basename_is(word, "git") || has_unresolved_expansion(word) {
            let Some(invocation) =
                parse_git_invocation(&tokens, index, &current_cwd, &pending_git_env)
            else {
                command_position = false;
                continue;
            };
            if let Some(alias_script) = &invocation.alias_script {
                if depth < MAX_NESTED_COMMAND_DEPTH {
                    let nested_decision = decide_with_context(
                        alias_script,
                        invocation.repo_context.clone(),
                        canonical_checkout,
                        &pending_git_env,
                        depth.saturating_add(1),
                    );
                    if matches!(nested_decision, Decision::Deny { .. }) {
                        return nested_decision;
                    }
                }
            }
            // Fail closed when the subcommand token still carries an unresolved
            // expansion/transformation after normalization — `$@`/positional
            // params, `${x:+…}`, `$r`, brace/glob, backtick (review #685 r6/r9).
            // A literal non-verb token (e.g. a path arg of a phantom `git` match
            // from a wrapper scan) is NOT flagged — only transformation sigils,
            // so `grep -r git /path` stays ALLOW. ANSI-C numeric escapes and
            // line-continuations are decoded/joined upstream in normalization,
            // so they arrive here as the real verb and hit is_blocked_operation.
            // An unresolved subcommand expansion (`git $P`, `git "${P[@]}"`,
            // `git $@` from an eval-/local-/array-hidden binding) could carry
            // BOTH the mutating verb AND a `-C <canonical>` retarget the guard
            // cannot see — so it must fail closed on the TARGET as well, not
            // just the operation (review #685 r12: convergent fail-closed —
            // an argv-position expansion that can't be proven free of a
            // canonical -C + mutating verb is denied regardless of cwd). This
            // is the general rule that subsumes the per-spelling binding
            // collectors; literal non-verb tokens stay ALLOW (grep -r git /path).
            let subcommand_unresolved = has_unresolved_expansion(&invocation.subcommand);
            // Only a `$`/backtick subcommand expansion can carry a HIDDEN
            // `-C <canonical>` retarget (its text is arbitrary); a brace/glob
            // subcommand only obfuscates the VERB in place and cannot move the
            // target (review #685 r12). So the former forces fail-closed on the
            // target; the latter leaves the explicit/cwd target intact — keeping
            // `git -C <worktree> {reset,} --hard` an ALLOWED worktree mutation.
            let subcommand_retargetable =
                subcommand_carries_value_expansion(&invocation.subcommand);
            let blocked_operation = is_blocked_operation(&invocation.subcommand, &invocation.args)
                || subcommand_unresolved;
            let work_tree_blocked = match &invocation.target {
                CwdState::Known(target) => path_is_within(target, canonical_checkout),
                CwdState::Unknown => true,
            };
            let git_dir_blocked = invocation.git_dir.as_ref().is_some_and(|git_dir| {
                git_dir_blocks_target(
                    git_dir,
                    &invocation.target,
                    invocation.explicit_work_tree,
                    canonical_checkout,
                )
            });
            let repo_context_blocked = invocation.git_dir.is_none()
                && matches!(
                    &invocation.repo_context,
                    CwdState::Known(repo_context) if path_is_within(repo_context, canonical_checkout)
                );
            let blocked_target = subcommand_retargetable
                || work_tree_blocked
                || git_dir_blocked
                || repo_context_blocked;
            if blocked_operation && blocked_target {
                return Decision::Deny {
                    reason: DENY_REASON.to_owned(),
                };
            }
        }
        if let Some((name, value)) = parse_env_assignment(word) {
            pending_git_env.record_assignment(name, value, &current_cwd);
            continue;
        }
        // Known transparent process-runner wrappers: skip the wrapper's own
        // options, then recurse on the remainder as a nested command.  This
        // covers nohup/nice/timeout/stdbuf/setsid/ionice/chrt/taskset/watch/
        // chronic/sudo/doas (pass-through wrappers) and xargs/parallel (which
        // feed argv to a subprocess and may supply args the parser cannot see).
        if let Some(remainder) = transparent_wrapper_remainder(&tokens, index, word) {
            if depth < MAX_NESTED_COMMAND_DEPTH {
                let nested_decision = decide_with_context(
                    &remainder,
                    current_cwd.clone(),
                    canonical_checkout,
                    &pending_git_env,
                    depth.saturating_add(1),
                );
                if matches!(nested_decision, Decision::Deny { .. }) {
                    return nested_decision;
                }
            }
            command_position = false;
            continue;
        }
        // Default-closed scan-through (review #685 r2; FRIC-022/FRIC-1781062867):
        // an UNRECOGNISED leading command word is a wrapper we do not model. A
        // hardcoded wrapper allowlist always lags the wrapper space (firejail,
        // flock, systemd-run, cpulimit, runuser, eatmydata, proxychains, ...),
        // so instead of silently ALLOWing, scan this command's remaining words
        // for a git invocation and let the normal decision recurse. Allowlist
        // membership is thus non-load-bearing for safety: an unknown wrapper
        // carrying a canonical-targeted mutating git op fails closed.
        if !command_never_executes_arguments(word)
            && let Some(remainder) = unmodelled_wrapper_remainder(&tokens, index)
        {
            if depth < MAX_NESTED_COMMAND_DEPTH {
                let nested_decision = decide_with_context(
                    &remainder,
                    current_cwd.clone(),
                    canonical_checkout,
                    &pending_git_env,
                    depth.saturating_add(1),
                );
                if matches!(nested_decision, Decision::Deny { .. }) {
                    return nested_decision;
                }
            }
        }
        command_position = false;
    }

    Decision::Allow
}

/// From an unrecognised leading command word, find the first subsequent `git`
/// invocation within the SAME command (before any command separator) and return
/// the remainder beginning at that `git` token. Powers the default-closed
/// fall-through so wrappers the guard does not explicitly model cannot smuggle a
/// mutating git op past it merely by being unknown. Returns None when no `git`
/// token precedes the next separator (genuine non-git commands stay ALLOWed).
/// Commands that provably never execute a subsequent token as a command — their
/// arguments are pure data (printed, evaluated as a string, or ignored). These
/// are EXEMPT from the default-closed scan-through so a textual `git` argument
/// (`echo git switch ...`) is not mistaken for an executed mutation. The set is
/// intentionally tiny and audited; everything NOT listed is treated as a
/// possibly-executing wrapper and fails closed. Adding a name here is a
/// security decision — the command must be incapable of running its arguments.
fn command_never_executes_arguments(command_word: &str) -> bool {
    matches!(
        command_basename(command_word),
        Some("echo" | "printf" | "print" | "true" | "false" | ":" | "pwd")
    )
}

/// The remainder of an unrecognised leading command word's invocation: every
/// word after the wrapper up to the next command separator, re-quoted so word
/// boundaries (and therefore embedded `sh -c '<body>'` scripts and
/// `GIT_DIR=<path>` env-context) survive a re-tokenisation. Powers the
/// default-closed fall-through: an unmodelled wrapper's full argument tail is
/// re-evaluated as a nested command, so the existing shell-interpreter,
/// env-assignment and git-context machinery decides it. The previous git-token
/// slice anchoring was unsound — it discarded preceding env/quote context, so
/// `<wrapper> sh -c '<mut>'` and `<wrapper> GIT_DIR=<canon>/.git git <mut>`
/// leaked (review #685 r3). Returns None when nothing follows the wrapper.
fn unmodelled_wrapper_remainder(tokens: &[ShellToken], start: usize) -> Option<String> {
    let words = words_until_separator(tokens, start.checked_add(1)?);
    if words.is_empty() {
        return None;
    }
    Some(
        words
            .iter()
            .map(|word| requote_word(word))
            .collect::<Vec<_>>()
            .join(" "),
    )
}

/// Re-quote a single already-unquoted shell word so re-tokenising the joined
/// remainder reproduces the original word boundaries. Safe-charset words are
/// emitted bare; anything else is single-quoted with embedded single quotes
/// escaped (`'\''`), keeping `sh -c 'git … reset --hard'` one argument across
/// the default-closed recursion.
fn requote_word(word: &str) -> String {
    let safe = !word.is_empty()
        && word.chars().all(|ch| {
            ch.is_ascii_alphanumeric()
                || matches!(
                    ch,
                    '-' | '_' | '/' | '=' | '.' | ':' | ',' | '@' | '+' | '%'
                )
        });
    if safe {
        word.to_owned()
    } else {
        format!("'{}'", word.replace('\'', "'\\''"))
    }
}

pub fn extract_command_from_hook_payload(payload: &str) -> Option<String> {
    let value: Value = serde_json::from_str(payload).ok()?;
    value
        .get("tool_input")
        .and_then(|tool_input| tool_input.get("command"))
        .and_then(Value::as_str)
        .or_else(|| {
            value
                .get("input")
                .and_then(|input| input.get("command"))
                .and_then(Value::as_str)
        })
        .or_else(|| {
            value
                .get("parameters")
                .and_then(|parameters| parameters.get("command"))
                .and_then(Value::as_str)
        })
        .or_else(|| value.get("command").and_then(Value::as_str))
        .map(str::to_owned)
}

pub fn default_canonical_checkout(repo_root: &Path, git_common_dir: &Path) -> Option<PathBuf> {
    if git_common_dir == Path::new(".git") {
        Some(repo_root.to_path_buf())
    } else {
        let common_dir = if git_common_dir.is_absolute() {
            git_common_dir.to_path_buf()
        } else {
            repo_root.join(git_common_dir)
        };
        if common_dir.file_name() == Some(Path::new(".git").as_os_str()) {
            common_dir.parent().map(normalize_path)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CwdState {
    Known(PathBuf),
    Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct GitEnv {
    git_dir: Option<CwdState>,
    work_tree: Option<CwdState>,
    vars: Vec<(String, String)>,
}

impl GitEnv {
    fn from_assignments(assignments: &[(String, String)], current_cwd: &CwdState) -> Self {
        let mut git_env = GitEnv::default();
        for (name, value) in assignments {
            git_env.record_assignment(name, value, current_cwd);
        }
        git_env
    }

    fn record_assignment(&mut self, name: &str, value: &str, current_cwd: &CwdState) {
        self.vars.retain(|(existing, _)| existing != name);
        self.vars.push((name.to_owned(), value.to_owned()));
        let target = if value.is_empty() {
            CwdState::Unknown
        } else {
            resolve_target_path(current_cwd, value)
        };
        match name {
            "GIT_DIR" => self.git_dir = Some(target),
            "GIT_WORK_TREE" => self.work_tree = Some(target),
            _ => {}
        }
    }

    fn value(&self, name: &str) -> Option<&str> {
        self.vars
            .iter()
            .rev()
            .find_map(|(existing, value)| (existing == name).then_some(value.as_str()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandPrefix {
    Command,
    Exec,
    Builtin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CdTarget {
    Known(PathBuf),
    Previous,
    Unknown,
}

fn apply_cd_target(
    target: CdTarget,
    current_cwd: &mut CwdState,
    previous_cwd: &mut Option<PathBuf>,
) {
    match target {
        CdTarget::Known(next_cwd) => {
            *previous_cwd = known_cwd(current_cwd).map(Path::to_path_buf);
            *current_cwd = CwdState::Known(next_cwd);
        }
        CdTarget::Previous => {
            let old_current = known_cwd(current_cwd).map(Path::to_path_buf);
            match previous_cwd.take() {
                Some(previous) => *current_cwd = CwdState::Known(previous),
                None => *current_cwd = CwdState::Unknown,
            }
            *previous_cwd = old_current;
        }
        CdTarget::Unknown => {
            *previous_cwd = known_cwd(current_cwd).map(Path::to_path_buf);
            *current_cwd = CwdState::Unknown;
        }
    }
}

fn known_cwd(current_cwd: &CwdState) -> Option<&Path> {
    match current_cwd {
        CwdState::Known(path) => Some(path),
        CwdState::Unknown => None,
    }
}

fn parse_cd_target(tokens: &[ShellToken], cd_index: usize, current_cwd: &CwdState) -> CdTarget {
    let Some(mut index) = cd_index.checked_add(1) else {
        return CdTarget::Unknown;
    };
    while index < tokens.len() {
        match &tokens[index] {
            ShellToken::Word(word) => {
                if word == "--" {
                    index = index.saturating_add(1);
                    continue;
                }
                if word == "-" {
                    return CdTarget::Previous;
                }
                if is_cd_option(word) {
                    index = index.saturating_add(1);
                    continue;
                }
                return match resolve_target_path(current_cwd, word) {
                    CwdState::Known(path) => CdTarget::Known(path),
                    CwdState::Unknown => CdTarget::Unknown,
                };
            }
            ShellToken::Separator(_) => return CdTarget::Unknown,
        }
    }
    CdTarget::Unknown
}

fn parse_pushd_target(
    tokens: &[ShellToken],
    pushd_index: usize,
    current_cwd: &CwdState,
) -> CdTarget {
    let Some(mut index) = pushd_index.checked_add(1) else {
        return CdTarget::Unknown;
    };
    while index < tokens.len() {
        match &tokens[index] {
            ShellToken::Word(word) => {
                if word == "--" || word == "-n" {
                    index = index.saturating_add(1);
                    continue;
                }
                if is_directory_stack_index(word) {
                    return CdTarget::Unknown;
                }
                return match resolve_target_path(current_cwd, word) {
                    CwdState::Known(path) => CdTarget::Known(path),
                    CwdState::Unknown => CdTarget::Unknown,
                };
            }
            ShellToken::Separator(_) => return CdTarget::Unknown,
        }
    }
    CdTarget::Unknown
}

fn is_cd_option(word: &str) -> bool {
    if word == "-" || word == "--" {
        return false;
    }
    matches!(word, "--logical" | "--physical")
        || word.strip_prefix('-').is_some_and(|rest| {
            !rest.is_empty() && !rest.starts_with('-') && rest.chars().all(is_cd_option_char)
        })
}

fn is_cd_option_char(ch: char) -> bool {
    matches!(ch, 'L' | 'P' | 'e' | '@')
}

fn is_directory_stack_index(word: &str) -> bool {
    word.strip_prefix('+')
        .or_else(|| word.strip_prefix('-'))
        .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|ch| ch.is_ascii_digit()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GitInvocation {
    target: CwdState,
    git_dir: Option<CwdState>,
    repo_context: CwdState,
    explicit_work_tree: bool,
    subcommand: String,
    args: Vec<String>,
    alias_script: Option<String>,
}

fn parse_git_invocation(
    tokens: &[ShellToken],
    git_index: usize,
    session_cwd: &CwdState,
    git_env: &GitEnv,
) -> Option<GitInvocation> {
    let mut index = git_index.checked_add(1)?;
    let mut option_cwd = session_cwd.clone();
    let mut target = git_env
        .work_tree
        .clone()
        .or_else(|| git_env.git_dir.as_ref().map(checkout_from_resolved_git_dir))
        .unwrap_or_else(|| session_cwd.clone());
    let mut explicit_work_tree = git_env.work_tree.is_some();
    let mut git_dir = git_env.git_dir.clone();
    let mut aliases = git_env_config_aliases(git_env);

    while index < tokens.len() {
        let ShellToken::Word(token) = &tokens[index] else {
            break;
        };
        if is_shell_redirection(token) {
            index = index.saturating_add(if redirection_consumes_next(token) {
                2
            } else {
                1
            });
            continue;
        }
        if token == "-C" {
            let path_index = index.checked_add(1)?;
            let Some(ShellToken::Word(raw_path)) = tokens.get(path_index) else {
                return None;
            };
            // A `-C` target that still carries an unresolved expansion could
            // SWALLOW the subcommand at runtime (`git -C $@`, where `$@` carries
            // `<canon> reset --hard`) — fail closed instead of returning None →
            // ALLOW (review #685 r10). Synthesize a sentinel mutating verb so
            // the caller denies; the target is already dynamic/Unknown.
            if has_unresolved_expansion(raw_path) {
                return Some(GitInvocation {
                    target: CwdState::Unknown,
                    git_dir: git_dir.map(normalize_cwd_state),
                    repo_context: CwdState::Unknown,
                    explicit_work_tree,
                    subcommand: UNRESOLVED_TARGET_SENTINEL.to_owned(),
                    args: Vec::new(),
                    alias_script: None,
                });
            }
            option_cwd = resolve_target_path(&option_cwd, raw_path);
            if !explicit_work_tree {
                target = option_cwd.clone();
            }
            index = path_index.saturating_add(1);
            continue;
        }
        if token == "--work-tree" {
            let path_index = index.checked_add(1)?;
            let Some(ShellToken::Word(raw_path)) = tokens.get(path_index) else {
                return None;
            };
            target = resolve_target_path(&option_cwd, raw_path);
            explicit_work_tree = true;
            index = path_index.saturating_add(1);
            continue;
        }
        if let Some(raw_path) = token.strip_prefix("--work-tree=") {
            if raw_path.is_empty() {
                return None;
            }
            target = resolve_target_path(&option_cwd, raw_path);
            explicit_work_tree = true;
            index = index.saturating_add(1);
            continue;
        }
        if token == "--git-dir" {
            let path_index = index.checked_add(1)?;
            let Some(ShellToken::Word(raw_path)) = tokens.get(path_index) else {
                return None;
            };
            git_dir = Some(resolve_target_path(&option_cwd, raw_path));
            if !explicit_work_tree {
                target = checkout_from_git_dir(&option_cwd, raw_path);
            }
            index = path_index.saturating_add(1);
            continue;
        }
        if let Some(raw_path) = token.strip_prefix("--git-dir=") {
            if raw_path.is_empty() {
                return None;
            }
            git_dir = Some(resolve_target_path(&option_cwd, raw_path));
            if !explicit_work_tree {
                target = checkout_from_git_dir(&option_cwd, raw_path);
            }
            index = index.saturating_add(1);
            continue;
        }
        if token == "-c" {
            let config_index = index.checked_add(1)?;
            let Some(ShellToken::Word(config)) = tokens.get(config_index) else {
                return None;
            };
            record_git_alias_config(config, &mut aliases);
            index = config_index.saturating_add(1);
            continue;
        }
        if token == "--config-env" {
            let config_index = index.checked_add(1)?;
            let Some(ShellToken::Word(config)) = tokens.get(config_index) else {
                return None;
            };
            record_git_alias_config_env(config, &mut aliases, git_env);
            index = config_index.saturating_add(1);
            continue;
        }
        if let Some(config) = token.strip_prefix("--config-env=") {
            record_git_alias_config_env(config, &mut aliases, git_env);
            index = index.saturating_add(1);
            continue;
        }
        if git_global_option_consumes_next(token) {
            index = index.saturating_add(2);
            continue;
        }
        if token.starts_with('-') {
            index = index.saturating_add(1);
            continue;
        }

        let alias = resolve_git_alias(token, &aliases);
        let subcommand = alias
            .and_then(GitAliasTarget::subcommand)
            .unwrap_or(token)
            .to_owned();
        let alias_script = alias
            .and_then(GitAliasTarget::shell_script)
            .map(str::to_owned);
        let mut args = alias.map(GitAliasTarget::args).unwrap_or(&[]).to_vec();
        args.extend(words_until_separator(tokens, index.saturating_add(1)));

        return Some(GitInvocation {
            target: normalize_cwd_state(target),
            git_dir: git_dir.map(normalize_cwd_state),
            repo_context: normalize_cwd_state(option_cwd),
            explicit_work_tree,
            subcommand,
            args,
            alias_script,
        });
    }

    None
}

fn checkout_from_git_dir(option_cwd: &CwdState, raw_path: &str) -> CwdState {
    let git_dir = resolve_target_path(option_cwd, raw_path);
    checkout_from_resolved_git_dir(&git_dir)
}

fn checkout_from_resolved_git_dir(git_dir: &CwdState) -> CwdState {
    let Some(path) = known_cwd(&git_dir) else {
        return CwdState::Unknown;
    };
    if path.file_name() == Some(Path::new(".git").as_os_str()) {
        path.parent()
            .map(normalize_path)
            .map(CwdState::Known)
            .unwrap_or(CwdState::Unknown)
    } else {
        CwdState::Known(normalize_path(path))
    }
}

fn normalize_cwd_state(cwd: CwdState) -> CwdState {
    match cwd {
        CwdState::Known(path) => CwdState::Known(normalize_path(&path)),
        CwdState::Unknown => CwdState::Unknown,
    }
}

fn git_dir_blocks_target(
    git_dir: &CwdState,
    target: &CwdState,
    explicit_work_tree: bool,
    canonical_checkout: &Path,
) -> bool {
    let CwdState::Known(git_dir) = git_dir else {
        return true;
    };
    if !path_is_within(git_dir, canonical_checkout) {
        return false;
    }
    if explicit_work_tree
        && is_linked_worktree_git_dir(git_dir, canonical_checkout)
        && matches!(target, CwdState::Known(target) if !path_is_within(target, canonical_checkout))
    {
        return false;
    }
    true
}

fn is_linked_worktree_git_dir(git_dir: &Path, canonical_checkout: &Path) -> bool {
    let worktrees_dir = canonical_checkout.join(".git").join("worktrees");
    git_dir
        .strip_prefix(&worktrees_dir)
        .ok()
        .and_then(|rest| rest.components().next())
        .is_some()
}

fn env_option_consumes_next(word: &str) -> bool {
    matches!(word, "-u" | "--unset" | "-C" | "--chdir" | "-P")
}

fn env_split_string_consumes_next(word: &str) -> bool {
    matches!(word, "-S" | "--split-string")
}

fn env_split_next_command(tokens: &[ShellToken], split_option_index: usize) -> Option<String> {
    let script_index = split_option_index.checked_add(1)?;
    let Some(ShellToken::Word(script)) = tokens.get(script_index) else {
        return None;
    };
    Some(env_split_command_with_args(
        script,
        tokens,
        script_index.saturating_add(1),
    ))
}

fn env_split_inline_command(
    tokens: &[ShellToken],
    split_option_index: usize,
    word: &str,
) -> Option<String> {
    let script = attached_env_option_value(word, "-S")
        .or_else(|| word.strip_prefix("--split-string="))
        .filter(|value| !value.is_empty())?;
    Some(env_split_command_with_args(
        script,
        tokens,
        split_option_index.saturating_add(1),
    ))
}

fn env_split_command_with_args(script: &str, tokens: &[ShellToken], args_start: usize) -> String {
    let mut command = script.to_owned();
    let remaining_args = words_until_separator(tokens, args_start);
    if !remaining_args.is_empty() {
        if !command.is_empty() {
            command.push(' ');
        }
        command.push_str(&remaining_args.join(" "));
    }
    command
}

fn env_chdir_option_consumes_next(word: &str) -> bool {
    matches!(word, "-C" | "--chdir")
}

fn env_chdir_inline_value(word: &str) -> Option<&str> {
    attached_env_option_value(word, "-C")
        .or_else(|| word.strip_prefix("--chdir="))
        .and_then(|value| (!value.is_empty()).then_some(value))
}

fn is_env_attached_value_option(word: &str) -> bool {
    attached_env_option_value(word, "-u").is_some()
        || attached_env_option_value(word, "-P").is_some()
}

fn attached_env_option_value<'a>(word: &'a str, option: &str) -> Option<&'a str> {
    word.strip_prefix(option)
        .and_then(|value| (!value.is_empty()).then_some(value))
}

fn is_env_option_flag(word: &str) -> bool {
    matches!(word, "--ignore-environment" | "--null" | "--")
        || word.starts_with("--unset=")
        || word.strip_prefix('-').is_some_and(|rest| {
            !rest.is_empty()
                && !rest.starts_with('-')
                && rest.chars().all(|ch| matches!(ch, '0' | 'i' | 'v'))
        })
}

fn git_global_option_consumes_next(token: &str) -> bool {
    matches!(
        token,
        "-c" | "--namespace" | "--exec-path" | "--super-prefix"
    )
}

fn shell_wrapper_script(
    tokens: &[ShellToken],
    command_index: usize,
    command_word: &str,
) -> Option<String> {
    if !is_shell_interpreter(command_word) {
        return None;
    }

    let args = words_until_separator(tokens, command_index.saturating_add(1));
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == "-c" || short_shell_option_has_c(arg) {
            return args.get(index.saturating_add(1)).cloned();
        }
        if shell_option_consumes_next(arg) {
            index = index.saturating_add(2);
        } else {
            index = index.saturating_add(1);
        }
    }

    None
}

fn is_shell_interpreter(command_word: &str) -> bool {
    command_basename(command_word)
        .is_some_and(|name| matches!(name, "sh" | "bash" | "dash" | "zsh"))
}

/// For known transparent process-runner wrappers, return the remainder of the
/// current simple command (after skipping the wrapper's own option flags) as a
/// joined string so the caller can recurse into it.
///
/// Covers:
///  - pass-through wrappers that exec their argument directly:
///    nohup, nice, setsid, watch, chronic, ionice, chrt, taskset, stdbuf, timeout
///  - privilege-escalation wrappers: sudo, doas
///  - argument-feeding wrappers whose argument IS the git command: xargs, parallel
///
/// Returns `None` if the word is not a recognised transparent wrapper.
fn transparent_wrapper_remainder(
    tokens: &[ShellToken],
    command_index: usize,
    command_word: &str,
) -> Option<String> {
    let name = command_basename(command_word)?;
    match name {
        // Simple pass-through: exec the remaining argv directly.
        // Skip single-letter flags and their attached/next values conservatively:
        // any `-<letter>` that is not `--` ends the flag cluster when a
        // non-flag argument is found.
        "nohup" | "setsid" | "watch" | "chronic" => {
            // These wrappers have few/no options that consume a following value
            // and none that look like git subcommands; skip leading -flag tokens.
            let remainder = skip_flag_args_and_join(tokens, command_index.saturating_add(1));
            Some(remainder)
        }
        // nice/ionice/chrt/taskset: may have option-value pairs; skip them.
        "nice" | "ionice" | "chrt" | "taskset" => {
            let remainder = skip_flag_args_and_join(tokens, command_index.saturating_add(1));
            Some(remainder)
        }
        // stdbuf: has -i/-o/-e options that each consume the next token.
        "stdbuf" => {
            let remainder = skip_flag_args_and_join(tokens, command_index.saturating_add(1));
            Some(remainder)
        }
        // timeout: `timeout [OPTIONS] DURATION COMMAND [ARG]...`
        // DURATION comes before the command; skip flags then the duration token.
        "timeout" => {
            let args = words_until_separator(tokens, command_index.saturating_add(1));
            // Skip leading option flags, then the mandatory duration argument,
            // then join the rest as the nested command.
            let mut skip = 0usize;
            while skip < args.len() && args[skip].starts_with('-') {
                // Options that consume the next token (e.g. --signal, --kill-after)
                if matches!(
                    args[skip].as_str(),
                    "-k" | "--kill-after" | "--signal" | "-s"
                ) {
                    skip = skip.saturating_add(2);
                } else {
                    skip = skip.saturating_add(1);
                }
            }
            // skip the duration argument itself
            skip = skip.saturating_add(1);
            Some(args[skip..].join(" "))
        }
        // sudo/doas: privilege wrappers; skip their option flags then recurse.
        "sudo" | "doas" => {
            let remainder = skip_flag_args_and_join(tokens, command_index.saturating_add(1));
            Some(remainder)
        }
        // xargs: the command to execute is the remaining non-option arguments.
        // `xargs [OPTIONS] [COMMAND [INITIAL-ARGS]]`
        // Skip options (some consume a next token), then join the rest.
        // Append a sentinel argument so that subcommands like `checkout` that
        // require at least one arg to be blocked are treated as blocked: at
        // runtime xargs always supplies at least one argument from stdin.
        "xargs" => {
            let base = skip_flag_args_and_join(tokens, command_index.saturating_add(1));
            if base.is_empty() {
                Some(base)
            } else {
                Some(format!("{base} __xargs_sentinel__"))
            }
        }
        // parallel (GNU parallel): `parallel [OPTIONS] COMMAND ::: ARG...`
        // Everything before `:::` / `::::` delimiters is the command template;
        // strip args after `:::` by joining only up to the first `:::`.
        // Same sentinel logic as xargs: parallel always supplies at least one arg.
        "parallel" => {
            let args = words_until_separator(tokens, command_index.saturating_add(1));
            let command_args: Vec<&str> = args
                .iter()
                .map(String::as_str)
                .take_while(|a| *a != ":::" && *a != "::::")
                .collect();
            let base = command_args.join(" ");
            if base.is_empty() {
                Some(base)
            } else {
                Some(format!("{base} __parallel_sentinel__"))
            }
        }
        _ => None,
    }
}

/// Skip leading flag-like tokens (those starting with `-`) and any token that
/// immediately follows an option known to consume a value argument, then join
/// the remaining tokens with spaces.  This is intentionally conservative:
/// unrecognised options that look like flags are skipped, but if a non-flag
/// argument is reached the rest is joined as-is (i.e. the command + its args).
fn skip_flag_args_and_join(tokens: &[ShellToken], start: usize) -> String {
    let args = words_until_separator(tokens, start);
    let mut index = 0usize;
    while index < args.len() && args[index].starts_with('-') {
        index = index.saturating_add(1);
        // If the previous option is `-e`, `-u`, `-n`, `-P`, `-I`, `-a`, `-j`,
        // `-s`, `-d`, `-k`, `-a`, etc. and the current token does not start
        // with `-`, it may be a value — skip it too if the option was a
        // single-char option cluster ending with a char that consumes next.
        // Rather than enumerate every option, the conservative choice is to
        // keep looping: once we hit a non-flag we stop and include everything.
    }
    // Re-quote so an embedded `sh -c '<body>'` script survives re-tokenisation
    // (review #685 r3 F1: sudo/nohup/eval + sh -c lost quotes via a bare join).
    args[index..]
        .iter()
        .map(|word| requote_word(word))
        .collect::<Vec<_>>()
        .join(" ")
}

fn command_basename_is(command_word: &str, expected: &str) -> bool {
    command_basename(command_word) == Some(expected)
}

fn command_basename(command_word: &str) -> Option<&str> {
    Path::new(command_word)
        .file_name()
        .and_then(|name| name.to_str())
}

fn short_shell_option_has_c(arg: &str) -> bool {
    arg.strip_prefix('-')
        .is_some_and(|rest| !rest.is_empty() && !rest.starts_with('-') && rest.contains('c'))
}

fn shell_option_consumes_next(arg: &str) -> bool {
    matches!(arg, "-o" | "--rcfile" | "--init-file")
}

fn is_shell_builtin_prefix_option(prefix: CommandPrefix, word: &str) -> bool {
    match prefix {
        CommandPrefix::Command => word == "--" || is_command_builtin_flag(word),
        CommandPrefix::Exec => word == "--" || exec_flag_option(word) || attached_exec_argv0(word),
        CommandPrefix::Builtin => word == "--",
    }
}

fn shell_builtin_prefix_option_consumes_next(prefix: CommandPrefix, word: &str) -> bool {
    prefix == CommandPrefix::Exec && exec_argv0_option_consumes_next(word)
}

fn is_command_builtin_flag(word: &str) -> bool {
    word.strip_prefix('-').is_some_and(|rest| {
        !rest.is_empty() && !rest.starts_with('-') && {
            rest.chars().all(|ch| matches!(ch, 'p' | 'v' | 'V'))
        }
    })
}

fn attached_exec_argv0(word: &str) -> bool {
    let Some(rest) = exec_short_option_cluster(word) else {
        return false;
    };
    let Some(a_index) = rest.find('a') else {
        return false;
    };
    rest[..a_index].chars().all(|ch| matches!(ch, 'c' | 'l')) && !rest[a_index + 1..].is_empty()
}

fn exec_argv0_option_consumes_next(word: &str) -> bool {
    let Some(rest) = exec_short_option_cluster(word) else {
        return false;
    };
    rest.ends_with('a')
        && rest[..rest.len().saturating_sub(1)]
            .chars()
            .all(|ch| matches!(ch, 'c' | 'l'))
}

fn exec_flag_option(word: &str) -> bool {
    exec_short_option_cluster(word)
        .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|ch| matches!(ch, 'c' | 'l')))
}

fn exec_short_option_cluster(word: &str) -> Option<&str> {
    word.strip_prefix('-')
        .filter(|rest| !rest.is_empty() && !rest.starts_with('-'))
}

/// Rewrite the statically-determinable shell expansions a literal tokenizer
/// would otherwise miss, leaving every other byte untouched. Conservative by
/// construction: only ANSI-C `$'…'`, `${name}`/`$name` bound to a same-line
/// `name=value`, `${name:-default}` with an unbound name, and `echo`/`printf`
/// command substitutions are resolved. Unknown `$VAR` and non-echo/printf
/// `$(prog)` / backticks are preserved verbatim so legitimate commands never
/// mis-expand (review #685 r5; founder directive 2026-06-10).
fn normalize_static_expansions(command: &str) -> String {
    let bindings = collect_same_line_bindings(command);
    // A same-line non-default `IFS=` reassignment makes unquoted expansions
    // word-SPLIT on attacker-chosen characters (`IFS=x; y=resetx; git … $y`
    // splits `resetx`→`reset`; `IFS=x; p=<canon>x; git -C $p`), defeating a
    // single-word substitution model on BOTH verb and path sides (review #685
    // r8 F4). When present, suppress value-producing expansion entirely so the
    // residual-sigil (verb) and dynamic-path (target) fail-closed rules fire.
    // Iterate to a bounded fixpoint so NESTED substitutions/params resolve
    // (`$(echo $(echo git … reset))`, `${x:-$(echo reset)}`) — a single pass
    // only peels one layer (review #685 r6 F1). Each pass also inlines shell
    // function calls and expands positional params bound by `set --` (review
    // #685 r10: `g(){ git "$@"; }; g -C <canon> reset`, `set -- … ; git $@`).
    // Capped to bound pathological input; whatever survives the cap is caught
    // fail-closed by the residual-expansion check at the git decision.
    let mut current = command.to_owned();
    for _ in 0..8 {
        let inlined = inline_function_calls(&current);
        let ifs_unsafe = same_line_ifs_reassigned(&inlined);
        // Pre-resolve command substitutions / ANSI-C / line-continuation FIRST
        // (empty bindings → `$var`/`$@`/arrays stay literal) so `set -- $(echo
        // …)` and `read … <<< "$(…)"` reassemble before positionals/bindings
        // are collected — otherwise the `$(` subshell tokens corrupt capture
        // (review #685 r13). Assignment-RHS substitutions are re-quoted here so
        // a multi-word value fails closed at its use site.
        // Pre-pass resolves ONLY substitutions/ANSI-C (`substitutions_only`);
        // `${…}`/`$var`/`$@` stay literal so a `${V:-x}` default is NOT eaten
        // before `V=…` is collected (review #685 r17 F2).
        let resolved = expand_with_bindings(&inlined, &[], &[], &[], ifs_unsafe, true);
        let bindings = collect_same_line_bindings(&resolved);
        let positionals = collect_positional_params(&resolved);
        let arrays = collect_array_bindings(&resolved);
        let next = expand_with_bindings(
            &resolved,
            &bindings,
            &positionals,
            &arrays,
            ifs_unsafe,
            false,
        );
        if next == current {
            break;
        }
        current = next;
    }
    current
}

/// Positional parameters bound by a same-line `set -- <words>` (review #685
/// r10). The last `set --` before end wins (shell semantics); used to expand
/// `$@`/`$*`/`$N` so `set -- reset --hard; git -C <canon> $@` resolves to the
/// real verb. An unbound `$@`/`$N` expands to empty (also shell-faithful).
fn collect_positional_params(command: &str) -> Vec<String> {
    let tokens = shell_tokens(command);
    let mut params = Vec::new();
    let mut at_command_pos = true;
    let mut index = 0;
    while index < tokens.len() {
        match &tokens[index] {
            ShellToken::Separator(_) => at_command_pos = true,
            ShellToken::Word(word) => {
                if at_command_pos
                    && word == "set"
                    && matches!(tokens.get(index + 1), Some(ShellToken::Word(d)) if d == "--")
                {
                    let mut captured = Vec::new();
                    let mut j = index + 2;
                    while let Some(ShellToken::Word(p)) = tokens.get(j) {
                        captured.push(p.clone());
                        j += 1;
                    }
                    params = captured;
                } else if at_command_pos && word == "shift" {
                    // `shift [n]` drops the first n positionals (review #685 r11
                    // ROOT B) so `set -- …; shift; git -C $@` reindexes correctly.
                    let n = match tokens.get(index + 1) {
                        Some(ShellToken::Word(w)) => w.parse::<usize>().unwrap_or(1),
                        _ => 1,
                    };
                    for _ in 0..n {
                        if params.is_empty() {
                            break;
                        }
                        params.remove(0);
                    }
                }
                at_command_pos = false;
            }
        }
        index += 1;
    }
    params
}

/// Inline calls to same-line shell functions (`name(){ body }`,
/// `function name { body }`) by replacing each call `name <args>` with the
/// function body, substituting `$@`/`$*`/`$N` in the body with the call args
/// (review #685 r10: `g(){ git "$@"; }; g -C <canon> reset --hard`). Bodies are
/// captured with balanced-brace matching; only the simple single-name-call form
/// is modeled — anything unrecognized is left verbatim (then fail-closed by the
/// residual checks if it still reaches a git decision).
fn inline_function_calls(command: &str) -> String {
    let defs = collect_function_defs(command);
    if defs.is_empty() {
        return command.to_owned();
    }
    // Strip the definition text so the body (`git "$@"`) is NOT evaluated as a
    // live command at the def site — only the inlined CALLS carry the args
    // (review #685 r13 F2: a leftover `g(){ git "$@"; }` def made worktree
    // calls over-deny). The call sites are inlined from the stripped string.
    let stripped = strip_function_defs(command);
    inline_with_defs(&stripped, &defs, 0)
}

/// Remove `name(){…}` / `function name {…}` definition spans (replacing each
/// with `;`) so only their inlined call sites remain (review #685 r13 F2).
fn strip_function_defs(command: &str) -> String {
    let mut spans = function_def_spans(command);
    spans.sort_by_key(|(start, _)| *start);
    let mut out = String::with_capacity(command.len());
    let mut cursor = 0;
    for (start, end) in spans {
        if start < cursor || end > command.len() {
            continue;
        }
        out.push_str(&command[cursor..start]);
        out.push(';');
        cursor = end;
    }
    out.push_str(&command[cursor..]);
    out
}

/// Byte spans of every function definition (name start .. after closing `}`).
fn function_def_spans(command: &str) -> Vec<(usize, usize)> {
    let bytes = command.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{'
            && let Some((_, _, end)) = try_capture_function(command, i)
        {
            // Def start = first non-separator after the previous separator.
            let header = command[..i].trim_end();
            let start = header
                .rfind([';', '\n', '\r', '&', '|', '('])
                .map(|p| p + 1)
                .unwrap_or(0);
            let start = start
                + command[start..i]
                    .find(|c: char| !c.is_whitespace())
                    .unwrap_or(0);
            spans.push((start, end));
            i = end;
            continue;
        }
        i += 1;
    }
    spans
}

/// Inline body resolution with the def set threaded through, recursing on each
/// substituted body so a multi-hop chain (`a(){ b "$@"; }; b(){ git "$@"; }`)
/// resolves in one call (review #685 r11 ROOT C). Bounded by depth.
fn inline_with_defs(command: &str, defs: &[(String, String)], depth: usize) -> String {
    if depth >= MAX_NESTED_COMMAND_DEPTH {
        return command.to_owned();
    }
    let tokens = shell_tokens(command);
    let mut out: Vec<String> = Vec::new();
    let mut at_command_pos = true;
    let mut index = 0;
    while index < tokens.len() {
        match &tokens[index] {
            ShellToken::Separator(kind) => {
                out.push(separator_text(*kind));
                at_command_pos = true;
                index += 1;
            }
            ShellToken::Word(word) => {
                if at_command_pos
                    && let Some((_, body)) = defs.iter().find(|(name, _)| name == word)
                    // Skip the DEFINITION site itself (followed by `(` or `{`).
                    && !matches!(tokens.get(index + 1), Some(ShellToken::Separator(SeparatorKind::SubshellStart)))
                    && !next_word_is_brace(&tokens, index + 1)
                {
                    let mut call_args = Vec::new();
                    let mut j = index + 1;
                    while let Some(ShellToken::Word(a)) = tokens.get(j) {
                        call_args.push(a.clone());
                        j += 1;
                    }
                    // `shift` inside the body drops leading call args before the
                    // body's own `$@`/`$N` are bound (review #685 r11 ROOT B/C).
                    let (shifts, body) = strip_leading_shift(body);
                    let effective = if shifts <= call_args.len() {
                        call_args[shifts..].to_vec()
                    } else {
                        Vec::new()
                    };
                    // A body with its OWN `set --` rebinds `$@`; emit it verbatim
                    // so the normalize fixpoint's positional collector governs it
                    // rather than clobbering `$@` with the call args (review #685
                    // r13: g(){ set -- …; git "$@"; }).
                    let substituted = if body_rebinds_positionals(&body) {
                        body.clone()
                    } else {
                        substitute_positionals(&body, &effective)
                    };
                    out.push(inline_with_defs(&substituted, defs, depth + 1));
                    index = j;
                    at_command_pos = false;
                } else {
                    out.push(requote_word(word));
                    at_command_pos = false;
                    index += 1;
                }
            }
        }
    }
    out.join(" ")
}

/// True when a function body contains its own `set --` (rebinding positionals),
/// so the call args must NOT be substituted into its `$@` (review #685 r13).
fn body_rebinds_positionals(body: &str) -> bool {
    let tokens = shell_tokens(body);
    let mut at_command_pos = true;
    for (i, token) in tokens.iter().enumerate() {
        match token {
            ShellToken::Separator(_) => at_command_pos = true,
            ShellToken::Word(word) => {
                if at_command_pos
                    && word == "set"
                    && matches!(tokens.get(i + 1), Some(ShellToken::Word(d)) if d == "--")
                {
                    return true;
                }
                at_command_pos = false;
            }
        }
    }
    false
}

/// Count and strip leading `shift [n]` statements from a function body so the
/// body's positionals bind to the post-shift call args (review #685 r11 ROOT B).
fn strip_leading_shift(body: &str) -> (usize, String) {
    let mut shifts = 0usize;
    let mut rest = body.trim_start();
    while let Some(after) = rest.strip_prefix("shift") {
        // Must be `shift` as a whole word (next char is space/`;`/end), not a
        // longer identifier like `shifty`.
        let after = match after.chars().next() {
            None | Some(' ' | '\t' | ';') => after.trim_start(),
            Some(_) => break,
        };
        let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        let count = if digits.is_empty() {
            1
        } else {
            digits.parse::<usize>().unwrap_or(1)
        };
        let tail = after[digits.len()..].trim_start();
        rest = tail.strip_prefix(';').unwrap_or(tail).trim_start();
        shifts += count;
    }
    (shifts, rest.to_owned())
}

fn separator_text(kind: SeparatorKind) -> String {
    match kind {
        SeparatorKind::Pipe => "|".to_owned(),
        SeparatorKind::SubshellStart => "(".to_owned(),
        SeparatorKind::SubshellEnd => ")".to_owned(),
        _ => ";".to_owned(),
    }
}

fn next_word_is_brace(tokens: &[ShellToken], index: usize) -> bool {
    matches!(tokens.get(index), Some(ShellToken::Word(w)) if w == "{" || w.starts_with('{'))
}

/// Capture `name(){ body }` / `name () { body }` / `function name { body }`
/// definitions from the raw command string with balanced-brace body matching.
fn collect_function_defs(command: &str) -> Vec<(String, String)> {
    let mut defs = Vec::new();
    let bytes = command.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Find a `{` that opens a function body and walk back to the name.
        if bytes[i] == b'{' {
            if let Some((name, body, end)) = try_capture_function(command, i) {
                defs.push((name, body));
                i = end;
                continue;
            }
        }
        i += 1;
    }
    defs
}

fn try_capture_function(command: &str, brace: usize) -> Option<(String, String, usize)> {
    // Header preceding `{` must look like `name()` / `name ()` / `function name`.
    let head = command[..brace].trim_end();
    let name = if let Some(stripped) = head.strip_suffix(')') {
        let inner = stripped.trim_end();
        let inner = inner.strip_suffix('(')?.trim_end();
        inner.rsplit([' ', '\t', ';', '\n']).next()?.to_owned()
    } else {
        // `function name {`
        let last = head.rsplit([' ', '\t', ';', '\n']).next()?;
        let before = head[..head.len() - last.len()].trim_end();
        if !before.ends_with("function") {
            return None;
        }
        last.to_owned()
    };
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c == '_' || c == '-' || c.is_ascii_alphanumeric())
        || name.as_bytes()[0].is_ascii_digit()
    {
        return None;
    }
    // Balanced-brace body.
    let body_start = brace + 1;
    let rest = command.get(body_start..)?;
    let mut depth = 1usize;
    for (off, ch) in rest.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let body = rest[..off].trim().trim_end_matches(';').trim().to_owned();
                    return Some((name, body, body_start + off + 1));
                }
            }
            _ => {}
        }
    }
    None
}

/// Substitute `$@`/`$*`/`$N`/`${@}`/`${N}` in a function body with the call
/// args (review #685 r10). Conservative: only positional forms are touched.
fn substitute_positionals(body: &str, args: &[String]) -> String {
    let joined = args.join(" ");
    let mut out = String::with_capacity(body.len());
    let mut chars = body.char_indices().peekable();
    while let Some((_, ch)) = chars.next() {
        // Drop double quotes so `"$@"` expands to SEPARATE words (`reset`
        // `--hard`) on re-tokenisation, not one quoted token (review #685 r10).
        if ch == '"' {
            continue;
        }
        if ch != '$' {
            out.push(ch);
            continue;
        }
        match chars.peek().map(|(_, c)| *c) {
            Some('@') | Some('*') => {
                out.push_str(&joined);
                chars.next();
            }
            Some('{') => {
                chars.next();
                let mut inner = String::new();
                while let Some((_, c)) = chars.next() {
                    if c == '}' {
                        break;
                    }
                    inner.push(c);
                }
                match inner.as_str() {
                    "@" | "*" => out.push_str(&joined),
                    digits if digits.chars().all(|c| c.is_ascii_digit()) => {
                        out.push_str(positional_at(args, digits));
                    }
                    other => {
                        out.push_str("${");
                        out.push_str(other);
                        out.push('}');
                    }
                }
            }
            Some(c) if c.is_ascii_digit() => {
                let mut digits = String::new();
                while let Some((_, d)) = chars.peek() {
                    if d.is_ascii_digit() {
                        digits.push(*d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                out.push_str(positional_at(args, &digits));
            }
            _ => out.push('$'),
        }
    }
    out
}

fn positional_at<'a>(args: &'a [String], digits: &str) -> &'a str {
    digits
        .parse::<usize>()
        .ok()
        .filter(|n| *n >= 1)
        .and_then(|n| args.get(n - 1))
        .map(String::as_str)
        .unwrap_or("")
}

/// True when a token still carries an unresolved shell expansion sigil after
/// static normalization — `` ` ``, `$(`, `${`, or `$` before a name/quote. Such
/// a token in command-name or git-subcommand position is treated fail-closed:
/// the guard cannot prove it is NOT a mutating git op (review #685 r6 F2).
fn has_unresolved_expansion(token: &str) -> bool {
    let bytes = token.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        // Any shell word-altering metacharacter makes the verb non-determinable
        // by static parsing — the SAME set the path side uses in
        // `target_path_is_dynamic` (review #685 r7: brace `{reset,}` / glob /
        // tilde resolve to a real mutating verb without a `$`/backtick sigil).
        if matches!(b, b'`' | b'*' | b'?' | b'[' | b']' | b'{' | b'}' | b'~') {
            return true;
        }
        // A `$` before any non-space follower is an expansion of SOME kind —
        // parameter (`$name`/`${…}`/`$'…'`), command (`$(…)`), arithmetic
        // (`$((…))`), or a special/positional param (`$@ $* $# $- $! $? $$ $N`,
        // review #685 r9 F1). Treat them all as unresolved in command-name
        // position; a bare trailing `$` is the only literal exception.
        if b == b'$' && bytes.get(i + 1).is_some_and(|c| !c.is_ascii_whitespace()) {
            return true;
        }
    }
    false
}

/// True when a token carries a `$`-parameter / command-substitution / backtick
/// expansion whose TEXT is arbitrary — it could inject a hidden `-C <canonical>`
/// retarget, so a git invocation with such a subcommand must fail closed on the
/// target regardless of the visible cwd/`-C` (review #685 r12). Brace/glob/tilde
/// are excluded: they obfuscate the verb in place but cannot move the target.
fn subcommand_carries_value_expansion(token: &str) -> bool {
    let bytes = token.as_bytes();
    bytes.iter().enumerate().any(|(i, &b)| {
        b == b'`' || (b == b'$' && bytes.get(i + 1).is_some_and(|c| !c.is_ascii_whitespace()))
    })
}

/// Collect `name=value` and `name+=value` assignments (quote-aware via
/// shell_tokens, so a quoted multi-word value stays ONE token). Values are kept
/// FAITHFULLY — a multi-word value is bound whole and word-split only at an
/// UNQUOTED `$name` use site, matching bash (review #685 r14: `P=reset;
/// P+=" --hard"; git $P` → `git reset --hard` → deny; `P=log; P+=" --oneline";
/// git $P` → `git log --oneline` → allow). A value still carrying an
/// unresolved `$`/backtick expansion is dropped (left unresolved → fail closed).
fn collect_same_line_bindings(command: &str) -> Vec<(String, String)> {
    let mut acc: Vec<(String, String)> = Vec::new();
    for token in shell_tokens(command) {
        let ShellToken::Word(word) = token else {
            continue;
        };
        let name_len = word
            .chars()
            .take_while(|c| *c == '_' || c.is_ascii_alphanumeric())
            .count();
        let name = &word[..name_len];
        if name.is_empty() || name.as_bytes()[0].is_ascii_digit() {
            continue;
        }
        let rest = &word[name_len..];
        // Bash expands an assignment's RHS at the assignment's EXECUTION point,
        // so `$ref` sees only the bindings to its LEFT — never a later
        // reassignment (`A=reset; B=$A; A=log` binds B=reset). Resolving
        // against the final last-wins map diverged from bash and ALLOWed
        // `git -C <canonical> $B --hard` (review #685 r19 F1). The left-fold
        // also removes the r18 fixpoint: left-only references cannot cycle,
        // and a still-unresolved ref propagates its sigil into the referencing
        // value, which the filter below drops → fail closed at the use site.
        if let Some(value) = rest.strip_prefix("+=") {
            let value = resolve_binding_refs(value, &acc);
            match acc.iter_mut().find(|(n, _)| n == name) {
                Some(entry) => entry.1.push_str(&value),
                None => acc.push((name.to_owned(), value)),
            }
        } else if let Some(value) = rest.strip_prefix('=') {
            let value = resolve_binding_refs(value, &acc);
            match acc.iter_mut().find(|(n, _)| n == name) {
                Some(entry) => entry.1 = value,
                None => acc.push((name.to_owned(), value)),
            }
        }
    }
    // Keep only fully-resolved values; any residual expansion sigil leaves the
    // var unbound so `$name` fails closed at its use site.
    acc.into_iter()
        .filter(|(_, value)| !value.is_empty() && !value.contains('$') && !value.contains('`'))
        .collect()
}

/// Resolve `${N:-default}` / `${N-default}` / `${N:=default}` (N a positional
/// index) against the set-- positionals (review #685 r18 F3). Returns None when
/// `inner` is not a digit-indexed default form, or positionals are unbound
/// (then the literal `${…}` survives → fail closed).
fn resolve_positional_default(inner: &str, positionals: &[String]) -> Option<String> {
    if positionals.is_empty() {
        return None;
    }
    let (digits, default) = inner
        .split_once(":-")
        .or_else(|| inner.split_once(":="))
        .or_else(|| inner.split_once('-'))?;
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let value = positional_at(positionals, digits);
    Some(if value.is_empty() {
        default.to_owned()
    } else {
        value.to_owned()
    })
}

/// Substitute `$name` / `${name}` references in a binding value with other
/// bindings' values (review #685 r18 F2). Unknown names are left literal (still
/// carry `$` → dropped by the caller's resolved-only filter → fail closed).
fn resolve_binding_refs(value: &str, bindings: &[(String, String)]) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.char_indices().peekable();
    while let Some((_, ch)) = chars.next() {
        if ch != '$' {
            out.push(ch);
            continue;
        }
        let braced = chars.peek().map(|(_, c)| *c) == Some('{');
        if braced {
            chars.next();
        }
        let mut name = String::new();
        while let Some((_, c)) = chars.peek() {
            if *c == '_' || c.is_ascii_alphanumeric() {
                name.push(*c);
                chars.next();
            } else {
                break;
            }
        }
        if braced && chars.peek().map(|(_, c)| *c) == Some('}') {
            chars.next();
        }
        match binding_value(bindings, &name) {
            Some(v) if !name.is_empty() => out.push_str(v),
            _ => {
                out.push('$');
                if braced {
                    out.push('{');
                }
                out.push_str(&name);
                if braced {
                    out.push('}');
                }
            }
        }
    }
    out
}

fn binding_value<'a>(bindings: &'a [(String, String)], name: &str) -> Option<&'a str> {
    bindings
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}

/// Arrays bound by `read -a/-ra NAME <<< "<words>"` (review #685 r11 ROOT A) so
/// `${NAME[@]}` re-tokenises into the canonical-mutation words. Only the
/// here-string (`<<<`) literal form is modeled — a runtime `read` from stdin/a
/// pipe is genuinely unknowable and left for the fail-closed residual checks.
fn collect_array_bindings(command: &str) -> Vec<(String, Vec<String>)> {
    let tokens = shell_tokens(command);
    let mut arrays = Vec::new();
    let mut at_command_pos = true;
    let mut index = 0;
    while index < tokens.len() {
        match &tokens[index] {
            ShellToken::Separator(_) => at_command_pos = true,
            ShellToken::Word(word) => {
                if at_command_pos && command_basename(word) == Some("read") {
                    let mut name = None;
                    let mut value = None;
                    let mut j = index + 1;
                    while let Some(ShellToken::Word(arg)) = tokens.get(j) {
                        if (arg == "-a" || arg == "-ra" || arg == "-ar")
                            && let Some(ShellToken::Word(n)) = tokens.get(j + 1)
                        {
                            name = Some(n.clone());
                            j += 1;
                        } else if arg == "<<<"
                            && let Some(ShellToken::Word(v)) = tokens.get(j + 1)
                        {
                            value = Some(v.clone());
                            j += 1;
                        } else if let Some(v) = arg.strip_prefix("<<<") {
                            if !v.is_empty() {
                                value = Some(v.to_owned());
                            }
                        }
                        j += 1;
                    }
                    if let (Some(n), Some(v)) = (name, value) {
                        arrays.push((n, v.split_whitespace().map(str::to_owned).collect()));
                    }
                }
                at_command_pos = false;
            }
        }
        index += 1;
    }
    arrays
}

fn array_value<'a>(arrays: &'a [(String, Vec<String>)], name: &str) -> Option<&'a [String]> {
    arrays
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_slice())
}

/// Emit a decoded command-substitution result, single-quoting it when it is the
/// RHS of an assignment AND contains whitespace (review #685 r13 F1). In bash,
/// `P=$(echo "reset --hard")` assigns the WHOLE multi-word output to P without
/// word-splitting; emitting it bare would let the binding collector capture only
/// the first word (`P=reset`, dropping `--hard`). Re-quoting keeps it one value
/// so the collector rejects it (spaces) → `$P` stays unresolved → fail closed,
/// matching the literal `P="reset --hard"` handling. In command position the
/// output is emitted bare so it parses as a command (`$(echo git … reset)`).
fn emit_substitution_output(out: &mut String, produced: &str) {
    if produced.chars().any(char::is_whitespace) && out_ends_with_assignment_lhs(out) {
        out.push('\'');
        out.push_str(&produced.replace('\'', "'\\''"));
        out.push('\'');
    } else {
        out.push_str(produced);
    }
}

/// True when `out` currently ends with a shell assignment LHS `NAME=` (the `=`
/// preceded by a valid identifier at a word boundary), i.e. the next emitted
/// token is an assignment value.
fn out_ends_with_assignment_lhs(out: &str) -> bool {
    let prefix = out
        .strip_suffix('=')
        .map(|p| p.strip_suffix('+').unwrap_or(p));
    let Some(prefix) = prefix else {
        return false;
    };
    let name_rev: String = prefix
        .chars()
        .rev()
        .take_while(|c| *c == '_' || c.is_ascii_alphanumeric())
        .collect();
    // `name_rev` holds the identifier reversed; its LAST char is the real first
    // char (an identifier must not start with a digit).
    if name_rev.is_empty() || name_rev.chars().last().is_some_and(|c| c.is_ascii_digit()) {
        return false;
    }
    let before = &prefix[..prefix.len() - name_rev.len()];
    before.is_empty() || before.ends_with([' ', '\t', ';', '\n', '\r'])
}

/// If `rest` begins with a double-quoted positional/array expansion that bash
/// would word-SPLIT (`"$@"`, `"$*"`, `"${@}"`, `"${*}"`, `"${NAME[@]}"`,
/// `"${NAME[*]}"`), return its space-separated expansion and the byte length
/// consumed (opening quote through closing quote). None for any other quoted
/// content so command substitutions and literals keep their quotes (review
/// #685 r11 ROOT A — surgical, not a blanket quote strip).
fn quoted_split_expansion(
    rest: &str,
    positionals: &[String],
    arrays: &[(String, Vec<String>)],
) -> Option<(String, usize)> {
    let inner = rest.strip_prefix('"')?;
    // The quoted body must be EXACTLY one positional/array expansion then `"`.
    let close = inner.find('"')?;
    let body = &inner[..close];
    let consumed = 1 + close + 1; // opening quote + body + closing quote
    let expansion = match body {
        // Unbound positionals → None (keep quotes → literal → fail closed, r12).
        "$@" | "$*" | "${@}" | "${*}" if positionals.is_empty() => return None,
        "$@" | "$*" | "${@}" | "${*}" => positionals.join(" "),
        _ => {
            let braced = body.strip_prefix("${").and_then(|b| b.strip_suffix('}'))?;
            expand_array_subscript(braced, arrays)?
        }
    };
    Some((expansion, consumed))
}

/// Expand a `${NAME[@]}` / `${NAME[*]}` / `${NAME[N]}` array subscript against
/// the collected array bindings (review #685 r11 ROOT A). Returns None for
/// non-array `${…}` forms (handled by the parameter path).
fn expand_array_subscript(inner: &str, arrays: &[(String, Vec<String>)]) -> Option<String> {
    let open = inner.find('[')?;
    let close = inner.strip_suffix(']')?.len();
    if close < open {
        return None;
    }
    let name = &inner[..open];
    let subscript = &inner[open + 1..inner.len() - 1];
    let elems = array_value(arrays, name)?;
    match subscript {
        "@" | "*" => Some(elems.join(" ")),
        digits if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) => Some(
            digits
                .parse::<usize>()
                .ok()
                .and_then(|n| elems.get(n))
                .cloned()
                .unwrap_or_default(),
        ),
        _ => None,
    }
}

/// Same-line `IFS=…` reassignment that could re-split unquoted expansions.
/// Any `IFS=` assignment word triggers it — empty/default values only make the
/// suppression slightly more eager (a read keeps literal `$REF` in arg
/// position, so no real command is over-denied; review #685 r8 F4).
fn same_line_ifs_reassigned(command: &str) -> bool {
    command
        .split([' ', '\t', ';', '\n', '\r'])
        .any(|word| word == "IFS" || word.starts_with("IFS="))
}

fn expand_with_bindings(
    command: &str,
    bindings: &[(String, String)],
    positionals: &[String],
    arrays: &[(String, Vec<String>)],
    ifs_unsafe: bool,
    substitutions_only: bool,
) -> String {
    let positional_join = positionals.join(" ");
    let xpg_echo = command.contains("xpg_echo");
    let mut out = String::with_capacity(command.len());
    let mut chars = command.char_indices().peekable();
    let mut single_quote = false;
    while let Some((idx, ch)) = chars.next() {
        if single_quote {
            out.push(ch);
            if ch == '\'' {
                single_quote = false;
            }
            continue;
        }
        match ch {
            '\'' => {
                single_quote = true;
                out.push(ch);
            }
            // Drop the quotes ONLY around a positional/array expansion
            // (`"$@"`, `"${@}"`, `"${P[@]}"`) so it re-tokenises into SEPARATE
            // words rather than one mega-arg (review #685 r11 ROOT A). Other
            // double-quoted content keeps its quotes so `bash -c "$(…)"` still
            // groups the script into one `-c` argument (regression guard).
            '"' if quoted_split_expansion(&command[idx..], positionals, arrays).is_some() => {
                let (expansion, consumed) =
                    quoted_split_expansion(&command[idx..], positionals, arrays)
                        .unwrap_or_default();
                out.push_str(&expansion);
                // Advance past the consumed `"…"` span (consumed counts bytes
                // from the opening quote through the closing quote inclusive).
                let target = idx + consumed;
                while chars.peek().is_some_and(|(i, _)| *i < target) {
                    chars.next();
                }
            }
            '\\' => {
                // Backslash line-continuation (`re\<newline>set`) is JOINED by
                // the shell — drop the `\`+newline so the verb reassembles
                // (review #685 r9 F3). Any other escaped char is kept literal.
                match chars.peek() {
                    Some((_, '\n')) | Some((_, '\r')) => {
                        chars.next();
                    }
                    _ => {
                        out.push(ch);
                        if let Some((_, next)) = chars.next() {
                            out.push(next);
                        }
                    }
                }
            }
            '$' if chars.peek().is_some_and(|(_, c)| *c == '\'') => {
                chars.next();
                let mut raw = String::new();
                while let Some((_, c)) = chars.next() {
                    if c == '\\' {
                        if let Some((_, e)) = chars.next() {
                            raw.push('\\');
                            raw.push(e);
                        }
                    } else if c == '\'' {
                        break;
                    } else {
                        raw.push(c);
                    }
                }
                // Decode ANSI-C escapes — incl. \xHH / \NNN octal / \uHHHH that
                // the shell decodes (review #685 r9 F2: `$'\x72\x65…'`→`reset`).
                // ANSI-C `$'…'` is a QUOTING construct → its decoded value is
                // ALWAYS one word. Single-quote it unconditionally so adjacent
                // segments concatenate into one word (`$'reset'$'\t--hard'` →
                // 'reset'' --hard' → reset --hard one binding) and it never
                // splits at the ANSI-C site itself (review #685 r15/r16 F3).
                let decoded = normalize_split_ws(&decode_ansi_c(&raw));
                out.push('\'');
                out.push_str(&decoded.replace('\'', "'\\''"));
                out.push('\'');
            }
            '$' if chars.peek().is_some_and(|(_, c)| *c == '{') => {
                chars.next();
                let mut inner = String::new();
                while let Some((_, c)) = chars.next() {
                    if c == '}' {
                        break;
                    }
                    inner.push(c);
                }
                if substitutions_only {
                    out.push_str("${");
                    out.push_str(&inner);
                    out.push('}');
                    continue;
                }
                if !ifs_unsafe && matches!(inner.as_str(), "@" | "*") {
                    // Unbound positionals (no visible set--/read) stay literal so
                    // an eval-/scope-hidden binding fails closed (review #685 r12).
                    if positionals.is_empty() {
                        out.push_str("${");
                        out.push_str(&inner);
                        out.push('}');
                    } else {
                        out.push_str(&positional_join);
                    }
                } else if !ifs_unsafe
                    && !inner.is_empty()
                    && inner.chars().all(|c| c.is_ascii_digit())
                {
                    // Unbound positionals stay literal so a later set-- pass can
                    // bind them (review #685 r13 two-phase normalization).
                    if positionals.is_empty() {
                        out.push_str("${");
                        out.push_str(&inner);
                        out.push('}');
                    } else {
                        out.push_str(positional_at(positionals, &inner));
                    }
                } else if let Some(expanded) = (!ifs_unsafe)
                    .then(|| resolve_positional_default(&inner, positionals))
                    .flatten()
                {
                    // `${N:-default}` / `${N-default}` / `${N:=default}` against
                    // the set-- positionals (review #685 r18 F3).
                    out.push_str(&expanded);
                } else if let Some(expanded) = (!ifs_unsafe)
                    .then(|| expand_array_subscript(&inner, arrays))
                    .flatten()
                {
                    out.push_str(&expanded);
                } else {
                    match resolve_param(&inner, bindings).filter(|_| !ifs_unsafe) {
                        Some(value) => out.push_str(&value),
                        None => {
                            out.push_str("${");
                            out.push_str(&inner);
                            out.push('}');
                        }
                    }
                }
            }
            '$' if !ifs_unsafe
                && !substitutions_only
                && chars.peek().is_some_and(|(_, c)| *c == '@' || *c == '*') =>
            {
                let sigil = chars.next().map(|(_, c)| c).unwrap_or('@');
                // Unbound positionals stay literal → fail closed (review #685 r12).
                if positionals.is_empty() {
                    out.push('$');
                    out.push(sigil);
                } else {
                    out.push_str(&positional_join);
                }
            }
            '$' if !ifs_unsafe
                && !substitutions_only
                && chars.peek().is_some_and(|(_, c)| c.is_ascii_digit()) =>
            {
                let mut digits = String::new();
                while let Some((_, c)) = chars.peek() {
                    if c.is_ascii_digit() {
                        digits.push(*c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                // Unbound positionals stay literal (review #685 r13).
                if positionals.is_empty() {
                    out.push('$');
                    out.push_str(&digits);
                } else {
                    out.push_str(positional_at(positionals, &digits));
                }
            }
            '$' if chars.peek().is_some_and(|(_, c)| *c == '(') => {
                if let Some((end, body)) = extract_balanced_dollar_command(command, idx + 2) {
                    match static_command_output(&body, xpg_echo).filter(|_| !ifs_unsafe) {
                        Some(produced) => emit_substitution_output(&mut out, &produced),
                        // An unmodeled VALUE-PRODUCER (printf/echo whose exact
                        // output we cannot reproduce) → emit a sigil token so it
                        // fails closed in git position; a real command body
                        // (`$(git …)`, `$(curl)`) keeps its raw text so the
                        // separate substitution-recursion still evaluates it
                        // (review #685 r16 fail-closed allowlist).
                        None if is_value_producer(&body) => {
                            out.push_str(UNRESOLVED_TARGET_SENTINEL)
                        }
                        None => out.push_str(&command[idx..=end]),
                    }
                    while chars.peek().is_some_and(|(i, _)| *i <= end) {
                        chars.next();
                    }
                } else {
                    out.push(ch);
                }
            }
            '$' if !substitutions_only
                && chars
                    .peek()
                    .is_some_and(|(_, c)| c.is_ascii_alphabetic() || *c == '_') =>
            {
                let mut name = String::new();
                while let Some((_, c)) = chars.peek() {
                    if *c == '_' || c.is_ascii_alphanumeric() {
                        name.push(*c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                match binding_value(bindings, &name).filter(|_| !ifs_unsafe) {
                    // Requote a multi-word value landing as an assignment RHS
                    // (`B=$A`) so it stays one binding, else word-split at use
                    // (review #685 r14: chained multi-word vars).
                    Some(value) => emit_substitution_output(&mut out, value),
                    None => {
                        out.push('$');
                        out.push_str(&name);
                    }
                }
            }
            '`' => {
                let start = idx + ch.len_utf8();
                if let Some(end) = find_unescaped_backtick(command, start) {
                    let body = unescape_backtick_body(&command[start..end]);
                    match static_command_output(&body, xpg_echo).filter(|_| !ifs_unsafe) {
                        Some(produced) => emit_substitution_output(&mut out, &produced),
                        // An unmodeled VALUE-PRODUCER (printf/echo whose exact
                        // output we cannot reproduce) → emit a sigil token so it
                        // fails closed in git position; a real command body
                        // (`$(git …)`, `$(curl)`) keeps its raw text so the
                        // separate substitution-recursion still evaluates it
                        // (review #685 r16 fail-closed allowlist).
                        None if is_value_producer(&body) => {
                            out.push_str(UNRESOLVED_TARGET_SENTINEL)
                        }
                        None => out.push_str(&command[idx..=end]),
                    }
                    while chars.peek().is_some_and(|(i, _)| *i <= end) {
                        chars.next();
                    }
                } else {
                    out.push(ch);
                }
            }
            _ => out.push(ch),
        }
    }
    out
}

/// Resolve EXACTLY the modeled `${…}` parameter forms — `${name}`,
/// `${name:-default}`, `${name-default}`, `${name:=default}`. Returns None for
/// every other operator (`${P/x/y}`, `${P^^}`, `${P,,}`, `${P:off:len}`,
/// `${#P}`, `${!P}`, …) so it stays unresolved → fail closed, rather than an
/// under-approximation that drops an injected mutation (review #685 r16:
/// fail-closed allowlist of exactly-modeled shapes, not best-effort output).
fn resolve_param(inner: &str, bindings: &[(String, String)]) -> Option<String> {
    let is_ident =
        |s: &str| !s.is_empty() && s.chars().all(|c| c == '_' || c.is_ascii_alphanumeric());
    // `${name:-default}` / `${name:=default}` (default when unset/empty).
    for sep in [":-", ":="] {
        if let Some((name, default)) = inner.split_once(sep) {
            if is_ident(name) {
                return Some(
                    binding_value(bindings, name)
                        .map(str::to_owned)
                        .unwrap_or_else(|| default.to_owned()),
                );
            }
            return None;
        }
    }
    // `${name-default}` (default only when UNSET) — the `name` must be a clean
    // identifier; reject `${P/ZZ/ --}` where the part before `-` is not one.
    if let Some((name, default)) = inner.split_once('-') {
        if is_ident(name) {
            return Some(
                binding_value(bindings, name)
                    .map(str::to_owned)
                    .unwrap_or_else(|| default.to_owned()),
            );
        }
        return None;
    }
    if is_ident(inner) {
        return binding_value(bindings, inner).map(str::to_owned);
    }
    None
}

/// Static output of a command-substitution body when it is a simple `echo …`
/// or `printf …` — the only forms whose output we can determine without
/// executing them. Anything else returns None (kept verbatim, so a real
/// `$(prog)` is never silently treated as producing a command).
/// True when a substitution body's command is a value-producer (echo/printf)
/// whose output we attempt to model — so an unmodeled one fails closed rather
/// than passing through (review #685 r16).
fn is_value_producer(body: &str) -> bool {
    let cmd = body
        .trim()
        .split_once(char::is_whitespace)
        .map_or(body.trim(), |(c, _)| c);
    matches!(command_basename(cmd), Some("echo" | "printf"))
}

fn static_command_output(body: &str, xpg_echo: bool) -> Option<String> {
    let body = body.trim();
    let (cmd, rest) = body.split_once(char::is_whitespace).unwrap_or((body, ""));
    // A nested unresolved substitution in a value-producer's args must be
    // resolved INNER-FIRST; our naive dequote would strip the inner quotes and
    // mis-model it. Return the args with the substitution PRESERVED so the
    // normalize fixpoint resolves the inner, then this re-runs (review #685 r17:
    // `$(echo $(printf 'git … reset'))`, nested backticks).
    // Only ECHO passes a nested substitution through verbatim (its dequote
    // would strip the inner quotes). printf must NOT preserve — keeping the
    // outer quotes would group its split output into one arg (review #685 r18
    // F1 regression); it falls through to the format/args model, which leaves
    // the inner `$(…)` for the fixpoint and word-splits the result.
    if (rest.contains("$(") || rest.contains('`')) && command_basename(cmd) == Some("echo") {
        return Some(rest.trim().to_owned());
    }
    match command_basename(cmd)? {
        "echo" => {
            let rest = rest.trim_start();
            // `echo -e` decodes backslash escapes (\t/\n/\xNN) into REAL
            // whitespace bash then word-splits on (review #685 r15).
            let decode = xpg_echo || rest.starts_with("-e ") || rest.starts_with("-ne ");
            let rest = rest
                .strip_prefix("-n ")
                .or_else(|| rest.strip_prefix("-e "))
                .or_else(|| rest.strip_prefix("-ne "))
                .or_else(|| rest.strip_prefix("-en "))
                .unwrap_or(rest);
            // `echo -e` keeps escapes for decode (strip quote chars only); plain
            // echo dequotes normally (review #685 r15).
            Some(if decode {
                normalize_split_ws(&decode_ansi_c(&strip_quote_chars(rest)))
            } else {
                dequote_simple(rest)
            })
        }
        // printf FMT [ARGS]: the format string (dequoted) is the produced text
        // for the specifier-free forms an evasion would use.
        "printf" => {
            // `printf FMT ARGS`: if the format is a single `%s`/`%b` (optionally
            // `\n`-terminated), the output is the ARGS (review #685 r14:
            // `printf '%s' "-C … clean -fdx"`). A specifier-free format is its
            // own literal output (`printf 'git … reset'`). printf ALWAYS decodes
            // backslash escapes in the format, so `printf "reset\t--hard"` →
            // real TAB → word-splits (review #685 r15).
            // Keep escapes (strip quote chars only) so printf's \t/\n/\xNN decode
            // to real whitespace. FAIL-CLOSED ALLOWLIST (review #685 r16): model
            // ONLY (a) a specifier-free format, or (b) a single leading `%s`/`%b`
            // then args. Any other format (`%-10s` width, `%s%s` multi-spec,
            // `%c`, precision, format-reuse over N args) → None → unresolved →
            // fail closed, never a best-effort under-approximation.
            let mut r = rest.trim();
            if r.starts_with("--") {
                r = r[2..].trim_start();
            }
            // Split FORMAT (first token) from ARGS via shell_tokens so a quoted
            // multi-word format (`"%s --hard"`) is one unit — the strip_quote
            // approach lost that boundary and mis-ordered the tail (review #685
            // r17 F1). Model ONLY: a specifier-free literal format → itself, or
            // a BARE single `%s`/`%b` (optionally `\n`/`\t`) → the args. Any
            // format with a trailing literal or extra specifier → None → sigil.
            // Split FORMAT (first quote-aware word, backslashes PRESERVED for
            // decode) from ARGS (review #685 r17 F1). shell_tokens would eat the
            // `\t` backslash in double quotes before decode, so use a custom
            // quote-aware split that keeps escapes.
            let (fmt_raw, args_raw) = split_first_shell_word(r);
            let fmt = strip_quote_chars(&fmt_raw);
            let modeled = if fmt.contains('%') {
                let bare = matches!(
                    fmt.as_str(),
                    "%s" | "%b" | "%s\\n" | "%b\\n" | "%s\\t" | "%b\\t"
                );
                if bare {
                    Some(strip_quote_chars(args_raw.trim()))
                } else {
                    None
                }
            } else {
                Some(fmt)
            };
            modeled.map(|m| normalize_split_ws(&decode_ansi_c(&m)))
        }
        _ => None,
    }
}

/// Decode the ANSI-C (`$'…'`) escape sequences the shell expands: standard
/// letter escapes, `\xHH` hex, `\NNN` octal, and `\uHHHH`/`\UHHHHHHHH` unicode.
/// Unknown escapes keep their literal char. Used so a hex-encoded git verb
/// (`$'\x72\x65\x73\x65\x74'`→`reset`) reaches the blocked-verb table instead
/// of slipping through as the literal `x72…` (review #685 r9 F2).
fn decode_ansi_c(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        let Some(&esc) = chars.peek() else {
            out.push('\\');
            break;
        };
        match esc {
            'n' => {
                out.push('\n');
                chars.next();
            }
            't' => {
                out.push('\t');
                chars.next();
            }
            'r' => {
                out.push('\r');
                chars.next();
            }
            'a' => {
                out.push('\u{07}');
                chars.next();
            }
            'b' => {
                out.push('\u{08}');
                chars.next();
            }
            'e' | 'E' => {
                out.push('\u{1b}');
                chars.next();
            }
            'f' => {
                out.push('\u{0c}');
                chars.next();
            }
            'v' => {
                out.push('\u{0b}');
                chars.next();
            }
            '\\' | '\'' | '"' | '?' => {
                out.push(esc);
                chars.next();
            }
            'x' => {
                chars.next();
                let mut hex = String::new();
                while hex.len() < 2 && chars.peek().is_some_and(|c| c.is_ascii_hexdigit()) {
                    hex.push(chars.next().unwrap_or('\0'));
                }
                push_codepoint(&mut out, &hex, 16);
            }
            'u' | 'U' => {
                let width = if esc == 'u' { 4 } else { 8 };
                chars.next();
                let mut hex = String::new();
                while hex.len() < width && chars.peek().is_some_and(|c| c.is_ascii_hexdigit()) {
                    hex.push(chars.next().unwrap_or('\0'));
                }
                push_codepoint(&mut out, &hex, 16);
            }
            '0'..='7' => {
                let mut oct = String::new();
                while oct.len() < 3 && chars.peek().is_some_and(|c| ('0'..='7').contains(c)) {
                    oct.push(chars.next().unwrap_or('\0'));
                }
                push_codepoint(&mut out, &oct, 8);
            }
            other => {
                out.push('\\');
                out.push(other);
                chars.next();
            }
        }
    }
    out
}

fn push_codepoint(out: &mut String, digits: &str, radix: u32) {
    if digits.is_empty() {
        return;
    }
    if let Some(ch) = u32::from_str_radix(digits, radix)
        .ok()
        .and_then(char::from_u32)
    {
        out.push(ch);
    }
}

/// Collapse internal tab/newline/CR to spaces: in an UNQUOTED command
/// substitution (and decoded ANSI-C value) bash word-SPLITS on IFS whitespace,
/// so these are argument boundaries, not command separators (review #685 r15:
/// `$(printf "clean\n-fdx")` → `clean -fdx`, not two commands).
fn normalize_split_ws(s: &str) -> String {
    s.chars()
        .map(|c| {
            if matches!(c, '\t' | '\n' | '\r') {
                ' '
            } else {
                c
            }
        })
        .collect()
}

/// Split off the first quote-aware shell word, PRESERVING backslash escapes
/// (unlike shell_tokens, which processes them). Returns (first_word_raw, rest).
fn split_first_shell_word(s: &str) -> (String, String) {
    let s = s.trim_start();
    let mut quote = None;
    let mut end = s.len();
    for (i, ch) in s.char_indices() {
        match quote {
            Some(q) => {
                if ch == q {
                    quote = None;
                }
            }
            None => match ch {
                '\'' | '"' => quote = Some(ch),
                c if c.is_whitespace() => {
                    end = i;
                    break;
                }
                _ => {}
            },
        }
    }
    (s[..end].to_owned(), s[end..].trim_start().to_owned())
}

/// Remove shell quote characters (`'` `"`) while PRESERVING backslash escapes,
/// so a following `decode_ansi_c` can turn `\t`/`\n`/`\xNN` into the real
/// whitespace bash word-splits on (review #685 r15: printf / `echo -e`).
fn strip_quote_chars(s: &str) -> String {
    s.chars().filter(|c| *c != '"' && *c != '\'').collect()
}

/// Remove one level of shell quoting/escaping from a static string.
fn dequote_simple(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut quote = None;
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                } else if q == '"' && c == '\\' {
                    if let Some(n) = chars.next() {
                        out.push(n);
                    }
                } else {
                    out.push(c);
                }
            }
            None => match c {
                '\'' | '"' => quote = Some(c),
                '\\' => {
                    if let Some(n) = chars.next() {
                        out.push(n);
                    }
                }
                _ => out.push(c),
            },
        }
    }
    out
}

fn extract_command_substitutions(command: &str) -> Vec<String> {
    let mut substitutions = Vec::new();
    let mut chars = command.char_indices().peekable();
    let mut quote = None;

    while let Some((index, ch)) = chars.next() {
        if let Some(quote_ch) = quote {
            if ch == quote_ch {
                quote = None;
                continue;
            }
            if quote_ch == '\'' {
                continue;
            }
        } else if ch == '\'' || ch == '"' {
            quote = Some(ch);
            continue;
        }
        if ch == '\\' {
            chars.next();
            continue;
        }
        if ch == '$' && chars.peek().is_some_and(|(_, next)| *next == '(') {
            chars.next();
            if let Some((end, body)) = extract_balanced_dollar_command(command, index + 2) {
                substitutions.push(body);
                while chars
                    .peek()
                    .is_some_and(|(next_index, _)| *next_index <= end)
                {
                    chars.next();
                }
            }
            continue;
        }
        if ch == '`' {
            let start = index + ch.len_utf8();
            if let Some(end) = find_unescaped_backtick(command, start) {
                // Unescape the backtick body so a NESTED `\`git …\`` substitution
                // is recoverable on the recursive scan (review #685 r4 F2: the
                // old plain find('`') stopped at the first escaped backtick and
                // never reached the inner git mutation).
                substitutions.push(unescape_backtick_body(&command[start..end]));
                while chars
                    .peek()
                    .is_some_and(|(next_index, _)| *next_index <= end)
                {
                    chars.next();
                }
            }
        }
    }

    substitutions
}

/// Index of the first UNescaped backtick at or after `start`, honouring `\`
/// escaping within the backtick body (POSIX: `\` escapes `` ` ``, `\`, `$`).
fn find_unescaped_backtick(command: &str, start: usize) -> Option<usize> {
    let mut escaped = false;
    for (offset, ch) in command[start..].char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '`' => return Some(start + offset),
            _ => {}
        }
    }
    None
}

/// Unescape a backtick-substitution body: inside `` `…` `` a backslash is only
/// special before `` ` ``, `\`, and `$`. Recovers the literal inner command so a
/// nested substitution re-scans correctly.
fn unescape_backtick_body(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some(next @ ('`' | '\\' | '$')) => out.push(next),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn extract_balanced_dollar_command(command: &str, start: usize) -> Option<(usize, String)> {
    let mut depth = 1usize;
    let mut escaped = false;
    for (relative_index, ch) in command[start..].char_indices() {
        let index = start + relative_index;
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '(' {
            depth = depth.saturating_add(1);
        } else if ch == ')' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some((index, command[start..index].to_owned()));
            }
        }
    }
    None
}

fn words_until_separator(tokens: &[ShellToken], start: usize) -> Vec<String> {
    let mut words = Vec::new();
    for token in &tokens[start..] {
        match token {
            ShellToken::Word(word) => words.push(word.to_owned()),
            ShellToken::Separator(_) => break,
        }
    }
    words
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GitAliasTarget {
    Subcommand { name: String, args: Vec<String> },
    ShellScript(String),
    UnknownShellScript,
}

impl GitAliasTarget {
    fn subcommand(&self) -> Option<&str> {
        match self {
            GitAliasTarget::Subcommand { name, .. } => Some(name),
            GitAliasTarget::ShellScript(_) | GitAliasTarget::UnknownShellScript => {
                Some("__shell_alias")
            }
        }
    }

    fn args(&self) -> &[String] {
        match self {
            GitAliasTarget::Subcommand { args, .. } => args,
            GitAliasTarget::ShellScript(_) | GitAliasTarget::UnknownShellScript => &[],
        }
    }

    fn shell_script(&self) -> Option<&str> {
        match self {
            GitAliasTarget::ShellScript(script) => Some(script),
            GitAliasTarget::Subcommand { .. } | GitAliasTarget::UnknownShellScript => None,
        }
    }
}

fn record_git_alias_config(config: &str, aliases: &mut Vec<(String, GitAliasTarget)>) {
    let Some((key, value)) = config.split_once('=') else {
        return;
    };
    let Some(alias_name) = key.strip_prefix("alias.") else {
        return;
    };
    if alias_name.is_empty() {
        return;
    }
    aliases.push((alias_name.to_owned(), git_alias_target_from_value(value)));
}

fn resolve_git_alias<'a>(
    subcommand: &'a str,
    aliases: &'a [(String, GitAliasTarget)],
) -> Option<&'a GitAliasTarget> {
    aliases
        .iter()
        .rev()
        .find_map(|(name, target)| (name == subcommand).then_some(target))
}

fn record_git_alias_config_env(
    config: &str,
    aliases: &mut Vec<(String, GitAliasTarget)>,
    git_env: &GitEnv,
) {
    let Some((key, env_name)) = config.split_once('=') else {
        return;
    };
    let Some(alias_name) = key.strip_prefix("alias.") else {
        return;
    };
    if alias_name.is_empty() {
        return;
    }
    let alias_target = git_env.value(env_name).map_or(
        GitAliasTarget::UnknownShellScript,
        git_alias_target_from_value,
    );
    aliases.push((alias_name.to_owned(), alias_target));
}

fn git_env_config_aliases(git_env: &GitEnv) -> Vec<(String, GitAliasTarget)> {
    let Some(count) = git_env
        .value("GIT_CONFIG_COUNT")
        .and_then(|value| value.parse::<usize>().ok())
    else {
        return Vec::new();
    };
    let mut aliases = Vec::new();
    for index in 0..count {
        let key_name = format!("GIT_CONFIG_KEY_{index}");
        let value_name = format!("GIT_CONFIG_VALUE_{index}");
        let Some(key) = git_env.value(&key_name) else {
            continue;
        };
        let Some(value) = git_env.value(&value_name) else {
            continue;
        };
        let Some(alias_name) = key.strip_prefix("alias.") else {
            continue;
        };
        if alias_name.is_empty() {
            continue;
        }
        aliases.push((alias_name.to_owned(), git_alias_target_from_value(value)));
    }
    aliases
}

fn git_alias_target_from_value(value: &str) -> GitAliasTarget {
    if let Some(script) = value.strip_prefix('!') {
        return GitAliasTarget::ShellScript(script.trim_start().to_owned());
    }
    let mut words = value.split_whitespace();
    let Some(name) = words.next() else {
        return GitAliasTarget::UnknownShellScript;
    };
    GitAliasTarget::Subcommand {
        name: name.to_owned(),
        args: words.map(str::to_owned).collect(),
    }
}

fn is_shell_reserved_word(word: &str) -> bool {
    matches!(
        word,
        "{" | "}"
            | "!"
            | "if"
            | "then"
            | "else"
            | "elif"
            | "fi"
            | "case"
            | "esac"
            | "for"
            | "select"
            | "while"
            | "until"
            | "do"
            | "done"
            | "in"
            | "time"
    )
}

fn is_shell_redirection(word: &str) -> bool {
    let rest = strip_redirection_fd(word);
    rest.starts_with('<') || rest.starts_with('>') || rest.starts_with("&>")
}

fn redirection_consumes_next(word: &str) -> bool {
    matches!(
        strip_redirection_fd(word),
        "<" | ">" | ">>" | "<>" | ">|" | "<&" | ">&" | "<<" | "<<<" | "&>" | "&>>"
    )
}

fn strip_redirection_fd(word: &str) -> &str {
    let first_non_digit = word
        .char_indices()
        .find_map(|(index, ch)| (!ch.is_ascii_digit()).then_some(index))
        .unwrap_or(word.len());
    if first_non_digit > 0 {
        &word[first_non_digit..]
    } else {
        word
    }
}

fn parse_env_assignment(word: &str) -> Option<(&str, &str)> {
    let Some((name, _value)) = word.split_once('=') else {
        return None;
    };
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        || name.as_bytes()[0].is_ascii_digit()
    {
        return None;
    }
    word.split_once('=')
}

fn is_blocked_operation(subcommand: &str, args: &[String]) -> bool {
    match subcommand {
        "checkout" => !args.is_empty(),
        "switch" => true,
        // `git restore` is the modern equivalent of `git checkout -- <path>`:
        // it discards working-tree changes (default / --worktree target).
        // `--staged`-only touches the index but is still a mutating op on the
        // canonical checkout and must be denied for consistency.
        "restore" => true,
        "reset" => args.iter().any(|arg| arg == "--hard"),
        "clean" => args.iter().any(|arg| is_force_clean_arg(arg)),
        "rebase" => true,
        "merge" => merge_is_blocked(args),
        "pull" => pull_is_blocked(args),
        "stash" => args.iter().any(|arg| arg == "pop" || arg == "apply"),
        "branch" => args.iter().any(|arg| is_branch_force_arg(arg)),
        "__shell_alias" => true,
        _ => false,
    }
}

fn merge_is_blocked(args: &[String]) -> bool {
    !has_ff_only(args) || args.iter().any(|arg| is_merge_conflicting_arg(arg))
}

fn pull_is_blocked(args: &[String]) -> bool {
    !has_ff_only(args) || args.iter().any(|arg| is_pull_conflicting_arg(arg))
}

fn has_ff_only(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--ff-only")
}

fn is_merge_conflicting_arg(arg: &str) -> bool {
    matches!(arg, "--no-ff" | "--squash" | "--no-commit" | "--commit")
}

fn is_pull_conflicting_arg(arg: &str) -> bool {
    is_merge_conflicting_arg(arg)
        || arg == "--rebase"
        || arg.starts_with("--rebase=")
        || arg.strip_prefix('-').is_some_and(|rest| {
            !rest.starts_with('-') && !rest.is_empty() && rest.chars().any(|ch| ch == 'r')
        })
}

fn is_branch_force_arg(arg: &str) -> bool {
    if arg == "--force" || arg.starts_with("--force=") {
        return true;
    }
    let Some(rest) = arg.strip_prefix('-') else {
        return false;
    };
    !rest.starts_with('-') && rest.contains('f')
}

fn is_force_clean_arg(arg: &str) -> bool {
    if arg == "--force" {
        return true;
    }
    let Some(rest) = arg.strip_prefix('-') else {
        return false;
    };
    !rest.starts_with('-') && rest.contains('f')
}

fn resolve_target_path(session_cwd: &CwdState, raw_path: &str) -> CwdState {
    if target_path_is_dynamic(raw_path) {
        return CwdState::Unknown;
    }
    let path = Path::new(raw_path);
    if path.is_absolute() {
        return CwdState::Known(normalize_path(path));
    }
    match session_cwd {
        CwdState::Known(cwd) => CwdState::Known(normalize_path(&cwd.join(path))),
        CwdState::Unknown => CwdState::Unknown,
    }
}

fn target_path_is_dynamic(raw_path: &str) -> bool {
    raw_path.starts_with('~')
        || raw_path
            .chars()
            .any(|ch| matches!(ch, '$' | '`' | '*' | '?' | '[' | ']' | '{' | '}'))
}

fn path_is_within(path: &Path, parent: &Path) -> bool {
    path == parent || path.starts_with(parent)
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(Path::new("/")),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    normalized
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ShellToken {
    Word(String),
    Separator(SeparatorKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SeparatorKind {
    Sequence,
    Pipe,
    Background,
    SubshellStart,
    SubshellEnd,
}

impl SeparatorKind {
    fn resets_parent_cwd(self) -> bool {
        matches!(self, SeparatorKind::Pipe | SeparatorKind::Background)
    }
}

fn shell_tokens(command: &str) -> Vec<ShellToken> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut chars = command.chars().peekable();

    while let Some(ch) = chars.next() {
        if let Some(quote_ch) = quote {
            if ch == quote_ch {
                quote = None;
            } else if ch == '\\' {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            } else {
                current.push(ch);
            }
            continue;
        }

        match ch {
            '\'' | '"' => quote = Some(ch),
            '\\' => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            ';' | '\n' | '\r' => {
                push_current_word(&mut tokens, &mut current);
                tokens.push(ShellToken::Separator(SeparatorKind::Sequence));
            }
            '(' => {
                push_current_word(&mut tokens, &mut current);
                tokens.push(ShellToken::Separator(SeparatorKind::SubshellStart));
            }
            ')' => {
                push_current_word(&mut tokens, &mut current);
                tokens.push(ShellToken::Separator(SeparatorKind::SubshellEnd));
            }
            '|' => {
                push_current_word(&mut tokens, &mut current);
                if chars.next_if_eq(&'|').is_some() {
                    tokens.push(ShellToken::Separator(SeparatorKind::Sequence));
                } else {
                    tokens.push(ShellToken::Separator(SeparatorKind::Pipe));
                }
            }
            '&' => {
                if current.ends_with('>') || current.ends_with('<') {
                    current.push(ch);
                    continue;
                }
                if current.is_empty() && chars.peek().is_some_and(|next| *next == '>') {
                    current.push(ch);
                    current.push('>');
                    chars.next();
                    continue;
                }
                push_current_word(&mut tokens, &mut current);
                if chars.next_if_eq(&'&').is_some() {
                    tokens.push(ShellToken::Separator(SeparatorKind::Sequence));
                } else {
                    tokens.push(ShellToken::Separator(SeparatorKind::Background));
                }
            }
            '\t' | ' ' => {
                push_current_word(&mut tokens, &mut current);
            }
            _ => current.push(ch),
        }
    }

    push_current_word(&mut tokens, &mut current);

    tokens
}

fn push_current_word(tokens: &mut Vec<ShellToken>, current: &mut String) {
    if current.is_empty() {
        return;
    }
    tokens.push(ShellToken::Word(std::mem::take(current)));
}

#[cfg(test)]
mod tests {
    use super::*;

    const CANONICAL: &str = "/repo/oyatie";
    const WORKTREE: &str = "/repo/oyatie-worktrees/g011";

    fn input(command: &str, session_cwd: &str, canonical_checkout: Option<&str>) -> DecisionInput {
        input_with_env(command, session_cwd, canonical_checkout, [])
    }

    fn input_with_env<const N: usize>(
        command: &str,
        session_cwd: &str,
        canonical_checkout: Option<&str>,
        process_env: [(&str, &str); N],
    ) -> DecisionInput {
        DecisionInput {
            command: command.to_owned(),
            session_cwd: PathBuf::from(session_cwd),
            canonical_checkout: canonical_checkout.map(PathBuf::from),
            process_env: process_env
                .into_iter()
                .map(|(name, value)| (name.to_owned(), value.to_owned()))
                .collect(),
        }
    }

    fn assert_denied(command: &str) {
        let decision = decide(input(command, CANONICAL, Some(CANONICAL)));
        match decision {
            Decision::Deny { reason } => {
                assert!(reason.contains("worktree policy"));
                assert!(reason.contains("FRIC-022"));
                assert!(reason.contains("FRIC-1781062867"));
            }
            Decision::Allow => panic!("expected deny for {command}"),
        }
    }

    fn assert_allowed(command: &str, cwd: &str, canonical: Option<&str>) {
        assert_eq!(Decision::Allow, decide(input(command, cwd, canonical)));
    }

    #[test]
    fn denies_mutating_git_operations_in_canonical_checkout() {
        for command in [
            "git checkout review-branch",
            "git switch review-branch",
            "git reset --hard HEAD",
            "git clean -fdx",
            "git rebase origin/dev",
            "git merge feature-branch",
            "git merge --ff-only --no-ff feature-branch",
            "git pull",
            "git pull --ff-only --rebase",
            "git stash pop",
            "git stash apply",
            "git branch -f dev HEAD",
        ] {
            assert_denied(command);
        }
    }

    #[test]
    fn denies_unmodelled_wrapper_prefixed_mutations_default_closed() {
        // Wrappers NOT in any allowlist must still fail closed when they carry a
        // canonical-targeted mutating git op (review #685 r2; default-closed
        // scan-through). A hardcoded allowlist always lags the wrapper space.
        for command in [
            "firejail git -C /repo/oyatie switch review-branch",
            "flock /tmp/lock git -C /repo/oyatie reset --hard HEAD",
            "systemd-run git -C /repo/oyatie switch review-branch",
            "cpulimit -l 50 git -C /repo/oyatie checkout review-branch",
            "runuser -u u -- git -C /repo/oyatie reset --hard HEAD",
            "eatmydata git -C /repo/oyatie switch review-branch",
            "proxychains git -C /repo/oyatie checkout review-branch",
            "catchsegv git -C /repo/oyatie reset --hard HEAD",
            "nice -n5 git -C /repo/oyatie switch review-branch",
            "true && nohup git -C /repo/oyatie switch review-branch",
            "timeout 5 nohup git -C /repo/oyatie reset --hard HEAD",
            // Enumerated wrapper leaking via separate-token value flags (F4-R2).
            "xargs -a file git -C /repo/oyatie checkout review-branch",
            "xargs -P 4 -n 1 git -C /repo/oyatie checkout review-branch",
            // git restore (modern checkout --) against canonical.
            "git -C /repo/oyatie restore .",
            // sh -c body must survive quoting across an unmodelled wrapper (r3 F1).
            "firejail sh -c 'git -C /repo/oyatie reset --hard HEAD'",
            "flock /tmp/l sh -c 'git -C /repo/oyatie switch review-branch'",
            // GIT_DIR env-context must survive the wrapper remainder (r3 F2).
            "firejail GIT_DIR=/repo/oyatie/.git git reset --hard HEAD",
            "flock /tmp/l env GIT_DIR=/repo/oyatie/.git git checkout review-branch",
            // Deeply nested unmodelled wrappers must not exhaust the depth budget.
            "firejail flock /tmp/l systemd-run git -C /repo/oyatie reset --hard HEAD",
            // Modeled transparent wrappers + sh -c must also preserve quoting (r3 F1).
            "sudo sh -c 'git -C /repo/oyatie reset --hard HEAD'",
            "nohup sh -c 'git -C /repo/oyatie switch review-branch'",
            "nice -n5 sh -c 'git -C /repo/oyatie checkout review-branch'",
            // Nested escaped-backtick substitution must be recovered (r4 F2).
            "echo `echo \\`git -C /repo/oyatie reset --hard\\``",
            // Substitution-as-command / eval+substitution (r5 F1).
            "$(echo git -C /repo/oyatie reset --hard)",
            "`echo git -C /repo/oyatie reset --hard`",
            "eval $(echo git -C /repo/oyatie reset --hard)",
            "eval `echo git -C /repo/oyatie reset --hard`",
            "bash -c \"$(echo git -C /repo/oyatie reset --hard)\"",
            "eval $(printf 'git -C /repo/oyatie reset --hard')",
            // Expansion-produced command word / subcommand (r5 F2).
            "git -C /repo/oyatie $'reset' --hard",
            "git -C /repo/oyatie re$'set' --hard",
            "g=git; $g -C /repo/oyatie reset --hard",
            "r=reset; git -C /repo/oyatie $r --hard",
            "git -C /repo/oyatie ${x:-reset} --hard",
            "git -C /repo/oyatie \"$(printf 'reset')\" --hard",
            // Nested substitution must resolve to a fixpoint (r6 F1).
            "$(echo $(echo git -C /repo/oyatie reset --hard))",
            "$(echo $(echo git)) -C /repo/oyatie reset --hard",
            "g=$(echo $(echo git)); $g -C /repo/oyatie reset --hard",
            "$(echo $(printf 'git -C /repo/oyatie reset --hard'))",
            // ${} operators beyond :- must resolve or fail closed (r6 F2).
            "git -C /repo/oyatie ${x:=reset} --hard",
            "git -C /repo/oyatie ${x:+reset} --hard",
            "git -C /repo/oyatie ${x+reset} --hard",
            "git -C /repo/oyatie ${x/a/e} --hard",
            "git -C /repo/oyatie ${x:0:5} --hard",
            "git -C /repo/oyatie ${x:-$(echo reset)} --hard",
            // Unresolved command-name expansion targeting canonical (r6 F2).
            "$g -C /repo/oyatie reset --hard",
            // Brace expansion in verb / command-name position (r7 F3).
            "git -C /repo/oyatie {reset,} --hard",
            "git -C /repo/oyatie {,reset} --hard",
            "git -C /repo/oyatie {switch,} other",
            "git -C /repo/oyatie {restore,} .",
            "git -C /repo/oyatie {checkout,} other",
            "git -C /repo/oyatie {clean,} -fdx",
            "git -C /repo/oyatie {stash,} pop",
            "{git,} -C /repo/oyatie {reset,} --hard",
            "git -C /repo/oyatie r{eset,} --hard",
            // IFS word-split resplit, verb side and path side (r8 F4).
            "IFS=x; y=resetx; git -C /repo/oyatie $y --hard",
            "IFS=-; y=reset-; git -C /repo/oyatie $y --hard",
            "IFS=z; c=cleanz; git -C /repo/oyatie $c -fdx",
            "IFS=w; r=restorew; git -C /repo/oyatie $r .",
            "IFS=x; y=resetx; git -C /repo/oyatie ${y} --hard",
            "bash -c 'IFS=x; y=resetx; git -C /repo/oyatie $y --hard'",
            "IFS=x; p=/repo/oyatiex; git -C $p reset --hard",
            // Positional params after set -- (r9 F1).
            "set -- reset --hard; git -C /repo/oyatie $@",
            "set -- clean -fdx; git -C /repo/oyatie $@",
            "bash -c 'set -- reset --hard; git -C /repo/oyatie $@'",
            // ANSI-C hex/octal escapes decode to a real verb (r9 F2).
            "git -C /repo/oyatie $'\\x72\\x65\\x73\\x65\\x74' --hard",
            "git -C /repo/oyatie $'\\162\\145\\163\\145\\164' --hard",
            // Backslash line-continuation reassembles the verb (r9 F3).
            "git -C /repo/oyatie re\\\nset --hard",
            // Positional binding via set -- (r10 F1).
            "set -- -C /repo/oyatie reset --hard; git $@",
            "set -- /repo/oyatie reset --hard; git -C $@",
            "set -- reset --hard; git -C /repo/oyatie $@",
            "set -- clean -fdx; git -C /repo/oyatie $@",
            "set -- --hard; git -C /repo/oyatie reset $1",
            // Shell function argument binding (r10 F1).
            "g(){ git \"$@\"; }; g -C /repo/oyatie reset --hard",
            "g(){ git $@; }; g -C /repo/oyatie reset --hard",
            "g() { git -C /repo/oyatie \"$@\"; }; g reset --hard",
            "function g { git \"$@\"; }; g -C /repo/oyatie reset --hard",
            // r11 ROOT A — quoted top-level positional / array re-split.
            "set -- -C /repo/oyatie reset --hard; git \"$@\"",
            "set -- -C /repo/oyatie reset --hard; git \"${@}\"",
            "read -ra P <<< \"-C /repo/oyatie reset --hard\"; git \"${P[@]}\"",
            "read -a P <<< '-C /repo/oyatie reset --hard'; git \"${P[@]}\"",
            // r11 ROOT B — shift reindex.
            "set -- X -C /repo/oyatie reset --hard; shift; git \"$@\"",
            "set -- X Y -C /repo/oyatie reset --hard; shift 2; git \"$@\"",
            "g(){ shift; git \"$@\"; }; g X -C /repo/oyatie reset --hard",
            // r11 ROOT C — multi-hop function chains.
            "a(){ b \"$@\"; }; b(){ git \"$@\"; }; a -C /repo/oyatie reset --hard",
            "a(){ b \"$@\"; }; b(){ c \"$@\"; }; c(){ git \"$@\"; }; a -C /repo/oyatie reset --hard",
            // r12 — convergent fail-closed: unresolved binding reaches git argv.
            "P=\"-C /repo/oyatie reset --hard\"; git $P",
            "P=(-C /repo/oyatie reset --hard); git \"${P[@]}\"",
            "read -ra P <<< \"-C /repo/oyatie reset --hard\"; n=1; git \"${P[$n]}\" reset --hard",
            "eval \"set -- -C /repo/oyatie reset --hard\"; git \"$@\"",
            "g(){ set -- -C /repo/oyatie reset --hard; git \"$@\"; }; g",
            "r(){ local a=-C; git \"$a\" \"$@\"; }; r /repo/oyatie reset --hard",
            "git \"$@\"",
            // r13 — static substitution into an assignment RHS, then unquoted use.
            "P=$(echo \"reset --hard\"); git -C /repo/oyatie $P",
            "P=$(echo \"-C /repo/oyatie reset --hard\"); git $P",
            "P=$(printf '%s' \"-C /repo/oyatie clean -fdx\"); git $P",
            "A=$(echo \"-C /repo/oyatie reset --hard\"); B=$A; git $B",
            "set -- $(echo \"-C /repo/oyatie reset --hard\"); git \"$@\"",
            // r14 — scalar += append assembles the mutation.
            "P=reset; P+=\" --hard\"; git -C /repo/oyatie $P",
            "P=clean; P+=\" -fdx\"; git -C /repo/oyatie $P",
            "A=-C; A+=\" /repo/oyatie\"; A+=\" reset --hard\"; git $A",
            "P=\"reset --hard\"; git -C /repo/oyatie $P",
            "P+=$(echo \"-C /repo/oyatie reset --hard\"); git $P",
            // r15 — escape-decode assembles whitespace bash word-splits on.
            "git -C /repo/oyatie $(printf \"reset\\t--hard\")",
            "P=reset; P+=$'\\t--hard'; git -C /repo/oyatie $P",
            "P=$(printf \"reset\\t--hard\"); git -C /repo/oyatie $P",
            "git -C /repo/oyatie $(echo -e \"reset\\t--hard\")",
            // r16 — fail-closed value-model allowlist.
            "git -C /repo/oyatie $(printf \"%-10s--hard\" reset)",
            "git -C /repo/oyatie $(printf -- \"%s\\n\" reset --hard)",
            "P=$'reset'$'\\t--hard'; git -C /repo/oyatie $P",
            "P=resetZZhard; git -C /repo/oyatie ${P/ZZ/ --}",
            "shopt -s xpg_echo; git -C /repo/oyatie $(echo \"reset\\t--hard\")",
            // r17 — printf %s with trailing format text + ${V:-default} pre-pass.
            "git -C /repo/oyatie $(printf \"%s --hard\" reset)",
            "git -C /repo/oyatie $(printf \"%s -fdx\" clean)",
            "P=$(printf \"%s --hard\" reset); git -C /repo/oyatie $P",
            "V=reset; git -C /repo/oyatie ${V:-x} --hard",
            "V=reset; git -C /repo/oyatie ${V-x} --hard",
            "W=--hard; git -C /repo/oyatie reset ${W:-x}",
            // r18 — printf with nested sub, chained binding, positional default.
            "git -C /repo/oyatie $(printf \"$(echo reset) --hard\")",
            "V=reset; W=$V; git -C /repo/oyatie ${W:-x} --hard",
            "set -- reset; git -C /repo/oyatie ${1:-x} --hard",
            // r19 F1 — RHS expands at the assignment's execution point: a later
            // reassignment must NOT retro-rewrite an earlier capture (bash binds
            // B=reset here; resolving against the final last-wins map modeled
            // B=log and ALLOWed the wipe).
            "A=reset; B=$A; A=log; git -C /repo/oyatie $B --hard",
            "A=/repo/oyatie; B=$A; A=/tmp/elsewhere; git -C $B reset --hard",
            "A=reset; A+=\" --hard\"; B=$A; A=log; git -C /repo/oyatie $B",
        ] {
            assert_denied(command);
        }
    }

    #[test]
    fn binding_capture_order_is_not_a_false_positive() {
        // The left-fold must keep modeling values faithfully: a capture-then-
        // reassign chain whose CAPTURED value is a read must still ALLOW
        // (review #685 r19 — the inverse of the stale-overwrite bypass).
        for command in [
            ("A=log; B=$A; A=reset; git -C /repo/oyatie $B", CANONICAL),
            (
                "A=sta; B=${A}tus; A=reset; git -C /repo/oyatie $B",
                CANONICAL,
            ),
        ] {
            assert_allowed(command.0, command.1, Some(CANONICAL));
        }
    }

    #[test]
    fn multiword_binding_reads_are_not_false_positives() {
        // A multi-word value that forms a READ must ALLOW (review #685 r14):
        // model the value faithfully, do not blanket-deny `$P`.
        for command in [
            (
                "P=log; P+=\" --oneline\"; git -C /repo/oyatie $P",
                CANONICAL,
            ),
            ("P=\"log --oneline\"; git -C /repo/oyatie $P", CANONICAL),
            ("F=status; git -C /repo/oyatie $F", CANONICAL),
        ] {
            assert_allowed(command.0, command.1, Some(CANONICAL));
        }
    }

    #[test]
    fn brace_and_glob_in_argument_position_are_not_false_positives() {
        // Metacharacters in NON-verb positions (paths, refs, pathspecs of a READ)
        // must not deny — the fail-closed rule is verb/command-name-position only.
        for command in [
            ("git -C /repo/oyatie log {a,b}", CANONICAL),
            ("git -C /repo/oyatie diff HEAD~{1,2}", CANONICAL),
            ("git -C /repo/oyatie show *.rs", CANONICAL),
            // Mutations in a NON-canonical worktree still ALLOW even with braces.
            (
                "git -C /repo/oyatie-worktrees/g1 {reset,} --hard",
                "/repo/oyatie-worktrees/g1",
            ),
        ] {
            assert_allowed(command.0, command.1, Some(CANONICAL));
        }
    }

    #[test]
    fn depth_exhaustion_fails_closed_not_open() {
        // 33 nested wrappers exceed the recursion budget (32); the unevaluated
        // deeper command must DENY (fail closed), never fall through to ALLOW
        // (review #685 r4 F1). Real commands never nest this deep.
        let deep = "nice ".repeat(33);
        assert_denied(&format!("{deep}git -C /repo/oyatie reset --hard HEAD"));
        let mixed = "nohup sudo ".repeat(17);
        assert_denied(&format!("{mixed}git -C /repo/oyatie switch review-branch"));
    }

    #[test]
    fn allows_unmodelled_wrappers_without_mutating_git_no_false_positive() {
        // Default-closed must NOT start denying genuine non-mutating commands.
        for command in [
            ("firejail ls -la", CANONICAL),
            ("systemd-run echo hello", CANONICAL),
            ("grep -r git /repo/oyatie", CANONICAL),
            ("firejail git -C /repo/oyatie status", CANONICAL),
            (
                "flock /tmp/lock git -C /repo/oyatie fetch origin",
                CANONICAL,
            ),
            // A canonical-targeted mutation from a NON-canonical worktree stays ALLOW.
            (
                "firejail git -C /repo/oyatie-worktrees/g1 switch foo",
                "/repo/oyatie-worktrees/g1",
            ),
        ] {
            assert_allowed(command.0, command.1, Some(CANONICAL));
        }
    }

    #[test]
    fn allows_read_only_and_merge_train_safe_git_operations_in_canonical_checkout() {
        for command in [
            "git fetch origin dev",
            "git pull --ff-only",
            "git log --oneline -5",
            "git show HEAD",
            "git diff origin/dev...HEAD",
            "git status --short",
            "git worktree add ../lane branch",
            "git worktree remove ../lane",
            "git worktree list",
            "git branch -D stale-branch",
            "git branch -d merged-branch",
            "git push -u origin agent/g011-checkout-guard",
            "git merge --ff-only origin/dev",
        ] {
            assert_allowed(command, CANONICAL, Some(CANONICAL));
        }
    }

    #[test]
    fn git_dash_c_controls_effective_target() {
        assert_denied("git -C /repo/oyatie switch review-branch");
        assert_allowed(
            "git -C /repo/oyatie-worktrees/g011 switch review-branch",
            CANONICAL,
            Some(CANONICAL),
        );
    }

    #[test]
    fn git_work_tree_and_git_dir_options_control_effective_target() {
        assert_denied(
            "git --git-dir /repo/oyatie/.git --work-tree /repo/oyatie switch review-branch",
        );
        assert_denied(
            "git --git-dir=/repo/oyatie/.git --work-tree=/repo/oyatie switch review-branch",
        );
        assert_denied("git --git-dir /repo/oyatie/.git switch review-branch");
        assert_denied("git --git-dir=/repo/oyatie/.git switch review-branch");
        assert_denied("git --git-dir /repo/oyatie/.git/worktrees/g011 switch review-branch");
        assert_denied(
            "git --git-dir /repo/oyatie/.git -C /repo/oyatie-worktrees/g011 switch review-branch",
        );
        assert_denied(
            "git -C /repo/oyatie --work-tree /repo/oyatie-worktrees/g011 switch review-branch",
        );
        assert_denied(
            "git --work-tree /repo/oyatie-worktrees/g011 --git-dir /repo/oyatie/.git switch review-branch",
        );
        assert_allowed(
            "git --git-dir /repo/oyatie/.git/worktrees/g011 --work-tree /repo/oyatie-worktrees/g011 switch review-branch",
            CANONICAL,
            Some(CANONICAL),
        );
    }

    #[test]
    fn common_command_prefixes_do_not_bypass_guard() {
        assert_denied("env GIT_CONFIG_NOSYSTEM=1 git switch review-branch");
        assert_denied(
            "env GIT_DIR=/repo/oyatie/.git GIT_WORK_TREE=/repo/oyatie git switch review-branch",
        );
        assert_denied(
            "GIT_DIR=/repo/oyatie/.git GIT_WORK_TREE=/repo/oyatie git switch review-branch",
        );
        assert_denied(r#"env GIT_WORK_TREE="$(printf /repo/oyatie)" git switch review-branch"#);
        assert_denied("env -i git reset --hard HEAD");
        assert_denied("env -P /usr/bin git -C /repo/oyatie switch review-branch");
        assert_denied("/usr/bin/env -P /usr/bin git -C /repo/oyatie switch review-branch");
        assert_denied("env -P/usr/bin git -C /repo/oyatie switch review-branch");
        assert_denied("env -v git -C /repo/oyatie switch review-branch");
        assert_denied("env -uPATH git -C /repo/oyatie switch review-branch");
        assert_denied("command git checkout review-branch");
        assert_denied("command -p git -C /repo/oyatie switch review-branch");
        assert_denied("command -- git -C /repo/oyatie switch review-branch");
        assert_denied("exec git checkout review-branch");
        assert_denied("exec -a disguised git -C /repo/oyatie switch review-branch");
        assert_denied("exec -adisguised git -C /repo/oyatie switch review-branch");
        assert_denied("exec -cl git -C /repo/oyatie switch review-branch");
        assert_denied("exec -lc git -C /repo/oyatie switch review-branch");
        assert_denied("exec -ca disguised git -C /repo/oyatie switch review-branch");
        assert_denied("exec -la disguised git -C /repo/oyatie switch review-branch");
        assert_denied("exec -cla disguised git -C /repo/oyatie switch review-branch");
        assert_denied("exec -- git -C /repo/oyatie switch review-branch");
        assert_denied("builtin git checkout review-branch");
        assert_denied("builtin cd /repo/oyatie && git checkout review-branch");
        assert_denied("command /usr/bin/git checkout review-branch");
        assert_denied("/usr/bin/git -C /repo/oyatie switch review-branch");
        assert_denied("./git -C /repo/oyatie switch review-branch");
        assert_denied("git -C /repo/oyatie -c advice.detachedHead=false switch review-branch");
    }

    #[test]
    fn shell_redirections_do_not_hide_command_words() {
        for command in [
            ">/dev/null git -C /repo/oyatie switch review-branch",
            "git >/dev/null -C /repo/oyatie switch review-branch",
            "git -C /repo/oyatie >/dev/null switch review-branch",
            "git -C /repo/oyatie 2>&1 switch review-branch",
            "git -C /repo/oyatie < /dev/null switch review-branch",
            "exec >/dev/null git -C /repo/oyatie switch review-branch",
        ] {
            assert_denied(command);
        }
    }

    #[test]
    fn git_aliases_do_not_hide_blocked_subcommands() {
        assert_denied("git -C /repo/oyatie -c alias.co=checkout co review-branch");
        assert_denied("git -C /repo/oyatie -c 'alias.co=checkout review-branch' co");
        assert_denied("git -C /repo/oyatie -c alias.sw=switch sw review-branch");
        assert_denied("git -C /repo/oyatie -c 'alias.rh=reset --hard' rh HEAD");
        assert_denied("git -C /repo/oyatie -c 'alias.cl=clean -fdx' cl");
        assert_denied("git -C /repo/oyatie -c 'alias.sp=stash pop' sp");
        assert_denied("git -C /repo/oyatie -c 'alias.co=!git checkout' co review-branch");
        assert_denied("git -C /repo/oyatie --config-env=alias.co=GIT_ALIAS co review-branch");
        assert_denied(
            "git -C /repo/oyatie-worktrees/g011 -c 'alias.co=!git -C /repo/oyatie switch' co review-branch",
        );
        assert_eq!(
            Decision::Deny {
                reason: DENY_REASON.to_owned(),
            },
            decide(input_with_env(
                "git -C /repo/oyatie-worktrees/g011 --config-env=alias.co=GIT_ALIAS co review-branch",
                WORKTREE,
                Some(CANONICAL),
                [("GIT_ALIAS", "!git -C /repo/oyatie switch")]
            ))
        );
        assert_denied(
            "GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=alias.co GIT_CONFIG_VALUE_0='!git -C /repo/oyatie switch' git -C /repo/oyatie-worktrees/g011 co review-branch",
        );
        assert_denied(
            "GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=alias.rh GIT_CONFIG_VALUE_0='reset --hard' git -C /repo/oyatie rh HEAD",
        );
        assert_eq!(
            Decision::Deny {
                reason: DENY_REASON.to_owned(),
            },
            decide(input_with_env(
                "git -C /repo/oyatie --config-env=alias.rh=GIT_ALIAS rh HEAD",
                WORKTREE,
                Some(CANONICAL),
                [("GIT_ALIAS", "reset --hard")]
            ))
        );
        assert_allowed(
            "git -C /repo/oyatie -c alias.st=status st --short",
            WORKTREE,
            Some(CANONICAL),
        );
        assert_allowed(
            "git -C /repo/oyatie-worktrees/g011 --config-env=alias.st=GIT_ALIAS st --short",
            WORKTREE,
            Some(CANONICAL),
        );
    }

    #[test]
    fn env_chdir_prefix_controls_only_prefixed_git_command() {
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "env -C /repo/oyatie git switch review-branch",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "env -S git -C /repo/oyatie switch review-branch",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "env -S'git -C /repo/oyatie switch review-branch'",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "/usr/bin/env -S git -C /repo/oyatie switch review-branch",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "dash -c 'git -C /repo/oyatie switch review-branch'",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "env --chdir=/repo/oyatie git switch review-branch",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_allowed(
            "env -C /repo/oyatie-worktrees/g011 git switch review-branch",
            CANONICAL,
            Some(CANONICAL),
        );
        assert_allowed(
            "env GIT_DIR=/repo/oyatie/.git/worktrees/g011 GIT_WORK_TREE=/repo/oyatie-worktrees/g011 git switch review-branch",
            CANONICAL,
            Some(CANONICAL),
        );
        assert_allowed(
            "env -C /repo/oyatie git status; git switch review-branch",
            WORKTREE,
            Some(CANONICAL),
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "/usr/bin/env -C /repo/oyatie git switch review-branch",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
    }

    #[test]
    fn cd_before_git_controls_effective_target_when_git_has_no_dash_c() {
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "cd /repo/oyatie && git switch review-branch",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "cd -P /repo/oyatie && git switch review-branch",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_allowed(
            "cd /repo/oyatie-worktrees/g011 && git switch review-branch",
            WORKTREE,
            Some(CANONICAL),
        );
        assert_allowed(
            "cd /repo/oyatie && git -C /repo/oyatie-worktrees/g011 switch review-branch",
            WORKTREE,
            Some(CANONICAL),
        );
    }

    #[test]
    fn dynamic_target_paths_fail_closed_for_mutating_git() {
        for command in [
            r#"git -C "$(printf /repo/oyatie)" switch review-branch"#,
            r#"cd "$(printf /repo/oyatie)" && git switch review-branch"#,
            r#"env -C "$(printf /repo/oyatie)" git switch review-branch"#,
            r#"git --git-dir="$(printf /repo/oyatie/.git)" switch review-branch"#,
            r#"git -C ~/Developer/oyatie switch review-branch"#,
        ] {
            assert_eq!(
                Decision::Deny {
                    reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
                },
                decide(input(command, WORKTREE, Some(CANONICAL)))
            );
        }
        assert_allowed(
            r#"git -C "$(printf /repo/oyatie)" status --short"#,
            WORKTREE,
            Some(CANONICAL),
        );
    }

    #[test]
    fn pushd_controls_effective_target_and_popd_fails_closed_for_mutations() {
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "pushd /repo/oyatie >/dev/null && git switch review-branch",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_allowed(
            "pushd /repo/oyatie-worktrees/g011 >/dev/null && git switch review-branch",
            WORKTREE,
            Some(CANONICAL),
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input("popd && git switch review-branch", WORKTREE, Some(CANONICAL)))
        );
        assert_allowed(
            "pushd /tmp >/dev/null && popd >/dev/null && git switch review-branch",
            WORKTREE,
            Some(CANONICAL),
        );
    }

    #[test]
    fn cd_dash_returns_to_previous_known_directory() {
        assert_allowed(
            "cd /repo/oyatie && cd - && git switch review-branch",
            WORKTREE,
            Some(CANONICAL),
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "cd /repo/oyatie-worktrees/g011 && cd - && git switch review-branch",
                CANONICAL,
                Some(CANONICAL),
            ))
        );
    }

    #[test]
    fn quoted_cd_target_controls_effective_target() {
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "cd '/repo/oya tie' && git switch review-branch",
                WORKTREE,
                Some("/repo/oya tie"),
            ))
        );
        assert_allowed(
            "cd '/repo/oya tie-worktrees/g011' && git switch review-branch",
            WORKTREE,
            Some("/repo/oya tie"),
        );
    }

    #[test]
    fn cd_tracking_respects_sequence_and_pipeline_separators() {
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "cd /repo/oyatie; git switch review-branch",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "cd /repo/oyatie && git switch review-branch",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_allowed(
            "cd /repo/oyatie | git switch review-branch",
            WORKTREE,
            Some(CANONICAL),
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "{ cd /repo/oyatie; git switch review-branch; }",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "if cd /repo/oyatie; then git switch review-branch; fi",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "cd /repo/oyatie; (cd /repo/oyatie-worktrees/g011); git switch review-branch",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_allowed(
            "cd /repo/oyatie-worktrees/g011; (cd /repo/oyatie); git switch review-branch",
            WORKTREE,
            Some(CANONICAL),
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "(cd /repo/oyatie; git switch review-branch)",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
    }

    #[test]
    fn shell_wrappers_are_recursively_evaluated() {
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "bash -lc \"git -C /repo/oyatie switch review-branch\"",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "sh -c 'git -C /repo/oyatie switch review-branch'",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "/usr/bin/env bash -lc \"git -C /repo/oyatie switch review-branch\"",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "env -S 'bash -lc \"git -C /repo/oyatie switch review-branch\"'",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "eval \"git -C /repo/oyatie switch review-branch\"",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "function f { git -C /repo/oyatie switch review-branch; }; f",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "function f { cd /repo/oyatie; git switch review-branch; }; f",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_allowed(
            "bash -lc \"git -C /repo/oyatie-worktrees/g011 switch review-branch\"",
            WORKTREE,
            Some(CANONICAL),
        );
    }

    // F1: transparent process-runner wrapper bypass — every command from the
    // reviewer's confirmed-bypass list must now DENY (FRIC-022/FRIC-1781062867).
    #[test]
    fn transparent_wrapper_prefix_bypass_is_denied() {
        // nohup passes its argument directly to exec
        assert_denied("nohup git -C /repo/oyatie switch foo");
        // nice adjusts scheduling priority, then execs the command
        assert_denied("nice git -C /repo/oyatie switch foo");
        // timeout wraps with a duration then execs the command
        assert_denied("timeout 5 git -C /repo/oyatie reset --hard HEAD");
        // stdbuf rebuffers stdio then execs the command
        assert_denied("stdbuf -oL git -C /repo/oyatie switch foo");
        // setsid creates a new session, then execs the command
        assert_denied("setsid git -C /repo/oyatie switch foo");
        // xargs feeds stdin arguments to the git command; at runtime it always
        // supplies at least one argument from stdin so checkout is always armed
        assert_denied("echo x | xargs git -C /repo/oyatie checkout");
        // xargs -I{} variant — {} placeholder is already an explicit arg
        assert_denied("echo x | xargs -I{} git -C /repo/oyatie checkout {}");
        // watch executes the command repeatedly
        assert_denied("watch git -C /repo/oyatie switch foo");
        // parallel (GNU) feeds ::: arguments to the command template
        assert_denied("parallel git -C /repo/oyatie checkout ::: foo");
        // Subshell around nohup wrapper
        assert_denied("(nohup git -C /repo/oyatie switch foo)");
        // Piped prefix before timeout wrapper (reset --hard is always blocked)
        assert_denied("echo foo | timeout 5 git -C /repo/oyatie reset --hard HEAD");
        // && chain with nohup wrapper
        assert_denied("true && nohup git -C /repo/oyatie switch foo");
        // Leading env assignments before nohup wrapper
        assert_denied("A=1 B=2 nohup git -C /repo/oyatie switch foo");
    }

    // F2: git restore is a mutating working-tree command equivalent to
    // `git checkout -- <path>` and must be denied in the canonical checkout.
    #[test]
    fn git_restore_is_denied_in_canonical_checkout() {
        // bare restore targeting cwd (canonical)
        assert_denied("git restore .");
        // restore with explicit -C pointing at canonical
        assert_denied("git -C /repo/oyatie restore .");
        // restore with explicit file target
        assert_denied("git restore src/lib.rs");
        // restore --worktree is still a working-tree mutation
        assert_denied("git restore --worktree .");
        // restore --staged touches the index — also blocked for consistency
        assert_denied("git restore --staged .");
        // restore via a transparent wrapper must also be caught
        assert_denied("nohup git -C /repo/oyatie restore .");
        // restore in the worktree (non-canonical) must be ALLOWED
        assert_allowed("git restore .", WORKTREE, Some(CANONICAL));
    }

    #[test]
    fn command_substitutions_are_recursively_evaluated() {
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                ": \"$(git -C /repo/oyatie switch review-branch)\"",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                r#"echo "don't $(git -C /repo/oyatie switch review-branch)""#,
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                ": `git -C /repo/oyatie switch review-branch`",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
        assert_allowed(
            "printf '$(git -C /repo/oyatie switch review-branch)'",
            WORKTREE,
            Some(CANONICAL),
        );
    }

    #[test]
    fn commands_outside_canonical_checkout_are_allowed() {
        assert_allowed("git switch review-branch", WORKTREE, Some(CANONICAL));
        assert_allowed("git reset --hard HEAD", WORKTREE, Some(CANONICAL));
    }

    #[test]
    fn missing_canonical_checkout_means_fail_open_for_worktrees() {
        assert_allowed("git switch review-branch", WORKTREE, None);
    }

    #[test]
    fn ignores_git_words_that_are_not_shell_commands() {
        assert_allowed("echo git switch review-branch", CANONICAL, Some(CANONICAL));
        assert_allowed("printf 'git reset --hard HEAD'", CANONICAL, Some(CANONICAL));
    }

    #[test]
    fn canonical_subdirectories_are_guarded() {
        assert_denied("git checkout review-branch");
        assert_eq!(
            Decision::Deny {
                reason: "worktree policy: mutating git command denied in canonical checkout for FRIC-022/FRIC-1781062867".to_owned(),
            },
            decide(input(
                "git -C /repo/oyatie/docs switch review-branch",
                WORKTREE,
                Some(CANONICAL),
            ))
        );
    }

    #[test]
    fn extracts_bash_command_from_claude_or_codex_payload() {
        let payload = r#"{"tool_name":"Bash","tool_input":{"command":"git status --short"}}"#;
        assert_eq!(
            Some("git status --short".to_owned()),
            extract_command_from_hook_payload(payload)
        );
    }

    #[test]
    fn extracts_bash_command_from_hook_payload_envelope_variants() {
        for payload in [
            r#"{"input":{"command":"git -C /repo/oyatie switch review-branch"}}"#,
            r#"{"parameters":{"command":"git -C /repo/oyatie switch review-branch"}}"#,
        ] {
            assert_eq!(
                Some("git -C /repo/oyatie switch review-branch".to_owned()),
                extract_command_from_hook_payload(payload)
            );
        }
    }

    #[test]
    fn extracts_command_from_flat_payload_as_compatibility_fallback() {
        let payload = r#"{"command":"git fetch origin dev"}"#;
        assert_eq!(
            Some("git fetch origin dev".to_owned()),
            extract_command_from_hook_payload(payload)
        );
    }

    #[test]
    fn default_canonical_checkout_uses_git_common_dir_fallback() {
        assert_eq!(
            Some(PathBuf::from(CANONICAL)),
            default_canonical_checkout(Path::new(CANONICAL), Path::new(".git"))
        );
        assert_eq!(
            Some(PathBuf::from(CANONICAL)),
            default_canonical_checkout(Path::new(WORKTREE), Path::new("/repo/oyatie/.git"))
        );
        assert_eq!(
            None,
            default_canonical_checkout(
                Path::new(WORKTREE),
                Path::new("/repo/oyatie/.git/worktrees/g011")
            )
        );
    }
}
