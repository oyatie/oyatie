# Reorg lane taxonomy (AREA × CLASS × SLICE)

**Not merge authority.** Complements `REORG-DOCTRINE.md`, `NORTH-STAR-SHAPE.md` (first principles), `REORG-REBRAND-BACKLOG.md`.

**Do not accept multi-absorb registry rows as the architecture.** They are burn-down inventory. Lanes exist to **remove** dual-homes and legacy zones, not to stabilize them.

## Naming

```text
REORG-<AREA>-<CLASS>-<SLICE>
```

| AREA | Meaning |
|------|---------|
| FABRIC | Process/automation under `.grok/` only |
| OYA | Residual / dual-home under `oya/*` |
| CLOUD | Residual / dual-home under `cloud/*` |
| INTEL | Intelligence multi-home (often **move** + singleton) |
| INFRA | `infra/*` leaf work (non-CAS-ordered unless labeled CAS) |
| TOOLS | `tools/*` disposition |
| LIBS | `libs/*` disposition |
| K8S-PORT | Go→Rust **rewrite** epic (not a move-plan by default) |
| CAS | Ordered CAS rehome 3A→3B→3C |
| DOCS | Corpus classify / brand |

| CLASS | Move-plan? |
|-------|------------|
| MOVE | **Yes** when path bijection is the work |
| REFACTOR | No |
| REWRITE | No (unless also rehoming) |
| DELETE | No |
| REBRAND | No |
| MIXED | Per stage |

## Parallelism

- Path-disjoint leaves run in parallel.
- **Serial:** at most one live `specs/reorg/*-move-plan.json`.
- k8s-port **rewrite** slices ≠ dual-home **delete** ≠ CAS **move**.
- W2 implements; W3 is sole babysit (see `BABYSIT-SINGLE-FLIGHT.md`).

## Seed IDs (mm-reorg-enqueue)

Fabric / process: `RR-FABRIC-SINGLE-FLIGHT`  
Dual-home: `RR-DUAL-TOPO-{MARKETPLACE,COMPLIANCE,OBSERVABILITY,TASKS}`, `RR-INTEL-REMAINDER`  
Areas: `RR-INFRA-LEAF-DELETE`, `RR-TOOLS-LEAF-DISPOSITION`, `RR-CLOUD-SCAFFOLD-SWEEP`, `RR-LIBS-FLAT-DISPOSITION`  
Rewrite: `RR-K8S-PORT-REWRITE-W0B`  
Ordered: `RR-CAS-3A` (blocked on G039)
