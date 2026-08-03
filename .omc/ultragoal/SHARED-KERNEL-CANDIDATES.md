# Shared-kernel consolidation candidates (founder directive 2026-06-10: AST/parse logic reused and shared)

Audit method: targeted grep over gate/tool/check crates (non-test paths), 2026-06-10. Precedent
pattern: `libs/oya-workspace-members-kernel` — ONE oracle, all consumers import it, gate enforces
non-duplication (workspace-glob-coverage). Each candidate below should follow that shape: extract a
single-concern kernel under `libs/`, migrate consumers, then a lint/gate prevents re-duplication.
Long-term, parsing kernels become frontends of the W2 rowan-style AST core (ADR-0541 corpus graph).

| # | Class | Duplicated in (today) | Kernel candidate |
|---|---|---|---|
| 1 | BUCK manifest parsing | oya-buck-test-wiring-app (text heuristics, 2 review findings), accounting-registry producer, manifest-hygiene | `oya-buck-syntax-kernel` (task #10) |
| 2 | ADR front-matter/metadata parsing | accounting-registry main.rs, oya-governance-purpose-kernel, oya-governance-substance-bar, oya-check-honest-claims, oya-governance-adr-shape-kernel, oya-check-tenant-cost-labels-coverage | `oya-governance-adr-shape-kernel` already exists — promote it to THE oracle, migrate the other 5 |
| 3 | OWNERS / ownership resolution (nearest-ancestor walk) | oya-cloud-ci-total-accounting-app, accounting-registry (lib+main), oya-check-doc-catalog, oya-check-codeowners-mirror | `oya-ownership-resolution-kernel` |
| 4 | gate-baseline model + key-set diff | firewall-app, freshness-app, accounting-registry, leader inline python (FRIC-1781121000), future merge-base ratchet (FRIC-1781112000 / task #7) | `oya-gate-baseline-kernel` — task #7 builds the merge-base diff ON this kernel |
| 5 | append-only JSONL ledger read/append | accounting-registry, oya-check-honest-claims, oya-check-brand-residue, oya-governance-banned-primitives-kernel, claude-agent-sdk session store, lane-supervisor (new), audit-chain crate (oya/audit-chain — check overlap first) | `oya-jsonl-ledger-kernel` (audit-chain may already be the seed) |
| 6 | Hook-event (PreToolUse JSON) parsing | oya-checkout-guard-app is the FIRST Rust hook (#685) — no duplication yet | pre-empt: extract `oya-hook-event-kernel` when the SECOND Rust hook appears, not before |

Ordering recommendation: #4 first (task #7 needs it anyway — one lane delivers the laundering fix
AND the kernel), then #2 (existing crate promotion, cheap), then #1/#3, then #5. Each consolidation
lane ships with the non-duplication lint per enforcement-layering doctrine.
