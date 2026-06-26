//! PII/secret **redactor** guardrail adapter for cloud-intelligence.
//!
//! Implements the kernel [`Guardrail`] port with a production redaction engine
//! and exposes the same engine as a **non-bypassable** [`RedactingEventSink`]
//! decorator at the [`EventSink`] boundary, so no downstream sink (ClickHouse,
//! Valkey, in-process log) can ever persist a secret.
//!
//! ## What it strips
//!
//! - Bearer / `Authorization` tokens, JWTs, and common API-key shapes
//!   (`sk-…`, `ghp_…`, `AKIA…`, `xoxb-…`, `AIza…`).
//! - Absolute filesystem paths (`/Users/…`, `/home/…`, `C:\…`) that leak host layout.
//! - Host/environment context (`<env>…</env>` blocks, `Platform:` / `Working
//!   directory:` lines).
//! - `gitStatus:` working-tree dumps.
//!
//! Each match is replaced with a `[REDACTED:<label>]` marker; the original
//! secret never appears in the output, nor in any [`GuardFinding`] (findings
//! carry only the class, label, and occurrence count).
//!
//! ## Design
//!
//! Redaction is pure, deterministic CPU work — no network, clock, or randomness
//! — so the [`Guardrail`] impl never blocks: it sanitizes and returns
//! [`GuardOutcome::redacted`] (which collapses to `allow` when nothing matched).
//! Blocking verdicts are the job of other guardrails (e.g. moderation) composed
//! via [`intelligence_kernel::guardrail::GuardrailChain`].
//!
//! ADR-0083 Tier-3 panic-free: no `unwrap`/`expect`/`panic!` outside tests.
//! Regex patterns are compiled fallibly at first use; a malformed pattern is
//! dropped rather than panicking (a unit test asserts every pattern compiles).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::sync::LazyLock;

use intelligence_kernel::guardrail::{
    GuardClass, GuardContent, GuardContext, GuardFinding, GuardFuture, GuardOutcome, Guardrail,
};
use intelligence_kernel::{EventSink, LlmGatewayEvent};
use regex::Regex;

// ---------------------------------------------------------------------------
// Pattern catalogue
// ---------------------------------------------------------------------------

/// (class, machine-label, regex source). Order matters: the most specific /
/// most sensitive credential patterns run first so a token embedded in an
/// `Authorization:` line is redacted before the line-level fallback sees it.
///
/// The `regex` crate has no look-around or backreferences; every pattern below
/// stays within that subset.
const PATTERN_SPECS: &[(GuardClass, &str, &str)] = &[
    // `Bearer <token>` — run before the authorization-line fallback so the
    // token itself (not just the word "Bearer") is removed.
    (
        GuardClass::Credential,
        "bearer-token",
        r"(?i)\bbearer\s+[A-Za-z0-9._~+/=-]+",
    ),
    // `Authorization: <value>` / `Authorization=<value>`.
    (
        GuardClass::Credential,
        "authorization",
        r#"(?i)\bauthorization\s*[:=]\s*[^\s,;"']+"#,
    ),
    // JSON Web Tokens (three base64url segments with an `eyJ` header).
    (
        GuardClass::Credential,
        "jwt",
        r"\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b",
    ),
    // Common provider API-key shapes.
    // ponytail: prefix+length heuristic; broaden the alternation as new
    // provider key formats appear rather than reaching for an entropy scanner.
    (
        GuardClass::Credential,
        "api-key",
        r"(?:sk-[A-Za-z0-9_-]{16,}|gh[pousr]_[A-Za-z0-9]{16,}|AKIA[0-9A-Z]{16}|xox[baprs]-[A-Za-z0-9-]{10,}|AIza[0-9A-Za-z_-]{20,})",
    ),
    // Email addresses (PII).
    (
        GuardClass::PersonalData,
        "email",
        r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b",
    ),
    // Harness-injected host/environment block.
    (GuardClass::HostContext, "host-context-env", r"(?s)<env>.*?</env>"),
    // `gitStatus:` dump — greedily to the next blank line or end of input.
    // ponytail: paragraph-bounded (`\n\n` or end); if a porcelain dump is
    // separated by single newlines from following prose, widen the terminator.
    (
        GuardClass::RepositoryState,
        "git-status",
        r"(?s)gitStatus:.*?(?:\n\n|\z)",
    ),
    // Absolute filesystem paths (unix well-known roots + Windows drive paths).
    (
        GuardClass::FilesystemPath,
        "filesystem-path",
        r#"(?:/(?:Users|home|root|var|private|tmp|etc|opt|mnt|srv|usr)/[^\s'"`)\]]*|[A-Za-z]:\\[^\s'"`)\]]*)"#,
    ),
    // Stand-alone host-context key lines (outside an `<env>` block).
    (
        GuardClass::HostContext,
        "host-context-line",
        r"(?im)^[ \t]*(?:Platform|OS Version|Working directory|Is directory a git repo|Today's date|Model ID)\s*:\s*.+$",
    ),
];

