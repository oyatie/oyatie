# Container image convention (ADR-0146 + ADR-0515)

Authority: ADR-0146 keeps `gcr.io/distroless/static-debian12:nonroot` as the
canonical runtime base. ADR-0515 changes image assembly to Buck2-native OCI:
source → Buck2 target → OCI layout → registry push. Dockerfiles and Cargo build
commands are not active CI/CD/build authority.

## Canonical Buck2-native pattern

```python
# cloud/<service>/iac/oci/BUCK
oci_image(
    name = "<service>-oci",
    base = ":distroless-base",
    layers = [":<service>-binary-layer"],
    entrypoint = ["/usr/local/bin/<service>"],
    user = "65532:65532",
)
```

The binary layer consumes the service's Buck2 binary target. The push path uses
`tools/oci/push-oci-image.py` against the OCI layout emitted by Buck2.

## Required directives

| Directive | Required value | Validator |
| --- | --- | --- |
| Base image | `gcr.io/distroless/static-debian12:nonroot` or ADR-approved carve-out | `oya gate validate container-base-image` as local/bridge evidence; cloud-ci owns required status |
| Runtime user | `65532:65532` (or bare `65532`) | same |
| Assembly | Buck2-native OCI target | `specs/buck2-authority-policy.json` + `//:buck2-authority-policy-check` |
| Registry push | `tools/oci/push-oci-image.py` consuming Buck2 output | script/static policy |

## Production release optimization exception

Cargo may be used only for production release image/binary optimization evidence:
release profile selection, target triple, binary-size comparison, allocator or codegen
knob under test, commit SHA, and an explicit non-claim label that the run is not CI
merge authority. The exception is documented in `specs/buck2-authority-policy.json`
and grounded in official Rust release-profile/codegen/allocator docs.

## How to adopt for a new service

1. Put OCI assembly under `cloud/<service>/iac/oci/BUCK` or `oya/<service>/iac/oci/BUCK`.
2. Consume the service's Buck2 binary target as the layer input.
3. Wire build/push scripts to `buck2 build --show-full-simple-output //<path>:<service>-oci`
   and then `tools/oci/push-oci-image.py`.
4. Add the target to the Buck2 authority policy or service registry when it becomes a
   required lane.

## Why distroless and not scratch

`scratch` removes the CA-cert bundle, `/etc/passwd`, and timezone data. Most Rust
services require at least one of those. ADR-0146 documents the alternative analysis;
ADR-0515 keeps distroless static and moves assembly to Buck2-native OCI.

## CI/CD lane

The active lane is Buck2-only:

```sh
buck2 build //:buck2-authority-policy-check
buck2 build //cloud/<service>/iac/oci:<service>-oci
```

A live branch-protected claim still requires `oya-ci-required` from cloud-ci/oya-ci;
local gates and checked-in targets are evidence, not Phase-0 exit authority.
