# Post-Cutover Program: Substrate → Colocation → First Real Service

## Problem Statement
How might we move Oyatie from a spec-saturated, code-starved platform (e.g. `identity`: 272 design
files, 0 crates) to one that **runs its own workload as a tenant of its own cloud** — without the
governance machinery seducing us into specifying more instead of building?

## Context / Diagnosis
The cutover PRs (1/2/3) and all prior waves sharpened the *tool* (97 gates, Jenkins CI, clean
doctrine). The product is barely built: the microservice catalog is mostly "Not authored yet," and
the one foundational service (`identity`) is fully specified and entirely unimplemented. The binding
constraint is not direction or design — it is the pull toward more specification. Every merge today
also still requires the `enforce_admins`-toggle admin hack because nothing posts the 15 required
checks. Both facts point the same way: **make the substrate real, then write code, not specs.**

## Recommended Direction (execute in order, after PR-1/2/3)

**T1 — Substrate-realization (kill the admin-merge seam).** Stand up GitHub (interim) on the k3s
farm; wire Jenkins → GitHub Commit Status API; prove the required checks post green against a
**mirror** first; *then* flip primary off GitHub (ADR-0247 post-bootstrap). The cutover is the last,
deliberate, reversible step — never flip the host before checks demonstrably post green. Exit: a
reviewed PR merges on green checks alone, no admin override.

**T2 — Crate colocation (make absorption true in the filesystem).** Move the 116 `intelligence-*`
crates physically under `microservices/intelligence/` per ADR-0131/0357 vertical-slice nesting
(task #10). Catalog records move with crates (architecture-boundaries will fail otherwise). Pure
structure; no behavior change. Exit: `presubmit` green with code colocated; "absorbed into
Intelligence" is true on disk, not just in docs.

**T3 — First real service: workload identity (NOT enterprise IAM).** Build the minimum `identity-*`
crates that implement the already-specified principal + Cedar authz gate
(`IP-journey-j85/j80/j76-principal-and-authz-gate`): an OIDC issuer/verifier, a `principal` model,
and token issuance gated by Cedar default-deny. Deploy it on the farm via the GitOps pipeline. Do
**not** build SCIM/SAML/passkey/step-up — those are over-specified already and out of scope for the
first proof.

## First "it's real" milestone (intermediate, reachable)
The platform's own **Intelligence** service authenticates to a running **identity** service as
principal `oyatie:<service>`, receives a Cedar-permitted token, performs one authorized action, and
the exchange is deployed on the k3s farm with ADR-0263 audit evidence. This is **self-tenant dogfood
in miniature** — the same loop as the north star, scoped to one principal and one resource — in
weeks, not months. The full "Oyatie runs as a tenant of its own cloud" north star is the direction,
not the first deliverable.

## Key Assumptions to Validate
- [ ] The 272 identity design files are *consistent enough* to implement from without a re-spec pass
      — validate by extracting the principal/authz-gate contract from the IP-journeys before coding.
- [ ] Jenkins can post GitHub commit-statuses with the existing shared-library lane — validate
      against a GitHub mirror before any host flip.
- [ ] Colocating crates won't break the 999-edge architecture-boundaries graph — validate by moving
      one crate + its catalog record first, gate, then bulk-move.
- [ ] `specs/tenant-model.json` already defines the `oyatie:` self-tenant principal shape — validate
      it covers workload (service) principals, not just human/customer tenants.

## Minimum scope (the first proof)
**In:** GitHub + Jenkins commit-status (T1); intelligence crate colocation (T2); `identity-*`
OIDC issuer/verifier + principal + Cedar authz-gate; one deployed service-to-service authenticated
token exchange with audit evidence.
**Out:** SCIM, SAML, passkeys, step-up auth, account recovery, human-facing login UI, multi-region,
the full GitHub→GitHub data migration (deferred to its own deliberate cutover after commit-status
is proven).

## Not Doing (and Why)
- **More identity specs** — already 272 files; the gap is code, not design. Re-speccing is the trap.
- **Enterprise IAM surface (SCIM/SAML/passkey) first** — not needed for self-tenant dogfood; it's the
  hard 80% that doesn't prove the loop.
- **Full self-tenant dogfood as the first milestone** — it needs identity + tenant + deploy + obs all
  real; pick the miniature loop first or stall for months.
- **Flipping the repo host before checks post green** — trades one bootstrap hack for a worse one.
- **A merge queue** — ADR-0363 §3: auto-merge + required checks suffice at current volume.

## Open Questions
- Does `identity-*` reuse an existing OIDC library from the approved allowlist, or is there a
  blessed crate? (dependency-seam check before coding.)
- Where does the Cedar policy engine live — its own crate, or inside identity? (layering decision.)
- Is the k3s farm's ArgoCD wired to deploy a *new* service today, or only the existing demo lanes?
