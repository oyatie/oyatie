# Ultragoal checkpoint — 2026-06-10 (refreshed, verified non-stale)

> **Manifest of all durable records: `.omc/ultragoal/INDEX.md` · Resume: `.omc/ultragoal/RESUME-PROMPT.md`**

**Session:** FD-001 Enterprise SaaS ultragoal (first vertical + unified shell + full cloud substrate, dogfooded).
**Verified ground truth at checkpoint:** `dev` @ `15de7815a` (local synced). Base session start `8acec8920`.
**Merged this session: 23 PRs.** **Friction ledger: 51 rows.** **Session memories: 21.**

## Durable authority (read first next session)
- `.omc/ultragoal/RESUME-PROMPT.md` — START HERE.
- `.omc/ultragoal/INDEX.md` — manifest of every durable record.
- `.omc/ultragoal/brief.md` — mission + AMENDMENTS 1–13 + WIND-DOWN (binding founder directives).
- `.omc/ultragoal/friction-ledger.jsonl` — 51 frictions → the G011 enforcement backlog (each has its fix).
- `.omc/ultragoal/RECOMMENDATION-corpus-liveness-graph.md` — fundamental decay/drift/staleness fix (all granularities + conformance classes).
- `.omc/ultragoal/goals.json` + `ledger.jsonl` — 13-story plan (G001 complete, G002 active pointer).
- `.omc/research/` — 4 source-grounded corpora. Session memories index: `MEMORY.md` (21 entries).

## FD-001 substrate — LANDED on dev (G02–G09 + G12 all merged)
- **G02 trust** (KMS enclave one-way-door, crypto-shred + attributable cancel, typed root-provenance, zero-static-secrets lease) — #655, #658.
- **G03 persistence** (oya-data SQL port + HLC, transactional outbox + CDC) — #645, #656.
- **G04 authz** — Cedar PDP, full **RBAC+ABAC+PBAC**, structural tenant-isolation forbid, zookie freshness — #649.
- **G06 tenancy** (lifecycle control plane, dir consolidation) — #647, #653.
- **G07 shell** (production Leptos portal + ADR-0393 supersession lint) — #652.
- **G08 audit** (audit-event kernel + awslc digest chain) — #648.
- **G09 messaging** (Pulsar surfaces + metering) — #650.
- **G12 consolidation** — kernel→cloud-kernel #640, os→cloud-os slices1-4 #636/#639/#641/#643, office #637, intelligence SDKs→cloud-intelligence #638.
- Foundation: G001 contract-lock + ADR-0536/0537 (Proposed) #642; docs rescue #635; hooks disposition #646; hook quoted-string fix #654; sqlx-Debug hotfix #657; **buck2 cache-key fix #659**.
Every PR adversarially reviewed (CI-green ≠ review-clean); several forced real fixes by review BLOCKs that CI had called green.

## Open PRs (refreshed 2026-06-10 ~17:20Z; #660 merged earlier today)
- **#670** G011 test-wiring generator + libs batch-1 (20 wired, baseline 634→614) — CI 21/21 green, CLEAN; adversarial review IN FLIGHT (review-pr670-r1.md when done).
- **#663** cloud-intelligence agent canary — ANOTHER SESSION'S lane (.omx runtime), XPROXY parity cluster; merge-DIRTY vs dev; HELD with the founder #644 ruling, this session does not touch it.
- **#651** G05 IdP — workload-identity core is plane-3-correct (lands as-is); rework = rescope its OIDC issuer behind IdentityIssuerPort. HELD on identity-architecture ratification.
- **#644** XPROXY — BLOCK: no authority chain, 7/9 cited tests don't exist. Founder sanction-or-close.
- Lanes in flight (no PR yet): checkout-guard (recovery worker after compaction-death, FRIC-1781110000) · lane-supervisor (FRIC-1781110000/FRIC-1781111000 automation).

## Held for founder (door:one-way)
1. **Identity architecture** — RESOLVED recommendation (option b, 3 planes: cloud-iam substrate + oya-identity ADR-0476 shared human IdP dogfooding cloud-iam + oya/identity workload plane). Ratify → merge #651 workload core, rescope issuer, author ADR-05xx. (memory: cloud-idp-vs-oya-product-identity)
2. **ADR-0536 / ADR-0537** — Proposed, await sign-off.
3. **Corpus Liveness Graph** — research precedents → ADR (decay/drift fundamental fix; scope = all granularities ADR→file→folder→symbol→line→token + classes liveness/reference/format/template/freshness/directive-compliance).
4. **FRIC-003** signing enforcement (required_signatures=false live) — PAUSE-AND-PAIR ruleset toggle.

## Known dev-state notes
- ~~dev locally red on 4 buck2 gate tests~~ **RESOLVED 2026-06-10**: verified Pass 4 / Fail 0 locally on dev @ 2705d1c96 (firewall/slo-coverage/registry-drift/generated-artifact-control-plane) — faces are fresh after #662/#664 regeneration and the ADR-0539 freshness gate now keeps them fresh. FRIC-009 local-materialization symptom cleared; always-green-dev restored locally.
- No live merge queue: manual train (rebase onto fresh dev + content-assert + green-on-rebased-head). `delete_branch_on_merge=true` set.

