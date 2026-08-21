---
doc_class: Reference
shape: Reference
length_cap: 200
microservice: policy
related_adrs:
  - ADR-0243
  - ADR-0294
inbound_citations:
  - policy/README.md
---

# Why no fragment here opens with a bare `forbid`

Six of this repository's capabilities open Cedar fragments with

```cedar
forbid (principal, action, resource);
```

The intent is "default deny". Cedar is **already** default-deny — an absent permit denies — so the
line adds no denial. What it does add is a rule that, because **forbid unconditionally overrides
permit**, denies *everything* in whatever `PolicySet` it ends up in.

That is harmless while each fragment is evaluated as its own policy store. It stops being harmless
the moment a loader concatenates a capability's fragments into one bundle — which is exactly what
this capability's C0 face does when it distributes a snapshot.

Measured against `cedar-policy` 4.12 (the workspace lock), evaluating
`OyaPolicy::Action::"ReadDecisionLog"`:

```
permit alone                   -> Allow
permit + bare forbid (bundled) -> Deny
```

So the fragments in `policy/policy/` are **closed under concatenation**: `policies.cedar` is their
byte-exact concatenation and decides identically to the fragments evaluated separately. That property
is asserted by the conformance suite in `CONFORMANCE.md`, which runs against the concatenated bundle.

## The same shape elsewhere in the tree

64 of 448 tracked `.cedar` fragments carry a bare `forbid (principal, action, resource);`:

```
$ for f in $(git ls-files '*/policy/*.cedar' '*/cedar/*.cedar'); do
    grep -qE '^\s*forbid\s*\(\s*principal\s*,\s*action\s*,\s*resource\s*\)\s*;' "$f" && echo "$f"; done | wc -l
64
```

They span `audit/policy/` (35), `oya/global-trade/policy/` (10), `gateway/connector/policy/` (6),
`marketplace/**/policy/` (6), `oya/slides/policy/` (4), `iam/identity/policy/` (2), `flags/` (1).

**This is a latent hazard, not a live defect.** No loader concatenates those fragments today, and each
capability's hand-authored `<cap>/cedar/policies.cedar` is a separate document that does *not* carry
the bare forbid — `audit/cedar/policies.cedar` holds 9 policies and zero bare forbids. The hazard is
realised only by a future bundle-distribution path that concatenates.

Fixing those 64 files is **outside this capability's envelope** (`policy/**`, ADR-0711 D-9) and is not
attempted here. It is recorded in `policy/PROMOTION.md` as a finding for the owning capabilities.
