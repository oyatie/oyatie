---
doc_class: User-Journey-Handshake
journey_id: j172-lev-kahn-investor-relations-shareholder-meeting-livestream
date: 2026-05-20
authority_tier: 2
status: draft
---

# j172 — Handshake matrix

Every named µservice call for the 24-hour AGM cycle (May 20 04:48 CDT pre-meeting → 18:18 CDT post-meeting filings). Transport HTTPS over QUIC per ADR-0253. Reg FD simultaneous-disclosure gate enforced per ADR-0243 + ADR-0251. en-US + en-UK + zh-Hans + ko-KR + ja-JP + Russian + Hebrew preservation UTF-8 NFC byte-exact.

## Notation

- `[HEL]` Helios principal
- `[CS]` Computershare registrar principal
- `[CH]` Carl Hagberg inspector principal
- `[ABC]` ABC Linguistic Services interpreter principal
- `[SH]` Shareholder principal (institutional / retail / proxy advisor / press)
- `→` synchronous request
- `←` synchronous response
- `↪` side-effect on bus
- `⟂` denied path

## §1 Pre-meeting (04:48 CDT — 09:30 EDT)

### 1.1 IR command console open

`[HEL:lev.kahn] → ir-command` — `GET /v1/agm/command-console/open`

```json
{
  "principal": "lev.kahn@helios-industries-inc-nyse-hlos",
  "role_assertion": "investor_relations_director",
  "passkey_assertion_token": "wb-jwt-...",
  "yubikey_attestation": "yk-5c-nfc-lev-2025",
  "cfo_co_sign_delegation_token": "delegation-cfo-marguerite-9d-live-window-jwt"
}
```

Cedar: permit (ir_director + passkey + yubikey + cfo_co_sign). Audit: `EVT-J172-AGM-COMMAND-CONSOLE-OPENED-Δ000`.

### 1.2 ja-JP language stream activation (Nikkei accommodation)

`[HEL:lev.kahn] → meet` — `POST /v1/meet/agm/language-stream/activate`

```json
{
  "agm_session_id": "agm-helios-2027-fy2026",
  "language_code": "ja-JP",
  "interpreter_principal": "kazuhiko.yamamoto@abc-linguistic-services-geneva",
  "interpreter_credential": "IATTI-certified-2018",
  "interpretation_mode": "simultaneous",
  "target_region_cell": "apac-tokyo-tier-2-region-edge",
  "closed_caption_path": "auto_plus_interpreter_verified",
  "target_latency_ms": 170,
  "sla_window_utc": ["2027-05-20T13:25:00Z", "2027-05-20T15:00:00Z"],
  "authorization_chain": ["lev.kahn", "marguerite.vasquez-ortiz"]
}
```

Cedar: permit (ir_director + cfo_co_sign + interpreter_credential_valid). Audit: `EVT-J172-LANGUAGE-STREAM-ADDED-jaJP-Δ001a`.

### 1.3 Dry-run + Reg FD gate test

`[HEL:lev.kahn] → meet` — `POST /v1/meet/agm/dry-run`

```json
{
  "agm_session_id": "agm-helios-2027-fy2026",
  "test_class": "reg_fd_simultaneous_disclosure_gate",
  "test_scenarios": [
    {
      "scenario_id": "eps-synthetic-injection",
      "material_info_class": "preliminary_eps",
      "expected_window_ms": 200
    },
    {
      "scenario_id": "dividend-synthetic-injection",
      "material_info_class": "dividend_declaration",
      "expected_window_ms": 200
    }
  ],
  "iteration_count": 3
}
```

Audit: `EVT-J172-DRY-RUN-PASSED-Δ001b`.

## §2 Shareholder authentication wave (08:00 — 09:30 CDT)

### 2.1 Direct registered owner authentication

`[SH:any] → identity` — `POST /v1/identity/agm/auth/direct-registered`

```json
{
  "shareholder_id": "computershare-direct-reg-shareholder-987654",
  "agm_session_id": "agm-helios-2027-fy2026",
  "computershare_proxyview_sso_token": "cs-sso-jwt-...",
  "auth_method": "passkey_plus_proxyview_sso"
}
```

### 2.2 Beneficial-owner-via-broker authentication

`[SH:any] → identity` — `POST /v1/identity/agm/auth/street-name`

```json
{
  "shareholder_id": "beneficial-owner-schwab-Δ048210",
  "broker_tenant": "schwab",
  "broker_control_number": "CN-2027-05-20-Δ048210",
  "agm_session_id": "agm-helios-2027-fy2026",
  "broker_sso_token": "schwab-sso-jwt-...",
  "auth_method": "broker_sso_plus_control_number"
}
```

