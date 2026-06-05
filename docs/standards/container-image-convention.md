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

The binary layer consumes the service's Buck2 binary target. The durable push
path is a Rust/Buck2 release-conveyor target against the OCI layout emitted by
Buck2. Legacy Python OCI helpers are migration backlog only; they are not CI,
CD, or merge authority.

## Required directives

| Directive | Required value | Validator |
| --- | --- | --- |
| Base image | `gcr.io/distroless/static-debian12:nonroot` or ADR-approved carve-out | Rust/Buck2 container-base-image check plus trusted Prow/Kubernetes-native `oya-ci-required` |
| Runtime user | `65532:65532` (or bare `65532`) | same |
| Assembly | Buck2-native OCI target | `specs/buck2-authority-policy.json` + `//:buck2-authority-policy-check` |
| Registry push | Rust/Buck2 release-conveyor target consuming Buck2 OCI output | conveyor/static policy |

## Production release optimization exception

Cargo may be used only for production release image/binary optimization evidence:
release profile selection, target triple, binary-size comparison, allocator or codegen
knob under test, commit SHA, and an explicit non-claim label that the run is not CI
merge authority. The exception is documented in `specs/buck2-authority-policy.json`
and grounded in official Rust release-profile/codegen/allocator docs.

## How to adopt for a new service

1. Put OCI assembly under `cloud/<service>/iac/oci/BUCK` or `oya/<service>/iac/oci/BUCK`.
2. Consume the service's Buck2 binary target as the layer input.
3. Wire release-conveyor push through a Rust/Buck2 target that consumes
   `buck2 build --show-full-simple-output //<path>:<service>-oci`.
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
