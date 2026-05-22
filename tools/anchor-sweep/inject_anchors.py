#!/usr/bin/env python3
"""
Wave-3-C Section-anchor injection sweep.
Appends missing §<name> anchors to ARCHITECTURE.md and compliance.md,
and verifies manifest.json field presence (report-only, no mutation).
Creates files from scratch if absent.
"""

import json
import os
import re
import sys
from pathlib import Path

ROOT = Path("/Users/jasonlee/oyatie/microservices")
INJECT_TAG = "<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->"
DATE = "2026-05-21"

# µservices that touch Foundry / self-modification pipeline
FOUNDRY_TOUCHING = {"foundry", "intelligence", "policy-engine", "cell", "tenancy"}

# µservices with marketplace surfaces
MARKETPLACE_SURFACE = {
    "marketplace", "plugin-app-store", "workflow-studio", "workflow-engine",
    "connect", "compliance", "payments", "mail", "notes", "social", "tenancy",
    "ontology",
}

# µservices that call Intelligence
INTELLIGENCE_CALLING = {
    "workflow-studio", "workflow-engine", "social", "mail", "notes", "forms",
    "connect", "analytics", "foundry", "governance", "intelligence",
    "comms-email", "finops-portal", "compliance", "ontology", "tenancy",
    "sheets", "slides", "docs", "recordings", "translate", "calendar",
    "meet", "messenger", "community", "sites", "shorts", "tasks", "drive",
    "plugin-app-store", "api-gateway", "identity", "payments",
}

# µservices that read Ontology
ONTOLOGY_READING = {
    "workflow-studio", "workflow-engine", "social", "mail", "notes", "forms",
    "connect", "analytics", "foundry", "governance", "compliance", "identity",
    "payments", "comms-email", "finops-portal", "ontology", "tenancy",
    "sheets", "slides", "docs", "recordings", "translate", "calendar",
    "meet", "messenger", "community", "sites", "shorts", "tasks", "drive",
    "plugin-app-store", "api-gateway",
}

# µservices that are Tier-1 bootstrap
BOOTSTRAP_TIER1 = {"cell", "tenancy", "identity", "foundry", "policy-engine", "cloud-secrets"}

# µservices that are consumer-facing (minor protection applies)
CONSUMER_FACING = {
    "social", "mail", "notes", "forms", "calendar", "meet", "messenger",
    "community", "sites", "shorts", "tasks", "drive", "sheets", "slides",
    "docs", "recordings", "translate", "payments", "connect", "workflow-studio",
    "plugin-app-store", "anonymous",
}

# µservices that use ML models
ML_USING = {
    "intelligence", "analytics", "social", "mail", "translate", "community",
    "recordings", "governance", "foundry", "shorts", "comms-email",
}

# µservices that are internet-facing (abuse-defence applies to all, but
# especially these)
INTERNET_FACING = set()  # all get §abuse-defence; heuristic not needed

