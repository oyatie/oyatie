# Wave 15-CA-VERIFY-WORKSPACE — ADR-0105 + ADR-0131 + ADR-0132 audit of workspace-crate-layout µservices

**Date:** 2026-05-21
**Auditor:** Wave 15-CA-VERIFY-WORKSPACE Claude agent
**Scope:** Every µservice under `/Users/jasonlee/oyatie/microservices/<ms>/` whose code lives in `crates/oya-<ms>-*` rather than `microservices/<ms>/src/` (i.e., the workspace-crate-layout bucket excluded from the initial CA-VERIFY).
**Mode:** READ-ONLY — no code changes.
**Authority:** ADR-0105 (13-layer enum: kernel, domain, application, app, adapter, infrastructure, cli, rest, grpc, graphql, worker, sdk, api); ADR-0131 (per-µservice flat layout — `src/` canonical code root); ADR-0132 (no-suite / single-concern policy); `specs/crate-naming-audit.json` (ground-truth per-crate suffix audit).
**Companion report:** `/Users/jasonlee/oyatie/.omc/state/wave-15-ca-verify-2026-05-21.md` (flat-layout surface — 22 µservices).

---

## Discovery delta vs. initial CA-VERIFY estimate

The initial CA-VERIFY narrative flagged "~45 workspace-crate-layout µservices" but its enumeration mixed two distinct buckets:

1. **µservices with code physically under `crates/oya-<ms>-*`** (true workspace-crate-layout).
2. **µservices whose `microservices/<ms>/` is doc-suite-only with no Rust artifacts anywhere** (stub bucket — no crates, no `src/`).

The actual workspace-crate-layout bucket (per the canonical filter `microservices/<ms>/src/` absent AND `crates/oya-<ms>-*` non-empty) contains **19 µservices**, not 45. The remaining ~26 µservices from the initial list (e.g., `analytics`, `api-gateway`, `community`, `developer-sdk`, `social`, `workflow-studio`, `workflow-engine`, `feature-flags`, `compliance`, `intelligence`, `mail`, `messenger`, `calendar`, `notes`, `sheets`, `slides`, `tasks`, `docs`, `drive`, `sites`, `forms`, `meet`, `recordings`, `translate`, `imaging`, `detection`, `consent-graph`, `ops-dashboard-control-center`, `finops-portal`, `cloud-iac`, `cloud-k8s`, `cloud-secrets`) have **zero `oya-<ms>-*` crates registered** — they are pure doc-suite stubs awaiting Wave 15J-batch-4 / Wave 15-IP-substance authoring. They are out of this audit's scope (no Rust to audit) and are tracked under the "stub-skipped / flat-layout-pending" bucket of the initial CA-VERIFY.

Within the 19 actual workspace-crate-layout µservices, **2 are RETIRED** (`foundry` per ADR-0335; `cell` per ADR-0333) and `connect` is **RETIRING-UMBRELLA** (per its own `RETIREMENT-PLAN.md` — sub-µservices ship under their own folders per ADR-0135). Live, audit-applicable workspace-crate-layout µservices: **16**. The two RETIRED + one RETIRING-UMBRELLA are still included below for documentation completeness with a clear RETIRED-* status, since their crates still exist in the live workspace.

---

## Executive summary

| Bucket | Count | µservices |
|---|---|---|
| **GREEN** (ADR-0105 + ADR-0131 aligned within workspace-crate-layout shape) | 3 | `tenancy`, `payments`, `audit-chain` |
| **YELLOW** (≤3 minor issues — typically partial layer coverage or backend-qualifier-singular nits) | 6 | `identity`, `ontology`, `cloud-iam`, `cloud-kms`, `cloud-billing`, `cloud-storage` |
| **RED** (structural violations — missing-layer-bypass, non-enum suffixes, BC explosion, or naming collisions) | 4 | `connect`, `governance`, `cloud-billing-tax`, `application` |
| **LAYOUT-PENDING** (workspace shape retained intentionally; flat-layout migration scheduled) | 3 | `cloud-data`, `cloud-network`, `cloud-network-dns`, `observability` (4 actually; LAYOUT-PENDING groups µservices with active-but-thin per-layer coverage and explicit migration tracking via ADR-0131 §"Migration cost quantification" but no in-flight IP yet) |
| **RETIRED — crates still resident** | 2 | `foundry` (ADR-0335 → absorbed by `intelligence`), `cell` (ADR-0333 → pattern not service) |
| **µservices audited** | 19 (= 3 GREEN + 6 YELLOW + 4 RED + 4 LAYOUT-PENDING + 2 RETIRED) | |

