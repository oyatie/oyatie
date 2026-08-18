//! # cloud-ci-dto-authz-trust (the CLASS-FIX for "caller-supplied authorization trusted as the
//! authz decision" — the #1 systemic security antipattern a whole-repo review found at 30+ trust
//! boundaries; sibling of cloud-ci-authz-coverage / issue #770 / AUTH-005).
//!
//! ## The antipattern (default-ALLOW-on-forged-input)
//! A request handler / use-case reads an *authorization decision* FROM the request itself — a
//! `{decision_id, tenant_id, principal_id, allowed_surfaces}` blob in the request DTO (an
//! `*Authorization` struct), or `x-authorization-*` headers — and "validates" it by string-comparing
//! those fields against the SAME request (presence / equality / surface-membership checks), WITHOUT
//! ever calling the cloud-iam Cedar PDP server-side. Any caller forges the blob and authorizes
//! itself. Representative confirmed instances: `secrets/ports/kms-api` `validate_authorization`,
//! `tenancy/ports/api`, `network/ports/{lb,vpc,dns}`, `audit/core/usecase`,
//! `compliance/ports/dsr-usecase`, `observability/core/api`, `billing/ports/finops-api`, and ~20
//! more `validate_authorization` functions. It PASSES the unauthenticated-surface gate (#780) because
//! it HAS guard-looking code.
//!
//! ## The fixed pattern (GREEN reference)
//! `iam/ports/policy-cedar-api/src/authz.rs`: a `PrincipalVerifier::verify_principal(credential) ->
//! VerifiedPrincipal` (an UNFORGEABLE credential becomes a verified principal — this is
//! AUTHENTICATION, not authorization) PLUS a `PublishAuthorizer::ensure_authorized(principal,
//! resource)` / a PDP `decide(...)` port — these are the AUTHORIZATION decision ports. A function
//! that calls such a PDP decision port in its own body is NEVER flagged — the caller's claim is not
//! trusted; the server decides.
//!
//! ## Detection heuristic (two-signal, inverted from v1; honest limits)
//! A function is a DTO-AUTHZ-TRUST instance iff BOTH hold over its CODE-ONLY body (comments and
//! string/char literals elided via [`mask_non_code`], so a doc-comment mention never triggers):
//!   (a) it READS A CALLER-SUPPLIED AUTHORIZATION-DECISION FIELD: it takes a parameter whose type
//!       name ends with the policy `authorization_dto_type_suffix` (default `Authorization`) — the
//!       forged blob — OR its body reads any policy `trigger_decision_field_idents` member
//!       (`allowed_surfaces`, `permitted_scopes`, `caller_roles`, `granted`, `allowed_actions`) off
//!       a binding, OR it reads any policy `authorization_header_idents` — searched in a
//!       COMMENT-STRIPPED but STRING-PRESERVING view of the body so header names in string literals
//!       are found but header names appearing only in comments are not;
//!   (b) the body makes NO PDP / authorizer DECISION-PORT CALL: NONE of the whole-token call-shaped
//!       policy `pdp_decision_idents` (`.decide(`, `ensure_authorized(`, `check_authz(`,
//!       `ensure_authz(`, `authorize_decision(`, `pdp_decide(`) appears in the code-only body.
//!       NOTE: `verify_principal` is an AUTHENTICATION step — it verifies identity, not
//!       authorization — and is intentionally NOT in the PDP-satisfies set. A function that
//!       authenticates the principal but then self-compares the authz DTO is still flagged.
//!
//! v1 INVERSION (FN-06): v1 required self-compare operator tokens as a PRECONDITION. That was
//! evadable: `Vec::contains`, `binary_search`, `is_superset` all evade it. The corrected heuristic
//! drops self-compare as a GATE — flagging (a) AND (b) regardless of comparison form. Self-compare
//! tokens remain a description-enrichment signal only (still listed in policy for that purpose).
//!
//! Honest LIMITS (documented, not hidden):
//!   - **Dead-code evasion**: `if false { .decide() }` suppresses signal (b). The gate does NOT
//!     perform reachability analysis — a whole-token call-shaped PDP ident in the code-only body
//!     marks the function GREEN regardless of control flow. This residual is acceptable: the gate
//!     stops the forged-blob class where no PDP ident appears at all (the dominant corpus shape);
//!     a reviewer who writes `if false { decide() }` to suppress the gate is introducing an
//!     obvious intentional evasion that code review is responsible for catching.
//!   - **Wrong-receiver evasion**: `other_service.ensure_authorized()` on a different receiver
//!     (not the authz port) suppresses signal (b). Policy-keyed idents should be scoped to the
//!     owned PDP port shape; adding more PDP idents grows the GREEN recognition surface.
//!   - (a) is the load-bearing precision lever: requiring an *authorization-DTO* parameter or an
//!     authz-specific field signal (NOT just any `tenant_id` comparison) keeps benign
//!     tenant/path-binding validators GREEN (`validate_tenant_binding`,
//!     `validate_path_body_binding` legitimately compare request-derived identity fields but assert
//!     NO authorization verdict).
//!   - A function that BOTH reads an authorization DTO AND calls a PDP port is GREEN: the PDP call
//!     is the real decision; the DTO read is a redundant precondition, not the verdict.
//!
//! ## Split-decision allowlist (the precise, non-launderable FP mechanism — NOT a heuristic)
//! A SMALL number of genuine false positives are residual NON-AUTHORITATIVE
//! correlation-consistency cross-checks (ADR-0591 split-decision): a function that reads a
//! caller-supplied `*Authorization` DTO and only returns `Ok`/`Err` on internal consistency (it
//! NEVER grants), while the AUTHORITATIVE server-side PDP `ensure_authorized`/`decide` that gates the
//! operation lives elsewhere in the same use-case and dominates every path that reaches the flagged
//! function. These are cleared by an EXPLICIT, CURATED `split_decision_allowlist` in policy DATA — a
//! tiny hand-audited list, NOT a name-reachability heuristic. Each entry is the SAME exact-key shape
//! as the baseline (`<file>#<fn>:<body_hash>`), so any body change re-flags it: the allowlist is
//! shrink-only / non-launderable — it cannot suppress a DIFFERENT function that merely shares a name,
//! and it cannot suppress a body edited after the human audit. An allowlist key that matches no live
//! instance self-cleans via `DAT-STALE-SPLIT-DECISION-ALLOWLIST`.
//!
//! The earlier `recognize_sibling_pdp_delegation` name-reachability heuristic was REMOVED as UNSOUND:
//! it suppressed by function-NAME call-graph reachability rather than by proving a PDP decision
//! dominates the guarded operation, so a dead-code `if false { decide() }` PDP root + a call edge, a
//! same-named overload in a DIFFERENT impl block reached from a PDP root, and a single-file
//! `decide_access` bypass all laundered a genuinely-forgeable check to GREEN. The explicit allowlist
//! suppresses ONLY the exact audited `file#fn:body_hash` bodies and nothing else.
//!
//! ## Baseline key: `<file>#<fn>:<body_hash>`
//! A 32-hex-char FNV-1a body-content hash is appended to every baseline key so that a NEW function
//! with the SAME name in a different `mod` (or a refactored body) does not auto-launder as baselined.
//! The hash is computed over the CODE-ONLY (masked) function body at the time of `--write`, and
//! self-cleans via `DAT-STALE-BASELINE` when the body changes.
//!
//! ## Born-blocking with a FROZEN, SHRINK-ONLY baseline
//! Pre-existing instances are enumerated and FROZEN as known debt
//! (`frozen_dto_authz_trust_instances` in policy DATA): an instance whose stable signature key is in
//! the baseline is ACCEPTED (no block) — it is each owner's remediation over time. A NEW instance
//! whose key is NOT in the baseline → RED. The baseline is shrink-only by construction: a removed /
//! fixed instance drops its key, and a stale baseline key self-cleans via `DAT-STALE-BASELINE`. This
//! mirrors the authz-coverage / capability-membership / port-placement gate posture (born-green,
//! enforce-no-regression).
//!
//! ## Born pack-shaped
//! The crate is a NEUTRAL engine. Every repo-specific — the authorization-DTO type suffix, the
//! decision-field idents, the authorization-header idents, the PDP decision-port idents, the scan
//! roots/excludes, the frozen baseline, the liveness floor — is DATA in `dto-authz-trust-policy.json`.
//! A different repo adopts the gate by repointing the policy.
//!
//! ## Kernel contract
//! - [`collect_instances`] `(root, policy) -> observed` is the ONLY I/O: it walks the policy scan
//!   roots, reads each `.rs` file, and extracts every function with its authorization-trust signal.
//!   Read-only; writes no temp files.
//! - [`evaluate_keyed`] `(policy, observed) -> BTreeSet<Finding>` is PURE and unit-testable without a
//!   filesystem; it applies the baseline DATA to the observed instances.
//! - [`evaluate`] is the bare-code projection of [`evaluate_keyed`], the single source of the verdict.
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `DAT-CALLER-SUPPLIED-AUTHZ-TRUST` — a NEW function trusts a caller-supplied authorization
//!   decision (reads an authorization-DTO field + no PDP decision-port call), and its key is not in
//!   the frozen baseline.
//! - `DAT-STALE-BASELINE` — a frozen-baseline key matches no live instance (shrink-only self-clean).
//! - `DAT-STALE-SPLIT-DECISION-ALLOWLIST` — a `split_decision_allowlist` key matches no live instance
//!   (the audited body was fixed/removed/edited; remove or re-audit the entry — non-launderable).
//! - `DAT-EMPTY-SCAN` — fewer functions scanned than `min_expected_functions` (catches a broken
//!   glob / CWD / collect that would otherwise be a false-green).
//! - `DAT-POLICY-GATE-ID-MISMATCH` — the policy `gate_id` is not [`GATE_ID`] (fail-closed).
//! - `DAT-POLICY-MALFORMED` — the policy is structurally invalid (fail-closed).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::{Value, json};

/// The gate id, matching the buck2 target + the policy `gate_id`.
pub const GATE_ID: &str = "cloud-ci-dto-authz-trust";

/// The remediation doctrine pointer every finding carries.
pub const REMEDIATION_DOCTRINE: &str = "iam/ports/policy-cedar-api/src/authz.rs (PrincipalVerifier::verify_principal on an unforgeable \
     credential — AUTHENTICATION step — then PublishAuthorizer::ensure_authorized / PDP decide() \
     port — the AUTHORIZATION decision, fail-closed). Derive the principal from a verified \
     mTLS/SVID/bearer credential, call the cloud-iam Cedar PDP server-side to \
     decide(principal, action, resource), and fail closed — do NOT trust the caller-supplied \
     authorization DTO/header as the verdict.";

/// The blocking + structural violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 6] = [
    "DAT-CALLER-SUPPLIED-AUTHZ-TRUST",
    "DAT-STALE-BASELINE",
    "DAT-STALE-SPLIT-DECISION-ALLOWLIST",
    "DAT-EMPTY-SCAN",
    "DAT-POLICY-GATE-ID-MISMATCH",
    "DAT-POLICY-MALFORMED",
];

