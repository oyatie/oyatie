---
doc_class: Owner-ADR
owner: app/application
status: Accepted
date: 2026-08-29
inherits:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
---

# Application decisions in force

This file specializes ADR-0719 for `app/application/`. It records the face
boundary the product now keeps. It does not claim that the product's serving
path, its plugin surface, or its shell frontend have been reworked.

<current_state>

## Evidence at owner-law adoption

- `core/foundation` composes the platform kernels into one tenant-facing
  slice: onboarding, identity and tokens, data-use grants, capability
  registration and invocation under an autonomy ceiling, MCP gateway access,
  cost budgets, runs, steps, evidence, the outbox, and the object graph.
- `core/surface` holds the phase-invariant cloud product surface: the SKU
  taxonomy and the compute, storage, network, identity, region, billing,
  observability and FinOps surfaces a customer binds to.
- `facade/` retains the shell frontend and the SaaS plugin app.

</current_state>

<decisions>

## A face names what a crate is, not where it was first written

**achieves:** the layout law and the code agree, so a reader and the
admission gate reach the same conclusion about any crate in this product.

**origin:** every crate in this product was born in `facade/`, including a
crate named `surface-domain` and a composition layer with no server, no
async runtime and no configuration. The gate defines a `facade/` leaf as a
runnable service and requires `src/main.rs`; these crates have none, because
they are not services. The mismatch was invisible until a path changed.

**rule:** a crate that composes kernels or owns domain types lives in
`core/`. `facade/` is for crates that actually run: a binary with an entry
point. A crate is not placed by history, by its dependents, or by which face
already existed.

**ensure:** every `facade/` leaf in this product has `src/main.rs`, and no
`core/` leaf declares a binary; the admission gate refuses the alternative
on any changed path.

**overturn_when:** a recorded challenge shows a crate that is genuinely
runnable and genuinely a domain owner, and splitting it costs more than the
mismatch does.

## Invocation is a pipeline of named phases

**achieves:** a reviewer can see where a capability invocation is authorized,
where capacity is reserved, and where the provider is called, and can change
one without reading the others.

**origin:** the whole invocation path was one 949-line function. Every gate,
every audit, every settlement branch and the receipt shared one scope, so no
part of it could be reasoned about or moved on its own.

**rule:** invocation is an orchestrator over phases with explicit boundaries.
A phase takes and returns a named value; it does not reach into the caller's
locals. A gate denial writes its trail and returns; it does not fall through.

**ensure:** each phase compiles as its own module within the file budget, and
the invocation suites drive allow and deny through every gate.

**overturn_when:** measured evidence shows a phase boundary that cannot carry
a required ordering guarantee the single scope provided.

</decisions>
