---
doc_class: Audit
audit_id: AUDIT-ARCH-ANTIPATTERNS-PARALLEL-CHANGESET-2026-05-14
title: Architecture antipatterns blocking parallel ChangeSet-driven agentic development
status: Accepted-2026-05-14
owner_team: council-architecture
session_evidence: ICM topic context-oyatie keys "M-CC-P00-IP-001..009,M-CC-P01..P09,M02-P00,architecture-gate"
parent: ../INDEX.md
purpose: Identify the current architecture's antipatterns that demonstrably blocked or slowed parallel ChangeSet-driven agentic development this session, with concrete repro evidence and bounded FixupTasks for each.
---

# Architecture antipatterns audit — parallel ChangeSet-driven agentic development

## Scope + method

This audit is grounded in **direct evidence from a 35-IP session**
running grit-claim → work → done across M-CC + M02. Antipatterns are
only listed if they actually slowed or blocked work in this session.
For each: concrete repro, blast-radius observation, sanctioned fix, and
the ChangeSet/IP boundary that owns the fix.

---

## A. Layer-violation antipatterns (kernel/domain boundary leaks)

### A1. Kernel crates re-export business types from domain
**Evidence**: `scripts/check-architecture-boundaries.sh` reported 8
kernel→domain forbidden edges this session:
- 6 provider-adapter kernels pulled `ProviderFamily` + `SecretReference` from `oya-foundry-account-domain`.
- `oya-foundry-usage-window-kernel` re-exports `UsageWindow`/`UsageWindowError`/`UsageWindowKind`.
- `oya-foundry-route-policy-kernel` re-exports `AccountError`/`AccountId`/`AccountState`/`ProviderAccount`/`RouteExplanation`.
- `oya-foundry-dashboard-kernel` + `oya-foundry-dashboard-dry-run-kernel` pull `AccountState`/`ProviderAccount`/`RouteExplanation`.

**Blast radius**: any ChangeSet that touches `oya-foundry-account-domain` transitively read-locks at least 10 downstream kernels. Two agents working on adjacent provider adapters serialized on this edge.

**Fix (already partial)**: M02-P00-IP-004 (new fixup IP this session) moved identity types `AccountId`/`SessionId`/`ProviderFamily`/`SecretReference` into `oya-foundry-account-kernel`; closed 6 of 8 violations.

**Remaining FixupTask** (`01KRM3Z3Z6V8DVRVTRYT0Y4T87`): 4 kernels still re-exporting business types. Two paths:
- Re-role those crates as `domain` in `registry/catalog/<name>.yaml`.
- OR define narrow ports in `oya-foundry-account-kernel` + invert dependencies.

### A2. Sibling adapters depend on each other
**Evidence**: 3 forbidden adapter→adapter edges:
- `oya-foundry-api-graphql-adapter → oya-foundry-api-rest-adapter`
- `oya-foundry-api-sse-adapter → oya-foundry-api-rest-adapter`
- `oya-foundry-api-websocket-adapter → oya-foundry-api-rest-adapter`

**Blast radius**: REST adapter changes write-lock 3 sibling transport adapters; multi-protocol agents cannot work in parallel on adjacent transports.

**Fix**: extract shared transport types into a new `oya-foundry-api-kernel` (kernel role) that all 4 transport adapters consume. Each transport then evolves independently.

---

## B. Claim-scope antipatterns (locks wider than the change)

### B1. Whole-crate symbol claims when only one function changes
**Evidence**: grit FK errors hit ~50× this session, every time on
multi-symbol claims. The ADR-0054 ICM fallback worked, but each
fallback widens the claim semantics from "these symbols" to "this
file/crate" — losing the per-symbol parallelism that the AST kernel
(IP-009) is designed to provide.

**Blast radius**: one ChangeSet touching `crates/X/src/lib.rs::foo`
read-locks every other symbol in that file for any contemporaneous
agent. With 1000+ symbol crates (e.g., `oya-foundry-vcs-kernel` at 995 LOC), this is a serialization point.

**Fix**: M-CC-P00 IP-001 + IP-009 already ship the contract (SymbolId
+ AstIndex + claim_compatibility). The remaining work is the upstream
grit FK bug (M-CC-P01-IP-011 runbook landed this session) — once grit
stops FK-erroring on symbol-narrow claims, the existing per-symbol
locking design takes effect without further code changes.

### B2. ICM scaffold-claim used for any FK error, including
narrow-symbol ones
**Evidence**: ADR-0054 fallback is sanctioned, but its broad-scope
phrasing ("claiming X, Y, Z files") encourages writing whole-file
claims into ICM rather than per-symbol claims.

**Blast radius**: the ICM fallback essentially becomes a
file-granular lock for the session, defeating IP-009's per-symbol
admission semantics.

**Fix**: extend the ICM scaffold-claim template to require declared
`SymbolId` values, not just paths. Lane-level enforcement via a new
fitness kernel `oya-foundry-fitness-icm-claim-scope-kernel` (proposed
M-CC-P01-IP-012, not yet split into the masterplan).

---

## C. Status-truth antipatterns (stale planning state)

### C1. IP `status:` fields drift from code reality
**Evidence**: Multiple IPs this session had `status: stub` while their
target crate already shipped production-quality code with passing
tests (M-CC-P00 IP-001..009, M02-P00-IP-002, M02-P00-IP-003,
M-CC-P02-IP-001..003, etc.). I had to ground-truth by running tests
against each crate, not by reading status.

