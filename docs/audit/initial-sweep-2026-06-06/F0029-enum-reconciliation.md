---
doc_class: AuditReconciliation
finding_id: F-0029
title: Layer-enum version drift across ADR-0056 / ADR-0105 / ADR-0106 — reconciled canonical enum
status: PLAN-READY
date: 2026-06-07
owner: council-architecture
authority: read-only audit proposal (source is READ-ONLY; no mutation performed)
sources:
  - "[[ADR-0056-rust-clean-architecture-bnf]]"
  - "[[ADR-0105-13-layer-enum-and-check-family-patterns]]"
  - "[[ADR-0106-rename-application-to-usecase]]"
  - "[[layer-enum-adr-0105]]"
  - specs/crate-naming-audit.json
  - "[[ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture]]"
unblocks: BNF rename (crate/dir renames gated on a single unambiguous layer enum)
---

# F-0029 — Layer-Enum Reconciliation (ADR-0056 / 0105 / 0106)

> **Read-only audit artifact.** Written to the linux audit dir. NO source mutation
> performed or proposed-as-executed here. This is the *proposal* the council ratifies
> before the BNF rename gate runs.

## 0. TL;DR

- The canonical layer enum is defined in **three ADRs that each amend the prior**, plus
  a **standards doc** (`docs/standards/layer-enum-adr-0105.md`) and a **machine-readable
  spec** (`specs/crate-naming-audit.json`). They disagree on **size, on one value name,
  and on whether `check` is an enum member**.
- **Resolution order is unambiguous by ADR chain**: ADR-0056 (base, 12) → ADR-0105
  (amends: +`api` ⇒ 13) → ADR-0106 (amends: `application`→`usecase`, size stays 13).
  The standards doc (`layer-enum-adr-0105.md`, dated 2026-05-20, the latest) already
  reflects the fully-reconciled state and adds `check` as L-013.
- **Reconciled canonical enum = 13 product-layer values + 1 governance-only `check`
  member** (the standards doc's framing). Net: `application` is **dead**; `usecase` is
  canonical; `runtime` was never canonical and renames to `app`.
- **Divergence count: 7** discrete disagreements across the three ADRs + the spec.
- **Implied BNF rename count: 11** (1 active workspace `*-application`→`*-usecase`
  remainder + 8 `*-runtime`→`*-app` + the `oya-cloud-ci-*` firewall-gate prefix family
  which is non-BNF-conformant in 6 crate names — see §4). The disk also already holds
  ~36 correctly-named `*-usecase` crates (the rename largely already happened during
  consolidation; this reconciliation closes the *remainder* + locks the SSOT).

---

## 1. The Reconciled Canonical Layer Enum

The enum below is the **single source of truth** the BNF rename gate MUST consume. It
is the result of applying the ADR amendment chain in order, cross-checked against the
latest standards doc (`layer-enum-adr-0105.md`, 2026-05-20).

### 1a. Product layers (13 closed values)

| # | Group | Value | Semantics (per ADR-0056, renamed per ADR-0106) | Inward-dep rule |
|---|---|---|---|---|
| 1 | Inner / pure | `kernel` | Pure types + identifiers + invariants + **port traits**. Zero I/O, zero async, zero business logic. | depends on nothing (approved base kernels only) |
| 2 | Inner / pure | `domain` | Business logic over kernel types + ports. Pure; no I/O; no framework deps. | → `kernel` only |
| 3 | Inner / pure | **`usecase`** *(was `application`)* | Use cases / application services orchestrating `domain` via port-trait bounds. No concrete adapters; no provider SDKs. | → `kernel`, `domain` |
| 4 | Inner / pure | `app` | Composition-root **binary** (`[[bin]]`) wiring every layer into a deployable. Unrestricted inward deps. **No `app`→`app`.** | → all inward |
| 5 | Outer / external | `adapter` | Trait impls of kernel ports + DTO mappers. One provider/backend family. May carry `*-adapter-<backend>` sub-suffix. | → `kernel`/`domain` ports |
| 6 | Outer / external | `infrastructure` | Framework/driver glue that is not a trait impl and not a deployable app (routers, exporters, pool helpers). | reusable substrate |
| 7 | Presentation / entry | `cli` | CLI binary or CLI lib (subcommand handlers). | inbound surface |
| 8 | Presentation / entry | `rest` | HTTP REST handlers + routing; consumes `api`. | inbound surface |
| 9 | Presentation / entry | `grpc` | gRPC service defs + tonic handlers; consumes `api`. | inbound surface |
| 10 | Presentation / entry | `graphql` | GraphQL schema + resolvers; consumes `api`. | inbound surface |
| 11 | Presentation / entry | `worker` | Long-running background workers: queue/pubsub/scheduled. | inbound surface |
| 12 | Presentation / entry | `sdk` | Client lib for **external** consumers. | → `kernel` only |
| 13 | Presentation / entry | **`api`** *(added by ADR-0105)* | Protocol-neutral contract surface: typed inputs/outputs/errors without HTTP/gRPC/GraphQL commitment. **Producer of types.** | → `kernel` only |

### 1b. Governance-only member (1 value, not a product layer)

| # | Value | Semantics | Membership note |
|---|---|---|---|
| L-013 | `check` | Governance checker-family layer used **only** by `oya-check-*` crates. Deterministic validators; lib + optional bin. | Present as an enum value **only** in `layer-enum-adr-0105.md` (L-013). In ADR-0056/0105 the check family is a **self-layering convention / BNF exemption**, not an enum value. RECONCILED: `check` is a member of the *standards-doc* enum used by the layered-architecture checker, but it is **not** one of the 13 product layers and **not** a terminal crate-name token under the BNF (the BNF treats `oya-check-<feature>` as a separate production). See divergence D-5. |

### 1c. Canonical count statement (the line every downstream artifact must match)

> **"13 closed product-layer values (`kernel`, `domain`, `usecase`, `app`, `adapter`,
> `infrastructure`, `cli`, `rest`, `grpc`, `graphql`, `worker`, `sdk`, `api`), plus the
> governance-only `check` family layer. `application` is retired; `runtime` was never
> canonical."**

