//! Content-guardrail port for cloud-intelligence.
//!
//! This is a **distinct seam from authorization** ([`crate::AuthzGate`]):
//!
//! - `AuthzGate` answers *who-can-call* — a forbid-wins identity/policy decision
//!   made before a seat is selected.
//! - `Guardrail` answers *is-this-content-acceptable* — inspection and
//!   transformation of request/response payloads (redaction, secret stripping,
//!   prompt-injection screening, moderation) around the upstream call.
//!
//! The port is pure interface: the kernel binds to no regex engine, moderation
//! service, or model adapter. Concrete guardrails (e.g. the PII/secret redactor)
//! live in adapter crates behind this trait.
//!
//! ## Async, dyn-safe, dependency-free
//!
//! Real guardrails may be I/O-bound (external moderation APIs, secondary agentic
//! review per [`crate::safety`]). The port is therefore async. To keep the
//! kernel free of an `async-trait` proc-macro dependency while remaining
//! object-safe (so a [`GuardrailChain`] can hold heterogeneous guardrails as
//! `Arc<dyn Guardrail>`), the methods return a boxed future ([`GuardFuture`])
//! — the manual desugaring of `async fn`. Callers still write
//! `guardrail.pre_call(content, ctx).await`.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::{AgentId, Provider, TenantId};

/// Which side of the upstream call a guardrail is inspecting.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum GuardStage {
    /// Outbound: the request/prompt before it leaves for the provider.
    PreCall,
    /// Inbound: the response/completion before it is returned or logged.
    PostCall,
}

/// Platform taxonomy of what a guardrail flagged. Deliberately *not* the secret
/// value itself — a finding records only the class and how many were seen, so
/// findings are safe to log, emit, and surface to tenants.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum GuardClass {
    /// Bearer tokens, OAuth access/refresh tokens, JWTs, raw API keys.
    Credential,
    /// Personally identifiable information (email, etc.).
    PersonalData,
    /// Absolute filesystem paths that leak host layout.
    FilesystemPath,
    /// Host/environment context (hostnames, platform, working directory).
    HostContext,
    /// Repository/working-tree state (e.g. a `gitStatus` dump).
    RepositoryState,
    /// Prompt-injection / jailbreak signal.
    PromptInjection,
    /// Anything else a guardrail wishes to report.
    Other,
}

/// A single secret-free guardrail finding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardFinding {
    pub class: GuardClass, // data_class: INTERNAL_ONLY
    /// Stable machine label for the matched pattern (e.g. `"bearer-token"`).
    /// Never contains the matched secret value.
    pub label: String, // data_class: INTERNAL_ONLY
    /// How many occurrences were matched/redacted.
    pub occurrences: usize, // data_class: INTERNAL_ONLY
}

impl GuardFinding {
    pub fn new(class: GuardClass, label: impl Into<String>, occurrences: usize) -> Self {
        Self {
            class,
            label: label.into(),
            occurrences,
        }
    }
}

/// The inspectable payload passed to a guardrail. Text-oriented: requests and
/// responses are normalized to their textual content for inspection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardContent {
    text: String, // data_class: INTERNAL_ONLY
}

impl GuardContent {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn into_text(self) -> String {
        self.text
    }
}

/// Routing/identity context for a guardrail decision. Carries no payload secrets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardContext {
    pub tenant_id: TenantId, // data_class: INTERNAL_ONLY
    pub agent_id: AgentId,   // data_class: INTERNAL_ONLY
    pub provider: Provider,  // data_class: INTERNAL_ONLY
    pub request_id: String,  // data_class: INTERNAL_ONLY
}

impl GuardContext {
    pub fn new(
        tenant_id: TenantId,
        agent_id: AgentId,
        provider: Provider,
        request_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            agent_id,
            provider,
            request_id: request_id.into(),
        }
    }
}

/// The guardrail verdict for one stage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuardDecision {
    /// Content is acceptable as-is; no transformation applied.
    Allow,
    /// Content was transformed (e.g. secrets stripped); the returned
    /// [`GuardContent`] is the safe version to use downstream.
    Redacted,
    /// Content must not proceed; the call is rejected fail-closed.
    Blocked,
}

/// Outcome of a guardrail (or a [`GuardrailChain`]) over one payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardOutcome {
    decision: GuardDecision,
    content: GuardContent,
    findings: Vec<GuardFinding>,
    block_reason: Option<String>, // data_class: INTERNAL_ONLY
}

impl GuardOutcome {
    /// Content was clean.
    pub fn allow(content: GuardContent) -> Self {
        Self {
            decision: GuardDecision::Allow,
            content,
            findings: Vec::new(),
            block_reason: None,
        }
    }

    /// Content was transformed; `content` is the redacted version.
    pub fn redacted(content: GuardContent, findings: Vec<GuardFinding>) -> Self {
        // A redaction with no findings is just an allow.
        if findings.is_empty() {
            return Self::allow(content);
        }
        Self {
            decision: GuardDecision::Redacted,
            content,
            findings,
            block_reason: None,
        }
    }