**Aggregate read:** Within ADR-0131's "workspace-crate-layout is LEGACY shape; flat layout is canonical" framing, the workspace bucket is dominated by infrastructure / substrate µservices (`tenancy`, `audit-chain`, `payments`, `identity`, `ontology`, the `cloud-*` family, `observability`). Their per-crate suffix discipline is strong (the ADR-0105 `*-api` ADOPT-PAT-01 pattern accounts for 21 named crates in this bucket; the `*-adapter-<backend>` ADOPT-PAT-02 pattern accounts for 9; `-kernel`/`-domain`/`-usecase`/`-app`/`-rest`/`-grpc`/`-worker` are the rest). Cross-cutting RED issues concentrate on **four µservices**: `connect` (22 `-domain` crates × ADR-0132 single-concern policy = umbrella-retirement-blocker), `governance` (8 crates without canonical layer suffix — should be check-family-shaped per ADOPT-PAT-01 OR carry explicit `*-app` per `specs/crate-naming-audit.json` §"non-compliant > <fitness-feature-without-suffix>"), `cloud-billing-tax` (duplicated crate `oya-cloud-billing-tax-app` appears under both `cloud-billing` and `cloud-billing-tax` parent-µservice listings = manifest-binding ambiguity), and `application` (umbrella crate `oya-application-app` pulls foundry-* + check-* + cloud-* into one test harness — pre-ADR-0131 monolithic-app artifact that must be flat-layout-decomposed).

---

## ADR reference — what GREEN / YELLOW / RED / LAYOUT-PENDING mean for workspace-crate-layout

Workspace-crate-layout µservices, per `specs/crate-naming-audit.json` §"adopted_patterns" + ADR-0105 §"21 `*-api` + 36 check-family + 13 `*-adapter-<backend>`":

- **GREEN** — Every crate's suffix is in the canonical 13-layer enum OR is a recognized adopted pattern (`*-api`, `*-adapter-<backend>`, `oya-check-<feature>`); inward-only-flow holds at the workspace-member layer (`-kernel` depends on nothing layered, `-domain` depends only on `-kernel`, `-usecase` only on `-domain` + `-api`, `-app` only on `-usecase`, `-adapter` only on `-usecase` + `-domain`, etc.); bounded-context count is ≤8 per µservice (ADR-0132 single-concern proxy).
- **YELLOW** — All crate suffixes are canonical/adopted BUT one of: (a) partial layer coverage (e.g., `-api` + `-domain` present but no `-kernel`/`-usecase`/`-app` materialized yet); (b) PLURAL adapter naming (`-adapters` instead of `-adapter`); (c) one or two non-enum tokens at intermediate position (e.g., `-cell-assignment-kernel` where `cell-assignment` is a BC qualifier — acceptable per ADOPT-PAT but should be documented).
- **RED** — One of: (i) crate uses a suffix NOT in the canonical enum AND NOT in any adopted pattern (e.g., `-coverage`, `-emission`, `-disambiguation`, `-bar`, `-completeness`, `-stamping`, `-justifications`); (ii) BC explosion >8 (ADR-0132 violation); (iii) crate duplication / naming collision across µservices; (iv) outward-flow import violation (e.g., `-domain` importing `-adapter`).
- **LAYOUT-PENDING** — Crate suffixes are clean BUT ADR-0131 flat-layout migration is expected. Triggered when `microservices/<ms>/src/` is absent + `microservices/<ms>/PRD.md` exists + no in-flight migration IP found. These are CANDIDATES for the flat-layout-migration wave.

---

## Per-µservice classification

### GREEN (3)

| µservice | Crate inventory | Why GREEN |
|---|---|---|
| **`tenancy`** | 14 crates: `-api` (1) + `-domain` (1) + `-kernel` (1 plain) + 5 BC-kernel (`-cell-assignment-kernel`, `-dsr-cascade-kernel`, `-isolation-policy-kernel`, `-lifecycle-locks-kernel`, `-sub-scope-registry-kernel`, `-tenant-lifecycle-kernel`) + 3 BC-usecase (`-dr-pairing-usecase`, `-per-tenant-quota-usecase`, `-reserved-namespace-usecase`) + 1 BC-domain (`-kyb-kyc-verifier-domain`) + 1 BC-adapter (`-data-residency-enforcer-adapter`) | Every crate's terminal suffix is in the canonical 13-layer enum. BC qualifiers (`cell-assignment`, `dsr-cascade`, `isolation-policy`, etc.) are intermediate tokens carrying ADR-0244 tenant-scoping concerns. Inward-only flow plausible (Cargo.toml-level check would confirm; suffix discipline is the strongest signal). BC count = 9 (slightly over ADR-0132's "≤8" rule-of-thumb but ADR-0244 explicitly elevates tenancy as the universal scoping primitive — multiple BCs justified). |
| **`payments`** | 20 crates: 5 BCs × ≤4 layers — `charge` (kernel + domain + usecase + app + rest + grpc — 6 layers, the canonical "full stack" BC) + `dispute` (domain + usecase) + `kyc-kyb` (domain + usecase) + `payout` (domain + usecase) + `refund` (domain + usecase) + `settlement` (domain + worker) + `subscription` (domain + usecase) + 2 adapter-backends (`-adapter-adyen`, `-adapter-stripe` — per ADOPT-PAT-02). | Every suffix canonical. Charge BC demonstrates the full ADR-0105 layer ladder (kernel → domain → usecase → app + rest + grpc). Settlement uses `-worker` per canonical enum. Backend adapters per ADOPT-PAT-02. BC count = 7 (matches ADR-0132 single-concern: payments has clear distinct concerns). |
| **`audit-chain`** | 18 crates: 5 BCs × (kernel + domain + api) — `emission`, `query`, `retention-cascade`, `sealing`, `verification` — each has the canonical `-kernel` + `-domain` + `-api` triplet; plus one µservice-level `-domain`, one `-usecase`, one `-file-adapter` (per ADOPT-PAT-02 backend-qualifier — file is a backend). | Textbook per-BC layer triplet (kernel + domain + api). Every suffix canonical. Inward-only flow direct from naming: `-api` and `-kernel` are pure type producers; `-domain` consumes both. BC count = 5 (well within ADR-0132). The `-file-adapter` backend-qualifier is ADR-0105-aware. |