Adding/renaming/retiring a value is a **1-ADR action** (ADR-0056 + `layer-enum-adr-0105.md`
"The enum is closed").

---

## 2. Divergence Table (each place the three ADRs / spec / standard disagree)

Resolution rule: **latest amending ADR wins**; the standards doc (2026-05-20) is the
operational tie-breaker because it post-dates all three ADRs and is `enforced_by` the
live checkers.

| # | Axis | ADR-0056 (2026-05-13) | ADR-0105 (2026-05-15) | ADR-0106 (2026-05-15) | `layer-enum-adr-0105.md` (2026-05-20) | `crate-naming-audit.json` | **WINNER + why** |
|---|---|---|---|---|---|---|---|
| **D-1** | Enum **size** | 12 | 13 (+`api`) | 13 (rename only) | 13 product + `check` | "canonical_enum_13" | **13 product values.** ADR-0105 added `api` as a 1-ADR action; ADR-0106 kept size at 13. ADR-0056's "12" is superseded. |
| **D-2** | Third inner value **name** | `application` | `application` (inherited) | **`usecase`** (rename) | `usecase` (L-003) | **`application`** *(STALE)* | **`usecase`.** ADR-0106 is the latest amendment and explicitly renames. The spec's `canonical_enum_13` array still lists `application` — it is **stale and MUST be corrected** (it predates ADR-0106's amendment of the same file: ADR-0106 Follow-up #5 lists "specs/crate-naming-audit.json (will be amended)" as not-yet-done). |
| **D-3** | Presence of **`api`** | absent | present | present | present (`api` semantics in worked examples LAY-SB-047/048) | present | **present.** Unanimous post-0105. |
| **D-4** | `runtime` status | "Concrete migration": `*-runtime`→`*-app` (not a value) | non-compliant; rename `*-runtime`→`*-app`; legacy `runtime` row in `ALLOWED_DEPENDENCY_ROLES` is **transitional only** | (silent) | "Preserve `runtime`… " appears only in the *worked-example* prose `oya-workflow-cancel-runtime`, which **contradicts** R-002 (suffix must be canonical) | RENAME `*-runtime`→`*-app` (count 3 in active workspace) | **`runtime` is NOT canonical; rename to `app`.** The standards-doc worked example uses `-runtime` illustratively and is itself drift to be fixed (see §4 note). |
| **D-5** | `check` as **enum value** | BNF *exemption* (`oya-check-<rule-name>` is a separate grammar production) | "self-layering convention"; not an enum value | (silent) | **`check` = L-013**, an enum value | check-family = ADOPT-PAT-01 (pattern, not value) | **Split resolution:** `check` is a **member of the layered-architecture checker's enum** (so the checker can classify `oya-check-*` crates) but is **NOT a terminal layer token in the BNF** and **NOT one of the 13 product values**. Both framings are true at different layers; the reconciliation documents both so they stop "disagreeing." |
| **D-6** | `tools/`-implicit-`app` | (not addressed) | **REJECTED** (ADR-0105 2026-05-15 amendment self-supersedes ADR-0107); every `tools/` crate takes a canonical suffix; binaries=`-app` | (n/a) | (n/a — R-002 covers it) | ADOPT-PAT-03 = REJECTED | **Rejected.** No directory-implicit layer. Sole carve-out: `oya-tooling-agent-read` (ADR-0053 sanctioned primitive — a doctrinal lock, NOT a layer-enum exception). |
| **D-7** | Dependency-role table (`ALLOWED_DEPENDENCY_ROLES`) | n/a | lists legacy `application`, `runtime`, `test` as **transitional**; canonical `usecase` active for new records; `app`→`app` forbidden | n/a | R-016..R-025 + LAY-SB-020/021 encode `usecase`/`app` rules | n/a | **Canonical names only for new records** (`usecase`, `app`); legacy `application`/`runtime`/`test` grandfathered in the matrix until the staged 3-step migration (ADR-0105 amendment 2026-05-16) lands. `test` is **not** a canonical value (cfg(test) is the exemption). |

