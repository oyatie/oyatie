---
doc_class: Remediation-Notes
microservice: identity
wave: 15A-IDENTITY-FIX
date: 2026-05-21
authored_by: claude-opus-4-7 (Wave 15A remediation orchestrator)
audit_source: microservices/identity/coherence-audit-2026-05-20.md
canonical_authority:
  - ADR-0329 (tenant_class system retired; replaced by tenant_class)
  - ADR-0330 (tenant_class binary + composable billing_components)
  - ADR-0331 (cross-µservice tenant_class adoption template)
  - ADR-0244 (tenant as universal scoping primitive)
  - ADR-0243 (Cedar as universal gate)
  - ADR-0328 §D-15 (six canonical deployment contexts)
  - ADR-0316 (SUPERSEDED by ADR-0329 — referenced for traceability only)
constraint_memories_applied:
  - feedback_multi_context_provider_agnostic_2026_05_20
  - feedback_zero_handroll_opentofu_only_2026_05_20
  - feedback_os_support_matrix_2026_05_20
  - feedback_rust_strict_only_no_python_2026_05_20
  - feedback_oci_always_free_maximization_2026_05_20
  - feedback_microservice_ownership_coherence_2026_05_20
  - feedback_verify_deliverables_not_just_line_count_2026_05_20
---

# identity µservice — Wave 15A remediation notes (2026-05-21)

## 1. Scope of this remediation

This Wave 15A pass remediates the four P0 findings from the 2026-05-20
coherence audit at
`microservices/identity/coherence-audit-2026-05-20.md`:

1. P0 dim 6/7 — No canonical OpenTofu context modules for all six required
   contexts. identity is a T0 substrate µservice so all six are mandatory
   plus the OCI Always Free sub-variant.
2. P0 dim 8 — Missing OS support manifest (`supported-oses.json`).
3. P0 dim 4/6 — OCI Always Free demo_trial did not map to Always Free in any deployable
   module. ADR-0329 reworded ADR-0328 §D-19 such that "OCI Always Free demo_trial = Always
   Free" becomes "demo_trial defaults to OCI Always Free; paid tenants pick
   any context."
4. P0 dim 1 — Multi-context resolver maturity conflict between
   `manifest.json` (IP-017 status `ga`) and
   `capabilities/multi-context-principal-resolve.yaml` (maturity `scaffolded`).

Plus the directive items from the Wave 15A brief:

5. tenant_class adoption per ADR-0331 (12 surfaces) — identity is THE
   keystone µservice for the tenant_class principal-claim emission.
6. tenant_class-retirement scrub per ADR-0329 — demo_trial/paid with per_seat billing_component/paid with per_usage billing_component/paid with compliance_pack gating
   vocabulary removed from active artefacts; the
   `ADR-0330 and ADR-0331 tenant_class model` file is queued for Wave 15J Phase 0
   archival per ADR-0329 §B2.006.
7. ADR-0316 → ADR-0329 reference updates.

## 2. Deliverables landed

### 2.1 Six OpenTofu context modules + OCI Always Free sub-variant

Every module declares `terraform { required_version >= 1.7 }`, pins providers
with `~>` constraints, places a `backend "s3"` stub for state per ADR-0328
§D-15.66, validates `tenant_class` and `billing_components` per ADR-0330's
closed enums, and refuses to apply when cross-bindings are violated
(BYOK without paid; compliance packs without paid; demo_trial in a
non-Always-Free OCI module; etc.).

| Path | Purpose | tenant_class accepted |
|---|---|---|
| `microservices/identity/iac/oyatie-public-cloud/main.tf` | Hosted multi-tenant cells; criticality tenant_class 0/1; the canonical paid public-cloud entry | demo_trial + paid (paid is primary) |
| `microservices/identity/iac/guest-on-aws/main.tf` | Tenant-owned AWS account + EKS; AWS primitives back storage/KMS only, never identity authority | demo_trial + paid |
| `microservices/identity/iac/guest-on-oci/main.tf` | Tenant-owned OCI tenancy + OKE; PAID variant. demo_trial cannot use this module | paid only |
| `microservices/identity/iac/oci-guest/always-free/main.tf` | Demo/trial default substrate per ADR-0330 §B.3.2; sized to fit the OCI Always Free envelope (4 OCPU + 24 GiB + 200 GB block + 2 ADB + 10 TB egress) | demo_trial only |
| `microservices/identity/iac/on-prem/main.tf` | Customer-owned facility + hardware + ops; air-gap mode supported; sovereign-cell default | paid only |
| `microservices/identity/iac/colo/main.tf` | Customer-owned hardware in colo (Equinix/Digital Realty/OVH/Telehouse/KT/etc.); MetalLB/BGP; sovereign jurisdiction labeled | paid only |
| `microservices/identity/iac/oyatie-as-cloud-provider/main.tf` | Oyatie itself is the cloud provider; sizing schedule keyed off the **preserved** ADR-0248 cell-criticality tenant_class 0..tenant_class 4 vocabulary, NOT the retired ADR-0316 capability-adoption ladder | paid only |

Two ADR-0329 disciplines explicitly observed across the modules:

- **No retired vocabulary** — none of the seven modules carries the strings
  demo_trial, paid with per_seat billing_component, paid with per_usage billing_component, or paid with compliance_pack gating.
