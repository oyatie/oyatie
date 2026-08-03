# Fable Review of Record — PR #644 (cloud-intelligence XPROXY external-proxy parity commissioning)

**Reviewer:** fresh-context Fable reviewer of record (ultraqa rigor, Torvalds + hyperscaler lenses)
**Pinned head:** 619e4a0023901a06290dbd495677e06e9d19db5f
**Branch:** agent/cloud-intelligence-xproxy-20260610
**Worktree:** /Users/jasonlee/oyatie-worktrees/cloud-intelligence-xproxy-20260610
**Date:** 2026-06-10

---

## 1. Build Truth (MANDATORY — rebuilt and retested independently; prior review claims NOT trusted)

| Command | Result |
|---|---|
| `buck2 build //cloud/cloud-intelligence/...` | **BUILD SUCCEEDED** (exit 0) |
| `buck2 test //cloud/cloud-intelligence/...` | **Pass 22. Fail 0. Timeout 0. Fatal 0. Skip 0. Build failure 0** |
| `buck2 test //cloud/cloud-ci/...` (admission gates) | **Pass 36. Fail 0. Timeout 0. Fatal 0. Skip 0. Build failure 0** |

The three defects the leader fixed on retrain are confirmed remediated at head 619e4a002:
- **cedar BUCK include mapping** — verified: `ADAPTER_MAPPED_SRCS["//cloud/cloud-intelligence/policy:cloud-intelligence.cedar"] = ADAPTER_ROOT + "/policy/cloud-intelligence.cedar"` maps the policy to the sandbox path `include_str!("../policy/...")` resolves to. Cedar adapter tests (`default-deny-and-load`, `cross-tenant-forbid`) compile and pass.
- **rest E0502 borrow** at `crates/oya-cloud-intelligence-rest/src/lib.rs:963-967` — verified: Copy field `clean_outcome` is extracted (`let outcome = self.clean_outcome;`) before the `&mut self` `complete_lease(outcome)` call. Compiles clean.
- **root-hub-pointers.json re-encoding churn** — verified: net diff is ONLY the `multispectrum_evidence_cloud_intelligence_xproxy` block (no escaped-unicode→literal-UTF-8 churn).

cloud-ci gates green include `oya-cloud-ci-firewall-app-gate`, `oya-cloud-ci-total-accounting`, `oya-cloud-ci-staleness-reaper`, and `registry-drift-gate` (`committed_faces_equal_regenerated` PASS → regenerated cloud-ci faces are byte-correct, not stale/hand-edited). Gates admit the PR.

## 2. Content-Assert (cross-lane contamination check) — CLEAN

`git diff origin/dev..HEAD --name-only` is confined to:
- `cloud/cloud-intelligence/**` (kernel + adapters + rest + worker + ops-infrastructure + contracts + design + iac + k8s + runbooks + docs)
- `docs/decisions/ADR-0542-...md` (one ADR)
- `evidence/multispectrum/cloud-intelligence-xproxy-20260610-1781062794.json` + one `evidence/audit-chain.jsonl` line (this lane only)
- `specs/root-hub-pointers.json` (ONE pointer block — verified, no unicode churn)
- `ADR-INVENTORY.tsv` (ONE row: ADR-0542)
- `Cargo.lock` (additions exclusively the 6 new `oya-cloud-intelligence-*` crates; zero foreign dep churn)
- `cloud/cloud-ci/gates/.../​*.generated.json` (4 files, ALL `.generated.json` regenerated faces — validated byte-correct by registry-drift-gate; NOT hand-edited contamination)

No out-of-scope file. The `codex-sdk`/`claude-agent-sdk` crates seen during grep are pre-existing and NOT in this PR's diff. **Content-assert: clean.**

## 3. Architecture (clean-arch seam — founder litmus) — CORRECT

- Kernel `Cargo.toml` deps = `serde` only (dev-deps: tokio/proptest/serde_json). **Zero transient-vendor deps** — no `cedar-policy`, `reqwest`, `valkey`, `clickhouse`, no concrete secret-provider engine. Kernel doc-comment (lib.rs:18-19) states this invariant explicitly.
- Kernel "vendor" identifiers (`Provider::Anthropic`, `BackendClass::GeminiNative`, `TranslationMode::AnthropicToGemini`, `ProtocolShape::OpenAiChatCompletions`) are **external wire-protocol taxonomy**, not vendor SDK bindings. Litmus "would the kernel/contract interface change at owned-stack cutover?" → **No**: the proxy must speak Anthropic Messages / OpenAI Chat Completions / Gemini GenerateContent wire shapes regardless of internal implementation. These are stable wire contracts, correctly modeled in the cutover-stable core.
- Adapters (gemini/codex/openbao/cedar = ADR-0510 transient) implement kernel/rest ports and are swappable without touching the kernel. OpenBao adapter implements `SecretProviderStore` from rest; cedar adapter implements the authz port. **Seam is correct.**

