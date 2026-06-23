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
//! VerifiedPrincipal` (an UNFORGEABLE credential becomes a verified principal) PLUS a
//! `PublishAuthorizer::ensure_authorized(principal, resource)` / a PDP `decide(...)` port, all
//! fail-closed (any refusal = deny). A function that calls such a PDP decision port in its own body
//! is NEVER flagged — the caller's claim is not trusted; the server decides.
//!
//! ## Detection heuristic (robust + low-false-positive; honest limits)
//! A function is a DTO-AUTHZ-TRUST instance iff ALL THREE hold over its CODE-ONLY body (comments and
//! string/char literals elided via [`mask_non_code`], so a doc-comment mention never triggers):
//!   (a) it READS A CALLER-SUPPLIED AUTHORIZATION-DECISION FIELD: it takes a parameter whose type
//!       name ends with the policy `authorization_dto_type_suffix` (default `Authorization`) — the
//!       forged blob — OR its body reads any policy `decision_field_idents` member
//!       (`allowed_surfaces` / `decision_id`) off a binding, OR it reads any policy
//!       `authorization_header_idents` (`x-authorization-decision`, ...);
//!   (b) the "check" is ONLY SELF-COMPARISON / EQUALITY / MEMBERSHIP against the request: the body
//!       contains an equality/inequality comparison (`==` / `!=`) on, or an iterator membership probe
//!       (`.iter().any(` / `.contains(`) of, the authorization fields — i.e. the body decides
//!       authorization by string-matching the request against itself;
//!   (c) the body makes NO PDP / authorizer DECISION-PORT call: NONE of the whole-token call-shaped
//!       policy `pdp_decision_idents` (`.decide(`, `ensure_authorized(`, `verify_principal(`,
//!       `.authorize(` as a port call, `check_authz(`, `ensure_authz(`) appears in the code-only body.
//!
//! Conservative in the SAFE direction for (c): a PDP-port call recognized as a whole-token CALL in
//! the code-only body marks the function GREEN (the gate never invents a false finding for a function
//! that genuinely delegates to a PDP). The residual risk traded away — a PDP function CALLED on a
//! never-taken branch — is acceptable; this gate stops the forged-blob class, not a full call-graph
//! reachability proof.
//!
//! Honest LIMITS (documented, not hidden):
//!   - It recognizes the corpus's dominant shape: a synchronous `fn validate_authorization(.., authz:
//!     &*Authorization, ..)` that self-compares. A handler that hides the same self-comparison behind
//!     an opaque helper whose name + signature give NO authorization-DTO signal is outside the
//!     envelope (human review + the authz-coverage gate own that). This is the deliberate boundary
//!     that keeps the baseline meaningful and the false-positive rate near zero.
//!   - (a) is the load-bearing precision lever: requiring an *authorization-DTO* parameter (not just
//!     any `tenant_id` comparison) is what stops the gate flagging benign tenant/path binding
//!     validators (`validate_tenant_binding`, `validate_path_body_binding`) that legitimately compare
//!     request-derived identity fields but assert NO authorization verdict.
//!   - A function that BOTH self-compares an authorization DTO AND calls a PDP port is GREEN: the PDP
//!     call is the real decision; the self-comparison is a redundant precondition, not the verdict.
//!
//! ## Born-blocking with a FROZEN, SHRINK-ONLY baseline
//! The ~30 pre-existing instances are enumerated and FROZEN as known debt
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
//!   decision (reads an authorization-DTO field + self-compares it + no PDP decision-port call), and
//!   its key is not in the frozen baseline.
//! - `DAT-STALE-BASELINE` — a frozen-baseline key matches no live instance (shrink-only self-clean).
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
pub const REMEDIATION_DOCTRINE: &str =
    "iam/ports/policy-cedar-api/src/authz.rs (PrincipalVerifier::verify_principal on an unforgeable \
     credential + PublishAuthorizer::ensure_authorized / PDP decide() port, fail-closed). Derive the \
     principal from a verified mTLS/SVID/bearer credential, call the cloud-iam Cedar PDP server-side \
     to decide(principal, action, resource), and fail closed — do NOT trust the caller-supplied \
     authorization DTO/header as the verdict.";

/// The blocking + structural violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 5] = [
    "DAT-CALLER-SUPPLIED-AUTHZ-TRUST",
    "DAT-STALE-BASELINE",
    "DAT-EMPTY-SCAN",
    "DAT-POLICY-GATE-ID-MISMATCH",
    "DAT-POLICY-MALFORMED",
];

