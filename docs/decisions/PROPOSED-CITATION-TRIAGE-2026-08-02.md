---
purpose: Decision packet — triage of Proposed ADRs cited as authority by Accepted ADRs
doc_status: published
last_audited: 2026-08-02
---

# Decision packet — Proposed ADRs cited as authority (2026-08-02)

STATUS: triage-complete. This artifact is a **decision input**, not a decision. It ratifies
nothing. Every ADR status field in `docs/decisions/` is unchanged by this change.

ROLE: Lane C2 deliverable. Produce a RATIFY / DE-CITE ruling for every Proposed ADR that
Accepted ADRs lean on, plus the mechanical blockers that stop the rulings from being applied.

MODE: read-only measurement over the committed corpus. No ADR was edited, no generated face
was touched, no cluster was contacted.

---

## 0. The measurement, and the correction to the premise

Reproduce (all commands run from the repo root):

```
# status of every ADR — first status-bearing line per file, all four authoring shapes
grep -H -m1 -iE '^[-|>#* ]*\**Status\b' docs/decisions/ADR-*.md \
  | sed -E 's#docs/decisions/(ADR-[0-9]{4})[^:]*:#\1|#'
# => 443 files: 188 proposed | 217 accepted | 36 superseded | 2 other

# distinct (citing Accepted ADR, cited ADR) pairs
xargs grep -oH -E 'ADR-[0-9]{4}' < accepted-files.txt \
  | sed -E 's#docs/decisions/(ADR-[0-9]{4})[^:]*:#\1 #' | sort -u
# => 2484 pairs; filtered to cited-is-Proposed and citer!=cited:
# => 455 citations across 101 distinct Proposed ADRs
```

| Premise as briefed | Measured | Delta |
|---|---|---|
| 429 citations | **455** (101 distinct Proposed ADRs) | +26 |
| ADR-0009 x31 | **x32** | +1 |
| ADR-0003 x23 | **x24** | +1 |
| ADR-0111 x16 | **x17** | +1 |
| — | **ADR-0049 x21, ADR-0043 x18 both outrank ADR-0111** | omitted from the brief |

The brief's counts are close enough to be the same phenomenon; the ranking is not. ADR-0111 is
the 5th-most-cited Proposed ADR, not the 3rd, and it is the one case in the top ten whose
correct disposition is neither RATIFY nor DE-CITE (§3).

**Namespace contamination — read this before trusting any raw count.** The corpus carries a
second, foreign ADR namespace written `Bominal ADR-NNNN`, inherited from the predecessor
org, whose ids collide with Oyatie's. 18 distinct ids are affected. A bare `ADR-0020` in
those documents is ambiguous, and for ADR-0020 it is decisive: 4 of its 7 apparent citers
mean **Bominal ADR-0020 (observability posture)**, not Oyatie ADR-0020 (intelligence
multi-provider adapter model). Two of ADR-0009's 32 citers are the same shape.

```
xargs grep -hoiE 'bominal ADR-[0-9]{4}' < accepted-files.txt | grep -oE 'ADR-[0-9]{4}' | sort | uniq -c
#   11 ADR-0107   11 ADR-0028   10 ADR-0106    9 ADR-0132    7 ADR-0209
#    7 ADR-0208    7 ADR-0123    7 ADR-0020    7 ADR-0009    5 ADR-0140 ...
```

---

## 1. The ruling test (repo doctrine, not invented)

`docs/decisions/README.md` §"Status Semantics" already states the test:

> `Proposed` means the decision is under review or **advisory until its enforcement gates
> promote**. `Accepted` means new work follows it.

So the disposition question is answerable from evidence, not taste:

- **RATIFY** — the enforcement gates HAVE promoted, or Accepted ADRs already treat it as
  binding. The decision is de-facto authority and `Proposed` misdescribes it.
- **DE-CITE** — the citation is decorative (a bare `related:` entry, a background mention),
  or a live Accepted successor already owns the substance. Repoint the citation.

Three evidence axes, all mechanically derived:

| Axis | Meaning | How measured |
|---|---|---|
| **A** | distinct Accepted ADRs citing it | pair set above |
| **N** | normative-form citations (`per/under/required by/governed by/defined in/enforced by ADR-NNNN`) | `grep -hoiE '(per\|under\|required by\|mandated by\|governed by\|defined in\|enforced by) ADR-NNNN'` |
| **C** | references in `*.rs` / `BUCK` / `*.toml` / `*.yml` / `*.yaml` | `grep -rohE 'ADR-[0-9]{4}' --include=...` |