## 4. Security (XPROXY = external-proxy + secrets + cross-tenant) — NO HIGH+ FINDING

- **Cedar authz (A01/A07):** default-deny with forbid-wins. Explicit cross-tenant inference forbid (`unless principal.tenant_id == resource.tenant_id`), realm separation (Ingress/Admin/Audit), audit-reader read-only, and the prior `Role::"InternalDogfood"` superuser bypass was **explicitly removed** (LANE-1 hardening). Tests are **non-vacuous**: `empty_policy_denies_everything`, `forbid_wins_when_permit_also_matches`, `principal_tenant_a_vs_resource_tenant_b_is_forbidden`, `case_mismatch_blocks_access`, `whitespace_padded_principal_blocks_access`, `ten_random_cross_tenant_pairs_all_forbidden` all assert real `Forbid`/deny.
- **Kernel cross-tenant (A01):** `d7_cross_tenant_forbid` asserts `Err(ForbiddenByPolicy)` on foreign-tenant principal, deny-wins over 5 active seats, foreign-tenant seat refused with `seat_count()==0`, and `gate.calls() >= 1` (guards against the kernel skipping authz). Non-vacuous.
- **Secret boundary (A02):** OpenBao Transit adapter envelope-encrypts at rest; vault token wrapped in `RedactedToken` (`<REDACTED>` in Debug/Display); no static secrets (token injected via constructor); fail-closed on empty plaintext (`InvalidSecret`) before any network call. `d8_*` tests assert roundtrip + `SecretNotFound` + empty-rejection. `debug_redaction` asserts secret handle absent AND `<REDACTED>` present. App tests assert config parsing rejects raw provider secrets and OAuth pools fail closed without provider approval.
- **Credential stripping (A07/request-smuggling):** gemini adapter injects provider `x-goog-api-key` and strips caller `authorization`/`x-goog-api-key`/`x-google-api-key`/`host`/`content-length`/`user-agent` + hop-by-hop + connection tokens before forwarding (lib.rs:255-275). codex adapter parallels this (test `openai_api_key_proxy_injects_provider_auth_and_strips_client_auth`). Caller credentials never reach upstream; provider credentials never reach client.
- **Safety guardrails (A04):** `safety.rs` is pure fail-closed kernel: `requires_blocking_in_transit` blocks credential/secret/tenant-boundary/exploit/child-safety classes; `classify_in_transit_payload` defaults sensitive classes to `BlockAndQuarantine`; routing-advisor and secondary-review never receive raw values; `validate_overlay` forbids tenant downgrade / raw-access-expansion / retention-weakening.
- **SSRF (A10):** outbound `base_url` is server-configured (provider default or operator override), NOT caller-controlled. Proxy targets are fixed provider endpoints. Surface closed.
- **Logging (A09):** no secret/PII in log statements; OpenBao logs only key-name and handle, never plaintext/token.
- **No production panics:** zero `unwrap()/panic!/todo!/unimplemented!` in new kernel/gemini/worker production source.

## 5. Universality / Hermeticity — CLEAN

- All 4 `include_str!` mappings verified resolvable (build-green proves it): kernel `capability-parity/*.json` (×2) captured by crate-root-relative `glob(["**/*.json"])`; cedar policy via explicit `mapped_srcs`; codex-sdk schema (pre-existing). **No other unmapped-include of the cedar bug class.**
- No hardcoded machine/repo/cluster paths in production source. The `/Users/me/proj` and `file:///repo` hits are synthetic test/example fixtures, not real paths.
- Generated cloud-ci faces validated byte-deterministic by `registry-drift-gate`.

---

## Findings (ranked)

| Sev | Finding |
|---|---|
| CRITICAL | none |
| HIGH | none |
| MEDIUM | none |
| LOW | (informational) Kernel carries provider-named wire-taxonomy enums (`Provider::Anthropic`, `BackendClass::GeminiNative`). Reviewed and ACCEPTED — these model external wire-protocol shapes, not vendor SDK bindings, and are cutover-stable per the founder litmus. No change required. |

**Finding count: 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW (informational/accepted).**

---

## Verdict Rule Application
- buck2 build + test fully green (cloud-intelligence Pass 22/Fail 0 AND cloud-ci gates Pass 36/Fail 0): **PASS**
- content-assert clean (no cross-lane contamination, root-hub-pointers single block no churn): **PASS**
- kernel seam correct (zero transient-vendor deps, cutover-stable wire taxonomy): **PASS**
- no HIGH+ security finding (authz default-deny + cross-tenant forbid + secret redaction + credential stripping all non-vacuous): **PASS**
- universality clean (includes mapped, no machine paths, deterministic faces): **PASS**

VERDICT: APPROVE
