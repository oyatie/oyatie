# G030-B unit-class histogram — 2026-08-02

State: **PLANNING_ONLY — READ-ONLY PRODUCER-EQUIVALENT HISTOGRAM, NOT DELETION**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Classifier: first-match walk of committed `ci/facade/artifact-inventory-registry/src/unit-class-policy.json` (byte-identical to `libs/oya-ci-config/src/bundled/unit-class-policy.json`).  
TTL table: committed `ci/facade/artifact-inventory-registry/src/ttl-policy.json`.  
No producer binary run, no scm-facts materialization, no face write, no deletion, no push, no activation.

## Method

1. Universe = `git ls-tree -r --name-only b651080374113aeb57500eecbd9d1326f0404e48` → **18,886** paths.
2. Focus family = paths ending in `.md` / `.yaml` / `.yml` / `.json` / `.toml` → **13,959** (matches G030-A).
3. Each path classified by the committed rule table in declaration order (scratch → ephemeral → vendor → generated → build_config → evidence → registry → spec → doc → code → default husk).
4. This is **producer-equivalent classification only**. It is not owner/justification/reachability resolution and not a total-accounting RED/GREEN verdict. Those require scm-facts + owners + reachability faces (CI-materialized; decommitted on trunk).

## Whole-tree unit_class histogram (all 18,886)

| unit_class | count | ttl_class | budget_days | action | protected |
|---|---:|---|---:|---|---|
| husk | 7,177 | husk | 14 | archive | false |
| doc | 5,811 | doc | null | report | false |
| code | 2,473 | code | null | report | false |
| build_config | 1,821 | build_config | null | report | true |
| registry | 971 | registry | null | report | true |
| spec | 315 | spec | null | report | true |
| evidence | 203 | evidence | 90 | archive | false |
| vendor | 70 | vendor | null | report | true |
| generated | 28 | generated | null | report | false |
| ephemeral | 17 | ephemeral | 2 | archive | false |
| scratch | **0** | scratch | 0 | delete | false |

## Focus-family histogram (13,959)

| unit_class | count | share | action | protected | G030 reading |
|---|---:|---:|---|---|---|
| husk | **5,798** | 41.54% | archive | false | **Default residual**, not proof of darkness |
| doc | **5,721** | 40.98% | report | false | Markdown-retirement / docs lane |
| registry | 953 | 6.83% | report | true | Machine SSOT — no delete |
| build_config | 907 | 6.50% | report | true | Cargo.toml etc. — no delete |
| spec | 304 | 2.18% | report | true | Machine SSOT — no delete |
| evidence | 189 | 1.35% | archive | false | Provenance; archive/freeze only |
| vendor | 66 | 0.47% | report | true | No delete |
| ephemeral | 11 | 0.08% | archive | false | Smallest freeze/delete *candidate class* after dual proof |
| generated | 10 | 0.07% | report | false | Lifecycle-owned; no hand-edit |
| scratch | **0** | 0% | delete | false | No focus-family scratch shape on tip |

Protected focus rows (registry + build_config + spec + vendor) = **2,230**.  
Archive-action focus rows (husk + evidence + ephemeral) = **5,998**.  
Delete-action focus rows (scratch) = **0**.

## Focus husk density by top-level prefix

| prefix | husk focus count | note |
|---|---:|---|
| `oya/` | **4,729** | almost all non-md product/capability YAML+JSON (openslo, catalogs, cedar-adjacent shells) |
| `cloud/` | 461 | service YAML/JSON outside doc/spec rules |
| `ci/` | 74 | gate fixtures/policies not caught earlier |
| `data/` | 69 | |
| `comms/` | 53 | |
| `infra/` | 48 | includes live GitOps/ARC values shapes |
| other tops | 364 | os, marketplace, storage, workflow, gateway, iam, … |
| **total** | **5,798** | |

`oya/` husk extension split: yaml **4,103**, json **626**, toml **0**, md **0** (all `oya/*.md` classify as `doc` via suffix rule). Top second-segment husk densities include `intelligence` 326, `governance` 125, and many product shells at ~76–114 each — the same population G026 already marked non-code shells / APP_FACE_BIRTH_REQUIRED, **not** bulk-delete.

## Exact small classes (complete lists)

### Focus ephemeral (11) — committed session-state leakage under `docs/`

All hit `ephemeral-omc-state` (`contains "/.omc/state/"`):