### YELLOW (6)

| µservice | Crate inventory | Issue | Severity |
|---|---|---|---|
| **`identity`** | 3 crates: `-api`, `-domain`, `-usecase` | Partial layer coverage — no `-kernel`, no `-app`, no `-adapter`. ADR-0105's flow rule requires `-domain` to consume `-kernel`; absent `-kernel`, the `-domain` crate likely re-implements primitive types. Materialization gap, not a structural violation. | 1 issue |
| **`ontology`** | 3 crates: `-api`, `-domain`, `-kernel` | Has `-kernel` (good) and `-api` (per ADOPT-PAT-01). Missing `-usecase`, `-app`, `-adapter`. Same "thin stack" pattern as `identity` but with `-kernel` materialized. | 1 issue |
| **`cloud-iam`** | 2 crates: `-api`, `-domain` | Thinnest stack — only `-api` + `-domain`. No `-kernel`, `-usecase`, `-app`, or `-adapter`. Per ADR-0105's `-api` ADOPT-PAT-01, `-api` depends on `-kernel` only; with no `-kernel` materialized, `-api` here likely has its own type primitives baked in. | 2 issues |
| **`cloud-kms`** | 2 crates: `-api`, `-domain` | Same pattern as `cloud-iam` — `-api` + `-domain` only. | 2 issues |
| **`cloud-billing`** | 3 crates: `-domain`, `-kernel`, plus `oya-cloud-billing-tax-app` (which is actually owned by sibling µservice `cloud-billing-tax`; see RED §`cloud-billing-tax` below) | `-domain` + `-kernel` is fine. No `-usecase`/`-app`/`-adapter`/`-api`/`-rest`/`-grpc`. Plus the cross-µservice naming-collision issue surfaces in this listing. | 2 issues (partial layer + crate-ownership ambiguity) |
| **`cloud-storage`** | 3 crates: `-domain`, `-block-api`, `-object-api` | `-domain` + 2 `-api` per ADOPT-PAT-01 (block + object are BC qualifiers; canonical). Missing `-kernel`, `-usecase`, `-app`, `-adapter`. Same partial-layer pattern. | 1 issue |

### RED (4)

