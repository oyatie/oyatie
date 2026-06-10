# 20 — Runbooks Lane Review (rest-of-docs sweep)

**Reviewer lane:** runbooks (operational procedures)
**Corpus:** `/Users/jasonlee/Developer/source/docs/runbooks/` — **207 `.md` files**
(170 top-level entries; subdirs: `ads`, `analytics`, `cloud`, `cross-axis`, `foundry`,
`foundry/supervisor`, `ops`, `saas`, `sanctioned-primitives`, `search`,
`vertical-fintech|healthcare|industrial|logistics`, `workspace`).
**Method:** grep-driven canon-conflict sweep (foundry / Forgejo / Jenkins / tenant-tier /
Redis / Kafka / Cedar-engine / isolation-model / tier-vocab / identity) + sample-read of all
substantive infra/CI/data/identity/autonomy runbooks. ADR files are out of scope (SSOT, swept
separately).
**Canon reference:** the 12 RULED-CANON items from the WF2 brief.

> **Cross-ref:** the *mechanical* corpus-wide term footprint lives in
> `10-stale-term-footprint.md`. This doc is the runbooks-lane **qualitative** review —
> it judges which procedures are operationally stale / canon-violating, not just term presence.

---

## TL;DR — headline numbers

| Signal | Count | Note |
|---|---|---|
| Total runbooks | **207** | |
| **Substantive** (real procedure) | **71** | the only ones that can carry an operational canon-conflict |
| **Empty stubs** (`TODO — fill at W-Foundation`) | **136 (66%)** | skeleton-only; structural AI-slop / not-yet-reachable |
| `Status: Stub` marked | **184 (89%)** | |
| `foundry`-branded files (name or content) | **38** | brand RETIRED (canon #2) — 17 `foundry-*.md` + 7 `foundry/*` + supervisor + 13 content-only |
| `Forgejo`-bound runbooks | **4** | canon #3 DROPPED — hard-bound to Forgejo as canonical board/VCS |
| `Jenkins` ref | **1** | inside the Forgejo workflow ("keep Jenkins as the bridge") |
| `tenant-tier` / `tier-system` | **0** | clean — no contradiction of canon #9 |
| `Redis` | **0** | clean (canon #5) |
| `Kafka` | **1** | `kafka-topic-provisioning.md` — an empty stub |
| `Cedar`-as-engine (genuine) | **1** | `cedar-fragment-emergency-rollback.md` — Cedar-as-runtime-engine (canon #6) |
| `microvm/native-default` | **1 (aligned)** | only hit is Firecracker (microVM) — canon #7 CONSISTENT, not contradicted |

---

## GENUINE CANON-CONTRADICTIONS (lead findings)

### C1 — The Forgejo board cluster: Forgejo treated as the CANONICAL VCS/board substrate (canon #3) + Jenkins-as-bridge (canon #4 framing inverted)
**Files (all 4 substantive, all "Active/published"):**
- `forgejo-agent-board-workflow.md`
- `forgejo-board-verification-checklist.md`
- `forgejo-board-webhook-projection.md`
- `forgejo-claim-ref-cas.md`

**Canon #3** = Forgejo DROPPED; GitHub NOW → bespoke VCS later; Forgejo mirror-at-most.
These 4 runbooks instead make Forgejo the **primary, authoritative** board+VCS surface and
treat **GitHub as the thing to avoid** — the inversion of canon:

- `forgejo-agent-board-workflow.md:13-16` — scope explicitly forbids "GitHub Projects"; whole
  workflow is "Plain Git worktrees, **Forgejo issues**, exclusive `state/*` labels, … webhooks."
- `:143-145` — "use plain `git` plus **Forgejo pull requests against `dev`**; do not use … **GitHub
  PR/merge flows** … **keep Jenkins as the bridge** until Phase-1 parallel-run evidence and
  founder/operator approval authorize a cutover." → **canon #3 + #4 contradiction.**
- `:157-159` — "If a worktree still has a GitHub `origin`, the lane must add or select the
  **self-hosted Forgejo remote** before pushing … without using GitHub PR or merge commands."
- `:174` — "Jenkins deletion … is outside the worker-lane default scope."
- `forgejo-claim-ref-cas.md:15` / `:108` / `forgejo-board-verification-checklist.md:186` —
  pinned to live `Forgejo 11.0.14` on `oya-forge`, ADR-0377 "conditional authority for the
  Forgejo board spike."
- `forgejo-board-webhook-projection.md:129` — "Do not depend on GitHub Projects" (Forgejo-primary).

**Nuance worth recording:** the Jenkins framing here is actually the *right shape* — "keep
Jenkins **as the bridge** until … approval authorize a cutover" matches canon #4's
build-first-cutover-later / Jenkins-is-NOT-the-endpoint. The contradiction is (a) Forgejo as the
canonical board at all, and (b) GitHub demoted to "remove your origin" rather than being the NOW
endpoint. **Disposition:** reframe as GitHub-NOW board/VCS (Forgejo mirror-at-most, or retire the
spike runbooks if the Forgejo board experiment is dead); Jenkins-bridge framing can stay if
reframed against **oya-ci** as the endpoint, not against Forgejo-native CI. Cite ADR-0377 — if
that ADR still grants Forgejo "conditional authority," it is itself a canon-#3 conflict (flag to
the ADR lane).

### C2 — Cedar operated as the runtime policy ENGINE (canon #6: Cedar = CONTRACT, owned PARC = engine)
**File:** `cedar-fragment-emergency-rollback.md` (substantive, "Active")
- `:9` owner `axis-policy-engine`; `:65` "**Verify Cedar permit** for emergency revocation"
  with direct `cedar-cli authorize`; `:35-58` queries `cedar_fragments` / `cedar_fragment_rollbacks`
  tables — the procedure treats **Cedar itself as the evaluation engine** at runtime, not as the
  policy *contract* compiled into an owned PARC engine.
- Same Cedar-as-evaluator shape recurs in `bootstrap-ci-compromise.md:74-104` (Cedar fragment =
  the kill-switch that "disables Stage-1 SPIFFE trust roots") and `:240 cedar-cli authorize`.
  Bootstrap's usage is more defensible (Cedar policy *fragment* as a contract artifact), but the
  `cedar-cli authorize` call-path still implies Cedar-as-engine.

**Disposition:** reframe Cedar as the **policy contract** (authored/validated) and route runtime
authorization decisions through the **owned PARC engine** (`cedar-cli authorize` → PARC eval
call). Low operational-risk text change; do NOT touch the audit/rollback table schema.
Secondary: 14 other substantive runbooks reference "Cedar policy denial event" as a *signal*
(e.g. `foundry/autonomy-ceiling-breach-attempt.md:17`) — those are fine (Cedar-as-contract,
denial is an output signal) and are FALSE POSITIVES for the engine-contradiction.

### C3 — "foundry" brand RETIRED (canon #2) — pervasive across the AI/agent runbooks
**38 files** carry the retired brand. This is the largest footprint but is **mostly MECHANICAL,
sense-routed** (matches `10-`'s split verdict). All foundry hits in runbooks are the **AI/agent
substrate sense → `intelligence`**, with ONE governance-lane file:

- → **intelligence** (AI substrate): the 17 `foundry-*.md` (agent-daemon, model-cutover/training,
  capability-publish, mcp-gateway, sandbox-escape/warm-pool, vision, robotics-*, platform-incident,
  autonomy-*), the 7 `foundry/*.md` (autonomy-ceiling, capability-eval, cost-ceiling,
  prompt-injection, provider-quota, sandbox-escape, subscription-token), and
  `foundry/supervisor/lifecycle.md`. Owning-axis token is `axis-foundry` throughout
  (e.g. `foundry-autonomy-break-glass.md:9,22`) → `axis-intelligence`.
- → **governance** (fitness/policy lane): `foundry-fitness-rollback.md` — this is the
  `*-fitness` token family; route to **governance**, not intelligence (matches `10-` routing rule).

**Mid-transition evidence (the rename is already half-done — and inconsistently):**
- `foundry/supervisor/lifecycle.md:21` ships binary `./target/release/oya-foundry-supervisor`
  but `:24` builds cargo package `oya-intelligence-supervisor-app` — **the binary name lags the
  package rename.** This is a live operational inconsistency (an operator copy-pasting `:21` runs a
  binary that may not exist under the new name).
- `foundry/autonomy-ceiling-breach-attempt.md:17` — signal source `oya-foundry-policy` runtime
  block log (stale service name).
- `bootstrap-ci-compromise.md:18` — "first **Foundry-equivalent** workflow running on the
  bootstrap cell" (stale brand in otherwise canon-clean prose).

**Disposition:** mechanical `foundry → intelligence` per-token swap (governance for the
`*-fitness` file), BUT verify the actual binary/package/service names against `microservices/`
before swapping doc commands — the supervisor binary mismatch proves the doc can't be blind-renamed.

---

## STALE / SLOP / REACHABILITY (secondary findings)

### S1 — 136 empty-stub runbooks (66% of the lane) — structural AI-slop + not-yet-reachable
136 runbooks are **pure skeletons**: identical boilerplate (`## Symptom\nTODO — fill at
W-Foundation authoring pass.`, the same 4-line "First-response checklist", the same
"Verify-recovery" / "File MFL row" lines) and **no actual procedure**. Representative:
`foundry-robotics-safe-stop.md`, `kafka-topic-provisioning.md`, `foundry-model-cutover.md`,
`region-failover.md`, `cve-critical-patch.md`, `security-incident-response.md`,
`sev1-incident-response.md`.

- **AI-slop flavor:** fabricated structure / false sense of coverage — 207 "runbooks" exist but
  only 71 contain a real procedure; the index links resolve, so doc-link integrity is green while
  the operational content is absent ("stub authored to satisfy doc-link integrity" is stated
  verbatim in the headers).
- **SAFETY concern:** several stubs are Sev-1 / safety-critical and empty —
  `foundry-robotics-safe-stop.md` (robot E-stop), `foundry/autonomy-ceiling-breach-attempt.md`
  (Sev 1, partly filled), `industrial-ot-write-emergency-stop.md`, `healthcare-break-glass.md`.
  Canon #12 (governance-owned safety-gate set incl. no-lethal / no-actuation) has **no executable
  runbook backing** for the robotics-safe-stop / OT-emergency-stop cases.
- **Reachability class:** these are **INSTRUCTION docs deferred to the "W-Foundation gate"** — i.e.
  they belong to a future gate-defined wave. They are reachable (indexed via `RUNBOOKS-INDEX.md`,
  authored from `templates/runbook-template.md`, both confirmed present) but **content-empty**.
  Recommend: keep the index entry, but down-rank from "207 runbooks" to "71 runbooks + 136 planned
  stubs" in any coverage metric, and explicitly bind each stub to its gate-defined wave (NOT the
  retired M0-M3/MVP vocab — see S3).

### S2 — Data-tier: Postgres (`psql`) in 10 substantive runbooks — transitional-bridge, needs framing (canon #5)
`psql`/Postgres appears in `bootstrap-ci-compromise.md`, `cedar-fragment-emergency-rollback.md`,
`cell-evacuation.md`, `compliance-pack-emergency-suspension.md`, `compliance-pack-revocation.md`,
`byok-rotation-{encryption,provider}-tenant-duress.md`, `meta-trust-root-recovery.md`,
`provider-credential-leak-response.md`, `tenant-data-residency-violation.md`. Also one ClickHouse
ref: `ads/auction-engine-overload.md:31` ("BigQuery / ClickHouse saturation"). Per canon #5 these
are **transitional bridges** (OWN the tier is the endpoint) — not a contradiction, but the
operational text presents them as the permanent data plane with no bridge framing. **Disposition:**
low-priority; add bridge framing or leave for the data-ownership wave. NOT operationally urgent.
**No Redis/Kafka in substantive runbooks** (Kafka's only hit is an empty stub) → canon #5's
Redis→Valkey / Kafka→Pulsar has near-zero runbook footprint. Clean.

### S3 — Wave-vocab: light, mostly clean (canon #9)
Only 2 files hit `\bM0-M3\b`: `meta-trust-root-recovery.md`, `shamir-share-loss-or-coercion.md`
(check in context — likely Shamir M-of-N "M" thresholds = FALSE POSITIVE, not wave vocab).
The retired-vocab phrase that IS present is **"W-Foundation gate"** (in all 136 stubs) and
`foundry/supervisor/lifecycle.md:11` "M02 exit-readiness gate." These are gate-named (better than
M0-M3) but "W-Foundation"/"M02" should be reconciled to the canonical **gate-defined wave** names.
Soft finding.

### S4 — Tier vocabulary: `cell-tier` (canon #9 namespace question)
`cell-tier-promotion.md` and `cell-evacuation.md:212` (`helm/cell-tier-3/`) use "cell tier."
Canon #9 retires `tenant-tier`/`tier-system` in favor of **tenant-CLASS**, while allowing
namespaced `*_tier` (autonomy_tier, dr_tier, storage_tier, …). "cell-tier" is **borderline**: it's
a cell-class concept but uses the bare "tier" word, and `cell-tier-3` is a live IaC helm path
(mechanical, hard to rename). **Disposition:** decide whether "cell tier" is a sanctioned namespace
or should become "cell-class"; if sanctioned, add `cell_tier` to the allowed `*_tier` list. Low
priority. Canon-aligned vocab `autonomy ceiling` / `autonomy_tier` is used correctly throughout
(`autonomy-ceiling-breach-response.md`, `foundry-autonomy-break-glass.md`).

### S5 — Isolation model: CONSISTENT with canon #7 (no contradiction — recorded for completeness)
Only isolation hit is `foundry/sandbox-escape-detected.md:14` — "Wasmtime / **Firecracker**
sandbox escape per ADR-0023." Firecracker = microVM = **canon-aligned** (assume-breach microVM
default). **Zero** `native-default` / `secure-by-default-native` / `framekernel-as-default`
phrasing in runbooks. Canon #7 is not contradicted in this lane.

### S6 — Identity: canon-neutral (no contradiction — recorded)
No Zitadel / Keycloak / Auth0 / Okta refs. Identity substrate in substantive runbooks is **SPIFFE
workload identity** (`bootstrap-ci-compromise.md:20-122`) + KMS/HSM/BYOK key rotation
(`cloud/kms-emergency-rotation.md`, `byok-rotation-*`). Canon #6's oya-identity-owned /
Zitadel-bridge distinction has no runbook footprint to contradict. Clean.

---

## REFINEMENT OPPORTUNITIES
- **Coverage honesty:** stop counting 207 runbooks; the operational reality is 71 written + 136
  stubs. Bind each stub to its gate-defined wave and de-slop the boilerplate (the identical
  "First-response checklist" across 136 files is filler, not procedure).
- **Safety-critical stubs first:** prioritize authoring `foundry-robotics-safe-stop.md`,
  `industrial-ot-write-emergency-stop.md`, `healthcare-break-glass.md`,
  `foundry/autonomy-ceiling-breach-attempt.md` — canon #12 safety gates currently have no runnable
  procedure.
- **Binary/package name drift:** `oya-foundry-supervisor` (binary) vs `oya-intelligence-supervisor-app`
  (package) — fix during the foundry→intelligence rename; verify against `microservices/`.
- **`RUNBOOKS-INDEX.md` + `templates/runbook-template.md`** both exist and resolve — link integrity
  is genuinely green; the issue is content, not structure.

---

## Reachability classification (lane summary)
| Class | Files | Examples |
|---|---|---|
| DECISION → ADR | bound | ADR-0377 (Forgejo board), ADR-0022/0025 (autonomy/break-glass), ADR-0009 (cell isolation), ADR-0023 (sandbox), ADR-0247 (bootstrap) |
| INSTRUCTION → session/wave bundle | 136 stubs | all `TODO — fill at W-Foundation` skeletons (deferred to gate-defined wave) |
| GENERATED-REFERENCE | — | none (runbooks are hand-authored, not generated) |
| ORPHAN → not-needed candidate | 4 Forgejo + Kafka stub | `forgejo-*` (if board spike dead → retire/mirror-only); `kafka-topic-provisioning.md` (empty + retired-tech name → Pulsar) |

---

## Files to amend (priority order)
1. `forgejo-agent-board-workflow.md` + 3 `forgejo-*.md` — canon #3/#4 (GitHub-NOW reframe; verify ADR-0377)
2. `cedar-fragment-emergency-rollback.md` (+ `bootstrap-ci-compromise.md` cedar-cli path) — canon #6 (Cedar=contract / PARC=engine)
3. `foundry/supervisor/lifecycle.md` — binary/package name drift + brand (canon #2)
4. 38 `foundry`-branded files — mechanical → `intelligence` (governance for `foundry-fitness-rollback.md`); verify service names
5. 136 stubs — de-slop + wave-bind + author the safety-critical ones (canon #12)
6. `kafka-topic-provisioning.md` — empty stub w/ retired-tech name → Pulsar framing or retire (canon #5)
