# Independent Verification — Synthesis Headline Claims

**Verifier role:** independent. Trust nothing as-is. Every check below was run against PRIMARY SOURCES (the real ADR `.md` files under `/Users/jasonlee/Developer/source/docs/decisions` and the on-disk docs tree), never against the synthesis's own restatement of them.

**Date:** 2026-06-06
**Source root:** `/Users/jasonlee/Developer/source/docs`
**Verdict summary:** 6/6 CONFIRMED. No phantom findings. Several claims came back *stronger* than the headline stated (corruption + phantom-citation counts are larger than implied).

---

## Claim 1 — KCMVP corruption (`KCMVP` → `KCminimum-shippable-tier`)

**Verdict: CONFIRMED.** A find/replace accident overwrote the Korean crypto term **KCMVP** (Korean Cryptographic Module Validation Program) with the literal expansion of an unrelated acronym **MVP = "minimum-shippable-tier"**, producing the nonsense token `KCminimum-shippable-tier`.

Evidence (`grep -rn 'KCminimum-shippable-tier'`):
- **decisions/: 4 files, multiple occurrences each.**
  - `ADR-0009-cell-architecture-per-tenant-per-region.md:22, 55, 73, 155`
  - `ADR-0043-secrets-management-openbao-and-hsm-per-cell.md:7, 22, 28, 52, 53, 55, 56, 150, 158, 168, 184, 215`
  - `ADR-0002-tenant-and-identity-kernel.md:143`
  - `ADR-0121-onprem-k8s-stack-kubeadm-containerd-istio-envoy.md:54`
- **docs-wide: 20 total occurrences across 6 files** (the 4 ADRs above + `ADR-INDEX.md` + `machine-readable/decisions.json`).

That `KCMVP` is a real term (not invented) is proven by its 46 *correct* surviving occurrences elsewhere in docs (e.g. `COMPLIANCE-MATRIX.md`, `SECURITY-PROGRAM.md`, `RISK-REGISTER.md`, `GLOSSARY.md`, `SPEC.md`). The corruption is self-evident in context: `ADR-0043:215` reads `KR KCminimum-shippable-tier (Korean Cryptographic Module Validation Program)` — the parenthetical is the literal definition of **KCMVP**, confirming the token should read `KCMVP`.

---

## Claim 2 — Phantom Cedar-engine ADR-0150

**Verdict: CONFIRMED (stronger than stated).** ADR-0150 on disk is **cursor-pagination**, not a Cedar policy engine. A large body of ADRs cite a non-existent file `ADR-0150-cedar-policy-engine.md`.

Evidence:
- `ls` returns exactly one file: `ADR-0150-cursor-pagination-canonical.md`.
  - Its header (`:1`) is `# ADR-0150: Cursor Pagination Canonical`, Status: Accepted, Date 2026-05-18. No Cedar content; it is about opaque cursor pagination (AWS NextToken / Stripe `starting_after`).
- No file named `ADR-0150-cedar*` exists (glob returns "no matches found").
- **37 distinct decision files** reference the phantom `ADR-0150-cedar-policy-engine.md`, with **40 total occurrences** of that string. Examples with explicit "Cedar policy engine" prose treating 0150 as the engine:
  - `ADR-0348-...:32, 775` ("ADR-0150 (Cedar policy engine)")
  - `ADR-0255-...:32, 1902` ("ADR-0150 — Cedar policy engine")
  - `ADR-0313-...:31, 513, 751, 830, 2561, 2695` ("ADR-0150 Cedar policy engine", "Cedar v4.2 LTS per ADR-0150 §D-5")
  - `ADR-0337-...:46, 294, 668` ("does not amend ADR-0150 (Cedar policy engine)")
  - `ADR-0341-...:38, 451`, `ADR-0251-...:23, 39`
- By contrast only **2 files** reference the real `ADR-0150-cursor-pagination`.

So the canonical "Cedar policy engine" is referenced fleet-wide by a slug that points at the wrong on-disk document. (The actual Cedar substrate appears to live under ADR-0243 "cedar-as-universal-gate" / ADR-0007 — a number collision/misroute, not merely a typo.)

---

## Claim 3 — Identity contradiction: ADR-0187 (Zitadel) vs ADR-0476 (bespoke Rust IdP)

**Verdict: CONFIRMED.** Two mutually-exclusive identity decisions are both **Accepted** and live side-by-side; the earlier one is **not** marked superseded.

ADR-0187 (`ADR-0187-canonical-oidc-idp-zitadel-primary.md`):
- Front-matter: `status: Accepted` (`:3`), `supersedes: []` (`:7`), **`superseded_by: []`** (`:8`).
- Title `:17`: "Canonical OIDC IdP: Zitadel primary". Names **Zitadel v2.55+** as "the canonical Identity Provider … for the `identity` µservice" (`:21`, `:37`).
- **Explicitly rejects Self-built IdP** (`:98–100`): "### Self-built IdP / Rejected. Identity is undifferentiated heavy-lifting; building it ourselves consumes engineering capacity for zero competitive advantage…".

ADR-0476 (`ADR-0476-oya-identity-bespoke-human-identity.md`):
- Front-matter: `status: Accepted` (`:4`), `authority: founder`, `supersedes: [ADR-0421]` (`:9`).
- Title (`:3`, `:14`): "oya-identity: bespoke Rust human identity substrate". Status (`:18`): "Accepted — 2026-05-28 (founder-locked). Supersedes ADR-0421 (Keycloak)." Decision (`:29`, `:35`): "build `oya-identity`, a bespoke Rust-native OIDC provider".
- **Explicitly rejects Zitadel** in its alternatives table (`:103`): "**Zitadel** | Go-based; newer; smaller federation adoption; same Go-stack objection".

