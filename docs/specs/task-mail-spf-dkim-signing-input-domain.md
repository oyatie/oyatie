# Spec: SPF Alignment + DKIM Signing-Input Domain Layer

**Task slug:** mail-spf-dkim-signing-input-domain  
**Vertical:** mail  
**Crate:** oya-mail-domain  
**RFC authority:** RFC 7208 §2.6 (SPF), RFC 6376 §3.4/§3.5/§3.7 (DKIM canonicalization + signing-input)  
**Existing guard extended:** `sending_domain_authentication.rs` (SPF/DMARC/DKIM posture admission)  

---

## Objective

The existing `evaluate_sending_domain_authentication` in `sending_domain_authentication.rs` checks SPF/DMARC/DKIM *posture evidence* and explicitly disclaims cryptographic signing, DNS lookup, OpenBao reads, and SMTP delivery. This task adds the pure typed domain logic that an adapter needs to:

1. Evaluate SPF identifier alignment between envelope-from domain and header-from (RFC5322.From) domain (strict vs relaxed), returning a typed verdict consistent with the existing `SendingDomainAuthReason` vocabulary.
2. Produce RFC 6376 relaxed and simple header+body canonicalization from already-parsed inputs.
3. Build the canonical DKIM signing-input/template (selected headers, `bh=` placeholder over canonical body, `b=` empty) that an adapter later feeds to `aws-lc-rs` for actual signing.

No new crate, no new dependency, no DNS lookup, no OpenBao read, no actual signing. Three new modules under `src/`; re-exported from `lib.rs`.

---

## Vertical and Module Layout (flat clean-arch)

```
crates/oya-mail-domain/
  src/
    lib.rs                             # pub mod + pub use re-exports (extended)
    governance.rs                      # existing — organizational_domain() reused by spf_alignment
    sending_domain_authentication.rs   # existing — DkimSigningAlgorithm::supported_for_signing() seam
    thread_state.rs                    # existing — unchanged
    spf_alignment.rs                   # NEW ST1
    dkim_canonicalization.rs           # NEW ST2
    dkim_signing_input.rs              # NEW ST3
```

All new domain logic lives as top-level items or `impl` blocks inside the named modules. No sub-modules, no new crates.

---

## Module Contracts

### `spf_alignment.rs` (ST1)

```rust
/// RFC 7208 §2.6 alignment mode governing whether a subdomain of the
/// authenticated SPF domain counts as aligned with the RFC5322.From domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpfAlignmentMode {
    /// Only an exact domain match (case-insensitive) is aligned.
    Strict,
    /// Organizational-domain match is sufficient (same registered domain).
    Relaxed,
}

/// Result of SPF identifier alignment evaluation.
///
/// Consistent with `SendingDomainAuthReason` vocabulary: a non-`Aligned`
/// verdict maps to `SenderDomainMismatch` or `SpfMissing` at the admission
/// layer; this type carries finer-grained information for routing decisions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpfAlignmentVerdict {
    /// Exact domain match (valid for both strict and relaxed modes).
    Aligned,
    /// Organizational-domain match under relaxed mode only.
    RelaxedAligned,
    /// No alignment (always returned under strict mode for non-exact matches).
    Misaligned,
}

/// Evaluate SPF identifier alignment per RFC 7208 §2.6 and RFC 7489 §3.1.
///
/// # Arguments
/// * `envelope_from_domain` — domain from the SMTP `MAIL FROM` (envelope-from).
/// * `header_from_domain`   — RFC5322.From domain extracted from the message header.
/// * `mode`                 — alignment strictness.
///
/// # Returns
/// `SpfAlignmentVerdict` — never performs DNS lookup or network I/O.
pub fn evaluate_spf_alignment(
    envelope_from_domain: &str,
    header_from_domain: &str,
    mode: SpfAlignmentMode,
) -> SpfAlignmentVerdict
```

**Domain normalization:** case-insensitive ASCII comparison; trailing dots stripped. Reuses the `normalized_domain()` pattern from `sending_domain_authentication.rs`. Organizational domain extraction delegates to `governance::organizational_domain()`.

**Verdict derivation:**

