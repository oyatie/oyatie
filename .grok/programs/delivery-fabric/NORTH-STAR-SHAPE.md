# North-star monorepo shape (first principles + hyperscaler patterns)

**Not merge authority.** Live ADRs (apex lineage **0700/0701**: 0515, 0131/0512, 0562/0615/0635, 0613–0616, 0619) + REORG-DOCTRINE override this file on conflict.  
**Do not accept the current tree as the target.** The tree is a migration state; this file is the attractor.

---

## 0. First principles (challenge defaults)

| Principle | Meaning | Rejects |
|-----------|---------|---------|
| **One home per concern** | A capability has exactly one durable root for crates + runtime artifacts | `oya/X` + `cloud/cloud-X` + top-level `X/` as permanent peers |
| **Registry is inventory of debt, not destiny** | `absorbs_current_dirs` lists **where code still lives**, not where it should stay | Treating multi-absorb as “supported multi-home architecture” |
| **Platform process ≠ product surface** | Delivery automation, agent kits, preflight live under `.grok/` | Process scripts/crates under `infra/`, `tools/`, `cloud/*` |
| **Admission is singular** | One required context: `oya-ci-required` | Parallel “source of truth” CI / bespoke merge ratchets |
| **Blast radius is a feature** | PR = one dominant class + owned path set | Mega-PR (CAS + k8s + reorg + runners) |
| **Generated is never hand truth** | Faces materialize from producers | Hand-edited `*.generated.json` |
| **Net debt ≤ 0 in migration zones** | Reorg executes only reduce dual-homes / legacy | “Helpful” new scaffolds in `oya/` / `cloud/` during reorg |
| **Constant work > input-dependent thrash** | Affected-set, single babysit owner, no tip thrash | Multi-poll CI; dual-critic pin storms |
| **Live ADR resolution** | `status: Accepted` is not sufficient alone | Stale Accepted without supersession walk |
| **Empty board ≠ done** | Quiet queue with dual-homes present is **false green** | W2 `empty-board` while multi-absorb paths still exist |

---

## 1. Target topology (attractor)

| Zone | Role | New surface? |
|------|------|----------------|
| **Registered capability root** (`intelligence/`, `marketplace/`, `compliance/`, `iam/`, `billing/`, `k8s/`, `storage/`, …) | **Sole durable home** for that capability’s crates + co-located contracts/IaC/SLOs | Only real product work |
| `ci/facade/` | Productized admission gates (post–cloud-ci rehome) | Gate fixes only |
| `specs/` | Machine law / plans (move recipes under `specs/reorg/`) | Plans yes; no hand-tracked derived faces |
| `.grok/` | **Only** agent/process kit | Yes |
| `docs/decisions/` | Live apex ADRs | Disposition yes; no cosplay Accepted |
| `libs/`, `tools/`, `microservices/`, residual `oya/*`, residual `cloud/cloud-*` | **Migration debt** | **No new crates/surfaces** — only reduce |

**Transitional allowed:** a second path may exist **while** a disposition lane is open; every multi-absorb entry is a **burn-down item**, not a pattern to copy.

---

## 2. Hyperscaler monorepo patterns (must)

Patterns drawn from large single-repo platforms (ownership, hermetic CI, layering, cell isolation)—mapped to Oyatie ADRs:

1. **Strict ownership** — every path has a temporal owner on the PR/board; no drive-by multi-tree edits  
2. **Directed dependency** — domain/kernel do not import adapters; adapters depend inward  
3. **Single-concern services** — ADR-0131/0132; no bundle µservices  
4. **Capability graph** — closed registry (0562+0615+0635); one `dag_node` home  
5. **Hermetic / reproducible CI** — Buck/Cargo gates; materialize faces out-of-band  
6. **Affected / constant-work admission** — binding cone; FULL only when structural; sole `oya-ci-required`  
7. **Cell / blast-radius isolation** — changes fail in one cell/capability without repo-wide thrash  
8. **Platform vs product split** — process automation in `.grok/`, not product trees  
9. **Deprecation is explicit** — PARKED/BLOCKED plans, retirement markers; no silent dual-run forever  
10. **Audit is adversarial** — W5 challenges board/backlog/tree; does not rubber-stamp heartbeats  

---

## 3. Anti-patterns (forbid)

| Antipattern | Why it fails first principles |
|-------------|-------------------------------|
| Permanent dual-home | Two truths; ownership and deps diverge |
| New `libs/*` / `tools/*` / `microservices/*` crates | Debt growth in zones marked for reduction |
| Process tooling under `infra/*` or `cloud/*` | Contaminates product/substrate trees |
| Hand-edited generated faces | False green admission |
| Second concurrent live `*-move-plan.json` | Singleton (0614); thrash + merge fights |
| Mega-PR | Blast radius / review theater |
| Fabric writing “helpers” into `oya/`/`cloud/` | Process debt in reorg targets |
| Multi-worker CI poll / tip thrash | Constant-work violation; cancels binding runs |
| “W2 empty-board” with multi-absorb debt | False complete |
| Treating `absorbs_current_dirs` as multi-home design | Confuses inventory with architecture |

---

## 4. What agents may write

| Intent | Allowed paths |
|--------|----------------|
| Process / fabric / mm-* | **`.grok/**` only** |
| ADR disposition | `docs/decisions/` (+ archive as historical only) |
| CI fix for open PR | Paths that PR already owns; no new reorg-target surface |
| Reorg **execute** | Listed reduce paths only; **net_path_debt ≤ 0** |
| Product feature | **One** registered capability home after tip ADR re-query — not under a reorg claim |

---

## 5. Net-debt rule (fail closed)

```
net_path_debt = new_surfaces_in_debt_zones - removed_or_rehomed_surfaces
```

Debt zones (default): `oya/`, `cloud/`, `libs/`, `tools/`, `microservices/`, and residual duals still listed in registry absorbs.

- Reorg lanes: **net_path_debt ≤ 0**  
- Plan-only under `specs/reorg/` does not count as product debt  
- Tests that only enable delete/rehome are OK  

---

## 6. Lane cut (AREA × CLASS × SLICE)

See `REORG-LANE-TAXONOMY.md`. Examples that must stay separate:

- **REWRITE** k8s-port Go→Rust ≠ **MOVE** intel remainder ≠ **DELETE** dual-home residual  
- **INFRA** leaf delete ≠ **CAS** ordered move  
- **cloud/** and **oya/** are epics of leaves, not one board card each  

---

## 7. Fabric roles (do not blur)

| Class | Job | Not job |
|-------|-----|---------|
| W1 Portfolio | Discover / prioritize | Implement, babysit |
| W2 Implement | Path-disjoint PR | Multi-poll CI |
| W3 Babysit | Sole merge-on-green owner | Invent scope |
| W4 Productivity | Heartbeat liveness of W1–W3 | Declare north-star complete |
| W5 North-star audit | **Adversarial** gap vs this file + registry + board | Rubber-stamp empty-board |

W4 green + dual-homes remaining ⇒ **not** healthy. W5 must say so.
