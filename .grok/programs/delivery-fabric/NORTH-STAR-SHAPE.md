# North-star monorepo shape (hyperscaler clean architecture)

**Not merge authority.** Aligns to live ADRs (apex **ADR-0700/0701** lineage: 0515, 0131/0512, 0562/0615, 0613–0616, 0619) + REORG-DOCTRINE.

## Target topology

| Zone | Role | New debt allowed? |
|------|------|-------------------|
| `cloud/<capability>/` | Cloud-native services (flat single-concern) | **Only** real capability work behind stable seams — **no** process tooling, no dual-homes, no “temp” crates |
| `oya/<capability>/` | Product capability homes (post-reorg) | Same — **no** new dual-home under `oya/*` while disposition open |
| `intelligence/`, `iam/`, `billing/`, … | Registered capability trees | Same |
| `libs/` | Legacy flat crates | **No new crates** — disposition/move/delete only |
| `tools/`, `microservices/` | Legacy / removal-candidate | **No new surface** — retire or rehome per plan |
| `infra/*` | Transitional deploy/substrate | **No new process scripts** (preflight lives in `.grok/bin`); reorg moves only per plan |
| `ci/facade/`, `cloud/cloud-ci/` | CI gates / admission | Gate fixes only; no hand-edit `*.generated.json` |
| `specs/reorg/` | Move recipes only | Plans + derived manifests (never re-track derived faces) |
| `.grok/` | **Process kit only** (mm-*, fabric, workflows) | Yes — this is the home for delivery automation |
| `docs/decisions/` | Live apex ADRs + disposition | Live law only; archive is non-authority |

## Hyperscaler clean-architecture rules (must)

1. **Single-concern flat services** (ADR-0131 / no-grouping ADR-0132) — no new bundle µservices  
2. **Capability boundary** (0562+0615+0635) — one home per capability; dual-home is debt to **remove**, not extend  
3. **Inward deps** — domain/kernel not depending on adapters outward  
4. **Cells / blast radius** — changes bounded per PR  
5. **Generated faces** — materialize only; never hand-edit `*.generated.json`  
6. **Admission** — `oya-ci-required` sole merge context; kit never claims merge authority  
7. **Brand** — no gjc/omc/omx/hermes as live authority (0619)  
8. **origin/dev re-query** — every reorg PR from current tip  

## Anti-patterns (forbid)

| Antipattern | Why |
|-------------|-----|
| New crates under `libs/`, `tools/`, `microservices/` | Grows debt in reorg targets |
| New “temporary” dual-home `oya/X` + `cloud/X` | Violates north-star single home |
| Process scripts under `infra/*` | Reorg target; use `.grok/bin` |
| Feature work smuggled into reorg PR | One concern per PR |
| Hand-edited generated faces | Admission false-green |
| Mega-PR: runners+CAS+k8s+reorg | Blast radius / thrash |
| Second concurrent move-plan | ADR-0614 singleton |
| “Helpful” files in `cloud/*`/`oya/*` from fabric agents for process | **Debt in reorg target** |

## What fabric agents may write

| Intent | Allowed paths |
|--------|----------------|
| Process / mm-* / fabric | **`.grok/**` only** (and board under `.grok/programs/delivery-fabric/`) |
| ADR disposition | `docs/decisions/`, `docs/adr-archive/` (historical) |
| CI gate fix for open PR | Only paths that PR already owns; prefer not expanding into new reorg-target trees |
| Reorg **execute** | Paths listed on the card **to move/delete/refactor down** — net **reduction** of dual-homes / legacy; plan-only under `specs/reorg/` first when unsure |
| Product feature | Only registered capability home after re-query — **not** while claiming reorg lane |

## Net-debt rule (fail closed)

```
net_path_debt = new_surfaces_in_reorg_targets - removed_or_rehomed_surfaces
```

For any lane with `source=reorg` or paths intersecting reorg targets:

- **`net_path_debt` must be ≤ 0** (except plan-only JSON under `specs/reorg/` and OWNERS required by policy)  
- New test/docs that **only** enable deletion/rehome are OK  
- New product behavior in reorg targets requires a **non-reorg** product lane and north-star home  

## Reorg target prefixes (debt-sensitive)

```
cloud/  oya/  infra/  libs/  tools/  microservices/  marketplace/  (legacy tails)
```

Treat as **execute-to-reduce** only. Process automation stays in **`.grok/`**.