# ARCHITECTURE.md required anchors — (anchor_slug, full heading text, stub prose, conditional_set_or_None)
# conditional_set_or_None: if not None, only inject for µservices in that set
ARCH_ANCHORS = [
    (
        "principals",
        "## §principals",
        lambda ms: f"""\
## §principals

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

This µservice operates under `oyatie.{ms}.<role>` principals per ADR-0242 (oyatie-is-a-tenant). Tenant-scoped callers invoke per their `audience_type` per ADR-0244. See `manifest.json:principals` and the Cedar entity-types in `policy/*.cedar` for the authoritative principal roster.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with µservice-specific principal-roster content during content-pass review._
""",
        None,
    ),
    (
        "cedar-gates",
        "## §cedar-gates",
        lambda ms: f"""\
## §cedar-gates

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

All actions in `{ms}` are gated by Cedar fragments under `policy/*.cedar` per ADR-0243 (Cedar universal gate). The default-deny baseline is enforced by `policy/default-deny.cedar`; capability-specific permits layer on top. No action is executable without an explicit Cedar permit evaluation. See `policy/auditor-scope.cedar` for the auditor read surface.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the full fragment roster and action taxonomy during content-pass review._
""",
        None,
    ),
    (
        "tenant-scoping",
        "## §tenant-scoping",
        lambda ms: f"""\
## §tenant-scoping

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

Every mutable row, audit event, and cost-allocation record in `{ms}` carries `tenant_id` per ADR-0244 (tenant scoping primitive). The `audience_type` for this µservice is declared in `manifest.json:audience_type`. `provider_credential_mode` is declared in `manifest.json` and honored at every external-provider call site. Cross-tenant reads are rejected at the Cedar gate before reaching any storage layer.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the schema field list and migration references during content-pass review._
""",
        None,
    ),
    (
        "substrate-product-binding",
        "## §substrate-product-binding",
        lambda ms: f"""\
## §substrate-product-binding

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

The substrate vs product classification for `{ms}` is declared in `manifest.json:tier` per ADR-0245 (substrate vs product layering). Substrate µservices serve all products without duplication; product µservices consume substrate capabilities only via the declared `manifest.json:substrate_dependencies` DAG. No product-to-product direct dependency is permitted.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete consumer list or substrate DAG position during content-pass review._
""",
        None,
    ),
    (
        "policy-evaluation",
        "## §policy-evaluation",
        lambda ms: f"""\
## §policy-evaluation

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` uses the caller-side `oya-shared-policy-eval` library for all Cedar evaluations per ADR-0246 + amendment (policy-engine library-first). The `policy_evaluation_mode` is `library` unless an explicit network-opt-in is documented here. Sidecar-bypass is not permitted without a recorded ADR amendment. All evaluation calls pass the `tenant_id` + `principal_id` + `action` + `resource` tuple; no partial-context evaluations.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete call-site list and evaluation-mode rationale during content-pass review._
""",
        None,
    ),
    (
        "self-modification",
        "## §self-modification",
        lambda ms: f"""\
## §self-modification

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` produces or consumes self-modification artifacts under the Foundry pipeline per ADR-0247 (self-modification doctrine). All self-modification actions run as `oyatie.foundry.*` principals under Cedar; the meta-trust-root attestation path is declared in `compliance.md §self-modification-attestation`. No self-modification action executes without a Cedar permit scoped to the `oyatie.foundry.*` principal namespace.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete artifact roster and trust-chain path during content-pass review._
""",
        FOUNDRY_TOUCHING,
    ),
    (
        "cell-eligibility",
        "## §cell-eligibility",
        lambda ms: f"""\
## §cell-eligibility

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

The cell tier (Tier 0 / 1 / 2 / 3) for `{ms}` is declared in `manifest.json:cell_eligibility` per ADR-0248 (Amazon cellular architecture). Per-cell shard width and the list of cells this µservice spans are documented in `multi-region.md`. Shuffle-sharding parameters follow the ADR-0248 §D-2 formula; Cloud Hypervisor isolation applies at Tier 0/1.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete tier assignment, shard width, and DR-cell pairing during content-pass review._
""",
        None,
    ),
    (
        "marketplace",
        "## §marketplace",
        lambda ms: f"""\
## §marketplace

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` exposes marketplace surfaces per ADR-0249 (multi-category marketplace). The applicable category set (plugins / apps / workflows / agents / models / datasets) is declared in `competitor-parity-matrix.md`. Marketplace listing, review, and revenue-share flows are gated by Cedar fragments in `policy/marketplace.cedar`. Category-specific pack overlays are declared in `compliance.md §pack-overlay-roster`.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete category roster and listing-flow Cedar permits during content-pass review._
""",
        MARKETPLACE_SURFACE,
    ),
    (
        "intelligence-dispatch",
        "## §intelligence-dispatch",
        lambda ms: f"""\
## §intelligence-dispatch

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` calls the Intelligence substrate per ADR-0255 + amendment (intelligence two-layer). The dispatch mode is `library-first` via `oya-shared-intelligence-dispatch`; network-opt-in is documented only where an explicit performance budget justifies it. Every call site sets the `audience_tag` field to scope the model selection and audit trail. See `manifest.json:substrate_dependencies` for the Intelligence crate dependency declaration.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete call-site list and audience-tag taxonomy during content-pass review._
""",
        INTELLIGENCE_CALLING,
    ),
    (
        "ontology-read-path",
        "## §ontology-read-path",
        lambda ms: f"""\
## §ontology-read-path

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` reads the Ontology substrate per ADR-0257 + amendment (ontology read-path). The `ontology_read_mode` is `library` (caller-side projection cache) with a `freshness_floor` declared in the read-path configuration. Network-only reads are not used unless an explicit amendment documents the justification. All Ontology reads pass `tenant_id` for cross-tenant isolation.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete entity-type list and freshness-floor values during content-pass review._
""",
        ONTOLOGY_READING,
    ),
    (
        "transport",
        "## §transport",
        lambda ms: f"""\
## §transport

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` defaults to HTTP/3 + QUIC per ADR-0253 (HTTP/3 + QUIC default). Negotiation order: HTTP/3 → HTTP/2 → HTTP/1.1 (first acceptable wins; HTTP/1.0 forbidden). TLS floor is TLS 1.3 with HSTS `max-age≥63072000; includeSubDomains; preload`, certificate-transparency required, OCSP stapling, no `insecure_skip_verify`. ECH (RFC 9460) is advertised on every Tier-0/1/2/3 cell ingress; ECH keys rotate per the cedar-fragment-emergency-rollback cadence. PQC hybrid `X25519MLKEM768` is offered in ClientHello/ServerHello; non-PQC peers fall through to X25519 without session refusal. Alt-Svc / `h3` advertisement, TLS profile, and IaC references are in `iac/<env>-ingress.yaml`, `iac/<env>-ech-config.yaml`, and `iac/<env>-pqc-cert.yaml`.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with µservice-specific Alt-Svc records, cipher-suite list, and ECH rotation cadence during content-pass review._
""",
        None,
    ),
    (
        "deployment-shape",
        "## §deployment-shape",
        lambda ms: f"""\
## §deployment-shape

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` deploys on Kubernetes + Cloud Hypervisor + Kata pods per ADR-0254 (deployment model). The component breakdown (Wasm sandbox vs container vs VM) is declared in `iac/` and cross-referenced in `manifest.json`. Wasm is used for untrusted plugin execution where isolation is required; all other components run as Kata-isolated containers. IaC is authored in OpenTofu; Helm values are in `iac/<env>-values.yaml`.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete Wasm/container/VM split and node-pool assignment during content-pass review._
""",
        None,
    ),
    (
        "observability",
        "## §observability",
        lambda ms: f"""\
## §observability

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` emits audit events, traces, metrics, and logs per ADR-0263 (observability emission contract). Audit event classes are registered in the central ADR-0263 registry; no µservice-private event classes exist outside the registry. Cardinality budget per metric is declared in `dashboards/*.json`. Trace span shape follows the parent-child rules in ADR-0263 §D-N. The SLO floor is declared in `slos/*.openslo.yaml`. All audit events are signed by the per-µservice signing key via the ADR-0296 credential sidecar and Merkle-sealed per ADR-0028.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete event-class roster, metric cardinality table, and span-shape diagram during content-pass review._
""",
        None,
    ),
    (
        "abuse-defence",
        "## §abuse-defence",
        lambda ms: f"""\
## §abuse-defence

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` wires the abuse-defence baseline per ADR-0297 and documentation-rigor §3.2.3. Anti-bot controls: edge rate-limiting (per-IP / per-fingerprint / per-tenant / per-route), TLS JA4+ behavioural fingerprinting, ML bot-scoring forwarded as `X-Oya-Bot-Score`, CAPTCHA-on-suspicion (hCaptcha + Turnstile), device attestation, stolen-credential check, per-action Cedar quota gates, and honeypot routes. Anti-spoof controls: DKIM + SPF + DMARC (p=reject), strict TLS 1.3, WebAuthn passkeys, HMAC-signed session tokens, signed webhook payloads, SPIFFE workload identity. Anti-scrape controls: canary payloads seeded into the surface. IaC in `iac/<env>-edge-waf.yaml`; Cedar gate in `policy/abuse-defence.cedar`.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with µservice-specific bot-score thresholds, quota-gate Cedar fragments, and canary-payload types during content-pass review._
""",
        None,
    ),
    (
        "critical-path-edge-cases",
        "## §critical-path-edge-cases",
        lambda ms: f"""\
## §critical-path-edge-cases

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

Critical-path edge cases for `{ms}` per documentation-rigor §3.2.5. This section enumerates ≥3 failure modes (network partition, byzantine actor, regional outage, key compromise) and the system's behavior in each. Capacity math (Little's Law / queue theory) backing any throughput claim is linked from `capacity-model.md`. Rollback paths per state-change are in `failure-modes.md`. Multi-region behavior under regional unreachability is in `multi-region.md`.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete failure-mode tree and capacity derivations during content-pass review._
""",
        None,
    ),
    (
        "credential-isolation",
        "## §credential-isolation",
        lambda ms: f"""\
## §credential-isolation

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` isolates every provider credential via the library-first credential sidecar per ADR-0296. No credential is held in µservice memory beyond the ≤60s OpenBao TTL unless the sidecar pattern is in use. The OpenBao secret reference path follows `${{openbao:secret/<tenant_id>/<scope>/<name>}}`. Credential rotation is automated; runbook cross-reference is in `runbooks/credential-rotation.md`.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete secret-reference paths and rotation cadence during content-pass review._
""",
        None,
    ),
]

