---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: S-007 + Q-006
authoritative_source: RFC 3161 + eIDAS Article 42 + ETSI EN 319 422
related_packs: [eidas, kr-pipa]
date: 2026-05-21
---

# Time-Stamp Authority (TSA) Binding Model

eIDAS QES + AdES-B-LTA + Korean Certified Electronic Signature all require RFC 3161 time-stamps from qualified TSAs. CLM maintains per-tenant TSA preferences and falls over within the LOTL on TSA outage.

## TSA registry

```
TSARegistry {
  tenant_id: TenantId,
  primary_tsa: TSAReference,
  fallback_tsas: [TSAReference],                 // priority-ordered
  preference_set_at: Timestamp<RFC3339>,
  preference_set_by: PrincipalId,
}

struct TSAReference {
  tsa_id: TSAIdentifier,
  tsa_url: HTTPUrl,                              // RFC 3161 endpoint
  trust_list_membership: [TrustListMembership],  // LOTL, KISA, etc.
  qualified_status: QualifiedStatus,
  hash_algorithms_supported: [HashAlgorithm],
  fips_140_level: u8?,                           // if FIPS-validated
  geographic_residency: CountryCode,
}

enum TrustListMembership {
  EULOTL { country: EUCountryCode, trust_list_url: HTTPUrl },
  KISARooted { kisa_registry_id: String },
  JPNintei,                                      // Japanese 認定
  USFederalPKI,                                  // US Federal PKI
  Custom { tenant_attestation: ArtefactId },
}
```

## Approved TSA catalog

### EU LOTL-qualified TSAs

| TSA | Country | Status | Notes |
|---|---|---|---|
| SwissSign Time-Stamping Authority | CH | Qualified | EU LOTL via Switzerland equivalence agreement |
| Trustpro Qualified TSA | EU multi-country | Qualified | LOTL-listed |
| D-Trust Qualified TSA | DE | Qualified | LOTL-listed |
| Certigna TSA | FR | Qualified | LOTL-listed |
| Symantec / DigiCert TSA EU | EU | Qualified | LOTL-listed |
| GlobalSign Qualified TSA | EU multi-country | Qualified | LOTL-listed |
| KIR S.A. Qualified TSA | PL | Qualified | LOTL-listed |
| Asseco Data Systems TSA | PL | Qualified | LOTL-listed |
| InfoCert Qualified TSA | IT | Qualified | LOTL-listed |
| Namirial Qualified TSA | IT | Qualified | LOTL-listed |
| Universign TSA | FR | Qualified | LOTL-listed |
| Buypass Qualified TSA | NO | Qualified | LOTL-listed |

### KISA-rooted TSAs

| TSA | Country | Status |
|---|---|---|
| KISA Time-Stamping Authority | KR | Qualified |
| KICA (Korea Information Certificate Authority) | KR | Qualified |
| KOSCOM TSA | KR | Qualified |
| KTNET TSA | KR | Qualified |
| Yessign TSA | KR | Qualified |

### Other jurisdictional TSAs

| TSA | Country | Status |
|---|---|---|
| Cybertrust Japan TSA | JP | 認定 |
| GlobalSign TSA Japan | JP | 認定 |
| TKL TSA | JP | 認定 |
| DigiCert TSA US Federal | US | Federal PKI |
| SAFE-BioPharma TSA | US | Sector PKI |
| ICP-Brasil TSA | BR | Qualified |
| Adobe Approved Trust List (AATL) TSAs | Global | Various |

## TSA usage rules

1. **Hash algorithm match**: the timestamp request hash must match the envelope hash algorithm.
2. **TSA jurisdictional match**:
   - eIDAS QES: any LOTL-qualified TSA.
   - eIDAS QES with EU sovereign cell: prefer in-country TSA.
   - KR 인증전자서명: KISA-rooted TSA mandatory.
   - JP 認定認証業務: JP-approved TSA mandatory.
3. **Failover policy**: if primary TSA returns error or exceeds 5s response time, fall over to next priority. After all preferences exhausted, sign with provisional timestamp and queue a re-stamp from the next-available TSA within 24 hours.
4. **TSA outage tolerance**: contract sealing must NOT block on TSA outage. The signature is sealed without long-term archive timestamp; the LTA timestamp is back-filled when TSA recovers. Audit-chain records the deferred LTA.

## LTA (Long-Term Archive) Timestamp

For AdES-B-LTA, after the initial timestamp expires (RFC 3161 timestamps have a validity window tied to the TSA certificate validity), a fresh timestamp is applied to renew the archive. CLM schedules:

- Initial LTA timestamp at signature seal time.
- Renewal LTA timestamp at TSA certificate expiry - 90 days.
- Each renewal extends the AdES-B-LTA chain.

## Provider attestation

Each TSA in the registry must provide:

- Trust List membership evidence (URL to the trust list + verification).
- Operational SLA (typically ≤ 100 ms p99 response).
- Audit report (SOC-2, ISO 27001, or local equivalent).
- Periodic compliance reassertion.

## Cedar gate

```cedar
forbid (
  principal,
  action == Action::"TimestampRequest",
  resource is SignaturePacket
) when {
  resource.signature_level == "QES" &&
  resource.tsa.trust_list_membership matches "EULOTL" == false
};

forbid (
  principal,
  action == Action::"TimestampRequest",
  resource is SignaturePacket
) when {
  resource.signature_level == "KR_CERTIFIED" &&
  resource.tsa.trust_list_membership matches "KISARooted" == false
};
```

## Audit events

- `oya.contract.lifecycle.management.tsa.timestamp_requested`
- `oya.contract.lifecycle.management.tsa.timestamp_received`
- `oya.contract.lifecycle.management.tsa.timestamp_lta_renewed`
- `oya.contract.lifecycle.management.tsa.failover_triggered`
- `oya.contract.lifecycle.management.tsa.deferred_lta_filled`

## Standards references

- RFC 3161 — Internet X.509 Public Key Infrastructure Time-Stamp Protocol (TSP).
- RFC 5816 — ESSCertIDv2 update to RFC 3161.
- ETSI EN 319 422 — Time-stamping protocol and time-stamp token profiles.
- ETSI TS 119 312 — Cryptographic suites.
- ETSI TS 119 612 — Trusted Lists.
- Adobe Approved Trust List (AATL).