| envelope_from vs header_from | mode | verdict |
|------------------------------|------|---------|
| exact match (normalized) | either | `Aligned` |
| same org domain, not exact | Relaxed | `RelaxedAligned` |
| same org domain, not exact | Strict | `Misaligned` |
| different org domain | either | `Misaligned` |

---

### `dkim_canonicalization.rs` (ST2)

```rust
/// RFC 6376 §3.4 canonicalization algorithm selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DkimCanonicalizationAlgorithm {
    /// RFC 6376 §3.4.1 — folds header whitespace, lowercases names.
    Relaxed,
    /// RFC 6376 §3.4.2 — preserves header verbatim, normalizes body CRLF.
    Simple,
}

/// A single parsed mail header name/value pair (after unfolding, before
/// canonicalization).  Name and value are UTF-8 strings; the caller is
/// responsible for RFC 2822 unfolding if required.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawHeader {
    pub name: String,
    pub value: String,
}

/// Produce the canonical representation of the given headers per RFC 6376 §3.4.
///
/// Each header is emitted as `<name>:<value>\r\n` (relaxed) or verbatim
/// `<original>\r\n` (simple).  No I/O; pure string transformation.
pub fn canonicalize_header(
    headers: &[RawHeader],
    algorithm: DkimCanonicalizationAlgorithm,
) -> String

/// Produce the canonical representation of the message body per RFC 6376 §3.4.
///
/// An empty body (or all-whitespace body after stripping) becomes a single
/// `\r\n` for both relaxed and simple.  No I/O; pure byte transformation.
pub fn canonicalize_body(
    body: &[u8],
    algorithm: DkimCanonicalizationAlgorithm,
) -> Vec<u8>
```

**RFC 6376 §3.4.1 relaxed header rules:**
1. Header field name lowercased.
2. All whitespace runs (SP, HTAB, folded CRLF-SP/CRLF-HTAB) within the value collapsed to a single SP (0x20).
3. Leading and trailing whitespace in the value stripped.
4. Output per header: `<lowercased-name>:<normalized-value>\r\n`.

**RFC 6376 §3.4.2 simple header rules:**
1. Header field name and value preserved verbatim.
2. Terminated with CRLF if not already present.

**RFC 6376 §3.4.3 relaxed body rules:**
1. Remove trailing whitespace (SP, HTAB) from each line.
2. Reduce multiple trailing blank lines to a single CRLF.
3. Empty body → single `\r\n`.

**RFC 6376 §3.4.4 simple body rules:**
1. Body MUST end with exactly one `\r\n`.
2. Multiple trailing `\r\n` sequences collapsed to one.
3. Empty body → single `\r\n`.

---

### `dkim_signing_input.rs` (ST3)

```rust
use crate::{DkimSigningAlgorithm, NON_CLAIM};
use crate::dkim_canonicalization::{DkimCanonicalizationAlgorithm, RawHeader};

/// All inputs required to produce a DKIM signing-input string.
/// No key material; no cryptographic operations.
pub struct DkimSigningInputRequest {
    /// Header field names to include in `h=`, in the order given.
    pub signed_headers: Vec<String>,
    /// Full set of parsed message headers.  Signing selects from these.
    pub headers: Vec<RawHeader>,
    /// Raw message body bytes (pre-canonicalization).
    pub body: Vec<u8>,
    /// DKIM selector (s= tag).
    pub selector: String,
    /// Signing domain (d= tag).
    pub signing_domain: String,
    /// Opaque key-version reference (informational; not key material).
    pub key_version_ref: String,
    /// Signing algorithm; validated via `DkimSigningAlgorithm::supported_for_signing`.
    pub algorithm: DkimSigningAlgorithm,
    /// Canonicalization applied to headers.
    pub header_canonicalization: DkimCanonicalizationAlgorithm,
    /// Canonicalization applied to the body.
    pub body_canonicalization: DkimCanonicalizationAlgorithm,
}

/// Typed signing-input material returned to the adapter.
/// Contains no key material and performs no signing.
pub struct DkimSigningInputMaterial {
    /// The DKIM-Signature header stub with `b=` left empty (ready for signing).
    /// Shape:
    ///   `DKIM-Signature: v=1; a=<alg>; c=<hdr>/<body>; d=<domain>; s=<sel>;
    ///    h=<h-tag>; bh=<bh>; b=`
    /// where `<bh>` is the literal placeholder string the adapter replaces with
    /// the base64-encoded hash of `canonical_body` before signing.
    pub signing_input: String,
    /// Canonical body bytes.  The adapter hashes these to compute `bh=`.
    pub canonical_body: Vec<u8>,
    /// Canonical header strings in the order they contribute to the signature.
    /// The DKIM-Signature stub (with `b=` empty) is appended last per RFC 6376 §3.7.
    pub canonical_signed_headers: Vec<String>,
    /// Invariant: this module performs no signing, DNS lookup, OpenBao read, or
    /// SMTP delivery.  Value is `NON_CLAIM` from `sending_domain_authentication`.
    pub non_claim: &'static str,
}