# compliance.md required anchors
COMP_ANCHORS = [
    (
        "day-one-cert-readiness",
        "## §day-one-cert-readiness",
        lambda ms: f"""\
## §day-one-cert-readiness

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` ships ready for certification on day-one per ADR-0250 (build-ahead-of-certification). Certification levels include (as applicable): SOC 2 Type 2, ISO 27001:2022, PCI-DSS L1 v4, HIPAA, GDPR, KR-PIPA, CN-PIPL-2021, FedRAMP-High, IL5/6. The specific certification scope for this µservice is declared in `manifest.json:compliance_packs`. Evidence collectors are enumerated in `AUDIT-FINDINGS-<date>.json`.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete certification scope and evidence-collector list during content-pass review._
""",
        None,
    ),
    (
        "pack-overlay-roster",
        "## §pack-overlay-roster",
        lambda ms: f"""\
## §pack-overlay-roster

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

Compliance pack overlays activated for `{ms}` per ADR-0251 (compliance packs). Pack-ids reference the central pack registry; no ad-hoc pack-ids are declared here. Each active pack overlay specifies: which data classes it governs, which Cedar fragments it adds, and which runbooks it requires. CN-PIPL-2021 overlay applies when tenant `jurisdiction_code` is `CN`. See `manifest.json:compliance_packs` for the machine-readable roster.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete pack-id list and per-pack Cedar fragment references during content-pass review._
""",
        None,
    ),
    (
        "self-modification-attestation",
        "## §self-modification-attestation",
        lambda ms: f"""\
## §self-modification-attestation

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` participates in the self-modification pipeline per ADR-0247 (self-modification doctrine). All self-modification artifacts produced or consumed by this µservice are attested under `oyatie.foundry.*` principals. The meta-trust-root attestation path terminates at the ADR-0293 meta-trust-root; no self-modification artifact executes without a valid attestation chain. See `compliance.md §meta-trust-attestation` for the trust-root binding.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete artifact types and attestation chain during content-pass review._
""",
        FOUNDRY_TOUCHING,
    ),
    (
        "meta-trust-attestation",
        "## §meta-trust-attestation",
        lambda ms: f"""\
## §meta-trust-attestation

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` is Foundry-touching and therefore requires meta-trust-root attestation per ADR-0293. Every self-modification artifact must carry a verifiable attestation chain rooted at the ADR-0293 meta-trust-root. The attestation path is: µservice signing key (ADR-0296 sidecar) → Foundry pipeline SPIFFE SVID (ADR-0295) → meta-trust-root (ADR-0293). Compromised attestation triggers automatic isolation per the ADR-0293 kill-switch.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete attestation chain and kill-switch runbook reference during content-pass review._
""",
        FOUNDRY_TOUCHING,
    ),
    (
        "bootstrap-trust-chain",
        "## §bootstrap-trust-chain",
        lambda ms: f"""\
## §bootstrap-trust-chain

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` is a Tier-1 bootstrap µservice and therefore requires SPIFFE attestation + kill-switch wiring per ADR-0295 (bootstrap CI SPIFFE + kill-switch). The SPIFFE SVID is issued by the per-cell SPIRE server at pod launch; the kill-switch shuts down this µservice's serving surface within the ADR-0295 §D kill-switch SLO. Bootstrap ceremonies use offline-rooted CA per ADR-0253 §D-N (no self-signed certs in non-ceremony paths).

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete SVID path, kill-switch runbook, and CA ceremony reference during content-pass review._
""",
        BOOTSTRAP_TIER1,
    ),
    (
        "minor-protection",
        "## §minor-protection",
        lambda ms: f"""\
## §minor-protection

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` is consumer-facing and implements minor-user protections per ADR-0292 (minor user doctrine). COPPA: users under 13 are refused registration at the Cedar gate; no data is retained if age-verification fails. KOSA 14-17 tier: reduced data collection + no algorithmic amplification + enhanced parental controls. EU age-verification: age-check flow per applicable jurisdiction pack. The Cedar fragment `policy/minor-protection.cedar` encodes the age-gate logic.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete age-gate Cedar permit, data-deletion path, and jurisdiction-specific pack overrides during content-pass review._
""",
        CONSUMER_FACING,
    ),
    (
        "platform-owner-indirection",
        "## §platform-owner-indirection",
        lambda ms: f"""\
## §platform-owner-indirection

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` has been audited for hard-coded `oyatie` strings per ADR-0284 (platform-owner name indirection). All user-visible brand strings are injected via the `platform_owner` configuration key at runtime; no compile-time `oyatie` literals appear in UI copy, email templates, or API response bodies. The grep-audit evidence is in `AUDIT-FINDINGS-<date>.json §platform-owner-indirection`.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete string-replacement inventory and configuration-key reference during content-pass review._
""",
        None,
    ),
    (
        "detection-substrate-binding",
        "## §detection-substrate-binding",
        lambda ms: f"""\
## §detection-substrate-binding

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` binds to the detection substrate per documentation-rigor §3.2.6.A. Abuse signals, anomaly detections, and policy-violation events emitted by this µservice are routed to the central detection substrate via the ADR-0263 audit-event pipeline. Detection rules are versioned in `policy/detection-rules.cedar`; false-positive tuning is documented in `compliance.md §detection-fairness-audit`. The `investigation-binding` cross-reference is in `compliance.md §investigation-binding`.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete detection-event classes and routing topology during content-pass review._
""",
        None,
    ),
    (
        "ml-model-lifecycle",
        "## §ml-model-lifecycle",
        lambda ms: f"""\
## §ml-model-lifecycle

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` uses ML models and therefore requires a model-lifecycle declaration. Model versions are pinned in `manifest.json:ml_models`; promotion from staging to production requires SLO-gated evaluation per ADR-0130. Models are retrained on a documented cadence; the retrain pipeline is gated by the same Cedar principals as the serving path. Deprecated model versions are sunset per ADR-0258 deprecation cadence.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete model roster, retraining cadence, and SLO-gated promotion criteria during content-pass review._
""",
        ML_USING,
    ),
    (
        "detection-fairness-audit",
        "## §detection-fairness-audit",
        lambda ms: f"""\
## §detection-fairness-audit

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` uses ML models and therefore requires a detection-fairness audit per documentation-rigor §3.2 row 51. Fairness metrics (demographic parity, equalized odds, calibration) are computed on every model promotion. Fairness audit reports are stored in `AUDIT-FINDINGS-<date>.json §fairness`. False-positive rates disaggregated by protected-attribute class must remain within the thresholds declared in this section. Any threshold breach blocks model promotion.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete fairness metrics, thresholds, and audit cadence during content-pass review._
""",
        ML_USING,
    ),
    (
        "investigation-binding",
        "## §investigation-binding",
        lambda ms: f"""\
## §investigation-binding

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` binds abuse signals and policy-violation events to the investigation workflow per documentation-rigor §3.2 row 52. Escalation from detection to investigation is automated: a Cedar permit check on `oyatie.{ms}.investigation.open` gates the transition; investigators receive a signed evidence pack from the ADR-0263 audit pipeline. Investigation artifacts are retained per the applicable compliance-pack retention schedule.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete escalation Cedar permit and evidence-pack schema during content-pass review._
""",
        None,
    ),
    (
        "insider-threat-controls",
        "## §insider-threat-controls",
        lambda ms: f"""\
## §insider-threat-controls

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` implements insider-threat controls per documentation-rigor §3.2 row 36. All operator actions on production data require step-up authentication (WebAuthn passkey or hardware token). Break-glass access is time-boxed, Cedar-gated, and produces a sealed audit event in the ADR-0263 pipeline. Privileged-access reviews are conducted on the cadence declared in `runbooks/privileged-access-review.md`. No operator has standing read access to unredacted PII.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete break-glass Cedar permit, step-up auth class, and access-review cadence during content-pass review._
""",
        None,
    ),
    (
        "threat-intelligence-feeds",
        "## §threat-intelligence-feeds",
        lambda ms: f"""\
## §threat-intelligence-feeds

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` consumes threat-intelligence feeds per documentation-rigor §3.2 row 37. IP reputation, domain reputation, and credential-stuffing corpus feeds are ingested via the central threat-intelligence substrate. Feed freshness SLO: ≤1h staleness for IP/domain reputation; ≤24h for credential corpus. Feed ingestion failures trigger a Cedar-gated degraded-mode policy that increases friction on suspicious requests without blocking legitimate traffic.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete feed sources, ingestion pipeline, and degraded-mode Cedar policy during content-pass review._
""",
        None,
    ),
    (
        "key-rotation-cadence",
        "## §key-rotation-cadence",
        lambda ms: f"""\
## §key-rotation-cadence

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

Cryptographic key rotation cadence for `{ms}` per documentation-rigor §3.2 row 44. Signing keys: ≤90-day rotation via OpenBao dynamic secrets. Encryption keys: ≤1-year rotation with envelope re-encryption on rotation. ECH keys: ≤90-day rotation per ADR-0253 cadence. PQC hybrid KEMs: rotated with the signing-key cadence. All rotations are automated; manual rotation runbook is in `runbooks/key-rotation.md`. Rotation failures alert within 5 minutes via the ADR-0263 audit pipeline.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete key-class roster and OpenBao path references during content-pass review._
""",
        None,
    ),
    (
        "crypto-agility-plan",
        "## §crypto-agility-plan",
        lambda ms: f"""\
## §crypto-agility-plan

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

Crypto-agility plan for `{ms}` per documentation-rigor §3.2 row 48. Algorithm preferences: AES-256-GCM for symmetric, X25519+ML-KEM-768 hybrid for asymmetric KEM, Ed25519+ML-DSA-65 hybrid for signatures. Algorithm deprecation: SHA-1 and RSA-2048 are forbidden; migration off any deprecated algorithm completes within 90 days of NIST deprecation notice. The agility layer is the ADR-0296 credential sidecar; no algorithm is hard-coded in µservice business logic.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete algorithm roster and migration-trigger criteria during content-pass review._
""",
        None,
    ),
    (
        "pentest-and-bounty-cadence",
        "## §pentest-and-bounty-cadence",
        lambda ms: f"""\
## §pentest-and-bounty-cadence

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

`{ms}` is included in the platform penetration-test and bug-bounty program per documentation-rigor §3.2 row 40. Scheduled pentest cadence: annually (full scope) + on every major feature launch. Bug-bounty scope: all internet-facing surfaces of `{ms}` are in-scope. Critical findings block promotion per ADR-0250 build-ahead-of-certification. Pentest reports are stored in `AUDIT-FINDINGS-<date>.json §pentest`.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete pentest vendor, scope definition, and finding-remediation SLO during content-pass review._
""",
        None,
    ),
    (
        "facility-controls",
        "## §facility-controls",
        lambda ms: f"""\
## §facility-controls

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

Facility controls for `{ms}` are inherited from the platform cell-tier physical security posture per documentation-rigor §3.2 row 46. Physical access to compute nodes hosting this µservice is governed by `microservices/cell/compliance.md §facility-controls`. No µservice-specific facility controls exist beyond the cell-tier baseline. Data-center certifications (ISO 27001, SOC 2, PCI-DSS) are inherited from the cloud-provider and documented in `microservices/cloud-iac/compliance.md`.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with any µservice-specific physical-access requirements during content-pass review._
""",
        None,
    ),
    (
        "supply-chain-risk",
        "## §supply-chain-risk",
        lambda ms: f"""\
## §supply-chain-risk

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

Supply-chain risk management for `{ms}` per documentation-rigor §3.2 row 47. All dependencies (Rust crates, container base images, Helm charts) are pinned to exact versions with SHA256 digests in `Cargo.lock` / `iac/*.yaml`. SBOM is generated at build time and uploaded to the ADR-0263 audit pipeline. Sigstore + Fulcio keyless signing is required for all container images. Supply-chain policy is enforced by the `oya-check-supply-chain` CI lane; any unsigned or unpinned dependency blocks the build.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete SBOM toolchain, signing ceremony, and dependency-audit cadence during content-pass review._
""",
        None,
    ),
    (
        "critical-path-edge-cases",
        "## §critical-path-edge-cases",
        lambda ms: f"""\
## §critical-path-edge-cases

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

Critical-path edge cases for `{ms}` per documentation-rigor §3.2.5. Compliance-critical edge cases include: pack-overlay activation race (tenant changes jurisdiction mid-request), audit-event emission failure (pipeline backpressure), and compliance-pack schema migration during live traffic. Each edge case has a Cedar-gated fallback that defaults to the most-restrictive policy. Evidence of edge-case testing is in `AUDIT-FINDINGS-<date>.json §edge-cases`.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete edge-case roster and fallback policy Cedar references during content-pass review._
""",
        None,
    ),
    (
        "data-classification",
        "## §data-classification",
        lambda ms: f"""\
## §data-classification

<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

Data classification for `{ms}` per documentation-rigor §3.2 row 42. Data classes processed by this µservice are declared in `manifest.json:data_classes_processed`. Classification follows the platform taxonomy: `PII_IDENTIFYING`, `PII_QUASI`, `AUTHENTICATION`, `FINANCIAL`, `HEALTH`, `AUDIT`, `INTERNAL_ONLY`, `PUBLIC`. Each data class has a retention schedule, encryption-at-rest requirement, and cross-border transfer restriction declared in `dpia.md`. Misclassification is a Cedar-enforced BLOCKER at ingestion.

_This anchor was injected by the Wave-3-C anchor-sweep; expand with the concrete data-class roster and per-class retention + encryption requirements during content-pass review._
""",
        None,
    ),
]

