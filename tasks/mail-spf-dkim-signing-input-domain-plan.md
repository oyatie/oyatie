# Plan: mail-spf-dkim-signing-input-domain

**Vertical:** mail  
**Crate:** oya-mail-domain  
**Branch:** feat/task-mail-spf-dkim-signing-input-domain-2026-05-28  
**Base:** origin/dev  

---

## Objective

Extend `crates/oya-mail-domain` with a pure no-I/O SPF-alignment + DKIM RFC 6376 signing-input domain layer. Adds:

1. SPF identifier-alignment evaluation (strict vs relaxed, envelope-from vs header-from).
2. RFC 6376 relaxed and simple header+body canonicalization.
3. DKIM signing-input builder (selected headers, `bh=` placeholder, `b=` empty template) returning typed material an adapter later feeds to `aws-lc-rs`.

No DNS lookup, no OpenBao read, no cryptographic signing, no SMTP delivery — those remain adapter/runtime responsibilities.

---

## Subtasks

### ST1 — SPF alignment evaluation

**What:** Add a pure function evaluating SPF identifier alignment between the envelope-from domain and the header-from (RFC5322.From) domain, returning a typed verdict consistent with the existing `SendingDomainAuthReason` vocabulary.

**Module:** `src/spf_alignment.rs`, re-exported from `lib.rs`.

**Types introduced:**
```rust
pub enum SpfAlignmentMode { Strict, Relaxed }

pub enum SpfAlignmentVerdict { Aligned, RelaxedAligned, Misaligned }

pub fn evaluate_spf_alignment(
    envelope_from_domain: &str,
    header_from_domain: &str,
    mode: SpfAlignmentMode,
) -> SpfAlignmentVerdict
```

- `Aligned` — exact domain match (always returned for both strict and relaxed when domains are identical).
- `RelaxedAligned` — organizational-domain match under relaxed mode (e.g. `mail.example.com` vs `example.com`).
- `Misaligned` — no match.
- Under `SpfAlignmentMode::Strict`, only `Aligned` or `Misaligned` can be returned.
- Domain comparison is case-insensitive, trailing dots stripped (reuses normalization pattern from `sending_domain_authentication.rs`).
- Organizational domain extraction reuses the `organizational_domain()` function already in `governance.rs`; the new module imports it.

**Acceptance:**
- `cargo check -p oya-mail-domain --all-targets` clean.
- Unit tests covering: exact match strict → `Aligned`; subdomain relaxed → `RelaxedAligned`; subdomain strict → `Misaligned`; unrelated domains → `Misaligned`.
- Verdict variants do not break `SendingDomainAuthReason` — no changes to that enum.

---

### ST2 — DKIM canonicalization (relaxed and simple)

**What:** Implement RFC 6376 relaxed and simple header+body canonicalization as pure functions over already-parsed header/body inputs (no network, no crypto).

**Module:** `src/dkim_canonicalization.rs`, re-exported from `lib.rs`.

**Types introduced:**
```rust
pub enum DkimCanonicalizationAlgorithm { Relaxed, Simple }

/// A single parsed mail header (name + value).
pub struct RawHeader { pub name: String, pub value: String }

/// Canonical forms produced from one or more headers.
pub fn canonicalize_header(
    headers: &[RawHeader],
    algorithm: DkimCanonicalizationAlgorithm,
) -> String

/// Canonical form of the message body.
pub fn canonicalize_body(
    body: &[u8],
    algorithm: DkimCanonicalizationAlgorithm,
) -> Vec<u8>
```

**RFC 6376 §3.4 rules implemented:**

*Relaxed header:*
- Header field names lowercased.
- All runs of whitespace (SP, HTAB, CRLF folding) within the value collapsed to a single SP.
- Leading and trailing whitespace in the value stripped.
- Output: `<lowercased-name>:<normalized-value>\r\n` for each header.

*Simple header:*
- Header field name and value preserved verbatim (only CRLF termination ensured).

*Relaxed body:*
- Ignore all whitespace at the end of lines (remove trailing SP/HTAB).
- Reduce all runs of blank lines at the end of the body to a single CRLF.
- Empty body becomes a single CRLF.

*Simple body:*
- Body MUST end with exactly one CRLF.
- Multiple trailing CRLFs collapsed to one.
- Empty body becomes a single CRLF.

**Acceptance:**
- `cargo nextest run -p oya-mail-domain` green.
- Canonicalization unit tests assert RFC 6376 example vectors for both relaxed and simple modes (header folding collapse, trailing-whitespace stripping, trailing-blank-line normalization, empty-body → single CRLF).

---

### ST3 — DKIM signing-input builder

**What:** Build the canonical DKIM signing-input/template (selected signed headers, `bh=` placeholder over canonical body, `b=` empty for signing) returning typed material an adapter signs.