/// Errors returned by `build_dkim_signing_input`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DkimSigningInputError {
    /// `algorithm.supported_for_signing()` returned false.
    UnsupportedAlgorithm,
    /// `selector` or `signing_domain` is empty/whitespace.
    EmptySelectorOrDomain,
    /// `signed_headers` list is empty.
    NoSignedHeaders,
}

/// Build the canonical DKIM signing-input string per RFC 6376 §3.5 / §3.7.
///
/// Steps:
/// 1. Validate algorithm via `DkimSigningAlgorithm::supported_for_signing`.
/// 2. Validate selector and signing_domain non-empty.
/// 3. Validate signed_headers non-empty.
/// 4. Canonicalize body per `body_canonicalization`.
/// 5. For each name in `signed_headers` (in order): find the last matching
///    header in `headers` (RFC 6376 §5.4 last-occurrence rule) and canonicalize.
/// 6. Build the DKIM-Signature stub with `bh=<bh>` and `b=` (empty).
/// 7. Append the canonical DKIM-Signature stub as the final signed-header.
///
/// No I/O, no crypto.  Returns `DkimSigningInputMaterial` for the adapter.
pub fn build_dkim_signing_input(
    request: DkimSigningInputRequest,
) -> Result<DkimSigningInputMaterial, DkimSigningInputError>
```

**`h=` tag construction:** header names joined by `:` (e.g. `from:to:subject`), lowercased.

**`a=` tag string mapping:**
| `DkimSigningAlgorithm` | `a=` string |
|------------------------|-------------|
| `Ed25519Sha256`        | `ed25519-sha256` |
| `RsaSha256`            | `rsa-sha256` |
| `RsaSha1`              | rejected (UnsupportedAlgorithm) |
| `Other`                | rejected (UnsupportedAlgorithm) |

**`c=` tag string mapping:**
| `DkimCanonicalizationAlgorithm` | tag token |
|---------------------------------|-----------|
| `Relaxed`                       | `relaxed` |
| `Simple`                        | `simple`  |

Format: `c=<header-canon>/<body-canon>` (e.g. `c=relaxed/simple`).

---

## Data Classification

All new types carry `DataClass::InternalOnly` (routing/operational metadata, no PII). No new `Classified<T>` wrappers are required on the signing-input structs because these are transient computation results, not stored domain aggregates. The `non_claim` field is a `&'static str` constant (no classification wrapper needed).

---

## Testing Strategy

All tests live in `#[cfg(test)] mod tests` blocks at the bottom of each new module file.

### ST1 — `spf_alignment.rs` tests

| Test | Scenario | Expected |
|------|----------|----------|
| `exact_match_strict_aligned` | `example.com` vs `example.com`, strict | `Aligned` |
| `exact_match_relaxed_aligned` | `example.com` vs `example.com`, relaxed | `Aligned` |
| `subdomain_relaxed_relaxed_aligned` | `mail.example.com` vs `example.com`, relaxed | `RelaxedAligned` |
| `subdomain_strict_misaligned` | `mail.example.com` vs `example.com`, strict | `Misaligned` |
| `unrelated_domain_misaligned` | `unrelated.net` vs `example.com`, relaxed | `Misaligned` |
| `case_insensitive_normalization` | `EXAMPLE.COM` vs `example.com`, strict | `Aligned` |
| `trailing_dot_normalization` | `example.com.` vs `example.com`, strict | `Aligned` |

### ST2 — `dkim_canonicalization.rs` tests

