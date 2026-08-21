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

Cedar fragments across this repository open with

```cedar
forbid (principal, action, resource);
```

The intent is "default deny". Cedar is **already** default-deny — an absent permit denies — so the
line adds no denial. What it does add is a rule that, because **forbid unconditionally overrides
permit**, denies *everything* in whatever `PolicySet` it ends up in.

An earlier draft of this page said that was "harmless while each fragment is evaluated as its own
policy store". **That was wrong, and an audit caught it.** A bare forbid nullifies the permits in its
OWN file, not merely in a bundle — and **176 of the 176 affected files also contain `permit`
statements**, so every one of them, loaded standalone, grants nothing at all. Concatenation is not
the precondition for the hazard; it only widens the blast radius from one file to a whole bundle.

Measured against `cedar-policy` 4.12:

```
permit alone                   -> Allow
permit + bare forbid (bundled) -> Deny
```

So `policy/policy/*.cedar` is **closed under concatenation**: `policies.cedar` is their concatenation
plus a header and one separator comment per fragment, and it decides identically. (Comment-stripped
the two are byte-identical; an earlier revision claimed "byte-exact", which the separators make
false.) `CONFORMANCE.md` asserts the property that matters by running the whole suite against the
concatenated bundle rather than against the fragments.

### The same shape elsewhere in the tree

Measured at `origin/dev@7f8a5a075` over the 448 tracked files matching `*/policy/*.cedar` and
`*/cedar/*.cedar`, with comments stripped:

| form matched | files |
|---|---|
| bare forbid written on ONE line | **63** |
| bare forbid in ANY form, including the multi-line shape | **176** |
| of those 176, files that ALSO contain a `permit` | **176** |

The multi-line shape is the dominant one in this repo:

```cedar
forbid (
  principal,
  action,
  resource
);
```

so a single-line grep understates the finding by ~2.8x. **A first pass of this page reported "64 of
448" across "six capabilities" and listed `flags/` as an owner. All three were wrong**: the count was
line-anchored, `flags/` has zero, and the true spread is **19 capability roots** — `audit` 39,
`oya/intelligence` 30, `comms` 15, `workflow` 12, `oya/global-trade` 10, `console` 9,
`oya/community` 8, `gateway` 6, `marketplace` 6, `oya/payments` 6, `storage` 5, and 4 or fewer each
in `data`, `observability`, `oya/application`, `oya/notes`, `oya/sheets`, `oya/slides`,
`oya/translate`, `iam`.

**This is a latent hazard, not a live defect — but not for the reason first given.** It is latent
because *nothing loads these files*: no Rust `include_str!` of a `.cedar` targets one of the 176, and
no code enumerates a capability's `policy/` directory into a `PolicySet`. That is a weaker and more
fragile guarantee than "each file is fine standalone", which is false. All 68 hand-authored
`<cap>/cedar/policies.cedar` bundles are clean — zero bare forbids — so the consolidated path is not
affected today.

Not checked: non-Rust loaders (a Buck rule, a container build step, a ConfigMap assembly). If one
exists, the hazard is live rather than latent for whatever it loads.

Fixing those files is outside this capability's envelope (`policy/**`; ADR-0711 §D-2 for the
fail-closed adjunct rule, §D-9 for ownership = path = integ scope) and is not attempted here. It is
recorded in `policy/PROMOTION.md` §5.

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