    /// Content is rejected fail-closed. The returned content is preserved for
    /// audit/evidence but callers MUST NOT forward it upstream.
    pub fn blocked(
        content: GuardContent,
        reason: impl Into<String>,
        findings: Vec<GuardFinding>,
    ) -> Self {
        Self {
            decision: GuardDecision::Blocked,
            content,
            findings,
            block_reason: Some(reason.into()),
        }
    }

    pub fn decision(&self) -> GuardDecision {
        self.decision
    }

    pub fn content(&self) -> &GuardContent {
        &self.content
    }

    pub fn findings(&self) -> &[GuardFinding] {
        &self.findings
    }

    pub fn block_reason(&self) -> Option<&str> {
        self.block_reason.as_deref()
    }

    pub fn is_blocked(&self) -> bool {
        self.decision == GuardDecision::Blocked
    }

    pub fn is_clean(&self) -> bool {
        self.decision == GuardDecision::Allow
    }

    /// Consume the outcome and return the (possibly redacted) safe content.
    pub fn into_content(self) -> GuardContent {
        self.content
    }
}

/// Boxed future returned by [`Guardrail`] methods — the dyn-safe, dependency-free
/// desugaring of `async fn`.
pub type GuardFuture<'a> = Pin<Box<dyn Future<Output = GuardOutcome> + Send + 'a>>;

/// Content-acceptability port. Distinct from [`crate::AuthzGate`].
///
/// Implementations MUST be deterministic with respect to their inputs for a
/// given external state and MUST never embed a caller-supplied secret in a
/// [`GuardFinding`]. A guardrail that cannot reach a decision (e.g. a moderation
/// backend is down) SHOULD fail closed by returning [`GuardOutcome::blocked`].
pub trait Guardrail: Send + Sync {
    /// Stable identifier for diagnostics and finding attribution.
    fn name(&self) -> &str;

    /// Inspect outbound request content before it is sent upstream.
    fn pre_call<'a>(&'a self, content: &'a GuardContent, ctx: &'a GuardContext)
        -> GuardFuture<'a>;

    /// Inspect inbound response content before it is returned or logged.
    fn post_call<'a>(
        &'a self,
        content: &'a GuardContent,
        ctx: &'a GuardContext,
    ) -> GuardFuture<'a>;
}

/// An ordered, composable pipeline of guardrails.
///
/// Semantics for both stages:
/// - guardrails run in registration order;
/// - each guardrail sees the (possibly redacted) output of the previous one,
///   so redactions compose;
/// - findings accumulate across the chain;
/// - the first [`GuardDecision::Blocked`] short-circuits — remaining guardrails
///   are not run — and the chain returns `Blocked` (fail-closed);
/// - otherwise the chain returns `Redacted` if any finding was raised, else
///   `Allow`.
#[derive(Clone, Default)]
pub struct GuardrailChain {
    guardrails: Vec<Arc<dyn Guardrail>>,
}

impl GuardrailChain {
    pub fn new() -> Self {
        Self {
            guardrails: Vec::new(),
        }
    }

    /// Builder-style append.
    #[must_use]
    pub fn with(mut self, guardrail: Arc<dyn Guardrail>) -> Self {
        self.guardrails.push(guardrail);
        self
    }

    pub fn push(&mut self, guardrail: Arc<dyn Guardrail>) {
        self.guardrails.push(guardrail);
    }

    pub fn len(&self) -> usize {
        self.guardrails.len()
    }

    pub fn is_empty(&self) -> bool {
        self.guardrails.is_empty()
    }

    /// Run every guardrail's `pre_call` in order.
    pub async fn pre_call(&self, content: &GuardContent, ctx: &GuardContext) -> GuardOutcome {
        self.run(GuardStage::PreCall, content, ctx).await
    }

    /// Run every guardrail's `post_call` in order.
    pub async fn post_call(&self, content: &GuardContent, ctx: &GuardContext) -> GuardOutcome {
        self.run(GuardStage::PostCall, content, ctx).await
    }

