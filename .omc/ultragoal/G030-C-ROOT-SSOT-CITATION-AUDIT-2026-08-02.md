# G030-C root authority and machine-SSOT citation audit — 2026-08-02

State: **PLANNING_ONLY — ROOT AUTHORITY PROVEN; MACHINE-SSOT CLASS PROTECTED BUT SEMANTIC COVERAGE PARTIAL**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
No policy edit, registry edit, deletion, freeze, push, or activation occurred.

## Result

G030-C separates three claims that must not be collapsed:

1. **Root authority liveness** — proven for the four root Markdown files.
2. **Class-level retention** — proven for all `specs/` and `registry/` focus rows by the committed unit-class + TTL policy (`protected: true`).
3. **Artifact-specific semantic consumption** — only partially proven. The affected-set policy names exact synthetic dependencies for **7** `specs/` paths and **3** `registry/` paths, not every file in those trees. Whole-tree accounting scans prove structural visibility, not that a domain consumer interprets each artifact.

Therefore:

- the four root Markdown files may be assigned `ROOT_AUTHORITY`;
- `specs/` and `registry/` may be assigned **POLICY_PROTECTED_MACHINE_ARTIFACT** immediately;
- only paths with direct consumer citations may be assigned `MACHINE_SSOT` without further evidence;
- no unseeded protected path becomes deletable merely because semantic coverage is incomplete.

## Root authority — four files, three independent citations

| Path | Markdown retirement policy | Root hub / operating authority | Repo-root gate | Disposition |
|---|---|---|---|---|
| `README.md` | `retention_rules.survive_at_root_as_pointer_hubs`; `rebaseline.root_survival_set` | root entry hub | allowed by `repo-root-hygiene/root-workspace-hygiene-policy.json` | `ROOT_AUTHORITY` |
| `CLAUDE.md` | same | authoritative project-rules source | allowed by root hygiene | `ROOT_AUTHORITY` |
| `AGENTS.md` | same | shared root operating contract | allowed by root hygiene | `ROOT_AUTHORITY` |
| `HANDOFF.md` | founder-authorized thin redirect exception; exact contract forbids plan/backlog/baseline/completion claims | `root-hub-pointers.json#entry_points.session_handoff` | allowed by root hygiene | `ROOT_AUTHORITY_THIN_EXCEPTION` |

Evidence in `specs/markdown-retirement-policy.json`:

- Accepted status and owner: `_meta.status = Accepted`, owner `council-architecture + axis-foundry`.
- Four-file survival set at lines 61–66 and 210–215.
- Founder exception contract for `HANDOFF.md` at lines 29–37.
- Stop condition at lines 217–221: no Markdown class retires before replacement schema, producer, consumers, parity check, and live `oya-ci-required` enforcement.
- Execution authority is `specs/masterplan.json#masterplan_v2.work_items[id=MPV2-0003]`, not the historical dates in this policy.

Evidence in `specs/root-hub-pointers.json`:

- `_meta.status = Accepted`; owner `founder + platform-governance`.
- `_meta.purpose` names README/CLAUDE/AGENTS as root authority/entry hubs and HANDOFF as founder thin exception.
- `agent_entry_surface_allowlist` is the canonical mandatory agent entry surface.

Evidence in `ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json`:

- `allowed_root_files` names the allowed root surface, including all four authority Markdown files.
- Markdown-retirement’s HANDOFF exception directly cites this policy and the baseline signoff.

Anti-vacuity: immutable-tree root probe returns exactly these four `.md` paths — no fifth root Markdown file is hidden by the classification.

## `specs/` and `registry/` class-level retention

G030-B producer-equivalent first-match classification yields:

| Prefix | Focus rows | unit_class result | TTL action | protected |
|---|---:|---|---|---|
| `registry/` | **816** | registry | report | true |
| `specs/` | **360** | spec **223** + registry-fixtures **137** | report | true |

The rule ordering is explicit:

- `registry-tree` classifies `registry/*` as `registry`;
- `registry-fixtures` runs before `spec-tree`, so `specs/fixtures/*` is `registry`;
- remaining `specs/*` is `spec`;
- TTL table marks both `registry` and `spec` protected with no expiry and action `report`.

This policy result blocks automatic archive/delete. It does **not** prove every row is a semantic SSOT.

## Direct affected-set semantic seed census