Audit: `EVT-J172-SHAREHOLDERS-AUTHENTICATED-002` (rolling event).

### 2.3 Institutional authentication via ProxyView

`[SH:wellington] → identity` — `POST /v1/identity/agm/auth/institutional`

```json
{
  "institutional_principal": "agm-rep@wellington-management",
  "institutional_holdings_records": [
    {"share_class": "common_A", "share_count": 184028}
  ],
  "computershare_proxyview_token": "cs-inst-sso-...",
  "agm_session_id": "agm-helios-2027-fy2026"
}
```

## §3 AGM livestream open (09:30 EDT)

### 3.1 Livestream open across 5 streams

`[HEL:lev.kahn] → meet` — `POST /v1/meet/agm/livestream/open`

```protobuf
message OpenLivestreamRequest {
  string agm_session_id = 1;                  // "agm-helios-2027-fy2026"
  repeated LanguageStream language_streams = 2;
  RegFDGateConfig reg_fd_gate = 3;
  string recording_worm_cell = 4;             // "us-east-tier-1-worm-sec-17a-4f"
  string recording_seal_class = 5;             // "sec-17a-4f-helios-class-A"
  google.protobuf.Timestamp opened_at_utc = 6;
}

message LanguageStream {
  string language_code = 1;
  string interpreter_principal = 2;
  string closed_caption_path = 3;
  string target_region_cell = 4;
  uint32 target_latency_ms = 5;
}

message RegFDGateConfig {
  bool armed = 1;
  uint32 window_ms = 2;                          // 200
  repeated string release_paths = 3;             // wire + EDGAR + IR-page + 5 caption streams
}
```

Audit: `EVT-J172-LIVESTREAM-OPENED-001`.

### 3.2 Real-time caption verification (rolling)

`meet → audit-chain` — internal RPC `Meet/EmitCaptionVerificationRollup`

```json
{
  "agm_session_id": "agm-helios-2027-fy2026",
  "verification_window_minute": 8,
  "per_language_wer": {
    "en-US": 0.008,
    "en-UK": 0.012,
    "zh-Hans": 0.034,
    "ko-KR": 0.028,
    "ja-JP": 0.021
  },
  "target_wer": 0.05,
  "status": "PASS",
  "emitted_at": "2027-05-20T13:38:00Z"
}
```

Audit: `EVT-J172-CAPTIONS-VERIFIED-003` (rolling event).

## §4 Reg FD simultaneous-disclosure gate fire (T+18:42 = 13:48:42Z)

### 4.1 Gate fire request

`[HEL:lev.kahn] (via auto-detector + interpreter cues) → meet` — `POST /v1/meet/reg-fd-gate/fire`

```protobuf
message RegFDGateFireRequest {
  string material_info_id = 1;                  // "eps-fy2026q1-preliminary"
  string material_info_class = 2;                // "preliminary_eps_GAAP_diluted"
  bytes sealed_material_envelope = 3;            // staged value $1.84 sealed
  string sealed_envelope_release_token = 4;     // released only when gate fires
  google.protobuf.Timestamp target_disclosure_utc = 5;
  uint32 window_tolerance_ms = 6;                // 200
  repeated string release_paths = 7;
  bool interpreter_cues_all_received = 8;        // true if all 3 interpreters tap MATERIAL UTTERANCE button
  bool auto_detector_armed = 9;
}

message RegFDGateFireResponse {
  string material_info_id = 1;
  google.protobuf.Timestamp actual_fire_utc = 2;
  uint32 actual_window_ms = 3;
  map<string, google.protobuf.Timestamp> per_path_fire_t = 4;
  string status = 5;                              // "PASS" | "FAIL_OUT_OF_WINDOW"
  string audit_event_id = 6;
}
```

Audit: `EVT-J172-REG-FD-SIMULTANEOUS-DISCLOSURE-006`.

## §5 Vote tally streaming + Merkle anchors (per item)

### 5.1 Voting open (item 1 — dividend declaration)

`[HEL:gc.lakshmi] → governance` — `POST /v1/governance/agm/voting/open`

```json
{
  "agm_session_id": "agm-helios-2027-fy2026",
  "proposal_id": "item-1-dividend-declaration",
  "share_classes": ["common_A", "common_B_founder"],
  "voting_window_utc": ["2027-05-20T13:54:18Z", "2027-05-20T14:00:00Z"],
  "rolling_tally_enabled": true,
  "dual_sign_required": true,
  "dual_signers": [
    "computershare-registrar-tally-service",
    "carl-hagberg-inspector-of-elections"
  ]
}
```