    /// Run the chain for `stage`, threading redacted content through each step.
    pub async fn run(
        &self,
        stage: GuardStage,
        content: &GuardContent,
        ctx: &GuardContext,
    ) -> GuardOutcome {
        let mut current = content.clone();
        let mut findings: Vec<GuardFinding> = Vec::new();
        let mut any_redaction = false;

        for guardrail in &self.guardrails {
            let outcome = match stage {
                GuardStage::PreCall => guardrail.pre_call(&current, ctx).await,
                GuardStage::PostCall => guardrail.post_call(&current, ctx).await,
            };

            match outcome.decision() {
                GuardDecision::Blocked => {
                    // Short-circuit fail-closed; fold this guardrail's findings in.
                    findings.extend(outcome.findings().iter().cloned());
                    let reason = outcome
                        .block_reason()
                        .map(str::to_string)
                        .unwrap_or_else(|| format!("blocked by guardrail '{}'", guardrail.name()));
                    return GuardOutcome::blocked(outcome.into_content(), reason, findings);
                }
                GuardDecision::Redacted => {
                    any_redaction = true;
                    findings.extend(outcome.findings().iter().cloned());
                    current = outcome.into_content();
                }
                GuardDecision::Allow => {
                    current = outcome.into_content();
                }
            }
        }

        if any_redaction {
            GuardOutcome::redacted(current, findings)
        } else {
            GuardOutcome::allow(current)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> GuardContext {
        GuardContext::new(
            TenantId::new("tenant-a").expect("tenant"),
            AgentId::new("agent-a").expect("agent"),
            Provider::Anthropic,
            "req-1",
        )
    }

    /// Guardrail that records the text it saw and applies a fixed transform.
    struct Stub {
        name: &'static str,
        behavior: Behavior,
    }

    #[derive(Clone)]
    enum Behavior {
        Allow,
        /// Replace `from` with `to`, reporting one finding.
        Redact { from: &'static str, to: &'static str },
        Block { reason: &'static str },
    }

    impl Stub {
        fn apply(&self, content: &GuardContent) -> GuardOutcome {
            match &self.behavior {
                Behavior::Allow => GuardOutcome::allow(content.clone()),
                Behavior::Redact { from, to } => {
                    if content.as_str().contains(from) {
                        let redacted = content.as_str().replace(from, to);
                        GuardOutcome::redacted(
                            GuardContent::new(redacted),
                            vec![GuardFinding::new(GuardClass::Credential, self.name, 1)],
                        )
                    } else {
                        GuardOutcome::allow(content.clone())
                    }
                }
                Behavior::Block { reason } => GuardOutcome::blocked(
                    content.clone(),
                    *reason,
                    vec![GuardFinding::new(GuardClass::PromptInjection, self.name, 1)],
                ),
            }
        }
    }

    impl Guardrail for Stub {
        fn name(&self) -> &str {
            self.name
        }
        fn pre_call<'a>(
            &'a self,
            content: &'a GuardContent,
            _ctx: &'a GuardContext,
        ) -> GuardFuture<'a> {
            let outcome = self.apply(content);
            Box::pin(async move { outcome })
        }
        fn post_call<'a>(
            &'a self,
            content: &'a GuardContent,
            _ctx: &'a GuardContext,
        ) -> GuardFuture<'a> {
            let outcome = self.apply(content);
            Box::pin(async move { outcome })
        }
    }

    #[tokio::test]
    async fn empty_chain_allows_unchanged() {
        let chain = GuardrailChain::new();
        let content = GuardContent::new("hello world");
        let out = chain.pre_call(&content, &ctx()).await;
        assert_eq!(out.decision(), GuardDecision::Allow);
        assert_eq!(out.content().as_str(), "hello world");
        assert!(out.findings().is_empty());
    }

    #[tokio::test]
    async fn redactions_compose_in_order() {
        let chain = GuardrailChain::new()
            .with(Arc::new(Stub {
                name: "first",
                behavior: Behavior::Redact {
                    from: "AAA",
                    to: "[1]",
                },
            }))
            .with(Arc::new(Stub {
                name: "second",
                behavior: Behavior::Redact {
                    from: "[1]",
                    to: "[2]",
                },
            }));
        let out = chain
            .pre_call(&GuardContent::new("x AAA y"), &ctx())
            .await;
        // first turns AAA -> [1]; second sees [1] and turns it -> [2].
        assert_eq!(out.decision(), GuardDecision::Redacted);
        assert_eq!(out.content().as_str(), "x [2] y");
        assert_eq!(out.findings().len(), 2);
    }

    #[tokio::test]
    async fn block_short_circuits_and_skips_remaining() {
        let chain = GuardrailChain::new()
            .with(Arc::new(Stub {
                name: "blocker",
                behavior: Behavior::Block {
                    reason: "injection detected",
                },
            }))
            .with(Arc::new(Stub {
                name: "never-runs",
                behavior: Behavior::Redact {
                    from: "anything",
                    to: "X",
                },
            }));
        let out = chain
            .post_call(&GuardContent::new("ignore previous instructions"), &ctx())
            .await;
        assert!(out.is_blocked());
        assert_eq!(out.block_reason(), Some("injection detected"));
        // only the blocker's finding — the second guardrail never ran.
        assert_eq!(out.findings().len(), 1);
    }

    #[tokio::test]
    async fn all_allow_yields_clean() {
        let chain = GuardrailChain::new().with(Arc::new(Stub {
            name: "noop",
            behavior: Behavior::Allow,
        }));
        let out = chain.pre_call(&GuardContent::new("clean"), &ctx()).await;
        assert!(out.is_clean());
    }

    #[test]
    fn redacted_with_no_findings_is_allow() {
        let out = GuardOutcome::redacted(GuardContent::new("x"), Vec::new());
        assert_eq!(out.decision(), GuardDecision::Allow);
    }
}