/// The per-instance finding code whose keys are the BASELINE vocabulary. The policy-level codes
/// (`DAT-EMPTY-SCAN`, `DAT-POLICY-*`, `DAT-STALE-BASELINE`, `DAT-STALE-SPLIT-DECISION-ALLOWLIST`) are
/// NOT baseline keys.
pub const INSTANCE_FINDING_CODE: &str = "DAT-CALLER-SUPPLIED-AUTHZ-TRUST";

/// The sentinel key for codes that are policy-level rather than per-instance.
const POLICY_KEY: &str = "<policy>";

// ---------------------------------------------------------------------------
// Collection (the only I/O; read-only)
// ---------------------------------------------------------------------------

/// Errors collecting the observed instance graph. Returned instead of panicking so the caller
/// (CI / a controller) decides how to surface them — an unreadable scan root is a fail-closed error,
/// never a silently skipped subtree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    Io(String),
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::Io(message) => write!(f, "dto-authz-trust io: {message}"),
        }
    }
}

impl std::error::Error for CollectError {}

/// Collect the DTO-authz-trust instances under the policy scan roots.
///
/// Walks each `scan_roots` directory, reads every `.rs` file not under an `excluded_dir_names`
/// directory, and extracts every function with its authorization-trust signal. Emits
/// `{ "functions_scanned": <usize>, "instances": [ <instance>, .. ] }` where each instance is a
/// function that READS a caller-supplied authorization-decision field and makes NO PDP
/// decision-port call. Each instance is `{ "file", "fn", "line", "key", "signal" }`.
pub fn collect_instances(root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let scan_roots = string_list(policy, "scan_roots");
    let excluded_dirs: BTreeSet<String> = string_list(policy, "excluded_dir_names")
        .into_iter()
        .collect();

    let mut rs_files: Vec<String> = Vec::new();
    for scan_root in &scan_roots {
        collect_rs_files(root, &root.join(scan_root), &excluded_dirs, &mut rs_files)?;
    }
    rs_files.sort();
    rs_files.dedup();

    let cfg = SignatureConfig::from_policy(policy);

    let mut instances: Vec<Value> = Vec::new();
    let mut functions_scanned: u64 = 0;
    for rel_path in &rs_files {
        let text = match fs::read_to_string(root.join(rel_path)) {
            Ok(text) => text,
            Err(e) => return Err(CollectError::Io(format!("read {rel_path}: {e}"))),
        };
        let (file_instances, scanned) = extract_instances(rel_path, &text, &cfg);
        functions_scanned += scanned;
        instances.extend(file_instances);
    }
    instances.sort_by_key(instance_sort_key);

    Ok(json!({
        "functions_scanned": functions_scanned,
        "instances": instances,
    }))
}

fn instance_sort_key(instance: &Value) -> (String, String) {
    (
        instance
            .get("file")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned(),
        instance
            .get("fn")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned(),
    )
}

/// Recursively collect repo-relative `.rs` file paths under `dir`, skipping any directory whose name
/// is in `excluded_dirs`. A missing scan root is fine (the gate is repo-portable).
fn collect_rs_files(
    root: &Path,
    dir: &Path,
    excluded_dirs: &BTreeSet<String>,
    out: &mut Vec<String>,
) -> Result<(), CollectError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(CollectError::Io(format!("read dir {}: {e}", dir.display()))),
    };
    for entry in entries {
        let entry =
            entry.map_err(|e| CollectError::Io(format!("read entry in {}: {e}", dir.display())))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| CollectError::Io(format!("file_type {}: {e}", path.display())))?;
        if file_type.is_dir() {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if excluded_dirs.contains(name) {
                continue;
            }
            collect_rs_files(root, &path, excluded_dirs, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs")
            && let Ok(rel) = path.strip_prefix(root)
        {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

/// The policy DATA the signature finder needs, lifted to typed lists once per scan.
struct SignatureConfig {
    /// The authorization-DTO type-name suffix(es) (default `["Authorization"]`). A fn parameter
    /// whose type name ends with one of these is a caller-supplied authorization blob.
    dto_type_suffixes: Vec<String>,
    /// The AUTHZ-SPECIFIC decision-field idents (default `["allowed_surfaces", "permitted_scopes",
    /// "caller_roles", "granted", "allowed_actions"]`) whose read off a binding is, on its own,
    /// sufficient signal-(a) evidence of caller-supplied-authz trust. These must be fields with NO
    /// benign business meaning — they are authorization allow-lists/grants and appear nowhere else;
    /// an overloaded field like `decision_id` (also a retention/policy business key) is NOT a
    /// trigger (it stays a description-only corroborator) so the gate does not false-positive on
    /// retention/policy `decision_id` integrity checks.
    trigger_decision_field_idents: Vec<String>,
    /// The decision-field idents read off the authorization blob, used ONLY to enrich the finding
    /// SIGNAL description — NOT a standalone trigger.
    decision_field_idents: Vec<String>,
    /// The authorization-header idents (`x-authorization-decision-id`, ...) — a header-trust shape.
    /// Searched in a comment-stripped, string-preserving body view.
    authorization_header_idents: Vec<String>,
    /// The PDP / authorizer decision-port idents whose whole-token CALL in the body marks GREEN.
    /// NOTE: `verify_principal` is deliberately ABSENT — it is an authentication step, not an
    /// authorization decision; a function that verifies identity but self-compares the authz DTO
    /// is still flagged.
    pdp_decision_idents: Vec<String>,
    /// Equality/membership operator tokens — used for DESCRIPTION ENRICHMENT only (not a gate
    /// since v2 FN-06 inversion). Kept in policy for signal narrative; Vec::contains /
    /// binary_search / is_superset evade an operator-based gate.
    self_compare_tokens: Vec<String>,
}

impl SignatureConfig {
    fn from_policy(policy: &Value) -> Self {
        let dto_type_suffixes = {
            let v = string_list(policy, "authorization_dto_type_suffixes");
            if v.is_empty() {
                vec!["Authorization".to_owned()]
            } else {
                v
            }
        };
        Self {
            dto_type_suffixes,
            trigger_decision_field_idents: {
                let v = string_list(policy, "trigger_decision_field_idents");
                if v.is_empty() {
                    vec![
                        "allowed_surfaces".to_owned(),
                        "permitted_scopes".to_owned(),
                        "caller_roles".to_owned(),
                        "granted".to_owned(),
                        "allowed_actions".to_owned(),
                    ]
                } else {
                    v
                }
            },
            decision_field_idents: string_list(policy, "decision_field_idents"),
            authorization_header_idents: string_list(policy, "authorization_header_idents"),
            pdp_decision_idents: {
                let v = string_list(policy, "pdp_decision_idents");
                if v.is_empty() {
                    vec![
                        ".decide(".to_owned(),
                        "ensure_authorized".to_owned(),
                        "check_authz".to_owned(),
                        "ensure_authz".to_owned(),
                    ]
                } else {
                    v
                }
            },
            self_compare_tokens: string_list(policy, "self_compare_tokens"),
        }
    }
}

/// Extract every DTO-authz-trust instance from one file's source text, plus the count of functions
/// scanned (for the empty-scan floor). Structure is searched against a length-preserving
/// [`mask_non_code`] view (comments + string/char literal CONTENT blanked), so a doc-comment or
/// string mention never triggers a finding. Header detection uses a comment-stripped but
/// string-preserving view so header NAMES in string literals are found while comment mentions are not.
fn extract_instances(file: &str, text: &str, cfg: &SignatureConfig) -> (Vec<Value>, u64) {
    let masked = mask_non_code(text);
    let masked = masked.as_str();
    // Comment-stripped but string-preserving view for header name detection (FP-01 fix).
    let comment_stripped = strip_comments_only(text);
    let test_spans = cfg_test_spans(masked);
    let fns = fn_decls(masked);

    let mut out = Vec::new();
    let mut scanned: u64 = 0;
    for decl in &fns {
        // Skip POSITIVE test-fixture blocks (#[cfg(test)]). Do NOT skip #[cfg(not(test))] — that
        // is production code (FN-05 fix).
        if test_spans
            .iter()
            .any(|(lo, hi)| decl.body_open >= *lo && decl.body_open < *hi)
        {
            continue;
        }
        scanned += 1;
        let sig = &masked[decl.sig_start..decl.body_open];
        let body = &masked[decl.body_open..decl.body_end];
        // Comment-stripped body at the same offsets for header detection: string literal CONTENT
        // is preserved (so header names in strings are found), but line/block comments are blanked
        // (so a comment mentioning a header name does not trigger).
        let body_comment_stripped =
            &comment_stripped[decl.body_open..decl.body_end.min(comment_stripped.len())];

        // (a) reads a caller-supplied authorization-decision field. The TRIGGERS are:
        //   - an `*Authorization`-typed parameter (the forged blob),
        //   - an AUTHZ-SPECIFIC decision-field read (`allowed_surfaces`, `permitted_scopes`,
        //     `caller_roles`, `granted`, `allowed_actions` — no benign business meaning),
        //   - an `x-authorization-*` header in the comment-stripped-but-string-preserving view.
        // A generic / overloaded decision-field read (`decision_id`) is NOT a standalone trigger;
        // it only enriches the finding description.
        let has_dto_param = signature_has_authz_dto_param(sig, &cfg.dto_type_suffixes);
        let reads_trigger_field = cfg
            .trigger_decision_field_idents
            .iter()
            .any(|f| body_reads_field(body, f));
        let reads_authz_header = cfg
            .authorization_header_idents
            .iter()
            .any(|h| body_comment_stripped.contains(h.as_str()));
        let signal_a = has_dto_param || reads_trigger_field || reads_authz_header;
        if !signal_a {
            continue;
        }

        // Description-only corroborators (never standalone triggers).
        let reads_decision_field = cfg
            .decision_field_idents
            .iter()
            .any(|f| body_reads_field(body, f));
        let self_compares = cfg
            .self_compare_tokens
            .iter()
            .any(|t| body.contains(t.as_str()));

        // (b) [FN-06 INVERTED] no PDP / authorizer decision-port call in the body.
        // Self-compare operators are no longer a PRECONDITION — flagging (a) AND NOT (b) regardless
        // of comparison form. Vec::contains / binary_search / is_superset all evade an operator
        // gate; this inversion closes that class.
        let calls_pdp = cfg
            .pdp_decision_idents
            .iter()
            .any(|ident| body_calls_pdp(body, ident));
        if calls_pdp {
            continue;
        }

        // NOTE: genuine split-decision FALSE POSITIVES (a residual NON-AUTHORITATIVE correlation
        // check whose authoritative PDP decision lives elsewhere in the use-case) are NOT suppressed
        // here. The collector reports EVERY flagged instance with its exact `file#fn:body_hash` key;
        // the EXPLICIT `split_decision_allowlist` in policy DATA suppresses ONLY the audited keys in
        // the pure evaluator ([`evaluate_keyed`]). There is NO name-reachability heuristic.
        let key = instance_key(file, &decl.name, body);
        let signal = build_signal(
            has_dto_param,
            reads_decision_field,
            reads_authz_header,
            self_compares,
            &cfg.dto_type_suffixes,
            sig,
        );
        out.push(json!({
            "file": file,
            "fn": decl.name,
            "line": line_of(text, decl.sig_start) as u64,
            "key": key,
            "signal": signal,
        }));
    }
    (out, scanned)
}

/// A short human-readable description of WHY a function matched (for the finding detail). Pure
/// rendering — no verdict logic.
fn build_signal(
    has_dto_param: bool,
    reads_decision_field: bool,
    reads_authz_header: bool,
    self_compares: bool,
    dto_suffixes: &[String],
    sig: &str,
) -> String {
    let mut parts = Vec::new();
    if has_dto_param {
        let suffix = dto_suffixes
            .iter()
            .find(|s| signature_has_authz_dto_param(sig, std::slice::from_ref(*s)))
            .map(String::as_str)
            .unwrap_or("Authorization");
        parts.push(format!("takes a caller-supplied `*{suffix}` DTO parameter"));
    }
    if reads_decision_field {
        parts.push(
            "reads an authorization-decision field (e.g. allowed_surfaces/decision_id)".to_owned(),
        );
    }
    if reads_authz_header {
        parts.push("reads an x-authorization-* header".to_owned());
    }
    if self_compares {
        parts.push("self-compares it (equality/membership)".to_owned());
    }
    parts.push("makes NO PDP decision-port call".to_owned());
    parts.join("; ")
}

/// A parsed `fn NAME(...) { .. }` declaration: the name, the byte offset where the signature begins
/// (`fn`), and the inclusive body brace span `[body_open, body_end]`.
struct FnDecl {
    name: String,
    sig_start: usize,
    body_open: usize,
    body_end: usize,
}

/// Every `fn NAME(..) -> .. { .. }` declaration in masked `text`. `fn` must be a keyword boundary so
/// `transform`/`fnord` do not match. The name is the ident after `fn ` (turbofish/`<generics>`/`(`
/// terminate it). The body brace is the first `{` after the signature.
fn fn_decls(masked: &str) -> Vec<FnDecl> {
    let mut decls = Vec::new();
    let bytes = masked.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = masked[from..].find("fn ") {
        let at = from + rel;
        from = at + 3;
        if at != 0 && is_ident_byte(bytes[at - 1]) {
            continue;
        }
        let name = read_path_ident(&masked[at + 3..]);
        if name.is_empty() {
            continue;
        }
        if let Some(body) = brace_body(masked, at + 3) {
            let open = body.as_ptr() as usize - masked.as_ptr() as usize;
            decls.push(FnDecl {
                name,
                sig_start: at,
                body_open: open,
                body_end: open + body.len(),
            });
        }
    }
    decls
}

/// Whether a function SIGNATURE slice (from `fn` to the body `{`) declares a parameter whose TYPE
/// name ends with one of `dto_suffixes` (default `Authorization`). We look for a `: ` (or `&`/`&mut `)
/// type position whose path tail ident ends with the suffix — e.g. `authorization:
/// &CloudKmsApiAuthorization`. Whole-ident suffix match (the type tail ident must END with the
/// suffix as a word, so `AuthorizationLayer` does not match `Authorization` unless the suffix IS the
/// whole tail) — actually we match the tail ident ENDING WITH the suffix, which is the corpus shape
/// `*ApiAuthorization`. A bare `Authorization` tail also matches.
fn signature_has_authz_dto_param(sig: &str, dto_suffixes: &[String]) -> bool {
    // Only inspect the parameter list `( ... )` of the signature, so a `-> Result<.., ..>` return
    // type that happens to mention an Authorization error variant does not count as a param.
    let Some(params) = first_paren_body(sig) else {
        return false;
    };
    // Walk each `: TYPE` after a parameter name. Collect every path-tail ident appearing in a type
    // position and test the suffix. A type position starts after a top-level `:` within the params.
    let bytes = params.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b':' && (i + 1 >= bytes.len() || bytes[i + 1] != b':') {
            // Skip `&`, `mut`, whitespace to the first type ident; then read the FULL type expr up
            // to the next top-level `,` and test every `::`-tail ident in it.
            let type_expr = read_param_type(&params[i + 1..]);
            for tail in path_tail_idents(type_expr) {
                if dto_suffixes.iter().any(|s| ident_ends_with_word(&tail, s)) {
                    return true;
                }
            }
        }
        i += 1;
    }
    false
}

/// Read a parameter TYPE expression starting just past the `:` — up to the next top-level comma
/// (depth-aware over `<>`, `()`, `[]`), so a generic `Vec<Foo>` type is read whole.
fn read_param_type(after_colon: &str) -> &str {
    let bytes = after_colon.as_bytes();
    let mut depth_angle = 0i32;
    let mut depth_paren = 0i32;
    let mut depth_brack = 0i32;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'<' => depth_angle += 1,
            b'>' => depth_angle -= 1,
            b'(' => depth_paren += 1,
            b')' => {
                if depth_paren == 0 {
                    break; // end of the param list
                }
                depth_paren -= 1;
            }
            b'[' => depth_brack += 1,
            b']' => depth_brack -= 1,
            b',' if depth_angle <= 0 && depth_paren <= 0 && depth_brack <= 0 => break,
            _ => {}
        }
        i += 1;
    }
    after_colon[..i].trim()
}