**Module:** `src/dkim_signing_input.rs`, re-exported from `lib.rs`.

**Types introduced:**
```rust
pub struct DkimSigningInputRequest {
    /// Headers to include in the signature, in the order they appear in h=.
    pub signed_headers: Vec<String>,
    /// Parsed message headers (full set; signed_headers selects from these).
    pub headers: Vec<RawHeader>,
    /// Raw message body bytes (pre-canonicalization).
    pub body: Vec<u8>,
    /// Selector (s= tag).
    pub selector: String,
    /// Signing domain (d= tag).
    pub signing_domain: String,
    /// Key version reference (for bh= construction; not the key material itself).
    pub key_version_ref: String,
    /// Signing algorithm posture (validated via DkimSigningAlgorithm::supported_for_signing seam).
    pub algorithm: crate::DkimSigningAlgorithm,
    /// Header canonicalization algorithm.
    pub header_canonicalization: DkimCanonicalizationAlgorithm,
    /// Body canonicalization algorithm.
    pub body_canonicalization: DkimCanonicalizationAlgorithm,
}

pub struct DkimSigningInputMaterial {
    /// The DKIM-Signature header stub with b= left empty, ready for signing.
    /// Shape: `DKIM-Signature: v=1; a=<alg>; c=<hdr>/<body>; d=<domain>;
    ///          s=<selector>; h=<signed-headers>; bh=<body-hash-placeholder>; b=`
    pub signing_input: String,
    /// Canonical body bytes (used by adapter to compute bh=).
    pub canonical_body: Vec<u8>,
    /// Ordered list of canonical header strings included in the signature.
    pub canonical_signed_headers: Vec<String>,
    /// NON_CLAIM invariant: this module performs no signing.
    pub non_claim: &'static str,
}

pub enum DkimSigningInputError {
    UnsupportedAlgorithm,
    EmptySelectorOrDomain,
    NoSignedHeaders,
}

pub fn build_dkim_signing_input(
    request: DkimSigningInputRequest,
) -> Result<DkimSigningInputMaterial, DkimSigningInputError>
```

**Logic:**
1. Reject if `request.algorithm.supported_for_signing()` returns false → `UnsupportedAlgorithm`.
2. Reject if selector or signing_domain is empty → `EmptySelectorOrDomain`.
3. Reject if `signed_headers` is empty → `NoSignedHeaders`.
4. Canonicalize body per `body_canonicalization`.
5. For each header name in `signed_headers` (in order): locate the last matching header (RFC 6376 §5.4 last-occurrence rule) in `headers` and canonicalize per `header_canonicalization`.
6. Build the DKIM-Signature header stub: `v=1; a=<alg-string>; c=<hdr-canon>/<body-canon>; d=<domain>; s=<selector>; h=<h-tag>; bh=<PLACEHOLDER>; b=` where `<PLACEHOLDER>` is the literal string `<bh>` (adapter replaces with real hash before signing).
7. Append the canonicalized DKIM-Signature header stub to the canonical signed-headers list (RFC 6376 §3.7 requirement: the DKIM-Signature itself is the final item in the signing input).
8. `non_claim` is set to `NON_CLAIM` from `sending_domain_authentication.rs`.

**Acceptance:**
- `cargo check -p oya-mail-domain --all-targets` + `cargo nextest run -p oya-mail-domain` pass.
- Tests assert: signing-input string shape (contains `v=1`, `b=` suffix, `bh=<bh>` placeholder, correct h= list); unsupported algorithm returns `UnsupportedAlgorithm`; `non_claim` contains "no DNS lookup"; `DkimSigningInputMaterial::non_claim` invariant preserved (module contains no signing logic).

---

## Acceptance Summary

| Check | Command |
|-------|---------|
| Compile clean | `cargo check -p oya-mail-domain --all-targets` |
| All tests pass | `cargo nextest run -p oya-mail-domain` |
| No root Cargo.toml edit | `git diff HEAD -- Cargo.toml` empty |
| No new crate | workspace member count unchanged |

---

## Constraints

- Operate ONLY in `crates/oya-mail-domain/src/` (new modules `spf_alignment.rs`, `dkim_canonicalization.rs`, `dkim_signing_input.rs`), `lib.rs` (re-export), and this plan + spec doc.
- No new dependencies; crate depends only on `data-boundary-kernel`.
- No root `Cargo.toml` edit.
- No DNS lookup, no OpenBao read, no crypto signing, no SMTP delivery.
- Match existing patterns: `int()` / `classified_internal()` helpers, `normalized_domain()` normalization, `NON_CLAIM` constant, `#[cfg_attr(test, allow(...))]` at crate top.
- `DkimSigningAlgorithm::supported_for_signing()` seam in `sending_domain_authentication.rs` is the single gate for algorithm support (do not duplicate the check).
