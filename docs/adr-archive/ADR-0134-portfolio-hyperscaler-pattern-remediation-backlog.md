---
id: ADR-0134
status: Superseded
deciders: council-architecture, ops-sre-reliability, workflow-studio-product-council
date: 2026-05-17
owner: ops-sre-reliability
supersedes: []
superseded_by: [ADR-704]
related:
  - ADR-0114
  - ADR-0123
  - ADR-0133
  - ADR-0514
related_specs:
  - /specs/masterplan.json
  - /specs/products/workflow-studio.json
  - /evidence/autoresearch/hyperscaler-pattern-meta-audit-1779012603.json
version: 1.1.0
purpose: Record the portfolio hyperscaler remediation backlog as proposed acceptance criteria without claiming that the named validators or branch-protected CI lanes already exist.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Hyperscaler pattern remediation backlog (living backlog ADR)

# ADR-0134: Portfolio Hyperscaler Pattern Remediation Backlog

## Status

Proposed - 2026-05-17.

This ADR records candidate portfolio-wide acceptance criteria. It is not a
production-readiness claim, a branch-protection claim, or a hyperscaler-maturity
claim.

## Context

The hyperscaler pattern audit and current PR review queue identified recurring
portfolio gaps across Foundry, Workflow, Workflow Studio, Ontology, and Cloud:

- LLM/tool invocation loops need bounded retry budgets and circuit-breaker state.
- High-volume APIs need per-tenant admission control.
- Foundry needs explicit all-providers-degraded shed behavior.
- Workflow Studio needs the full SRE signal set for operator UX and safety.
- Product SLOs need error-budget burn-rate policy before GA claims.

The earlier PR #135 draft marked these lanes as accepted and enforced even
though the validators, workflow files, branch-protection rows, and negative tests
were not present. This ADR keeps the useful remediation shape while making the
enforcement state honest.

## Decision

Adopt the following remediation items as a **proposed remediation backlog**,
organized into two lanes: **(A) product-SLO** (the original five — LLM circuit
breaker, per-tenant rate limit, provider-degraded shed, Workflow Studio golden
signals, error-budget burn rate) and **(B) build/CI/CD pipeline** (the P0 + LATER
items, per ADR-0514). Each item may become binding only in the PR that ships its
validator, fixture coverage, branch-protection integration, and product-specific
wiring.

