---
purpose: "Extend `oya-foundry-agent-runtime::foundry::auth` with a Claude.ai-subscription-specific OAuth capture path (and an OpenAI parallel where applicable)."
---

---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-004-oauth-subscription-capture
parent: ./INDEX.md
milestone: M02
phase: P02-multi-subscription-pool
status: pending approval
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
purpose: |
  Extend `oya-foundry-agent-runtime::foundry::auth` with a Claude.ai-subscription-specific
  OAuth capture path (and an OpenAI parallel where applicable). Operator launches the flow
  from the operator console; browser handles the PKCE handshake against the upstream
  provider; oyatie receives the redirect on a loopback port (35593 default, matching
  ccproxy-api's `oauth_claude` plugin), stores the resulting subscription token exclusively
  as a `SecretReference` via the OpenBao adapter (ADR-0043), and emits a fully redacted
  `EVT-PROVIDER-ACCOUNT-VERIFIED`. Raw token never enters repo, chat, checkpoint, log, or
  trace.
grit_claim_symbols:
  - "crates/oya-foundry-agent-runtime/src/foundry/auth.rs::capture_subscription_token"
  - "crates/oya-foundry-agent-runtime/src/foundry/auth.rs::SubscriptionOAuthFlow"
  - "crates/oya-foundry-agent-runtime/src/foundry/auth.rs::SubscriptionTokenCaptureRequest"
  - "crates/oya-foundry-agent-runtime/src/foundry/auth.rs::SubscriptionTokenCaptureResponse"
  - "crates/oya-foundry-agent-runtime/src/foundry/auth.rs::OAuthLoopbackServer"
agent_prerequisites:
  - .omc/plans/MASTERPLAN.md
  - ./INDEX.md
  - docs/AGENTS.md
  - /specs/cross-cutting/decision-principles.json
  - /specs/cross-cutting/forbidden-operations.json
  - docs/decisions/ADR-0043-secrets-management-openbao-and-hsm-per-cell.md
  - .omc/standards/security-review.md
  - .omc/scratch/foundry-salvage-from-ultragoal-2026-05-12.md
final_shape_compliance: true
dependency_additions:
  - { crate: "axum 0.8 (loopback callback)", lts: true, adr_exception: null }
  - { crate: "oauth2 5.0", lts: true, adr_exception: null }
  - { crate: "rand 0.9 (PKCE verifier)", lts: true, adr_exception: null }
  - { crate: "sha2 0.10 (PKCE challenge)", lts: true, adr_exception: null }
  - { crate: "url 2.5", lts: true, adr_exception: null }
  - { crate: "secrecy 0.10 (in-memory zeroize)", lts: true, adr_exception: null }
decision_log: |
  Linus good-taste row: eliminated the "Anthropic API key fallback path" as a separate
  branch by representing it as a degenerate flow with `flow_kind = ApiKeyImport`; the
  surface keeps a single `capture_subscription_token` entry, and the API-key import is
  one variant in the enum (no parallel function).
authority_chain_declaration: |
  /specs/cross-cutting/decision-principles.json + /specs/cross-cutting/forbidden-operations.json > rest of docs/ > catalog records > Redirect-class > working drafts.
---

# IP-004-oauth-subscription-capture: Subscription-token OAuth capture flow

## Purpose

Ships the OAuth handshake that lets an operator add a Claude.ai (or ChatGPT Pro/Plus, where
the upstream supports it) subscription token to a ProviderAccount without ever exposing the
raw token to the agent runtime, audit chain, or persistent log. Pattern is field-proven in
ccproxy-api's `oauth_claude` plugin (PKCE, loopback redirect to `http://localhost:35593/callback`,
scopes `org:create_api_key`, `user:profile`, `user:inference`). The Rust implementation tightens
the storage discipline: the token *only* exists in transit; on receipt it is wrapped in
`secrecy::SecretString`, written through `SecretStorePort` (OpenBao adapter) as a
`SecretReference`, and zeroized.

## Symbols to grit-claim

```
crates/oya-foundry-agent-runtime/src/foundry/auth.rs::capture_subscription_token
crates/oya-foundry-agent-runtime/src/foundry/auth.rs::SubscriptionOAuthFlow
crates/oya-foundry-agent-runtime/src/foundry/auth.rs::SubscriptionTokenCaptureRequest
crates/oya-foundry-agent-runtime/src/foundry/auth.rs::SubscriptionTokenCaptureResponse
crates/oya-foundry-agent-runtime/src/foundry/auth.rs::OAuthLoopbackServer
crates/oya-foundry-agent-runtime/src/foundry/auth.rs::FlowKind
```

### Flow

```
1. Operator initiates capture in operator console; oyatie issues SubscriptionTokenCaptureRequest.
2. Runtime generates PKCE verifier/challenge; starts OAuthLoopbackServer on 35593.
3. Browser opens https://claude.ai/oauth/authorize?...code_challenge=…&scope=org:create_api_key+user:profile+user:inference&redirect_uri=http://localhost:35593/callback.
4. User logs in upstream; provider redirects to loopback with `code`.
5. Runtime exchanges code at https://console.anthropic.com/v1/oauth/token (PKCE verifier).
6. Token wrapped in SecretString; written through SecretStorePort → OpenBao → returns SecretReference("sref://…<hash>").
7. ProviderAccount transitions Draft → Verified; emits audit event `account_verified` with auth_mode = SubscriptionOAuth.
8. Pool membership candidacy enabled (subject to IP-006 ToS-ack gate).
9. Loopback server shuts down; PKCE verifier zeroized.
```