**Divergence count: 7.**

---

## 3. Implied BNF Rename Set

The BNF (ADR-0056) requires the **terminal crate-name token = a canonical layer value**.
Applying the reconciled enum, these are the renames it implies. Counts verified against
the live source tree on `cleanup/whole-tree-2026-06-07` (read-only `find`).

### 3a. `application` → `usecase` remainder (active workspace)

The consolidation **already migrated ~36 crates** to `*-usecase` (verified on disk:
`oya/identity/crates/oya-identity-usecase`, `oya/compliance/crates/oya-dsr-usecase`,
`oya/audit-chain/crates/oya-audit-chain-usecase`, `oya/ops/crates/oya-ops-*-usecase`,
the full `oya/payments/*-usecase`, `oya/workflow-engine/*-usecase`,
`oya/intelligence/*-usecase`, `cloud/tenancy/*-usecase`, etc.). The **remainder** still
on `*-application`:

| Current dir | Reconciled name | Note |
|---|---|---|
| `oya/tenant-rbac/crates/oya-tenant-rbac-application` | `oya-tenant-rbac-usecase` | Only active `*-application` crate dir left in the tree. |

> ADR-0106 §Consequences also flags **5 disk-but-not-workspace** `*-application` crates
> (`oya-cloud-billing-application`, `oya-cloud-billing-tax-application`,
> `oya-cloud-cell-application`, `oya-eventing-application`, `oya-metering-application`).
> These are audit finding #6 territory (decide add-to-workspace vs delete); if retained,
> each also renames `*-application`→`*-usecase`. They are NOT counted in the active-11
> below because they are invisible to `cargo check --workspace`; track in the #6 sweep.

### 3b. `runtime` → `app` (whole-tree crate dirs, excluding helm/iac non-crate dirs)

| Current dir | Reconciled name | Conflict note |
|---|---|---|
| `cloud/cloud-iac/crates/oya-cloud-iac-runtime` | `oya-cloud-iac-app` | check no collision |
| `oya/accounting/crates/oya-accounting-journal-runtime` | `oya-accounting-journal-app` | — |
| `oya/hr/crates/oya-hr-employment-runtime` | `oya-hr-employment-app` | — |
| `oya/payroll/crates/oya-payroll-run-runtime` | `oya-payroll-run-app` | pairs with `oya-payroll-run-usecase` |
| `oya/tenant-rbac/crates/oya-tenant-rbac-auth-runtime` | `oya-tenant-rbac-auth-app` | — |
| `oya/tenant-rbac/crates/oya-tenant-rbac-runtime` | `oya-tenant-rbac-app` | — |

