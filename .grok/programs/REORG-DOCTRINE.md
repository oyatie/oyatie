# Reorg doctrine — authority, span, and operation classes

**Status:** process law for parallel drive (2026-08-05)  
**Not merge authority.** Complements `REORG-REBRAND-BACKLOG.md` and AUTHORITY §1.0.

---

## 1. Authority order (fail-closed)

1. **Live-resolved ADRs on `origin/dev`** — **not** raw `status: Accepted`  
   - Resolve supersession: follow `superseded_by`, and reverse-index later ADRs’ `supersedes:` lists (even if the older file still says Accepted).  
   - Resolve amendments: always load `amended_by` live peers; never apply bare parent alone.  
   - Examples: **0562** only with **0615 / 0635** (and any later); **0515** only with **0624 / 0639** + fabric amends; faces **0613–0616** as a cluster.  
   - Brand: **0619**. Admission floor: live **0515** chain / `oya-ci-required`.  
2. **Consensus program plans** (e.g. CAS plan) — **execution intent only**; must re-run ADR disposition before each PR  
3. **`specs/reorg/*-move-plan.json`** — **move recipes only** (ADR-0614: manifest derived, not re-tracked)  
4. **Mined backlog / ultragoal** — queue signals; go stale; never sole law  

**If plan and ADR disagree:** stop. Amend/supersede plan or ADR edges. Do **not** implement the stale path.

**Defects (all block dispatch):**

| Defect | Meaning |
|--------|---------|
| **Plan lag** | Plan does not account for more recent live ADRs |
| **Stale execute** | Implementing without re-querying tip ADRs and live tree |
| **Stale Accepted** | Treating `status: Accepted` as current law when a later ADR supersedes or amends it |

---

## 2. Reorg is not only “move”

A reorg **slice** may be any of:

| Class | Intent | Typical artifacts | Move-plan required? |
|-------|--------|-------------------|---------------------|
| **move** | Path/label rehome; preserve behavior | `*-move-plan.json`, codemod, OWNERS/reachability | **Yes** when path bijection is the work (0614) |
| **refactor** | Same capability(ies); structure/API cleanup; no intentional behavior change | crates, modules, tests; no new product surface | **No** unless paths relocate across trees |
| **rewrite** | Replace implementation behind a stable seam | new impl + parity tests + delete/strangle old | **No** by default; may pair with later delete |
| **delete** | Remove dead/stale/duplicate surface | deletion PR + consumer proof + OWNERS/registry cleanup | **No**; fail-closed if live consumers remain |
| **rebrand** | Names/docs/brand residue only | docs, package metadata (not repo slug path) | **No** (ADR-0619 constraints) |
| **mixed** | Ordered combo (e.g. refactor → move → delete) | staged PRs, one concern each | Per stage |

**Straight move is a subclass**, not the definition of reorg.

---

## 3. Multi-capability span

Reorg **may span multiple capabilities** when:

- A real dependency or dual-home defect crosses capability boundaries (e.g. `libs/*` → several homes, or `oya/X` vs registered capability)  
- ADR-0615/0562 disposition requires coordinated boundary rulings  
- Blast radius is still **bounded per PR** (see §4)

It must **not**:

- Use multi-capability as an excuse for a mega-PR (runners + CAS + k8s + reorg)  
- Skip **temporal ownership**: one PR has one temporal owner set for every path it touches  
- Run **two active move-plans** at once (0614 / CAS plan singleton rule for *move* lanes)  

**Cross-capability work is fine as a program epic;** **each PR remains one concern** (one class dominant, or one explicit stage of a mixed sequence).

---

## 4. PR shaping rules

| Rule | Detail |
|------|--------|
| One concern per PR | Dominant class = move \| refactor \| rewrite \| delete \| rebrand |
| Isolated worktree | From current `origin/dev`; rebase when trunk moves |
| ADR re-query gate | Before open/push: §1.0 check + amended 0562 reading |
| Face policy | No hand-edit `*.generated.json`; no re-track de-committed faces |
| Consumers | All live consumers updated or proven absent before delete |
| Dual-critic + CI | Agent dual-critic + `oya-ci-required` for merge (human APPROVE not mandatory) |
| Close bead | After squash merge (+ R3 packet if product-complete claim) |

---

## 5. How cards should be authored

Each reorg card (backlog or bead) should state:

1. **class:** move | refactor | rewrite | delete | rebrand | mixed(stages)  
2. **capability span:** single | multi (list capabilities)  
3. **authority:** Accepted ADRs re-queried on `origin/dev` (+ plan SHA if any)  
4. **owned paths / non-goals**  
5. **blast radius / rollback**  
6. **verify:** focused tests or consumer-absence proof  

---

## 6. Relation to CAS / k8s / product

- CAS 3A–3C moves are **move-class** under CAS plan + Accepted placement ADRs  
- K8s Go→Rust **port** is rewrite/generate under **0637/0638** (W0+), not a generic reorg move-plan  
- Portfolio intelligence/libs/tools cards may be **multi-capability** and **refactor/rewrite/delete**, not only rehome  

---

## 7. Beads

- Process: `oyatie-oso.18` (ADR re-query + this doctrine)  
- Do not collapse all reorg into “run move-plan”  
- Spent/stale plans → close or supersede; do not re-execute  