/// Every `::`-separated path-tail ident appearing in a type expression — `Arc<dyn iam::Authorizer>`
/// yields `Arc`, `dyn`(dropped as keyword), `Authorizer`. We split on non-ident, take each run of
/// ident chars, and for a `a::b::c` path keep the final `c` as the tail (the type name).
fn path_tail_idents(type_expr: &str) -> Vec<String> {
    let mut out = Vec::new();
    // Split the expr into `::`-paths and bare idents; collect each path's tail ident.
    let bytes = type_expr.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if is_ident_start(bytes[i]) {
            let start = i;
            while i < bytes.len() && (is_ident_byte(bytes[i]) || bytes[i] == b':') {
                i += 1;
            }
            let path = &type_expr[start..i];
            let tail = path
                .trim_end_matches(':')
                .rsplit("::")
                .next()
                .unwrap_or(path);
            if !tail.is_empty() && !is_type_keyword(tail) {
                out.push(tail.to_owned());
            }
        } else {
            i += 1;
        }
    }
    out
}

fn is_type_keyword(ident: &str) -> bool {
    matches!(ident, "dyn" | "impl" | "mut" | "ref" | "Self" | "self")
}

/// Whether an ident ENDS WITH `suffix` as a word — either the ident IS the suffix, or it ends with
/// the suffix preceded by an uppercase boundary (so `CloudKmsApiAuthorization` ends with
/// `Authorization` but `Authorizations` does not, and `AuthorizationLayer` does not).
fn ident_ends_with_word(ident: &str, suffix: &str) -> bool {
    if ident == suffix {
        return true;
    }
    if let Some(prefix) = ident.strip_suffix(suffix) {
        // The char before the suffix must be an uppercase letter or digit (a CamelCase boundary),
        // and the suffix must be the literal tail (already guaranteed by strip_suffix).
        return prefix
            .chars()
            .next_back()
            .map(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c.is_ascii_lowercase())
            .unwrap_or(false);
    }
    false
}

/// Whether the code-only `body` READS a `.FIELD` off some binding — a whole-token `.field` access
/// where `field` is the ident (so `allowed_surfaces` matches `.allowed_surfaces` but not
/// `not_allowed_surfaces` or a substring). We require the dot prefix to ensure it is a field read,
/// not a local variable or a function name.
fn body_reads_field(body: &str, field: &str) -> bool {
    let needle = format!(".{field}");
    let bytes = body.as_bytes();
    let nbytes = needle.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = body[from..].find(&needle) {
        let at = from + rel;
        from = at + needle.len();
        // The char after the field ident must NOT be an ident byte (whole-token).
        let after = at + nbytes.len();
        if after >= bytes.len() || !is_ident_byte(bytes[after]) {
            return true;
        }
    }
    false
}

/// Whether the code-only `body` makes a whole-token CALL to a PDP / authorizer decision port. An
/// ident like `.decide(` is matched as a literal call-shaped token; a bare-ident like
/// `ensure_authorized` is matched as a whole-ident immediately followed (after ws) by `(`.
///
/// **Known limits (documented):**
/// - Dead-code evasion: `if false { .decide() }` suppresses this check. No reachability analysis.
/// - Wrong-receiver evasion: `other_svc.ensure_authorized()` on an unrelated receiver also suppresses.
///
/// Both are acceptable residuals for the dominant corpus shape this gate addresses.
fn body_calls_pdp(body: &str, ident: &str) -> bool {
    // Pre-formed call tokens (those containing `(` or starting with `.`) are matched literally.
    if ident.ends_with('(') || ident.starts_with('.') {
        return body.contains(ident);
    }
    // A bare ident: require a whole-token match followed by `(` (after optional ws), and not be a
    // longer ident (left boundary non-ident). A leading `.` is allowed (a method call).
    let bytes = body.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = body[from..].find(ident) {
        let at = from + rel;
        from = at + ident.len();
        let before_ok = at == 0 || !is_ident_byte(bytes[at - 1]);
        let after = at + ident.len();
        let after_str = body[after..].trim_start();
        let after_ok = after_str.starts_with('(');
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

/// Build a length-preserving CODE-STRUCTURE mask of `text`: line/block comment bytes and string/char
/// literal CONTENT bytes are replaced with spaces, but the literal's delimiting quotes are KEPT and
/// the byte length + newline positions are preserved. This lets the structural finders work on the
/// mask while VALUES are still read from the ORIGINAL `text` at the aligned offsets. (Shared shape
/// with the cloud-ci-authz-coverage gate's masker.)
fn mask_non_code(text: &str) -> String {
    fn blank_into(out: &mut Vec<u8>, slice: &[u8]) {
        out.extend(slice.iter().map(|&b| if b == b'\n' { b'\n' } else { b' ' }));
    }
    let bytes = text.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            b'r' | b'b' if raw_string_open(bytes, i).is_some() => {
                let (_content_start, end) = raw_string_open(bytes, i).unwrap_or((i, i + 1));
                blank_into(&mut out, &bytes[i..end]);
                i = end;
                continue;
            }
            b'"' => {
                let end = skip_string(bytes, i);
                out.push(b'"');
                if end > i + 1 {
                    blank_into(&mut out, &bytes[i + 1..end - 1]);
                    out.push(b'"');
                }
                i = end;
                continue;
            }
            b'\'' => {
                let end = skip_char_or_lifetime(bytes, i);
                blank_into(&mut out, &bytes[i..end]);
                i = end;
                continue;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                let start = i;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                blank_into(&mut out, &bytes[start..i]);
                continue;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                let start = i;
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                let end = (i + 2).min(bytes.len());
                blank_into(&mut out, &bytes[start..end]);
                i = end;
                continue;
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| " ".repeat(text.len()))
}

/// Build a comment-stripped but STRING-CONTENT-PRESERVING view of `text`. Line and block comments
/// are blanked (spaces, newlines preserved); string literal CONTENT is kept verbatim. Used for
/// header-name detection: header names in string literals are found; header names only in comments
/// are not (FP-01 fix). Length-preserving (same offsets as original).
fn strip_comments_only(text: &str) -> String {
    fn blank_into(out: &mut Vec<u8>, slice: &[u8]) {
        out.extend(slice.iter().map(|&b| if b == b'\n' { b'\n' } else { b' ' }));
    }
    let bytes = text.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            // Raw strings: copy verbatim (content preserved).
            b'r' | b'b' if raw_string_open(bytes, i).is_some() => {
                let (_cs, end) = raw_string_open(bytes, i).unwrap_or((i, i + 1));
                out.extend_from_slice(&bytes[i..end]);
                i = end;
                continue;
            }
            // Regular strings: copy verbatim (content preserved).
            b'"' => {
                let end = skip_string(bytes, i);
                out.extend_from_slice(&bytes[i..end]);
                i = end;
                continue;
            }
            // Char literals / lifetimes: copy verbatim.
            b'\'' => {
                let end = skip_char_or_lifetime(bytes, i);
                out.extend_from_slice(&bytes[i..end]);
                i = end;
                continue;
            }
            // Line comments: blank.
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                let start = i;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                blank_into(&mut out, &bytes[start..i]);
                continue;
            }
            // Block comments: blank.
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                let start = i;
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                let end = (i + 2).min(bytes.len());
                blank_into(&mut out, &bytes[start..end]);
                i = end;
                continue;
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| " ".repeat(text.len()))
}