struct CompiledPattern {
    class: GuardClass,
    label: &'static str,
    re: Regex,
}

static PATTERNS: LazyLock<Vec<CompiledPattern>> = LazyLock::new(|| {
    PATTERN_SPECS
        .iter()
        .filter_map(|&(class, label, src)| {
            Regex::new(src).ok().map(|re| CompiledPattern { class, label, re })
        })
        .collect()
});

/// Replacement marker for a redacted match. Contains the class label only —
/// never the matched secret value.
fn marker(label: &str) -> String {
    format!("[REDACTED:{label}]")
}

/// Strip every known secret/PII/host pattern from `input`.
///
/// Returns the sanitized text and a secret-free list of findings (class, label,
/// occurrence count). Deterministic and side-effect-free.
pub fn redact(input: &str) -> (String, Vec<GuardFinding>) {
    let mut text = input.to_string();
    let mut findings: Vec<GuardFinding> = Vec::new();

    for pattern in PATTERNS.iter() {
        let count = pattern.re.find_iter(&text).count();
        if count == 0 {
            continue;
        }
        // Replacement string is a fixed literal with no `$` — no capture-group
        // expansion can occur, so the secret cannot leak through `replace_all`.
        text = pattern
            .re
            .replace_all(&text, marker(pattern.label).as_str())
            .into_owned();
        findings.push(GuardFinding::new(pattern.class, pattern.label, count));
    }

    (text, findings)
}

// ---------------------------------------------------------------------------
// Guardrail impl
// ---------------------------------------------------------------------------

/// The production PII/secret redactor as a [`Guardrail`].
///
/// Sanitizes both request (`pre_call`) and response (`post_call`) content.
/// Redaction is non-destructive: it never blocks the call, it removes secrets
/// and lets sanitized content proceed.
#[derive(Clone, Debug)]
pub struct RedactorGuardrail {
    name: &'static str,
}

impl Default for RedactorGuardrail {
    fn default() -> Self {
        Self {
            name: "pii-secret-redactor",
        }
    }
}

impl RedactorGuardrail {
    pub fn new() -> Self {
        Self::default()
    }

    fn guard(content: &GuardContent) -> GuardOutcome {
        let (sanitized, findings) = redact(content.as_str());
        GuardOutcome::redacted(GuardContent::new(sanitized), findings)
    }
}

impl Guardrail for RedactorGuardrail {
    fn name(&self) -> &str {
        self.name
    }

    fn pre_call<'a>(
        &'a self,
        content: &'a GuardContent,
        _ctx: &'a GuardContext,
    ) -> GuardFuture<'a> {
        let outcome = Self::guard(content);
        Box::pin(async move { outcome })
    }

    fn post_call<'a>(
        &'a self,
        content: &'a GuardContent,
        _ctx: &'a GuardContext,
    ) -> GuardFuture<'a> {
        let outcome = Self::guard(content);
        Box::pin(async move { outcome })
    }
}

// ---------------------------------------------------------------------------
// Non-bypassable EventSink redaction stage
// ---------------------------------------------------------------------------