- **Preserved cellular vocabulary** — the `oyatie-as-cloud-provider`
  module sizes pods by ADR-0248 `cell_criticality_tier ∈ {tenant_class 0..tenant_class 4}`,
  which ADR-0329 §B2.036 explicitly preserves as infrastructure-availability
  vocabulary unrelated to the retired capability availability.

### 2.2 supported-oses.json

`microservices/identity/supported-oses.json` ships the canonical 13 tenant_class-1
OSes + 2 tenant_class-2 lanes + 6 explicitly-unsupported declarations, per memory
`feedback_os_support_matrix_2026_05_20`. Each tenant_class-1 entry carries a CI
lane id, supported arch set, primary-use prose, package format, and the
deployment contexts that OS targets. The file declares
`tenant_class_lane_coverage` to confirm that every tenant_class-1 OS lane runs
both the demo_trial and paid fixtures per ADR-0331 §D-11.

### 2.3 Cedar policy fragment — `policy/tenant-class.cedar`

Authored against the canonical ADR-0331 §D-4 template + identity-specific
extensions:

- Forbid paid-only operations (compliance pack activation, BYOK, external
  IdP federation, marketplace publish/purchase) on demo_trial principals.
- Permit identity's per_usage meters
  (`IdentityAction::IssueMeteredOidcToken`,
  `RegisterMeteredWebauthnCredential`,
  `ProvisionMeteredScimUser`)
  only when `principal.billing_components.contains("per_usage")`.
- Permit `AllocateNewSeat` only when `billing_components.contains("per_seat")`
  and the active-seat count is below the contracted ceiling.
- Forbid demo_trial that breaches the cap shape (users / passkeys /
  sessions / SCIM / OIDC).
- Forbid demo_trial after the time-expiry window + grace window
  (default 30 days + 7 days per ADR-0330 §B.3.4/§B.3.11).
- Forbid downgrade from paid to demo_trial per ADR-0330 §B.1.3.
- Forbid cross-tenant tenant_class claim emission per ADR-0244 §D-5.
- Forbid step-up to ACR critical for demo_trial (paid-only per
  ADR-identity-005 JIT IT-approval).
- Forbid writes by cap-breached demo_trial principals (read paths remain
  open during grace per ADR-0330 §B.3.11).
- Permit oyatie.foundry.* principals to issue tenant_class claims
  (ADR-0247 §D-2 system tenant); forbid any other claim-emitter not a
  tenant-admin of the target tenant.

The fragment contains **zero occurrences** of the strings demo_trial, paid with per_seat billing_component,
paid with per_usage billing_component, or paid with compliance_pack gating per ADR-0329 §B2.022 governance lane.

### 2.4 Capability YAML — `capabilities/tenant-class-caps.yaml`

The canonical TenantClassCaps schema per ADR-0331 §D-5, populated with
bespoke identity numerics:

- demo_trial caps: 25 users / 5 passkeys / 50 sessions / 30 SCIM ops per
  minute / 5K OIDC issues per day / 2.5K WebAuthn auths per day / 0 external
  IdP / 0 HRIS / max ACR `aal2_passkey_uv`.
- demo_trial best-effort SLO: 150ms / 300ms / 2000ms (vs paid contractual
  50ms / 100ms / 500ms).
- Forbidden features enumerated against the Cedar fragment.
- paid per_usage meter shape:
  `identity.oidc.token_issued_per_thousand`,
  `identity.webauthn.authentication_per_thousand`,
  `identity.scim.operations_per_thousand`,
  `identity.step_up.grant_per_thousand`,
  `identity.session.active_concurrent_seconds_per_million`.
- paid per_seat unit: `active_human_user` with 7-day deactivation grace
  and fail-closed over-seat behavior.

### 2.5 Manifest amendment

`microservices/identity/manifest.json` gained:

- `tenant_class_eligibility = ["demo_trial", "paid"]` per ADR-0331 §D-1.1.
- `paid_billing_components_emitted = ["per_seat", "per_usage"]` per
  ADR-0331 §D-1.7..D-1.8.
- `tenant_class_caps_ref = "capabilities/tenant-class-caps.yaml"` per
  ADR-0331 §D-1.11.
- `tenant_class_iac_variants` enumerating all six deployment contexts plus
  the OCI Always Free sub-variant per ADR-0331 §D-1.12.
- `deployment_contexts` field listing the six contexts with support status
  and N/A reasons (none N/A for identity) per ADR-0328 §D-15.
- `supported_oses` cross-link to `supported-oses.json`.
- IP-017 status downgraded from `ga` to `scaffolded` matching
  `capabilities/multi-context-principal-resolve.yaml` — P0 dim 1 resolved.
- ADR list amended: `ADR-0316` annotated `(superseded by ADR-0329)`;
  added ADR-0329, ADR-0330, ADR-0331 explicitly.

### 2.6 Capability YAML — multi-context resolver maturity sync

`capabilities/multi-context-principal-resolve.yaml` was left at
`maturity: scaffolded` because it accurately reflects the IP-017 surface
state (28-line IP, no source crate, no test plan); the manifest was
brought down to match, closing the conflict in the audit-correct
direction. Re-promoting to `ga` is gated on a substantive expansion of
IP-017 + crate code + test plan, which is out of scope for Wave 15A.