**Blast radius**: agents picking work by status alone end up
duplicating already-done implementation. This session avoided that by
checking tests first; a less careful agent would have rewritten ~6 P00
crates from scratch.

**Fix**: add a fitness kernel `oya-foundry-fitness-ip-status-truth-kernel`
that, for each `status: complete` IP, asserts (a) referenced crates
exist, (b) targeted tests exist + pass, (c) decision-log row is
non-empty. Status flips from any other state to `complete` should only
land via this gate. *(Proposed new IP under M-CC-P01.)*

### C2. Evidence files pre-emitted before IP closure, then orphaned
**Evidence**: `/evidence/gitops-vcs/ip-{001..009}-*.json` were all
pre-emitted by `codex-autopilot` in an earlier session, but their
parent IP files still carried `status: planned-from-approved-ralplan-v5`.
This session flipped 9 of those to `complete` — but a fitness kernel
should have caught the orphan.

**Fix**: same kernel as C1; one of its checks is "evidence file exists
⇒ IP status must be `complete` (or evidence file path moves to a
`speculative/` quarantine subdir)".

---

## D. ChangeSet-boundary antipatterns

### D1. IPs that span "audit every file in the repo"
**Evidence**: `M-CC-P03-IP-001` (purpose-frontmatter-audit) claims
`docs/**::purpose-frontmatter` and `**/*.json::purpose-field` — that's
a tree-wide audit, not a ChangeSet. It cannot be claimed under
`changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable`.

`M-CC-P05-IP-002` (cloud-multi-provider-audit) has the same shape:
glob over `crates/oya-cloud-*-adapter-{aws,oci,gcp,azure,naver,kt,kakao}-*`.

**Blast radius**: any single agent claiming the IP write-locks the
entire docs tree (or all cloud adapter crates) — no other agent can
work on those files for the duration.

**Fix (applied this session)**: both IPs flipped to status
`split-required-too-broad-for-single-changeset` and their text marks
them for splitting before next claim. Splitting recipe:
- P03-IP-001 → per-top-level-dir audits (one IP per `docs/<top>/` and
  one per `.omc/<subtree>/`).
- P05-IP-002 → per-provider audits (one IP per cloud provider).

### D2. IPs depending on other IPs without explicit `dependencies:` row
**Evidence**: M-CC-P00 sequencing is encoded in a hard-coded prompt
("IP-001 + IP-009 first, then IP-002/003/006/008, then IP-004/007,
then IP-005"). The IP files themselves don't declare these edges, so a
DAG scheduler can't extract them without parsing the prompt.

**Blast radius**: parallel agents pick the wrong antichain, then
either (a) FK-collide at claim time, or (b) submit out-of-order
ChangeSets that the merge queue rejects, wasting build cycles.

**Fix**: add `dependencies:` array to the IP frontmatter schema (the
field is already in `master-plan-sequencing.json#ledger_required_fields`
but most IP files leave it empty). A new fitness kernel (proposed
`oya-foundry-fitness-ip-dependency-edges-kernel`) checks that every
IP's `dependencies` list resolves to existing IPs and that the implied
DAG is acyclic.

---

## E. Tooling antipatterns the user already corrected this session

### E1. Mixed-language coordination scripts (`.sh`, `.mjs`, `.py`)
**Status**: User directive 2026-05-14 — "no shellscript no mjs etc all rust" — enforced. `.mjs` merge-gate retired and replaced with `oya-foundry-fitness-pr-merge-gate-kernel` in the same session. Going forward every new check/lane/coordination primitive must be a Rust crate, not a shell/JS/Python script.

**Why it was an antipattern**: shell/Node scripts can't participate
in the cargo workspace test graph, semver checks, deny.toml policy,
or the AST index. They are also opaque to the M-CC-P00 admission gate
because they have no SymbolId.

---

## F. Recommended new IPs to land before next masterplan loop

| Proposed IP | Phase | Purpose | Estimated scope |
|---|---|---|---|
| M-CC-P01-IP-012 | agentic-pipeline-cutover | ICM-claim-scope kernel (B2 fix) | 1 new crate, ~150 LOC, ~10 tests |
| M-CC-P01-IP-013 | agentic-pipeline-cutover | IP-status-truth kernel (C1 + C2 fix) | 1 new crate, ~200 LOC, ~12 tests |
| M-CC-P01-IP-014 | agentic-pipeline-cutover | IP-dependency-edges kernel (D2 fix) | 1 new crate, ~150 LOC, ~10 tests |
| M-CC-architecture-fixup-IP-001 | new arch-cleanup phase | Re-role usage-window/route-policy/dashboard kernels (A1 residual) | 4 catalog yaml edits + downstream import sweep, ~30 LOC churn |
| M-CC-architecture-fixup-IP-002 | new arch-cleanup phase | Extract `oya-foundry-api-kernel` for shared transport types (A2 fix) | 1 new crate, ~200 LOC, plus 3 adapter Cargo.toml + import edits |

## G. Closing observations (Linus good-taste row)

- The biggest blocker this session was **not** the architecture itself but the **status-truth gap**: half a dozen IPs were already substantively done in code but marked `stub` or `planned-*`. That cost ~3 read-cycles per IP to ground-truth. A single fitness kernel (C1/C2) eliminates that class of waste forever.
- The architecture's **per-symbol claim semantics already work**; the upstream grit FK bug is the only thing forcing fallbacks to wider scopes. Fix the bug upstream → automatic parallelism unlock without code changes.
- The four remaining gate violations are **boundary leaks**, not deep architectural mistakes. Two small ChangeSets close them. The architecture is fundamentally sound; the leaks are well-localized.
