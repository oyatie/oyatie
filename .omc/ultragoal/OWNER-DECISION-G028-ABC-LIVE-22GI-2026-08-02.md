# Owner decision — G028 live 22Gi path = class B — 2026-08-02

State: **CLASS B SELECTED BY PRIOR FOUNDER RULING — V1 REVIEW `REQUEST_CHANGES` — V2 FACT COLLECTION IN PROGRESS — NO LIVE MUTATION**  
Tip: `origin/dev` `0c1014b87f0d881a821faa6a872b309deba0cfbf` (declared ARC request `22Gi`)  
Live: ARS/ERS request still `20Gi` on `admin@oya-talos` (no Argo/CAPI/Flux)

## Decision authority

Founder ruling dated 2026-07-29: treat the laptop Talos cluster as a **permanent** part of the cluster with merge authority; make its operation declarative rather than preserving the CLI-installed Helm state. No later contrary ruling was found in the durable ultragoal record.

That resolves the packet's conditional mechanically:

```text
KEEP_CURRENT_LAB=true → class=B
```

Accountability row:

```text
founder-ruling-2026-07-29 | class=B | KEEP_CURRENT_LAB=true | protected admitted immutable Argo bootstrap bundle | platform/CI owner | n/a | APPROVE_CLASS_SELECTION_ONLY | 2026-08-02 | durable-founder-ruling
```

This is **not** authorization to apply anything live. Independent review of v1 returned `REQUEST_CHANGES`; the staged ARC-only v2 remains a draft with unresolved principals, pins, inventory, and protected fan-in. Implementation still requires independent design `APPROVE` on the frozen v2 bytes, exact-head code `APPROVE`, protected `oya-ci-required` green, and a separately authorized bootstrap actor at the live boundary.

```text
owner A|B|C + independent design APPROVE
→ admitted reconciler / bootstrap
→ live ARS/ERS/new pod = 22Gi
→ #1526 cold FULL
→ #1523 restack push
→ G023 deletion / W0-C/D
```

Nothing downstream is authorized while live request remains 20Gi.

## Measured facts (not optional)

| Fact | Value |
|---|---|
| #1529 | MERGED; tip declares request `22Gi`, limit `60Gi`, maxRunners 3 |
| live Helm | `oya-arm64` rev 12 (2026-07-30) still 20Gi — sole observed apply path |
| Argo | namespace/controllers/CRDs/Applications **absent** |
| CAPI | Cluster/CRS CRDs **absent**; nodes lack `cluster.x-k8s.io/cluster-name` |
| `cells: []` | cannot adopt running lab; adding a row provisions a **new** cell |
| CRS | management→workload only; not an in-place lab adopt tool |
| tip FULL | run `30767156146` FAILED on `//oya:corpus-yaml-facts` no-exit-code; runner survived |
| OOM/DiskPressure/eviction | **not proven** on tip FULL; 22Gi necessary ≠ sufficient |
| #1526 | open corpus shard repair; rerun unauthorized at 20Gi |
| #1523 | restack unpushed until live 22Gi + #1526 path healthy |
| #1524 | draft DO-NOT-MERGE; preserve only |
| manual helm / CRS / render.sh | **not** authorized apply authority |

## Choose exactly one

### A — CAPI replacement / migration

Use only if the permanent FULL substrate will be a **new** CAPI-managed workload cell (not in-place adopt of `admin@oya-talos`).

Requires before implementation:

```text
owner | KEEP_CURRENT_LAB=false | class=A | management_cluster_identity | workload_cells_row | REPLACE|MIGRATE | continuity | rollback_owner | APPROVE
```

Packet: `G028-GITOPS-BOOTSTRAP-GAP-2026-08-02.md` (class A = replacement-only after REQUEST_CHANGES).

### B — permanent-lab non-CAPI Argo bootstrap (recommended if lab stays)

Use only if `admin@oya-talos` remains the permanent FULL CI cell.

Requires before implementation:

```text
owner | KEEP_CURRENT_LAB=true | class=B | bootstrap_authority | rollback_owner | APPROVE
```

Packet: `G028-CLASS-B-PERMANENT-LAB-GITOPS-DESIGN-2026-08-02.md`.

Hard rules inside B:

- reuse root-app / app-of-apps / admitted 22Gi values;
- hermetic Buck2 render + RED/GREEN fixtures;
- one-time apply of **admitted immutable Argo bootstrap bundle only**;
- never direct `helm upgrade oya-arm64`, CRS against non-CAPI lab, or scratchpad values.

### C — KEEP_INERT + alternate FULL substrate

Use only if founder accepts that this lab will **not** converge 22Gi and names a different FULL-running substrate.

Requires:

```text
owner | KEEP_INERT=true | class=C | alternate_full_substrate | admission_path | APPROVE
```

#1526/#1523 remain blocked on this lab until that alternate is live and admitted.

## Independent design APPROVE gate

Owner signature alone is insufficient. Exact chosen packet head needs a real independent design APPROVE.

Transport failures (`encrypted_content` decrypt 400) are **FAILED_TRANSPORT**, never APPROVE.

## Coordinator non-actions until the row exists

- No cluster mutation
- No helm / CRS / render.sh apply
- No #1526/#1528 rerun
- No #1523 push
- No #1524 mutation
- No canonical dirty-checkout mutation
- No G036 activation / G037 hatch edit / G030 delete / G026 move-plan JSON
- No weakening request below 22Gi

## Response format (copy one row)

```text
owner | class=A|B|C | KEEP_CURRENT_LAB=true|false|n/a | bootstrap_or_management_authority | rollback_owner | alternate_substrate_if_C | APPROVE|REJECT | date | signature
```