| Test | Scenario | Expected |
|------|----------|----------|
| `relaxed_header_lowercases_name` | `Subject: Hello` | `subject:Hello\r\n` |
| `relaxed_header_collapses_whitespace` | `Subject:  foo  bar  ` | `subject:foo bar\r\n` |
| `relaxed_header_strips_folded_whitespace` | folded value with CRLF-SP | single-line normalized |
| `simple_header_preserves_verbatim` | `Subject: Hello` | `Subject: Hello\r\n` |
| `relaxed_body_strips_trailing_whitespace` | `foo   \r\nbar\r\n` | `foo\r\nbar\r\n` |
| `relaxed_body_collapses_trailing_blank_lines` | `foo\r\n\r\n\r\n` | `foo\r\n` |
| `relaxed_body_empty_yields_crlf` | `b""` | `\r\n` |
| `simple_body_collapses_trailing_crlf` | `foo\r\n\r\n\r\n` | `foo\r\n` |
| `simple_body_empty_yields_crlf` | `b""` | `\r\n` |

### ST3 — `dkim_signing_input.rs` tests

| Test | Scenario | Expected |
|------|----------|----------|
| `signing_input_contains_v1_tag` | valid request | `signing_input` contains `v=1` |
| `signing_input_b_tag_is_empty` | valid request | `signing_input` ends with `; b=` |
| `signing_input_bh_placeholder_present` | valid request | contains `bh=<bh>` |
| `signing_input_h_tag_matches_request` | `signed_headers=["from","subject"]` | `h=from:subject` |
| `signing_input_dkim_stub_appended_last` | valid request | last element of `canonical_signed_headers` is the DKIM-Signature stub |
| `unsupported_algorithm_rejected` | `RsaSha1` | `Err(UnsupportedAlgorithm)` |
| `empty_selector_rejected` | `selector=""` | `Err(EmptySelectorOrDomain)` |
| `empty_signed_headers_rejected` | `signed_headers=[]` | `Err(NoSignedHeaders)` |
| `non_claim_invariant_preserved` | valid request | `non_claim` contains "no DNS lookup" |

---

## Boundaries and Non-Claims

- **No new crate.** All logic extends one existing crate (`oya-mail-domain`).
- **No root `Cargo.toml` edit.** Crate depends only on `oya-data-boundary-kernel`.
- **No new dependency.** All logic is pure domain computation.
- **No signing.** `build_dkim_signing_input` returns a string template; the `b=` value is left empty. The adapter passes `canonical_signed_headers` and `canonical_body` to `aws-lc-rs` (ADR-0506).
- **No DNS lookup.** Alignment evaluation is purely over already-resolved domain strings.
- **No OpenBao read.** Key material is entirely out of scope.
- **No SMTP delivery.** This is domain logic only.
- **NON_CLAIM invariant.** `DkimSigningInputMaterial::non_claim` is set to `NON_CLAIM` from `sending_domain_authentication.rs`, thread-safe and statically verified.

---

## OpenAPI / Proto Note

The SPF alignment verdict and DKIM signing-input material are domain aggregates, not yet exposed via a REST or gRPC surface in this vertical. When an outbound SMTP adapter is added, it will consume these types internally. No OpenAPI 3.2.0 or proto3 schema change is required by this task.

---

## References

- RFC 7208 §2.6 — SPF Result Codes and Identifier Alignment
- RFC 7489 §3.1 — DMARC Identifier Alignment (governing definitions)
- RFC 6376 §3.4 — DKIM Canonicalization Algorithms
- RFC 6376 §3.5 — DKIM-Signature Header Field
- RFC 6376 §3.7 — Computing the Signature
- RFC 6376 §5.4 — Last-occurrence rule for signed headers
- `crates/oya-mail-domain/src/sending_domain_authentication.rs` — `DkimSigningAlgorithm`, `NON_CLAIM`, `normalized_domain()`
- `crates/oya-mail-domain/src/governance.rs` — `organizational_domain()`, `int()` helper pattern
- `docs/adr-archive/ADR-0506-aws-lc-rs-canonical-crypto-provider.md` — cryptographic signing library (out of scope for this task; referenced for adapter contract)
- `docs/adr-archive/ADR-0130-deprecate-knowledge-graph-registry-file-migrate-to-ontology.md` — SLO gate (no SLO change this task)