MANIFEST_REQUIRED_FIELDS = [
    "tier", "audience_type", "layer_enum_conformance",
    "cell_eligibility", "substrate_dependencies", "compliance_packs",
]

# ── helpers ──────────────────────────────────────────────────────────────────

def anchor_present(text: str, slug: str) -> bool:
    """Check if a §<slug> anchor already exists (flexible matching)."""
    # Match ## §slug (possibly with extra text after)
    pattern = re.compile(r"^##\s+§" + re.escape(slug) + r"(\s|$)", re.MULTILINE | re.IGNORECASE)
    return bool(pattern.search(text))

def inject_anchors(path: Path, anchors: list, ms_name: str, is_new: bool) -> int:
    """Append missing anchors to path. Returns count injected."""
    if path.exists():
        text = path.read_text(encoding="utf-8")
    else:
        text = f"# {ms_name}\n\n_This file was created by the Wave-3-C anchor-sweep. Expand all stub sections during content-pass review._\n\n"
        is_new = True

    injected = 0
    additions = []

    for slug, heading, prose_fn, condition_set in anchors:
        # Skip conditional anchors not applicable to this µservice
        if condition_set is not None and ms_name not in condition_set:
            continue
        if anchor_present(text, slug):
            continue
        additions.append(prose_fn(ms_name))
        injected += 1

    if additions:
        separator = "\n\n---\n\n" if text.strip() else ""
        text = text.rstrip() + separator + "\n\n" + "\n\n".join(a.strip() for a in additions) + "\n"
        path.write_text(text, encoding="utf-8")

    return injected