- `docs/audit/initial-sweep-2026-06-06/.omc/state/sessions/6725dbb8-…/{last-tool-error-state,mission-state,pre-tool-advisory-throttle,subagent-tracking-state}.json` (×2 under audit + architecture)
- `docs/audit/initial-sweep-2026-06-06/synthesis/.omc/state/sessions/6725dbb8-…/pre-tool-advisory-throttle.json`
- `docs/decisions/.omc/state/last-tool-error.json`
- `docs/decisions/.omc/state/sessions/8f603fc7-…/session-started.json`

Non-focus ephemeral also present (jsonl / evidence): `.omc/ultragoal/friction-ledger.jsonl`, audit replay jsonl under the same sweep, `evidence/audit-chain.jsonl`, `registry/fixuptasks.jsonl`, `registry/governance-corpora/banned-primitives/command-log.v1.jsonl`. Those are outside the G030 focus family but matter if a freeze PR widens.

### Focus generated (10)

All under `docs/machine-readable/*.json` via `generated-machine-readable`. Cargo.lock rows are generated but mostly non-focus-extension for this histogram’s generated-focus count (locks are `.lock`, not in md/yaml/json/toml focus — except none of the locks are focus extensions). Focus generated = the ten machine-readable JSON faces only.

### Scratch

**Zero** tracked paths match any scratch rule on this tip. The delete-action class is empty; G030 cannot open with a scratch purge.

### Root authority markdown

| path | class | rule |
|---|---|---|
| `README.md` | doc | doc-md |
| `CLAUDE.md` | doc | doc-md |
| `AGENTS.md` | doc | doc-md |
| `HANDOFF.md` | doc | doc-md |

Root tool tomls classify as husk (`deny.toml`, `oya-ci.toml`, `oya-deps.toml`, `reindeer.toml`, `rustfmt.toml`) or build_config (`Cargo.toml`, `rust-toolchain.toml`). **`oya-ci.toml` as husk is a classifier residual, not a delete signal** — it is live machine policy consumed by the producer.

## Binding interpretation for G030

1. **Husk ≠ DARK_BUREAUCRACY.** 41.5% of the focus family falls through to default husk because the unit-class table is coarse (prefix/suffix/contains only). Live OpenSLO YAML, product catalogs, and infra values are husk under this table while remaining operationally load-bearing. Dual consumer/authority proof from G030-A still gates any deletion.
2. **TTL action `archive` is not `delete`.** Even a true husk row has `budget_days: 14` and `action: archive`. Scratch is the only `delete` class and it is empty on tip.
3. **Smallest honest freeze/delete candidate set today** is the **11 focus ephemeral** docs session-state files (plus sibling non-focus ephemeral if the PR scope widens), and only after:
   - authority proof they are not retained audit evidence (they sit under `docs/audit/` and `docs/decisions/`, so this is *not* automatic),
   - consumer proof no gate/workflow seeds them,
   - anti-vacuity before/after counts on the same tip class,
   - independent APPROVE.
4. **Do not open G030-D on `oya/` husk bulk.** That mass is G026 capability/app migration + OpenSLO ownership, not a corpus janitor sweep.
5. **Do not open on `docs/` mass.** `doc` is 5,721 focus rows; markdown-retirement (MPV2-0003) owns sequencing; root survival set must stay cardinality 4.
6. **Classifier improvement is a separate gate PR**, not a G030 deletion: e.g. promoting live `*.openslo.yaml`, root `oya-ci.toml`, and GitOps values out of husk into protected/report classes would *re-label* counts without deleting bytes. Re-label PRs need registry-drift + producer tests; they are not reduction.

## What G030-B deliberately did not do

- Did not run `oya-cloud-ci-accounting-registry-app` (scm-facts face absent on trunk by ADR-0613 design).
- Did not invent owner/justification/reachability.
- Did not hand-author any `*.generated.json`.
- Did not claim total-accounting RED on any path.
- Did not obtain independent APPROVE (review transports still fused/quota-blocked).

## Next slices (unchanged order, refined)

| Slice | Status after G030-B |
|---|---|
| G030-A baseline | done |
| G030-B histogram | **done (this file)** |
| G030-C | classify root authority + `specs/` + `registry/` as ROOT_AUTHORITY / MACHINE_SSOT with consumer citations — plan-only |
| G030-D | only if dual proof clears a **tiny** set (start from the 11 ephemeral docs paths, expect most to fail authority proof under audit retention) |
| G030-E | `oya/` husk joins G026 face birth / capability lanes |

## Non-claims

- Not a deletion PR and not a freeze PR.
- Not evidence that 5,798 husk files are unused.
- Not permission to treat TTL archive budgets as auto-delete.
- Not a substitute for CI producer materialization when admitting an accounting-facing change.