OpenAI parallel uses `https://auth.openai.com/...` per ccproxy-api `oauth_codex` plugin where
the upstream supports it; ApiKeyImport flow handles cases where OAuth is not available.

## Agent prerequisites

<!-- agent-instructions:start -->
Before `grit claim`, the agent **MUST**:
1. `icm recall-context "P02 oauth subscription capture claude.ai openbao" --limit 5`.
2. Read `docs/decisions/ADR-0043-secrets-management-openbao-and-hsm-per-cell.md` (mandatory).
3. Read `.omc/standards/security-review.md §7` (secret handling).
4. Confirm `crates/oya-foundry-agent-runtime/src/foundry/auth.rs::capture_subscription_token` unclaimed via `oya-tooling-agent-read grit-status`.
5. Read `docs/AGENTS.md §Pre-flight checklist` and `/specs/cross-cutting/forbidden-operations.json` (FO-01..FO-10; specifically: no raw secrets in repo / log / chat / checkpoint).
6. Read parent INDEX `./INDEX.md`.
<!-- agent-instructions:end -->

**Human path:** open operator console → "Add subscription" → choose provider → complete browser OAuth → console shows `Verified` with the redacted `sref://…<hash>` reference; raw token never displayed.

## Acceptance test commands

```
$ cargo nextest run -p oya-foundry-agent-runtime --test subscription_oauth_capture  # expect: PASS, 0 failures
$ cargo clippy -p oya-foundry-agent-runtime -- -D warnings                          # expect: PASS, 0 warnings
$ cargo deny check                                                                  # expect: PASS
$ oya gate validate oya-foundry-fitness-secret-rotation                             # expect: PASS
$ node scripts/hooks/guard-secrets.mjs --scan crates/oya-foundry-agent-runtime      # expect: PASS (no raw token strings)
$ trufflehog filesystem crates/oya-foundry-agent-runtime --only-verified            # expect: 0 findings
$ gitleaks detect --source crates/oya-foundry-agent-runtime --no-banner             # expect: 0 leaks
$ oya-tooling-agent-read run-evidence "scripts/smoke/oauth-capture-mock-server.sh"  # expect: mock IdP → loopback → SecretReference returned; raw-token redaction scan = 0 hits in audit chain
```

End-to-end test required: spin up a `wiremock` upstream simulating Claude.ai OAuth; drive
the full flow; assert (a) returned SecretReference scheme is `sref://`, (b) audit-chain
payload shows `auth_mode = SubscriptionOAuth`, (c) raw token never appears in any captured
log/trace/audit-payload (`silent-failure-hunter` reviewer agent grep with constant-time
comparison).

## Done criteria

- [ ] All `grit_claim_symbols` claimed → work → `grit done`.
- [ ] D1-D18 done-definition walked.
- [ ] All acceptance commands PASS; outputs in PR `## Verification`.
- [ ] `cargo deny check` + `cargo vet` certifications for oauth2, secrecy, rand, sha2.
- [ ] `icm store -t context-foundry` emitted (§Icm-store-payload).
- [ ] Audit-chain `EVT-PROVIDER-ACCOUNT-VERIFIED` emitted (with redacted metadata only).
- [ ] `silent-failure-hunter` + `security-reviewer` agent verdicts: APPROVE.
- [ ] No raw token appears anywhere in repo / chat / checkpoint / log / trace.
- [ ] Loopback port 35593 closed within 60 s of code exchange; PKCE verifier zeroized.

## Rollback procedure

1. Identify rollback boundary: feature flag `foundry.auth.subscription_oauth.enabled = false`; or revert PR.
2. Execute: `oya policy update foundry.auth.subscription_oauth.enabled false`; existing SubscriptionOAuth-tier ProviderAccount records remain Verified but cannot be used for new pool routing decisions (IP-006 policy refuses).
3. Verify: capture-token endpoint returns 403; audit-chain emits `EVT-SUBSCRIPTION-OAUTH-DISABLED`.
4. Postmortem trigger: Sev-1 if any raw token leak detected during incident; Sev-2 otherwise.

## Next IP pointer

`IP-005-upstream-api-drift-lane.md`.

## Icm-store-payload

```
icm store \
  -t context-foundry \
  -c "IP-004-oauth-subscription-capture merged at <git-sha>; grit symbols released: capture_subscription_token, SubscriptionOAuthFlow, OAuthLoopbackServer, FlowKind; acceptance lanes green: -secret-rotation, -no-placeholder, silent-failure-hunter+security-reviewer APPROVE; next IP: IP-005-upstream-api-drift-lane" \
  -i high \
  -k "M02,P02,IP-004,oauth-subscription,secret-reference,ccproxy-parity"
```

## Decision log (Linus good-taste row)

Eliminated the parallel "ApiKey import" function by collapsing it into a `FlowKind`
variant of the same `capture_subscription_token` entry; one entry surface, no branching.

## Cross-references

- Master Plan: `.omc/plans/MASTERPLAN.md` §2 Directives 2, 3, 4, 8.
- Phase INDEX: `./INDEX.md`.
- ADR-0043 — OpenBao secrets management (this IP is a primary consumer).
- ADR-0053 — sanctioned primitives.
- `.omc/standards/security-review.md §7` — secret handling.
- Progressive-delivery + branch-pipeline composers.
- ccproxy-api source of inspiration: https://github.com/CaddyGlow/ccproxy-api/blob/main/ccproxy/plugins/oauth_claude/README.md (PKCE flow, loopback port 35593, scopes).
- Anthropic OAuth endpoints: `https://claude.ai/oauth/authorize`, `https://console.anthropic.com/v1/oauth/token`.