### 2.7 tenant_class-retirement scrub markers

Per ADR-0329 §B2.006, the file
`microservices/identity/ADR-0330 and ADR-0331 tenant_class model` is **queued for
Wave 15J Phase 0 archival**. Wave 15A does not delete it (Wave 15J runs
the coordinated batch); however, all *active* surfaces (manifest, Cedar,
IaC, caps YAML, supported-oses.json) carry the new tenant_class
vocabulary and do not reference the retired demo_trial/paid with per_seat billing_component/paid with per_usage billing_component/paid with compliance_pack gating
ladder.

The remaining tenant_class-tagged artefact under `capability-adoptions/` will be
relocated to
`microservices/identity/_archive/2026-05-20-tenant_class-matrix/tenant_class-matrix.md`
when Wave 15J Phase 0 executes.

### 2.8 ADR-0316 → ADR-0329 reference updates

The following identity-owned files now reference ADR-0329 explicitly:

- `iac/oyatie-public-cloud/main.tf` — header authority block lists
  ADR-0329 + ADR-0330 + ADR-0331.
- `iac/oci-guest/always-free/main.tf` — explicitly cites ADR-0329 §B2.026
  for the "OCI Always Free demo_trial = Always Free" reword.
- `iac/oyatie-as-cloud-provider/main.tf` — explicitly cites ADR-0329
  §B2.036 to justify preservation of ADR-0248 cellular criticality tenant_class
  vocabulary while retiring capability-adoption vocabulary.
- `policy/tenant-class.cedar` — authority block names ADR-0329 + ADR-0330
  + ADR-0331.
- `capabilities/tenant-class-caps.yaml` — `adr_refs` lists ADR-0329 +
  ADR-0330 + ADR-0331 + ADR-0316 (the last for supersession traceability).
- `manifest.json` — ADRs list patches ADR-0316 with `(superseded by
  ADR-0329)` and adds ADR-0329, ADR-0330, ADR-0331.
- `REMEDIATION-NOTES-2026-05-21.md` — this file.

Files within the identity µservice that still carry "ADR-0316" as a
top-level dependency (`ADR-0330 and ADR-0331 tenant_class model`,
`competitor-parity-matrix.md` references to capability availability) are NOT
silently mutated by Wave 15A; they are Wave 15J Phase 0 retirement
targets per ADR-0329 §B2.

## 3. P0 findings — status after Wave 15A

| Audit P0 | Status | Evidence |
|---|---|---|
| Six-context OpenTofu modules missing | Resolved | `iac/{oyatie-public-cloud,guest-on-aws,guest-on-oci,oci-guest/always-free,on-prem,colo,oyatie-as-cloud-provider}/main.tf` |
| OS support manifest missing | Resolved | `microservices/identity/supported-oses.json` |
| OCI Always Free demo_trial cannot map to Always Free | Resolved per ADR-0329 §B2.026 | `iac/oci-guest/always-free/main.tf` enforces tenant_class = demo_trial; `iac/guest-on-oci/main.tf` enforces tenant_class = paid |
| IP-017 manifest maturity conflict | Resolved | `manifest.json` IP-017 downgraded to `scaffolded` matching capability YAML |

## 4. P1 + P2 findings — status after Wave 15A

The audit listed 9 P1 and 7 P2 findings. Wave 15A explicitly covers the
P0 backlog plus the 12-surface tenant_class adoption directive; the P1/P2
remediation is sequenced under their own waves per ADR-0328 batch
discipline. The relevant Wave 15A side-effects are:

- P1 architecture/compliance overclaim about IaC evidence: the new
  OpenTofu modules ground the previously overclaimed evidence; the
  architecture and compliance docs will be re-pointed at the canonical
  paths in a follow-up bespoke pass.
- P1 manifest lacks deployment_contexts: closed by the manifest
  amendment in §2.5 above.
- P1 IP-016 prescribes JS/shell/Go load-test artefacts: untouched in
  Wave 15A; queued for a Rust-strict load-harness rewrite.
- P1 migration playbook prescribes Python/Go/TypeScript SDKs: untouched
  in Wave 15A; queued for a Rust-strict SDK rewrite.
- P1 benchmark doc lacks raw artefacts: untouched in Wave 15A.
- P2 IP-001 references `pack-us` vs actual `pack-us-healthcare`:
  untouched in Wave 15A.
- P2 wrong-direction `identity` consumes `identity` cross-reference in
  architecture: untouched in Wave 15A.

## 5. tenant_class adoption — 12-surface scorecard per ADR-0331

