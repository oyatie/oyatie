---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-tenancy-substrate-stable
impl_plan_id: IP-015-legacy-crates-migration
status: pending
owner: axis-tenancy
acceptance_lanes: [buck2-check, buck2-test, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: Migrate legacy crates to microservices/tenancy/ flat layout

## Intent

Migrate existing `crates/oya-tenancy-{kernel,domain,api}` → `microservices/tenancy/src/crates/oya-tenancy-tenant-lifecycle-{kernel,domain,api}` per ADR-0131 + ADR-0105 + ADR-0106 (`api` rename rules). Preserve git history via `git mv`. M01-P01-IP-001 (Data Use Boundary) precedent — these existing crates are already RLS-correct + stable per the M01-P01 closeout.

## ChangeSet boundary

3 crates physically moved (preserving content + git history). Naming converted per ADR-0106 (`application` already absent; legacy crates use `kernel` + `domain` + `api`; only path move needed). New target crates from IP-002–IP-007 absorb relevant code; legacy crates retained read-only with workspace compatibility patch-redirects during transition.

## Concrete File Targets

| Action | Path |
|---|---|
| `git mv crates/oya-tenancy-kernel microservices/tenancy/src/crates/oya-tenancy-tenant-lifecycle-kernel` | move |
| `git mv crates/oya-tenancy-domain microservices/tenancy/src/crates/oya-tenancy-tenant-lifecycle-domain` | move |
| `git mv crates/oya-tenancy-api microservices/tenancy/src/crates/oya-tenancy-tenant-lifecycle-api` | move |
| Update `Cargo.toml` workspace members for new paths | update |
| Update every importing crate's `Cargo.toml` to point to new paths | update |
| Remove stale legacy paths from workspace exclude list | update |
| Update catalog rows for moved crates | update |

## Migration Sequence

1. **Pre-migration check**: ensure IP-002 has authored the target `oya-tenancy-tenant-lifecycle-kernel` shape; resolve any naming conflicts via re-export adapter in IP-002.
2. **Execute `git mv`** for all 3 crates; verify diff is rename-only.
3. **Update workspace Cargo.toml**: remove old paths; add new paths; verify no duplicates.
4. **Update consumers**: every crate that imported `oya-tenancy-kernel` now imports `oya-tenancy-tenant-lifecycle-kernel`. Use Buck2/Prow query/check evidence plus AST-search to discover all importers; bulk-update with `sed -i` (verified carefully).
5. **Verify Buck2 build/check evidence clean** across workspace.
6. **Verify tests pass** (`buck2 test //...`).
7. **Verify LEAN lanes green** (`per-microservice-layout`).

## Acceptance Gates

```bash
buck2 build //:repo-hygiene-automation-check # native Buck2/Prow workspace check evidence
buck2 build //:repo-hygiene-automation-check # Buck2/Prow workspace build evidence
buck2 test //... # native Buck2/Prow workspace test evidence
buck2 build //:repo-hygiene-automation-check # Buck2/Prow native gate evidence for per-microservice-layout --microservice tenancy
git log --follow microservices/tenancy/src/crates/oya-tenancy-tenant-lifecycle-kernel/src/lib.rs | head -20  # verify history preserved
```

## Test Plan

- All existing tenancy tests pass post-move (no functionality change).
- LEAN per-microservice-layout green.
- Git history preserved across move (`git log --follow` works).

## Halt Conditions

- If any importer cannot resolve new path: emergency-merge stop; resolve before commit.
- If Buck2 build/check evidence fails post-move: revert; investigate.
- If git history disrupted: revert; use `git mv` exclusively (no copy+delete).

## Next IP

(end of P01 — exit gate is `per-microservice-layout` green + all 15 IPs merged)

## References

- ADR-0131: Per-microservice flat layout (target location).
- ADR-0105: 13-layer enum (target shape).
- ADR-0106: `application` → `usecase` rename (legacy crates use `application`-free shape; no rename impact).
- M01-P01-IP-001 evidence: `oya-tenancy-kernel` stable row-isolation shipped + approved.