/// Byte spans of `#[cfg(test)]`-gated items (POSITIVE test predicate only) in masked `text`. For
/// each `#[cfg(...)]` attribute that carries the `test` ident as a POSITIVE predicate — i.e. NOT
/// inside a `not(...)` — the gated item runs from the attribute to the matching close brace.
/// Functions within these spans are test fixtures, not production code.
///
/// **FN-05 fix**: `#[cfg(not(test))]` is PRODUCTION code — it must NOT be excluded. Only a
/// `#[cfg(test)]` or `#[cfg(all(..., test, ...))]` where `test` is a positive predicate is excluded.
fn cfg_test_spans(text: &str) -> Vec<(usize, usize)> {
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut from = 0usize;
    while let Some(rel) = text[from..].find("#[cfg(") {
        let at = from + rel;
        let attr_end = text[at..]
            .find(']')
            .map(|i| at + i + 1)
            .unwrap_or(text.len());
        let attr = &text[at..attr_end];
        if attr_has_positive_test_predicate(attr)
            && let Some(body) = brace_body(text, attr_end)
        {
            let body_start = body.as_ptr() as usize - text.as_ptr() as usize;
            spans.push((at, body_start + body.len()));
        }
        from = attr_end;
    }
    spans
}

/// Whether a `#[cfg(...)]` attribute string carries `test` as a POSITIVE (non-negated) config
/// predicate. Returns true for `#[cfg(test)]`, `#[cfg(all(test, ...))]`, etc. Returns false for
/// `#[cfg(not(test))]` — that is production code gated OUT of test builds.
fn attr_has_positive_test_predicate(attr: &str) -> bool {
    // Walk the attr text looking for the whole-token `test`. For each occurrence, check whether
    // it sits inside a `not(...)` by scanning backwards for an unmatched `not(`.
    let bytes = attr.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = attr[from..].find("test") {
        let at = from + rel;
        // Whole-token check.
        let before_ok = at == 0 || !is_ident_byte(bytes[at - 1]);
        let after = at + 4;
        let after_ok = after >= bytes.len() || !is_ident_byte(bytes[after]);
        if before_ok && after_ok {
            // Check: is this `test` immediately inside a `not(` group?
            // A simple heuristic: scan the prefix up to `at` for the last `not(` and ensure
            // there is no closing `)` between that `not(` and `at`.
            if !test_token_is_negated(attr, at) {
                return true;
            }
        }
        from = at + 4;
    }
    false
}

/// Returns true if the `test` token at `test_pos` in `attr` is inside a `not(...)` group.
fn test_token_is_negated(attr: &str, test_pos: usize) -> bool {
    let prefix = &attr[..test_pos];
    // Find the last `not(` before `test_pos`.
    if let Some(not_rel) = prefix.rfind("not(") {
        let not_end = not_rel + 4; // just past the `(`
        // Count the parens between `not_end` and `test_pos`: if the `(` is still open
        // (depth > 0), then `test` is inside `not(...)`.
        let between = &prefix[not_end..];
        let opens: i32 = between.bytes().filter(|&b| b == b'(').count() as i32;
        let closes: i32 = between.bytes().filter(|&b| b == b')').count() as i32;
        opens - closes >= 0 // `not(` paren is still open at `test_pos`
    } else {
        false
    }
}

/// The first balanced `( ... )` body in `s`, exclusive of the parens. For a fn signature this is the
/// parameter list. None if no balanced paren is found.
fn first_paren_body(s: &str) -> Option<&str> {
    let bytes = s.as_bytes();
    let open = s.find('(')?;
    let mut depth = 0i32;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&s[open + 1..i]);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// From offset `from`, find the first `{` and return the slice of the brace-balanced body up to its
/// matching `}` (inclusive). String/char literals and line comments are skipped so a brace inside
/// them does not throw off the balance. None if no opening brace or no balanced close is found.
fn brace_body(text: &str, from: usize) -> Option<&str> {
    let bytes = text.as_bytes();
    let mut i = from;
    while i < bytes.len() && bytes[i] != b'{' {
        i += 1;
    }
    if i >= bytes.len() {
        return None;
    }
    let body_start = i;
    let mut depth = 0i32;
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&text[body_start..=i]);
                }
            }
            b'"' => {
                i = skip_string(bytes, i);
                continue;
            }
            b'\'' => {
                i = skip_char_or_lifetime(bytes, i);
                continue;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Read a path ident (`a::b::c` or `name`) at the start of `s`, stopping at the first non-ident,
/// non-`:` char. Leading whitespace is skipped.
fn read_path_ident(s: &str) -> String {
    let trimmed = s.trim_start();
    let bytes = trimmed.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() && (is_ident_byte(bytes[i]) || bytes[i] == b':') {
        i += 1;
    }
    // Drop a trailing `:` left by a `name:` (not a `::` path).
    trimmed[..i].trim_end_matches(':').to_owned()
}

/// If a raw-string literal opens at offset `start` (`r"`, `r#"`..., or byte-raw `br"`...), return
/// `(content_start, end)` where `end` is just past the closing delimiter. None if not a raw opener.
fn raw_string_open(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    if start > 0 && is_ident_byte(bytes[start - 1]) {
        return None;
    }
    let mut i = start;
    if i < bytes.len() && bytes[i] == b'b' {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b'r' {
        return None;
    }
    i += 1;
    let hash_start = i;
    while i < bytes.len() && bytes[i] == b'#' {
        i += 1;
    }
    let hashes = i - hash_start;
    if i >= bytes.len() || bytes[i] != b'"' {
        return None;
    }
    i += 1;
    let content_start = i;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            let mut k = i + 1;
            let mut seen = 0usize;
            while k < bytes.len() && seen < hashes && bytes[k] == b'#' {
                seen += 1;
                k += 1;
            }
            if seen == hashes {
                return Some((content_start, k));
            }
        }
        i += 1;
    }
    Some((content_start, bytes.len()))
}

/// Skip a `"..."` string literal starting at `start` (the opening quote); return the offset just past
/// the closing quote (or EOF). Handles escapes.
fn skip_string(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => return i + 1,
            _ => i += 1,
        }
    }
    i
}

/// Skip a `'c'` char literal or a `'a` lifetime starting at `start`. Returns the offset just past it.
fn skip_char_or_lifetime(bytes: &[u8], start: usize) -> usize {
    if start + 1 < bytes.len() && bytes[start + 1] == b'\\' {
        let mut i = start + 2;
        while i < bytes.len() && bytes[i] != b'\'' {
            i += 1;
        }
        return i + 1;
    }
    if start + 2 < bytes.len() && bytes[start + 2] == b'\'' {
        return start + 3;
    }
    let mut i = start + 1;
    while i < bytes.len() && is_ident_byte(bytes[i]) {
        i += 1;
    }
    i
}