Contradiction confirmed: 0187 (Accepted) mandates Zitadel and rejects a self-built IdP; 0476 (Accepted, founder-locked, later date 2026-05-28) mandates a bespoke Rust IdP and rejects Zitadel. **Crucially, 0476 supersedes ADR-0421 (Keycloak), NOT ADR-0187** — and no ADR anywhere marks 0187 as superseded (`grep superseded_by.*0187` → 0 hits; 0187's own `superseded_by: []`). Both remain live and contradictory.

---

## Claim 4 — Duplicate ADR-0377

**Verdict: CONFIRMED.** Two distinct files share id ADR-0377:
- `ADR-0377-forgejo-board-git-ref-cas-fallback.md` — id ADR-0377, status `Proposed (conditional: Accepted only after ADR-0377-D2 and ADR-0377-D3 code/tests pass)`. Subject: Forgejo board / git-ref CAS fallback.
- `ADR-0377-kafka-to-pulsar-via-kop.md` — id ADR-0377, title "Migrate Kafka to Pulsar via KoP wire-compat". Subject: Kafka→Pulsar messaging migration.

Two files, same number, two unrelated subjects (a VCS/board storage decision vs a messaging-bus migration). Number collision confirmed.

---

## Claim 5 — "Ontology renamed to Ontology" tautology in ADR-0006

**Verdict: CONFIRMED.** The same find/replace that hit KCMVP collapsed the *old* and *new* names into the identical token, producing self-referential nonsense.

Evidence (`ADR-0006-ontology-typed-entity-layer.md`):
- `:11` — `> **Date:** 2026-05-09 (rewritten 2026-05-13 — "Ontology" renamed to "Ontology")`
- `:22` — `"Ontology" was the prior name for this layer. Per session decision 2026-05-13, it is renamed to **Ontology**, matching Palantir's established term.` (prior-name == new-name; a rename to itself)
- `:123` — `- ADR-0055 (Ontology renamed to Ontology)`

The intended sentence was clearly "**X renamed to Ontology**" where X was the prior project-internal name; the bulk replace overwrote the prior name with "Ontology" too, yielding the tautology. (Same line also shows the related corruption "Bominal ADR-0106" / "Bominal" → likely "Nominal".)

---

## Claim 6 — Masterplan fork: ADR-0364 + ADR-0365 make masterplan generated-from-ADRs, yet MASTERPLAN.md still self-declares `compatibility_projection`

**Verdict: CONFIRMED.**

- `ADR-0364-generative-adr-template-and-masterplan-generation.md`: `status: Accepted` (`:3`). Purpose (`:47`): "Make the masterplan a GENERATED projection of the ADR decision log." `:67–71`: "### 1. The masterplan is a GENERATED projection of the ADR log / `oya gen masterplan` reads accepted `planning_impact: true` ADRs … `specs/masterplan.json` becomes build output, never hand-authored. A **drift gate** (committed == regenerated)…".
- `ADR-0365-automated-adr-lifecycle-and-propagation.md`: `status: Accepted` (`:3`). `:52`: "ADR-0364 made the masterplan a generated projection of the ADR log. The remaining gap: …" — i.e. 0365 builds on and reaffirms 0364's generated-masterplan model.
- Both Accepted, both `deciders: council-architecture, founder`.

Meanwhile the live `docs/MASTERPLAN.md` front-matter (`:1–6`) still declares:
```
doc_class: MasterPlan
shape: compatibility_projection
...
status: Accepted
canonical_authority: /specs/masterplan.json
```
`shape: compatibility_projection` appears exactly once in `docs/` and it is in `MASTERPLAN.md:3`. So the fork is real: two Accepted ADRs declare the masterplan a *generated projection of the ADR log* (drift-gated build output from `specs/masterplan.json`), while the masterplan document itself is still hand-authored and tags its own shape as `compatibility_projection` — the unreconciled "two SSOTs" condition the synthesis flags.

---

## Digest

| # | Claim | Verdict | Key evidence |
|---|-------|---------|--------------|
| 1 | KCMVP → `KCminimum-shippable-tier` corruption | **CONFIRMED** | 20 occ / 6 files docs-wide (4 ADRs); 46 correct `KCMVP` survive elsewhere; ADR-0043:215 self-defines it |
| 2 | Phantom Cedar-engine ADR-0150 | **CONFIRMED (stronger)** | 0150 file = cursor-pagination; 37 files / 40 occ cite phantom `ADR-0150-cedar-policy-engine.md`; vs 2 cite real slug |
| 3 | 0187 (Zitadel) vs 0476 (bespoke Rust) | **CONFIRMED** | both `status: Accepted`; 0187 `superseded_by: []`, rejects self-built (:98); 0476 rejects Zitadel (:103), supersedes 0421 not 0187 |
| 4 | dup-0377 | **CONFIRMED** | two files: forgejo-board-git-ref-cas-fallback (Proposed) + kafka-to-pulsar-via-kop |
| 5 | "Ontology renamed to Ontology" | **CONFIRMED** | ADR-0006:11, :22, :123 — rename-to-self tautology from same bulk replace |
| 6 | masterplan fork (0364+0365 vs MASTERPLAN.md) | **CONFIRMED** | 0364 & 0365 both Accepted, masterplan = generated projection; MASTERPLAN.md:3 still `shape: compatibility_projection` |

No claim refuted. No phantom finding in the synthesis's six headline claims (all load-bearing assertions resolve to real file:line evidence). Two claims (1, 2) are materially understated by the headline if anything — the corruption and phantom-citation blast radius is larger than a single instance.