/// Mandatory redaction stage at the [`EventSink`] boundary.
///
/// Wraps any inner [`EventSink`] and scrubs the free-text fields of every
/// [`LlmGatewayEvent`] before delegating. The inner sink is private: the only
/// path to it is through redaction, so no sink wrapped this way can persist a
/// secret. Compose this around the fan-out at the composition root so *every*
/// downstream sink inherits the guarantee.
#[derive(Clone, Debug)]
pub struct RedactingEventSink<S> {
    inner: S, // data_class: INTERNAL_ONLY
}

impl<S> RedactingEventSink<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }

    /// Consume the wrapper and return the inner sink (e.g. for shutdown).
    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S: EventSink> EventSink for RedactingEventSink<S> {
    fn emit(&self, mut event: LlmGatewayEvent) {
        // Only the free-text fields can carry adversary-influenced content.
        let (request_id, _) = redact(&event.request_id);
        let (model, _) = redact(&event.model);
        event.request_id = request_id;
        event.model = model;
        self.inner.emit(event);
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pattern_compiles() {
        // The lazy filter_map silently drops malformed patterns; this guards
        // against a typo shipping a non-compiling (and thus inert) pattern.
        assert_eq!(
            PATTERNS.len(),
            PATTERN_SPECS.len(),
            "a PATTERN_SPECS regex failed to compile"
        );
    }

    #[test]
    fn strips_bearer_token_and_keeps_no_secret() {
        let secret = "Authorization: Bearer eyJabc123.def456.ghi789";
        let (out, findings) = redact(secret);
        assert!(!out.contains("eyJabc123"), "token leaked: {out}");
        assert!(!out.contains("def456"), "token leaked: {out}");
        assert!(!findings.is_empty());
        // No finding embeds the secret.
        for f in &findings {
            assert!(!f.label.contains("eyJabc123"));
        }
    }

    #[test]
    fn strips_api_keys() {
        for key in [
            "sk-ant-api03-AAAAAAAAAAAAAAAAAAAAAAAA",
            "ghp_AAAAAAAAAAAAAAAAAAAAAAAA",
            "AKIAIOSFODNN7EXAMPLE",
            "xoxb-1234567890-abcdefghij",
            "AIzaSyDxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        ] {
            let (out, findings) = redact(&format!("key={key} end"));
            assert!(!out.contains(key), "leaked {key}: {out}");
            assert!(out.contains("[REDACTED:"));
            assert!(!findings.is_empty());
        }
    }

    #[test]
    fn strips_unix_and_windows_paths() {
        let (out, _) = redact("see /Users/jason/secret/file.rs and C:\\Users\\jason\\x.txt");
        assert!(!out.contains("/Users/jason"), "{out}");
        assert!(!out.contains("C:\\Users"), "{out}");
    }

    #[test]
    fn strips_env_block_and_git_status() {
        let input = "before <env>Working directory: /Users/x\nOS Version: Darwin</env> mid\n\
                     gitStatus: M file.rs\n D other.rs\n\nafter";
        let (out, findings) = redact(input);
        assert!(!out.contains("Darwin"), "env leaked: {out}");
        assert!(!out.contains("file.rs"), "git status leaked: {out}");
        assert!(out.contains("before"));
        assert!(out.contains("after"));
        let labels: Vec<&str> = findings.iter().map(|f| f.label.as_str()).collect();
        assert!(labels.contains(&"host-context-env"));
        assert!(labels.contains(&"git-status"));
    }

    #[test]
    fn opaque_secret_handles_are_not_redacted() {
        // These are references, not secrets — false-positive redaction would
        // break seat resolution.
        let input = "handle secret-ref://tenant-a/anthropic and kms-ref://k/1";
        let (out, findings) = redact(input);
        assert_eq!(out, input, "opaque handle was wrongly redacted");
        assert!(findings.is_empty());
    }

    #[test]
    fn clean_text_is_untouched() {
        let (out, findings) = redact("the quick brown fox writes Rust");
        assert_eq!(out, "the quick brown fox writes Rust");
        assert!(findings.is_empty());
    }
}
