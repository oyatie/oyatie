// connector service — payload-signature-verification policy fragment
// Cedar v4.2 LTS
//
// Purpose: gate which signature algorithms are accepted per vendor; forbid
//          SHA1 + downgrade; require constant-time compare.
// Owner:   axis-integration + ops-security
// References: threat-model.md T-S-04/T-T-04; failure-modes.md signature-verification
// soak-window-seconds: 60
// publish-stage: active

// =============================================================================
// Default deny
// =============================================================================

forbid (principal, action, resource);

// =============================================================================
// PERMIT 1 — verify with HMAC-SHA256 / SHA384 / SHA512 / Ed25519
// =============================================================================

permit (
  principal,
  action == Action::"signature_verify",
  resource in SignatureContext::?s
)
when {
  resource has algorithm &&
  resource.algorithm in [
    "hmac-sha256",
    "hmac-sha384",
    "hmac-sha512",
    "ed25519",
    "ecdsa-p256-sha256",
    "rsa-pss-sha256"
  ]
};

// =============================================================================
// FORBID — SHA1 (deprecated; downgrade attack)
// =============================================================================

forbid (
  principal,
  action == Action::"signature_verify",
  resource in SignatureContext::?s
)
when {
  resource has algorithm &&
  resource.algorithm in ["hmac-sha1", "hmac-md5", "sha1", "md5"]
};

// =============================================================================
// FORBID — sandbox skip-verify (sandbox only; never prod)
// =============================================================================

forbid (
  principal,
  action == Action::"signature_skip_verify",
  resource
)
when {
  context has environment &&
  context.environment != "sandbox"
};