`ci/facade/affected-target-set/affected-set-policy.json.synthetic_dependencies` contains exact rows for:

### `specs/` — 7 exact seeds

1. `specs/artifact-profile-defaults.json`
2. `specs/decision-rights.json`
3. `specs/forbidden-operations.json`
4. `specs/product-protocol-contract.json`
5. `specs/api-contract-ssot-canonical.json`
6. `specs/root-hub-pointers.json`
7. `specs/markdown-retirement-policy.json`

`specs/markdown-retirement-policy.json` seeds `root//ci/facade/cross-artifact-agreement:ci-cross-artifact-agreement-gate`. `specs/root-hub-pointers.json` seeds nine gate targets. These exact edges support `MACHINE_SSOT` for those paths, subject to their unchanged semantic readers.

### `registry/` — 3 exact seeds

1. `registry/artifact-capabilities-registry.json`
2. `registry/dependency-rationales.json`
3. `registry/quality/lanes/lean-settings-drift.json`

The first seeds nine gate targets; dependency rationales seeds seven. These exact edges support `MACHINE_SSOT` for those paths.

### Important negative

`specs/masterplan.json` and `specs/capability-registry.json` are authoritative by the trusted project authority chain, but they have **no exact synthetic-dependency row** in the current affected-set policy. That is not evidence they are inert:

- masterplan is the sole live plan authority by root `CLAUDE.md` and root-hub pointer contract;
- capability registry is the closed layout authority by ADR-0562/ADR-0615 and root project rules;
- unknown/unmapped changes escalate to FULL rather than select nothing.

It is, however, evidence that exact semantic affected-set routing is incomplete. Do not describe all `specs/` as directly graph-wired.

## Structural consumers that cover the full tracked universe

The following machinery sees all tracked paths structurally:

- `artifact-inventory-registry` receives the sorted tracked-path universe from scm-facts and assigns unit class, owner, reachability, justification, and TTL.
- `artifact-accountability` evaluates unaccounted/unowned/unjustified/unreachable/no-TTL/scratch findings.
- `corpus-index-coverage` and stale-artifact machinery consume declared inventory/facts.
- affected-set defaults to FULL for unmapped paths rather than declaring them inert.

These prove that a path is governed/accounted. They do not prove a domain parser reads its content. G030 must preserve that distinction.

## Refined disposition vocabulary

| Disposition | Minimum proof | Mutation rule |
|---|---|---|
| `ROOT_AUTHORITY` | survival policy + hub/contract + root gate | no delete |
| `MACHINE_SSOT` | policy protection + direct semantic consumer/authority citation | no delete; migrate atomically |
| `POLICY_PROTECTED_MACHINE_ARTIFACT` | unit-class/TTL protection but semantic consumer not yet cited | no delete; investigate consumer |
| `GRAPH_WIRED_INPUT` | Buck/affected-set/gate edge, even if not global authority | no delete until consumer rewrite |
| `STRUCTURALLY_ACCOUNTED_ONLY` | inventory/accountability sees path, no semantic reader proven | no delete; candidate for liveness investigation |
| `DARK_BUREAUCRACY` | dual negative consumer + authority proof | freeze/delete candidate after review |

No path moves directly from `POLICY_PROTECTED_MACHINE_ARTIFACT` to `DARK_BUREAUCRACY`; first, owner/authority must explicitly declassify or replace it.

## Smallest next work

1. **Do not edit policy yet.** Existing `spec`/`registry` protection is safe and non-destructive.
2. Build a **read-only semantic-consumer census** for the 1,176 unique focus rows under `specs/` + `registry/` (360 + 816):
   - exact affected-set synthetic edge;
   - Buck `srcs` / `$(location)` edge;
   - Rust path literal / parser call;
   - root-hub authority pointer;
   - owner/reachability row when CI materialization is available.
3. Report counts in `MACHINE_SSOT` / `GRAPH_WIRED_INPUT` / `POLICY_PROTECTED_MACHINE_ARTIFACT`; do not mutate files.
4. Only separately propose missing affected-set exact seeds where a measured semantic reader exists. Do not seed paths merely to make the census green.

## Non-claims

- Not a claim that every `specs/` or `registry/` file is semantically live.
- Not a claim that lack of an exact synthetic dependency means unused.
- Not permission to weaken `protected: true`.
- Not a second authority registry.
- Not independent APPROVE; review transports remain unavailable.
