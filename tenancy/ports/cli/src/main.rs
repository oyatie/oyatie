//! Tenant-facing `oya` CLI surface per ADR-0167.
//!
//! # Scope
//!
//! This crate ships the TENANT-facing CLI surface. It is distinct from
//! the internal `oya-dev-cli` per ADR-0167 §"Decision":
//!
//! - Tier-A semver-protected per ADR-0037.
//! - Depends on the public SDK only (no `oya-check-*`, no `oya-foundry-*`).
//! - Distributed to tenants via Homebrew tap, apt repo, winget, ghcr.
//! - Built in the workspace as `oya-tenant` to avoid colliding with
//!   the internal `oya-dev-cli` binary target.
//!
//! # Skeleton scope (v0.1)
//!
//! Per ADR-0167 §"Migration / rollout plan" Phase 1, the v0.1 surface
//! ships `auth`, `version`, `completion` only. Remaining command groups
//! (`workflow`, `messenger`, `tasks`, `foundry`, `ontology`, `audit`,
//! `webhook`, `status`) ship at M01.5 / M02 / M03.
//!
//! Each subcommand currently prints an `unimplemented` notice referring
//! the user to the ADR. Full impl tracked under
//! `registry/placeholder-debt/adr-follow-ups.yaml#adr-0167-tenant-cli-commands`.
//!
//! # Naming justification
//!
//! The crate de-brands to `tenancy-cli` (ADR-0562 capability-first home
//! tenancy/ports/cli); the tenant-facing binary NAME `oya-tenant` is
//! deliberately preserved (Tier-A distribution channel id, ADR-0167) and
//! distinguishes this CLI from the internal `oya-dev-cli`.
//!
//! # Hardening (2026-08-20)
//!
//! The command surface is retirement-marked per the repository
//! `cli_surface_policy`, so this pass adds NO command and NO flag. It
//! adds the test suite the skeleton never had and closes four defects.
//! Three of them are BEHAVIOUR CHANGES on a Tier-A semver-protected
//! binary and need a release note; they are listed here rather than
//! implied.
//!
//! 1. **`--output` is a closed value set.** It was typed `String`, so
//!    `--output yaml` parsed clean and was then silently discarded — a
//!    tenant script asked for a format it never got and was never told
//!    about. It now parses into [`OutputFormat`], whose value set is
//!    exactly the three spellings the flag's help text has always
//!    documented. **Behaviour change:** `oya --output yaml version`
//!    previously exited 0 and now exits 1.
//!
//! 2. **`OYA_OUTPUT` is resolved by this crate, not by clap's `env`.**
//!    An earlier revision of this hardening pass kept clap's
//!    `env = "OYA_OUTPUT"` while closing the value set. That combination
//!    is wrong in three separate ways, each verified against the built
//!    binary, and each is now pinned by a regression test. First, clap
//!    treats a set-but-EMPTY variable as a supplied value, so the
//!    ubiquitous unset-passthrough shape (`docker run -e OYA_OUTPUT`, a
//!    blank CI variable, `export OYA_OUTPUT="$OYA_OUTPUT"`) aborted EVERY
//!    command, `version` and `completion` included. Second, clap
//!    validates the env value even when the flag is present, so
//!    `OYA_OUTPUT=yaml oya --output json version` also failed — that
//!    inverts ADR-0167 §"Configuration precedence" (flag highest) and
//!    leaves no command-line escape from a stale exported variable.
//!    Third, clap's message names only `--output <OUTPUT>` and never
//!    names the variable that actually supplied the value.
//!    [`resolve_output`] now implements the documented precedence
//!    directly: an explicit flag wins outright and the environment is
//!    not consulted at all; an unset, empty or whitespace-only variable
//!    means "unset" and falls through to the `human` default; any other
//!    value is parsed by the same case-sensitive parser the flag uses,
//!    and a value it rejects is a typed [`OutputEnvError`] whose message
//!    names `OYA_OUTPUT`, the offending value and the escape hatch.
//!    `--help` names the variable too, since a diagnosis an operator
//!    cannot follow up is only half a fix: the flag's help is now
//!    hand-written ([`OUTPUT_LONG_HELP`]) rather than harvested from a
//!    doc comment that carries rustdoc link syntax.
//!    **Behaviour change:** `OYA_OUTPUT=yaml oya version` previously
//!    exited 0 and now exits 1. `OYA_OUTPUT= oya version` is unchanged
//!    (exit 0), and is a regression test.
//!
//! 3. **Parse failures exit 1, not 2.** ADR-0167 §"Output contract"
//!    line 97 reserves `2` for a SERVER error and marks that class
//!    retryable, so a tenant CI wrapper that retries on 2 would retry a
//!    permanently-typo'd flag until its budget was gone. `main` now maps
//!    parse outcomes through [`parse_exit_code`]: `--help` / `--version`
//!    exit 0, every other parse failure is a user error and exits 1.
//!    **Behaviour change:** this also moves clap's PRE-EXISTING parse
//!    errors off 2 — `oya definitely-not-a-command` exited 2 before this
//!    change and exits 1 after it. That is the ADR's stated contract,
//!    but it is a change to invocations that predate this pass.
//!
//! 4. **The unimplemented notice no longer echoes user values.** It
//!    `Debug`-printed the whole parsed subcommand, so message bodies,
//!    search queries and the free-form `--args` JSON blob went to stderr
//!    verbatim — into CI job logs, journald and an operator's scrollback
//!    (`oya messenger send --body "SSN ..."` reproduced it). The notice
//!    now renders [`Command::path`], a `&'static str` built only from
//!    command names, so no user-supplied value can reach the stream by
//!    construction. This is an ADR-0008 data-use-boundary fix, not a
//!    cosmetic one.
//!
//! # Gaps
//!
//! Deliberately deferred, so that the deferral is visible rather than
//! implied:
//!
//! - **Command bodies.** Every group except `version` and `completion`
//!   still prints the skeleton notice and exits 1. Tracked under
//!   `registry/placeholder-debt/adr-follow-ups.yaml#adr-0167-tenant-cli-commands`;
//!   implementing them here would grow a retirement-marked surface.
//! - **`output` is resolved and validated, then dropped.** Nothing
//!   downstream renders through it: the only two live commands have
//!   fixed output, and re-shaping `version` into JSON would be a further
//!   behaviour change this pass does not take. Resolving it at startup
//!   is what turns a bad `OYA_OUTPUT` into a diagnosable exit-1 instead
//!   of a value that is accepted and ignored.
//! - **Config-file precedence is not implemented.** ADR-0167
//!   §"Configuration precedence" rungs 3 and 4 (`./.oya/config.toml`,
//!   `$XDG_CONFIG_HOME/oya/config.toml`) are not read. [`resolve_output`]
//!   implements rungs 1, 2 and 5 only, and takes the environment as a
//!   PARAMETER so the file rungs can slot in without touching `main`.
//! - **No process-exit tests.** `main` returns `ExitCode`, which cannot
//!   be inspected in-process, and a binary-only crate cannot host a
//!   `tests/` integration target without a lib target (adding one is
//!   forbidden here). The exit contract is instead pinned as a pure
//!   function, [`parse_exit_code`], tested against the real
//!   [`clap::error::ErrorKind`] values the parser produces; only the
//!   two-line hand-off from `main` into it is unpinned.
//! - **`OYA_OUTPUT` is exercised as a value, not as a real process
//!   variable.** `std::env::set_var` is `unsafe` on edition 2024 and
//!   this crate is `#![forbid(unsafe_code)]`, and mutating the real
//!   environment would race the other tests in this binary. The tests
//!   call [`resolve_output`] with the exact `Option<&OsStr>` that
//!   `std::env::var_os` yields, which is the whole of the env path bar
//!   that one library call.
//! - **The buck2 lane runs none of these tests.** `BUCK` declares only
//!   `rust_binary`, which does not build inline `#[cfg(test)]` modules;
//!   a companion `rust_test` target is the repo convention. It is NOT
//!   added here because this lane is forbidden to edit BUCK files.
//!   Under ADR-0716 cargo is the merge path, so the tests do gate the
//!   PR; the buck2 local-hermeticity smoke covers zero of them.
//!
//! # References
//!
//! - ADR-0167 — tenant-facing CLI binary `oya` (this skeleton).
//! - ADR-0037 — public API stability tiers (Tier-A semver).
//! - ADR-0120 — Rust-first on-prem tooling authority.

// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::ffi::OsStr;
use std::io::stdout;
use std::process::ExitCode;

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};

/// The environment variable that supplies `--output` when the flag is
/// absent, per ADR-0167 §"Configuration precedence" rung 2.
const OUTPUT_ENV: &str = "OYA_OUTPUT";

/// The `--output` long help, exactly as `oya --help` renders it.
///
/// Written as a `concat!` of whole lines rather than a `\`-continued
/// literal: rustfmt rejoins a continued literal and bakes the source
/// indentation into the string, which reaches the tenant's terminal.
const OUTPUT_LONG_HELP: &str = concat!(
    "Output format: human (default), json, or ndjson.\n",
    "\n",
    "When --output is omitted, the OYA_OUTPUT environment variable supplies the ",
    "value. An explicit --output always wins over OYA_OUTPUT, and an unset or ",
    "empty OYA_OUTPUT selects human (ADR-0167 configuration precedence)."
);

/// ADR-0167 §"Output contract": exit `0` on success.
const EXIT_SUCCESS: u8 = 0;

/// ADR-0167 §"Output contract": exit `1` on a USER error.
///
/// `2` is reserved for a server error and is the retryable class, so no
/// locally-diagnosable failure may use it — a tenant wrapper that retries
/// on `2` would retry a typo forever.
const EXIT_USER_ERROR: u8 = 1;

/// How much of a rejected `OYA_OUTPUT` value the diagnostic echoes back.
///
/// Enough to recognise a typo, bounded so that a variable holding a large
/// or accidental blob does not spill it into a log.
const MAX_ECHOED_ENV_VALUE: usize = 32;

/// Tenant-facing `oya` CLI per ADR-0167.
#[derive(Parser, Debug)]
#[command(name = "oya", version, about = "Oyatie tenant CLI (ADR-0167)")]
struct Cli {
    /// Output format: `human` (default), `json`, or `ndjson`.
    ///
    /// Typed as a closed [`OutputFormat`] rather than a free-form
    /// `String`: an unrecognised format is a diagnosed error instead of a
    /// value that is accepted and then quietly ignored.
    ///
    /// When the flag is omitted, the `OYA_OUTPUT` environment variable
    /// supplies it (ADR-0167 §"Configuration precedence"); the flag wins
    /// whenever it is present, and an empty `OYA_OUTPUT` counts as unset.
    /// The variable is read by [`resolve_output`] rather than by clap's
    /// `env` support — see the module header for why.
    ///
    /// `help` / `long_help` are written out rather than harvested from
    /// this doc comment: a tenant reading `oya --help` must not be shown
    /// rustdoc link syntax or the name of a private function. The
    /// long form names `OYA_OUTPUT`, because clap's own rejection message
    /// for this argument names only `--output` and a triaging operator
    /// otherwise has nothing pointing at the environment.
    #[arg(
        long,
        global = true,
        value_name = "FORMAT",
        help = "Output format: human (default), json, or ndjson",
        long_help = OUTPUT_LONG_HELP
    )]
    output: Option<OutputFormat>,

    #[command(subcommand)]
    command: Command,
}

/// The closed set of renderings `--output` / `OYA_OUTPUT` may select.
///
/// This is the value set the flag's help text has always documented;
/// naming it as a type is what makes an unrecognised format a diagnosed
/// error rather than a silently discarded string.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    /// Human-readable text for an interactive terminal (the default).
    Human,
    /// A single JSON document per invocation.
    Json,
    /// Newline-delimited JSON, one record per line, for streaming.
    Ndjson,
}

impl OutputFormat {
    /// The format used when neither the flag nor the environment selects
    /// one — ADR-0167 §"Configuration precedence" rung 5.
    const DEFAULT: Self = Self::Human;

    /// The canonical spelling accepted on the command line.
    ///
    /// This is the single source of truth for the flag's rendering: the
    /// [`core::fmt::Display`] impl and the help text both route through
    /// it, so the documented default can never drift from a value the
    /// parser accepts.
    const fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Json => "json",
            Self::Ndjson => "ndjson",
        }
    }

    /// Parse one spelling exactly as the `--output` flag path parses it.
    ///
    /// `ignore_case` is `false`, matching clap's derive default, so the
    /// environment can never admit a spelling the flag rejects (or the
    /// reverse). Both paths call THIS function, so the two cannot drift.
    fn parse_spelling(raw: &str) -> Option<Self> {
        <Self as ValueEnum>::from_str(raw, false).ok()
    }
}

