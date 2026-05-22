---
doc_status: published
---

# Checklist: Build-vs-Buy Decision

> **When:** New external dependency considered for adoption (Cargo crate / npm package / container image / managed service / SaaS as system-of-record).
> **Owner:** Owning axis + `council-architecture` review + `ops-security` for licensing.
> **Validator:** `oya-governance-build-vs-buy` lane + ADR-0014 enforcement.

---

## 0. Pre-flight

1. ☐ Confirm need is real — feature must ship; no in-house surface covers it
2. ☐ Confirm not kernel-grade-allowed (axum / tokio / serde / rustls / Postgres driver / kernel — these don't need this checklist)

## 1. Per-axis matrix lookup

3. ☐ Look up the dep in [ADR-0014 build-vs-buy matrix](../decisions/ADR-0014-build-vs-buy-policy.md) per the consuming axis
4. ☐ Classify: **in-house obligatory** / **external acceptable** / **requires ADR review**

## 2. License gate (per ADR-0013)

5. ☐ Identify license (cargo-deny / pnpm audit / SBOM scan)
6. ☐ Allowed: Apache-2 / MIT / BSD-2/3 / ISC / 0BSD / MPL-2 — proceed
7. ☐ Forbidden: AGPL / GPL — STOP unless dev-only carve-out
8. ☐ Requires-review: LGPL / SSPL / BUSL / Elastic / RSAL / TSL / Confluent / AWS-FSL / Commons Clause — open ADR + legal review

## 3. Maturity classification

9. ☐ Maturity tier:
   - **kernel-grade**: axum-class — adopt freely
   - **mature**: ≥ 3 years stable; widely adopted; active maintenance — adopt with port boundary
   - **maturing**: stable but evolving — adopt only if no kernel-grade alternative; replacement-plan documented
   - **experimental**: NEVER in product runtime; dev-only OK with carve-out

## 4. Isolation boundary

10. ☐ Port-isolated (behind a Rust trait boundary) — preferred
11. ☐ Embedded (linked into product crate) — only kernel-grade or with explicit ADR
12. ☐ System-of-record (external SaaS as authoritative) — FORBIDDEN for cross-axis contracts; per-axis only with ADR

## 5. Replacement plan

13. ☐ In-house alternative considered — yes / no
14. ☐ If yes: estimated effort + replacement criteria + ETA
15. ☐ If no: rationale (why in-house not feasible)

## 6. Owning team + ledger entry

16. ☐ Owning team identified
17. ☐ [VENDOR-PARTNER-LEDGER.md](../VENDOR-PARTNER-LEDGER.md) row added with: name + version + license + tier + purpose + isolation + replacement plan + owner

## 7. ADR if cross-axis impact

18. ☐ If dep affects ≥ 2 axes: ADR drafted in `docs/decisions/` (or proposed addendum to existing ADR)

## 8. CI gate

19. ☐ `cargo deny check` passes
20. ☐ `cargo machete` confirms no orphan deps newly created
21. ☐ Trivy / SBOM updated
22. ☐ `oya-governance-build-vs-buy` lane passes

## 9. Council review (for `requires-review` tier)

23. ☐ Architecture council schedule
24. ☐ Founder + legal sign-off if `requires-review` license tier
25. ☐ Per-pack regulatory implications check

## 10. Anti-patterns

- "Just use Library X for now; we'll replace later" — forbidden without explicit replacement plan + ETA + owning team
- External SaaS as system-of-record for any cross-axis contract — forbidden
- Adopting popular library because it's popular — popularity is not a sufficient criterion
- Per-team per-axis decisions in isolation when the dep crosses axes — surface to council

## 11. Sources

- [ADR-0014 build-vs-buy policy](../decisions/ADR-0014-build-vs-buy-policy.md)
- [ADR-0013 product license policy](../decisions/ADR-0013-product-license-policy.md)
- [TOOLCHAIN.md §6 decision flow chart](../TOOLCHAIN.md)
- [VENDOR-PARTNER-LEDGER.md](../VENDOR-PARTNER-LEDGER.md)
