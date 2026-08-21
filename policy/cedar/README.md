---
doc_class: Reference
shape: Reference
length_cap: 250
microservice: policy
related_adrs:
  - ADR-0243
  - ADR-0294
inbound_citations:
  - policy/README.md
---

# Two authoring rules this bundle follows, and why

Both were paid for by an adversarial review that broke the first version of these fragments.

## 1. No fragment opens with a bare `forbid`

Six of this repository's capabilities open Cedar fragments with

```cedar
forbid (principal, action, resource);
```

The intent is "default deny". Cedar is **already** default-deny — an absent permit denies — so the
line adds no denial. What it does add is a rule that, because **forbid unconditionally overrides
permit**, denies *everything* in whatever `PolicySet` it ends up in.

Harmless while each fragment is its own policy store. Not harmless the moment a loader concatenates
a capability's fragments into one bundle — which is exactly what this capability's C0 face does when
it distributes a snapshot. Measured against `cedar-policy` 4.12:

```
permit alone                   -> Allow
permit + bare forbid (bundled) -> Deny
```

So `policy/policy/*.cedar` is **closed under concatenation**: `policies.cedar` is their byte-exact
concatenation and decides identically. `CONFORMANCE.md` asserts this by running the whole suite
against the concatenated bundle rather than the fragments.

### The same shape elsewhere in the tree

64 of 448 tracked `.cedar` fragments carry a bare `forbid (principal, action, resource);`:
`audit/policy/` (35), `oya/global-trade/policy/` (10), `gateway/connector/policy/` (6),
`marketplace/**/policy/` (6), `oya/slides/policy/` (4), `iam/identity/policy/` (2), `flags/` (1).

**This is a latent hazard, not a live defect.** No loader concatenates those fragments today, and
each capability's hand-authored `<cap>/cedar/policies.cedar` is a separate document without the bare
forbid — `audit/cedar/policies.cedar` holds 9 policies and zero. The hazard is realised by the first
bundle-distribution path that concatenates. Fixing those files is outside this capability's envelope
(`policy/**`, ADR-0711 D-9) and is not attempted here; it is recorded in `policy/PROMOTION.md` §5.

## 2. Every safety bound appears twice — and the forbid is `has`-guarded

Each bound (credential age, soak window, snapshot staleness, signature, cell, attestation) is written
in **two** places: inside the permit that grants the action, and again in a forbid.

That is not redundancy. It closes two different failure modes, and neither closes the other:

- **A permit that forgets the bound.** Then only the forbid stands between the request and an Allow.
  The first version of `runtime-grants.cedar` bounded staleness inside the permit alone, so any
  future permit omitting it would have served stale state.
- **A request that omits the attribute.** Cedar drops a policy that errors during evaluation, and
  dropping a *forbid* leaves the permit standing — the guard deletes itself exactly when it is
  needed. Measured on v1.0.0: an `ActivatePolicy` request with no `soak_elapsed_seconds` in context
  returned **Allow**, with the diagnostic `SKIPPED-ON-ERROR: policy8 record does not have the
  attribute soak_elapsed_seconds`. The ADR-0294 soak window, documented as having no break-glass,
  was bypassed by omitting a field.

The structural fix for the second is the `has` operator, so the forbid is *false* rather than
*erroring* when the attribute is absent:

```cedar
forbid (principal, action, resource)
unless {
  context has token_age_seconds && context.token_age_seconds <= 900
};
```

`CONFORMANCE.md` proves both halves: cases `S1a`–`S1c` omit the attribute entirely, and a
**bound-blind permit fixture** injects permits that deliberately forget each bound, so that every
forbid is actually reached by at least one case instead of being shadowed by the permits.

## What these rules do NOT buy

`step_up_class` is an attribute the caller asserts, not membership, because step-up is a property of
a session rather than of an identity. If the PEP populates the entity slice, it can claim any
step-up class. The mitigation is that everything carrying *authority* — tenant, role, cell, policy
authorship — is entity membership or an entity reference, so a compromised slice cannot invent a
tenant or a role. Step-up remains a claim, and that is a deliberate, bounded residual.
