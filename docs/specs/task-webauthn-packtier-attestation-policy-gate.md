# Spec: webauthn-packtier-attestation-policy-gate

**Crate**: `shared-webauthn-server-kernel`
**Authority**: ADR-0188 §"Attestation policy"
**Kind**: Additive pure function — no I/O, no new dependencies

---

## 1. Motivation

`PackTier` already exposes `required_attestation()` and
`requires_aaguid_allowlist()` as building blocks, but callers must manually
assemble the admission logic. This creates duplication risk and divergent
enforcement. A single canonical `admit_credential` method centralises the
gate, making it trivial to unit-test and impossible to accidentally skip.

---

## 2. Public API addition

```rust
impl PackTier {
    /// Evaluate whether `credential` may be admitted for this pack tier.
    ///
    /// Returns `Ok(())` when all policy checks pass.
    /// Returns `Err(WebauthnError::AttestationLevelInsufficient)` when the
    /// presented conveyance is weaker than the tier requires.
    /// Returns `Err(WebauthnError::AaguidNotAllowlisted)` when the tier
    /// requires an AAGUID allowlist and either:
    ///   - the credential's AAGUID is the zero AAGUID, or
    ///   - `aaguid_allowlist` is `None`, or
    ///   - the AAGUID is absent from the supplied allowlist.
    ///
    /// `aaguid_allowlist` is ignored for tiers that do not require an
    /// allowlist (`SandboxOrDev`, `PackStandard`).
    pub fn admit_credential(
        self,
        presented_conveyance: AttestationConveyance,
        credential: &Credential,
        aaguid_allowlist: Option<&BTreeSet<Aaguid>>,
    ) -> Result<(), WebauthnError>;
}
```

---

## 3. Attestation conveyance ordering

The ordering used by the gate (weakest to strongest):

| Level | Variant |
|---|---|
| 0 | `AttestationConveyance::None` |
| 1 | `AttestationConveyance::Indirect` |
| 2 | `AttestationConveyance::Direct` |
| 3 | `AttestationConveyance::Enterprise` |

The gate **passes** when `level(presented) >= level(required)`.

---

## 4. Tier matrix

| Tier | Required conveyance | Requires allowlist |
|---|---|---|
| `SandboxOrDev` | `None` (level 0) | No |
| `PackStandard` | `Indirect` (level 1) | No |
| `PackRegulated` | `Direct` (level 2) | Yes |
| `AcrCritical` | `Direct` (level 2) | Yes |

`Enterprise` (level 3) is accepted wherever `Direct` is required because it
exceeds the minimum.

---

## 5. AAGUID enforcement rules

When `requires_aaguid_allowlist()` is `true`:

1. If `credential.aaguid.is_zero()` → `Err(AaguidNotAllowlisted(Aaguid::ZERO))`
2. If `aaguid_allowlist.is_none()` → `Err(AaguidNotAllowlisted(credential.aaguid))`
3. If `!allowlist.contains(&credential.aaguid)` → `Err(AaguidNotAllowlisted(credential.aaguid))`

---

## 6. Acceptance criteria

All of the following unit tests must pass (hermetic, no I/O):

1. `sandbox_or_dev_always_admits` — `SandboxOrDev` + `None` conveyance + any AAGUID → `Ok`
2. `pack_standard_indirect_admits` — `PackStandard` + `Indirect` → `Ok`
3. `pack_standard_none_rejects` — `PackStandard` + `None` → `Err(AttestationLevelInsufficient)`
4. `regulated_direct_allowlisted_admits` — `PackRegulated` + `Direct` + known AAGUID in allowlist → `Ok`
5. `regulated_direct_zero_aaguid_rejects` — `PackRegulated` + `Direct` + zero AAGUID → `Err(AaguidNotAllowlisted)`
6. `regulated_direct_not_in_allowlist_rejects` — `PackRegulated` + `Direct` + AAGUID not in allowlist → `Err(AaguidNotAllowlisted)`
7. `regulated_none_rejects_conveyance` — `PackRegulated` + `None` → `Err(AttestationLevelInsufficient)`
8. `critical_enterprise_admits` — `AcrCritical` + `Enterprise` + allowlisted → `Ok`
9. `critical_direct_admits` — `AcrCritical` + `Direct` + allowlisted → `Ok`
10. `critical_indirect_rejects` — `AcrCritical` + `Indirect` → `Err(AttestationLevelInsufficient)`
11. `regulated_nil_allowlist_rejects` — `PackRegulated` + `Direct` + valid AAGUID but `allowlist=None` → `Err(AaguidNotAllowlisted)`

---

## 7. Non-goals

- No changes to `WebauthnServer`, `WebauthnRpAdapter`, sign-count logic, or
  any other existing item.
- No new workspace members or Cargo.toml entries.
- No async, no network, no file I/O.