**C is an occurrence count and it inflates.** Two known modes: a provenance stamp repeated
per row of a generated catalog (ADR-0314 reads 827 across nine `oya/*/catalog` trees at 54
files each), and test fixtures using an ADR filename as a dummy string
(`tools/oya-reorg-codemod-app/src/plan.rs:892` uses `ADR-0002-unrelated.md`). Where C
carries a ruling on its own, the file spread is stated instead of the raw count.

**65 of the 188 Proposed ADRs are cited inside `ci/facade/**` — the required-context gate
apps themselves.** The required merge gate reasons from 65 unratified decisions.

```
grep -rohE 'ADR-[0-9]{4}' ci/facade/ | sort -u | grep -F -f proposed.txt | wc -l   # => 65
```

Two of those are enforcement-level, not provenance:

- `ci/facade/facade-core-layering/facade-core-layering-policy.json:5` — `"sequence_adr": "ADR-0328"`.
  A policy field whose *value* is a Proposed ADR.
- `ci/facade/artifact-accountability/src/lib.rs:354,404` — the gate emits
  `"justification_ref": "ADR-0555"`. A Proposed ADR is the justification a blocking gate hands back.

---

## 2. Ruling table

A = Accepted-ADR citers · N = normative-form citations · C = code/config references.

### RATIFY (26) — load-bearing; `Proposed` misdescribes them

| ADR | Title | A | N | C | Why it is already authority |
|---|---|---:|---:|---:|---|
| ADR-0009 | cell architecture per-tenant per-region | 32 | 21 | 16 | in `ci/facade/topology-manifest-contract` + `libs/oya-governance-lifecycle-kernel`; RTO/RPO tables cite it as the source |
| ADR-0003 | audit chain + evidence emission | 24 | 14 | 57 | every mutation hook in the ontology ADRs emits per ADR-0003; cited in gate kernels |
| ADR-0049 | cross-region replication + residency | 21 | 8 | 18 | sovereignty rejections in Accepted ADRs are decided by it |
| ADR-0043 | secrets — OpenBao + per-cell HSM | 18 | 12 | 30 | the cell is "the KMS-shred boundary (per ADR-0043)"; shard key custody, at-rest encryption and HSM partitions all resolve through it |
| ADR-0007 | Cedar authorization + persona tier | 17 | 3 | 16 | Cedar is the live PDP substrate. **Scope caveat:** ratify the Cedar half; the persona-tier half needs reconciling against ADR-0329 (Accepted, tier system retired → tenant-class) |
| ADR-0245 | substrate vs product layering | 10 | 10 | 38 | the `oya/` vs `cloud/` split rule the reorg executes against |
| ADR-0002 | tenant + identity kernel | 10 | 1 | 101 (19 files) | `TenantId` is a type in the tree, not a proposal; refs land in `contracts/openapi/platform`, `oya/identity/catalog`, `iam/observability/slos`, `tenancy/core/kernel` |
| ADR-0244 | tenant as universal scoping primitive | 9 | 10 | 336 | third-highest code footprint of any Proposed ADR |
| ADR-0243 | Cedar as universal gate | 9 | 17 | 234 | highest normative-citation count after ADR-0009/ADR-0328 |
| ADR-0010 | regional pack architecture | 9 | 6 | 15 | its own amendment note says pack roots "remain valid **until ADR-0010/ADR-0064 are explicitly superseded**" — a live authority claim from a Proposed doc |
| ADR-0328 | substance bar canonical sequence | 8 | 19 | 85 | `"sequence_adr": "ADR-0328"` is a required-gate policy field |
| ADR-0251 | compliance-pack cell certification levels | 8 | 12 | 75 | "SOC2/HIPAA/GDPR enterprise tiers (per ADR-0251 certification levels)" decides cell pinning |
| ADR-0248 | Amazon-shape cellular architecture | 8 | 4 | 60 | an Accepted ADR states "ADR-0248 remains the canonical cellular architecture doctrine" |
| ADR-0039 | supply-chain security (Trivy/Cosign/SBOM) | 8 | 2 | 25 | "no native binary blob without an SBOM (per ADR-0039)" |
| ADR-0038 | trust framework + DSR cascade + proof-of-erasure | 8 | 9 | 9 | the DSR cascade is a regulator-facing obligation |
| ADR-0035 | workflow engine state-machine + DAG hybrid | 8 | 3 | 3 | "the workflow engine (per ADR-0035) is the only sanctioned coordinator" |
| ADR-0555 | unaccounted artifacts unmergeable | 7 | 11 | 50 | a required gate emits it as `justification_ref` |
| ADR-0019 | doc catalog + update protocol | 7 | 3 | 7 | deprecation governance is executed against it |
| ADR-0397 | Pulsar 4.x + Oxia canonical event bus | 6 | 1 | 18 | **ADR-0557 (Accepted) rests on it**: "The cluster runs Pulsar 4.x + Oxia (per ADR-0397) as the sole canonical event-bus." Its own frontmatter says `authority: founder-pending-ratification` — it is asking for exactly this decision |
| ADR-0263 | observability emission contract | 6 | 14 | 358 | second-highest code footprint |
| ADR-0045 | database tier strategy | 6 | 3 | — | "commits to per-cell per-µservice Postgres as the canonical primary store" |
| ADR-0255 | intelligence as two-layer AI substrate | 5 | 7 | 82 | — |
| ADR-0253 | network topology — edge + service mesh | 5 | 1 | 461 | **highest code footprint of any Proposed ADR: 411 distinct files** across contracts, SLOs, IaC, journey schemas |
| ADR-0213 | ecosystem-as-a-service architecture | 5 | 2 | 44 | plugin app-store governance cited as `per ADR-0213` |
| ADR-0014 | build-vs-buy policy | 5 | 3 | 1 | used as the *decision rule* in Accepted ADRs' alternatives-rejected sections |
| ADR-0563 | rename-aware path-keyed CI baseline relabel | 5 | 0 | 12 | drives `specs/reorg/move-manifest.generated.json` + `scm-facts-snapshot`; **ADR-0614 (Accepted) declares `amends: [ADR-0563]`** — an Accepted ADR amending an unratified one |