Audit: `EVT-J172-VOTING-OPENED-Δ005a`.

### 5.2 Rolling tally streaming

`governance → governance` — internal RPC `Governance/StreamRollingTally`

```protobuf
message RollingTallyEnvelope {
  string proposal_id = 1;
  string share_class = 2;                         // "common_A" | "common_B_founder"
  uint64 votes_pre_recorded_proxy = 3;
  uint64 votes_live_during_meeting = 4;
  uint64 total_votes = 5;
  uint64 in_favor = 6;
  uint64 against = 7;
  uint64 abstain = 8;
  google.protobuf.Timestamp tally_t = 9;
  string merkle_root = 10;                        // running per-class root
}
```

### 5.3 Dual-sign certification

`[CS:karen.adebola-park + CH:carl.hagberg] → governance` — `POST /v1/governance/agm/vote-tally/certify`

```json
{
  "proposal_id": "item-1-dividend-declaration",
  "share_class": "common_A",
  "final_in_favor": 3842768,
  "final_against": 432184,
  "final_abstain": 142023,
  "final_total": 4416975,
  "computershare_sign_t": "2027-05-20T14:00:08Z",
  "computershare_sign_principal": "karen.adebola-park@computershare-registrar-services",
  "carl_hagberg_sign_t": "2027-05-20T14:00:14Z",
  "carl_hagberg_sign_principal": "carl.hagberg@carl-hagberg-inspectors-of-elections",
  "merkle_root_per_share_class": "sha256:a1b3...ef21",
  "dual_sign_state": "certified_dual_signed"
}
```

Cedar: permit (computershare_registrar + carl_hagberg_inspector + dual_sign_required + merkle_anchor_emitted). Audit: `EVT-J172-VOTE-TALLY-005`.

### 5.4 Merkle anchor emission (per item per share class — 12 anchors total)

`governance → audit-chain` — internal RPC `AuditChain/EmitAGMAnchor`

```protobuf
message EmitAGMAnchorRequest {
  string anchor_id = 1;                          // "anchor-agm-helios-2027-item-1-common-A"
  string agm_session_id = 2;
  string proposal_id = 3;
  string share_class = 4;
  bytes merkle_root = 5;
  string external_transparency_log_batch = 6;
  ProofClass proof_class = 7;                     // INCLUSION_PROOF
  google.protobuf.Timestamp emitted_at = 8;
}
```

Audit: `EVT-J172-MERKLE-ANCHORS-007` (12 emissions).

## §6 Q&A queue + community filter

### 6.1 Question submission (institutional)

`[SH:wellington.david.park] → meet` — `POST /v1/meet/q-and-a/submit`

```json
{
  "agm_session_id": "agm-helios-2027-fy2026",
  "shareholder_principal": "agm-rep@wellington-management",
  "shareholder_class": "institutional",
  "question_text": "Your guidance for free cash flow conversion increased to 92%...",
  "question_topic_hint": "Margins",
  "submitted_at": "2027-05-20T14:32:18Z"
}
```

Audit: `EVT-J172-Q-AND-A-SUBMITTED-Δ004a` (per question).

### 6.2 Community-filtered retail question stream

`[SH:retail.margaret.k] → community` — `POST /v1/community/retail-q-and-a/submit`

```json
{
  "agm_session_id": "agm-helios-2027-fy2026",
  "retail_shareholder_principal": "margaret.k@schwab-bo-shareholder",
  "beneficial_holdings": 280,
  "broker_tenant": "schwab",
  "question_text": "With your industrial automation segment up 18% YoY, can the CIO share what the M&A pipeline looks like in robotic process automation?",
  "submitted_at": "2027-05-20T14:35:48Z"
}
```

### 6.3 Ombudsperson filter

`community → ombuds-filter-service` — internal RPC `OmbudsFilter/ReviewRetailQuestion`

```json
{
  "question_id": "q-retail-068",
  "civility_filter_result": "PASS",
  "reg_fd_filter_result": "PASS",
  "promotion_decision": "promote_to_primary_queue",
  "reviewer_principal": "naveen.iyer-krishnamurthy@helios-industries-inc-nyse-hlos",
  "reviewed_at": "2027-05-20T14:36:14Z"
}
```