/// The per-instance finding code whose keys are the BASELINE vocabulary. The policy-level codes
/// (`DAT-EMPTY-SCAN`, `DAT-POLICY-*`, `DAT-STALE-BASELINE`) are NOT baseline keys.
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
/// function that READS a caller-supplied authorization-decision field, SELF-COMPARES it, and makes
/// NO PDP decision-port call. Each instance is `{ "file", "fn", "line", "signal" }`.
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
        instance.get("file").and_then(Value::as_str).unwrap_or("").to_owned(),
        instance.get("fn").and_then(Value::as_str).unwrap_or("").to_owned(),
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
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
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
    /// The AUTHZ-SPECIFIC decision-field idents (default `["allowed_surfaces"]`) whose read off a
    /// binding is, on its own, sufficient signal-(a) evidence of caller-supplied-authz trust. These
    /// must be fields with NO benign business meaning — `allowed_surfaces` is an authorization
    /// allow-list and appears nowhere else; an overloaded field like `decision_id` (also a retention
    /// / policy business key) is NOT a trigger (it stays a description-only corroborator) so the gate
    /// does not false-positive on retention/policy `decision_id` integrity checks.
    trigger_decision_field_idents: Vec<String>,
    /// The decision-field idents read off the authorization blob, used ONLY to enrich the finding
    /// SIGNAL description (`allowed_surfaces`, `decision_id`) — NOT a standalone trigger.
    decision_field_idents: Vec<String>,
    /// The authorization-header idents (`x-authorization-decision`, ...) — a header-trust shape.
    authorization_header_idents: Vec<String>,
    /// The PDP / authorizer decision-port idents whose whole-token CALL in the body marks GREEN.
    pdp_decision_idents: Vec<String>,
    /// Equality/membership operator tokens that signal a self-comparison "check".
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
                    vec!["allowed_surfaces".to_owned()]
                } else {
                    v
                }
            },
            decision_field_idents: string_list(policy, "decision_field_idents"),
            authorization_header_idents: string_list(policy, "authorization_header_idents"),
            pdp_decision_idents: string_list(policy, "pdp_decision_idents"),
            self_compare_tokens: {
                let v = string_list(policy, "self_compare_tokens");
                if v.is_empty() {
                    vec![
                        "==".to_owned(),
                        "!=".to_owned(),
                        ".iter().any(".to_owned(),
                        ".contains(".to_owned(),
                    ]
                } else {
                    v
                }
            },
        }
    }
}