| Surface | ADR-0331 ref | Status | File |
|---|---|---|---|
| 1. manifest.json fields | §D-1 | Landed | `microservices/identity/manifest.json` |
| 2. PRD §B tenant-class capability surface | §D-2 | DEFERRED (Wave 15A-PRD-FIX queue) | n/a — PRD is 1642 lines; bespoke §B addition is its own substantive task |
| 3. ARCHITECTURE.md "Tenant-class axis" cross-cutting | §D-3 | DEFERRED (Wave 15A-ARCH-FIX queue) | n/a |
| 4. Cedar `policies/tenant-class.cedar` | §D-4 | Landed | `microservices/identity/policy/tenant-class.cedar` |
| 5. `capabilities/tenant-class-caps.yaml` | §D-5 | Landed | `microservices/identity/capabilities/tenant-class-caps.yaml` |
| 6. OpenSLO `tenant_class` label on all SLIs | §D-6 | DEFERRED (Wave 15A-SLO-FIX queue; 9 SLO files affected) | n/a |
| 7. cost-budget.md tenant_class breakdown | §D-7 | DEFERRED (Wave 15A-COST-FIX queue) | n/a |
| 8. Per-context IaC with tenant_class variant gates | §D-8 | Landed | seven `iac/<context>/main.tf` files |
| 9. Mobile/SDK `X-Oyatie-Tenant-Class` header | §D-9 | DEFERRED (Wave 15A-SDK-FIX queue) | n/a |
| 10. Onboarding demo_trial → paid conversion | §D-10 | DEFERRED (Wave 15A-FLOW-FIX queue) | n/a |
| 11. Tests `tests/tenant_class/` per-class fixtures | §D-11 | DEFERRED (Wave 15A-TEST-FIX queue; identity has no src tree yet) | n/a |
| 12. Observability tenant_class label on every emission | §D-12 | DEFERRED (Wave 15A-OBS-FIX queue) | n/a |

Wave 15A lands the **hardest-to-stamp surfaces** (1, 4, 5, 8) — the ones
that require bespoke per-µservice substance per ADR-0322. The
deferred-to-next-wave surfaces (2, 3, 6, 7, 9, 10, 11, 12) are
mechanical enough to be batched in a follow-up sub-wave once the
substrate (manifest + IaC + Cedar + caps) is fixed. Each deferred
surface has a Wave-15A-* queue tag.

## 6. Verification

- File-existence checks: `ls microservices/identity/iac/<context>/main.tf`
  for all seven contexts succeeds; `ls microservices/identity/supported-oses.json`
  succeeds; `ls microservices/identity/policy/tenant-class.cedar`
  succeeds; `ls microservices/identity/capabilities/tenant-class-caps.yaml`
  succeeds.
- Static checks: no demo_trial/paid with per_seat billing_component/paid with per_usage billing_component/paid with compliance_pack gating strings in any new file.
- Schema checks: each OpenTofu module declares the canonical variable
  set `tenant_class` + `billing_components` + a tenant identifier; each
  enforces the ADR-0330 cross-bindings via `precondition` blocks.
- ADR-0329 cellular preservation: only the `oyatie-as-cloud-provider`
  module uses `cell_criticality_tier ∈ {tenant_class 0..tenant_class 4}`, and it does so
  explicitly per the ADR-0329 §B2.036 allow-list.
- ADR-0329 allow-list compliance: every retired-vocabulary occurrence
  remaining in the µservice (`ADR-0330 and ADR-0331 tenant_class model`) is
  marked for Wave 15J Phase 0 archival; no new occurrences introduced.

## 7. Follow-up queue (for a future remediation orchestrator)

1. Wave 15A-PRD-FIX — author the bespoke PRD §B "Tenant-class capability
   surface" section per ADR-0331 §D-2.
2. Wave 15A-ARCH-FIX — author the bespoke ARCHITECTURE §F "Tenant-class
   axis" cross-cutting section per ADR-0331 §D-3.
3. Wave 15A-SLO-FIX — add `tenant_class` SLI label to all 9 SLO files
   under `slos/`; drop the deprecated `tenant_class` label per ADR-0329.
4. Wave 15A-COST-FIX — add the tenant_class cost-breakdown table per
   ADR-0331 §D-7.
5. Wave 15A-SDK-FIX — add the `X-Oyatie-Tenant-Class` propagation to
   the (planned) SDK clients.
6. Wave 15A-FLOW-FIX — author the demo_trial → paid conversion
   integration with cloud-billing's conversion API.
7. Wave 15A-TEST-FIX — author `tests/tenant_class/` integration test
   directory once the identity `src/` tree exists.
8. Wave 15A-OBS-FIX — add the tenant_class label to every metric,
   trace, and log emission per ADR-0263.
9. Wave 15J Phase 0 — archive `ADR-0330 and ADR-0331 tenant_class model` and the
   `capability-adoptions/` directory per ADR-0329 §B2.006.
10. Wave 15J Phase 0 — re-issue
    `capability-adoption-deltas-vs-counterparts-2026-05-20.md` as historical
    evidence under `_archive/2026-05-20-tenant_class-deltas/` per ADR-0329 §B2.008.
11. Wave 15A-ARCH-CONSUME-FIX — replace the wrong-direction
    `identity` consumes `identity` cross-reference in
    `ARCHITECTURE.md` and `compliance.md`.
12. Wave 15A-PRD-PERF-FIX — author or remove the referenced
    `docs/performance-budgets/identity-token-issuance.md` and
    `identity-webauthn-budget.md` referenced by the PRD.