Audit: `EVT-J172-COMMUNITY-RETAIL-FILTER-Δ004a`.

### 6.4 Promotion to primary queue

`community → meet` — internal RPC `Community/PromoteToPrimaryQueue`

```json
{
  "question_id": "q-retail-068",
  "agm_session_id": "agm-helios-2027-fy2026",
  "promoted_at": "2027-05-20T14:36:18Z",
  "topic_tag": "M&A",
  "suggested_respondent": "hideki.akiyama-holt@helios-industries-inc-nyse-hlos"
}
```

Audit: `EVT-J172-Q-AND-A-ROLLUP-004` (composite event by end of Q&A).

## §7 Livestream close + WORM seal

### 7.1 Livestream close

`[HEL:lev.kahn] → meet` — `POST /v1/meet/agm/livestream/close`

```json
{
  "agm_session_id": "agm-helios-2027-fy2026",
  "closed_at_utc": "2027-05-20T15:00:00Z",
  "peak_concurrent_viewers": 11948,
  "duration_minutes": 90.00
}
```

Audit: `EVT-J172-LIVESTREAM-CLOSED-Δ001c`.

### 7.2 SEC 17a-4(f) WORM seal (24 artifacts)

`[HEL:lev.kahn] → drive` — `POST /v1/drive/sec-17a-4f/seal`

```protobuf
message SEC17a4fSealRequest {
  string agm_session_id = 1;
  string worm_cell = 2;                          // "us-east-tier-1-worm-sec-17a-4f"
  string seal_class = 3;                          // "sec-17a-4f-helios-class-A"
  uint32 retention_years_minimum = 4;             // 6
  bool indelible_storage_attestation = 5;         // true
  string time_stamp_authority_id = 6;
  bool audit_trail_attached = 7;
  repeated ArtifactSealRequest artifacts = 8;
}

message ArtifactSealRequest {
  string artifact_id = 1;
  string filename = 2;
  uint64 size_bytes = 3;
  string sha256 = 4;
  string mime_type = 5;
  ArtifactClass artifact_class = 6;               // RECORDING | SLIDE_DECK | TRANSCRIPT | VOTE_TALLY_RECORD | DUAL_SIGN_ATTESTATION
  string language_code = 7;                       // for per-language artifacts
}
```

Audit: `EVT-J172-SEC-17A-4F-WORM-SEALED-008`.

## §8 Post-meeting filings

### 8.1 SEC Form 8-K filing (dividend declaration)

`[HEL:lakshmi.subramanian-brodsky] → sec-edgar-bridge` — `POST /v1/sec-edgar/form-8k/file`

```json
{
  "filing_id": "8k-helios-2027-05-20-dividend",
  "filer_cik": "0001234567",
  "filing_type": "8-K",
  "items": ["Item 8.01 Other Events: Declaration of Quarterly Cash Dividend"],
  "filing_date_target_utc": "2027-05-20T20:00:00Z",
  "filed_at_utc": "2027-05-20T18:48:18Z",
  "signing_principal": "lakshmi.subramanian-brodsky@helios-industries-inc-nyse-hlos",
  "signing_capacity": "General_Counsel_Secretary"
}
```

Audit: `EVT-J172-SEC-FORM-8K-FILED-Δ008a`.

## §9 Cedar deny coverage + observability + pack manifest

### 9.1 Cedar deny coverage

`[HEL:naveen.iyer-krishnamurthy] → audit-chain` — `GET /v1/audit-chain/cedar-deny-coverage?agm_session=agm-helios-2027-fy2026`

```json
{
  "agm_session_id": "agm-helios-2027-fy2026",
  "denied_enumeration_attempts_on_board_channel": 18,
  "denied_single_language_caption_disable_attempts": 4,
  "denied_pre_eps_release_material_disclosure": 2,
  "denied_partial_share_class_tally_disclose": 0,
  "total_denied_actions": 24
}
```

Audit: `EVT-J172-CEDAR-DENY-COVERAGE-009`.

### 9.2 Per-region latency report

`observability → audit-chain` — internal RPC `Observability/EmitLatencyReport`

```json
{
  "agm_session_id": "agm-helios-2027-fy2026",
  "per_edge_latency_ms": {
    "nyc-edge": {"target": 80, "actual": 62, "status": "PASS"},
    "london-edge": {"target": 120, "actual": 98, "status": "PASS"},
    "frankfurt": {"target": 120, "actual": 104, "status": "PASS"},
    "singapore": {"target": 180, "actual": 148, "status": "PASS"},
    "tokyo": {"target": 170, "actual": 142, "status": "PASS"},
    "seoul": {"target": 160, "actual": 138, "status": "PASS"},
    "sao-paulo": {"target": 120, "actual": 112, "status": "PASS"},
    "sydney": {"target": 180, "actual": 168, "status": "PASS"}
  },
  "overall_status": "ALL_PASS"
}
```