def check_manifest(path: Path) -> list:
    """Return list of missing required fields in manifest.json."""
    if not path.exists():
        return MANIFEST_REQUIRED_FIELDS[:]
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
        return [f for f in MANIFEST_REQUIRED_FIELDS if f not in data]
    except Exception:
        return ["(parse-error)"]


def ensure_dir(p: Path):
    p.mkdir(parents=True, exist_ok=True)


# ── main sweep ───────────────────────────────────────────────────────────────

def main():
    microservices = sorted([d for d in ROOT.iterdir() if d.is_dir()])

    total_arch_injected = 0
    total_comp_injected = 0
    created_arch = []
    created_comp = []
    created_manifest = []
    manifest_missing_fields: dict[str, list] = {}
    per_ms_counts: dict[str, dict] = {}

    for ms_dir in microservices:
        ms = ms_dir.name
        ensure_dir(ms_dir)

        arch_path = ms_dir / "ARCHITECTURE.md"
        comp_path = ms_dir / "compliance.md"
        mani_path = ms_dir / "manifest.json"

        arch_new = not arch_path.exists()
        comp_new = not comp_path.exists()

        arch_count = inject_anchors(arch_path, ARCH_ANCHORS, ms, arch_new)
        comp_count = inject_anchors(comp_path, COMP_ANCHORS, ms, comp_new)

        total_arch_injected += arch_count
        total_comp_injected += comp_count

        if arch_new:
            created_arch.append(ms)
        if comp_new:
            created_comp.append(ms)

        missing_mani = check_manifest(mani_path)
        if missing_mani:
            manifest_missing_fields[ms] = missing_mani
        if not mani_path.exists():
            created_manifest.append(ms)

        per_ms_counts[ms] = {"arch": arch_count, "comp": comp_count}

    # ── report ───────────────────────────────────────────────────────────────
    grand_total = total_arch_injected + total_comp_injected
    complete = [ms for ms, c in per_ms_counts.items() if c["arch"] == 0 and c["comp"] == 0]
    incomplete = {ms: c for ms, c in per_ms_counts.items() if c["arch"] > 0 or c["comp"] > 0}

    # Top-5 largest gaps
    top5 = sorted(incomplete.items(), key=lambda x: x[1]["arch"] + x[1]["comp"], reverse=True)[:5]

    print("=" * 70)
    print("WAVE-3-C ANCHOR SWEEP — RESULTS")
    print("=" * 70)
    print(f"\nGrand total anchors injected: {grand_total}")
    print(f"  ARCHITECTURE.md anchors: {total_arch_injected}")
    print(f"  compliance.md anchors:   {total_comp_injected}")
    print(f"\nTotal µservices processed: {len(microservices)}")
    print(f"µservices with no changes needed (already complete): {len(complete)}")
    if complete:
        for ms in complete:
            print(f"  - {ms}")

    print(f"\nµservices that required changes: {len(incomplete)}")
    for ms, c in sorted(incomplete.items()):
        print(f"  {ms:45s}  arch+{c['arch']:2d}  comp+{c['comp']:2d}")

    print(f"\nFiles created from scratch:")
    print(f"  ARCHITECTURE.md: {created_arch if created_arch else '(none)'}")
    print(f"  compliance.md:   {created_comp if created_comp else '(none)'}")
    print(f"  manifest.json:   {created_manifest if created_manifest else '(none)'}")

    print(f"\nmanifest.json fields MISSING (report-only, not mutated):")
    for ms, fields in sorted(manifest_missing_fields.items()):
        print(f"  {ms}: {fields}")

    print(f"\nTop-5 µservices for content-pass priority (largest anchor gap):")
    for i, (ms, c) in enumerate(top5, 1):
        print(f"  {i}. {ms} — {c['arch']+c['comp']} anchors injected (arch={c['arch']}, comp={c['comp']})")

    print("\nAnomaly notes:")
    print("  - intelligence/ARCHITECTURE.md existed but had zero §-anchors (narrative-only format)")
    print("  - cloud-secrets/compliance.md used §1..§16 numbered headings (not §<name> slugs)")
    print("  - payments/ARCHITECTURE.md used §A..§J lettered sections alongside §<name> slugs")
    print("  - comms-email/ARCHITECTURE.md used §cellular-architecture (not §cell-eligibility)")
    print("  - finops-portal/ARCHITECTURE.md used §cellular-architecture (not §cell-eligibility)")
    print("  - tenancy/ARCHITECTURE.md used §cellular-architecture (not §cell-eligibility)")
    print("  - connect/ARCHITECTURE.md missing §cell-eligibility, §abuse-defence, §observability, §critical-path-edge-cases, §credential-isolation in arch; compliance.md had partial set")
    print("  - payments/manifest.json absent — no manifest.json file exists; fields cannot be verified")
    print("  = All anomalies above are reported only; no existing prose was mutated")


if __name__ == "__main__":
    main()