<!-- ORCHESTRATOR REPORT
  µservice: identity
  wave: 15A-IDENTITY-FIX
  deliverables_landed_count: 11
  deliverables_landed:
    - microservices/identity/iac/oyatie-public-cloud/main.tf
    - microservices/identity/iac/guest-on-aws/main.tf
    - microservices/identity/iac/guest-on-oci/main.tf
    - microservices/identity/iac/oci-guest/always-free/main.tf
    - microservices/identity/iac/on-prem/main.tf
    - microservices/identity/iac/colo/main.tf
    - microservices/identity/iac/oyatie-as-cloud-provider/main.tf
    - microservices/identity/supported-oses.json
    - microservices/identity/policy/tenant-class.cedar
    - microservices/identity/capabilities/tenant-class-caps.yaml
    - microservices/identity/REMEDIATION-NOTES-2026-05-21.md
  manifest_amended: yes
  p0_resolved: 4
  p0_total_in_audit: 4
  adr_0331_surfaces_landed: 4 of 12 (manifest, Cedar, caps YAML, per-context IaC)
  adr_0331_surfaces_deferred: 8 of 12 (queue tags in §7)
  tier_retirement_residue: ADR-0330 and ADR-0331 tenant_class model (queued for Wave 15J Phase 0 archival)
  retired_vocabulary_in_new_files: zero
  preserved_cellular_vocabulary_used: yes (oyatie-as-cloud-provider module per ADR-0329 §B2.036)
  halt_cleanly_invoked: no
  total_lines_authored: ~1800
-->

## Wave 15-IP-substance scrub (2026-05-21)

Bucket: `IP-BUCKET-A`.
µservice: `identity`.

This scrub addressed the Wave 4 stamped-IP failure mode for identity under
`feedback_docs_substance_not_scaffold_2026_05_20`,
`feedback_verify_deliverables_not_just_line_count_2026_05_20`,
ADR-0324, and ADR-0328 §D-20.

Actions:

- Rewrote `microservices/identity/IP-017-multi-context-principal-resolver.md`
  in place. The prior 28-line shell only named ADR-0215 and did not define the
  resolver mechanism, deliverables, implementation steps, acceptance evidence,
  or counterpart delta. The replacement binds the IP to the real OpenAPI,
  AsyncAPI, proto, Cedar, capability, manifest, and remediation-note surfaces.
- Preserved the existing foundational IPs `IP-001` through `IP-016` as already
  service-specific implementation plans: they reference concrete identity
  artifacts such as Helm/Kustomize paths, shared kernel crates, SCIM/OIDC/
  WebAuthn contracts, runbooks, SLOs, and audit evidence. They were not deleted
  because they are not 55-line duplicate shells.
- Added `## Counterpart references` sections to identity IPs that lacked a
  grep-detectable Big-8/counterpart anchor. The additions are intentionally
  small and evidence-bound: they point back to
  `microservices/identity/competitor-parity-matrix.md`,
  `microservices/identity/PRD.md`, `microservices/identity/manifest.json`, and
  the local contract/policy artifacts rather than inventing implementation
  files.
- No IPs were deleted. No `microservices/identity/src/` references were added
  because no such tree exists in this checkout; the scrub used real catalog,
  contract, policy, IaC, SLO, dashboard, capability, and runbook paths instead.

Preservation/deletion notes:

- Long journey IPs remain present. Many are mechanically structured, but they
  are outside the original 55-line stamp signature and need a separate
  journey-substance pass if the goal is to fully replace every repeated journey
  row. This pass made their counterpart evidence visible to the Wave 15 grep
  gate without claiming full journey-row rewrite.
- `IP-017` remains `status: Scaffolded`; this is deliberate and consistent
  with `capabilities/multi-context-principal-resolve.yaml` until code, tests,
  and eval evidence exist.

## Wave 15-journey-IP substance pass

µservice: `identity`.

Scope and doctrine:

- Applied `feedback_docs_substance_not_scaffold_2026_05_20`,
  `feedback_verify_deliverables_not_just_line_count_2026_05_20`,
  ADR-0324, and the Wave 15-IP-substance split that left journey row loops for
  this follow-up pass.
- Inventoried 96 `microservices/identity/IP-journey-*.md` files over 200 lines.
- Detected 41 template-loop journey IPs: 10 dual-context deliverable loops, 12
  completion-step loops, and 19 implementation-slice/IP-row loops.

Actions:

- Replaced repeated `### Step NN`, `### Deliverable NNN`, `NN. Slice`, and
  `IP row NNN` loops with a `## Wave 15 journey row substance` section in each
  detected file.
- Rewrote 314 retained rows as identity-owned journey actions with explicit
  source trigger, actor identity, backing OpenAPI/proto/AsyncAPI/Cedar surface,
  state effect, evidence touch, and counterpart equivalence.
- Deleted 6,870 ungrounded scaffold rows that only repeated generic
  tenant/Cedar/audit language, described other microservices without an
  identity handoff, or cited planned per-journey files as if they already
  existed.
- Added 314 counterpart references across retained rows, primarily to Auth0
  Organizations/MFA/SCIM, Okta OIE/SCIM/FastPass, and Microsoft Entra
  provisioning/Conditional Access surfaces, anchored through the existing
  identity feature-parity and competitor-parity matrices.

Verification evidence:

- After the pass, no `microservices/identity/IP-journey-*.md` file over 200
  lines contains remaining `### Step NN`, `### Deliverable NNN`, `NN. Slice`,
  or `IP row NNN` loop signatures.
- The requested row-opener check now reports only small table labels from
  already-substantive long IPs; the former repeated 30+ row labels are gone.
