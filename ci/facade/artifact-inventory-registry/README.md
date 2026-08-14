# artifact-inventory-registry

The producer of `accounting-registry.generated.json` + companion faces for the
conformance floor. It resolves, per tracked path, its **owner** (`OWNERS`),
**justification** (an ADR under `docs/decisions/` that names the exact path token), and
**reachability** (masterplan / root-hub-pointers / DOC-CATALOG / the reviewed
reachability-registry / workspace Cargo members). The merge-base frozen baseline grandfathers
pre-existing debt; the registry face is also the input for the registry-drift byte-diff and the
baseline ratchet.

The `[cloud-ci-total-accounting]` admission gate, the born-accounting pre-push check mode
(`--check-paths` / `--check-diff`), and the catalog-liveness / manifest-hygiene face collection
are retired with ADR-0718: the registry stays as the durable accounting inventory, but its
verdicts no longer ride a dedicated admission gate.

Do not hand-edit the generated faces — the registry-drift gate makes committed != regenerated RED.

## Local bridges (never merge authority)

```
cargo run -p ci-artifact-inventory-registry --bin oya-cloud-ci-accounting-registry-app -- \
    --fix-owners <dir>=<owner>          # write a schema-valid OWNERS file
cargo run -p ci-artifact-inventory-registry --bin oya-cloud-ci-accounting-registry-app -- \
    --fix-reachability <prefix>=<anchor> # append a reviewed reachability-registry entry
```

## slo-coverage face

The `slo-coverage` face is one row per tracked `*.openslo.yaml` envelope (ADR-0718), keyed by
repo-relative path with the envelope's `metadata.name` as the declaration; the retired
`registry/catalog/*.yaml` mirror is gone.