| Item | Candidate validator | Minimum acceptance criterion |
|---|---|---|
| LLM circuit breaker | `oya-governance-circuit-breaker-presence` | T1 invocation surfaces declare `max_retry_budget`, `circuit_breaker_threshold`, and circuit state, with max retry budgets <= 3 unless a product ADR justifies a higher value. |
| Per-tenant rate limit | `oya-governance-per-tenant-rate-limit` | Public capability/action/canvas APIs have tenant-keyed token buckets and explicit 429 + `Retry-After` behavior. |
| Provider-degraded shed | `oya-governance-provider-degraded-shed` | Foundry provider queues define all-providers-degraded behavior, defaulting to bounded 503 or bounded queue drop rather than unbounded enqueue. |
| Workflow Studio golden signals | `oya-governance-workflow-studio-golden-signals` | Workflow Studio exposes latency, traffic, errors, and saturation signals. Availability remains an SLO, not a substitute for the missing golden signals. |
| Error-budget burn rate | `oya-governance-error-budget-policy` | Product SLOs define fast-burn and slow-burn thresholds, notification targets, and rollback/escalation behavior. |
| rust_binary Linux final-link / psm (P0 #96, closes #78) | `oya-governance-rust-binary-linux-link` | rust_binary final link succeeds on aarch64-linux. Root cause: the `psm_asm` cxx_library hardcoded `-DCFG_TARGET_OS_darwin` → Mach-O `_`-decorated symbol, while linux-ELF rust references undecorated `rust_psm_stack_pointer` → undefined symbol. Fix: per-OS `select()` (linux `-DCFG_TARGET_OS_linux`). PROVEN green (cloud-intelligence-app links on aarch64-linux). Same darwin-hardcoded-fixup class as #91/#93. Proposed in ADR-0514. |
| Hermetic toolchain (P0 #83) | `oya-governance-hermetic-toolchain-default` | Pinned clang+lld+sysroot cell is the default toolchain (not host `/usr/bin/clang`); builds cleanly on aarch64-linux + aarch64-darwin; per-crate `LDFLAGS=-nostartfiles` deleted; zero OS-divergence at the link layer. Proposed in ADR-0514. |
| Trunk-sourced gate security (P0 #95) | `oya-governance-controller-trunk-sourcing` | Deployed controller spawns K8s Job that clones `dev` (trusted) + fetches PR-ref as data; PR cannot weaken its own gate by editing `buck2-affected-gate.sh`; untrusted Job pod has NetworkPolicy isolation (no reach to controller/OpenBao/GitHub token). Proposed in ADR-0514. |
| Idempotent buckify + durable DEP (P0 post-buckify-patch) | `oya-governance-third-party-buckify-idempotent` | `reindeer buckify` + post-buckify patch step makes `make third-party` idempotent; per-OS `select()` + `$(location)` DEP env survive buckify; CI enforces `git diff third-party/BUCK` clean; fixes non-durable third-party corrections. Proposed in ADR-0514. |
| rdeps depth-cap + two-tier + CAS (P0 #94-secondary) | `oya-governance-rdeps-scope-capped` | Presubmit gate runs depth-limited `rdeps(//…, %Ss, N≈3-5)` for third-party/toolchain changes, gated under 30min; postsubmit runs unbounded + attributes failures to commit; NativeLink CAS-only MVP deployed with >60% hit-rate; measured wall-time <20min on third-party-only changes. Proposed in ADR-0514. |
| oya-ci controller P1 cutover (P0 #88) | `oya-governance-controller-dispatch-cutover` | Gateway adds `ControllerDispatcher` impl; deploys `oya-ci-controller` (kube-rs Job orchestration + trunk-sourced gate + structured log harvesting); both gates run in parallel (green); Jenkins gate path deleted; failure modes (parse fragility, self-deadlock, opaque logs) eliminated. Proposed in ADR-0514. |
| Structured failure summary (P0 + P1) | `oya-governance-structured-failure-summary` | Controller harvests buck2 event-log JSON; crier posts `{target, error_type, first_stderr}[]` summary to GitHub; logs persisted to S3 with direct URL; replaces fragile `grep 'Action failed:'` and eliminates `kubectl exec` diagnosis. Proposed in ADR-0514. |
| Tide merge-queue (LATER #89 / Phase 2) | `oya-governance-merge-queue-tide` | Merge-queue pools PRs, batches, speculatively retests, auto-merges on green; subsumes ADR-0111 + Sweep auto-merge; enables conflict-free zero-rebase landing. Deferred, proposed in ADR-0514 Phase 2. |
| Reviewer-agent + auto-fix (LATER / Phase 4) | `oya-governance-reviewer-agent-autofixing` | Reviewer-agent APPROVE + automated fix loop closes the review/fix arc in the fan-out loop; enables closed-loop dogfood. Deferred, proposed in ADR-0514 Phase 4. |
| oya-ci deck Leptos UI (LATER / Phase 3) | `oya-governance-oya-ci-deck` | Leptos/Rust-WASM CI-visibility surface for founder + agents; introspectable job status, logs, failure taxonomy. Deferred, proposed in ADR-0514 Phase 3. |
| oya-ci plugins / ChatOps (LATER / Phase 4) | `oya-governance-oya-ci-plugins` | Governance pipeline + command dispatch on the gateway; enables ChatOps governance. Deferred, proposed in ADR-0514 Phase 4. |
| buck2-native OCI images (LATER / post-gate) | `oya-governance-buck2-native-oci` | Retire BuildKit/Dockerfile (non-hermetic, transitory) for buck2-native image targets; erases image-build non-hermetic class. Deferred, proposed in ADR-0514 LATER. |
| bespoke NativeLink RE scheduler+workers (LATER / measurement-gated) | `oya-governance-nativelink-re-ops` | Deploy RE tier (scheduler+workers) only after CAS action-cache hit-rate measured >60%; measure wall-time + parallelism delta. Deferred, measurement-gated, proposed in ADR-0514 LATER. |

## Boundaries

- Named validators above are planned identifiers until their crates or CLI gates
  exist on the branch.
- Product PRDs may reference these items only as advisory targets until the
  relevant validator passes on that product's implementation and spec surfaces.
- Branch protection must not list a status check until the workflow exists and
  the check name is stable.
- Numeric thresholds in this ADR are acceptance criteria for future validators,
  not evidence that current products satisfy them.

## Rejected Alternatives

- **Keep the earlier accepted/enforced wording.** Rejected because it would
  reintroduce aspirational enforcement and allow green CI while real product
  surfaces are not checked.
- **Delete the backlog completely.** Rejected because the identified safety and
  SRE gaps are valid and need a durable planning surface.
- **Bundle all five validators into one PR.** Rejected because the implementation
  spans distinct product boundaries and would be too large to review safely.

## Consequences

- Hyperscaler-maturity claims remain blocked unless the concrete product
  validators and evidence pass.
- Workflow Studio remains the priority UX surface for the golden-signal work
  because its visual editor needs clear operator feedback for traffic, errors,
  saturation, and latency.
- Later PRs should land one validator family at a time with tests, evidence, and
  branch-protection updates in the same slice.

## Verification

- `oya doc adr-index --write --format json`
- `oya gate validate adr-citation`
- `oya gate validate hyperscaler-maturity-claims`
- Reviewer-agent check that every claim is advisory unless a validator exists
  and is wired into CI.
