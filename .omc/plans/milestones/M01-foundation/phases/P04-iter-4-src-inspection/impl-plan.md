---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P04-iter-4-src-inspection
impl_plan_id: IP-001-iter-4-src-inspection
status: pending
owner: council-architecture
blocked_by:
- impl_plan: P02/IP-001-shard-1-atomic-rename
  reason: src-inspection runs on renamed crate dirs; requires P02 complete
acceptance_lanes:
- cargo-check
purpose: Auto-backfilled purpose for impl-plan.md
---
# IP-001-iter-4-src-inspection: Resolve 88 STUB-pending layer-evidence cells

## Intent

Inspects `src/` of each crate with `STUB-pending-iter-4-src-inspection` in its
`layer_evidence` cell; fills the cell with a concrete `file:line — <pattern>`
cite; confirms or amends the layer assignment; provides protocol-classification
evidence for the 26 PROTOCOL-UNKNOWN rows (which gates P03).

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `docs/plans/rename-plan-v4-clean-arch-2026-05-13.md` §3.1–§3.5 | update | Replace all 88 STUB cells + 26 PROTOCOL-UNKNOWN cells with evidence cites |
| Per reclassified crate `Cargo.toml` | update (if any kernel→domain flip) | Package name suffix change if layer amendment discovered |

---

## Crate Naming

No new crates. If a kernel→domain reclassification changes a layer suffix, the
renaming follows the same BNF justification pattern as P02.

---

## Code Shape

Evidence format for each resolved cell:
```
crates/<new-name>/src/lib.rs:<line> — <pattern that fixes layer>
```

Examples:
```
# kernel confirmation (pure types + ports):
crates/oya-data-boundary-kernel/src/lib.rs:1 — zero fn bodies; only struct/enum/trait declarations; no impl logic

# domain confirmation (business logic):
crates/oya-tenancy-domain/src/lib.rs:42 — fn validate_tier(tier: &Tier) -> Result<()> { ... }

# application confirmation (use-case orchestrator):
crates/oya-dsr-application/src/lib.rs:15 — struct CreateDsrUseCase<R: DsrRepository> { ... }

# protocol rest:
crates/oya-foundry-policy-api/src/main.rs:8 — Router::new().route("/evaluate", post(evaluate_handler))

# protocol grpc:
crates/oya-foundry-rag-api/src/main.rs:12 — Server::builder().add_service(RagServiceServer::new(svc))
```

---

## Acceptance Gates

```bash
# Zero STUB markers in audit body
grep -c "STUB-pending" \
  docs/plans/rename-plan-v4-clean-arch-2026-05-13.md   # must be 0

# 26 PROTOCOL-UNKNOWN markers still present (correct — gates P03)
grep -c "PROTOCOL-UNKNOWN" \
  docs/plans/rename-plan-v4-clean-arch-2026-05-13.md   # must be >= 26

# Workspace still compiles (no reclassification broke anything)
rtk cargo check --workspace --all-features             # exit 0
```

---

## Test Plan

Doc-update phase. Acceptance criterion is the grep counts above.

---

## Clean Architecture Compliance

Any kernel→domain reclassification is noted and tracked. The reclassification
does not break compilation (kernel is a subset of domain's allowed deps). The
`oya-check-architecture -- layer-correctness` subcommand will validate all
layer assignments once fully implemented in M02.

---

## Load Test

Not applicable.

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent iter-4-inspector \
  --intent "iter-4 src-inspection: fill 88 STUB layer_evidence cells + 26 protocol-classification cells" \
  --ttl 3600 \
  docs/plans/rename-plan-v4-clean-arch-2026-05-13.md::Section3
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-001-iter-4-src-inspection merged. 88 STUB cells resolved with file:line cites. 26 PROTOCOL-UNKNOWN cells have protocol classification evidence (gates P03). N kernel→domain reclassifications found: <list>. oya-data-boundary-kernel confirmed kernel (pure types+ports, ~95 consumers). Rename plan §3 audit body now accurate." \
  -i high \
  -k "M01,P04,IP-001,iter-4,src-inspection,88-stubs,layer-evidence"
```

---

## Halt Conditions

1. A crate `src/` is empty or missing — mark `layer_evidence = EMPTY-SRC-scaffold-only`; not a failure (scaffold-empty crates are expected).
2. A crate shows mixed-layer patterns (e.g. kernel + business logic) — flag for split; document in §3 as Exception with rationale; do not mask.
3. Protocol classification is genuinely ambiguous after inspection — document multi-protocol candidate; gates P03 split decision.

---

## Next IP Pointer

`../P03-shard-1-5-protocol-unknown-deferred/impl-plan.md` (now unblocked)
`../P05-post-cutover-hardening/impl-plan.md` (can run in parallel after P03)

---

## Cross-References

- Phase spec: `phase-spec.md`
- ADR-0056 §2.2.4: canonical decision tree
- Rename plan §3.1–§3.5: STUB row locations