## Top G011 next-session order (each = a friction-ledger row with its fix)
1. ~~glob workspace-members + Cargo.lock merge-driver~~ **DONE 2026-06-10**: PR #661 (structural lock merge driver, 2 adversarial review rounds) + PR #662 (6 narrowed globs + oya-workspace-members-kernel canonical resolver + 5 parser migrations + workspace-glob-coverage born-blocking gate + ADR-0538; equivalence proven 816 members / delta = exactly the 2 new crates). dev @ 5aaa68ab4. FRIC-1781069288 RESOLVED-structurally.
2. ~~Pre-push freshness gate~~ **DONE 2026-06-10**: PR #664 (oya-cloud-ci-freshness-app: lock-freshness via members kernel + face-freshness via the exact CI producer binaries; dev-cli run-all lane + buck2-binary CI job; ADR-0539; diff-policy upgraded to precise byte-equality). dev @ 2705d1c96. FRIC-1781082000 RESOLVED; FRIC-1781062100 fixes 1-2 closed. APPROVE first pass, CI 18/18.
3. ~~hermetic-lane target completeness~~ **DONE 2026-06-10**: PR #665 (oya-cloud-ci-target-parity-app, ADR-0540): missing-buck born-blocking (debt 0) + unwired-test-code baseline-block-on-new (634 keys mechanically frozen, reviewer re-derived byte-exact). dev @ 2c097d181. FRIC-1781063357 resolved for NEW debt; **634-key BURN-DOWN = open follow-on** (mass rust_test BUCK wiring; expect latent uncompilable tests to surface — that is the point).
4. ~~Corpus Liveness Graph ADR~~ **DONE 2026-06-10**: PR #666 (ADR-0541 Proposed, precedent-grounded: research corpus at .omc/research/corpus-liveness-precedents-20260610.json, 25/25 claims verified; per-class enforcement posture honest to the three refutations). dev @ 0b03f15c0. FOUNDER RATIFICATION PENDING; D4 spike IP is the next executable step after ratification.
5. **buck2 NativeLink remote cache** (warm-by-default, fixes 0% cache) + cold-canary integrity job — NEEDS FOUNDER (cache hosting).
6. ~~CLAUDE.md citation fix~~ DONE (#667, dev @ ab732d87b). ~~settle-protocol automation~~ **DONE** (#668, dev @ a8797a4df: face-settle bin refuses dirty/untracked non-face changes, --settle/--commit faces-only; FRIC-1781100200 RESOLVED).
7. ~~enforcement-liveness (FRIC-012)~~ **DONE** (#669, dev @ 16f2e3b54: born-blocking gate — dual-wiring or stub-marker required for every tools/hooks/*.sh, mirror-drift + dangling refs fail closed; FRIC-012 RESOLVED-fully).
8. Remaining queue: 634-key target-parity burn-down (generator-first; lane = build the rust_test stanza generator + prove on one batch) · CI async owned-Rust parallelism (task #16 from prior session — scope needs re-derivation; #660 landed the safe quick wins).
9. Dispatch directive (founder 2026-06-10, AMENDED later same day): teammates + review agents via **Fable (Claude) subagents** — codex-exec dispatch superseded after process frictions (FRIC-1781110000 compaction death, FRIC-1781111000 stdin block, FRIC-1781113000 codex-env buck2 failures); in-flight codex lanes ran to completion only. Review agents load meta-skills + run `/oh-my-claudecode:ultraqa` with the rubric (Torvalds + hyperscaler lenses). AMENDMENT 14a: codex exec stays available as a SUPPLEMENTARY review lens (critical changes, dual-model consensus, extra insight) at leader discretion — Fable verdict is the verdict of record. Brief-file pattern unchanged; tmux-pane team runtime stays retired.

## Deferred (per wind-down)
Remaining substrate slices 2..N to FD-001 exit depth (K8s operators, slos, one-command bring-up, failure-injection, multi-arch OCI) · W2 owned AST core (bespoke rowan; reused by oyago/oyapy transpilers, tasks #9/#10) · G10 dogfood integration · G13 final quality gate.

---
## ADDENDUM — merge-conflict structural fix (still valid)
Root Cargo.toml has 791 EXPLICIT workspace members + Cargo.lock → every new-crate PR edits both shared files → concurrent lanes conflict 100% by construction. Fix: GLOB members ("libs/*","cloud/*/crates/*","oya/*/crates/*") so new crates need zero shared-file edit + a Cargo.lock git merge-driver + a glob-coverage CI gate. (Top G011 item.)

## ADDENDUM 2 — buck2 cache cold-vs-warm (still valid)
0% cache = no SHARED cache (.buckconfig has no [cache] section). Content-addressed → warm hit == bit-identical to cold, so WARM by default; COLD only for (1) release images, (2) cold-canary integrity job, (3) trust-boundary untrusted PRs. Landed interim #659 (stable cache key). Durable: NativeLink/CAS remote cache (G011/W3).