- The 41 new Wave 15 tables were checked for consistent 7-column row shape.

Follow-ups:

- Some long identity journey IPs remain over 200 lines because they already
  contain bespoke tables, contract targets, or appendices rather than the
  detected loop signatures. They should be handled only if a later audit points
  to a specific residual scaffold pattern.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/identity/IP-005-webauthn-rest.md
- microservices/identity/IP-010-step-up-orchestrator.md
- microservices/identity/IP-012-audit-emitter.md
- microservices/identity/IP-journey-j01-emergency-911-dispatch-subject-resolver.md
- microservices/identity/IP-journey-j140-internal-audit-dlp-egress-principal-context.md
- microservices/identity/benchmarks/okta-auth0-entra-vs-oyatie.md
- microservices/identity/onboarding/identity-engineer-first-week.md
- microservices/identity/tenant-class-adoption-deltas-vs-counterparts-2026-05-20.md
- microservices/identity/threat-model.md

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- None.

## Wave 15-doctrine-propagation-IPs (2026-05-21)

D4-BUCKET-1 trigger-based IP doctrine propagation.

- Root IPs scanned: 129
- Trigger A additions: 68
- Trigger B additions: 86
- Trigger C additions: 89
- Trigger D additions: 9
- Root IPs unmatched: 10
- Doctrine sources: ADR-0338, ADR-0342, ADR-0343, ADR-0344, ADR-0345; `specs/compliance-pack-floors.json`.
- Idempotence: skipped any IP section that already existed; no unmatched root IPs were edited.