impl core::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Why an `OYA_OUTPUT` value could not be turned into an [`OutputFormat`].
///
/// A plain enum with a hand-written [`core::fmt::Display`]: the message is
/// the whole point of the type, because clap's own wording names only
/// `--output <OUTPUT>` and never the variable that supplied the value.
#[derive(Clone, Debug, Eq, PartialEq)]
enum OutputEnvError {
    /// The variable held a value outside the closed set.
    Unrecognised {
        /// The rejected spelling, already bounded for logging.
        value: String,
    },
    /// The variable held bytes that are not valid UTF-8, so no spelling
    /// can be recovered from it to quote back.
    NotUtf8,
}

impl core::fmt::Display for OutputEnvError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unrecognised { value } => write!(
                f,
                "{OUTPUT_ENV} is set to `{value}`, which is not a valid output format\n  \
                 possible values: {}\n  \
                 unset {OUTPUT_ENV}, or pass `--output <FORMAT>`, which takes precedence \
                 over it (ADR-0167 §\"Configuration precedence\")",
                accepted_formats()
            ),
            Self::NotUtf8 => write!(
                f,
                "{OUTPUT_ENV} holds bytes that are not valid UTF-8\n  \
                 possible values: {}\n  \
                 unset {OUTPUT_ENV}, or pass `--output <FORMAT>`, which takes precedence \
                 over it (ADR-0167 §\"Configuration precedence\")",
                accepted_formats()
            ),
        }
    }
}

impl std::error::Error for OutputEnvError {}