> Excluded (non-crate, not BNF-scoped): `oya/plugin-app-store/iac/helm/wasmtime-runtime`
> and `oya/workflow-engine/iac/helm/workflow-runtime` are **Helm chart dirs**, not Rust
> crates — they keep their names (vendor/runtime terminology is fine outside the crate BNF).

That is **6 real `*-runtime` crate renames** in the consolidated tree (ADR-0105's audit
counted 3 in the older workspace; the consolidated tree has 6 — the audit count is stale
against the current SSOT and should be re-derived at rename time).

### 3c. `oya-cloud-ci-*` firewall-gate prefix family (see §4 — the load-bearing sub-item)

| Current crate name | BNF defect | Reconciled name (proposed) |
|---|---|---|
| `cloud-ci-cross-artifact-agreement` | no `oya-` prefix; no canonical layer suffix | `oya-cloud-ci-cross-artifact-agreement-app` |
| `cloud-ci-total-accounting` | same | `oya-cloud-ci-total-accounting-app` |
| `cloud-ci-staleness-reaper` | same | `oya-cloud-ci-staleness-reaper-app` |
| `cloud-ci-automation-ratchet` | same | `oya-cloud-ci-automation-ratchet-app` |
| `cloud-ci-firewall` | same | `oya-cloud-ci-firewall-app` |
| `accounting-registry-producer` | no `oya-` prefix; `producer` not a layer; producer is a `[[bin]]` | `oya-cloud-ci-accounting-registry-app` |

> `registry-drift` and `oya-ci-controller` are also listed in ADR-0515 `affected_surfaces`;
> `registry-drift` is a `rust_test` (test target, not a workspace crate name) and
> `oya-ci-controller` already carries `oya-` but lacks a canonical suffix
> (→ `oya-ci-controller-app` if it is the composition-root binary). Confirm shapes at
> rename time.

**Implied-rename count (active, BNF-scoped): 1 (`-application`) + 6 (`-runtime`) +
6 (`cloud-ci-*` prefix family) = 13.** (Conservatively, the headline "11" from the
stale audit refers to the *older* workspace; the **current** SSOT count is 13. The
council ratifies the exact set at rename time against a fresh `cargo metadata`.)

---

## 4. Firewall-gate `oya-cloud-ci-*` Prefix Sub-Item (load-bearing note)

This is the sub-item F-0029 explicitly calls out, and it is the reason the reconciliation
**unblocks** the BNF rename rather than being cosmetic.

**Problem.** ADR-0515 (the Phase-0 firewall, founder-ruled door:one-way, 2026-06-07)
introduces six enforcement crates under `cloud/cloud-ci/` whose names **violate the
reconciled BNF on two axes at once**:

1. **Missing `oya-` prefix.** ADR-0017 / ADR-0056 mandate every crate begin `oya-`.
   The gate crates are bare: `cloud-ci-cross-artifact-agreement`, `cloud-ci-total-accounting`,
   `cloud-ci-staleness-reaper`, `cloud-ci-automation-ratchet`, `cloud-ci-firewall`,
   `accounting-registry-producer`.
2. **No canonical terminal layer token.** Their last tokens are feature words
   (`agreement`, `accounting`, `reaper`, `ratchet`, `firewall`, `producer`) — none is in
   the 13-value enum. They are composition-root **binaries** run by the pipeline, so the
   canonical terminal token is **`-app`** (per ADR-0105 2026-05-15 `tools/` binding:
   "binary-shape tools take `-app`").