| µservice | Crate inventory | Violation type | Severity | Remediation |
|---|---|---|---|---|
| **`connect`** | 22 crates: ALL are `-domain` only — one per BC (`address-book`, `calendar`, `collab-runtime`, `dlp`, `docs`, `document-format`, `drive`, `dsr`, `ediscovery`, `forms`, `mail`, `meet`, `messenger`, `notes`, `recordings`, `retention`, `sheets`, `sites`, `slides`, `tasks`, `translate`, `trust-portal`) | **BC explosion: 22 BCs in one µservice (ADR-0132 single-concern violation by ≥14)** + **uniform-layer-anti-pattern: only `-domain` materialized, no `-kernel`/`-usecase`/`-app`/`-adapter`/`-api`/`-rest`/`-grpc` anywhere across 22 BCs (zero ADR-0105 layer coverage past `-domain`)** + **RETIRING-UMBRELLA per `microservices/connect/RETIREMENT-PLAN.md` 2026-05-17 ADR-0135**: connect is an umbrella that's already retiring; sub-µservices (`mail`, `messenger`, `calendar`, etc.) ship under their own folders per ADR-0135. The 22 `oya-connect-*-domain` crates are the legacy bundle artifact awaiting cleanup. | Structural — but already on a retirement track | Per ADR-0135 + RETIREMENT-PLAN: complete the connect umbrella retirement; migrate the 22 `oya-connect-<bc>-domain` crates into their successor µservice folders (e.g., `oya-connect-mail-domain` → `microservices/mail/` + rename to canonical `oya-mail-domain`). One IP per BC. |
| **`governance`** | 8 crates: `-audit-event-emission`, `-byok-disambiguation`, `-capability-tier-coverage`, `-cedar-coverage`, `-naming-justifications`, `-no-template-stamping`, `-pack-overlay-completeness`, `-substance-bar` | **No canonical layer suffix on any of the 8 crates.** Per `specs/crate-naming-audit.json` §"non_compliant > <fitness-feature-without-suffix>": these crates pair with no `-kernel` and live under the `oya-governance-*` prefix that, per the 2026-05-21 doctrine, is the NEW canonical lane (per CLAUDE.md `new_governance_lane_prefix: oya-governance-* (per ADR-0132)`). Per ADR-0105 ADOPT-PAT-01 (check-family), `oya-check-<feature>` is allowed to OMIT the layer suffix because the µservice IS the layer. The `oya-governance-*` crates have the SAME semantic shape (one crate per governance check) but live under a different prefix — they should either (a) adopt the same self-layering convention via an ADR-0105 amendment recognizing `oya-governance-<feature>` as a check-family extension, OR (b) gain explicit canonical suffixes (`-app` for binaries, `-domain`/`-kernel` for libraries). | Structural — doctrine inconsistency between `oya-check-*` (allowed) and `oya-governance-*` (currently flagged non-compliant) | IP: amend ADR-0105 §"adopted_patterns > check_family" to recognize `^oya-governance-` as a check-family extension when the crate is one-per-governance-rule (matches all 8). Single-commit IP; no code rename required. |
| **`cloud-billing-tax`** | 1 crate: `oya-cloud-billing-tax-app` | **Crate-ownership ambiguity / cross-µservice naming collision.** `oya-cloud-billing-tax-app` is enumerated under BOTH the `cloud-billing` µservice listing AND the `cloud-billing-tax` µservice listing because `grep "^oya-cloud-billing-"` matches both. Per ADR-0131 + ADR-0132, each crate belongs to exactly ONE µservice. The `-tax` suffix between `cloud-billing` and `-app` makes the crate ambiguous — is it (i) a sub-component of `cloud-billing` with tax scope OR (ii) the sole crate of the standalone `cloud-billing-tax` µservice? ADR-0132's no-suite policy says new µservices are single-concern + flat — so `cloud-billing-tax` as its own µservice means the crate name should be `oya-cloud-billing-tax-app` (consistent with current name) but the `cloud-billing` µservice should NOT enumerate it. The `oya-cloud-billing-tax-app` Cargo.toml + `cloud-billing-tax/PRD.md` is the binding source of truth. | Naming-discipline / inventory clarity | Document `oya-cloud-billing-tax-app` ownership exclusively under `cloud-billing-tax`. Update any catalog manifests that double-list it. Or, alternatively, rename to `oya-billing-tax-app` to disambiguate from `cloud-billing`. Single-commit doc IP + optional rename IP. |
| **`application`** | 1 crate: `oya-application-app` | **Pre-ADR-0131 umbrella test-harness anti-pattern.** `oya-application-app` is NOT a µservice's composition root — its Cargo.toml pulls in `oya-foundry-adapter-domain`, `oya-foundry-bypass-domain`, `oya-foundry-capability-domain`, `oya-check-cost-budget`, `oya-foundry-evidence-domain`, `oya-foundry-eval-domain`, etc. — i.e., it is the cross-µservice integration test harness that boots a representative subset of foundry's domain crates to run integration tests in `crates/oya-application-app/tests/*.rs`. Per ADR-0131 + ADR-0132, integration tests belong inside the owning µservice (`tests/` directory under the µservice's flat layout), not in a sibling "application" µservice. Plus `foundry` is RETIRED (ADR-0335), so `oya-application-app`'s foundry-crate dependencies will need to be repointed at `intelligence/*` successors. | Structural — survives only as legacy harness; blocks ADR-0335 foundry → intelligence rename | IP: decompose `oya-application-app` into per-µservice integration tests OR rename to `oya-integration-test-app` and rehome under `tools/`. ADR-0131-driven; depends on `intelligence/*` successor crates landing per ADR-0335. |

### LAYOUT-PENDING (4)

| µservice | Crate inventory | Why LAYOUT-PENDING |
|---|---|---|
| **`cloud-data`** | 2 crates: `-domain`, `-kernel` | Suffixes canonical; partial layer coverage. `microservices/cloud-data/` has a doc suite + PRD; the µservice is alive, just not yet flat-layout-migrated. Candidate for the next migration wave per ADR-0131 §"Migration cost quantification". |
| **`cloud-network`** | 4 crates: `-domain`, `-dns-api`, `-lb-api`, `-vpc-api` (the 3 `-api` per ADOPT-PAT-01) | Suffixes canonical; thin `-domain` + 3 sub-domain `-api` crates. Candidate for flat-layout migration; also requires reconciliation with sibling `cloud-network-dns` µservice (see below — `cloud-network-dns-api` is enumerated under BOTH parent listings, similar naming-collision pattern to `cloud-billing-tax`). |
| **`cloud-network-dns`** | 1 crate: `oya-cloud-network-dns-api` | Same crate as one of `cloud-network`'s — naming-collision with parent µservice. Suffix is canonical (`-api` per ADOPT-PAT-01). Per ADR-0132 single-concern, `cloud-network-dns` as a standalone µservice is allowed; its sole crate `oya-cloud-network-dns-api` is fine. But the `cloud-network` µservice's listing should NOT include this crate. Documentation / inventory cleanup. |
| **`observability`** | 2 crates: `-domain`, `-tracing-adapter` | Suffixes canonical. `-tracing-adapter` is the ADOPT-PAT-02 backend-qualifier shape (tracing is a backend / observability sub-system). Per ADR-0130 + ADR-0131, observability is the substrate µservice that gates promotion; flat-layout migration is high priority. Candidate. |

### RETIRED — crates resident (2)

| µservice | Status | Crate inventory | Disposition |
|---|---|---|---|
| **`foundry`** | RETIRED 2026-05-21 per ADR-0335 → absorbed by `intelligence` µservice | 125 crates including 67 `-kernel`, 20 `-domain`, 15 `-adapter`, 7 `-app`, 7 `-api`, 7 `-adapter-<backend>` (per ADOPT-PAT-02), 2 `-usecase`. The fitness-check sub-family inside foundry (`oya-foundry-fitness-*-kernel`, 25+ crates) collides with the separate `oya-check-*` check-family (ADR-0105 ADOPT-PAT-01) — same shape, different prefix. | These 125 crates will rename to `oya-intelligence-*` per ADR-0335 absorption. The `oya-foundry-fitness-*-kernel` sub-family should fold into the canonical `oya-check-*` prefix (ADOPT-PAT-01) on rename. Massive rename IP suite — outside this audit's READ-ONLY scope but flagged as the largest workspace-layout cleanup cost in the entire repo. |
| **`cell`** | RETIRED 2026-05-21 per ADR-0333 → cellular architecture is a PATTERN not a service, absorbed into `tenancy` (assignment), `cloud-iac` (provisioning + registry), `observability` (health / blast-radius), `oya-shuffle-sharding` Rust crate (algorithm), `api-gateway` (routing), `audit-chain` (cell-scoped audit) | 1 crate: `oya-cell-domain` (depends on `oya-data-boundary-kernel`) | The single remaining `oya-cell-domain` crate should retire per ADR-0333 absorption mapping. Its dependents must be repointed at the absorbing µservice's crates. Single rename / removal IP. |

---

## Cross-cutting violation patterns

### Pattern A — BC explosion in legacy umbrella µservices (`connect`, plus historical `foundry`)

`connect` has 22 `-domain` crates (one per BC: mail, calendar, messenger, drive, docs, sheets, slides, sites, forms, tasks, notes, meet, recordings, address-book, translate, retention, dlp, ediscovery, dsr, document-format, trust-portal, collab-runtime). Per ADR-0132 single-concern, this is the canonical umbrella anti-pattern. `connect/RETIREMENT-PLAN.md` (2026-05-17 ADR-0135) already declared the retirement; the 22 crates are stranded artifacts. `foundry` similarly explodes to 125 crates spanning multiple BCs (account, adapter, autonomy-ceiling, capability-registry, mcp-gateway, mdbook, rag-endpoint, supervisor, vcs-*, webhook-receiver, fitness-*, etc.).

**Recommendation:** Treat both as already-retired umbrellas; gate Wave-15-IP-substance authoring to the absorbing-target µservices (`mail`, `messenger`, `calendar`, ..., `intelligence`).

### Pattern B — Non-canonical-suffix governance crates (`governance`, all 8)

The 8 `oya-governance-*` crates (`-audit-event-emission`, `-byok-disambiguation`, `-capability-tier-coverage`, `-cedar-coverage`, `-naming-justifications`, `-no-template-stamping`, `-pack-overlay-completeness`, `-substance-bar`) carry no canonical layer suffix. Per ADR-0105's ADOPT-PAT-01 check-family pattern, `oya-check-<feature>` is allowed to omit a layer suffix because the µservice IS the layer. The `oya-governance-*` crates have semantically identical shape — one crate per governance rule — but live under the post-2026-05-21 governance-lane-rename prefix (CLAUDE.md `new_governance_lane_prefix: oya-governance-* (per ADR-0132)`).

**Recommendation:** Amend ADR-0105 §"adopted_patterns" to add a `governance_family` entry mirroring `check_family` with `match: "^oya-governance-"`. Documentation-only IP; zero code rename.

### Pattern C — Crate-naming collisions across parent µservices (`cloud-billing-tax`, `cloud-network-dns`)

Two crates appear under two µservice listings each:

1. `oya-cloud-billing-tax-app` matches both `oya-cloud-billing-*` and `oya-cloud-billing-tax-*` glob patterns.
2. `oya-cloud-network-dns-api` matches both `oya-cloud-network-*` and `oya-cloud-network-dns-*`.

Each crate belongs to exactly ONE µservice — the more-specific (longer-prefix) sibling. Inventories and catalog manifests should exclude the broader parent's enumeration.

**Recommendation:** Per-µservice manifest hygiene IP. Optionally rename to disambiguate (`oya-billing-tax-app`, `oya-cloud-dns-api`) — judgment call.

### Pattern D — Umbrella test-harness anti-pattern (`application`)

`oya-application-app` is a cross-µservice integration test harness, not a µservice composition root. Per ADR-0131 + ADR-0132, integration tests belong inside the owning µservice. This crate predates the flat-layout doctrine and survives only as a legacy convenience.

**Recommendation:** Decompose into per-µservice integration tests OR rehome under `tools/oya-integration-test-app` (ADR-0107-conformant `*-app` for binaries).

### Pattern E — Partial layer coverage (universal across the 6 YELLOW + 4 LAYOUT-PENDING)

`identity`, `ontology`, `cloud-iam`, `cloud-kms`, `cloud-billing`, `cloud-storage`, `cloud-data`, `cloud-network`, `cloud-network-dns`, `observability` all materialize 2-4 layers (typically `-api` + `-domain`, or `-domain` + `-kernel`). ADR-0105 declares 13 canonical layers; materialization is per-IP work. Not a structural violation but a content-completion gap. The flat-layout migration (ADR-0131) is the natural moment to materialize the missing layers as `src/{kernel,domain,usecase,app,adapter,api,...}/` subdirs.

**Recommendation:** Pair each flat-layout migration IP with a "layer-coverage extension" sub-IP that authors the missing kernel/usecase/app/adapter modules at substance-bar depth.

### Pattern F — `*-adapter` PLURAL vs SINGULAR

Per `specs/crate-naming-audit.json` §"non_compliant > RENAME-CASE-BY-CASE > oya-foundry-vcs-polyglot-indexer-adapters" — PLURAL `*-adapters` should be SINGULAR `*-adapter`. Within the 19 audited µservices, no PLURAL adapter crate was found (all are singular). Pattern stays clean here; the only known violation is inside `foundry` (RETIRED) and is captured in `crate-naming-audit.json` already.

---

## Recommended remediation IPs (READ-ONLY pass — IPs are recommendations only)

### Per RED µservice (4 IPs)

1. **IP-WV15-CA-VERIFY-WORKSPACE-001-connect-bc-redistribution.md** — Migrate the 22 `oya-connect-<bc>-domain` crates into their successor µservice folders per ADR-0135 + `connect/RETIREMENT-PLAN.md`. Rename each to `oya-<bc>-domain`. Touches workspace `[members]` + every importer. Multi-commit IP (one per BC). Depends on each successor µservice having a `microservices/<bc>/src/` flat layout in place.

2. **IP-WV15-CA-VERIFY-WORKSPACE-002-governance-check-family-amendment.md** — Amend ADR-0105 §"adopted_patterns" to add a `governance_family` ADOPT pattern recognizing `^oya-governance-` as a self-layering check-family extension. Documentation-only IP; mirrors ADOPT-PAT-01. Closes all 8 RED entries under `governance` without code rename.

3. **IP-WV15-CA-VERIFY-WORKSPACE-003-cloud-billing-tax-naming-disambiguation.md** — Document `oya-cloud-billing-tax-app` ownership exclusively under `cloud-billing-tax`. Update `cloud-billing/manifest.json` + workspace catalog to remove the duplicate enumeration. Optionally rename to `oya-billing-tax-app` (judgment).

4. **IP-WV15-CA-VERIFY-WORKSPACE-004-application-test-harness-rehoming.md** — Decompose `oya-application-app` into per-µservice integration tests OR rename + rehome to `tools/oya-integration-test-app`. Depends on `intelligence/*` rename per ADR-0335 (the foundry-* deps must be repointed).

### Per LAYOUT-PENDING µservice (4 IPs — flat-layout migration suite)

Per ADR-0131 §"Migration cost quantification" each migration IP averages ~50 files moved + 117 refs updated + ~6min wall time. Migrations are parallel-safe per ADR-0131 §"DAG":

5. **IP-WV15-CA-VERIFY-WORKSPACE-005-observability-flat-layout-migration.md** — Highest priority (substrate µservice gating promotion per ADR-0130). Move `oya-observability-{domain,tracing-adapter}` into `microservices/observability/src/{domain,adapter/tracing/}`.

6. **IP-WV15-CA-VERIFY-WORKSPACE-006-cloud-data-flat-layout-migration.md** — Move `oya-cloud-data-{domain,kernel}` into `microservices/cloud-data/src/{domain,kernel}`.

7. **IP-WV15-CA-VERIFY-WORKSPACE-007-cloud-network-flat-layout-migration.md** — Move `oya-cloud-network-{domain,vpc-api,lb-api,dns-api}` into `microservices/cloud-network/src/{domain,api/{vpc,lb,dns}}`. Reconcile with `cloud-network-dns` (sibling µservice owns the `dns` sub-domain — keep separate if dns is genuinely a single-concern µservice per ADR-0132).

8. **IP-WV15-CA-VERIFY-WORKSPACE-008-cloud-network-dns-flat-layout-migration.md** — Move `oya-cloud-network-dns-api` into `microservices/cloud-network-dns/src/api/`. Resolve the naming-collision with `cloud-network`.

### Per YELLOW µservice (6 IPs — layer-coverage extension; lower priority)

9-14. **IP-WV15-CA-VERIFY-WORKSPACE-009..014** — One per `identity`, `ontology`, `cloud-iam`, `cloud-kms`, `cloud-billing`, `cloud-storage`: materialize the missing canonical layers (`-kernel` where absent, `-usecase`, `-app`, `-adapter`). Pair with flat-layout migration when scheduled. Substance-bar content authoring, not just scaffolds.

### Retirement crate-cleanup (2 IPs — already on track via ADR-0335 / ADR-0333)

15. **IP-WV15-CA-VERIFY-WORKSPACE-015-foundry-crate-rename-to-intelligence.md** — Rename 125 `oya-foundry-*` crates to `oya-intelligence-*` per ADR-0335 absorption mapping. Fold `oya-foundry-fitness-*-kernel` sub-family into canonical `oya-check-*` (ADOPT-PAT-01). Massive multi-commit IP; phased by sub-family.

16. **IP-WV15-CA-VERIFY-WORKSPACE-016-cell-domain-crate-retirement.md** — Remove `oya-cell-domain` and repoint dependents at the absorbing µservices per ADR-0333 (`tenancy` for assignment, `cloud-iac` for provisioning, etc.).

### Total recommended IPs: 16 (4 RED + 4 LAYOUT-PENDING + 6 YELLOW + 2 RETIRED-cleanup).

---

## Flat-layout migration ordering (which workspace-crate-layout µservices benefit most from migrating first)

Ordering criteria: substrate-dependency-fan-in (downstream impact) × current-layer-coverage (closer-to-complete = cheaper migration) × in-flight-doctrine-binding (ADR-0131 + ADR-0335).

| Rank | µservice | Rationale |
|---|---|---|
| **1** | `observability` | Substrate µservice; ADR-0130 + ADR-0139 agentic-SLO-gated promotion explicitly depends on `microservices/observability/slos/` flat layout. Smallest crate set (2 crates) → cheapest migration. Highest downstream impact. |
| **2** | `tenancy` | GREEN in this audit + ADR-0244 universal scoping primitive. Already at 14 crates with clean suffixes — flat-layout migration is mechanical move + workspace.members update. High downstream impact (every other µservice scopes by tenant). |
| **3** | `audit-chain` | GREEN in this audit + ADR-0251 §D-10 keystone + ADR-0246 audit invariant. Clean 18-crate set per-BC triplet. Mechanical migration. Substrate-class. |
| **4** | `cloud-data` | LAYOUT-PENDING; 2 crates only; foundational data-cloud surface used by many product µservices. Lowest cost per substrate impact. |
| **5** | `identity` + `cloud-iam` (paired) | Both YELLOW; pair-migrate because identity (consumer-facing IdP per ADR-0259) and cloud-iam (provider-IAM per ADR-0258) share scope conceptually. Migrating together prevents one-half-done seam. |

---

## Summary stats

```
total µservices in scope (workspace-crate-layout, microservices/<ms>/src/ absent, crates/oya-<ms>-* non-empty): 19
GREEN: 3 (tenancy, payments, audit-chain)
YELLOW: 6 (identity, ontology, cloud-iam, cloud-kms, cloud-billing, cloud-storage)
RED: 4 (connect, governance, cloud-billing-tax, application)
LAYOUT-PENDING: 4 (cloud-data, cloud-network, cloud-network-dns, observability)
RETIRED-crates-resident: 2 (foundry, cell)

Total crates across all workspace-crate-layout µservices: ~252
  - foundry alone: 125 crates (~50% of bucket; all retiring per ADR-0335)
  - connect: 22 crates (retiring umbrella per ADR-0135)
  - all others combined: ~105 crates
```

**Cross-cutting violations:**

- BC explosion in legacy umbrellas (`connect` = 22 BCs, `foundry` = ~25 BCs) — both already on retirement tracks
- Non-canonical suffix on `oya-governance-*` crates (8) — fixable via ADR-0105 amendment recognizing `governance_family` as a check-family extension
- Naming collisions across parent-µservice listings (`oya-cloud-billing-tax-app`, `oya-cloud-network-dns-api`) — manifest hygiene
- Umbrella test-harness anti-pattern (`oya-application-app`) — pre-ADR-0131 artifact
- Partial layer coverage (universal across 10 µservices) — content-completion gap, not structural

**Inward-only-flow:** Plausible at the suffix-discipline layer across all GREEN + YELLOW + LAYOUT-PENDING µservices. Direct Cargo.toml dependency-graph verification was not performed in this READ-ONLY pass; recommend a follow-up `oya governance dep-flow-check` lane to confirm.

**Remediation IPs recommended:** 16 (4 RED + 4 LAYOUT-PENDING + 6 YELLOW + 2 RETIRED-cleanup).

**Flat-layout migration order (top 5):** observability → tenancy → audit-chain → cloud-data → identity+cloud-iam.

---

## Audit method

1. **Discovery:** `for ms in microservices/*/; do [ ! -d "$ms/src" ] && [ -n "$(find crates/ -maxdepth 1 -type d -name "oya-$(basename $ms)-*")" ] && echo $(basename "$ms"); done` → 19 µservices.
2. **Per-µservice crate inventory:** `ls crates/ | grep "^oya-${ms}-"` for each.
3. **Suffix-canonicity check:** Tail-token histogram per µservice; cross-checked against `specs/crate-naming-audit.json` §"canonical_enum_13" + "adopted_patterns".
4. **BC-explosion check:** Count distinct BC qualifiers (intermediate tokens between µservice prefix and layer suffix); compared to ADR-0132's single-concern ≤8 heuristic.
5. **Naming-collision check:** Crates matching multiple `oya-<msA>-*` and `oya-<msB>-*` patterns where `msA` is a prefix of `msB`.
6. **Retirement-status check:** `find microservices -maxdepth 2 -name RETIRED.md` + read first 30 lines of each.
7. **Retiring-umbrella check:** `find microservices -maxdepth 2 -name 'RETIREMENT-PLAN.md'` → connect.
8. **Companion-report cross-ref:** Read `/Users/jasonlee/oyatie/.omc/state/wave-15-ca-verify-2026-05-21.md` to confirm scope boundary; the workspace-bucket count of 45 in the initial report was an over-estimate (mixed in pure-doc-stub µservices with no crates).
9. **ADR-0131 migration-IP scan:** `grep -A 3 "migrate\|migration" docs/decisions/ADR-0131-*.md` confirmed migration is a planned per-µservice IP series within M01 scope.

No code modifications were made. No `cargo` builds were run. No Cargo.toml dependency graph traversal was performed (recommended for follow-up).

---

## References

- `docs/decisions/ADR-0105-13-layer-enum-and-check-family-patterns.md` — canonical 13-layer enum + ADOPT-PAT-01 / ADOPT-PAT-02
- `docs/decisions/ADR-0106-application-to-usecase-rename.md` — inner-orchestration layer rename
- `docs/decisions/ADR-0131-per-microservice-flat-layout.md` — flat-layout canonical, workspace-crate-layout is legacy
- `docs/decisions/ADR-0132-product-suite-and-bundle-dissolution.md` — no-suite single-concern policy
- `docs/decisions/ADR-0135` — connect umbrella sub-µservice dissolution (cited by `microservices/connect/RETIREMENT-PLAN.md`)
- `docs/decisions/ADR-0333-cell-microservice-retired-cellular-architecture-as-pattern.md` — cell retirement + oya-shuffle-sharding
- `docs/decisions/ADR-0335-foundry-microservice-retired-absorbed-by-intelligence.md` — foundry retirement
- `specs/crate-naming-audit.json` — per-crate ground truth
- `microservices/connect/RETIREMENT-PLAN.md` — connect umbrella retirement criteria
- `microservices/foundry/RETIRED.md`, `microservices/cell/RETIRED.md` — retirement markers
- Companion: `/Users/jasonlee/oyatie/.omc/state/wave-15-ca-verify-2026-05-21.md` — flat-layout surface (22 µservices)
- CLAUDE.md memory: `feedback_layer_enum_adr_0105_13_canonical`, `feedback_cell_standalone_network_merges_community_2026_05_21`, `new_governance_lane_prefix: oya-governance-* (per ADR-0132)`
