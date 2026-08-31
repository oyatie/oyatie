---
doc_class: Owner-PLAN
owner: policy
status: Active
date: 2026-08-29
---

# Policy remaining work

<baseline>

## What has landed

- `policy/core/cedar-domain` — the Cedar-shaped policy kernel and the ReBAC
  vocabulary, extracted whole from `iam/core/policy-cedar-domain` with its
  behavior unchanged and its 45 tests at parity.
- Consumers in `app/application`, `iam/core/app-control`, and
  `iam/ports/policy-cedar-api` reach the crate through a behaviour-free
  re-export shim that keeps the `iam-policy-cedar-domain` package name at
  its old path; their sources are untouched.

</baseline>

<remaining>

## Delete the shim, and with it a rule this file already breaks

`ADR.md` states that a capability needing a decision depends on `policy/`,
never on `iam/`. The three consumers above do exactly what that rule
forbids: they reach a policy type through an `iam/` crate. The shim is the
reason, and it is transitional by intent but has no forcing function - no
gate, no deadline, only a description.

Each consumer drops the shim and depends on `policy-cedar-domain` directly
the next time it is opened for its own reasons. The shim is deleted with
the last of them, and the rule stops being aspirational.

## Extract the decision plane

Move the remaining authorization crates out of `iam/`, lowest blast radius
first: the runnable service and the publish API, which nothing depends on;
then the decision kernels, the bundle-store adapter, and the Cedar engine
adapter, whose consumers are `tenancy/adapters/tenant-lifecycle-authz-pdp`
and `iam/facade/tenant-rbac-app`.

The duplicated Cedar corpora consolidate to `policy/cedar/`. The decision
protobuf moves to `policy/facade/proto/policy/decision/v1/` under the package
the path grammar requires; this renames the wire package, which is accepted
because the contract is internal-only and has no out-of-repo consumer.

The workload-identity Cedar gate stays with identity per ADR-0631. The shared
platform contract kernel stays shared; splitting it is separate work.

## Join Cedar to the relationship graph

Bind `(object_type, relation)` to a `UsersetRewrite` through a namespace
configuration. Evaluate that tree against tuple-store reads at a pinned
snapshot, with cycle detection and a depth bound, and materialize the result
as entity-parent edges for the existing decision port. Land an in-memory
tuple store as the conformance substrate and a Postgres store whose
consistency tokens derive from commit order.

## Complete the relationship surface

Add reverse enumeration and change streaming. Enumeration reverse-walks the
negation-free part of a rewrite to candidates and then runs a full check per
candidate, because difference is not reverse-traversable; it is bounded by an
explicit limit and resumes by continuation token. Streaming reads an ordered
changelog, which makes the consistency token load-bearing rather than an
opaque echo.

</remaining>
