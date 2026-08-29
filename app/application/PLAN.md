---
doc_class: Owner-PLAN
owner: app/application
status: Active
date: 2026-08-29
---

# Application remaining work

<baseline>

## What has landed

- The product has a `core/` face. `facade/surface-domain` became
  `core/surface` and `facade/application-app` became `core/foundation`,
  because neither is a runnable service and the layout law defines a facade
  as one.
- The capability-invocation path is an orchestrator over named phases -
  reserve, start, complete - with gate denials lifted out of the hot path and
  the five that differed only in data collapsed onto one helper.
- Behaviour is unchanged throughout: 55 foundation tests and 5 surface tests
  at parity, verified after each extraction.

</baseline>

<remaining>

## Retire the dependency-rename aliases

`intelligence/core/api` reaches `core/foundation` through a Cargo
dependency-rename alias so that its own sources did not have to change in
this lane. The alias is migration debt: drop it and spell the crate by its
real name when that crate is next opened for its own reasons.

## Give the remaining facades an entry point or a face

`facade/application-shell-frontend` and `facade/saas-plugin-app` have not
been examined against the same test the two moved crates failed. Either they
run and should carry `src/main.rs`, or they do not and belong in `core/`.
This is unresolved, not decided.

## Restore the Buck graph for this product

`app/application/facade/application-app/BUCK` was deleted with its crate
rather than carried, because at 609 lines it could not be edited under the
file budget and a crate root admits no `.bzl` to split it into. The product
has no Buck targets until they are authored fresh. Buck is not on the merge
path, so this blocks nothing, but the gap is real.

## Reconcile the surface with its consumers

`core/surface` has no consumers. Either the product surface it models is not
yet wired to the serving path, or it duplicates something that is. That
question is open.

</remaining>
