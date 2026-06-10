# 20 — VERIFY: repo-structure tension (pure-split vs linux migration plan)

> Lane: backlog-reconciliation. READ-ONLY verification. Created 2026-06-06.
> Verifies the source backlog STRUCTURAL RULING (PURE SPLIT: exactly two service trees `oya/` + `cloud/`;
> `services/`/`platforms/`/`microservices/` are SPRAWL to eliminate; ADR-0131/0512 amended to NOT create a 3rd
> `microservices/` tree) against the linux migration plan's canonical-homes claim.

## A. LIVE top-level tree inventory — `ls /Users/jasonlee/Developer/source` (verified 2026-06-06)

Counted via `find <tree> -maxdepth 1 -mindepth 1 -type d | wc -l`.

| Tree | Exists? | Subdir count | Holds real service code? | Verdict |
|---|---|---|---|---|
| `oya/` | YES | **87** | YES — `oya/<svc>/crates/oya-*` confirmed (e.g. `oya/crm/crates/`, `oya/identity/crates/`) | CANONICAL product/domain tree |
| `cloud/` | YES | **25** | YES — `cloud/<svc>/crates/oya-cloud-*` confirmed (e.g. `cloud/cloud-data/crates/`, `cloud/managed-k8s-control-plane-host/crates/`) | CANONICAL platform tree |
| `services/` | YES | **5** | NO — empty husks. `services/analytics`= only `.DS_Store`; `services/app-shell-frontend`= empty; `services/policy/crates`= empty; `services/treasury/src`= empty; `services/ci-webhook-gateway`= only `target/` build dir | **SPRAWL / zombie — eradicate** |
| `platforms/` | YES | **0** | NO — contains only a stray `BUCK` file, zero subdirs | **SPRAWL / zombie — eradicate** |
| `microservices/` | **NO** | — | n/a — does NOT exist at top level. (`specs/microservices/` exists, but that is a *specs* subdir, not a service tree) | already absent (migration already removed it) |
| `packs/` | YES | 9 | localization packs (`au/br/cn-pipl/eu/in/jp/kr/mx/us`) | cross-cutting pack root — ADR-0131 Q3 keeps it |
| `regional-packs/` | YES | 5 | regional packs (`eu/jp/kr/ksa/us-government`) | cross-cutting pack root — ADR-0131 Q3 keeps it |
| `libs/` | YES | **168** | YES — shared cross-cutting code (`oya-check-*`, `oya-governance-*`, …) | CANONICAL shared root |

Additional structural finding (ADR-0512 conformance gap **inside source itself**):
- **Flat top-level `crates/` STILL EXISTS** with **2** subdirs (`oya-application-app`, `oya-audit-chain-emission-api`).
  ADR-0512 §Decision.1 declares a flat top-level `crates/` **"forbidden"** and a removal-candidate. Source has not
  finished its own consolidation — flat `crates/` is not yet empty.
- `cloud/cloud-k8s/` exists but currently has **no `crates/` subdir** (populated home pending k8s merge).

Net: the **only two real service-holding trees** are `oya/` (87) and `cloud/` (25) — i.e., the live state already
matches PURE SPLIT for *populated* trees. The deviation is **residual SPRAWL husks** (`services/` 5 husks,
`platforms/` 0-subdir husk, flat `crates/` 2 crates) that have not been deleted, plus the source-side k8s home not
yet populated. `microservices/` is already gone.

## B. ADR-0131 + ADR-0512 frontmatter + status (canonical copies at `source/docs/decisions/`)

### ADR-0131 — `ADR-0131-per-microservice-flat-layout.md`
- Frontmatter: `id: ADR-0131`, **`status: Accepted`**, `planning_impact: true`, `date: 2026-05-17`,
  `owner: council-architecture`, `related: [… ADR-0512]`.
- **Status body is AMENDED for pure split** (lines 24-28):
  > "**Amended — 2026-06-02 (pure split):** ADR-0512/platform-readiness updates the top-level service root from
  > `microservices/<ms>/` to `{oya,cloud}/<service>/`. … The old `microservices/` root is legacy only and must be
  > removed after migration evidence proves every service has landed under `oya/` or `cloud/` (or shared code under `libs/`)."
- Decision text (line 62): "flat colocated service folders under `{oya,cloud}/<service>/` … `oya/` holds
  product/domain services; `cloud/` holds platform/tenant-substrate services; shared cross-cutting code remains
  under `libs/`." Rationale (line 165): "preserves ADR-0131's colocation principle **without creating a third
  service tree**." ⇒ **ADR-0131 already amended to the pure split; no third tree.**

### ADR-0512 — `ADR-0512-canonical-monorepo-pattern.md`
- Frontmatter: `id: ADR-0512`, **`status: Accepted`**, `planning_impact: true`, `date: 2026-05-29`,
  `owners: [council-architecture, founder]`, `supersedes: [ADR-0357, ADR-0509]`, **`amends: [ADR-0131]`**.
- **Status body AMENDED for pure split** (lines 24-27):
  > "**Amendment — 2026-06-02 (platform-readiness pure split):** the top-level service root is no longer
  > `microservices/<ms>/`. Canonical service homes are `{oya,cloud}/<service>/` with shared cross-cutting libraries
  > under `libs/<lib>/`. `microservices/` is legacy only and must be removed after P0.1/P0.6 prove all migration
  > packets are complete."
