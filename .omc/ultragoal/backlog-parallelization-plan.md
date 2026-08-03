# Backlog parallelization plan (2026-06-20, triage workflow wmzh3q0rx)

Leader-state coordination memory. Anchor: dev @ 31aa56ec6 (after move-21 billing #766). Founder directive: "parallelize whatever can be parallelized; look at the entire backlog."

## Hard constraints
- Capability MOVES are strictly serial (shared registry/membership/acyclicity/firewall baselines + reindeer/third-party). One in flight: **move-22a intelligence sub-batch (a)** (PR #767, in test-wiring fix).
- Merges SERIALIZE through the firewall queue (build-parallel, merge-serial). Cap concurrent executors ~4-6.
- DO NOT touch outside-move-protocol: specs/capability-registry.json, specs/reorg/*, reindeer/third-party/**, gate-catalog index, registry/catalog/* (except a PR's own new crates).

## LAUNCHED NOW (parallel, disjoint subtrees, zero reorg conflict)
1. **Codemod hardening** (#61 single-token round-trip, #65 >1-plan hard-error, #63 sandbox-label literals) — p34-codemod / agent a2bde5727852da711. tools/oya-reorg-codemod-app only; serialized within (shared tree).
2. **AUTH-001 boot identity service** (#88) — p35-auth001 / agent aea6ce8cfead393dd. iam/facade/identity-service (post-move-18 location).
3. **AUTH-005 tenancy delivery** (#89) — p36-tenancy / agent a675afa23647b45fe. tenancy/ (post-move-14 location).
+ **Reorg serial lane** (not counted): move-22a fix / agent a706356334d0ddb32.

## WAVE 2 (after AUTH-001 lands a live API contract)
AUTH-002 webauthn-relying-party-rest · AUTH-006 bearer-auth middleware + /token,/refresh,/logout · AUTH-003 assertion→session mint · AUTH-007 login/signup/passkey UI + auth-gate (oya/application shell).

## BATCH-2 (serial / after reorg or after a batch-1 item)
- AUTH-004 Postgres/Valkey identity stores (after AUTH-002/006); AUTH-008 E2E journey suite (after Lane A).
- Workflow/freshness-gate cluster: #67 → #48 (same freshness-app), #74 (workflow YAML), #58 (target-parity) — serialize (gate-catalog/firewall registration races).
- Codemod follow-on #76 (custom-bin de-brand) after the Lane-B codemod queue.
- #75 (ADR prose-vs-move) shares cross-artifact-agreement-app with #84.

## STRICTLY AFTER intel-a / reorg (touch reorg-shared gate state or in-flight files)
Gate-engine cluster: **#81** (intra-cap face + cross-cap S-rank lint — the acyclicity blind-zone fix; bake into move-protocol before move-22b ideally), **#84** (rename-aware ADR-justification-source relabel), #87 (test-wiring generator per-file gaps), #77 (graph-invisible-test ratchet), #71 (catalog→affected-set inputs), #70 (catalog completeness 138), #72 (de-brand validator sweep), #78 (catalog internal crate: field), #79 (SLO globs). Reindeer/third-party: #44 (reindeer-parity gate), #46 (friction-terminal). Pre-existing dev breakages: #31 (FRIC-1781310100/400/500).

## REORG REMAINING (serial track)
move-22a (in-flight) → 22b provider-adapters(~14) → 22c account/supervisor/runtime(~18) → 22d feature-stacks(~36) → 22e dashboard/api/rag(~30) → 22f fitness/dev-cli(~8) → 22g collab/document-format(~6, registry-granularity fork: collab/docs cap? shuffle-sharding→cell?) → phase-2 #62 (non-crate residue).

## G001-G013 STATUS (founder stop-condition)
- G001 Contract Lock + Authority — **DONE** (#642)
- G002 Trust Substrate (KMS/Secrets/WID) — PARTIAL (1a/1b-i merged; gap: live mTLS PEP 1b-ii = FRIC-1781490000, in-flight #38)
- G003 Persistence (oya-data) — PENDING (pre-work only; gap: owned Rust SQL + CRDB multi-Raft + RLS + envelope)
- G004 Cedar PDP + Policy Store — PARTIAL (#717; gap: PDP caller-auth [blocked on G002] + policy-store CP + RBAC/ABAC/PBAC suites)
- G005 IdP to Production — PARTIAL (slices 1-3; gap: passkey/WebAuthn live path [AUTH-001/002/003] + E2E)
- G006 Tenancy + RBAC Core (FD-001) — PARTIAL (#647; gap: provision→credential→authorize→deny→audit E2E [AUTH-005])
- G007 Unified Shell (Leptos) — PARTIAL (#652; gap: prod crate + kill mock catalog + token brokerage [AUTH-007])
- G008 Observability + Audit — PARTIAL (#648; gap: collector binary + OpenSLO codegen + signed audit-chain)
- G009 Messaging + Metering + Billing — PARTIAL (#650; gap: owned Rust Pulsar client + outbox + double-entry subledger)
- G010 Dogfood + K8s Bring-up — PENDING (fan-in; needs G3-G9)
- G011 Pipeline-as-Product Ratchet — IN-FLIGHT (finish intel-a sub-batches + acyclicity blind-zone fix #81)
- G012 Sibling/Kernel Consolidation — PENDING (serial after G011; kernel/os → cloud-kernel/cloud-os)
- G013 Final Quality Gate — PENDING (fan-in triple-approval over G01-G12; harness proven #688)

**Biggest single gap:** live mTLS PDP path (G002 1b-ii) — chokes G004/G006/AC-W-13 (in-flight #38). Then: nothing E2E-operable until identity service boots (AUTH-001, launched).
