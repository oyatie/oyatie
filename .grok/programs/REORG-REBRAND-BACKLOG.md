# Reorg / rebrand / rewrite debt — actionable backlog

**Mined:** 2026-08-05 · **Doctrine:** [`REORG-DOCTRINE.md`](REORG-DOCTRINE.md) (authority + classes + multi-capability span)  
**Sources:** `AUTHORITY-AND-MINED-BACKLOG.md`, `SESSION-BACKLOG.md`, `MASTER-PARALLEL-DRIVE.md`,  
  CAS plan `cas-fabric/evidence/approved-plan-cas-re-20260805.md`, ultragoal snapshot  
  `cas-fabric/evidence/ultragoal-goals.snapshot.json`, Accepted face/brand ADRs **0613–0616, 0619**.  
**Not merge authority.** Cards are backlog, not admission.

### Policy (expanded)

- **Accepted ADRs (as amended) beat reorg plans.** Re-query on `origin/dev` before each PR.  
- **Reorg ≠ only move.** Classes: **move | refactor | rewrite | delete | rebrand | mixed(stages)**.  
- **May span multiple capabilities** as a program epic; **each PR stays one concern** with temporal path ownership.  
- **Move-plans** apply only to **move-class** path bijections (ADR-0614: at most one active `*-move-plan.json`; manifest derived, never hand-edited / re-tracked).  
- Refactor / rewrite / delete / rebrand do **not** require a move-plan unless paths also rehome.  
- No hand-edit of `*.generated.json`; no external harness brand as live authority (ADR-0619).  
- Spent/stale plans → close bead with evidence; do **not** re-execute (billing #1107 pattern).

**Lenses:** Essentialism · Chesterton · Opportunity cost · Blast-radius · Systems · Zero-trust · Operability.

---

## Ranking key

| Wave | Meaning |
|------|---------|
| **W0** | Path-disjoint, low coupling; can run while CAS trials babysit (no warm CAS / RE) |
| **W1** | After G039 terminal packet (or path-disjoint process-only) |
| **W2** | Ordered CAS reorg 3A→3B→3C (program-critical; not “nice cleanup”) |
| **W3** | Portfolio reorg (intelligence/libs/tools/tail) — **may be multi-capability**; class may be refactor/rewrite/delete not only move |
| **W4** | Rewrite / brand residue / corpus reduction / delete — after graphs exist |
| **park** | Superseded or blocked by unowned authority |

**Risk:** L=local/docs · M=multi-consumer rehome · H=admission/CAS/security-adjacent · C=capability topology rewrite / multi-cap span.

**Card fields (required going forward):** `class` · `capability_span` · `authority` · owned paths · non-goals · verify.

---

## Ranked cards

### 1. `RR-CAS-3A` — NativeLink storage rehome (Lane 3A)

| Field | Value |
|-------|--------|
| **id** | RR-CAS-3A |
| **title** | Move NativeLink CAS manifest/OWNERS to `storage/adapters/nativelink/` |
| **owned paths** | `infra/nativelink/nativelink-cas.k8s.yaml`, `infra/nativelink/OWNERS` → `storage/adapters/nativelink/*`; `specs/reorg/nativelink-storage-move-plan.json` (sole active plan for lane); GitOps/reachability consumers of those paths |
| **prerequisite** | G039 terminal (#1558 promoted + packet); fresh worktree from promoted `dev` |
| **risk** | H — storage topology; must preserve cache-only / proof-only RWO byte-for-byte except path metadata |
| **wave** | W2 |
| **parallelizable?** | **No** with 3B/3C; yes with R1/R2/R7/R8 if path-disjoint |
| **notes** | Delete empty `infra/nativelink/`; no alias; population parity `N_pre=N_post=N_promoted>0`; no warm flip |

### 2. `RR-CAS-3B` — Buck2 cache package atomic move (Lane 3B)

| Field | Value |
|-------|--------|
| **id** | RR-CAS-3B |
| **title** | Atomic move of Buck2 cache package + warm overlays to `build/buck2/cache/` |
| **owned paths** | `toolchains/cache/{BUCK,defs.bzl,OWNERS}`; `infra/ci/buckconfig/warm-cache-*.buckconfig` → `build/buck2/cache/`; all active `toolchains//cache` consumers; `specs/reorg/buck2-cache-move-plan.json` |
| **prerequisite** | **Exact promoted 3A head** only |
| **risk** | H — CI/build graph; dual-home forbidden; root `.buckconfig` must stay dark |
| **wave** | W2 |
| **parallelizable?** | **No** with 3A/3C |

### 3. `RR-CAS-3C` — CI cache policy behavior-only closure (Lane 3C)

| Field | Value |
|-------|--------|
| **id** | RR-CAS-3C |
| **title** | Canonical-path daemon lifecycle + policy semantics (no move plan) |
| **owned paths** | `ci/facade/build-cache-policy/**` behavior; regression tests; bridge workflows stay in place (no rehome) |
| **prerequisite** | Exact promoted **3B** head; pre-edit old-path scan must be clean (else corrective 3B) |
| **risk** | H — cold inheritance / false warm |
| **wave** | W2 |
| **parallelizable?** | **No** with 3A/3B |

### 4. `RR-BILLING-PLAN` — Execute open billing move plan

| Field | Value |
|-------|--------|
| **id** | RR-BILLING-PLAN |
| **title** | Land REORG-002: `libs/oya-metering-pipeline-kernel` → `billing/core/metering-pipeline-kernel` |
| **owned paths** | `libs/oya-metering-pipeline-kernel/`; `billing/core/metering-pipeline-kernel/`; **sole** committed `specs/reorg/billing-move-plan.json` until replaced by another lane’s plan; workspace/Cargo/Buck consumers of crate name |
| **prerequisite** | Codemod + derived manifest on demand (ADR-0614); confirm no concurrent other active reorg plan conflict policy (plan says one committed `*-move-plan.json`) |
| **risk** | M — single leaf, registry-mapped straggler; cargo rename `billing-metering-pipeline-kernel` |
| **wave** | W0 (path-disjoint from CAS 3A if plans sequenced — **replace** active move-plan surface when landing) |
| **parallelizable?** | **No** concurrent second move-plan commit; yes with non-reorg lanes |

### 5. `RR-FACE-DECOMMIT` — Face de-commit leftovers (0613–0616)

| Field | Value |
|-------|--------|
| **id** | RR-FACE-DECOMMIT |
| **title** | Audit and remove re-committed derived faces / projections that violate Accepted de-commit ADRs |
| **owned paths** | Any tracked `*.generated.json` hand-drift; controller projections (ADR-0613); reorg **move manifests** re-tracked (ADR-0614); capability face layout drift vs ADR-0615; immutable merge-base face regen (ADR-0616); materialize script `infra/ci/materialize-cloud-ci-generated-faces.sh` consumers |
| **prerequisite** | Diff-policy / face-aware gates green baseline on `origin/dev`; R2 must not weaken face policy |
| **risk** | H — admission false-green if hand-edited faces re-enter |
| **wave** | W0 audit (docs+gate evidence) → W1 fix PRs per face family |
| **parallelizable?** | Yes per **disjoint face family**; no concurrent hand-edit of same generated face |
| **notes** | Never mint new committed projections for CAS plan; derive on demand |

### 6. `RR-HARNESS-0619` — External harness brand retirement residue

| Field | Value |
|-------|--------|
| **id** | RR-HARNESS-0619 |
| **title** | Sweep active context for retired external coordination brands (omc/omx/gjc/hermes as **control plane**) |
| **owned paths** | Live agent/docs/process surfaces that re-introduce brand as authority; kit `.grok/` must stay brand-neutral delivery harness; brand-residue gate inputs; historical `.omc`/`.omx` **provenance-only** citations (keep as history, not live drivers) |
| **prerequisite** | ADR-0619 Accepted; D-NO-OMC session directive |
| **risk** | L–M — process confusion / false control plane |
| **wave** | W0 |
| **parallelizable?** | Yes with product lanes if docs-only |
| **notes** | Kit `mm-drive` / `.grok` is the allowed harness; do not re-vendor external agent coordination |

### 7. `RR-INTEL-REMAINDER` — Intelligence remainder reorg (incomplete)

| Field | Value |
|-------|--------|
| **id** | RR-INTEL-REMAINDER |
| **title** | Author then execute intelligence remainder move plan for surviving `oya/intelligence` crates |
| **owned paths** | Plan-only first: `specs/reorg/intelligence-remainder-move-plan.json` (UG G024); later code under `oya/intelligence/**` → registered `intelligence/**` faces; reconcile with ~51 already-moved + prior three intelligence plans |
| **prerequisite** | Destination `intelligence/` registered; capability-registry amendments identified; **no** concurrent CAS 3B path ownership |
| **risk** | C — large surface (~78 crates in UG objective); consumer graph |
| **wave** | W3 (plan) → execute only after plan PR + review |
| **parallelizable?** | Plan-only yes; execution slices by **disjoint BC/face** after owner split |
| **status signal** | UG G024 **pending**; no `intelligence-remainder-move-plan.json` in `specs/reorg/` today |

### 8. `RR-LIBS-DISPOSITION` — Flat `libs/` six-way disposition

| Field | Value |
|-------|--------|
| **id** | RR-LIBS-DISPOSITION |
| **title** | Disposition ~129 flat `libs/*` crates; author `specs/reorg/libs-<capability>-move-plan.json` groups |
| **owned paths** | `libs/**` inventory; plan-only `specs/reorg/libs-*-move-plan.json`; no shared registry mutation in plan story (UG G025) |
| **prerequisite** | Real dep/build graphs; fan-out-zero reuse; HIGH/MEDIUM/LOW confidence homes; billing leaf may clear first as exemplar (RR-BILLING-PLAN) |
| **risk** | C — prior 38% inter-rater disagreement (UG evidence) |
| **wave** | W3 plan · W4 execute per capability |
| **parallelizable?** | Plan authoring by capability group yes; **one** active move-plan commit at a time for execute |
| **notes** | Prefer delete/derive/merge/reuse before rehome (CAS plan present-need ladder) |

### 9. `RR-TOOLS-OYA-TAIL` — Tools + oya product/CI tail plans

| Field | Value |
|-------|--------|
| **id** | RR-TOOLS-OYA-TAIL |
| **title** | Classify 21 tools crates + surviving oya product/CI tail (UG G026) |
| **owned paths** | `tools/**`; surviving `oya/**` product/CI tail; plan artifacts under `specs/reorg/`; keep reorg codemod stationary until prerequisites land |
| **prerequisite** | Separate `oya/ci-*` infrastructure from products; capability-registry amendment list |
| **risk** | M–C |
| **wave** | W3 plan |
| **parallelizable?** | Plan yes; execute after registry amendments + path owners |

### 10. `RR-MOVEPLAN-SINGLETON` — Enforce single active reorg move-plan surface

| Field | Value |
|-------|--------|
| **id** | RR-MOVEPLAN-SINGLETON |
| **title** | Process/gate: exactly one committed `specs/reorg/*-move-plan.json` when a rehome lane is live |
| **owned paths** | `specs/reorg/`; codemod docs; optional gate/check that fails on multiple active plans or tracked derived manifests |
| **prerequisite** | ADR-0614; CAS plan rule; current sole plan = billing |
| **risk** | L–M — concurrent reorg collision |
| **wave** | W0 |
| **parallelizable?** | Yes (docs/gate only) |

### 11. `RR-CLOUD-KERNEL-DEL` — Approved cloud-kernel deletion (UG G023)

| Field | Value |
|-------|--------|
| **id** | RR-CLOUD-KERNEL-DEL |
| **title** | Execute approved deletion of unowned cloud-kernel frame after #1523 promotion evidence |
| **owned paths** | `cloud/cloud-kernel/**` (deletion plan); preserve Asterinas substrate + recovery tag; producer projections regen only |
| **prerequisite** | Re-verify deletion plan oracles; protected green + postmerge |
| **risk** | H — owned-stack layer semantics |
| **wave** | W3 (portfolio; not CAS critical path) |
| **parallelizable?** | Yes if path-disjoint from CAS storage moves |

### 12. `RR-BRAND-RESIDUE` — Product brand / rename residue (ADR-0017 + brand gate)

| Field | Value |
|-------|--------|
| **id** | RR-BRAND-RESIDUE |
| **title** | Coordinated multi-batch brand residue cleanup (no blanket sed) |
| **owned paths** | Surfaces flagged by `oya-check-brand-residue` / brand gates; tautological rebrand pairs (MFL-0011); package/UI brand only — repo slug path retained per ADR-0017 |
| **prerequisite** | Brand gate RED fixtures understood; never sed history as live equality |
| **risk** | M — false equality statements |
| **wave** | W4 |
| **parallelizable?** | Yes by **batch scope** with disjoint paths |

### 13. `RR-CORPUS-G030` — Non-code artifact classify/reduce

| Field | Value |
|-------|--------|
| **id** | RR-CORPUS-G030 |
| **title** | Classify ~13.9k md/yaml/json/toml: keep/reorg/refactor/rewrite/delete/stale-mark |
| **owned paths** | Corpus under docs/specs/registry focus families; graph-wire maintained artifacts; freeze dark bureaucracy |
| **prerequisite** | Code/build graphs available; generated-face boundaries preserved |
| **risk** | M — false delete of authority |
| **wave** | W4 |
| **parallelizable?** | Yes by unit-class after census |

### 14. `RR-FACE-CI-BIRTH` — CI runtime face birth designs (G026 ledger residue)

| Field | Value |
|-------|--------|
| **id** | RR-FACE-CI-BIRTH |
| **title** | Finish CI controller/tide/webhook face birth only with approved move plans (no ad-hoc) |
| **owned paths** | `ci/**` face layout; future move plans only after design proofs |
| **prerequisite** | G026-class designs; ADR-0615 capability boundaries; no dual home with ARC bridge deletion story |
| **risk** | H — admission fabric |
| **wave** | W3 |
| **parallelizable?** | Limited — serialize with 3C / R2 workflow ownership |

### 15. `RR-REORG-INDEX` — Program index pointer (optional docs)

| Field | Value |
|-------|--------|
| **id** | RR-REORG-INDEX |
| **title** | Optional tiny docs pointer to this backlog if humans need a repo-visible index |
| **owned paths** | Prefer `.grok/programs/` only; **only if needed** a one-line pointer under `docs/` or specs root-hub |
| **prerequisite** | None |
| **risk** | L |
| **wave** | W0 |
| **parallelizable?** | Yes |
| **status** | **Not opened** — backlog lives under `.grok/programs/REORG-REBRAND-BACKLOG.md` |

---

## Open move plans under `specs/reorg/`

| Plan file | Capability | Status |
|-----------|------------|--------|
| `specs/reorg/billing-move-plan.json` | billing | **Present** — REORG-002 leaf; execute as RR-BILLING-PLAN |
| `specs/reorg/nativelink-storage-move-plan.json` | storage | **Absent** — author with RR-CAS-3A |
| `specs/reorg/buck2-cache-move-plan.json` | build | **Absent** — author with RR-CAS-3B |
| `specs/reorg/intelligence-remainder-move-plan.json` | intelligence | **Absent** — RR-INTEL-REMAINDER plan story |
| `specs/reorg/libs-*-move-plan.json` | multi | **Absent** — RR-LIBS-DISPOSITION |

---

## Accepted de-commit / brand authority (quick cite)

| ADR | Role for this backlog |
|-----|------------------------|
| **0613** | Controller projections stay derived/de-committed — do not re-commit |
| **0614** | Move manifest derived on demand; no re-track without Accepted reversal |
| **0615** | Capability boundaries; `build/` vs `ci/` placement for CAS reorg |
| **0616** | Frozen reference faces from immutable merge-base source |
| **0619** | Retired external coordination brand must not re-enter live context |
| **0562** | Capability-first topology (placement doctrine for rehomes) |

---

## Parallel drive mapping (MASTER-PARALLEL-DRIVE P8)

| Parallel now? | Cards |
|---------------|--------|
| **Yes (W0)** | RR-FACE-DECOMMIT (audit), RR-HARNESS-0619, RR-MOVEPLAN-SINGLETON, RR-BILLING-PLAN *(if move-plan singleton free)* |
| **After G039 packet** | RR-CAS-3A → 3B → 3C (serialized) |
| **Portfolio later** | RR-INTEL-REMAINDER, RR-LIBS-DISPOSITION, RR-TOOLS-OYA-TAIL, RR-CLOUD-KERNEL-DEL, RR-BRAND-RESIDUE, RR-CORPUS-G030 |

---

## Explicit non-goals for these cards

- No RE activation / `remote_enabled` (see `cas-fabric/evidence/RE-SANDBOX-READINESS.md`)  
- No warm CAS without #1541 + G041  
- No blanket rebrand sed  
- No concurrent multi-capability lift-and-shift into speculative scaffolds  
- No product activation from mining alone  


## Session update 2026-08-05

- **RR-BILLING-PLAN**: already landed via #1107 (2026-07-01); bead oyatie-oso.14 closed as superseded. Plan file retired from trunk.


## Class annotations (session)

| Card | class | capability_span | notes |
|------|-------|-----------------|-------|
| RR-CAS-3A/B | move | multi (storage/build/ci consumers) | path rehome + consumer rewrite; not product rewrite |
| RR-CAS-3C | refactor | single (ci cache policy face) | behavior-only; no move-plan |
| RR-BILLING-PLAN | move | single (billing) | **spent** #1107 — closed |
| RR-FACE-DECOMMIT | refactor (docs/control) | multi (ci faces / docs) | #1565 inventory; dual-home oya/* deferred as multi-cap rewrite/move |
| RR-HARNESS-0619 | rebrand | docs/process | #1566 |
| RR-INTEL-REMAINDER | mixed | multi (intelligence + consumers) | plan-first; may include delete of dual homes |
| RR-LIBS-DISPOSITION | mixed | multi (libs → many caps) | disposition = move and/or delete and/or rewrite per leaf |
| RR-TOOLS-OYA-TAIL | mixed | multi | rewrite/delete candidates, not only move |
| RR-CORPUS / classify | delete + rewrite | multi | corpus reduction is delete-class heavy |

