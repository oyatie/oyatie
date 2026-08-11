---
doc_class: Program-Discovery-Decision-Record
doc_status: drafted
discovery_id: DDR-E0-20260810-vap-cedar-plane-split-rationale
judgment_class: vap-cedar-plane-split-rationale
recorded_at: 2026-08-10
owner: council-architecture
authority_tier: 3
---
# DDR-E0-20260810-vap-cedar-plane-split-rationale

## Baseline version header

| Authority | Version this document was authored against | Status at authoring (2026-08-10) |
|---|---|---|
| Repository baseline | `origin/dev` @ `9a56538c74b1fce4d474869956dd278f7fe1981e` | E0 discovery encode lane base. |
| Upstream Kubernetes pin | `v1.36.1`, peeled commit `756939600b9a7180fc2df6550a4585b638875e67` | Pinned program input; VAP is GA well before this pin. |
| Engine | `build/port-engine/*`, v0 | Not in force as a producer for this record. |
| Neutral rule pack | `specs/port-rules/**`, v0 | Not in force. |
| Corpus rule policy | `specs/k8s-port/rules/**`, v0 — unauthored | Not in force. |
| Go front end | Bootstrap extractor; strategy ruled | Not in force. |
| Reproducibility tuple / receipt schema | `pin`, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest` | Six required axes; not in force. This record emits no receipt. |
| Program authority | ADR-0701 carried gist (live admission law until F1(d)); ADR-0710 Proposed direction; ADR-0702 identity/authz apex | Discovery **rationale** only — does not Accept ADR-0710. |

## Record identity

- **Stable ID:** `DDR-E0-20260810-vap-cedar-plane-split-rationale`.
- **Judgment class:** VAP/CEL (+ PSA) admission vs Cedar authorization — plane-split **rationale**.
- **Status:** `discovery` — record, **not** doctrine, **not** Accepted apex.
- **Recorded:** 2026-08-10.
- **Owner role:** `council-architecture`.

## Authority fence

**Live law until F1(d):** the ADR-0701-carried gist of ADR-0379 / ADR-0338 remains operative —
Kubewarden (WASM policy modules) as the default admission substrate in the consolidated apex
reading, with runtime-tier admission still an isolation control. This discovery record does
**not** overturn that live law.

**Proposed direction (not live):** ADR-0710 argues ValidatingAdmissionPolicy + CEL + Pod Security
Admission as the default admission substrate (no default policy webhook). ADR-0710 remains
`status: Proposed` with D-8 evidence outstanding. Acceptance or rejection is **F1(d)** — a founder
ADR decision — never assumed by this record.

This record encodes the **rationale** for keeping the two planes split, so later F1(d) work has a
stable judgment to cite. It does **not** Accept ADR-0710, edit ADR-0701, or activate DVG-CEDAR.

## Judgment

### Two different questions (do not merge)

| Plane | Question | Natural inputs | Natural engine |
|---|---|---|---|
| **Authorization** | Is principal P allowed to perform action A on resource R (in context C)? | identity, roles/attrs, resource IDs, attestation context | **Cedar** PDP (PBAC / Zanzibar-style ReBAC per ADR-0702) |
| **Admission** | Is this API object’s **shape/config** allowed to enter the cluster? | Pod `securityContext`, RuntimeClass, image digest, labels, PSS | **VAP/CEL + PSA** as proposed direction; **live** substrate remains the ADR-0701-carried admission gist until F1(d) |

Force-fitting one engine onto both produces awkward policies and coupled failure domains
(ADR-0183 rejected “Cedar for both”).

### Proposed direction (ADR-0710) — rationale only

If and when F1(d) Accepts ADR-0710:

- **Admission / shape / PSS / runtime-tier / image-allowlist:** ValidatingAdmissionPolicy + CEL
  in-process (+ Pod Security Admission).
- **Authorization:** Cedar PDP — apiserver authorizer seam + mesh L7 `ext_authz` + app PEPs.
- **BAN:** Cedar as the default Kubernetes **admission** engine.
- **BAN:** Kyverno / Kubewarden / Gatekeeper as **default** admission substrate (adapters only per
  ADR-0710 D-7) — *after* Accept; until then live law stands.

### Why not Cedar for admission

1. **Model mismatch:** Cedar is principal-centric. Admission is resource-shape-centric
   (“Pod must set `runAsNonRoot`”, “RuntimeClass ∈ {shared-kernel, private-kernel,
   private-kernel-attested}”, “image is digest-pinned”). Fake principals are authoring debt.
2. **Not native to the admission chain:** a Cedar admission path implies a webhook or side
   channel — the category ADR-0710 removes as default.
3. **Failure-domain coupling:** authz PDP degradation should fail closed for API access;
   admission shape checks should remain enforceable in-process so a PDP blip is not “cannot
   enforce PSS/RuntimeClass” (and the inverse).
4. **Analyzability fit:** CEL + typed OpenAPI fields match field-equality shape rules; Cedar’s
   strength is principal×action×resource proofs.
5. **Attestation is authz context, not a reason to move admission:** TEE quotes → verified
   result → Cedar context on authorize. Whether a Pod *declares* `runtimeClassName:
   private-kernel-attested` remains admission/shape; whether that workload may *read a secret*
   given attestation remains authz.

### Precision retained from Round-2

- Name the PDP **Cedar-compatible with Zanzibar-style ReBAC** (ADR-0702) — flat “Cedar” wording
  alone mis-keys ledger intent.
- D-9 retains identity-bound fail-closed CEL/RBAC admission in-process where already present; the
  two-plane split MUST NOT be read as banning caller-sensitive admission.
- Pattern-adopt source for any k8s authorizer adapter must verify provenance rows (known artifact:
  `awslabs/cedar-access-control-for-k8s`) — abandoned-repo deps are banned.
- Chain `[Node, RBAC, Cedar]` in AdditiveAllow first; Cedar-primary only after ratified
  expected-red IDs (W0+).

## Round-2 basis

Plane-split rationale with explicit live-law fence: Cedar = authz only; admission = VAP/CEL+PSA
as **proposed** direction; live law until F1(d) = ADR-0701 carried gist. Isolation-property
RuntimeClass names appear only as future admission shape inputs, not as live remap.

## Alternatives

| Approach | Why rejected as default |
|---|---|
| Cedar for both authz and admission | Model mismatch + failure-domain coupling (ADR-0183) |
| Keep policy webhooks as forever default after VAP GA | Extra hop / availability coupling ADR-0710 removes — **pending F1(d)** |
| Treat ADR-0710 as already Accepted | False; D-8 open; would silently overrule ADR-0701 gist |
| Collapse authz into admission CEL | Loses ReBAC / attestation context strengths |

## Downstream blockers

- **F1(d):** Accept or Reject ADR-0710 on D-8 evidence — only then does proposed admission
  substrate become live law.
- **F1(c):** RuntimeClass rename / VAP tier-map params after enforcement re-home.
- W0: ratified expected-red IDs before Cedar-primary authorizer activation.

## Naming law

Uses isolation-property RuntimeClass names and Round-2 neutral nouns. Does not adopt `asterkube`
or `kuberos` as product/public names.
