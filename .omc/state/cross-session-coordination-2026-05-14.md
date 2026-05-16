---
doc_class: CrossSessionCoordination
shape: anchor
status: open
date: 2026-05-14
author_session: 7e0309c2-f5ac-4d3f-846f-85c2292dd8b6 (ops-Wave-1-realignment)
target_session: <parallel-session WIP — M02b-substrate authoring sweep>
authority_chain: docs/MASTERPLAN.md → docs/decisions/ADR-0063 → .omc/plans/consensus-masterplan-2026-05-13.md → this note
companion_docs:
  - .omc/plans/ralplan-ops-freelance-realignment-2026-05-14.md (this session's realignment plan)
  - .omc/state/sessions/7e0309c2-f5ac-4d3f-846f-85c2292dd8b6/investigation-synthesis.md (gap analysis)
---

# Cross-Session Blocker — 297 declared workspace members missing on disk

## §1 The blocker

`Cargo.toml [workspace.members]` declares **439** entries; only **172** crate dirs exist under `crates/` at commit `4d6bf91` (1 of those is `.DS_Store` so the real count is 171). The set delta:

| Set | Count |
|---|---|
| Declared in `[workspace.members]` | 439 |
| On disk in `crates/` (excluding `.DS_Store`) | 171 |
| **MISSING** (declared, not on disk) | **297** |
| **ORPHAN** (on disk, not declared) | **29** |

`cargo build`, `cargo check`, `cargo metadata --no-deps`, `cargo test` ALL fail until this set is reconciled. Any new code authoring (ops Wave 1 substrate, ADR-0093 DSL kernel/runtime, anything) is blocked behind workspace-resolution recovery.

Full lists frozen for this hand-off:
- `/tmp/missing-crates.txt` (297 lines) — committed-by-reference; rerun the python script below to regenerate from HEAD if drifted.
- `/tmp/orphan-crates.txt` (29 lines) — same.

## §2 Distribution (top clusters)

297 missing crates cluster by 3-token prefix:

```
  15  oya-audit-chain-*
   8  oya-capability-registry-*
   8  oya-data-boundary-*
   8  oya-workflow-engine-*
   6  oya-application-product-*
   6  oya-policy-engine-*
   6  oya-records-fhir-*
   5×7  oya-ontology-{action,agent,audit,entity,function,link,pillar}-*   (= 35 total)
   4×8  oya-identity-{employees,employments,mfa,organizations,passkeys,persons,sessions,users}-*   (= 32 total)
   4×3  oya-eventing-{outbox,subscriptions,topics}-*   (= 12 total)
   4    oya-cloud-tenancy-*
   5    oya-cloud-{compute,network,storage}-* (each)
   4    oya-kms-envelope-*
   …rest scattered…
```

These prefixes mirror the M02b-substrate µservices catalog (audit-chain / identity / eventing / ontology / data-boundary / capability-registry / policy / cloud / kms / records / workflow-engine). The pattern says: this is **M02 substrate authoring sweep WIP**, not random rot.

## §3 Resolution choices (target session decides; ops realignment session declines to choose)

For each of the 297 entries, exactly one of the following lands:

1. **Author** — create `crates/<name>/Cargo.toml` + `src/lib.rs` substrate scaffold (kernel/domain/application/adapter/rest per the BC's BNF v4.1 layer enum). Use whichever scaffold tool the parallel session was running (likely `oya-dev-cli gate emit crate-scaffold` or `xtask-metadata-augment`).
2. **Remove** — delete the line from `Cargo.toml [workspace.members]` if the crate is no longer planned (e.g., superseded by a different BC split, retired per `feedback_autonomous_implementation_artifacts.md` stale-removal).

Mixed strategy is fine and likely the right shape: author the M02-P12..P18 substrate clusters (audit-chain, identity, eventing, ontology, data-boundary, capability-registry, policy, kms, records) and remove the stale rows that the masterplan §Follow-ups item #4 directive ("Remove legacy milestone dirs … per ADR-0063 §7 'stale removed in reality'") covers.

## §4 What the ops-Wave-1 session is doing in parallel (FYI)

While this blocker is open, the ops-Wave-1-realignment session ships work that does NOT depend on workspace resolution:

1. Authoring `.omc/plans/ralplan-ops-freelance-realignment-2026-05-14.md` — pure planning doc; reclassifies the 20 commits from session `7e0309c2`.
2. Amending `docs/decisions/ADR-0090` frontmatter Status → "Superseded by ADR-0091" — markdown edit; no compile dependency.
3. Editing `Cargo.toml [workspace.metadata.oya.microservices.ops]` — TOML-only edit to refine `bounded_contexts` (drop `workspace`; keep `docs`) per the realignment plan. This edit lands as the masterplan-defined M02-P19 IP-X1 catalog-registration delta, but DOES NOT add or rename any workspace members, so it doesn't depend on workspace resolution.

The actual ops Wave 1 substrate (24 docs crates + 16 extractors + watch daemon + 13 Leptos pages + 4 CI lane binaries + 4 Cedar fragments) is **deferred** until this blocker clears, per docs sub-plan §6(g) dispatch sequence (M02-P19 catalog → M02-P20 minimum Cedar + 5 G1 hot extractors + lean-a8 scaffold → … → M03-P04 IP-X1 substrate crates).

## §5 Regeneration command (if HEAD has moved)

```bash
python3 -c "
import re, os
declared = set()
in_members = False
for line in open('Cargo.toml'):
    s = line.strip()
    if s.startswith('members'): in_members=True
    if in_members:
        m = re.search(r'\"crates/([^\"]+)\"', s)
        if m: declared.add(m.group(1))
    if in_members and s==']': in_members=False
ondisk = {d for d in os.listdir('crates') if not d.startswith('.')}
print(f'declared={len(declared)} ondisk={len(ondisk)}')
print(f'missing={len(declared - ondisk)} orphan={len(ondisk - declared)}')
"
```

## §6 When to close

Close this note when `cargo metadata --no-deps` exits 0 on `main`. At that point the ops Wave 1 substrate dispatch (M02-P20 onwards) can begin per docs sub-plan §6(g).