/// The accepted `--output` spellings, rendered for a diagnostic.
fn accepted_formats() -> String {
    OutputFormat::value_variants()
        .iter()
        .map(|variant| variant.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Bound a rejected environment value before it reaches a log stream.
///
/// Truncation is by CHARACTER, never by byte, so a multi-byte value can
/// not be split mid-codepoint.
fn truncate_for_diagnostic(value: &str) -> String {
    if value.chars().count() <= MAX_ECHOED_ENV_VALUE {
        return value.to_owned();
    }
    let head: String = value.chars().take(MAX_ECHOED_ENV_VALUE).collect();
    format!("{head}…")
}

/// Resolve the effective output format under ADR-0167
/// §"Configuration precedence".
///
/// Rung 1 (the flag) beats rung 2 (the environment) beats rung 5 (the
/// hardcoded default). The environment arrives as a PARAMETER — exactly
/// the `Option<&OsStr>` `std::env::var_os` yields — so the resolution is
/// deterministic and testable without mutating the process environment.
///
/// An explicit `flag` short-circuits: the environment is not consulted at
/// all, so a stale exported `OYA_OUTPUT` always has a command-line escape.
/// An absent, empty or whitespace-only variable counts as UNSET, because
/// `OYA_OUTPUT=` is the ubiquitous unset-passthrough shape rather than a
/// deliberate request for a format named "".
///
/// # Errors
///
/// [`OutputEnvError`] when the variable holds a non-empty value that the
/// flag path would also reject, or bytes that are not UTF-8.
fn resolve_output(
    flag: Option<OutputFormat>,
    env_value: Option<&OsStr>,
) -> Result<OutputFormat, OutputEnvError> {
    if let Some(format) = flag {
        return Ok(format);
    }
    let Some(raw) = env_value else {
        return Ok(OutputFormat::DEFAULT);
    };
    let Some(raw) = raw.to_str() else {
        return Err(OutputEnvError::NotUtf8);
    };
    if raw.trim().is_empty() {
        return Ok(OutputFormat::DEFAULT);
    }
    OutputFormat::parse_spelling(raw).ok_or_else(|| OutputEnvError::Unrecognised {
        value: truncate_for_diagnostic(raw),
    })
}

/// The ADR-0167 exit code for a clap parse outcome.
///
/// `--help` and `--version` reach us as `Err`, but they are successful
/// invocations and exit `0`. Every other parse failure is a USER error and
/// exits `1`: clap's own default is `2`, which ADR-0167 reserves for the
/// retryable server-error class.
const fn parse_exit_code(kind: ErrorKind) -> u8 {
    match kind {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => EXIT_SUCCESS,
        _ => EXIT_USER_ERROR,
    }
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Authentication commands per ADR-0167 OAuth-2.1 device-code flow.
    #[command(subcommand)]
    Auth(AuthCommand),

    /// Workflow run + status commands (M01.5).
    #[command(subcommand)]
    Workflow(WorkflowCommand),

    /// Messenger send + search commands (M01.5).
    #[command(subcommand)]
    Messenger(MessengerCommand),

    /// Tasks create + list commands (M01.5).
    #[command(subcommand)]
    Tasks(TasksCommand),

    /// Foundry capability invoke + list commands (M02).
    #[command(subcommand)]
    Foundry(FoundryCommand),

    /// Ontology entity get commands (M02).
    #[command(subcommand)]
    Ontology(OntologyCommand),

    /// Audit-chain query commands (M03).
    #[command(subcommand)]
    Audit(AuditCommand),

    /// Webhook endpoint + delivery commands (M02.5, depends on ADR-0169).
    #[command(subcommand)]
    Webhook(WebhookCommand),

    /// Public status surface (M03, depends on ADR-0168).
    Status,

    /// Print build version + API version compatibility.
    Version,

    /// Emit shell-completion scripts.
    Completion { shell: Shell },
}

impl Command {
    /// The space-separated command path, e.g. `messenger send`.
    ///
    /// Every arm is a `&'static str` literal, so this CANNOT carry a
    /// user-supplied value. That is the point: the skeleton notice used to
    /// `Debug`-print the parsed subcommand, which put message bodies,
    /// search queries and the `--args` blob on stderr verbatim
    /// (ADR-0008 data-use boundary). Rendering a static path instead makes
    /// that exposure impossible by construction rather than by review.
    fn path(&self) -> &'static str {
        match self {
            Self::Auth(AuthCommand::Login) => "auth login",
            Self::Auth(AuthCommand::Logout) => "auth logout",
            Self::Auth(AuthCommand::Whoami) => "auth whoami",
            Self::Workflow(WorkflowCommand::Run { .. }) => "workflow run",
            Self::Workflow(WorkflowCommand::Status { .. }) => "workflow status",
            Self::Messenger(MessengerCommand::Send { .. }) => "messenger send",
            Self::Messenger(MessengerCommand::Search { .. }) => "messenger search",
            Self::Tasks(TasksCommand::Create { .. }) => "tasks create",
            Self::Tasks(TasksCommand::List) => "tasks list",
            Self::Foundry(FoundryCommand::Capability(FoundryCapabilityCommand::Invoke {
                ..
            })) => "foundry capability invoke",
            Self::Foundry(FoundryCommand::Capability(FoundryCapabilityCommand::List)) => {
                "foundry capability list"
            }
            Self::Ontology(OntologyCommand::Entity(OntologyEntityCommand::Get { .. })) => {
                "ontology entity get"
            }
            Self::Audit(AuditCommand::Chain(AuditChainCommand::Query { .. })) => {
                "audit chain query"
            }
            Self::Webhook(WebhookCommand::ListDeliveries { .. }) => "webhook list-deliveries",
            Self::Webhook(WebhookCommand::Retry { .. }) => "webhook retry",
            Self::Status => "status",
            Self::Version => "version",
            Self::Completion { .. } => "completion",
        }
    }
}

#[derive(Subcommand, Debug)]
enum AuthCommand {
    /// OAuth-2.1 device-code login (RFC 8628).
    Login,
    /// Revoke local token + clear OS credential store.
    Logout,
    /// Show current tenant + principal.
    Whoami,
}

#[derive(Subcommand, Debug)]
enum WorkflowCommand {
    Run { flow_id: String },
    Status { run_id: String },
}

#[derive(Subcommand, Debug)]
enum MessengerCommand {
    Send {
        #[arg(long)]
        to: String,
        #[arg(long)]
        body: String,
    },
    Search {
        #[arg(long)]
        query: String,
    },
}

#[derive(Subcommand, Debug)]
enum TasksCommand {
    Create {
        #[arg(long)]
        title: String,
    },
    List,
}

#[derive(Subcommand, Debug)]
enum FoundryCommand {
    #[command(subcommand)]
    Capability(FoundryCapabilityCommand),
}

#[derive(Subcommand, Debug)]
enum FoundryCapabilityCommand {
    Invoke {
        capability_id: String,
        #[arg(long)]
        args: String,
    },
    List,
}

#[derive(Subcommand, Debug)]
enum OntologyCommand {
    #[command(subcommand)]
    Entity(OntologyEntityCommand),
}

#[derive(Subcommand, Debug)]
enum OntologyEntityCommand {
    Get { urn: String },
}

#[derive(Subcommand, Debug)]
enum AuditCommand {
    #[command(subcommand)]
    Chain(AuditChainCommand),
}

#[derive(Subcommand, Debug)]
enum AuditChainCommand {
    Query {
        #[arg(long)]
        since: String,
    },
}

#[derive(Subcommand, Debug)]
enum WebhookCommand {
    ListDeliveries {
        #[arg(long)]
        endpoint_id: String,
    },
    Retry {
        #[arg(long)]
        delivery_id: String,
    },
}

/// The stderr notice for a command the skeleton does not implement yet.
///
/// Rendered from [`Command::path`] only, so no argument value the tenant
/// typed can reach the stream. Kept as a pure function so the redaction is
/// asserted by a test rather than trusted.
fn unimplemented_notice(command: &Command) -> String {
    format!(
        "oya: error: command not yet implemented in skeleton: {}\n\
         see ADR-0167 §\"Migration / rollout plan\" for the schedule",
        command.path()
    )
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            // `error.print()` writes help/version to stdout and a real
            // failure to stderr. A broken stdout must not change the exit
            // code we report, so the write result is deliberately dropped.
            drop(error.print());
            return ExitCode::from(parse_exit_code(error.kind()));
        }
    };

    // ADR-0167 §"Configuration precedence" rungs 1/2/5. The resolved value
    // is validated and then dropped: no command renders through it yet
    // (see the `Gaps` section). Validating it here is what turns a stale
    // `OYA_OUTPUT` into a diagnosed exit-1 instead of a silent no-op.
    if let Err(error) = resolve_output(cli.output, std::env::var_os(OUTPUT_ENV).as_deref()) {
        eprintln!("oya: error: {error}");
        return ExitCode::from(EXIT_USER_ERROR);
    }

    match cli.command {
        Command::Version => {
            println!("oya {} (ADR-0167 skeleton)", env!("CARGO_PKG_VERSION"));
            ExitCode::from(EXIT_SUCCESS)
        }
        Command::Completion { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "oya", &mut stdout());
            ExitCode::from(EXIT_SUCCESS)
        }
        other => {
            eprintln!("{}", unimplemented_notice(&other));
            // Exit-code 1 per ADR-0167 §"Output contract" (user error).
            // Once impl lands, this dispatches to the public SDK. Full impl
            // tracked under registry/placeholder-debt/adr-follow-ups.yaml
            //   #adr-0167-tenant-cli-commands.
            ExitCode::from(EXIT_USER_ERROR)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi::OsString;

    /// Parse an argv the way the real binary would, program name included.
    fn try_parse(argv: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(argv)
    }

    /// Parse an argv that MUST succeed, reporting the clap error verbatim
    /// when it does not.
    fn parse(argv: &[&str]) -> Cli {
        match try_parse(argv) {
            Ok(cli) => cli,
            Err(error) => panic!("expected {argv:?} to parse, clap said: {error}"),
        }
    }

    /// The [`ErrorKind`] clap rejects an argv with, failing loudly if the
    /// argv unexpectedly parsed. Asserting the KIND — not merely "an error
    /// happened" — is what keeps these from passing for the wrong reason.
    fn reject_kind(argv: &[&str]) -> ErrorKind {
        match try_parse(argv) {
            Err(error) => error.kind(),
            Ok(cli) => panic!("expected {argv:?} to be rejected, it parsed as {cli:?}"),
        }
    }

    /// The `--output` argument as clap sees it, for wiring assertions that
    /// do not need a parse.
    fn output_arg() -> clap::Arg {
        Cli::command()
            .get_arguments()
            .find(|arg| arg.get_id() == "output")
            .expect("the `--output` argument is part of the published surface")
            .clone()
    }

    /// Resolve `OYA_OUTPUT` from a string exactly as `main` would, i.e.
    /// through the same `Option<&OsStr>` `std::env::var_os` yields.
    fn resolve_env(value: &str) -> Result<OutputFormat, OutputEnvError> {
        let owned = OsString::from(value);
        resolve_output(None, Some(owned.as_os_str()))
    }

    /// The bash completion script, as `oya completion bash` emits it.
    fn bash_completion_script() -> String {
        let mut buffer: Vec<u8> = Vec::new();
        let mut command = Cli::command();
        generate(Shell::Bash, &mut command, "oya", &mut buffer);
        String::from_utf8(buffer).expect("completion scripts are UTF-8")
    }

    #[test]
    fn command_definition_is_internally_consistent() {
        // clap's own sanity check: catches a malformed derive configuration
        // (duplicate ids, conflicting defaults, an arg that can never be
        // reached) that would otherwise only surface at runtime.
        Cli::command().debug_assert();
    }

    #[test]
    fn binary_presents_itself_as_oya() {
        let command = Cli::command();
        assert_eq!(
            command.get_name(),
            "oya",
            "ADR-0167 pins the tenant-facing command name to `oya`, \
             independent of the `oya-tenant` cargo bin target"
        );
        assert!(
            command.get_version().is_some(),
            "`--version` must resolve; the Tier-A channel keys off it"
        );
    }

    #[test]
    fn parses_every_auth_subcommand() {
        assert!(matches!(
            parse(&["oya", "auth", "login"]).command,
            Command::Auth(AuthCommand::Login)
        ));
        assert!(matches!(
            parse(&["oya", "auth", "logout"]).command,
            Command::Auth(AuthCommand::Logout)
        ));
        assert!(matches!(
            parse(&["oya", "auth", "whoami"]).command,
            Command::Auth(AuthCommand::Whoami)
        ));
    }

    #[test]
    fn parses_workflow_run_and_status_with_their_positionals() {
        match parse(&["oya", "workflow", "run", "flow-42"]).command {
            Command::Workflow(WorkflowCommand::Run { flow_id }) => {
                assert_eq!(flow_id, "flow-42");
            }
            other => panic!("expected `workflow run`, got {other:?}"),
        }

        match parse(&["oya", "workflow", "status", "run-7"]).command {
            Command::Workflow(WorkflowCommand::Status { run_id }) => {
                assert_eq!(run_id, "run-7");
            }
            other => panic!("expected `workflow status`, got {other:?}"),
        }
    }

    #[test]
    fn parses_messenger_send_and_search_with_their_flags() {
        match parse(&[
            "oya",
            "messenger",
            "send",
            "--to",
            "user@tenant.example",
            "--body",
            "hello there",
        ])
        .command
        {
            Command::Messenger(MessengerCommand::Send { to, body }) => {
                assert_eq!(to, "user@tenant.example");
                // A body with a space must survive as ONE value, not be
                // re-split into positionals.
                assert_eq!(body, "hello there");
            }
            other => panic!("expected `messenger send`, got {other:?}"),
        }

        match parse(&["oya", "messenger", "search", "--query", "invoice"]).command {
            Command::Messenger(MessengerCommand::Search { query }) => {
                assert_eq!(query, "invoice");
            }
            other => panic!("expected `messenger search`, got {other:?}"),
        }
    }

    #[test]
    fn parses_tasks_create_and_list() {
        match parse(&["oya", "tasks", "create", "--title", "renew cert"]).command {
            Command::Tasks(TasksCommand::Create { title }) => assert_eq!(title, "renew cert"),
            other => panic!("expected `tasks create`, got {other:?}"),
        }

        assert!(matches!(
            parse(&["oya", "tasks", "list"]).command,
            Command::Tasks(TasksCommand::List)
        ));
    }

    #[test]
    fn parses_nested_foundry_capability_group() {
        match parse(&[
            "oya",
            "foundry",
            "capability",
            "invoke",
            "cap-9",
            "--args",
            "{}",
        ])
        .command
        {
            Command::Foundry(FoundryCommand::Capability(FoundryCapabilityCommand::Invoke {
                capability_id,
                args,
            })) => {
                assert_eq!(capability_id, "cap-9");
                assert_eq!(args, "{}");
            }
            other => panic!("expected `foundry capability invoke`, got {other:?}"),
        }

        assert!(matches!(
            parse(&["oya", "foundry", "capability", "list"]).command,
            Command::Foundry(FoundryCommand::Capability(FoundryCapabilityCommand::List))
        ));
    }

    #[test]
    fn parses_nested_ontology_and_audit_groups() {
        match parse(&["oya", "ontology", "entity", "get", "urn:oya:tenant:ten_1"]).command {
            Command::Ontology(OntologyCommand::Entity(OntologyEntityCommand::Get { urn })) => {
                assert_eq!(urn, "urn:oya:tenant:ten_1");
            }
            other => panic!("expected `ontology entity get`, got {other:?}"),
        }

        match parse(&["oya", "audit", "chain", "query", "--since", "2026-08-01"]).command {
            Command::Audit(AuditCommand::Chain(AuditChainCommand::Query { since })) => {
                assert_eq!(since, "2026-08-01");
            }
            other => panic!("expected `audit chain query`, got {other:?}"),
        }
    }

    #[test]
    fn parses_webhook_delivery_commands() {
        match parse(&["oya", "webhook", "list-deliveries", "--endpoint-id", "ep-3"]).command {
            Command::Webhook(WebhookCommand::ListDeliveries { endpoint_id }) => {
                assert_eq!(endpoint_id, "ep-3");
            }
            other => panic!("expected `webhook list-deliveries`, got {other:?}"),
        }

        match parse(&["oya", "webhook", "retry", "--delivery-id", "dlv-5"]).command {
            Command::Webhook(WebhookCommand::Retry { delivery_id }) => {
                assert_eq!(delivery_id, "dlv-5");
            }
            other => panic!("expected `webhook retry`, got {other:?}"),
        }
    }

    #[test]
    fn parses_leaf_commands_status_version_and_completion() {
        assert!(matches!(parse(&["oya", "status"]).command, Command::Status));
        assert!(matches!(
            parse(&["oya", "version"]).command,
            Command::Version
        ));

        match parse(&["oya", "completion", "zsh"]).command {
            Command::Completion { shell } => assert_eq!(shell, Shell::Zsh),
            other => panic!("expected `completion zsh`, got {other:?}"),
        }
    }

    #[test]
    fn output_is_absent_from_the_parse_when_the_flag_is_omitted() {
        // The flag no longer carries `default_value_t`: the default is
        // applied by `resolve_output`, AFTER the environment has had its
        // turn. A default baked into the parse would make "flag omitted"
        // indistinguishable from "flag set to human" and silently outrank
        // `OYA_OUTPUT`.
        assert_eq!(parse(&["oya", "version"]).output, None);
        assert_eq!(resolve_output(None, None), Ok(OutputFormat::DEFAULT));
        assert_eq!(OutputFormat::DEFAULT, OutputFormat::Human);
    }

    #[test]
    fn output_accepts_each_documented_format() {
        assert_eq!(
            parse(&["oya", "--output", "json", "version"]).output,
            Some(OutputFormat::Json)
        );
        assert_eq!(
            parse(&["oya", "--output", "ndjson", "version"]).output,
            Some(OutputFormat::Ndjson)
        );
        assert_eq!(
            parse(&["oya", "--output", "human", "version"]).output,
            Some(OutputFormat::Human)
        );
    }

    #[test]
    fn output_is_global_so_it_may_follow_the_subcommand() {
        // `global = true` is the whole reason `--output` may appear after
        // the subcommand; without it this argv is an unknown-argument error.
        let cli = parse(&["oya", "auth", "whoami", "--output", "ndjson"]);
        assert_eq!(cli.output, Some(OutputFormat::Ndjson));
        assert!(matches!(cli.command, Command::Auth(AuthCommand::Whoami)));
        assert!(
            output_arg().is_global_set(),
            "`--output` must stay global; tenants pass it after the subcommand"
        );
    }

    #[test]
    fn output_does_not_delegate_the_environment_to_clap() {
        // Regression pin. Declaring `env = "OYA_OUTPUT"` on a closed
        // value-enum arg made clap validate the variable EAGERLY: a
        // set-but-empty variable aborted every command, and a stale value
        // aborted even invocations that passed an explicit `--output`,
        // inverting ADR-0167 §"Configuration precedence". The variable is
        // read by `resolve_output` instead; if this assertion ever fails,
        // all three failures below are back.
        assert_eq!(
            output_arg().get_env(),
            None,
            "`OYA_OUTPUT` must be resolved by `resolve_output`, not by clap's \
             eager `env` validation"
        );
    }

    #[test]
    fn help_tells_the_operator_that_oya_output_feeds_the_flag() {
        // Finding: clap's rejection message for this argument names only
        // `--output`, so an operator who never typed the flag is pointed
        // at a command line where nothing is wrong. `--help` is the other
        // half of the diagnosis and must name the variable.
        let help = Cli::command().render_long_help().to_string();
        assert!(help.contains("OYA_OUTPUT"), "long help was: {help}");
        assert!(
            help.contains("always wins over OYA_OUTPUT"),
            "long help must state the precedence, was: {help}"
        );

        // The help text is hand-written rather than harvested from the doc
        // comment, because the doc comment carries rustdoc link syntax and
        // the name of a private function. Neither belongs in a tenant's
        // terminal.
        for leak in ["[`", "resolve_output", "OutputFormat"] {
            assert!(!help.contains(leak), "`{leak}` leaked into help: {help}");
        }

        // A `\`-continued string literal is rejoined by rustfmt with the
        // source indentation baked in, which reaches the terminal as a run
        // of spaces mid-sentence. `concat!` is what prevents that.
        assert!(
            !OUTPUT_LONG_HELP.contains("  "),
            "long help carries source indentation: {OUTPUT_LONG_HELP:?}"
        );
    }

    #[test]
    fn an_empty_oya_output_is_treated_as_unset() {
        // `OYA_OUTPUT=` is the unset-passthrough shape: `docker run -e
        // OYA_OUTPUT`, a declared-but-blank CI variable, or
        // `export OYA_OUTPUT="$OYA_OUTPUT"`. Under clap's `env` this made
        // EVERY command exit non-zero, `version` and `completion`
        // included. It must resolve to the default instead.
        assert_eq!(resolve_env(""), Ok(OutputFormat::Human));
        assert_eq!(resolve_env("   "), Ok(OutputFormat::Human));
        assert_eq!(resolve_env("\t\n"), Ok(OutputFormat::Human));
    }

    #[test]
    fn a_recognised_oya_output_selects_that_format() {
        assert_eq!(resolve_env("json"), Ok(OutputFormat::Json));
        assert_eq!(resolve_env("ndjson"), Ok(OutputFormat::Ndjson));
        assert_eq!(resolve_env("human"), Ok(OutputFormat::Human));
    }

    #[test]
    fn an_explicit_flag_outranks_the_environment_entirely() {
        // ADR-0167 §"Configuration precedence": the flag is rung 1 and the
        // environment rung 2. A BROKEN variable must not defeat the flag —
        // otherwise a tenant with a stale `export OYA_OUTPUT=yaml` has no
        // command-line escape and cannot even run `oya auth login`.
        let stale = OsString::from("yaml");
        assert_eq!(
            resolve_output(Some(OutputFormat::Json), Some(stale.as_os_str())),
            Ok(OutputFormat::Json)
        );
        let empty = OsString::from("");
        assert_eq!(
            resolve_output(Some(OutputFormat::Ndjson), Some(empty.as_os_str())),
            Ok(OutputFormat::Ndjson)
        );
        assert_eq!(
            resolve_output(Some(OutputFormat::Human), None),
            Ok(OutputFormat::Human)
        );
    }

    #[test]
    fn an_unrecognised_oya_output_is_a_typed_error_naming_the_variable() {
        let error = resolve_env("yaml").expect_err("`yaml` is not an output format");
        assert_eq!(
            error,
            OutputEnvError::Unrecognised {
                value: "yaml".to_owned()
            }
        );

        // clap's own wording names only `--output <OUTPUT>`, which points a
        // triaging operator at a command line where nothing is wrong. The
        // message must name the variable, the value, the accepted set and
        // the escape hatch.
        let message = error.to_string();
        assert!(message.contains("OYA_OUTPUT"), "message was: {message}");
        assert!(message.contains("yaml"), "message was: {message}");
        assert!(message.contains("human, json, ndjson"), "was: {message}");
        assert!(message.contains("--output"), "message was: {message}");
    }

    #[test]
    fn the_environment_admits_exactly_what_the_flag_admits() {
        // Both paths route through `OutputFormat::parse_spelling`, so this
        // cannot drift. Case sensitivity is the observable edge: clap's
        // derive default is `ignore_case = false`, and a proxy parser
        // called with `ignore_case = true` would silently model a WIDER
        // value set than the binary actually has.
        for spelling in ["JSON", "Json", "NDJSON", "Human", "yaml", "text", "0"] {
            assert_eq!(
                reject_kind(&["oya", "--output", spelling, "version"]),
                ErrorKind::InvalidValue,
                "the flag path must reject `{spelling}`"
            );
            assert!(
                resolve_env(spelling).is_err(),
                "the env path must reject `{spelling}` too"
            );
        }
        for spelling in ["human", "json", "ndjson"] {
            let by_flag = parse(&["oya", "--output", spelling, "version"]).output;
            assert_eq!(by_flag, resolve_env(spelling).ok());
        }
    }

    #[test]
    fn a_non_utf8_oya_output_is_reported_rather_than_lost() {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;

            let raw = OsString::from_vec(vec![0x66, 0x80, 0x6f]);
            let error = resolve_output(None, Some(raw.as_os_str()))
                .expect_err("invalid UTF-8 cannot name a format");
            assert_eq!(error, OutputEnvError::NotUtf8);
            let message = error.to_string();
            assert!(message.contains("OYA_OUTPUT"), "message was: {message}");
            assert!(message.contains("UTF-8"), "message was: {message}");
        }
        // On a non-unix target the same value cannot be constructed
        // without unsafe, and this crate forbids it; the UTF-8 arm is
        // still reachable there through a lone surrogate in the real
        // environment, which is why the arm exists.
        assert_eq!(resolve_env("json"), Ok(OutputFormat::Json));
    }

    #[test]
    fn a_huge_oya_output_value_is_bounded_before_it_reaches_a_log() {
        let blob = "z".repeat(4096);
        let error = resolve_env(&blob).expect_err("a 4 KiB blob is not a format");
        let message = error.to_string();
        assert!(
            message.len() < 512,
            "a rejected value must not spill into the log wholesale, got {} bytes",
            message.len()
        );
        assert!(message.contains('…'), "truncation must be visible");

        // Truncation is by character, never by byte: a multi-byte value
        // must not be split mid-codepoint.
        let multibyte = "é".repeat(MAX_ECHOED_ENV_VALUE + 5);
        let truncated = truncate_for_diagnostic(&multibyte);
        assert_eq!(truncated.chars().count(), MAX_ECHOED_ENV_VALUE + 1);
        assert!(truncated.chars().all(|c| c == 'é' || c == '…'));

        // A value at the limit is echoed whole, with no ellipsis.
        let exact = "y".repeat(MAX_ECHOED_ENV_VALUE);
        assert_eq!(truncate_for_diagnostic(&exact), exact);
    }

    #[test]
    fn every_output_variant_round_trips_through_its_own_spelling() {
        // Pins Display against the parser: the documented default renders
        // through Display, so a drift here would make the stated default a
        // value the parser itself rejects.
        for variant in OutputFormat::value_variants() {
            let rendered = variant.to_string();
            assert_eq!(rendered, variant.as_str());
            assert_eq!(
                parse(&["oya", "--output", &rendered, "version"]).output,
                Some(*variant),
                "`{rendered}` must parse back to the variant that rendered it"
            );
            assert_eq!(resolve_env(&rendered), Ok(*variant));
        }
        assert_eq!(accepted_formats(), "human, json, ndjson");
    }

    #[test]
    fn unrecognised_output_format_is_rejected_instead_of_silently_ignored() {
        // Regression pin for the defect this suite exposed: `output` was a
        // `String`, so `--output yaml` parsed clean and was then discarded,
        // leaving a tenant script that asked for a format it never got and
        // was never told about.
        assert_eq!(
            reject_kind(&["oya", "--output", "yaml", "version"]),
            ErrorKind::InvalidValue
        );
        assert_eq!(
            reject_kind(&["oya", "--output", "", "version"]),
            ErrorKind::InvalidValue,
            "an EXPLICIT empty flag value is a typo, unlike an empty OYA_OUTPUT"
        );
    }

    #[test]
    fn parse_failures_use_the_adr_0167_user_error_code() {
        // ADR-0167 §"Output contract" line 97: `1` user error, `2` SERVER
        // error. clap's own default is `2`, which marks a permanent local
        // typo as the retryable class — a tenant CI wrapper that retries on
        // `2` would burn its budget on a misspelled flag.
        for argv in [
            vec!["oya", "--output", "yaml", "version"],
            vec!["oya", "definitely-not-a-command"],
            vec!["oya", "workflow", "run"],
            vec!["oya", "--outupt", "json", "version"],
            vec!["oya"],
        ] {
            let kind = reject_kind(&argv);
            assert_eq!(
                parse_exit_code(kind),
                EXIT_USER_ERROR,
                "{argv:?} is a user error, not a server error"
            );
        }
    }

    #[test]
    fn help_and_version_requests_are_successful_invocations() {
        // clap reports both as `Err`; exiting non-zero for `oya --help`
        // would break `set -e` scripts and shell completion installers.
        for argv in [
            vec!["oya", "--help"],
            vec!["oya", "-h"],
            vec!["oya", "auth", "--help"],
            vec!["oya", "--version"],
        ] {
            let kind = reject_kind(&argv);
            assert_eq!(
                parse_exit_code(kind),
                EXIT_SUCCESS,
                "{argv:?} must exit 0, clap reported {kind:?}"
            );
        }
    }

    #[test]
    fn unknown_subcommand_is_rejected() {
        assert_eq!(
            reject_kind(&["oya", "definitely-not-a-command"]),
            ErrorKind::InvalidSubcommand
        );
        assert_eq!(
            reject_kind(&["oya", "auth", "reauthenticate"]),
            ErrorKind::InvalidSubcommand,
            "an unknown leaf under a real group must not fall back to the group"
        );
    }

    #[test]
    fn missing_required_arguments_are_rejected() {
        assert_eq!(
            reject_kind(&["oya", "workflow", "run"]),
            ErrorKind::MissingRequiredArgument,
            "a missing positional must not default to an empty flow id"
        );
        assert_eq!(
            reject_kind(&["oya", "messenger", "send", "--to", "user@tenant.example"]),
            ErrorKind::MissingRequiredArgument,
            "a send with no body must not go out"
        );
        assert_eq!(
            reject_kind(&["oya", "audit", "chain", "query"]),
            ErrorKind::MissingRequiredArgument,
            "an unbounded audit query must not be the default"
        );
    }

    #[test]
    fn a_bad_completion_shell_is_rejected() {
        assert_eq!(
            reject_kind(&["oya", "completion", "powershel"]),
            ErrorKind::InvalidValue
        );
        assert_eq!(
            reject_kind(&["oya", "completion"]),
            ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn an_unknown_flag_is_rejected() {
        assert_eq!(
            reject_kind(&["oya", "--outupt", "json", "version"]),
            ErrorKind::UnknownArgument
        );
    }

    #[test]
    fn the_unimplemented_notice_never_echoes_a_user_supplied_value() {
        // ADR-0008 data-use boundary. The notice used to `Debug`-print the
        // whole parsed subcommand, so a message body, a search query or the
        // `--args` blob went to stderr verbatim — into CI logs, journald
        // and an operator's scrollback. Every secret below is a value the
        // old notice printed in full.
        let cases: Vec<(Vec<&str>, &str, Vec<&str>)> = vec![
            (
                vec![
                    "oya",
                    "messenger",
                    "send",
                    "--to",
                    "patient@clinic.example",
                    "--body",
                    "SSN 123-45-6789 card 4111111111111111",
                ],
                "messenger send",
                vec!["123-45-6789", "4111111111111111", "patient@clinic.example"],
            ),
            (
                vec![
                    "oya",
                    "foundry",
                    "capability",
                    "invoke",
                    "cap-9",
                    "--args",
                    "{\"api_key\":\"sk-live-DEADBEEF\"}",
                ],
                "foundry capability invoke",
                vec!["sk-live-DEADBEEF", "api_key", "cap-9"],
            ),
            (
                vec!["oya", "messenger", "search", "--query", "oncology results"],
                "messenger search",
                vec!["oncology"],
            ),
            (
                vec!["oya", "ontology", "entity", "get", "urn:oya:patient:p_77"],
                "ontology entity get",
                vec!["p_77"],
            ),
            (
                vec!["oya", "tasks", "create", "--title", "call 555-0134"],
                "tasks create",
                vec!["555-0134"],
            ),
        ];

        for (argv, expected_path, secrets) in cases {
            let command = parse(&argv).command;
            assert_eq!(command.path(), expected_path);
            let notice = unimplemented_notice(&command);
            for secret in secrets {
                assert!(
                    !notice.contains(secret),
                    "notice for {argv:?} leaked `{secret}`: {notice}"
                );
            }
            assert!(notice.contains(expected_path), "notice was: {notice}");
            assert!(notice.contains("ADR-0167"), "notice was: {notice}");
            // A `{` is the tell of a `Debug` rendering of a struct-like
            // variant; the static text has none. (It DOES contain `"`,
            // around the ADR section name, so quotes prove nothing.)
            assert!(!notice.contains('{'), "notice was: {notice}");
        }

        // The decisive property: the notice is a function of the command
        // PATH alone. Two invocations of the same command carrying
        // completely different argument values must render identically —
        // an assertion the old `Debug` notice could never satisfy.
        let benign = parse(&["oya", "messenger", "send", "--to", "a", "--body", "b"]).command;
        let sensitive = parse(&[
            "oya",
            "messenger",
            "send",
            "--to",
            "patient@clinic.example",
            "--body",
            "SSN 123-45-6789",
        ])
        .command;
        assert_eq!(
            unimplemented_notice(&benign),
            unimplemented_notice(&sensitive),
            "the notice must not vary with the values the tenant typed"
        );
    }

    #[test]
    fn every_command_path_is_a_distinct_static_name() {
        let argvs: Vec<Vec<&str>> = vec![
            vec!["oya", "auth", "login"],
            vec!["oya", "auth", "logout"],
            vec!["oya", "auth", "whoami"],
            vec!["oya", "workflow", "run", "f1"],
            vec!["oya", "workflow", "status", "r1"],
            vec!["oya", "messenger", "send", "--to", "a", "--body", "b"],
            vec!["oya", "messenger", "search", "--query", "q"],
            vec!["oya", "tasks", "create", "--title", "t"],
            vec!["oya", "tasks", "list"],
            vec![
                "oya",
                "foundry",
                "capability",
                "invoke",
                "c1",
                "--args",
                "{}",
            ],
            vec!["oya", "foundry", "capability", "list"],
            vec!["oya", "ontology", "entity", "get", "u1"],
            vec!["oya", "audit", "chain", "query", "--since", "s"],
            vec!["oya", "webhook", "list-deliveries", "--endpoint-id", "e1"],
            vec!["oya", "webhook", "retry", "--delivery-id", "d1"],
            vec!["oya", "status"],
            vec!["oya", "version"],
            vec!["oya", "completion", "bash"],
        ];

        let mut paths: Vec<&'static str> = Vec::new();
        for argv in &argvs {
            let path = parse(argv).command.path();
            assert!(
                !path.is_empty() && path.is_ascii(),
                "{argv:?} produced an unusable path `{path}`"
            );
            paths.push(path);
        }
        let mut sorted = paths.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            paths.len(),
            "two commands share a path, so the notice cannot say which ran: {paths:?}"
        );
    }

    #[test]
    fn completion_generates_a_non_empty_script_per_shell() {
        let mut scripts = Vec::new();
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
            let mut buffer: Vec<u8> = Vec::new();
            let mut command = Cli::command();
            generate(shell, &mut command, "oya", &mut buffer);

            let script = String::from_utf8(buffer).expect("completion scripts are UTF-8");
            assert!(
                !script.trim().is_empty(),
                "{shell} completion generated an empty script"
            );
            assert!(
                script.contains("oya"),
                "{shell} completion must bind to the `oya` binary name"
            );
            scripts.push(script);
        }

        // Distinct shells must not collapse to the same text — that would
        // mean the shell argument is being ignored.
        assert_ne!(scripts[0], scripts[1]);
        assert_ne!(scripts[1], scripts[2]);
    }

    #[test]
    fn completion_scripts_cover_the_whole_command_surface() {
        let script = bash_completion_script();

        for group in [
            "auth",
            "workflow",
            "messenger",
            "tasks",
            "foundry",
            "ontology",
            "audit",
            "webhook",
            "status",
            "version",
            "completion",
        ] {
            // Match the generated dispatch token `oya__subcmd__<group>`,
            // not the bare word. A bare `contains("version")` was vacuous:
            // `#[command(version)]` puts a `--version` FLAG in every
            // script, so that assertion held even with the `version`
            // SUBCOMMAND deleted from the enum.
            let token = format!("oya__subcmd__{}", group.replace('-', "__"));
            assert!(
                script.contains(&token),
                "generated completion omits the `{group}` command group (`{token}`)"
            );
        }
    }

    #[test]
    fn the_completion_coverage_token_is_falsifiable() {
        // Guards the test above from regressing into the vacuous form: the
        // token must be absent for a command that does not exist, and must
        // NOT be satisfied by the `--version` flag alone.
        let script = bash_completion_script();
        assert!(!script.contains("oya__subcmd__definitely_not_a_command"));
        assert!(
            script.contains("--version"),
            "the `--version` flag is present, which is exactly why the bare \
             word `version` could not prove the SUBCOMMAND exists"
        );
    }
}
