// api-gateway µservice — TLS policy fragment
// Cedar v4.2 LTS
// Authority: ADR-0253 (HTTP/3 + TLS strict + ECH + PQC).

// =============================================================================
// FORBID — TLS version below 1.3
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  principal has tls_version &&
  principal.tls_version != "TLS_1_3"
};

// =============================================================================
// FORBID — non-AEAD cipher suite
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  principal has cipher_suite &&
  !(principal.cipher_suite in [
    "TLS_AES_128_GCM_SHA256",
    "TLS_AES_256_GCM_SHA384",
    "TLS_CHACHA20_POLY1305_SHA256"
  ])
};

// =============================================================================
// FORBID — no SCTs / CT not validated
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  principal has sct_count &&
  principal.sct_count < 2
};

// =============================================================================
// FORBID — must-staple cert without OCSP staple
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  principal has cert_must_staple &&
  principal.cert_must_staple == true &&
  principal has ocsp_stapled &&
  principal.ocsp_stapled == false
};

// =============================================================================
// PERMIT — TLS 1.3 + AEAD + ≥2 SCTs + OCSP-stapled
// =============================================================================
// The default-permit path.

permit (
  principal,
  action,
  resource
)
when {
  principal has tls_version &&
  principal.tls_version == "TLS_1_3" &&
  principal has cipher_suite &&
  principal.cipher_suite in [
    "TLS_AES_128_GCM_SHA256",
    "TLS_AES_256_GCM_SHA384",
    "TLS_CHACHA20_POLY1305_SHA256"
  ] &&
  principal has sct_count &&
  principal.sct_count >= 2
};

// =============================================================================
// PERMIT — FIPS 140-3 strict (pack-il5/6, pack-fedramp-high)
// =============================================================================

permit (
  principal,
  action,
  resource in SovCellResource::?r
)
when {
  resource has compliance_pack &&
  resource.compliance_pack in ["il5", "il6", "fedramp-high"] &&
  principal has cipher_suite &&
  principal.cipher_suite in ["TLS_AES_128_GCM_SHA256", "TLS_AES_256_GCM_SHA384"] &&
  principal has fips_validated &&
  principal.fips_validated == true
};

// =============================================================================
// FORBID — pack-fedramp-high without PQC
// =============================================================================

forbid (
  principal,
  action,
  resource in SovCellResource::?r
)
when {
  resource has compliance_pack &&
  resource.compliance_pack == "fedramp-high" &&
  principal has pqc_hybrid_negotiated &&
  principal.pqc_hybrid_negotiated == false
};