- Decision (lines 53-62): service code at `{oya,cloud}/<service>/crates/<crate>/`; flat top-level `crates/`
  **"forbidden"**; `microservices/` "legacy/removal-candidate and must be empty after verified migration";
  `architecture-boundaries` + `workspace-topology` gates enforce. ⇒ **ADR-0512 already amended to the pure split.**

**Both 0131 and 0512 are Accepted AND already carry the 2026-06-02 pure-split amendment.** The amendments the
source backlog calls for are, on disk, **already authored into the canonical ADR copies**. (Note: many stale
duplicate copies exist under `source/.claude/worktrees/**`; the canonical authority is `source/docs/decisions/`.)

## C. What `source/docs/AGENTS.md` (the cited topology authority) actually says

The linux plan (line 38-39) cites `source/docs/AGENTS.md §Repository topology` as the eradication authority. Verbatim:
- Line 254: ``{oya,cloud}/<service>/crates/<crate>` + `libs/<lib>/`` = "Canonical implementation homes per
  ADR-0131/ADR-0512 platform-readiness amendment. Top-level `crates/` is legacy/removal-candidate until verified migration."
- Line 256: ``modules/`, `services/`, `platform/`, `tools/`` = "Retired legacy implementation roots; do not recreate."

**Gap in the AGENTS.md retirement list** (matters for eradication completeness):
- It lists **`platform/` (singular)** — but the LIVE tree is **`platforms/` (plural)**. The actual on-disk husk
  is not literally named in the retirement row.
- It does **NOT list `microservices/`** in the retirement row at all (ADR-0131/0512 cover it in prose, but the
  AGENTS.md topology table omits it — moot on disk since `microservices/` no longer exists, but the eradication
  ruleset is incomplete as written).

## D. The linux migration plan's structural claim — `docs/migration/source-consolidation-plan.md`

- §2 (lines 38-39): "Canonical homes are **only** `{oya,cloud}/<service>/crates/<crate>` and `libs/<lib>/`
  (`tools/`, `services/`, `platform/`, `modules/` are RETIRED — `source/docs/AGENTS.md §Repository topology`)."
- §4 step 1 (line 66): every lane restructures "into `{oya,cloud}/<service>/crates/oya-*` (or `libs/`); crate basename == `[package].name`."
- Landing map (§2 lines 43-52) places every migrated tree under `oya/<service>/crates/` or `cloud/<service>/crates/`.

⇒ The plan's **canonical-home target is IDENTICAL to the pure-split ruling**: only `{oya,cloud}/<service>/crates/`
+ `libs/`. It introduces **no third service tree** and does **not** target `microservices/`.

## E. VERDICT

**COMPATIBLE (not contradicts) — but INCOMPLETE.**

1. **No contradiction.** The linux plan's canonical homes (`{oya,cloud}/<service>/crates/` + `libs/`) are exactly
   the pure-split destination. It creates no `microservices/` tree and adds nothing under `services/`/`platforms/`.
   Its landing map is fully pure-split-conformant. ADR-0131 and ADR-0512 are both `Accepted` and **already** carry
   the 2026-06-02 pure-split amendment — so there is no live ADR conflict for the plan to violate.

2. **Where it is incomplete (what the UNIFIED plan must ADD):**
   - **(a) Sprawl eradication is not a deliverable.** The plan only governs the *trees it migrates IN*; it never
     schedules **deletion of the residual husks** that already exist in source: `services/` (5 husk dirs),
     `platforms/` (0-subdir stray `BUCK`), and the flat top-level **`crates/` (2 crates, ADR-0512-forbidden)**.
     Pure split is not achieved until these are removed. The unified plan must add an explicit
     "eradicate sprawl husks + empty flat `crates/`" cleanup packet (git-mv the 2 flat crates into their
     `{oya,cloud}/<svc>/crates/` homes, then `rm` the husk trees), gated by the ADR-0512 `workspace-topology`
     check (which fails on flat `crates/`).
   - **(b) The 0131/0512 amendment-completeness gap is not flagged.** The amendments themselves are authored, but
     the AGENTS.md §Repository-topology retirement row (the plan's own cited authority) is stale: it lists
     `platform/` (singular) not the live `platforms/`, and omits `microservices/` entirely. The unified plan must
     add a doc-amendment to fix the AGENTS.md topology table so the retirement list literally names every husk
     (`platforms/`, `microservices/`) — otherwise a "do not recreate" gate can't catch the real dirs.
   - **(c) Stale duplicate ADR copies.** Both ADR-0131 and ADR-0512 are duplicated across many
     `source/.claude/worktrees/**` paths. Not a structural blocker (canonical copy under `docs/decisions/` is
     correct), but the unified plan's reachability/cleanup sweep should note these as removal candidates so a
     reader can't cite a stale worktree copy.
   - **(d) Source-side migration is itself unfinished.** Flat `crates/` non-empty + `cloud/cloud-k8s/` lacking a
     `crates/` home means source has NOT completed its own ADR-0131/0512 migration. The linux k8s-merge lane
     (§2 line 45) must verify/finish the `cloud/cloud-k8s/` + `managed-k8s-*` home population rather than assume it.

**One-line:** the linux migration plan is *compatible-but-incomplete* — it correctly targets the pure-split
canonical homes and creates no third tree, but it must additionally (a) eradicate the live `services/` +
`platforms/` husks and empty the forbidden flat `crates/`, and (b) repair the stale AGENTS.md retirement list so it
names `platforms/`/`microservices/`. No ADR amendment to 0131/0512 is needed for the split itself — those are
already `Accepted` + pure-split-amended; the missing work is *enforcement/eradication*, not re-decision.
