# artifact-inventory-registry

The producer of `accounting-registry.generated.json` + companion faces for the
`[cloud-ci-total-accounting]` firewall (GATE-2). It resolves, per tracked path, its
**owner** (`OWNERS`), **justification** (an ADR under `docs/decisions/` that names the exact
path token), and **reachability** (masterplan / root-hub-pointers / DOC-CATALOG / the reviewed
reachability-registry / workspace Cargo members). A path that is unowned, unjustified, or
unreachable is a firewall finding; the merge-base baseline grandfathers pre-existing debt, so
any **newly added** path that trips a code is a regression that REDs the gate.

Do not hand-edit the generated faces — the registry-drift gate makes committed != regenerated RED.

## Author-side pre-push check (avoid the `unjustified regressions` surprise)

Adding tracked files that no ADR justifies REDs `[cloud-ci-total-accounting]` in CI — historically
discoverable only after materializing scm-facts faces and running the firewall. This binary
answers the same question locally, BEFORE push, with **no materialized scm-facts face required**
(the added set IS the tracked-path universe), reusing the SAME resolvers + face-builder + the
firewall's own evaluator (so its verdict cannot drift from CI's):

```
# 1. Check the files your branch ADDS before you push:
buck2 run //ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin -- \
    --check-diff origin/dev
#    ...or name paths explicitly:
buck2 run //ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin -- \
    --check-paths ci/facade/my-new-gate/src/lib.rs docs/foo.md

# 2. Push once the check is clean.
```

PR bodies come from the canonical repository template and are evaluated by
independent review. ADR-0716 retired the local PR-body scaffold/check binary;
this inventory check is optional local feedback, not merge authority.

Per added path the check reports `reachable?` (which resolver) and `justified?` (which ADR), and
if a path would RED the firewall it names the exact fix. For `unjustified` that is: *add the
exact path token `<path>` to the governing ADR under `docs/decisions/`* — precedent **ADR-0515**
for `ci/` gate surfaces, **ADR-0251** for compliance artifacts. Exit code is `0` when clean and
`2` when at least one added path would RED (distinct from `1`, a usage/IO error).

`unowned` is intentionally out of scope for the pre-push check: owner resolution is full-tree (the
granting up-tree `OWNERS` file is usually not in the added set), so the full gate owns it.