/// Extract every DTO-authz-trust instance from one file's source text, plus the count of functions
/// scanned (for the empty-scan floor). Structure is searched against a length-preserving
/// [`mask_non_code`] view (comments + string/char literal CONTENT blanked), so a doc-comment or
/// string mention never triggers a finding; the original `text` supplies offsets for line numbers.
fn extract_instances(file: &str, text: &str, cfg: &SignatureConfig) -> (Vec<Value>, u64) {
    let masked = mask_non_code(text);
    let masked = masked.as_str();
    let test_spans = cfg_test_spans(masked);
    let fns = fn_decls(masked);

    let mut out = Vec::new();
    let mut scanned: u64 = 0;
    for decl in &fns {
        // Skip test-fixture functions.
        if test_spans.iter().any(|(lo, hi)| decl.body_open >= *lo && decl.body_open < *hi) {
            continue;
        }
        scanned += 1;
        let sig = &masked[decl.sig_start..decl.body_open];
        let body = &masked[decl.body_open..decl.body_end];
        // The ORIGINAL-text body at the SAME offsets (the mask is length-preserving, so the bounds
        // align). Header NAMES are string literals, whose CONTENT the mask blanks; so header-trust
        // detection reads the original text. Comment mentions are still excluded because the masked
        // `body` must independently show a self-compare token in CODE (step b) for a finding.
        let body_text = &text[decl.body_open..decl.body_end.min(text.len())];

        // (a) reads a caller-supplied authorization-decision field. The TRIGGERS are: an
        // `*Authorization`-typed parameter (the forged blob), an AUTHZ-SPECIFIC decision-field read
        // (`allowed_surfaces` — no benign business meaning), or an `x-authorization-*` header. A
        // generic / overloaded decision-field read (`decision_id`) is NOT a standalone trigger (it
        // also names retention/policy business keys); it only enriches the finding description.
        let has_dto_param = signature_has_authz_dto_param(sig, &cfg.dto_type_suffixes);
        let reads_trigger_field = cfg
            .trigger_decision_field_idents
            .iter()
            .any(|f| body_reads_field(body, f));
        let reads_authz_header = cfg
            .authorization_header_idents
            .iter()
            .any(|h| body_text.contains(h.as_str()));
        let signal_a = has_dto_param || reads_trigger_field || reads_authz_header;
        if !signal_a {
            continue;
        }
        // Description-only corroborator (never a standalone trigger).
        let reads_decision_field = cfg
            .decision_field_idents
            .iter()
            .any(|f| body_reads_field(body, f));

        // (b) the only check is self-comparison / equality / membership against the request. We
        // require BOTH an authorization-field read AND a self-compare operator so a function that
        // merely PASSES an authorization param through (no comparison) is not flagged.
        let self_compares = cfg
            .self_compare_tokens
            .iter()
            .any(|t| body.contains(t.as_str()));
        if !self_compares {
            continue;
        }

        // (c) no PDP / authorizer decision-port call in the body (whole-token, call-shaped).
        let calls_pdp = cfg
            .pdp_decision_idents
            .iter()
            .any(|ident| body_calls_pdp(body, ident));
        if calls_pdp {
            continue;
        }

        let signal = build_signal(
            has_dto_param,
            reads_decision_field,
            reads_authz_header,
            &cfg.dto_type_suffixes,
            sig,
        );
        out.push(json!({
            "file": file,
            "fn": decl.name,
            "line": line_of(text, decl.sig_start) as u64,
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
        parts.push("reads an authorization-decision field (e.g. allowed_surfaces/decision_id)".to_owned());
    }
    if reads_authz_header {
        parts.push("reads an x-authorization-* header".to_owned());
    }
    parts.push("self-compares it (equality/membership) and makes NO PDP decision-port call".to_owned());
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
            let tail = path.trim_end_matches(':').rsplit("::").next().unwrap_or(path);
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
/// ident like `.decide(` is matched as a literal call-shaped token; a bare-ident like `verify_principal`
/// is matched as a whole-ident immediately followed (after ws) by `(`. This is the GREEN signal.
fn body_calls_pdp(body: &str, ident: &str) -> bool {
    // Pre-formed call tokens (those containing `(` or starting with `.`) are matched literally.
    if ident.ends_with('(') {
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

/// Byte spans of `#[cfg(test)]`-gated items in masked `text`. For each `#[cfg(...)]` attribute
/// carrying a `test` predicate token, the gated item runs from the attribute to the matching close
/// brace of the first `{` after it. Functions within these spans are test fixtures, not production.
fn cfg_test_spans(text: &str) -> Vec<(usize, usize)> {
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut from = 0usize;
    while let Some(rel) = text[from..].find("#[cfg(") {
        let at = from + rel;
        let attr_end = text[at..].find(']').map(|i| at + i + 1).unwrap_or(text.len());
        let attr = &text[at..attr_end];
        if attr_contains_test_token(attr)
            && let Some(body) = brace_body(text, attr_end)
        {
            let body_start = body.as_ptr() as usize - text.as_ptr() as usize;
            spans.push((at, body_start + body.len()));
        }
        from = attr_end;
    }
    spans
}

/// Whether a `#[cfg(...)]` attribute string carries `test` as a config predicate token.
fn attr_contains_test_token(attr: &str) -> bool {
    let bytes = attr.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = attr[from..].find("test") {
        let at = from + rel;
        let before_ok = at == 0 || !is_ident_byte(bytes[at - 1]);
        let after = at + 4;
        let after_ok = after >= bytes.len() || !is_ident_byte(bytes[after]);
        if before_ok && after_ok {
            return true;
        }
        from = at + 4;
    }
    false
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
    text[..at.min(text.len())].bytes().filter(|&b| b == b'\n').count() + 1
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

/// The stable SIGNATURE key for an instance finding: `<file>#<fn>`. Independent of line numbers, so
/// an unrelated edit that shifts the function's line does NOT spuriously re-RED a baselined instance.
fn instance_key(file: &str, fn_name: &str) -> String {
    format!("{file}#{fn_name}")
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

    // Fail CLOSED on a structurally invalid policy: the PDP-decision-ident list is the gate's GREEN
    // recognition vocabulary; a MISSING (null/non-array) one signals a corrupt policy — fail closed.
    if policy.get("pdp_decision_idents").and_then(Value::as_array).is_none() {
        findings.insert(Finding::new(
            "DAT-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `pdp_decision_idents` must be an array of recognized PDP/authorizer decision-port ident strings; correct the policy before the gate can evaluate",
        ));
        return findings;
    }
    if policy.get("scan_roots").and_then(Value::as_array).is_none() {
        findings.insert(Finding::new(
            "DAT-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `scan_roots` must be an array of repo-relative scan-root strings; correct the policy before the gate can evaluate",
        ));
        return findings;
    }

    let frozen_baseline: BTreeSet<String> =
        string_list(policy, "frozen_dto_authz_trust_instances").into_iter().collect();

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

        let key = instance_key(file, fn_name);
        live_keys.insert(key.clone());

        if frozen_baseline.contains(&key) {
            continue;
        }
        findings.insert(Finding::new(
            "DAT-CALLER-SUPPLIED-AUTHZ-TRUST",
            &key,
            format!(
                "NEW caller-supplied-authorization-trust instance: function `{fn_name}` at {file}:{line} {signal}. This is default-ALLOW-on-forged-input — any caller forges the authorization blob and authorizes itself, because the verdict is decided by string-comparing the request against itself, never by a server-side PDP. {REMEDIATION_DOCTRINE}"
            ),
        ));
    }

    for key in &frozen_baseline {
        if !live_keys.contains(key) {
            findings.insert(Finding::new(
                "DAT-STALE-BASELINE",
                key,
                format!(
                    "frozen-baseline instance key `{key}` matched no live caller-supplied-authz-trust finding (the instance was fixed, removed, or moved). Remove it from `frozen_dto_authz_trust_instances` in the policy — the baseline is shrink-only."
                ),
            ));
        }
    }

    findings
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
    let prior: BTreeSet<String> =
        string_list(policy, "frozen_dto_authz_trust_instances").into_iter().collect();
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
    let mut out = String::from("dto-authz-trust gate failed (caller-supplied-authz-trust class):\n");
    for finding in findings {
        out.push_str(&format!("    - {} {}\n        {}\n", finding.code, finding.key, finding.detail));
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
            "trigger_decision_field_idents": ["allowed_surfaces"],
            "decision_field_idents": ["allowed_surfaces", "decision_id"],
            "authorization_header_idents": ["x-authorization-decision", "x-authorization-allowed-surfaces"],
            "pdp_decision_idents": [".decide(", "ensure_authorized", "verify_principal", "check_authz", "ensure_authz"],
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

    // RED via header trust: reads an x-authorization-* header and self-compares.
    const RED_HEADER_TRUST: &str = r#"
        fn authorize_from_headers(headers: &HeaderMap, surface: &str) -> Result<(), Error> {
            let claimed = headers.get("x-authorization-decision").map(|v| v.to_str());
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
        assert!(report.violations.contains("DAT-CALLER-SUPPLIED-AUTHZ-TRUST"));
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
        assert_eq!(observed["instances"].as_array().map(Vec::len).unwrap_or(0), 0);
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

    #[test]
    fn frozen_baseline_tolerates_known_instance() {
        let observed = observe(RED_DTO_SELF_COMPARE);
        let mut p = policy();
        p["frozen_dto_authz_trust_instances"] =
            json!(["src/lib.rs#validate_authorization"]);
        let report = evaluate(&p, &observed);
        assert_eq!(report.verdict, Verdict::Green, "baselined instance must be tolerated");
    }

    #[test]
    fn stale_baseline_self_cleans() {
        let observed = observe(GREEN_PDP); // no live instances
        let mut p = policy();
        p["frozen_dto_authz_trust_instances"] = json!(["src/lib.rs#gone_away"]);
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
    fn malformed_policy_fails_closed() {
        let mut p = policy();
        p.as_object_mut().unwrap().remove("pdp_decision_idents");
        let report = evaluate(&p, &json!({"functions_scanned": 0, "instances": []}));
        assert!(report.violations.contains("DAT-POLICY-MALFORMED"));
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
        assert_eq!(observed["instances"].as_array().map(Vec::len).unwrap_or(0), 0);
    }

    #[test]
    fn pdp_and_self_compare_is_green() {
        // A fn that BOTH self-compares an Authorization DTO AND calls a PDP port is GREEN.
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
        assert!(ident_ends_with_word("CloudKmsApiAuthorization", "Authorization"));
        assert!(ident_ends_with_word("Authorization", "Authorization"));
        assert!(!ident_ends_with_word("Authorizations", "Authorization"));
        assert!(!ident_ends_with_word("AuthorizationLayer", "Authorization"));
    }
}