Audit: `EVT-J172-LATENCY-TARGETS-MET-010`.

### 9.3 Pack manifest

`[HEL:lev.kahn] → compliance` — `GET /v1/compliance/pack-manifest?agm_session=agm-helios-2027-fy2026`

```json
{
  "agm_session_id": "agm-helios-2027-fy2026",
  "active_packs": [
    "pack-sec-reg-fd-2024",
    "pack-sec-17a-4f-worm-v3",
    "pack-nyse-listed-company-manual-2027",
    "pack-sox-302-404",
    "pack-gdpr-eu-shareholders-v4",
    "pack-eu-mar-article-17",
    "pack-delaware-gcl-shareholder-meetings",
    "pack-ias-1-fy2026"
  ],
  "cross_validation_state": "passed",
  "pack_manifest_signature": "sha256:e7c4...9921"
}
```

Audit: `EVT-J172-PACK-MANIFEST-011`.

## §10 Summary

| Event class | Count | Cedar permits | Cross-tenant | Share-class scope |
|---|---|---|---|---|
| EVT-J172-AGM-COMMAND-CONSOLE-OPENED-Δ000 | 1 | ir_director + passkey + cfo_co_sign | no | n/a |
| EVT-J172-LANGUAGE-STREAM-ADDED-jaJP-Δ001a | 1 | ir_director + cfo_co_sign + interpreter_credential | yes (ABC LS) | n/a |
| EVT-J172-DRY-RUN-PASSED-Δ001b | 1 | ir_director | no | n/a |
| EVT-J172-LIVESTREAM-OPENED-001 | 1 | ir_director + meet | no | n/a |
| EVT-J172-SHAREHOLDERS-AUTHENTICATED-002 | 12,042 | per-auth-path | yes (broker tenants) | per-shareholder |
| EVT-J172-CAPTIONS-VERIFIED-003 | rolling | meet + audit-chain | yes (ABC LS) | n/a |
| EVT-J172-REG-FD-SIMULTANEOUS-DISCLOSURE-006 | 1 | meet + reg_fd_gate | no | n/a |
| EVT-J172-Q-AND-A-SUBMITTED-Δ004a | 187 | meet + per-shareholder | yes | n/a |
| EVT-J172-COMMUNITY-RETAIL-FILTER-Δ004a | 88 | ombuds_filter | no | n/a |
| EVT-J172-Q-AND-A-ROLLUP-004 | 1 | meet + community + intelligence | yes | n/a |
| EVT-J172-VOTING-OPENED-Δ005a | 6 (per proposal) | gc_secretary + governance | no | per-share-class |
| EVT-J172-VOTE-TALLY-005 | 12 (per proposal per share class) | computershare + carl_hagberg + dual_sign | yes (CS + CH) | per-share-class |
| EVT-J172-MERKLE-ANCHORS-007 | 12 | governance + audit-chain | no | per-share-class |
| EVT-J172-FINAL-CERTIFICATION-Δ005f | 1 | gc + computershare + carl_hagberg | yes | all-classes |
| EVT-J172-LIVESTREAM-CLOSED-Δ001c | 1 | meet | no | n/a |
| EVT-J172-SEC-17A-4F-WORM-SEALED-008 | 1 (composite 24 artifacts) | drive + sec_17a_4f | no | n/a |
| EVT-J172-SEC-FORM-8K-FILED-Δ008a | 1 | gc_secretary + sec_edgar_bridge | yes (SEC EDGAR) | n/a |
| EVT-J172-CEDAR-DENY-COVERAGE-009 | 1 | naveen + audit-chain | no | n/a |
| EVT-J172-LATENCY-TARGETS-MET-010 | 1 | observability + audit-chain | no | n/a |
| EVT-J172-PACK-MANIFEST-011 | 1 | compliance | no | n/a |

Total: ~12,180 audit events across 24 hours. Reg FD simultaneous-disclosure gate enforced. Per-share-class vote tally Merkle-anchored. SEC 17a-4(f) WORM seal verified. en-US + en-UK + zh-Hans + ko-KR + ja-JP + Russian + Hebrew preservation UTF-8 NFC byte-exact.