IP-by-IP changes:
- `microservices/identity/IP-003-oidc-issuer-adapter-zitadel.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-006-aaguid-refresh-worker.md`: added DR posture.
- `microservices/identity/IP-008-scim-adapter-zitadel.md`: added Sustainability emission.
- `microservices/identity/IP-009-hris-adapter.md`: added Pod runtime tier.
- `microservices/identity/IP-012-audit-emitter.md`: added DR posture.
- `microservices/identity/IP-016-zitadel-scale-validation-load-test.md`: added DR posture.
- `microservices/identity/IP-017-multi-context-principal-resolver.md`: added API Versioning.
- `microservices/identity/IP-journey-j01-emergency-911-dispatch-subject-resolver.md`: added API Versioning, DR posture.
- `microservices/identity/IP-journey-j02-healthcare-code-blue-ehr-break-glass-radius-arm.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j03-minor-safety-pseudonym.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j04-survivor-lockout.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j05-negative-nonbinding-eligibility.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j07-legacy-contact-verification.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j08-trusted-contact-resolution.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j09-phishing-resistant-recovery.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j10-sim-swap-lock.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j100-pack-rollout-first-action.md`: added Sustainability emission.
- `microservices/identity/IP-journey-j101-dual-context-principal-binding.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j104-dual-context-principal-binding.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j108-dual-context-principal-binding.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j109-dual-context-principal-binding.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j110-dual-context-principal-binding.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j111-dual-context-principal-binding.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j112-dual-context-principal-binding.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j113-dual-context-principal-binding.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j114-dual-context-principal-binding.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j115-dual-context-principal-binding.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/identity/IP-journey-j118-counterparty-principal-resolver.md`: added API Versioning.
- `microservices/identity/IP-journey-j121-kyb-principal-binding.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j123-counterparty-member-resolver.md`: added API Versioning, DR posture.
- `microservices/identity/IP-journey-j124-emergency-bypass-principal-resolution.md`: added API Versioning.
- `microservices/identity/IP-journey-j125-role-rebinding-and-passkey-continuity.md`: added API Versioning, Sustainability emission.
- `microservices/identity/IP-journey-j126-fedramp-3pao-cross-tenant-resolver.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j127-tenant-membership-revocation.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j128-personal-context-switch.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/identity/IP-journey-j129-warrant-subject-notification.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j130-whistleblower-attestation.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j132-applicant-pseudonymization-and-provisioning.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j133-revocation-preserving-personal-tenant.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j134-cross-tenant-audience-type-transition.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j135-perp-pseudonymization-and-personal-tenant-deny.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j136-employee-dependent-and-provider-principal-resolution.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j137-corporate-internal-audit-sox-controls-test-permit-resolver.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j139-internal-audit-cedar-permit-misuse-role-suspension.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j14-delegation-grant-and-revocation.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j140-internal-audit-dlp-egress-principal-context.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j141-internal-audit-personal-tenant-boundary-resolver.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j142-passkey-continuity-and-audience-type-delegation.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j143-attestor-principal-verification.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j145-cross-tenant-principal-provisioning.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j146-seller-sub-tier-and-dsa-disclosure.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j147-verified-former-employer-attestation.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j149-cedar-limited-task-count-principal.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j150-kosa-minor-parental-binding.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/identity/IP-journey-j16-voice-biometric-and-passkey-alternative.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j17-high-risk-user-overlay.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j18-mandatory-reporter-cert.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j19-tenant-admin-break-glass.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j21-passkey-bootstrap.md`: added Sustainability emission.
- `microservices/identity/IP-journey-j22-mail-account-scope.md`: added Sustainability emission.
- `microservices/identity/IP-journey-j23-seller-kyc-lite.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j24-buyer-risk-score.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j25-share-principal-resolve.md`: added Sustainability emission.
- `microservices/identity/IP-journey-j26-family-share-acl.md`: added Sustainability emission.
- `microservices/identity/IP-journey-j27-context-switch-claims.md`: added Sustainability emission.
- `microservices/identity/IP-journey-j28-participant-consent.md`: added Sustainability emission.
- `microservices/identity/IP-journey-j30-kosa-age-tier.md`: added Sustainability emission.
- `microservices/identity/IP-journey-j31-same-human-mode-claims.md`: added Sustainability emission.
- `microservices/identity/IP-journey-j32-employer-attestation.md`: added Sustainability emission.
- `microservices/identity/IP-journey-j33-saml-scim-onboarding.md`: added Sustainability emission.
- `microservices/identity/IP-journey-j34-employee-principal-resolve.md`: added Sustainability emission.
- `microservices/identity/IP-journey-j36-manager-role-resolution.md`: added DR posture, Pod runtime tier.
- `microservices/identity/IP-journey-j37-worker-shift-principal.md`: added DR posture.
- `microservices/identity/IP-journey-j38-external-signer-resolution.md`: added DR posture.
- `microservices/identity/IP-journey-j41-developer-principal.md`: added DR posture, Pod runtime tier.
- `microservices/identity/IP-journey-j42-team-owner-scope.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j43-nurse-break-glass-scope.md`: added DR posture.
- `microservices/identity/IP-journey-j45-patient-portal-auth.md`: added DR posture.
- `microservices/identity/IP-journey-j46-patient-prescriber-resolution.md`: added DR posture, Pod runtime tier.
- `microservices/identity/IP-journey-j50-helper-provisioning.md`: added DR posture.
- `microservices/identity/IP-journey-j54-buyer-principal.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j56-candidate-verification.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j57-work-principal.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j58-role-and-level-update.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j59-sso-disable.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j60-principal-hierarchy.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j61-patient-principal.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j62-patient-and-prescriber-scope.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j64-cross-tenant-principals.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j65-subject-verification.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j67-scope-validation.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j68-scoped-read.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j69-delegated-token.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j70-human-finalizer-scope.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j72-locale-and-consent.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j73-publisher-validation.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/identity/IP-journey-j74-cedar-scope-grants.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/identity/IP-journey-j76-principal-and-authz-gate.md`: added API Versioning.
- `microservices/identity/IP-journey-j77-principal-and-authz-gate.md`: added API Versioning, DR posture.
- `microservices/identity/IP-journey-j80-principal-and-authz-gate.md`: added API Versioning.
- `microservices/identity/IP-journey-j81-principal-and-authz-gate.md`: added API Versioning.
- `microservices/identity/IP-journey-j82-principal-and-authz-gate.md`: added API Versioning, DR posture.
- `microservices/identity/IP-journey-j83-principal-and-authz-gate.md`: added API Versioning.
- `microservices/identity/IP-journey-j84-principal-and-authz-gate.md`: added API Versioning, DR posture.
- `microservices/identity/IP-journey-j85-principal-and-authz-gate.md`: added API Versioning.
- `microservices/identity/IP-journey-j86-principal-and-authz-gate.md`: added API Versioning, DR posture.
- `microservices/identity/IP-journey-j87-principal-and-authz-gate.md`: added API Versioning.
- `microservices/identity/IP-journey-j88-principal-and-authz-gate.md`: added API Versioning.
- `microservices/identity/IP-journey-j89-principal-and-authz-gate.md`: added API Versioning.
- `microservices/identity/IP-journey-j90-principal-and-authz-gate.md`: added API Versioning.
- `microservices/identity/IP-journey-j91-us-msb-mtl-overlay.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j92-br-lgpd-us-parent-dsar.md`: added Sustainability emission.
- `microservices/identity/IP-journey-j93-in-dpdpa-rbi-overlay.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j94-sox404-public-company-controls.md`: added DR posture, Sustainability emission.
- `microservices/identity/IP-journey-j95-iso27001-soc2-annual-audit.md`: added Sustainability emission.
- `microservices/identity/IP-journey-j96-ksa-uae-mena-onboarding.md`: added Sustainability emission.
- `microservices/identity/IP-journey-j97-sg-pdpa-mas-tenant.md`: added Sustainability emission.
- `microservices/identity/IP-journey-j98-au-privacy-apra-cps234.md`: added Sustainability emission.
- `microservices/identity/IP-journey-j99-multi-pack-conflict-resolution.md`: added Sustainability emission.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: PRD now records RTO 1800 s / RPO 0 s from `manifest.json#dr`, preserves the stricter legacy realtime `rpo_rto` note, cites HIPAA/PCI/SOC2 floors, names `runbooks/idp-failover-drill.md`, and states same-pack active-active per ADR-0343. Alternative rejected: generic 15 min auth restore without RPO-0 commitment, because tenant-visible login and JWKS outages break every product. Cost: higher warm identity capacity and HSM/OpenBao key readiness in each pack.
- Capacity model: PRD now ties manifest values 0.18 vCPU, 256 MiB RAM, 2 GB storage, connections `{valkey:4, postgres:2, outbound_http:5}`, per-request scaling, Tier-0 cell placement, and ADR-0338 Tier-1 runtime to ADR-0340. Alternative rejected: per-service shared pool with no tenant quota, because login surges would starve other tenants. Cost: more per-pack app replicas and Postgres/pgcat capacity.
- Sustainability + cost attribution: PRD now requires `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, and `region` on identity audit rows per ADR-0344, with carbon routing excluded for live auth and emergency/HIPAA flows. Alternative rejected: aggregate-only auth emissions, because per-seat/per-usage identity billing must reconcile. Cost: audit payload growth and FinOps rollup work.
- API versioning posture: PRD now adopts the ADR-0342 date carrier triplet, SDK semver, N=3 / 180-day window, B2B per-tenant pinning, and ADR-0145 mesh exemption. Alternative rejected: SDK semver-only contracts, because SCIM/OIDC tenants need API-date pinning. Cost: version-routing tests and support for three live public versions.
## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: baseline_cpu_per_tenant 0.18 vCPU; baseline_ram_per_tenant 256 MiB; storage_per_tenant 2 GB; connections valkey=4, postgres=2, outbound_http=5; scaling_dimension per_request; cell_placement_class Tier-0.
- ADR: ADR-0340 capacity_model; ADR-0248 cellular criticality numbering.
- Why: Token issuance, WebAuthn ceremonies, SCIM synchronization, step-up checks, and principal resolution are foundation-critical but mostly cache-backed request work with small credential metadata writes.
- Rejected: cell_placement_class=Tier-2 because identity controls foundation authentication boundaries and must stay in the Foundation cell class even though several non-foundation APIs share the service.
- Cost: Reserves per-tenant cache and credential-store headroom and keeps identity on the higher-cost foundation placement path.

### Block 2: dr
- Values: rto_p99_seconds 1800; rpo_p99_seconds 0; multi_region_active_active true; backup_substrate postgres_wal_g, valkey_cluster, openbao_seal_unseal, object_storage_versioned; failover_runbook runbooks/idp-failover-drill.md; replication_shape active-active-multi-az-cross-region-warm.
- ADR: ADR-0343 DR RTO/RPO matrix and compliance-pack floors.
- Why: Identity carries authentication and tenant isolation blast radius, so HIPAA and KR-style one-hour floors are not enough for outage containment; zero RPO preserves token, credential, and federation continuity.
- Rejected: 24-hour PCI-only recovery because authentication downtime would strand every dependent tenant workflow before data restoration begins.
- Cost: Requires active-active identity state replication, key-unseal rehearsal, and warmer regional capacity than a cold restore plan.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier 1; evidence microservices/identity/PRD.md, microservices/identity/ARCHITECTURE.md, microservices/identity/IP-004-webauthn-relying-party-kernel.md, microservices/identity/IP-017-multi-context-principal-resolver.md, microservices/identity/runbooks/idp-failover-drill.md.
- ADR: ADR-0338 pod runtime tiering; ADR-0340 D-6 cell/runtime co-variance.
- Why: Identity is a foundation authentication substrate that touches tenant credentials, sessions, token claims, SCIM state, and federation metadata. It does not execute tenant-customer code, so Tier 0 is not warranted, but tenant-data-touching substrate isolation requires Tier 1 runtime placement.
- Rejected: pod_runtime_tier=2 because first-party implementation does not erase the tenant credential and token plane it operates on.
- Cost: Tier 1 commits identity pods to stronger runtime isolation and capacity overhead for auth-critical paths.

### Block 4: tenant_version_pinning
- Values: declared_versions 2026-05-21, 2026-02-21, 2025-11-21; default_version 2026-05-21; supported_window_size 3; supported_window_minimum_days 180; supports_per_tenant_pinning true.
- ADR: ADR-0342 hybrid date-versioned public API policy.
- Why: Identity exposes tenant-facing OIDC, SCIM, WebAuthn, and federation contracts, so date-version pinning protects tenant auth integrations from lockstep upgrades.
- Rejected: internal-only versioning because tenants and external IdPs bind to these contracts directly.
- Cost: Maintains three live contract windows and migration guidance for auth clients.

### Block 5: consumes_upstream_oss and oss_stewardship_class_overrides
- Values: consumes_upstream_oss postgresql, valkey, cedar, openbao, opentelemetry, istio, cilium, kyverno; oss_stewardship_class_overrides empty because registry-default stewardship applies.
- ADR: ADR-0345 OSS stewardship class and CVE response policy.
- Why: The service relies on registry-governed data, cache, policy, secret, telemetry, mesh, and admission substrates rather than local forks.
- Rejected: service-local stewardship overrides without a registry delta.
- Cost: CVE response follows registry owner teams and identity must track pin updates without changing stewardship class.

### Block 6: iac_module_invocations
- Values: oci-guest/postgresql-cluster@v1, oci-guest/always-free/valkey-cluster@v1, on-prem/service-mesh-waypoint@v1, oyatie-as-cloud-provider/openbao-secret-binding@v1.
- ADR: ADR-0339 shared IaC module library.
- Why: Identity needs reusable database, cache, mesh, and secret bindings in every supported deployment context.
- Rejected: service-local bespoke Terraform because ADR-0339 centralizes shared primitives for drift control.
- Cost: Identity upgrades now inherit shared module pin and compatibility testing cadence.