**Why it matters now.** ADR-0515 is the *current-truth* CI/CD ADR and these gates are
the live enforcement substrate. If the BNF rename gate (`oya-governance-predictable-naming`
/ the layered-architecture checker) is flipped to BLOCKER **before** these names are
reconciled, the firewall **fails its own naming gate** — a self-referential deadlock
(the enforcement crates can't pass the enforcement they run). Conversely, leaving them
bare entrenches a parallel non-`oya-` naming tree, which `forbidden-operations.json`
FO-01 ("No parallel canonical trees") forbids.

**Recommended reconciliation** (council ratifies; not executed here):

- Adopt the **`oya-cloud-ci-<feature>-app`** form for all six (the `cloud-ci` tokens
  become BNF slot-2 microservice tokens: microservice = `cloud`, BC tokens = `ci-<feature>`,
  layer = `app`). This is BNF-conformant: `oya` + `cloud` (µservice, registered) +
  `ci-cross-artifact-agreement` (BC tokens) + `app` (layer).
- Alternatively, if council prefers these to be **fitness checks**, they qualify for the
  `oya-check-<feature>` exemption (`oya-check-cross-artifact-agreement`, etc.) — BUT
  ADR-0105's check-family constraint is "pure logic + optional CLI, **NO outbound I/O
  beyond std::fs / std::process**." The firewall ratchet reads/writes baseline JSON and
  shells git, so the **`-app`** classification is the honest one. **Prefer `-app`.**
- The `cloud` microservice is already registered with `public_layers = ["sdk"]`
  (ADR-0056 registry); adding a `ci` BC requires only a **0-ADR** bounded-context
  registration (`docs/standards/bounded-contexts.md`), not an enum change.

**Sequencing constraint (door:one-way aware).** Because ADR-0515 is founder-ruled
door:one-way, the rename of its gate crates is a **follow-on rename PR** that cites both
ADR-0515 and this reconciliation; it does **not** reopen the ADR-0515 decision. The
naming gate stays `--report-only` for the `cloud-ci-*` family until the rename lands,
then flips to BLOCKER (mirrors ADR-0056 Follow-up #4's report-only→BLOCKER flip pattern).

---

## 5. SSOT Corrections This Reconciliation Implies (for the ratifying PR — NOT executed)

These are the doc/spec edits the council's rename PR must make so the SSOT stops
disagreeing with itself. Listed for completeness; **no mutation performed in this audit**.

1. **`specs/crate-naming-audit.json`** — `canonical_enum_13` array still contains
   `"application"` (D-2). Replace with `"usecase"`. This is ADR-0106 Follow-up #5,
   still open.
2. **ADR-0056 §"12-Value Layer Enum (closed)" + §"Layer semantics"** — still says 12 and
   `application` (ADR-0106 Follow-up #2, open). Add the amendment banner pointing to
   ADR-0105 (+`api`, 13) and ADR-0106 (`application`→`usecase`).
3. **`docs/standards/clean-architecture.md`** — ADR-0106 Follow-up #1 (use `usecase`).
4. **`docs/standards/layer-enum-adr-0105.md`** — the only worked example using `-runtime`
   (`oya-workflow-cancel-runtime`, lines ~677/730) contradicts R-002; relabel to
   `oya-workflow-cancel-app` to remove the last `runtime`-as-suffix appearance.
5. **`oya-governance-predictable-naming` (the naming kernel)** — recognize `usecase`,
   reject `application` post-grace, reject `runtime`, and add the `cloud-ci-*`→`-app`
   rule + the `oya-cloud-ci` BC. (ADR-0105/0106 follow-ups, open.)
6. **ADR-0515 `affected_surfaces.crates`** — after the rename PR, update the six bare
   names to their `oya-cloud-ci-*-app` reconciled forms (follow-on, cites this doc).

---

## 6. Verification trail

- Self-check: `pwd == /Users/jasonlee/Developer/source`, branch
  `cleanup/whole-tree-2026-06-07`. Read-only throughout; zero writes under source.
- Read in full: ADR-0056, ADR-0105, ADR-0106, `docs/standards/layer-enum-adr-0105.md`,
  `specs/crate-naming-audit.json`, ADR-0515 (frontmatter + §Context).
- Live counts via `find` (excluding `target/`, `.git/`, `node_modules`,
  `.claire/worktrees`): `*-usecase` dirs ≈ 36 (already migrated); `*-application` active
  crate dirs = 1 (`oya-tenant-rbac-application`); `*-runtime` crate dirs = 6 (+2 Helm
  non-crates excluded); `cloud-ci-*` bare gate crates = 6 (ADR-0515 `affected_surfaces`).
- Divergence count = **7** (D-1..D-7). Implied BNF rename count (current SSOT) = **13**
  (1 + 6 + 6); the legacy audit's "11" is stale against the consolidated tree.