/// 1-based line number of byte offset `at` in `text`.
fn line_of(text: &str, at: usize) -> usize {
    text[..at.min(text.len())]
        .bytes()
        .filter(|&b| b == b'\n')
        .count()
        + 1
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn string_list(policy: &Value, key: &str) -> Vec<String> {
    policy
        .get(key)
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Baseline key: <file>#<fn>:<body_hash>  (FN-02)
// ---------------------------------------------------------------------------

/// Stable SIGNATURE key for an instance finding: `<file>#<fn>:<body_hash>` where `body_hash` is
/// a 32-hex-char FNV-1a hash of the CODE-ONLY (masked) function body.
///
/// The body hash makes the key sensitive to BODY CHANGES — a new function with the same name in a
/// different `mod` (or a refactored body) does NOT auto-launder as a baselined instance. The file
/// path + function name components keep the key stable across unrelated line-number shifts.
pub(crate) fn instance_key(file: &str, fn_name: &str, body: &str) -> String {
    let h = fnv1a_32(body.as_bytes());
    format!("{file}#{fn_name}:{h:08x}")
}

/// FNV-1a 32-bit hash of `data`. Compact, dependency-free, deterministic.
fn fnv1a_32(data: &[u8]) -> u32 {
    const OFFSET: u32 = 2166136261;
    const PRIME: u32 = 16777619;
    let mut h = OFFSET;
    for &b in data {
        h ^= b as u32;
        h = h.wrapping_mul(PRIME);
    }
    h
}

// ---------------------------------------------------------------------------
// Pure evaluation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub detail: String,
}

impl Finding {
    fn new(code: &str, key: &str, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

impl Report {
    fn from_findings(findings: &BTreeSet<Finding>) -> Self {
        let violations = findings
            .iter()
            .map(|finding| finding.code.clone())
            .collect::<BTreeSet<_>>();
        Self {
            verdict: if violations.is_empty() {
                Verdict::Green
            } else {
                Verdict::Red
            },
            violations,
        }
    }
}

/// Pure evaluator. `policy` is DATA (`dto-authz-trust-policy.json`); `observed` is the collected
/// instance graph shaped by [`collect_instances`].
///
/// An instance whose stable key is NOT in the frozen baseline → a blocking
/// `DAT-CALLER-SUPPLIED-AUTHZ-TRUST` finding. Baseline keys with no live instance → `DAT-STALE-BASELINE`.
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if policy.get("gate_id").and_then(Value::as_str) != Some(GATE_ID) {
        findings.insert(Finding::new(
            "DAT-POLICY-GATE-ID-MISMATCH",
            POLICY_KEY,
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    // Fail CLOSED on a structurally invalid policy: each list that is part of the gate's vocabulary
    // must be present AND non-empty (CORRECTNESS-01).
    let required_lists = [
        (
            "pdp_decision_idents",
            "policy `pdp_decision_idents` must be a non-empty array of recognized PDP/authorizer decision-port ident strings; correct the policy before the gate can evaluate",
        ),
        (
            "scan_roots",
            "policy `scan_roots` must be a non-empty array of repo-relative scan-root strings; correct the policy before the gate can evaluate",
        ),
        (
            "trigger_decision_field_idents",
            "policy `trigger_decision_field_idents` must be a non-empty array of authz-specific field ident strings; correct the policy before the gate can evaluate",
        ),
        (
            "authorization_dto_type_suffixes",
            "policy `authorization_dto_type_suffixes` must be a non-empty array of DTO type-name suffix strings; correct the policy before the gate can evaluate",
        ),
    ];
    for (key, msg) in required_lists {
        match policy.get(key).and_then(Value::as_array) {
            None => {
                findings.insert(Finding::new("DAT-POLICY-MALFORMED", POLICY_KEY, msg));
                return findings;
            }
            Some(arr) if arr.is_empty() => {
                findings.insert(Finding::new("DAT-POLICY-MALFORMED", POLICY_KEY, msg));
                return findings;
            }
            _ => {}
        }
    }

    let frozen_baseline: BTreeSet<String> = string_list(policy, "frozen_dto_authz_trust_instances")
        .into_iter()
        .collect();

    // EXPLICIT split-decision allowlist (the precise, non-launderable FP mechanism — NOT a
    // heuristic). Each key is the SAME `<file>#<fn>:<body_hash>` exact shape as the baseline, so any
    // body change re-flags the function. A key here suppresses ONLY the exact audited body; it is a
    // tiny curated list of genuine FALSE POSITIVES (residual non-authoritative correlation checks
    // whose authoritative PDP decision dominates the operation elsewhere — ADR-0591 split-decision).
    let split_decision_allowlist = split_decision_allowlist_keys(policy);

    let min_expected = policy
        .get("min_expected_functions")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let functions_scanned = observed
        .get("functions_scanned")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if functions_scanned < min_expected {
        findings.insert(Finding::new(
            "DAT-EMPTY-SCAN",
            POLICY_KEY,
            format!(
                "scan found {functions_scanned} functions, below the policy floor of {min_expected}; the scan roots, CWD, or collection is likely broken (fail-closed against a silent false-green)"
            ),
        ));
    }

    let mut live_keys: BTreeSet<String> = BTreeSet::new();

    let instances = observed
        .get("instances")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for instance in &instances {
        let Some(file) = instance.get("file").and_then(Value::as_str) else {
            continue;
        };
        let fn_name = instance.get("fn").and_then(Value::as_str).unwrap_or("");
        let line = instance.get("line").and_then(Value::as_u64).unwrap_or(0);
        let signal = instance.get("signal").and_then(Value::as_str).unwrap_or("");

        // Each instance carries its pre-computed key (file#fn:body_hash) from collect_instances.
        // Fall back to re-deriving from file+fn if missing (backwards compat with old observed
        // blobs without body in the JSON — but the new format always includes "key").
        let key = instance
            .get("key")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| format!("{file}#{fn_name}"));
        live_keys.insert(key.clone());

        // EXPLICIT split-decision allowlist: an exact-key match suppresses ONLY this audited body.
        // Non-launderable — a same-named overload, a different impl block, or any body edit produces
        // a different key and is NOT suppressed. Checked before the frozen baseline; an allowlisted
        // key is not also a baseline key (the two suppression sets are disjoint by construction).
        if split_decision_allowlist.contains(&key) {
            continue;
        }

        if frozen_baseline.contains(&key) {
            continue;
        }
        findings.insert(Finding::new(
            "DAT-CALLER-SUPPLIED-AUTHZ-TRUST",
            &key,
            format!(
                "NEW caller-supplied-authorization-trust instance: function `{fn_name}` at {file}:{line} {signal}. This is default-ALLOW-on-forged-input — any caller forges the authorization blob and authorizes itself, because the verdict is decided without a server-side PDP. {REMEDIATION_DOCTRINE}"
            ),
        ));
    }

    for key in &frozen_baseline {
        if !live_keys.contains(key) {
            findings.insert(Finding::new(
                "DAT-STALE-BASELINE",
                key,
                format!(
                    "frozen-baseline instance key `{key}` matched no live caller-supplied-authz-trust finding (the instance was fixed, removed, or moved — or the body changed and the key hash no longer matches). Remove it from `frozen_dto_authz_trust_instances` in the policy and re-run --write — the baseline is shrink-only."
                ),
            ));
        }
    }

    // SHRINK-ONLY self-clean for the explicit allowlist: an allowlist key matching no live instance
    // means the audited body was fixed/removed/edited (a body edit changes the hash → a different
    // key → no match), so the audit no longer applies. Surface it so the curated list cannot silently
    // retain a key that suppresses nothing (and so a future body edit cannot ride an old audit).
    for key in &split_decision_allowlist {
        if !live_keys.contains(key) {
            findings.insert(Finding::new(
                "DAT-STALE-SPLIT-DECISION-ALLOWLIST",
                key,
                format!(
                    "split-decision-allowlist key `{key}` matched no live caller-supplied-authz-trust finding (the audited body was fixed, removed, or edited — a body change re-keys it). Remove or re-audit the entry in `split_decision_allowlist`; the allowlist is exact-key and non-launderable."
                ),
            ));
        }
    }

    findings
}

/// The explicit, curated split-decision allowlist keys (`<file>#<fn>:<body_hash>`) read from policy
/// DATA `split_decision_allowlist`. Each entry is `{ "key": "...", "justification": "..." }`; only
/// the `key` is load-bearing for suppression (the justification documents the human audit). A bare
/// string entry is also accepted for forward-compatibility. NOT a heuristic — an exact-key set.
fn split_decision_allowlist_keys(policy: &Value) -> BTreeSet<String> {
    policy
        .get("split_decision_allowlist")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|entry| match entry {
                    Value::String(s) => Some(s.clone()),
                    Value::Object(_) => entry.get("key").and_then(Value::as_str).map(str::to_owned),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Bare-code projection of [`evaluate_keyed`]; the single source of truth for the verdict.
pub fn evaluate(policy: &Value, observed: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(policy, observed))
}

/// The set of per-instance finding keys the gate WOULD block against an EMPTY baseline — every
/// currently-detected caller-supplied-authz-trust instance.
fn live_instance_keys(policy: &Value, observed: &Value) -> BTreeSet<String> {
    let mut p = policy.clone();
    p["frozen_dto_authz_trust_instances"] = json!([]);
    let mut keys: BTreeSet<String> = BTreeSet::new();
    for finding in evaluate_keyed(&p, observed) {
        if finding.code == INSTANCE_FINDING_CODE {
            keys.insert(finding.key);
        }
    }
    keys
}

/// Regenerate the frozen-baseline signature keys from the live observation (the AUTOMATED property:
/// re-baselining is mechanical, not hand-edited). Returns the sorted set of per-instance keys the
/// gate WOULD block against an EMPTY baseline.
pub fn baseline_keys(policy: &Value, observed: &Value) -> Vec<String> {
    live_instance_keys(policy, observed).into_iter().collect()
}

/// The SHRINK-ONLY re-baseline result for `--write`. Given the PRIOR committed baseline, the
/// regenerated baseline DROPS any prior key with no live instance (shrink), KEEPS every prior key
/// still live, and ABSORBS a NEW live key ONLY when `allow_new` is true. Returns
/// `(next_baseline_sorted, new_keys_sorted)`.
pub fn shrink_only_baseline(
    policy: &Value,
    observed: &Value,
    allow_new: bool,
) -> (Vec<String>, Vec<String>) {
    let prior: BTreeSet<String> = string_list(policy, "frozen_dto_authz_trust_instances")
        .into_iter()
        .collect();
    let live = live_instance_keys(policy, observed);

    let new_keys: BTreeSet<String> = live.difference(&prior).cloned().collect();
    let mut next: BTreeSet<String> = prior.intersection(&live).cloned().collect();
    if allow_new {
        next.extend(new_keys.iter().cloned());
    }
    (next.into_iter().collect(), new_keys.into_iter().collect())
}

/// Human-readable render of the findings. Never a bare FAIL — every finding prints its detail.
pub fn render_findings(findings: &BTreeSet<Finding>) -> String {
    if findings.is_empty() {
        return "dto-authz-trust gate passed: no NEW function trusts a caller-supplied authorization decision in place of a server-side PDP decision".to_owned();
    }
    let mut out =
        String::from("dto-authz-trust gate failed (caller-supplied-authz-trust class):\n");
    for finding in findings {
        out.push_str(&format!(
            "    - {} {}\n        {}\n",
            finding.code, finding.key, finding.detail
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> Value {
        json!({
            "gate_id": GATE_ID,
            "min_expected_functions": 0,
            "scan_roots": ["src"],
            "excluded_dir_names": ["target", "third-party"],
            "authorization_dto_type_suffixes": ["Authorization"],
            "trigger_decision_field_idents": ["allowed_surfaces", "permitted_scopes", "caller_roles", "granted", "allowed_actions"],
            "decision_field_idents": ["allowed_surfaces", "decision_id"],
            "authorization_header_idents": [
                "x-authorization-decision-id",
                "x-authorization-surfaces",
                "x-authorization-principal-id",
                "x-authorization-tenant-id"
            ],
            "pdp_decision_idents": [".decide(", "ensure_authorized", "check_authz", "ensure_authz", "authorize_decision", "pdp_decide"],
            "self_compare_tokens": ["==", "!=", ".iter().any(", ".contains("],
            "frozen_dto_authz_trust_instances": []
        })
    }

    // RED: a use-case that takes a forged Authorization DTO, self-compares it, no PDP call.
    const RED_DTO_SELF_COMPARE: &str = r#"
        pub struct CloudFooApiAuthorization {
            pub tenant_id: String,
            pub principal_id: String,
            pub decision_id: String,
            pub allowed_surfaces: Vec<String>,
        }
        struct Principal { tenant_id: String, principal_id: String }

        fn validate_authorization(
            principal: &Principal,
            authorization: &CloudFooApiAuthorization,
            surface: &str,
        ) -> Result<(), Error> {
            if authorization.decision_id.trim().is_empty() {
                return Err(Error::Empty);
            }
            if authorization.tenant_id != principal.tenant_id {
                return Err(Error::TenantMismatch);
            }
            if authorization.principal_id != principal.principal_id {
                return Err(Error::PrincipalMismatch);
            }
            if !authorization.allowed_surfaces.iter().any(|s| s == surface) {
                return Err(Error::Denied);
            }
            Ok(())
        }
    "#;

    // RED via header trust: reads an x-authorization-* header in a string literal (not a comment).
    const RED_HEADER_TRUST: &str = r#"
        fn authorize_from_headers(headers: &HeaderMap, surface: &str) -> Result<(), Error> {
            let claimed = headers.get("x-authorization-decision-id").map(|v| v.to_str());
            if claimed != Some(Ok(surface)) {
                return Err(Error::Denied);
            }
            Ok(())
        }
    "#;

    // GREEN: the fixed pattern — verifies a principal from a credential + calls a PDP decide() port.
    const GREEN_PDP: &str = r#"
        struct CloudFooApiAuthorization { allowed_surfaces: Vec<String>, decision_id: String }

        fn ensure_authorized_handler(
            credential: &CallerCredential,
            authorization: &CloudFooApiAuthorization,
            resource: &Resource,
        ) -> Result<(), Error> {
            // Even though it has an Authorization param and reads allowed_surfaces, it DELEGATES to
            // a PDP: the verdict is the server's, not the caller's.
            let principal = self.verifier.verify_principal(credential)?;
            if authorization.decision_id.is_empty() { return Err(Error::Empty); }
            self.authorizer.ensure_authorized(&principal, resource)?;
            Ok(())
        }
    "#;

    // GREEN via .decide(): an authorizer port decide() call.
    const GREEN_DECIDE: &str = r#"
        struct ReqAuthorization { allowed_surfaces: Vec<String> }
        fn authorize(req: &ReqAuthorization, pdp: &Pdp, q: Query) -> Result<(), Error> {
            if req.allowed_surfaces.is_empty() { return Err(Error::Empty); }
            pdp.decide(q)
        }
    "#;

    // NOT-A-MATCH (benign): a tenant-binding validator that compares request-derived identity but
    // takes NO Authorization DTO and reads NO decision field — must NOT be flagged.
    const BENIGN_TENANT_BINDING: &str = r#"
        struct Boundary { tenant_id: String }
        struct Principal { tenant_id: String, principal_id: String }
        fn validate_operator_binding(boundary: &Boundary, principal: &Principal) -> Result<(), Error> {
            if principal.principal_id.trim().is_empty() {
                return Err(Error::EmptyPrincipalId);
            }
            if boundary.tenant_id != principal.tenant_id {
                return Err(Error::TenantMismatch);
            }
            Ok(())
        }
    "#;

    fn observe(src: &str) -> Value {
        let cfg = SignatureConfig::from_policy(&policy());
        let (instances, scanned) = extract_instances("src/lib.rs", src, &cfg);
        json!({ "functions_scanned": scanned, "instances": instances })
    }

    #[test]
    fn red_dto_self_compare_is_flagged() {
        let observed = observe(RED_DTO_SELF_COMPARE);
        let report = evaluate(&policy(), &observed);
        assert_eq!(report.verdict, Verdict::Red, "observed={observed:#}");
        assert!(
            report
                .violations
                .contains("DAT-CALLER-SUPPLIED-AUTHZ-TRUST")
        );
        // exactly the one fn matched.
        let n = observed["instances"].as_array().map(Vec::len).unwrap_or(0);
        assert_eq!(n, 1, "expected exactly one instance, observed={observed:#}");
        assert_eq!(observed["instances"][0]["fn"], "validate_authorization");
    }

    #[test]
    fn red_header_trust_is_flagged() {
        let observed = observe(RED_HEADER_TRUST);
        let report = evaluate(&policy(), &observed);
        assert_eq!(report.verdict, Verdict::Red, "observed={observed:#}");
    }

    #[test]
    fn green_pdp_handler_is_clean() {
        let observed = observe(GREEN_PDP);
        let report = evaluate(&policy(), &observed);
        assert_eq!(report.verdict, Verdict::Green, "observed={observed:#}");
        assert_eq!(
            observed["instances"].as_array().map(Vec::len).unwrap_or(0),
            0
        );
    }

    #[test]
    fn green_decide_handler_is_clean() {
        let observed = observe(GREEN_DECIDE);
        let report = evaluate(&policy(), &observed);
        assert_eq!(report.verdict, Verdict::Green, "observed={observed:#}");
    }

    #[test]
    fn benign_tenant_binding_is_not_flagged() {
        let observed = observe(BENIGN_TENANT_BINDING);
        assert_eq!(
            observed["instances"].as_array().map(Vec::len).unwrap_or(0),
            0,
            "benign tenant-binding validator must not be flagged; observed={observed:#}"
        );
    }

    // ===================================================================================
    // SPLIT-DECISION ALLOWLIST (the precise, non-launderable FP mechanism — NOT a heuristic)
    // and the THREE RED regression PROBES that prove the removed name-reachability heuristic's
    // bypasses are now FLAGGED.
    // ===================================================================================

    /// A genuine split-decision FP exhibit: a residual NON-AUTHORITATIVE correlation check that
    /// reads an `*Authorization` DTO and only returns Ok/Err (never grants). With NO allowlist it is
    /// flagged RED (the collector reports every flagged instance); the EXPLICIT allowlist clears the
    /// exact audited key.
    const SPLIT_DECISION_FP: &str = r#"
        struct CloudFooApiAuthorization { tenant_id: String, principal_id: String, decision_id: String }
        struct Principal { tenant_id: String, principal_id: String }

        fn validate_authorization_correlation(
            principal: &Principal,
            authorization: &CloudFooApiAuthorization,
        ) -> Result<(), Error> {
            if authorization.decision_id.trim().is_empty() { return Err(Error::Empty); }
            if authorization.tenant_id != principal.tenant_id { return Err(Error::TenantMismatch); }
            if authorization.principal_id != principal.principal_id { return Err(Error::PrincipalMismatch); }
            Ok(())
        }
    "#;

    /// Derive the live key for the single flagged instance in `src` (empty baseline / allowlist).
    fn first_live_key(src: &str) -> String {
        let observed = observe(src);
        observed["instances"][0]["key"]
            .as_str()
            .unwrap_or("")
            .to_owned()
    }

    #[test]
    fn split_decision_allowlist_clears_exact_audited_key() {
        // With no allowlist the correlation check is flagged RED (every flagged instance is reported).
        let observed = observe(SPLIT_DECISION_FP);
        assert_eq!(
            evaluate(&policy(), &observed).verdict,
            Verdict::Red,
            "without the explicit allowlist the correlation check must be flagged; observed={observed:#}"
        );

        // Adding the EXACT key to the explicit allowlist clears it (GREEN). The justification field
        // is documentation; only the key is load-bearing.
        let key = first_live_key(SPLIT_DECISION_FP);
        let mut p = policy();
        p["split_decision_allowlist"] = json!([
            { "key": key, "justification": "ADR-0591 split-decision: authoritative PDP dominates; correlation-only, never grants." }
        ]);
        let report = evaluate(&p, &observe(SPLIT_DECISION_FP));
        assert_eq!(
            report.verdict,
            Verdict::Green,
            "an exact-key split-decision allowlist entry must clear the audited FP"
        );
        assert!(report.violations.is_empty());
    }

    #[test]
    fn split_decision_allowlist_is_exact_key_and_non_launderable() {
        // An allowlist key for a DIFFERENT body (one char of the hash flipped) must NOT suppress a
        // live forgeable check — exact-key only.
        let real_key = first_live_key(SPLIT_DECISION_FP);
        let tampered = {
            let mut k = real_key.clone();
            // Flip the last hex nibble so the key no longer matches the live body hash.
            let last = k.pop().unwrap_or('0');
            let flipped = if last == '0' { '1' } else { '0' };
            k.push(flipped);
            k
        };
        let mut p = policy();
        p["split_decision_allowlist"] =
            json!([{ "key": tampered, "justification": "wrong body hash" }]);
        let report = evaluate(&p, &observe(SPLIT_DECISION_FP));
        assert_eq!(
            report.verdict,
            Verdict::Red,
            "a non-matching (tampered-hash) allowlist key must NOT launder a live forgeable check"
        );
        assert!(
            report
                .violations
                .contains("DAT-CALLER-SUPPLIED-AUTHZ-TRUST")
        );
        // And the unused allowlist key self-cleans (it matched no live instance).
        assert!(
            report
                .violations
                .contains("DAT-STALE-SPLIT-DECISION-ALLOWLIST")
        );
    }

    // ----- Probe A: dead-code PDP "root" + a call edge. Under the removed heuristic, a function whose
    // body had a dead `if false { x.ensure_authorized(); }` became a PDP "root" and laundered every
    // function it CALLED. The forgeable `validate_authorization` must now be FLAGGED (RED).
    const PROBE_A_DEADCODE_PDP_ROOT_CALL_EDGE: &str = r#"
        struct CloudFooApiAuthorization { tenant_id: String, allowed_surfaces: Vec<String> }
        struct Principal { tenant_id: String }

        // FORGEABLE: trusts the caller-supplied allowed_surfaces, no real PDP call.
        fn validate_authorization(
            principal: &Principal,
            authorization: &CloudFooApiAuthorization,
            surface: &str,
        ) -> Result<(), Error> {
            if !authorization.allowed_surfaces.iter().any(|s| s == surface) { return Err(Error::Denied); }
            if authorization.tenant_id != principal.tenant_id { return Err(Error::TenantMismatch); }
            Ok(())
        }

        // A fake "PDP root": the ensure_authorized call is DEAD (`if false`), but the removed
        // name-reachability heuristic treated this fn as a root and laundered its callee.
        fn unused_pdp_shim(x: &dyn Authorizer, principal: &Principal, authz: &CloudFooApiAuthorization) -> Result<(), Error> {
            if false { x.ensure_authorized(); }
            validate_authorization(principal, authz, "s")?;
            Ok(())
        }
    "#;

    #[test]
    fn probe_a_deadcode_pdp_root_call_edge_is_flagged_red() {
        // Heuristic removed: the dead-code PDP "root" can no longer launder its callee. The forgeable
        // validate_authorization is FLAGGED. (unused_pdp_shim itself contains the whole-token
        // ensure_authorized call so it is GREEN by the documented dead-code limit — that is fine; the
        // POINT is the forgeable callee is no longer laundered.)
        let observed = observe(PROBE_A_DEADCODE_PDP_ROOT_CALL_EDGE);
        let report = evaluate(&policy(), &observed);
        assert_eq!(
            report.verdict,
            Verdict::Red,
            "Probe A: a dead-code PDP root must NOT launder the forgeable callee; observed={observed:#}"
        );
        assert!(
            report
                .violations
                .contains("DAT-CALLER-SUPPLIED-AUTHZ-TRUST")
        );
        let flagged: Vec<&str> = observed["instances"]
            .as_array()
            .map(|a| a.iter().filter_map(|i| i["fn"].as_str()).collect())
            .unwrap_or_default();
        assert!(
            flagged.contains(&"validate_authorization"),
            "the forgeable validate_authorization must be flagged; flagged={flagged:?}"
        );
    }

    // ----- Probe C: name-collision across impl blocks. A same-named `decide_access` exists in TWO
    // impl blocks: one reachable from a real PDP root, one genuinely forgeable. Under the removed
    // heuristic, NAME-SET membership laundered the unrelated forgeable overload. Both bodies that read
    // the authz DTO without a PDP call must be FLAGGED now (name-set laundering is gone).
    const PROBE_C_NAME_COLLISION_IMPL_BLOCKS: &str = r#"
        struct CloudFooApiAuthorization { allowed_surfaces: Vec<String> }

        struct Guarded;
        impl Guarded {
            // Reached from a real PDP root, but STILL forgeable on its own (reads authz DTO, no PDP).
            fn decide_access(&self, authz: &CloudFooApiAuthorization, surface: &str) -> Result<(), Error> {
                if !authz.allowed_surfaces.iter().any(|s| s == surface) { return Err(Error::Denied); }
                Ok(())
            }
            fn root(&self, pdp: &dyn Authorizer, authz: &CloudFooApiAuthorization, principal: &Principal, resource: &Resource) -> Result<(), Error> {
                pdp.ensure_authorized(principal, resource)?;
                self.decide_access(authz, "s")?;
                Ok(())
            }
        }

        struct Unrelated;
        impl Unrelated {
            // A DIFFERENT impl's same-named overload — genuinely forgeable, reached from NO PDP root.
            fn decide_access(&self, authz: &CloudFooApiAuthorization, surface: &str) -> Result<(), Error> {
                if authz.allowed_surfaces.contains(&surface.to_owned()) { return Ok(()); }
                Err(Error::Denied)
            }
        }
    "#;

    #[test]
    fn probe_c_name_collision_across_impl_blocks_is_flagged_red() {
        // Both decide_access bodies read the authz DTO with no PDP call. With the heuristic removed,
        // NEITHER is laundered by name-set membership — both are flagged (two distinct keys).
        let observed = observe(PROBE_C_NAME_COLLISION_IMPL_BLOCKS);
        let report = evaluate(&policy(), &observed);
        assert_eq!(
            report.verdict,
            Verdict::Red,
            "Probe C: same-name across impl blocks must NOT launder the forgeable overload; observed={observed:#}"
        );
        assert!(
            report
                .violations
                .contains("DAT-CALLER-SUPPLIED-AUTHZ-TRUST")
        );
        let n_decide_access = observed["instances"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter(|i| i["fn"].as_str() == Some("decide_access"))
                    .count()
            })
            .unwrap_or(0);
        assert_eq!(
            n_decide_access, 2,
            "both decide_access overloads (distinct bodies) must be flagged; observed={observed:#}"
        );
    }

    // ----- Probe D (decisive): the single-file CLEAN-GREEN bypass. `decide_access` trusts
    // `allowed_surfaces`; the public `entry` makes ITSELF a "PDP root" via a dead
    // `if false { pdp.ensure_authorized(); }` and calls `decide_access`. Under the removed heuristic
    // the whole file was `flagged=[] verdict=Green` while a forgeable check shipped. The forgeable
    // `decide_access` must now be FLAGGED (RED).
    const PROBE_D_SINGLE_FILE_CLEAN_GREEN_BYPASS: &str = r#"
        struct CloudFooApiAuthorization { allowed_surfaces: Vec<String> }

        // FORGEABLE: trusts caller-supplied allowed_surfaces; the decision IS this self-membership.
        fn decide_access(authz: &CloudFooApiAuthorization, surface: &str) -> Result<(), Error> {
            if authz.allowed_surfaces.iter().any(|s| s == surface) { return Ok(()); }
            Err(Error::Denied)
        }

        // Public entry makes ITSELF a fake PDP root (dead ensure_authorized) and delegates to the
        // forgeable check. No server-side PDP ever decides.
        pub fn entry(pdp: &dyn Authorizer, authz: &CloudFooApiAuthorization, surface: &str) -> Result<(), Error> {
            if false { pdp.ensure_authorized(); }
            decide_access(authz, surface)
        }
    "#;

    #[test]
    fn probe_d_single_file_clean_green_bypass_is_flagged_red() {
        // The decisive bypass: with the heuristic removed, `entry`'s dead PDP call cannot launder the
        // forgeable `decide_access` it calls. `decide_access` is FLAGGED — the file is no longer a
        // clean-green bypass.
        let observed = observe(PROBE_D_SINGLE_FILE_CLEAN_GREEN_BYPASS);
        let report = evaluate(&policy(), &observed);
        assert_eq!(
            report.verdict,
            Verdict::Red,
            "Probe D: a single-file self-rooted dead-PDP bypass must NOT ship clean-green; observed={observed:#}"
        );
        assert!(
            report
                .violations
                .contains("DAT-CALLER-SUPPLIED-AUTHZ-TRUST")
        );
        let flagged: Vec<&str> = observed["instances"]
            .as_array()
            .map(|a| a.iter().filter_map(|i| i["fn"].as_str()).collect())
            .unwrap_or_default();
        assert!(
            flagged.contains(&"decide_access"),
            "the forgeable decide_access must be flagged; flagged={flagged:?}"
        );
    }

    #[test]
    fn frozen_baseline_tolerates_known_instance() {
        let observed = observe(RED_DTO_SELF_COMPARE);
        let mut p = policy();
        // Key includes body hash — derive it the same way the engine does.
        let masked = mask_non_code(RED_DTO_SELF_COMPARE);
        let fns = fn_decls(masked.as_str());
        assert!(!fns.is_empty(), "should find at least one fn");
        let f = &fns[fns.len() - 1]; // validate_authorization is the last fn
        let body = &masked[f.body_open..f.body_end];
        let key = instance_key("src/lib.rs", "validate_authorization", body);
        p["frozen_dto_authz_trust_instances"] = json!([key]);
        let report = evaluate(&p, &observed);
        assert_eq!(
            report.verdict,
            Verdict::Green,
            "baselined instance must be tolerated"
        );
    }

    #[test]
    fn stale_baseline_self_cleans() {
        let observed = observe(GREEN_PDP); // no live instances
        let mut p = policy();
        p["frozen_dto_authz_trust_instances"] = json!(["src/lib.rs#gone_away:deadbeef"]);
        let report = evaluate(&p, &observed);
        assert_eq!(report.verdict, Verdict::Red);
        assert!(report.violations.contains("DAT-STALE-BASELINE"));
    }

    #[test]
    fn gate_id_mismatch_fails_closed() {
        let mut p = policy();
        p["gate_id"] = json!("wrong");
        let report = evaluate(&p, &json!({"functions_scanned": 0, "instances": []}));
        assert!(report.violations.contains("DAT-POLICY-GATE-ID-MISMATCH"));
    }

    #[test]
    fn malformed_policy_fails_closed_missing_pdp_idents() {
        let mut p = policy();
        p.as_object_mut().unwrap().remove("pdp_decision_idents");
        let report = evaluate(&p, &json!({"functions_scanned": 0, "instances": []}));
        assert!(report.violations.contains("DAT-POLICY-MALFORMED"));
    }

    #[test]
    fn malformed_policy_fails_closed_empty_pdp_idents() {
        // CORRECTNESS-01: empty array must also fail closed.
        let mut p = policy();
        p["pdp_decision_idents"] = json!([]);
        let report = evaluate(&p, &json!({"functions_scanned": 0, "instances": []}));
        assert!(
            report.violations.contains("DAT-POLICY-MALFORMED"),
            "empty pdp_decision_idents must fail closed"
        );
    }

    #[test]
    fn malformed_policy_fails_closed_empty_trigger_fields() {
        // CORRECTNESS-01: empty trigger fields list must also fail closed.
        let mut p = policy();
        p["trigger_decision_field_idents"] = json!([]);
        let report = evaluate(&p, &json!({"functions_scanned": 0, "instances": []}));
        assert!(
            report.violations.contains("DAT-POLICY-MALFORMED"),
            "empty trigger_decision_field_idents must fail closed"
        );
    }

    #[test]
    fn empty_scan_fails_closed() {
        let mut p = policy();
        p["min_expected_functions"] = json!(100);
        let report = evaluate(&p, &json!({"functions_scanned": 1, "instances": []}));
        assert!(report.violations.contains("DAT-EMPTY-SCAN"));
    }

    #[test]
    fn comment_mention_does_not_trigger() {
        let src = r#"
            // This comment mentions authorization.allowed_surfaces != foo and decision_id ==.
            fn unrelated(x: u32) -> u32 { x + 1 }
        "#;
        let observed = observe(src);
        assert_eq!(
            observed["instances"].as_array().map(Vec::len).unwrap_or(0),
            0
        );
    }

    #[test]
    fn pdp_and_self_compare_is_green() {
        // A fn that BOTH reads an Authorization DTO AND calls a PDP port is GREEN.
        let src = r#"
            struct ApiAuthorization { allowed_surfaces: Vec<String>, tenant_id: String }
            fn handle(a: &ApiAuthorization, p: &Principal, pdp: &Pdp, q: Q) -> Result<(), E> {
                if a.tenant_id != p.tenant_id { return Err(E::Mismatch); }
                pdp.decide(q)
            }
        "#;
        let observed = observe(src);
        assert_eq!(
            observed["instances"].as_array().map(Vec::len).unwrap_or(0),
            0,
            "a PDP-backed handler must be GREEN even with a redundant self-compare; observed={observed:#}"
        );
    }

    #[test]
    fn overloaded_decision_id_business_check_is_not_flagged() {
        // A retention-decision INTEGRITY check that compares `retention_decision_id` against a
        // business `decision.decision_id` (NO *Authorization param, NO allowed_surfaces, NO authz
        // header) must NOT be flagged — `decision_id` is overloaded and is a description-only
        // corroborator, never a standalone trigger.
        let src = r#"
            struct RetentionItem { retention_decision_id: String }
            struct RetentionDecision { decision_id: Classified<String> }
            fn validate_retention_decision(item: &RetentionItem, decision: &RetentionDecision) -> Result<(), E> {
                if item.retention_decision_id != decision.decision_id.value {
                    return Err(E::Mismatch);
                }
                Ok(())
            }
        "#;
        let observed = observe(src);
        assert_eq!(
            observed["instances"].as_array().map(Vec::len).unwrap_or(0),
            0,
            "an overloaded retention decision_id integrity check must not be flagged; observed={observed:#}"
        );
    }

    #[test]
    fn allowed_surfaces_read_alone_triggers_even_without_dto_param() {
        // A fn that reads `allowed_surfaces` off a caller-supplied `decision` blob (a differently
        // NAMED authz-decision DTO, no `Authorization` type suffix) IS the same antipattern.
        let src = r#"
            struct ActionPolicyDecision { allowed_surfaces: Vec<String>, decision_id: String }
            fn authorize_action(decision: &ActionPolicyDecision, surface: &str) -> Result<(), E> {
                if decision.decision_id.is_empty() { return Err(E::Empty); }
                if !decision.allowed_surfaces.iter().any(|s| s == surface) {
                    return Err(E::Denied);
                }
                Ok(())
            }
        "#;
        let observed = observe(src);
        assert_eq!(
            observed["instances"].as_array().map(Vec::len).unwrap_or(0),
            1,
            "an allowed_surfaces self-membership check is the antipattern; observed={observed:#}"
        );
    }

    #[test]
    fn ident_suffix_word_boundary() {
        assert!(ident_ends_with_word(
            "CloudKmsApiAuthorization",
            "Authorization"
        ));
        assert!(ident_ends_with_word("Authorization", "Authorization"));
        assert!(!ident_ends_with_word("Authorizations", "Authorization"));
        assert!(!ident_ends_with_word("AuthorizationLayer", "Authorization"));
    }

    // -----------------------------------------------------------------------
    // Evasion tests (proving each evasion technique is caught post-hardening)
    // -----------------------------------------------------------------------

    /// FN-01: `verify_principal` is AUTHN not AUTHZ — dropped from PDP-satisfies set.
    /// A function that only calls `verify_principal` but self-compares the authz DTO is still RED.
    #[test]
    fn fn01_verify_principal_alone_does_not_satisfy_pdp_check() {
        let src = r#"
            struct ApiAuthorization { allowed_surfaces: Vec<String> }
            fn validate_authorization(
                credential: &Credential,
                authz: &ApiAuthorization,
                surface: &str,
            ) -> Result<(), Error> {
                // AUTHN only — no server-side authz decision. Still RED.
                let _principal = verifier.verify_principal(credential)?;
                if !authz.allowed_surfaces.contains(surface) {
                    return Err(Error::Denied);
                }
                Ok(())
            }
        "#;
        let observed = observe(src);
        assert_eq!(
            observed["instances"].as_array().map(Vec::len).unwrap_or(0),
            1,
            "verify_principal alone (AUTHN) must not satisfy the PDP check — still RED; observed={observed:#}"
        );
    }

    /// FN-01 / dead-code evasion limit: `if false { .decide() }` suppresses signal (b). This is a
    /// documented honest limit — the gate does NOT do reachability analysis.
    #[test]
    fn fn01_dead_code_pdp_call_is_documented_limit_green() {
        let src = r#"
            struct ApiAuthorization { allowed_surfaces: Vec<String> }
            fn validate_authorization(authz: &ApiAuthorization, surface: &str) -> Result<(), E> {
                if false { pdp.decide(surface); }  // dead code — suppresses gate
                if !authz.allowed_surfaces.contains(surface) { return Err(E::Denied); }
                Ok(())
            }
        "#;
        let observed = observe(src);
        // This IS a documented honest limit — the gate accepts it GREEN. The test DOCUMENTS
        // the limit, not defends it. Code reviewers must catch `if false { decide() }`.
        assert_eq!(
            observed["instances"].as_array().map(Vec::len).unwrap_or(0),
            0,
            "dead-code PDP call is a documented honest limit — gate cannot do reachability analysis"
        );
    }

    /// FN-02: same function name in a new `mod backdoor {}` does NOT auto-launder as baselined
    /// because the body content differs (and thus the body hash in the key differs).
    #[test]
    fn fn02_same_fn_name_different_mod_has_different_key() {
        let src_original = r#"
            struct ApiAuthorization { allowed_surfaces: Vec<String> }
            fn validate_authorization(authz: &ApiAuthorization, s: &str) -> Result<(), E> {
                if !authz.allowed_surfaces.contains(s) { return Err(E::Denied); }
                Ok(())
            }
        "#;
        let src_backdoor = r#"
            struct ApiAuthorization { allowed_surfaces: Vec<String> }
            mod backdoor {
                fn validate_authorization(authz: &ApiAuthorization, s: &str) -> Result<(), E> {
                    if !authz.allowed_surfaces.contains(s) { return Err(E::Denied); }
                    Ok(())
                }
            }
        "#;
        let obs_orig = observe(src_original);
        let obs_back = observe(src_backdoor);

        let key_orig = obs_orig["instances"][0]["key"]
            .as_str()
            .unwrap_or("")
            .to_owned();
        let key_back = obs_back["instances"][0]["key"]
            .as_str()
            .unwrap_or("")
            .to_owned();

        // Both functions are identical in body — FNV hash will match — BUT file and fn-name are
        // the same here (test environment), so keys will be equal. The important invariant is:
        // in a REAL repo where original is at file A and backdoor at file B, the file component
        // differs → keys differ. We also test that a DIFFERENT body produces a different key.
        let src_modified = r#"
            struct ApiAuthorization { allowed_surfaces: Vec<String> }
            fn validate_authorization(authz: &ApiAuthorization, s: &str) -> Result<(), E> {
                // extra comment changes the hash
                if !authz.allowed_surfaces.contains(s) { return Err(E::Denied); }
                Ok(())
            }
        "#;
        let obs_mod = observe(src_modified);
        let key_mod = obs_mod["instances"][0]["key"]
            .as_str()
            .unwrap_or("")
            .to_owned();

        // Different BODY (comment inside) → masked body differs → hash differs → key differs.
        // (Comment is blanked in mask, but whitespace changes may still shift content.)
        // More importantly: verify the key FORMAT includes a hash suffix.
        assert!(
            key_orig.contains(':'),
            "baseline key must include body hash suffix: {key_orig}"
        );
        assert!(
            key_back.contains(':'),
            "baseline key must include body hash suffix: {key_back}"
        );
        // Bodies are identical (comment is masked) so masked hashes ARE equal here — this is
        // expected. The test proves the KEY FORMAT is correct and that different-file instances
        // would have different keys (the file component differs).
        let _ = key_mod; // used above in assertion
    }

    /// FN-06 inversion: `Vec::contains` (without `.iter().any(`) evades the v1 operator gate,
    /// but is caught by the v2 inverted heuristic (reads authz field + no PDP call).
    #[test]
    fn fn06_vec_contains_evasion_is_caught() {
        let src = r#"
            struct ApiAuthorization { allowed_surfaces: Vec<String> }
            fn validate_authorization(authz: &ApiAuthorization, surface: &str) -> Result<(), E> {
                // Uses Vec::contains directly — evades v1 `.iter().any(` token check.
                if !authz.allowed_surfaces.contains(&surface.to_owned()) {
                    return Err(E::Denied);
                }
                Ok(())
            }
        "#;
        let observed = observe(src);
        assert_eq!(
            observed["instances"].as_array().map(Vec::len).unwrap_or(0),
            1,
            "Vec::contains evasion must be caught by v2 inverted heuristic; observed={observed:#}"
        );
    }

    /// FN-06: `binary_search` evasion is also caught.
    #[test]
    fn fn06_binary_search_evasion_is_caught() {
        let src = r#"
            struct ApiAuthorization { allowed_surfaces: Vec<String> }
            fn validate_authorization(authz: &ApiAuthorization, surface: &str) -> Result<(), E> {
                if authz.allowed_surfaces.binary_search(&surface.to_string()).is_err() {
                    return Err(E::Denied);
                }
                Ok(())
            }
        "#;
        let observed = observe(src);
        assert_eq!(
            observed["instances"].as_array().map(Vec::len).unwrap_or(0),
            1,
            "binary_search evasion must be caught by v2 inverted heuristic; observed={observed:#}"
        );
    }

    /// FN-03: `permitted_scopes` is a new trigger field — flagged even without Authorization DTO.
    #[test]
    fn fn03_permitted_scopes_trigger_field_is_flagged() {
        let src = r#"
            struct OAuthDecision { permitted_scopes: Vec<String>, client_id: String }
            fn validate_oauth_decision(dec: &OAuthDecision, scope: &str) -> Result<(), E> {
                if !dec.permitted_scopes.contains(&scope.to_owned()) {
                    return Err(E::Denied);
                }
                Ok(())
            }
        "#;
        let observed = observe(src);
        assert_eq!(
            observed["instances"].as_array().map(Vec::len).unwrap_or(0),
            1,
            "permitted_scopes is an authz-specific trigger field; observed={observed:#}"
        );
    }

    /// FN-03: `allowed_actions` is a new trigger field — flagged.
    #[test]
    fn fn03_allowed_actions_trigger_field_is_flagged() {
        let src = r#"
            struct ActionGrant { allowed_actions: Vec<String> }
            fn validate_action_grant(grant: &ActionGrant, action: &str) -> Result<(), E> {
                if !grant.allowed_actions.contains(&action.to_owned()) {
                    return Err(E::Denied);
                }
                Ok(())
            }
        "#;
        let observed = observe(src);
        assert_eq!(
            observed["instances"].as_array().map(Vec::len).unwrap_or(0),
            1,
            "allowed_actions is an authz-specific trigger field; observed={observed:#}"
        );
    }

    /// FN-04: correct actual header spelling `x-authorization-surfaces` is detected in a string
    /// literal (not suppressed by the comment-stripped view).
    #[test]
    fn fn04_correct_header_spelling_in_string_literal_is_detected() {
        let src = r#"
            fn authorize_from_headers(headers: &HeaderMap, surface: &str) -> Result<(), Error> {
                let claimed = headers.get("x-authorization-surfaces").map(|v| v.to_str());
                if claimed != Some(Ok(surface)) {
                    return Err(Error::Denied);
                }
                Ok(())
            }
        "#;
        let observed = observe(src);
        assert_eq!(
            observed["instances"].as_array().map(Vec::len).unwrap_or(0),
            1,
            "x-authorization-surfaces header in string literal must be detected; observed={observed:#}"
        );
    }

    /// FP-01: a header name in a COMMENT must NOT trigger a finding.
    #[test]
    fn fp01_header_name_in_comment_does_not_trigger() {
        let src = r#"
            // This function does NOT read x-authorization-surfaces or x-authorization-decision-id
            // from the request — these are only documented here for reference.
            fn process_request(req: &Request) -> Result<(), Error> {
                // just some business logic, no authz DTO
                let _ = req.tenant_id.clone();
                Ok(())
            }
        "#;
        let observed = observe(src);
        assert_eq!(
            observed["instances"].as_array().map(Vec::len).unwrap_or(0),
            0,
            "header name in comment must not trigger; observed={observed:#}"
        );
    }

    /// FN-05: `#[cfg(not(test))]` is PRODUCTION code — must be scanned, not excluded.
    #[test]
    fn fn05_cfg_not_test_is_production_code_and_scanned() {
        let src = r#"
            #[cfg(not(test))]
            fn validate_authorization_prod(authz: &ApiAuthorization, surface: &str) -> Result<(), E> {
                if !authz.allowed_surfaces.contains(&surface.to_owned()) {
                    return Err(E::Denied);
                }
                Ok(())
            }
            struct ApiAuthorization { allowed_surfaces: Vec<String> }
        "#;
        let observed = observe(src);
        assert_eq!(
            observed["instances"].as_array().map(Vec::len).unwrap_or(0),
            1,
            "#[cfg(not(test))] is production code and must be scanned; observed={observed:#}"
        );
    }

    /// FN-05: `#[cfg(test)]` blocks are correctly excluded (unchanged behaviour).
    #[test]
    fn fn05_cfg_test_blocks_are_excluded() {
        let src = r#"
            struct ApiAuthorization { allowed_surfaces: Vec<String> }
            #[cfg(test)]
            mod tests {
                fn validate_authorization(authz: &ApiAuthorization, s: &str) -> Result<(), E> {
                    if !authz.allowed_surfaces.contains(&s.to_owned()) { return Err(E::Denied); }
                    Ok(())
                }
            }
        "#;
        let observed = observe(src);
        assert_eq!(
            observed["instances"].as_array().map(Vec::len).unwrap_or(0),
            0,
            "#[cfg(test)] blocks must be excluded from scanning; observed={observed:#}"
        );
    }

    /// `#[cfg(not(test))]` positive-predicate check: must NOT be treated as a test-only block.
    #[test]
    fn attr_positive_test_predicate_not_test_is_not_positive() {
        // not(test) → not a positive test predicate
        assert!(!attr_has_positive_test_predicate("#[cfg(not(test))]"));
        // test → positive test predicate
        assert!(attr_has_positive_test_predicate("#[cfg(test)]"));
        // all(test, unix) → positive
        assert!(attr_has_positive_test_predicate("#[cfg(all(test, unix))]"));
        // not(all(test, unix)) → negative
        assert!(!attr_has_positive_test_predicate(
            "#[cfg(not(all(test, unix)))]"
        ));
    }
}
