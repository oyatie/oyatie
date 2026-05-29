---
doc_class: MigrationPlaybook
microservice: contact-center
source_vendor: Genesys Cloud CX
related_adrs: [ADR-0316, ADR-0263, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Migration Playbook — Genesys Cloud CX → oyatie contact-center

Audience: a contact-center operations team currently on Genesys Cloud CX (formerly PureCloud) seat-based pricing who wants to move to oyatie's substrate over 10-16 weeks without an availability gap.

Outcome: all DIDs ported, all agents re-provisioned, all IVR flows translated + simulator-green, all recordings exported + retention re-anchored, Genesys decommissioned per minimum-commit.

## Phase 0 — discovery (week 1)

1. Inventory Genesys assets:
   - DIDs (`Admin → DID Numbers`).
   - Agents (`People & Permissions`).
   - Skills + skill groups.
   - Queues + ACDs.
   - Architect flows (export each as `*.flow.yaml`).
   - Permissions roles (`Admin → Roles`).
   - Recording-retention policies.
   - Integrations (CRM connectors, webhooks, BYO trunks).
   - Open API integrations + custom scripts.
2. Inventory contractual exposure:
   - Genesys contract end date.
   - Minimum-seat commit (you typically pay for committed seats whether used or not).
   - Per-seat pricing package (CX 1 / CX 2 / CX 3 / CX AI).
   - Outbound minute pool + overage rate.
3. Identify pack-bound priorities: any tenant pack requiring sovereign residency (KR-PIPA, CSAP) MUST migrate first because Genesys cannot host them.
4. Estimate target tenant_class: small (≤ 50 seats) → demo_trial; medium (50-500 seats) → paid; large (500-5000) → paid; sovereign → paid compliance-pack.

Deliverable: `migration-plan.md`.

## Phase 1 — stand up oyatie + DID porting (weeks 2-3)

1. Deploy oyatie contact-center IaC into the target cell per `iac/contact-center-paid-helm.yaml`.
2. Smoke-test with a test DID provisioned through Bandwidth.com (US) or Inteliquent: call the DID, verify it routes to oyatie's IVR. Expected p99 setup latency ≤ 250 ms.
3. Initiate DID porting from Genesys to oyatie's carrier (Bandwidth.com / Inteliquent for US; Deutsche Telekom for EU; KT for Korea):
   - Provide Genesys carrier's CSR (Customer Service Record).
   - Submit Letter of Authorization (LOA).
   - FCC LNP process takes 10-15 business days for US; KCC equivalent 7-10 days; OFCOM (UK) 10-15 days.
4. During porting, configure Genesys to forward incoming calls on the porting DIDs to oyatie's substrate via SIP REFER. This provides a soft cutover with no caller-experience disruption.

## Phase 2 — IVR flow translation (weeks 4-6)

Genesys Architect flows are YAML-shaped. Each flow has:
- `actions[]` — the sequence of nodes (play prompt, decision, transfer, etc).
- `variables[]` — flow-local variables.
- `data` — flow metadata.

oyatie's flow JSON has equivalent constructs but different names. The substrate provides a converter:

```sh
cargo run -p oya-dev-cli -- contact-center flow-import \
    --source genesys-architect \
    --input flows/genesys-billing.flow.yaml \
    --output microservices/contact-center/flows/billing-translated.json
```

The converter handles 80 % of node types automatically:

| Genesys node | oyatie node |
|---|---|
| `playAudio` | `play-prompt` |
| `getInput` (DTMF) | `collect-dtmf` |
| `transferToACD` | `route-to-queue` |
| `transferToUser` | `route-to-agent` |
| `decision` | `branch` |
| `runDataAction` | `http-call` |
| `disconnect` | `hangup` |
| `setVariable` | `set-variable` |
| `loop` | (manual; iterate via branch + state variable) |

Manual translation needed for:
- Genesys Bridge Actions (no direct equivalent; usually implemented via `http-call` to a tenant API).
- Genesys Surveys (no direct equivalent; we have a survey flow in the substrate but the JSON shape differs).
- Genesys MyMessages chat overflow (chat is on the `community` µservice in oyatie; flow stays in contact-center for voice, escalation goes to community).

After conversion, run the simulator on every flow:

```sh
cargo run -p oya-dev-cli -- contact-center flow-simulate \
    --tenant <tenant-id> \
    --flow billing-translated \
    --replay-from-genesys-call-recordings flows/genesys-billing-test-calls.zip
```

The simulator replays real Genesys recordings through the new flow and compares outputs. Discrepancies > 5 % require manual review.

## Phase 3 — agent provisioning (week 7)

Genesys roles don't auto-translate to Cedar roles. Map manually:

| Genesys role | oyatie Cedar role |
|---|---|
| Telephony Admin | `contact-center::admin` |
| Supervisor | `contact-center::supervisor` |
| Agent | `contact-center::agent` |
| Quality Manager | `contact-center::qm` |
| Workforce Manager | `contact-center::wfm` |

For each Genesys agent:
1. Create the agent in oyatie with the same email + name + hire date.
2. Bind the equivalent Cedar role.
3. Re-assign skills (Genesys skill IDs don't transfer; you must re-tag).
4. Send the WebRTC onboarding email. Agents must re-pair their headsets and run the browser diagnostics test (their Genesys WebRTC config doesn't migrate).

Bulk-create via CSV:

```sh
cargo run -p oya-dev-cli -- contact-center agent-bulk-create \
    --tenant <tenant-id> \
    --input agents.csv
```

CSV columns: `email,name,role,skills,supervisor_email,hire_date`.

## Phase 4 — queue + skill translation (week 8)

For each Genesys ACD queue, create the oyatie equivalent. The schema is similar but oyatie's queue config is more granular per-pack. Required mapping:

| Genesys field | oyatie field |
|---|---|
| `name` | `name` |
| `description` | `description` |
| `mediaSettings` | `media-policy` |
| `skillsRequired` | `skills-required` (with Cedar skill IDs) |
| `routingRules` | `routing-policy` |
| `bullseyeBands` | `bullseye-rings` |
| `whisperAudio` | `whisper-audio` |

Configure `recording-policy` per queue: for any queue handling payments → enable PCI suppression; for any queue handling health info → enable HIPAA-Provider retention overlay; for any queue handling EU customers → enable GDPR retention overlay.

## Phase 5 — recording archive (weeks 9-10)

Genesys recordings are accessible via the Recordings API. Bulk-export:

```sh
genesys-cli recordings bulk-export \
    --start-date 2023-01-01 \
    --end-date <cutover-date> \
    --format wav \
    --output ./genesys-recordings/
```

Upload to oyatie's WORM cold tier:

```sh
cargo run -p oya-dev-cli -- contact-center recordings-import \
    --tenant <tenant-id> \
    --source-format genesys-export \
    --input ./genesys-recordings/ \
    --target-retention 7y \
    --emit-audit-chain-anchor true
```

The import (a) writes each recording to SeaweedFS WORM Compliance, (b) cross-emits a Merkle-anchor to the `audit-chain` µservice, (c) records the chain-of-custody (who imported, when, source system + ID). This makes the migrated recordings indistinguishable from natively-captured oyatie recordings for compliance audit purposes.

Retention re-anchor: if Genesys recorded for 5 y and the source recording is from year 2 of that 5 y, oyatie inherits the remaining 3 y minimum + adds your pack's residency minimum (KR-PIPA: 5 y from import date; HIPAA-Provider: 7 y from import date).

## Phase 6 — cutover (weeks 11-12)

1. Confirm all DIDs ported (FCC porting confirmation per DID).
2. Confirm all flows live + simulator-green.
3. Confirm all agents provisioned + headset-paired.
4. Communications:
   - Agent-facing: 7 days advance notice + 24 h reminder; training session 3 days before cutover.
   - Customer-facing: no notice required (DID + IVR experience preserved by design).
5. Cutover sequence (typically 11 PM local on a Friday):
   1. Update each ported DID's SIP routing to point to oyatie's SBC.
   2. Disable Genesys agent logins.
   3. Re-route any in-flight Genesys calls to oyatie via SIP REFER.
   4. Monitor first 2 h for media quality + call setup latency. Page substrate-team oncall if MOS drops below 4.0.
6. Run dual-system overlap for 7 days: keep Genesys in a "receive-only" mode where the system is up but no DIDs route to it; this allows rapid rollback if oyatie has a critical issue.
7. After 7 clean days: cancel Genesys contract per minimum-commit.

## Phase 7 — Genesys wind-down (weeks 13-16)

1. Cancel agent licences.
2. Export remaining recordings (any captured in the 7-day overlap period).
3. Cancel BYO trunks (if any).
4. Receive final invoice; pay any minimum-commit residual.
5. Update tenant ARCHITECTURE.md § "Contact Center" to reference oyatie exclusively.

## Common pitfalls

| Pitfall | Mitigation |
|---|---|
| Genesys Architect "Bridge Actions" with no oyatie equivalent | Implement via `http-call` to a tenant API; budget extra weeks for custom action porting |
| Recordings with PII not encrypted at rest in Genesys | oyatie's WORM import auto-encrypts; document the import as a "key change of custody" in audit-chain |
| Agent-specific Genesys scripts (Pop-Up scripts) | No direct equivalent; rebuild as CRM screen-pops via Open-Cadence URL integration |
| Genesys CX AI features (Predictive Engagement, Empathy Detection) | Map to oyatie's intelligence µservice features; some Genesys-specific ML may not have direct equivalents — accept the gap or build via the `intelligence` µservice's custom model facility |
| Outbound calling DNC list | Export from Genesys, import via `cargo run -p oya-dev-cli -- contact-center dnc-import` |
| Workforce Management forecasts | Oyatie WFM is in `contact-center` and consumes Genesys WFM CSV exports via the `wfm-import` command |
| Surveys (post-call NPS / CSAT) | Survey flows live in oyatie's contact-center as a flow node; translate from Genesys Surveys schema |