### DE-CITE (3) — the citation is decorative or points at the wrong decision

| ADR | Title | A | N | C | Repoint to | Evidence |
|---|---|---:|---:|---:|---|---|
| ADR-0044 | service mesh — Istio Ambient + Envoy Gateway | 6 | 2 | 6 | **ADR-0148** (Accepted) + **ADR-0182** (Accepted) | ADR-0148 "REWRITES the prior framing" into Cilium L3/L4 + Istio Ambient L7 layered; ADR-0182 owns north-south vs east-west. Neither declares a `supersedes` edge, so ADR-0044 sits Proposed looking live |
| ADR-0020 | intelligence multi-provider adapter model | 7→**3** | 0 | 4 | disambiguate | 4 of 7 citers mean **Bominal** ADR-0020 (observability). Real Oyatie citers = 3, none normative. De-cite, and qualify the Bominal refs so the collision stops counting |
| ADR-0114 | canary observability rollback | 10 | 0 | 6 | **ADR-0139** (agentic SLO-gated promotion, named in root `CLAUDE.md`) | N=0. Every sampled citation is a bare `related:` entry or "precedent; this ADR is its concrete implementation" — textbook decorative |

### RESOLVE-SUPERSESSION (2) — neither ruling fits

Forcing these into RATIFY/DE-CITE would be wrong in both directions: the substance is already
absorbed by an Accepted successor (so ratifying is a lie), and the citations are load-bearing
(so plain de-citation drops a real dependency on the floor).

| ADR | A | N | C | Correct end state | Evidence |
|---|---:|---:|---:|---|---|
| ADR-0111 merge queue | 17 | 1 | 29 | **Superseded by ADR-0515** | Its entire sibling cluster — ADR-0110, ADR-0112, ADR-0113, which are also its whole `related:` list — is Superseded by ADR-0363 (Accepted). ADR-0363 §"GitHub has no native merge queue" folds ADR-0111's projected-state semantics into cloud-ci/oya-ci Tide, now owned by ADR-0515 (Accepted). ADR-0111 itself: `status: Proposed`, `superseded_by: []`. Root `CLAUDE.md` still lists it under `current_substrate_adrs` |
| ADR-0005 Kafka eventing + outbox | 12 | 5 | 20 | **Superseded by ADR-0557**; repoint outbox-semantics citations at ADR-0557 §4 | Already inconsistent in its own frontmatter: `status: proposed` **and** `superseded_by: [ADR-0557]`. ADR-0557 (Accepted) declares `supersedes: [ADR-0005]` and restates the carry-forward: "The streaming semantics decisions (transactional outbox, at-least-once, consumer-group fanout) carry forward under Pulsar's equivalent primitives and remain normative" |

---

## 3. Mechanical blockers — the rulings cannot be applied without these

### B1 (blocking, one-line fix) — `proposed → superseded` is not a legal transition

`specs/lifecycle-configs/adr-status-lifecycle.json` declares exactly five transitions:

```
proposed → accepted    proposed → archived
accepted → superseded  accepted → archived   superseded → archived
```

There is **no `proposed → superseded`**. So neither ADR-0111 nor ADR-0005 can be moved to its
correct state. The three available moves are all wrong:

1. ratify it purely in order to retire it — records a decision that was never made;
2. archive it — `archived` has `requires_supersession_edge: false`, so the pointer to the
   successor is dropped and the trail to ADR-0515 / ADR-0557 is lost;
3. leave it — the status quo, which is the defect.

The fix is a config file, which is exactly the extension shape the gate was designed for
(`lifecycle-status-policy.json`: *"Adding a lifecycle is a CONFIG file here, never a new crate"*).
Adding the transition is a prerequisite for applying the §2 RESOLVE-SUPERSESSION rulings, and
it is not a founder decision — it is a modelling gap.

### B2 (undetected defect class) — a supersession edge on a non-terminal stage is not a violation

`libs/oya-governance-lifecycle-kernel/src/lib.rs:363-370` flags `MissingSupersession` only when
`stage.terminal && stage.requires_supersession_edge`. The inverse — a **non-terminal** stage
carrying a `superseded_by` edge — has no violation kind. Live instances:

```
xargs grep -H -m1 -iE '^status:' < has-superseded-by.txt | grep -ivE 'superseded'
# docs/decisions/ADR-0005-...:status: proposed      (superseded_by: [ADR-0557])
# docs/decisions/ADR-0065-...:status: accepted
# docs/decisions/ADR-0316-...:status: Proposed
```

Same class, seen from the other side: **ADR-0392 (Accepted) declares `supersedes: [ADR-0358]`
while ADR-0358 is still `Proposed`.** Full tally of the 29 ids named in some `supersedes:` list:

| Target state | Count | Ids |
|---|---:|---|
| correctly `Superseded` | 22 | 0042 0046 0052 0107 0110 0112 0113 0120 0121 0124 0140 0141 0183 0349 0359 0361 0372 0511 0513 0514 0550 0596 |
| exists, **not** Superseded | 3 | **ADR-0358** (Proposed), **ADR-0005** (proposed), ADR-0055 (accepted) |
| **file does not exist** | 4 | **ADR-0421, ADR-0429, ADR-0443, ADR-0457** |

The last row is a third undetected shape: four Accepted ADRs (ADR-0476, ADR-0478, ADR-0479,
ADR-0480) declare `supersedes:` edges pointing at ADRs that have no file in `docs/decisions/`.
Nothing resolves those ids, so nothing reports them.

### B3 (why the whole 455 is invisible to CI) — propagation is enforced for Accepted only

`ci/facade/cross-artifact-agreement/src/lib.rs:4043-4046`, verbatim:

```rust
// Propagation is required for the exact lifecycle spellings recognized as live by
// the ADR shape kernel. Superseded/Proposed decisions are not expected to carry
// masterplan/roadmap nodes.
let is_live = is_live_decision_status(status);
```

`orphan_decision` and `unpropagated_decision` — the corpus's only decision-coherence codes —
skip Proposed ADRs entirely. That exclusion is correct for a *proposal*. It is wrong for a
Proposed ADR that a required gate app cites as its own justification. There is no code today
that can observe "an Accepted decision depends on an unratified one", which is why 455
citations accumulated silently.

**This packet does not create that check.** Wiring one means either a new violation kind in
the shared lifecycle kernel or a new code in `cross-artifact-agreement`, both of which land
inside the required fan-in and both of which are ratification-shaped, not triage-shaped.
Naming it here is the honest stopping point.

---

## 4. What the founder is being asked to decide

1. **Ratify the 26.** Not one at a time — the ranking is stable across all three axes and the
   evidence is uniform. Cheapest coherent unit of work.
2. **Confirm the 3 de-citations**, including the ADR-0020 namespace disambiguation.
3. **ADR-0111 and ADR-0005 → Superseded**, contingent on B1 landing first.
4. **Root `CLAUDE.md`**: `current_substrate_adrs` lists ADR-0111 as current while its own
   comment says it was folded into ADR-0515. After (3) it moves to the historical list.
5. **B1, B2, B3** are separate lanes. B1 is a config line. B2 is a violation kind in the shared
   kernel. B3 is a design question, not a patch.

Scope honesty: 101 distinct Proposed ADRs are cited by Accepted ones. This packet rules on the
31 with ≥5 Accepted citers, which carry 316 of the 455 citations (69%). The remaining 70 ADRs
sit at 1-4 citations each and are not worth a founder's attention until the head of the
distribution is resolved.
