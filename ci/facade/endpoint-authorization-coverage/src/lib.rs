//! # cloud-ci-authz-coverage (issue #770; AUTH-005 pipeline-as-product backstop)
//!
//! PR #768 shipped a multi-tenant REST control plane (`tenancy/facade/tenant-lifecycle-app`)
//! with ZERO authn/authz — any network caller could `DELETE /v1/tenants/{id}` and irreversibly
//! retire any tenant — and it passed all cloud-ci gates green. The independent reviewer caught
//! it; the pipeline did not. Founder doctrine (friction = process failure; productize the engine
//! so the anti-pattern is IMPOSSIBLE to ship) demands the pipeline itself catch this class. This
//! gate is that backstop.
//!
//! The repo already carries the correct fail-closed doctrine in
//! `intelligence/adapters/rest/src/lib.rs` (`admin_tenant_allowed` + a PDP `gate.decide` +
//! `constant_time_eq`) and, post-fix, in `tenancy/facade/tenant-lifecycle-app/src/lib.rs`
//! (`authenticate_caller` + `authorize()` per route). This gate asserts that EVERY HTTP
//! control-plane surface follows it.
//!
//! ## What is a control-plane surface
//! Surface DISCOVERY is anchored on the ROUTE-INTRODUCTION call set — `.route(` / `.route_service(` —
//! NOT on a `Router::new()` constructor: a route cannot exist without one of those calls. Every such
//! call in a file is found and attributed to its enclosing function scope, so a route is discovered
//! regardless of how (or whether) a `Router::new()` / `Router::default()` / `Router::<S>::new()` / a
//! `Router` PARAMETER / an aliased binding / a builder-returned `Router` produced it, and regardless
//! of whether it is declared before or AFTER `.with_state(...)` (PR #780 second-pass BLOCKER-1/2/3).
//! Two route grammars are classified: axum `.route(path, METHOD(handler))` and the owned
//! `oya-http-router-kernel` `.route(HttpMethod::X, path, handler)`. A surface is a CONTROL PLANE when
//! any route is either
//! - a MUTATING method (`post`/`put`/`patch`/`delete`/`any`, or `HttpMethod::POST/PUT/PATCH/DELETE`)
//!   on a non-exempt path, or
//! - a per-resource path param (`{id}`/`{tenant_id}`/...) on a mutating method, or
//! - any route the engine cannot fully classify (unresolved path, unclassified method-router,
//!   structurally-unclassifiable call, or an unresolved `.merge`/`.nest` sub-router — all fail-closed).
//!
//! `/healthz`-style unauthenticated reads are exempt via an explicit DATA allowlist
//! (`exempt_path_substrings` in `authz-coverage-policy.json`) — never code.
//!
//! ## Required authz coverage
//! A control-plane surface is COVERED iff
//! - its builder chain carries a recognized router-level auth `.layer(...)` (a verified-principal
//!   extractor / auth middleware named in policy `auth_layer_idents`), OR
//! - every MUTATING handler bound in the chain invokes a recognized authz decision in its function
//!   body — including the repo-owned `handler_to_sync(HandlerStruct)` typed-handler bridge, which is
//!   resolved to the `impl Handler for HandlerStruct { fn call(...) { ... } }` body — an
//!   `admin_tenant_allowed`-style guard, the tenancy `authorize(...)` pattern, a PDP `decide(...)`
//!   port call, or a bearer/peer authentication guard, all named in policy `authz_guard_idents`.
//!
//! A mutating handler that derives no caller identity → the surface is UNAUTHENTICATED.
//!
//! ## Conservative in the SAFE direction
//! Handler-body authz detection is a token over-approximation: a recognized guard ident invoked as a
//! whole-token CALL (`authorize(...)`, `.decide(`, `guard!`) in the handler's CODE-ONLY `fn` body
//! (comments + string/char literals elided) counts as covered. It never invents a false
//! UNAUTHENTICATED finding for a handler that genuinely calls a guard. The whole-token + call-shape
//! match (PR #780 second-pass BLOCKER-5) closes the substring false-cover where
//! `unauthorized_response()` satisfied `authorize`; the code-only view (first-pass B3) closes the
//! comment/string false-cover. The residual risk it trades away — a guard *function* CALLED in code
//! but on a never-taken branch — is acceptable: this gate stops the ZERO-authz class plus the
//! reproduced idiomatic bypasses, not the full call-graph reachability proof (the audit-coverage gate
//! AC-W-13 and human review own that).
//!
//! ## Ratchet vs a frozen baseline of currently-known surfaces
//! Several pre-existing surfaces are unauthenticated today (k8s control-plane-host, tenant-quota,
//! cluster-lifecycle, the iam policy-cedar publish port, the ci controllers/webhooks). Blocking
//! them now is out of scope for this gate; that is each owner's remediation. So the gate ships with
//! a FROZEN baseline of today's known-unauthenticated surface keys
//! (`frozen_unauthenticated_surfaces` in policy DATA): a finding whose key is in the baseline is
//! ACCEPTED (no block); a finding whose key is NOT in the baseline is a NEW unauthenticated control
//! plane → RED. This mirrors the capability-membership / tier-acyclicity gate posture (born-green,
//! enforce-no-regression). The baseline is shrink-only by construction — a removed/fixed surface
//! drops its key, and a stale baseline key (no live finding) self-cleans via `AC-STALE-BASELINE`.
//!
//! ## Born pack-shaped
//! The crate is a NEUTRAL engine. All repo-specifics — the recognized authz-guard idents, the
//! auth-layer idents, the exempt read paths, the scan roots/excludes, the frozen baseline, the
//! liveness floor — are DATA in `authz-coverage-policy.json`. A different repo adopts the gate by
//! repointing the policy.
//!
//! ## Kernel contract
//! - [`collect_surfaces`] `(root, policy) -> observed` is the ONLY I/O: it walks the policy scan
//!   roots, reads each `.rs` file, and extracts every router surface with its routes + per-handler
//!   authz signal. Read-only; writes no temp files.
//! - [`evaluate_keyed`] `(policy, observed) -> BTreeSet<Finding>` is PURE and unit-testable without
//!   a filesystem; it applies the exempt/baseline DATA to the observed surfaces.
//! - [`evaluate`] is the bare-code projection of [`evaluate_keyed`], the single source of the verdict.
//!
//! ## Fail-closed spine (the TRUE post-fix guarantee — honest envelope)
//! A textual matcher need not be perfect IF its failure mode is fail-closed. Surface DISCOVERY itself
//! is now fail-closed: it anchors on the route-introduction call set (not a constructor), so it does
//! not miss a router shape it never imagined. Any route-introduction the engine cannot FULLY classify
//! (method, path, AND authz-coverage) produces a FINDING (RED), never a silent skip:
//! - a `.route(` path it cannot resolve to a concrete string (a `const`/`static` it cannot
//!   substitute, a `&format!(...)`/non-literal expr) → `AC-UNRESOLVED-ROUTE-PATH`.
//! - a `.route(`'s method-router it cannot classify (an unresolved `let`-bound var, an unrecognized
//!   call/macro) → `AC-UNCLASSIFIED-METHOD`, treated as potentially-mutating.
//! - a `.route(`/`.route_service(` whose whole call shape matches NEITHER the axum
//!   `(path, METHOD(handler))` nor the owned-kernel `(HttpMethod::X, path, handler)` grammar (a
//!   macro-shaped or truncated/unbounded call) → `AC-UNCLASSIFIED-SURFACE`.
//! - a `.merge(X)` / `.nest(path, X)` / `.nest_service(...)` whose sub-router X this file did not
//!   scan-and-clear (a cross-crate/module call, a function-returned router, a macro) →
//!   `AC-UNRESOLVED-SUBROUTER` — merged/nested content is NOT assumed covered.
//!
//! The honest envelope (NOT an absolute "impossible to ship" claim): the engine recognizes the two
//! HTTP route grammars the corpus uses and resolves file-local const/binding substitutions; a
//! `.route(` whose call shape is neither grammar AND whose arg-shape is indistinguishable from a
//! same-named DOMAIN/dispatch method (`usecase.route(input)`, an enum `.route()`) is treated as a
//! NON-route (dropped) rather than flagged — this is the deliberate boundary that keeps the baseline
//! meaningful, and it cannot hide an HTTP route (an HTTP route always carries a string-ish path or an
//! `HttpMethod::` verb). Within that envelope, the AUTH-005 zero-authz class and the reproduced
//! idiomatic bypasses (const path, on(MethodFilter), variable method router, comment/substring guard,
//! Router::default()/param/post-with_state route, owned-kernel POST) are mechanically blocked.
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `AC-UNAUTHENTICATED-CONTROL-PLANE` — a control-plane surface (mutating method and/or
//!   per-resource path) has ≥1 mutating handler that derives no caller identity, and its key is not
//!   in the frozen baseline (a NEW unauthenticated control plane).
//! - `AC-UNRESOLVED-ROUTE-PATH` — a `.route(` path argument could not be resolved to a concrete
//!   string (fail-closed: an unknown-authz control-plane surface), and its key is not baselined.
//! - `AC-UNCLASSIFIED-METHOD` — a `.route(`'s method-router could not be classified (fail-closed:
//!   treated as a potentially-mutating control plane requiring a guard), and it is uncovered +
//!   not baselined.
//! - `AC-UNCLASSIFIED-SURFACE` — a `.route(`/`.route_service(` whose whole call shape matched neither
//!   route grammar (macro-shaped / truncated) (fail-closed: potentially mutating), not baselined.
//! - `AC-UNRESOLVED-SUBROUTER` — a `.merge`/`.nest`/`.nest_service` sub-router the engine could not
//!   resolve to a scanned-and-cleared router (fail-closed: composition coverage unknown), not baselined.
//! - `AC-STALE-BASELINE` — a frozen-baseline key matches no live finding (shrink-only self-clean).
//! - `AC-EMPTY-SCAN` — fewer router surfaces than `min_expected_surfaces` (catches a broken
//!   glob / CWD / collect that would otherwise be a false-green).
//! - `AC-POLICY-GATE-ID-MISMATCH` — the policy `gate_id` is not [`GATE_ID`] (fail-closed).
//! - `AC-POLICY-MALFORMED` — the policy is structurally invalid (fail-closed).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::{Value, json};

/// The gate id, matching the buck2 target + the policy `gate_id`.
pub const GATE_ID: &str = "cloud-ci-authz-coverage";

/// The remediation doctrine pointer every finding carries.
pub const REMEDIATION_DOCTRINE: &str = "intelligence/adapters/rest/src/lib.rs (admin_tenant_allowed + PDP gate.decide + constant_time_eq) \
     and tenancy/facade/tenant-lifecycle-app/src/lib.rs (authenticate_caller + authorize() per route)";

/// The blocking + structural violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 9] = [
    "AC-UNAUTHENTICATED-CONTROL-PLANE",
    "AC-UNRESOLVED-ROUTE-PATH",
    "AC-UNCLASSIFIED-METHOD",
    "AC-UNCLASSIFIED-SURFACE",
    "AC-UNRESOLVED-SUBROUTER",
    "AC-STALE-BASELINE",
    "AC-EMPTY-SCAN",
    "AC-POLICY-GATE-ID-MISMATCH",
    "AC-POLICY-MALFORMED",
];

/// The sentinel key for codes that are policy-level rather than per-surface.
const POLICY_KEY: &str = "<policy>";

// ---------------------------------------------------------------------------
// Collection (the only I/O; read-only)
// ---------------------------------------------------------------------------

/// Errors collecting the observed surface graph. Returned instead of panicking so the caller
/// (CI / a controller) decides how to surface them — an unreadable scan root is a fail-closed
/// error, never a silently skipped subtree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    Io(String),
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::Io(message) => write!(f, "authz-coverage io: {message}"),
        }
    }
}

impl std::error::Error for CollectError {}

/// Collect the router surfaces under the policy scan roots.
///
/// Walks each `scan_roots` directory, reads every `.rs` file not under an `excluded_dir_names`
/// directory, and extracts every `Router::new()....route(...)` surface. Emits
/// `{ "surfaces_found": <usize>, "surfaces": [ <surface>, .. ] }` where each surface is
/// `{ "file": <repo-rel path>, "router_line": <1-based>, "routes": [ { "path", "method",
///   "handler" } ], "has_auth_layer": <bool>, "handler_authz": { <handler>: <bool> } }`.
pub fn collect_surfaces(root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let scan_roots = string_list(policy, "scan_roots");
    let excluded_dirs: BTreeSet<String> = string_list(policy, "excluded_dir_names")
        .into_iter()
        .collect();
    let auth_layer_idents = string_list(policy, "auth_layer_idents");
    let authz_guard_idents = string_list(policy, "authz_guard_idents");

    let mut rs_files: Vec<String> = Vec::new();
    for scan_root in &scan_roots {
        collect_rs_files(root, &root.join(scan_root), &excluded_dirs, &mut rs_files)?;
    }
    rs_files.sort();
    rs_files.dedup();

    let mut surfaces: Vec<Value> = Vec::new();
    for rel_path in &rs_files {
        let text = match fs::read_to_string(root.join(rel_path)) {
            Ok(text) => text,
            Err(e) => return Err(CollectError::Io(format!("read {rel_path}: {e}"))),
        };
        for surface in extract_surfaces(rel_path, &text, &auth_layer_idents, &authz_guard_idents) {
            surfaces.push(surface);
        }
    }
    surfaces.sort_by_key(surface_sort_key);

    Ok(json!({
        "surfaces_found": surfaces.len(),
        "surfaces": surfaces,
    }))
}

fn surface_sort_key(surface: &Value) -> (String, u64) {
    (
        surface
            .get("file")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned(),
        surface
            .get("router_line")
            .and_then(Value::as_u64)
            .unwrap_or(0),
    )
}

/// Recursively collect repo-relative `.rs` file paths under `dir`, skipping any directory whose
/// name is in `excluded_dirs`. A missing scan root is fine (the gate is repo-portable). Symlinks
/// are followed by `read_dir`'s default metadata; we only recurse real directories.
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

/// How a `.route(...)` call's method-router argument classifies. The whole point of the gate is to
/// treat anything it cannot PROVE non-mutating as a potential write. So the classification is
/// fail-closed: a method-router shape the parser cannot recognize is `Unclassified`, which the
/// evaluator treats as a potentially-mutating control plane requiring authz.
#[derive(Debug, Clone, PartialEq, Eq)]
enum MethodClass {
    /// A recognized mutating HTTP method (post/put/patch/delete) — and `on`/`on_service`/`any` with
    /// a mutating-or-unknown method filter. The string is the recognized method/shape label.
    Mutating(String),
    /// A recognized non-mutating HTTP method (get/head/options/trace) — and `on`/`on_service` with
    /// an exclusively non-mutating filter. The string is the recognized method label.
    NonMutating(String),
    /// The method-router argument is bound to a variable, an unrecognized call, or otherwise cannot
    /// be classified. FAIL-CLOSED: treated as a potentially-mutating control plane needing authz.
    Unclassified(String),
}

impl MethodClass {
    fn is_unclassified(&self) -> bool {
        matches!(self, MethodClass::Unclassified(_))
    }
    /// The label string emitted for the route's `method` observation field.
    fn label(&self) -> &str {
        match self {
            MethodClass::Mutating(s)
            | MethodClass::NonMutating(s)
            | MethodClass::Unclassified(s) => s,
        }
    }
    /// The discriminant string emitted for the route's `method_class` observation field.
    fn discriminant(&self) -> &'static str {
        match self {
            MethodClass::Mutating(_) => "mutating",
            MethodClass::NonMutating(_) => "non-mutating",
            MethodClass::Unclassified(_) => "unclassified",
        }
    }
}

/// A `let NAME = METHOD(handler);` MethodRouter binding: its classified method + bound handler, so a
/// variable-bound route (B2) is resolved to both its mutating-ness AND its handler for the guard probe.
#[derive(Debug, Clone, PartialEq, Eq)]
struct MethodBinding {
    class: MethodClass,
    handler: String,
}

/// A single parsed route from one route-introduction call.
///
/// `path` is the RESOLVED concrete route string (a literal, or a `const`/`static` ident substituted
/// from the file's declarations) — `None` when the path argument cannot be resolved to a concrete
/// string (fail-closed: such a route is an unknown-authz control-plane surface). `path_raw` is the
/// raw argument text (the ident name or the literal) kept for the finding detail and stable keying.
/// `surface_unclassified` is set when the WHOLE `.route(` call matched neither known route-grammar
/// (axum `(path, METHOD(h))` nor owned-kernel `(HttpMethod::X, path, handler)`) — a structurally
/// unclassifiable / macro-shaped route call → fail-closed `AC-UNCLASSIFIED-SURFACE` (item 5).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Route {
    path: Option<String>,
    path_raw: String,
    method: MethodClass,
    handler: String,
    surface_unclassified: bool,
}

/// Build a length-preserving CODE-STRUCTURE mask of `text`: line/block comment bytes and string/char
/// literal CONTENT bytes are replaced with spaces, but the literal's delimiting quotes are KEPT and
/// the byte length + newline positions are preserved. This lets the structural finders (`Router::new()`,
/// `.route(`, method tokens, `const`/`let` keywords) `.find()` against the mask — so a `Router::new()`
/// mention in a doc comment or a `find("Router::new()")` string literal never registers as a surface —
/// while path/const VALUES are still read from the ORIGINAL `text` at the same offsets (the offsets
/// align because masking is length-preserving). This is the file-wide analogue of [`code_only`] and
/// closes the gate-scans-its-own-source / comment-mention false positives.
fn mask_non_code(text: &str) -> String {
    /// Blank a byte slice to spaces, preserving newlines (so line counting stays aligned).
    fn blank_into(out: &mut Vec<u8>, slice: &[u8]) {
        out.extend(slice.iter().map(|&b| if b == b'\n' { b'\n' } else { b' ' }));
    }
    let bytes = text.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            // MINOR-1: raw string literal `r"..."` / `r#"..."#` / `r##"..."##` (and byte-raw
            // `br"..."`). A raw string has no escapes and may contain `"` and `\` literally, so the
            // ordinary `skip_string` would desync the mask. Recognize the raw opener as a whole token
            // (the char before `r`/`br` must not be an ident byte, else it is an ident like `for`),
            // and blank the WHOLE span to spaces (preserving newlines): the raw delimiters are not
            // structure the finders match, so blanking them keeps the mask length-aligned and inert.
            b'r' | b'b' if raw_string_open(bytes, i).is_some() => {
                let (_content_start, end) = raw_string_open(bytes, i).unwrap_or((i, i + 1));
                blank_into(&mut out, &bytes[i..end]);
                i = end;
                continue;
            }
            b'"' => {
                // Keep the opening quote, blank the content, keep the closing quote.
                let end = skip_string(bytes, i); // offset just past closing quote
                out.push(b'"');
                if end > i + 1 {
                    // Content runs from i+1 to end-1 (end-1 is the closing quote).
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
    // The mask is byte-for-byte length-aligned with `text`; it is valid UTF-8 because we only ever
    // replaced bytes with ASCII space/newline or kept original bytes (multibyte sequences inside
    // strings are blanked byte-wise to spaces, which is safe — they were content, not structure).
    String::from_utf8(out).unwrap_or_else(|_| " ".repeat(text.len()))
}

/// One `.route(` / `.route_service(` candidate route-introduction call site: its byte offset, the
/// balanced argument-list slice (masked + text views), and whether its argument list was UNBOUNDED
/// (no balanced close paren — a truncated call the parser could not bound, fail-closed). A zero-arg
/// `.route()` is bounded with empty args and is dropped (not a route-introduction).
struct RouteCall<'a> {
    offset: usize,
    args_masked: &'a str,
    args_text: &'a str,
    unbounded: bool,
}

/// Extract every route-introduction surface from one file's source text.
///
/// BLOCKER-1/2/3 RE-ANCHOR: a "surface" is no longer a `Router::new()` builder chain — a route
/// cannot exist without a route-INTRODUCTION call (`.route(` / `.route_service(`), and that is what
/// the gate anchors on, regardless of how (or whether) a `Router::new()` / `Router::default()` /
/// `Router::<S>::new()` / a `Router` PARAMETER / an aliased binding / a builder-returned `Router`
/// produced the router. Every such call in the file is found and attributed to its ENCLOSING
/// function scope (so a `fn add(r: Router) -> Router { r.route(...) }` helper, a route declared on a
/// bound variable AFTER `.with_state(...)`, and a `Router::default().route(...)` are all discovered).
/// All route-introduction calls in one fn scope (plus the file's top-level scope, for module-level
/// route declarations) are grouped into ONE surface keyed by that scope.
///
/// The parser is line/char based (no Rust-source AST kernel exists yet) but robust to the corpus
/// shapes: multiline calls, turbofish handlers, the axum `(path, METHOD(handler))` grammar AND the
/// owned `oya-http-router-kernel` `(HttpMethod::X, path, handler)` grammar. Any `.route(` shape it
/// cannot classify into either grammar → an `AC-UNCLASSIFIED-SURFACE` route (fail-closed, item 5).
///
/// Structure is searched against a length-preserving [`mask_non_code`] view (so comment/string
/// mentions of `.route(` never register), while literal VALUES (paths, consts) are read from the
/// original `text` at the aligned offsets.
fn extract_surfaces(
    file: &str,
    text: &str,
    auth_layer_idents: &[String],
    authz_guard_idents: &[String],
) -> Vec<Value> {
    let masked = mask_non_code(text);
    let masked = masked.as_str();
    let test_spans = cfg_test_spans(masked);
    let str_consts = collect_str_consts(masked, text);
    let method_bindings = collect_method_bindings(masked);
    let method_consts = collect_method_consts(masked);
    // Enclosing-fn spans (name + body-brace span) so each route call attributes to its scope.
    let fn_spans = fn_body_spans(masked);

    // Collect every route-introduction call site (outside test spans), grouped by scope key. The
    // scope key is the innermost enclosing fn name (`<file>#<fn>`), or `<file>#<file-scope>` for a
    // module-level route declaration. Grouping keeps a surface's identity stable across edits.
    let mut groups: std::collections::BTreeMap<String, Vec<RouteCall>> =
        std::collections::BTreeMap::new();
    // Per-scope: earliest call offset (for router_line) and composition (merge/nest) findings.
    let mut group_first_offset: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();

    for marker in [".route_service(", ".route("] {
        let mut from = 0usize;
        while let Some(rel) = masked[from..].find(marker) {
            let at = from + rel;
            from = at + marker.len();
            // `.route(` is a strict prefix of `.route_service(`; skip a `.route(` that is actually
            // the head of a `.route_service(` (handled by the `.route_service(` pass).
            if marker == ".route(" && masked[at..].starts_with(".route_service(") {
                continue;
            }
            if test_spans.iter().any(|(lo, hi)| at >= *lo && at < *hi) {
                continue;
            }
            let open = at + marker.len(); // just past `(`
            let (args_masked, args_text, unbounded) = match balanced_paren_body(masked, open) {
                Some(am) => {
                    let off = am.as_ptr() as usize - masked.as_ptr() as usize;
                    (am, &text[off..off + am.len()], false)
                }
                // Unbounded call: a truncated `.route(` we cannot parse → still a surface, fail-closed
                // (classified UNCLASSIFIED-SURFACE downstream).
                None => ("", "", true),
            };
            // A bounded zero-arg `.route()` (empty args) is a same-named domain method, never an HTTP
            // route-introduction (an HTTP route needs at least a path) — drop it.
            if !unbounded && args_masked.trim().is_empty() {
                continue;
            }
            let scope = scope_key(file, &fn_spans, at);
            group_first_offset
                .entry(scope.clone())
                .and_modify(|o| *o = (*o).min(at))
                .or_insert(at);
            groups.entry(scope).or_default().push(RouteCall {
                offset: at,
                args_masked,
                args_text,
                unbounded,
            });
        }
    }

    // Composition fail-closed (item 4): a `.merge(X)` / `.nest(path, X)` / `.nest_service(...)` whose
    // X is not a router this same file scanned-and-cleared (a cross-crate/module call, a macro, a
    // function-returned router) cannot be assumed authz-covered. Attribute each to its scope so the
    // surface emits an AC-UNRESOLVED-SUBROUTER composition note.
    let mut group_compositions: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for marker in [".nest_service(", ".nest(", ".merge("] {
        let mut from = 0usize;
        while let Some(rel) = masked[from..].find(marker) {
            let at = from + rel;
            from = at + marker.len();
            if test_spans.iter().any(|(lo, hi)| at >= *lo && at < *hi) {
                continue;
            }
            let open = at + marker.len();
            let Some(args_masked) = balanced_paren_body(masked, open) else {
                continue;
            };
            let off = args_masked.as_ptr() as usize - masked.as_ptr() as usize;
            let args_text = &text[off..off + args_masked.len()];
            let sub = subrouter_arg_display(marker, args_masked, args_text);
            let scope = scope_key(file, &fn_spans, at);
            group_compositions.entry(scope).or_default().push(sub);
        }
    }

    let mut out = Vec::new();
    for (scope, calls) in &groups {
        let mut routes: Vec<Route> = Vec::new();
        for call in calls {
            if let Some(route) = classify_route_call(
                call.args_masked,
                call.args_text,
                call.unbounded,
                &str_consts,
                &method_bindings,
                &method_consts,
            ) {
                routes.push(route);
            }
        }

        // A scope whose `.route(` calls were ALL non-route-introductions (domain/dispatch methods)
        // and that carries no unresolved composition is not a router surface — skip it.
        let has_composition = group_compositions
            .get(scope)
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        if routes.is_empty() && !has_composition {
            continue;
        }

        let router_line = group_first_offset
            .get(scope)
            .map(|o| line_of(text, *o))
            .unwrap_or(1);

        // Auth-layer detection runs over the enclosing fn body (the whole scope), so a `.layer(...)`
        // anywhere in the builder — before OR after the route calls — is seen (not bounded by a
        // truncated chain). For a file-scope group there is no fn body; fall back to scanning a
        // window around the route calls' span.
        let scope_slice = scope_masked_slice(masked, &fn_spans, calls);
        let has_auth_layer = chain_has_auth_layer(scope_slice, auth_layer_idents);

        let mut handler_authz = serde_json::Map::new();
        let mut handlers: BTreeSet<String> = BTreeSet::new();
        for route in &routes {
            handlers.insert(route.handler.clone());
        }
        for handler in &handlers {
            let covered = handler_body_has_guard(text, handler, authz_guard_idents);
            handler_authz.insert(handler.clone(), Value::from(covered));
        }

        let routes_json: Vec<Value> = routes
            .iter()
            .map(|r| {
                json!({
                    "path": match &r.path { Some(p) => Value::from(p.as_str()), None => Value::Null },
                    "path_raw": r.path_raw,
                    "method": r.method.label(),
                    "method_class": r.method.discriminant(),
                    "handler": r.handler,
                    "surface_unclassified": r.surface_unclassified,
                })
            })
            .collect();

        let compositions: Vec<Value> = group_compositions
            .get(scope)
            .map(|v| v.iter().map(|s| Value::from(s.as_str())).collect())
            .unwrap_or_default();

        out.push(json!({
            "file": file,
            "scope": scope_name(scope),
            "router_line": router_line as u64,
            "routes": routes_json,
            "has_auth_layer": has_auth_layer,
            "handler_authz": Value::Object(handler_authz),
            "unresolved_subrouters": compositions,
        }));
    }
    out
}

/// `(name, body_open_offset, body_end_offset)` for every `fn NAME(...) { .. }` in masked `text`. The
/// body span is `[brace_open, brace_close]` inclusive. Nested fns are included; the INNERMOST
/// containing span wins at attribution time. `fn` must be a keyword boundary so `transform` etc. do
/// not match. The name is the ident after `fn ` (turbofish/`<generics>`/`(` terminate it).
fn fn_body_spans(masked: &str) -> Vec<(String, usize, usize)> {
    let mut spans = Vec::new();
    let bytes = masked.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = masked[from..].find("fn ") {
        let at = from + rel;
        from = at + 3;
        if at != 0 && is_ident_byte(bytes[at - 1]) {
            continue;
        }
        // Read the name ident after `fn `.
        let name = read_path_ident(&masked[at + 3..]);
        if name.is_empty() {
            continue;
        }
        // The body brace is the first `{` after the signature; brace_body spans it.
        if let Some(body) = brace_body(masked, at + 3) {
            let open = body.as_ptr() as usize - masked.as_ptr() as usize;
            spans.push((name, open, open + body.len()));
        }
    }
    spans
}

/// The scope key for a route call at `at`: the innermost enclosing fn span, else file scope. Keyed
/// `<file>#<fn-or-fileScope>`; a smaller (more deeply nested) span wins so a route in an inner
/// helper attributes to that helper, not the outer fn.
fn scope_key(file: &str, fn_spans: &[(String, usize, usize)], at: usize) -> String {
    let mut best: Option<&(String, usize, usize)> = None;
    for span in fn_spans {
        if at >= span.1 && at < span.2 {
            match best {
                Some(b) if (span.2 - span.1) >= (b.2 - b.1) => {}
                _ => best = Some(span),
            }
        }
    }
    match best {
        Some((name, _, _)) => format!("{file}#{name}"),
        None => format!("{file}#<file-scope>"),
    }
}

/// The fn-name portion of a scope key (`<file>#<fn>` -> `<fn>`).
fn scope_name(scope: &str) -> &str {
    scope.rsplit('#').next().unwrap_or(scope)
}

/// The masked source slice to scan for an auth `.layer(...)` for a scope: the enclosing fn body when
/// the route calls live inside one, else a window spanning the calls (file-scope declarations).
fn scope_masked_slice<'a>(
    masked: &'a str,
    fn_spans: &[(String, usize, usize)],
    calls: &[RouteCall],
) -> &'a str {
    let Some(first) = calls.iter().map(|c| c.offset).min() else {
        return "";
    };
    let last = calls.iter().map(|c| c.offset).max().unwrap_or(first);
    // Innermost enclosing fn body.
    let mut best: Option<&(String, usize, usize)> = None;
    for span in fn_spans {
        if first >= span.1 && first < span.2 {
            match best {
                Some(b) if (span.2 - span.1) >= (b.2 - b.1) => {}
                _ => best = Some(span),
            }
        }
    }
    if let Some((_, lo, hi)) = best {
        return &masked[*lo..*hi];
    }
    // File scope: a window from the first call back to the start of its statement is hard to bound
    // textually; scan from the earliest call to the end of the last call's statement (next `;`).
    let end = masked[last..]
        .find(';')
        .map(|i| (last + i + 1).min(masked.len()))
        .unwrap_or(masked.len());
    &masked[first..end]
}

/// A short display string for a `.merge(X)` / `.nest(path, X)` / `.nest_service(...)` sub-router
/// argument, for the AC-UNRESOLVED-SUBROUTER finding detail + key. Normalized (whitespace stripped),
/// truncated. For `.nest(path, X)` the X (after the top-level comma) is shown; for `.merge(X)` /
/// `.nest_service(path, svc)` the whole/last arg is shown.
fn subrouter_arg_display(marker: &str, args_masked: &str, args_text: &str) -> String {
    let snippet = if marker == ".nest(" {
        // `.nest("/path", subrouter)` — the sub-router is after the top-level comma.
        match split_top_level_comma(args_masked) {
            (_, Some(_)) => {
                let off = split_top_level_comma(args_text).1.unwrap_or(args_text);
                off.trim().to_owned()
            }
            (whole, None) => whole.trim().to_owned(),
        }
    } else {
        args_text.trim().to_owned()
    };
    let normalized: String = snippet
        .chars()
        .filter(|c| !c.is_whitespace())
        .take(64)
        .collect();
    let op = marker.trim_start_matches('.').trim_end_matches('(');
    if normalized.is_empty() {
        format!("{op}(<empty>)")
    } else {
        format!("{op}({normalized})")
    }
}

/// 1-based line number of byte offset `at` in `text`.
fn line_of(text: &str, at: usize) -> usize {
    text[..at].bytes().filter(|&b| b == b'\n').count() + 1
}

/// Byte spans of `#[cfg(test)]`-gated items in `text`. For each `#[cfg(test)]` attribute, the gated
/// item runs from the attribute to the matching close brace of the FIRST `{` after it (a `mod
/// tests { .. }` or a `fn .. { .. }`). Routers within these spans are test fixtures, not production
/// surfaces. Brace matching reuses [`brace_body`] (string/char/comment aware). Both `#[cfg(test)]`
/// and the multi-cfg `#[cfg(all(test, ..))]`/`#[cfg(any(test, ..))]` forms are matched via the
/// `cfg(` + `test` token pair on the attribute line.
fn cfg_test_spans(text: &str) -> Vec<(usize, usize)> {
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut from = 0usize;
    while let Some(rel) = text[from..].find("#[cfg(") {
        let at = from + rel;
        // Read the attribute up to its closing `]`; require a `test` token within it.
        let attr_end = text[at..]
            .find(']')
            .map(|i| at + i + 1)
            .unwrap_or(text.len());
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

/// Whether a `#[cfg(...)]` attribute string carries `test` as a config predicate token (not a
/// substring of a larger ident like `tested`). Matches `cfg(test)`, `cfg(all(test, ..))`, etc.
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

/// Classify ONE `.route(` / `.route_service(` call's argument list, FAIL-CLOSED. Returns `None` when
/// the call is NOT an HTTP route-INTRODUCTION at all (a same-named domain/dispatch method like
/// `usecase.route(input)`, `self.router.route(req, ip)`, or an enum `intent.route()`), so it is not
/// mistaken for a router surface. Returns `Some(route)` for a genuine route-introduction, with the
/// route's path/method/handler extracted (and fail-closed `path: None` / `Unclassified` /
/// `surface_unclassified` where a field cannot be resolved).
///
/// A `.route(` is a genuine route-INTRODUCTION iff its argument shape matches one of the two HTTP
/// route grammars the corpus uses — and crucially an HTTP route ALWAYS carries either a string-ish
/// path or an `HttpMethod::` verb, so the discriminator never misses a real route:
/// - **owned `oya-http-router-kernel`** `(HttpMethod::X, path, handler)`: arg1 is `HttpMethod::X`.
/// - **axum** `(path, METHOD(handler))`: arg1 is a `"..."`/raw-string literal path OR a
///   route-path-shaped ident/`&expr` path AND arg2 is a recognized method-router
///   (`get/post/.../on(MethodFilter::X, h)`/a resolvable `let`-bound var). The literal-path case
///   alone qualifies (arg2 may be a var); the ident/`&expr` path case requires a method-router arg2
///   so a single-arg `route(self)` / `route(struct{..})` domain method does not register as a route.
///
/// `args_masked` empty ⇒ a truncated/unbounded call we still treat fail-closed as an unclassified
/// SURFACE (a real builder route the parser could not bound — never silently dropped).
fn classify_route_call(
    args_masked: &str,
    args_text: &str,
    unbounded: bool,
    str_consts: &std::collections::BTreeMap<String, String>,
    method_bindings: &std::collections::BTreeMap<String, MethodBinding>,
    method_consts: &std::collections::BTreeMap<String, String>,
) -> Option<Route> {
    if unbounded {
        // Truncated / unbounded call (no balanced close) — never silently dropped (fail-closed
        // unclassified surface). A bounded zero-arg `.route()` is dropped by the caller, not here.
        return Some(Route {
            path: None,
            path_raw: "<unparsed-route-args>".to_owned(),
            method: MethodClass::Unclassified("<unparsed>".to_owned()),
            handler: String::new(),
            surface_unclassified: true,
        });
    }
    if args_masked.trim().is_empty() {
        return None;
    }

    let (arg1_masked, rest_masked) = split_top_level_comma(args_masked);
    let arg1_text = split_top_level_comma(args_text).0;
    let arg1_trim = arg1_masked.trim();

    // The owned-kernel `(method, path, handler)` arg1 is `HttpMethod::X` OR a `const NAME: HttpMethod`.
    // Resolve a const method to its verb; an `HttpMethod`-typed const that did not resolve to a known
    // verb stays None (handled as an unknown owned-kernel method below).
    let owned_verb = strip_http_method_prefix(arg1_trim).or_else(|| {
        let id = arg1_trim;
        if !id.is_empty() && id.chars().all(is_ident_char) {
            method_consts.get(id).cloned()
        } else {
            None
        }
    });

    // ---- Owned-kernel grammar: `.route(HttpMethod::X | METHOD_CONST, path, handler)` --------------
    if let Some(verb) = owned_verb {
        let method = classify_http_method_verb(&verb);
        let (path, path_raw, handler) =
            owned_kernel_path_handler(args_masked, args_text, str_consts);
        return Some(Route {
            path,
            path_raw,
            method,
            handler,
            surface_unclassified: false,
        });
    }

    // ---- axum grammar: `.route(path, METHOD(handler))` --------------------------------------------
    // A plain `"..."` literal masks to `"   "` (quotes kept) so it is detectable on the masked arg; a
    // raw string `r"..."` masks to all-blanks, so detect it on the ORIGINAL text view.
    let arg1_is_string = arg1_trim.starts_with('"')
        || raw_string_open(arg1_text.trim_start().as_bytes(), 0).is_some();
    let arg1_is_path_ident = {
        let m = arg1_trim;
        !m.is_empty()
            && m.chars().all(|c| is_ident_char(c) || c == ':')
            && m.chars().any(is_ident_char)
    };
    // A `&format!(...)` / `&PATH` path expr is path-shaped.
    let arg1_is_ref_expr = arg1_trim.starts_with('&');
    // The disambiguator: arg2 (after the first comma) is a recognized method-router. A real axum
    // route always has one; a domain `route(x)`/`route()` does not.
    let (arg2_class, arg2_handler) =
        classify_method_router(rest_masked.unwrap_or(""), method_bindings);
    let arg2_is_method_router =
        !arg2_class.is_unclassified() || method_router_call_shaped(rest_masked.unwrap_or(""));

    // A genuine axum route-introduction: a literal path (arg2 may be a var) OR a path-shaped
    // ident/&expr WITH a method-router arg2.
    if arg1_is_string || ((arg1_is_path_ident || arg1_is_ref_expr) && arg2_is_method_router) {
        let (path, path_raw) = resolve_path_arg(arg1_masked, arg1_text, str_consts);
        return Some(Route {
            path,
            path_raw,
            method: arg2_class,
            handler: arg2_handler,
            surface_unclassified: false,
        });
    }

    // ---- Owned-kernel-SHAPED but method-UNRESOLVABLE: `.route(<ident>, <path>, <handler>)` --------
    // A 3-arg call whose arg1 is a bare ident (a method-typed value the gate could not resolve to a
    // verb) and whose arg2 is a path-shaped literal/const is an owned-kernel route with an unknown
    // method → fail-closed UNCLASSIFIED-METHOD (not dropped — a `const M = HttpMethod::Post` aliased
    // through a non-`HttpMethod`-typed const must never silently vanish).
    if arg1_is_path_ident && let Some(rest_m) = rest_masked {
        let (arg2_m, after2) = split_top_level_comma(rest_m);
        let arg2_t = split_top_level_comma(args_text).1.unwrap_or("");
        let (arg2_path_t, _) = split_top_level_comma(arg2_t);
        let arg2_is_path = {
            let a = arg2_m.trim();
            a.starts_with('"')
                || raw_string_open(a.as_bytes(), 0).is_some()
                || (!a.is_empty() && a.chars().all(|c| is_ident_char(c) || c == ':'))
        };
        if after2.is_some() && arg2_is_path {
            let (path, path_raw) = resolve_path_arg(arg2_m, arg2_path_t, str_consts);
            let handler = after2.map(handler_ident_of).unwrap_or_default();
            return Some(Route {
                path,
                path_raw,
                method: MethodClass::Unclassified(format!("method-ident:{arg1_trim}")),
                handler,
                surface_unclassified: false,
            });
        }
        // Fail-CLOSED: 3-arg owned-kernel-shaped call where arg2 is NOT a path-shaped
        // literal/ident (e.g. `route.path` — a field access containing `.`). We know there
        // ARE 3 args (after2.is_some()) and arg1 is a bare ident, so this is structurally a
        // `.route(method, <non-literal-path>, handler)` call the engine cannot fully classify.
        // Return AC-UNCLASSIFIED-SURFACE rather than dropping (which would be fail-open).
        if after2.is_some() {
            return Some(Route {
                path: None,
                path_raw: format!("<unclassified-field-path:{}>", arg2_m.trim()),
                method: MethodClass::Unclassified(format!("method-ident:{arg1_trim}")),
                handler: after2.map(handler_ident_of).unwrap_or_default(),
                surface_unclassified: true,
            });
        }
    }

    // A literal/HttpMethod-shaped arg1 was absent AND arg2 is not a method-router ⇒ this `.route(` is
    // not an HTTP route-introduction (a same-named domain/dispatch method). Drop it — not a surface.
    None
}

/// Extract `(path, path_raw, handler)` from an owned-kernel `.route(method, path, handler)` arg list
/// (the method arg1 already consumed by the caller). arg2 = path, arg3 = handler.
fn owned_kernel_path_handler(
    args_masked: &str,
    args_text: &str,
    str_consts: &std::collections::BTreeMap<String, String>,
) -> (Option<String>, String, String) {
    let (_, rest_masked) = split_top_level_comma(args_masked);
    let Some(rest_m) = rest_masked else {
        return (
            None,
            "<owned-kernel-missing-path>".to_owned(),
            String::new(),
        );
    };
    let rest_t = split_top_level_comma(args_text).1.unwrap_or("");
    let (path_arg_m, after_path_m) = split_top_level_comma(rest_m);
    let (path_arg_t, _) = split_top_level_comma(rest_t);
    let (p, raw) = resolve_path_arg(path_arg_m, path_arg_t, str_consts);
    let handler = after_path_m.map(handler_ident_of).unwrap_or_default();
    (p, raw, handler)
}

/// Whether a method-router argument is at least a CALL shape (`ident(...)`) — even if its inner ident
/// is not a recognized HTTP method. Used to keep a route with an unrecognized-but-call-shaped method
/// router (`route("/x", custom(h))`) as a route-introduction (fail-closed UNCLASSIFIED-METHOD) rather
/// than dropping it, while a non-call arg2 (`route(self, ip)`) does not falsely qualify.
fn method_router_call_shaped(arg: &str) -> bool {
    let t = arg.trim();
    // `ident( ... )` — a leading ident immediately followed (after ws) by `(`.
    let ident = read_path_ident(t);
    if ident.is_empty() {
        return false;
    }
    let after = t[ident.len()..].trim_start();
    after.starts_with('(')
}

/// If `arg` is an `HttpMethod::X` (or a `::`-qualified `..::HttpMethod::X`) verb expression, return
/// the verb `X`; else None. The owned `oya-http-router-kernel` first route arg.
fn strip_http_method_prefix(arg: &str) -> Option<String> {
    let needle = "HttpMethod::";
    let at = arg.find(needle)?;
    // Must be a whole-segment `HttpMethod` (preceded by start, `:`, or non-ident).
    let before = arg.as_bytes().get(at.wrapping_sub(1)).copied();
    if at != 0
        && let Some(b) = before
        && is_ident_byte(b)
    {
        return None;
    }
    let verb = read_path_ident(&arg[at + needle.len()..]);
    // The whole arg must be JUST the verb expression (no trailing call/args), else it is not a bare
    // method discriminant in the owned grammar.
    let tail = arg[at + needle.len() + verb.len()..].trim();
    if verb.is_empty() || !tail.is_empty() {
        return None;
    }
    Some(verb)
}

/// Classify an owned-kernel `HttpMethod::X` verb into a [`MethodClass`]. POST/PUT/PATCH/DELETE are
/// mutating; GET/HEAD/OPTIONS/TRACE non-mutating; an unrecognized verb is mutating-fail-closed.
fn classify_http_method_verb(verb: &str) -> MethodClass {
    let mutating = ["POST", "PUT", "PATCH", "DELETE", "CONNECT"];
    let nonmut = ["GET", "HEAD", "OPTIONS", "TRACE"];
    if mutating.iter().any(|m| m.eq_ignore_ascii_case(verb)) {
        MethodClass::Mutating(format!("HttpMethod::{verb}"))
    } else if nonmut.iter().any(|m| m.eq_ignore_ascii_case(verb)) {
        MethodClass::NonMutating(format!("HttpMethod::{verb}"))
    } else {
        MethodClass::Mutating(format!("HttpMethod::{verb}(unknown-verb)"))
    }
}

/// Read the handler ident from an owned-kernel `.route(method, path, HANDLER)` third arg. The handler
/// may be a bare ident, a `"string"` handler name (the kernel's H=&str test shape), or the repo-owned
/// `handler_to_sync(HandlerStruct)` typed-handler bridge; unwrap the bridge to the typed handler so
/// the guard probe can inspect `impl Handler for HandlerStruct { fn call(...) { ... } }`.
fn handler_ident_of(arg: &str) -> String {
    let trimmed = arg.trim().trim_start_matches('"');
    let wrapper = "handler_to_sync";
    if let Some(at) = trimmed.find(wrapper) {
        let before_ok = at == 0 || !is_ident_byte(trimmed.as_bytes()[at - 1]);
        let after_name = at + wrapper.len();
        let after = &trimmed[after_name..];
        if before_ok && let Some(open_rel) = after.find('(') {
            let inner = &after[open_rel + 1..];
            let (first_arg, _) = split_top_level_comma(inner);
            let inner_ident = read_path_ident(first_arg);
            if !inner_ident.is_empty() {
                return inner_ident;
            }
        }
    }
    read_path_ident(trimmed)
}

/// Resolve a `.route(` PATH argument to a concrete string. Structure (literal vs ident) is read from
/// `arg_masked` (a `"..."` literal masks to `"   "` with quotes kept); the literal VALUE is read from
/// `arg_text` (the original) at the aligned offset. A bare ident (`const`/`static`) is substituted
/// from `str_consts`. Returns `(resolved, raw)` where `resolved` is `None` for an ident/expression
/// that resolves to no concrete string (fail-closed). `raw` is the value/ident (for detail + key).
fn resolve_path_arg(
    arg_masked: &str,
    arg_text: &str,
    str_consts: &std::collections::BTreeMap<String, String>,
) -> (Option<String>, String) {
    let masked_trimmed = arg_masked.trim_start();
    if masked_trimmed.starts_with('"') {
        // String literal: read its value (escape-aware) from the ORIGINAL text at the same offset.
        let lead = arg_text.len() - arg_text.trim_start().len();
        if let Some((value, _)) = read_string_literal(&arg_text[lead..]) {
            let raw = value.clone();
            return (Some(value), raw);
        }
    }
    // Raw-string path `r"..."` / `r#"..."#`: the MASKED view blanks the whole raw span (no quotes
    // kept), so detect + read the value from the ORIGINAL text (a raw string has no escapes).
    let text_trimmed = arg_text.trim_start();
    if let Some((content_start, end)) = raw_string_open(text_trimmed.as_bytes(), 0) {
        // content runs to the closing delimiter; recompute its `#`-count to bound the content end.
        let hashes = text_trimmed[..content_start]
            .bytes()
            .rev()
            .skip(1) // the opening `"`
            .take_while(|&b| b == b'#')
            .count();
        let content_end = end.saturating_sub(1 + hashes); // `"` + N `#`
        if content_end >= content_start {
            let value = text_trimmed[content_start..content_end].to_owned();
            let raw = value.clone();
            return (Some(value), raw);
        }
    }
    // Bare ident / path / expression: take the LAST `::`-segment as the const name
    // (`crate::routes::FOO` -> `FOO`) and look it up. The ident is read from the masked arg (idents
    // are code, unaffected by masking). A missing entry / non-ident expression is fail-closed (None).
    let masked = masked_trimmed.trim();
    let ident = masked.rsplit("::").next().unwrap_or(masked).trim();
    let ident = ident.trim_end_matches(|c: char| !is_ident_char(c));
    // Only treat it as a const lookup if the whole arg is a clean path-ident (no call/format!/etc.).
    let is_clean_ident = !ident.is_empty()
        && ident.chars().all(is_ident_char)
        && masked.chars().all(|c| is_ident_char(c) || c == ':');
    if is_clean_ident {
        if let Some(value) = str_consts.get(ident) {
            return (Some(value.clone()), ident.to_owned());
        }
        return (None, ident.to_owned());
    }
    // A non-literal, non-clean-ident path expression (e.g. `&format!(...)`): unresolved, fail-closed.
    // Keep a short raw snippet (from the ORIGINAL text, normalized) for the finding/key stability.
    let raw: String = arg_text
        .trim()
        .chars()
        .filter(|c| !c.is_whitespace())
        .take(48)
        .collect();
    (
        None,
        if raw.is_empty() {
            "<empty-path-arg>".to_owned()
        } else {
            raw
        },
    )
}

/// Read a `"..."` string literal at the start of `text` (which must begin with `"`); return its
/// unescaped contents and the offset just past the closing quote.
fn read_string_literal(text: &str) -> Option<(String, usize)> {
    let bytes = text.as_bytes();
    if bytes.is_empty() || bytes[0] != b'"' {
        return None;
    }
    let mut i = 1usize;
    let mut value = String::new();
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'\\' && i + 1 < bytes.len() {
            value.push(bytes[i + 1] as char);
            i += 2;
            continue;
        }
        if c == b'"' {
            return Some((value, i + 1));
        }
        value.push(c as char);
        i += 1;
    }
    None
}

/// Classify a `.route(`'s METHOD-ROUTER argument into a [`MethodClass`] + handler ident, FAIL-CLOSED.
///
/// Recognized shapes:
/// - inline `get|head|options|trace(handler)` -> `NonMutating`
/// - inline `post|put|patch|delete(handler)`  -> `Mutating`
/// - `any(handler)`                            -> `Mutating` (any verb, includes writes)
/// - `on(MethodFilter::X, handler)` / `on_service(MethodFilter::X, handler)` -> `Mutating` if the
///   filter set contains any mutating-or-unknown method, else `NonMutating`
/// - a bare ident bound by `let m = METHOD(h);` -> resolved via `method_bindings`
///
/// Anything else (an unbound variable, an unrecognized call) -> `Unclassified` (treated as a
/// potentially-mutating control plane requiring authz).
fn classify_method_router(
    arg: &str,
    method_bindings: &std::collections::BTreeMap<String, MethodBinding>,
) -> (MethodClass, String) {
    let trimmed = arg.trim();
    // 1) on(...) / on_service(...) with a MethodFilter.
    for shape in ["on_service", "on"] {
        if let Some(rest) = strip_call(trimmed, shape) {
            let class = classify_on_filter(rest, shape);
            let handler = read_handler_after_filter(rest);
            return (class, handler);
        }
    }
    // 2) inline METHOD(handler).
    let nonmut = ["get", "head", "options", "trace"];
    let mutmeth = ["post", "put", "patch", "delete"];
    for m in nonmut {
        if let Some(rest) = strip_call(trimmed, m) {
            return (
                MethodClass::NonMutating(m.to_owned()),
                read_path_ident(rest),
            );
        }
    }
    for m in mutmeth {
        if let Some(rest) = strip_call(trimmed, m) {
            return (MethodClass::Mutating(m.to_owned()), read_path_ident(rest));
        }
    }
    if let Some(rest) = strip_call(trimmed, "any") {
        // `any` accepts every verb, writes included.
        return (
            MethodClass::Mutating("any".to_owned()),
            read_path_ident(rest),
        );
    }
    // 3) a bare ident — a `let m = METHOD(h);` MethodRouter variable. Resolve via the binding map.
    let ident = read_path_ident(trimmed);
    if !ident.is_empty() && ident == trimmed.trim_end_matches(|c: char| !is_ident_char(c)) {
        if let Some(bound) = method_bindings.get(&ident) {
            // The binding carries both the method class and the handler ident, so a variable-bound
            // mutating route is probed for its real handler's guard (not falsely flagged uncovered).
            return (bound.class.clone(), bound.handler.clone());
        }
        // Unresolvable variable -> fail closed.
        return (
            MethodClass::Unclassified(format!("var:{ident}")),
            String::new(),
        );
    }
    // 4) anything else -> fail closed.
    (
        MethodClass::Unclassified("<unrecognized>".to_owned()),
        String::new(),
    )
}

/// Collect `const NAME: HttpMethod = HttpMethod::X;` declarations into a `NAME -> verb` map, so an
/// owned-kernel route whose method arg is a const-of-type-`HttpMethod` (e.g.
/// `const MODULE_REGISTRY_REST_METHOD: HttpMethod = HttpMethod::Get;`) resolves to its verb instead
/// of being dropped — closing the fail-OPEN hole where a `const … = HttpMethod::Post;` mutating route
/// would otherwise be invisible. Located in `masked`; the `HttpMethod::X` initializer is an ident
/// (mask-safe), so the verb is read from `masked` directly.
fn collect_method_consts(masked: &str) -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    let bytes = masked.as_bytes();
    for kw in ["const ", "static "] {
        let mut from = 0usize;
        while let Some(rel) = masked[from..].find(kw) {
            let at = from + rel;
            from = at + kw.len();
            if at != 0 && is_ident_byte(bytes[at - 1]) {
                continue;
            }
            let decl = &masked[at + kw.len()..];
            let Some(colon) = decl.find(':') else {
                continue;
            };
            let name = decl[..colon].trim().trim_start_matches("mut ").trim();
            if name.is_empty() || !name.chars().all(is_ident_char) {
                continue;
            }
            let Some(eq) = decl[colon..].find('=') else {
                continue;
            };
            let init_start = colon + eq + 1;
            let semi = decl[init_start..]
                .find(';')
                .map(|i| init_start + i)
                .unwrap_or(decl.len());
            let init = decl[init_start..semi].trim();
            if let Some(verb) = strip_http_method_prefix(init) {
                out.insert(name.to_owned(), verb);
            }
        }
    }
    out
}

/// If `text` begins with `ident` immediately followed (after optional whitespace) by `(`, return the
/// slice just past that `(`; else None. Ensures `ident` is a whole call ident, not a prefix.
fn strip_call<'a>(text: &'a str, ident: &str) -> Option<&'a str> {
    let t = text.trim_start();
    let rest = t.strip_prefix(ident)?;
    // The char right after the ident must not be an ident char (so `on` != `onfoo`).
    if let Some(c) = rest.chars().next()
        && is_ident_char(c)
    {
        return None;
    }
    let rest = rest.trim_start();
    rest.strip_prefix('(')
}

/// Classify an `on(...)` / `on_service(...)` argument list by its `MethodFilter::X` tokens. A filter
/// set containing any mutating method (POST/PUT/PATCH/DELETE) — OR no recognized filter token at all
/// (fail-closed: an unknown/computed filter) — is `Mutating`; an exclusively non-mutating set
/// (GET/HEAD/OPTIONS/TRACE) is `NonMutating`.
fn classify_on_filter(args: &str, shape: &str) -> MethodClass {
    let mutating = ["POST", "PUT", "PATCH", "DELETE"];
    let nonmut = ["GET", "HEAD", "OPTIONS", "TRACE"];
    let mut saw_nonmut = false;
    let mut saw_mut = false;
    let mut saw_any_filter = false;
    let mut from = 0usize;
    while let Some(rel) = args[from..].find("MethodFilter::") {
        let at = from + rel + "MethodFilter::".len();
        let tok = read_path_ident(&args[at..]);
        from = at;
        if tok.is_empty() {
            continue;
        }
        saw_any_filter = true;
        if mutating.iter().any(|m| m.eq_ignore_ascii_case(&tok)) {
            saw_mut = true;
        } else if nonmut.iter().any(|m| m.eq_ignore_ascii_case(&tok)) {
            saw_nonmut = true;
        } else {
            // Unknown filter token -> fail closed.
            saw_mut = true;
        }
    }
    if !saw_any_filter {
        // `on(filter_expr, h)` with a non-literal filter -> fail closed (potentially mutating).
        return MethodClass::Unclassified(format!("{shape}(<dynamic-filter>)"));
    }
    if saw_mut {
        MethodClass::Mutating(format!("{shape}(MethodFilter)"))
    } else if saw_nonmut {
        MethodClass::NonMutating(format!("{shape}(MethodFilter)"))
    } else {
        MethodClass::Unclassified(format!("{shape}(MethodFilter)"))
    }
}

/// Read the handler ident from an `on(MethodFilter::X, handler)` argument list: the ident after the
/// top-level comma. Returns empty if none found.
fn read_handler_after_filter(args: &str) -> String {
    let (_, after) = split_top_level_comma(args);
    match after {
        Some(a) => read_path_ident(a),
        None => String::new(),
    }
}

/// Whether `c` is a Rust identifier char.
fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Return the balanced-paren body starting just after an opening `(` at offset `open` (i.e. the
/// slice between that `(` and its matching `)`), skipping string/char literals and line comments so
/// a paren inside them does not throw off the balance. Returns None if no matching close is found.
fn balanced_paren_body(text: &str, open: usize) -> Option<&str> {
    let bytes = text.as_bytes();
    let mut i = open;
    let mut depth = 1i32; // we are already inside the first `(`
    let body_start = open;
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&text[body_start..i]);
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

/// Split an argument list at its FIRST top-level comma (depth-0, outside strings/chars/comments).
/// Returns `(first_arg, rest)` where `rest` is `None` when there is no top-level comma.
fn split_top_level_comma(args: &str) -> (&str, Option<&str>) {
    let bytes = args.as_bytes();
    let mut depth = 0i32;
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            b'(' | b'[' | b'{' | b'<' => depth += 1,
            b')' | b']' | b'}' | b'>' => depth -= 1,
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
            b',' if depth == 0 => {
                return (&args[..i], Some(&args[i + 1..]));
            }
            _ => {}
        }
        i += 1;
    }
    (args, None)
}

/// Collect `const NAME: &str = "...";` and `static NAME: &str = "...";` string declarations in the
/// file into a `NAME -> value` map for const-path resolution (B1). Only string-literal initializers
/// are captured; a `const NAME: &str = other_const;` (no literal) is intentionally NOT captured, so
/// it resolves to fail-closed `None` at the route site.
fn collect_str_consts(masked: &str, text: &str) -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    for kw in ["const ", "static "] {
        let mut from = 0usize;
        while let Some(rel) = masked[from..].find(kw) {
            let at = from + rel;
            from = at + kw.len();
            // `const`/`static` must be a keyword boundary (preceded by start/non-ident).
            let bytes = masked.as_bytes();
            if at != 0 && is_ident_byte(bytes[at - 1]) {
                continue;
            }
            // Read NAME up to `:` (structure from masked; the name is an ident, mask-safe).
            let after_kw = at + kw.len();
            let decl_masked = &masked[after_kw..];
            let Some(colon) = decl_masked.find(':') else {
                continue;
            };
            let name = decl_masked[..colon].trim();
            // `static mut` / generics make the name non-simple — require a plain ident.
            let name = name.trim_start_matches("mut ").trim();
            if name.is_empty() || !name.chars().all(is_ident_char) {
                continue;
            }
            // Find the `=` (masked) then the first string literal before the terminating `;`. The
            // initializer VALUE is read from the ORIGINAL text at the aligned offset.
            let Some(eq) = decl_masked[colon..].find('=') else {
                continue;
            };
            let init_start = after_kw + colon + eq + 1;
            let semi = masked[init_start..]
                .find(';')
                .map(|i| init_start + i)
                .unwrap_or(masked.len());
            let init_masked = masked[init_start..semi].trim_start();
            // Only a direct string-literal initializer is captured (its masked form starts with `"`).
            if init_masked.starts_with('"') {
                let lead = (masked[init_start..semi].len())
                    - (masked[init_start..semi].trim_start().len());
                let init_text = &text[init_start + lead..semi];
                if let Some((value, _)) = read_string_literal(init_text) {
                    out.insert(name.to_owned(), value);
                }
            }
        }
    }
    out
}

/// Collect `let NAME = METHOD(handler);` MethodRouter bindings in the file into a `NAME -> class`
/// map for B2 variable-method resolution. Recognizes the same inline method shapes as
/// [`classify_method_router`]; a binding to an unrecognized expression is omitted (so it resolves
/// fail-closed at the route site). File-wide capture is a safe superset of per-fn capture: it can
/// only ever make a binding RESOLVABLE; an unresolved binding still fails closed.
fn collect_method_bindings(text: &str) -> std::collections::BTreeMap<String, MethodBinding> {
    let mut out = std::collections::BTreeMap::new();
    let empty: std::collections::BTreeMap<String, MethodBinding> =
        std::collections::BTreeMap::new();
    let mut from = 0usize;
    while let Some(rel) = text[from..].find("let ") {
        let at = from + rel;
        from = at + 4;
        let bytes = text.as_bytes();
        if at != 0 && is_ident_byte(bytes[at - 1]) {
            continue;
        }
        let decl = &text[at + 4..];
        let Some(eq) = decl.find('=') else { continue };
        // The binding name is the ident before `=` (strip `mut`, type ascription).
        let name_part = decl[..eq].trim();
        let name_part = name_part.trim_start_matches("mut ").trim();
        let name = name_part.split(':').next().unwrap_or(name_part).trim();
        if name.is_empty() || !name.chars().all(is_ident_char) {
            continue;
        }
        let init_start = eq + 1;
        let semi = decl[init_start..]
            .find(';')
            .map(|i| init_start + i)
            .unwrap_or(decl.len());
        let init = decl[init_start..semi].trim();
        let (class, handler) = classify_method_router(init, &empty);
        // Only record a binding the RHS classified as a real method-router shape; an Unclassified RHS
        // (e.g. `let x = compute();`) is not a method router and must not poison the map.
        if !class.is_unclassified() {
            out.insert(name.to_owned(), MethodBinding { class, handler });
        }
    }
    out
}

/// Build a CODE-ONLY view of `body`: line comments (`// ..`), block comments (`/* .. */`),
/// string/char literals are all elided to spaces (length-preserving is unnecessary; idents are what
/// matter). Every other byte is kept verbatim. Used by the B3 guard probe so a guard ident inside a
/// comment or string literal can never false-cover a handler — the ident must appear in real code.
/// Reuses the same string/char skip machinery as [`brace_body`].
fn code_only(body: &str) -> String {
    let bytes = body.as_bytes();
    let mut out = String::with_capacity(body.len());
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            b'"' => {
                let end = skip_string(bytes, i);
                out.push(' ');
                i = end;
                continue;
            }
            b'\'' => {
                let end = skip_char_or_lifetime(bytes, i);
                // Preserve a lifetime's leading-tick-less ident bytes? No — a lifetime is not a
                // guard ident; eliding it is safe. Push a space placeholder.
                out.push(' ');
                i = end;
                continue;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                // Block comment: skip to the matching `*/` (non-nested is the common case; nested
                // block comments are rare and over-skipping only ever ELIDES code, which is the safe
                // direction — it can never invent a guard, only fail to find one in odd nesting).
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
                continue;
            }
            _ => {
                out.push(c as char);
                i += 1;
            }
        }
    }
    out
}

/// Read a Rust path ident from the start of `text`, stopping at the first char that is not part of
/// an ident or path separator (`a-z A-Z 0-9 _ :`). A trailing `::<...>` turbofish or `::method`
/// tail is trimmed to the leading ident segment. Leading whitespace is skipped.
fn read_path_ident(text: &str) -> String {
    let trimmed = text.trim_start();
    let mut ident = String::new();
    for c in trimmed.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            ident.push(c);
        } else if c == ':' {
            // path separator or turbofish start — stop at the first segment for the handler name.
            break;
        } else {
            break;
        }
    }
    ident
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Whether `ident` appears in `hay` as a COMPLETE identifier token — both sides bounded by a
/// non-ident byte (or the string edge) — not as a substring of a longer ident. This is the
/// BLOCKER-4/5 fix: a raw `hay.contains("RequireAuth")` false-matches `RequireAuthMetricsRecorder`,
/// and `hay.contains("authorize")` false-matches `unauthorized_response`. A guard/auth-layer ident
/// must be a real token. An `ident` that itself ends in a non-ident byte (e.g. the guard-shape
/// `.decide(` or the macro `guard!`) is matched verbatim (no right-boundary requirement, since the
/// final byte is already the boundary); its LEFT boundary is still required so `.decide(` does not
/// match `xdecide(`-style tails when the ident begins with an ident byte.
fn contains_ident_token(hay: &str, ident: &str) -> bool {
    if ident.is_empty() {
        return false;
    }
    let hb = hay.as_bytes();
    let ib = ident.as_bytes();
    let first_is_ident = is_ident_byte(ib[0]);
    let last_is_ident = is_ident_byte(ib[ib.len() - 1]);
    let mut from = 0usize;
    while let Some(rel) = hay[from..].find(ident) {
        let at = from + rel;
        let before_ok = !first_is_ident || at == 0 || !is_ident_byte(hb[at - 1]);
        let after = at + ib.len();
        let after_ok = !last_is_ident || after >= hb.len() || !is_ident_byte(hb[after]);
        if before_ok && after_ok {
            return true;
        }
        from = at + 1;
    }
    false
}

/// Whether `body` (a code-only handler body) invokes a recognized authz GUARD ident as a real call
/// or token. BLOCKER-5: a guard ident must appear as a complete identifier token (so
/// `unauthorized_response` cannot satisfy `authorize`); for a plain-ident guard we additionally
/// require it to be FOLLOWED by `(` (a call), so a guard name used only as a value/type does not
/// false-cover. Guard idents that already encode their own call/boundary shape (`.decide(`, `guard!`)
/// or end in a non-ident byte are matched as whole tokens without the extra `(` requirement.
fn body_invokes_guard(body: &str, guard: &str) -> bool {
    if guard.is_empty() {
        return false;
    }
    let gb = guard.as_bytes();
    let last_is_ident = is_ident_byte(gb[gb.len() - 1]);
    if !last_is_ident {
        // Shapes like `.decide(` / `guard!` carry their own call/macro boundary; whole-token match.
        return contains_ident_token(body, guard);
    }
    // A plain-ident guard: require it to be a complete token immediately followed (after optional
    // whitespace) by `(` — i.e. an actual call, not a mention as a value/type/path tail.
    let hb = body.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = body[from..].find(guard) {
        let at = from + rel;
        let before_ok = at == 0 || !is_ident_byte(hb[at - 1]);
        let after = at + gb.len();
        let after_ok = after >= hb.len() || !is_ident_byte(hb[after]);
        if before_ok && after_ok {
            // Skip whitespace; the next non-space byte must be `(` for a call.
            let mut j = after;
            while j < hb.len()
                && (hb[j] == b' ' || hb[j] == b'\t' || hb[j] == b'\n' || hb[j] == b'\r')
            {
                j += 1;
            }
            if j < hb.len() && hb[j] == b'(' {
                return true;
            }
        }
        from = at + 1;
    }
    false
}

/// Whether the chain carries a recognized router-level auth `.layer(...)` — a `.layer(` (or
/// `.route_layer(`) whose argument names one of the policy `auth_layer_idents`. A generic
/// `.layer(DefaultBodyLimit::max(...))` is NOT an auth layer (its ident is not in the list), so the
/// intelligence/adapters/rest body-limit layer does not false-cover that surface.
fn chain_has_auth_layer(chain: &str, auth_layer_idents: &[String]) -> bool {
    if auth_layer_idents.is_empty() {
        return false;
    }
    for marker in [".layer(", ".route_layer("] {
        let mut from = 0usize;
        while let Some(rel) = chain[from..].find(marker) {
            let open = from + rel + marker.len();
            // Read up to the matching-ish end of this layer call (next `.layer(`/`.route(`/EOF) and
            // test for any auth ident.
            let bound = [".layer(", ".route(", ".route_layer(", ".with_state("]
                .iter()
                .filter_map(|m| chain[open..].find(m).map(|i| open + i))
                .min()
                .unwrap_or(chain.len());
            let arg = &chain[open..bound];
            // BLOCKER-4: whole-token match so a layer named `RequireAuthMetricsRecorder` does NOT
            // satisfy the `RequireAuth` auth-layer ident (substring false-cover). The ident must
            // appear as a complete identifier token.
            if auth_layer_idents
                .iter()
                .any(|ident| contains_ident_token(arg, ident.as_str()))
            {
                return true;
            }
            from = open;
        }
    }
    false
}

/// The maximum delegation depth followed when a handler body is a thin wrapper. One hop covers the
/// common corpus shape (`async fn h(..) -> R { delegate(..).await }`, e.g.
/// `handle_openai_chat_completions` -> `handle_openai_compatible_proxy`), where the authz guard
/// lives in the single local function the handler delegates to. Bounded so the scan cannot loop or
/// degrade into full call-graph analysis (the deeper proof is review's / the audit gate's job).
const MAX_DELEGATE_DEPTH: usize = 2;

/// Whether the `fn <handler>` body anywhere in the file invokes a recognized authz guard ident,
/// following up to [`MAX_DELEGATE_DEPTH`] thin-wrapper delegations.
///
/// Locates each `fn <handler>` definition (token-bounded), spans its body by brace matching, and
/// tests whether any `authz_guard_idents` token appears in that body. For the repo-owned
/// `handler_to_sync(HandlerStruct)` bridge, also locates
/// `impl Handler for HandlerStruct { fn call(...) { ... } }` and probes that impl body. A guard ident
/// may be a plain ident (`authorize`, `admin_tenant_allowed`) or a method tail (`.decide(`) — both are
/// simple substring probes, sound for the over-approximation this gate intends (a guard token in the
/// body ⇒ the handler derives caller identity). Async handlers are covered because we anchor on
/// `fn <handler>` regardless of the `async`/`pub` prefix.
///
/// If the body names no guard directly, it is probed for a SINGLE local-function delegate it calls
/// (a thin wrapper like `handle_openai_compatible_proxy(state, headers, body, ..).await`); the gate
/// recurses one hop into that delegate's body. This recognizes the real intelligence/adapters/rest
/// data-plane wrappers and typed Handler impls without a full call graph.
fn handler_body_has_guard(text: &str, handler: &str, guard_idents: &[String]) -> bool {
    let mut seen = BTreeSet::new();
    has_guard_rec(text, handler, guard_idents, MAX_DELEGATE_DEPTH, &mut seen)
        || handler_impl_has_guard_rec(text, handler, guard_idents, MAX_DELEGATE_DEPTH, &mut seen)
}

fn has_guard_rec(
    text: &str,
    handler: &str,
    guard_idents: &[String],
    depth: usize,
    seen: &mut BTreeSet<String>,
) -> bool {
    if handler.is_empty() || guard_idents.is_empty() || !seen.insert(handler.to_owned()) {
        return false;
    }
    let needle = format!("fn {handler}");
    let bytes = text.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = text[from..].find(&needle) {
        let at = from + rel;
        // Ensure `fn ` is a real keyword boundary (preceded by whitespace or start) and the char
        // after the handler name is not an ident char (so `fn handler` != `fn handler_extra`).
        let before_ok = at == 0 || !is_ident_byte(bytes[at - 1]);
        let after_name = at + needle.len();
        let after_ok = after_name >= bytes.len() || !is_ident_byte(bytes[after_name]);
        if before_ok
            && after_ok
            && let Some(body) = brace_body(text, after_name)
        {
            // B3 FIX: the guard probe runs on a CODE-ONLY view of the body — comments and
            // string/char literals elided — so a `// TODO: authorize()` comment or a
            // `"authorize"` string literal can NEVER false-cover a handler that does no real
            // authz. The guard ident must appear in genuine code.
            let code = code_only(body);
            // BLOCKER-5: whole-token + call-shape match so a body that only references
            // `unauthorized_response()` does NOT satisfy the `authorize` guard ident (substring
            // false-cover). A plain-ident guard must appear as a complete token followed by `(`.
            if guard_idents
                .iter()
                .any(|g| body_invokes_guard(&code, g.as_str()))
            {
                return true;
            }
            // No direct guard: follow up to one local delegate this body calls. Delegate calls
            // are read from the same code-only view (a name in a comment/string is not a call).
            if depth > 0 {
                for delegate in delegate_calls_in(&code, handler) {
                    if has_guard_rec(text, &delegate, guard_idents, depth - 1, seen) {
                        return true;
                    }
                }
            }
        }
        from = at + needle.len();
    }
    false
}

fn handler_impl_has_guard_rec(
    text: &str,
    handler: &str,
    guard_idents: &[String],
    depth: usize,
    seen: &mut BTreeSet<String>,
) -> bool {
    if handler.is_empty() || guard_idents.is_empty() || !seen.insert(format!("impl:{handler}")) {
        return false;
    }

    let needle = format!(" for {handler}");
    let bytes = text.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = text[from..].find(&needle) {
        let at = from + rel;
        let prefix_start = at.saturating_sub(200);
        let prefix = &text[prefix_start..at];
        let before_ok = !is_ident_byte(bytes[at]);
        let after_name = at + needle.len();
        let after_ok = after_name >= bytes.len() || !is_ident_byte(bytes[after_name]);
        if before_ok
            && after_ok
            && prefix.contains("impl")
            && prefix.contains("Handler")
            && let Some(body) = brace_body(text, after_name)
        {
            let code = code_only(body);
            if guard_idents
                .iter()
                .any(|g| body_invokes_guard(&code, g.as_str()))
            {
                return true;
            }
            if depth > 0 {
                for delegate in delegate_calls_in(&code, handler) {
                    if has_guard_rec(text, &delegate, guard_idents, depth - 1, seen)
                        || handler_impl_has_guard_rec(
                            text,
                            &delegate,
                            guard_idents,
                            depth - 1,
                            seen,
                        )
                    {
                        return true;
                    }
                }
            }
        }
        from = at + needle.len();
    }
    false
}

/// The set of local-function idents called within a handler body (candidate delegates). A call is a
/// bare `ident(` not preceded by `.` (method calls and qualified `Type::ident(` constructors are
/// excluded — we want a sibling free function the wrapper forwards to). The handler's own name is
/// excluded to avoid trivial self-recursion. Deterministically ordered.
fn delegate_calls_in(body: &str, own: &str) -> Vec<String> {
    let bytes = body.as_bytes();
    let mut out: BTreeSet<String> = BTreeSet::new();
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_alphabetic() || c == b'_' {
            let start = i;
            while i < bytes.len() && is_ident_byte(bytes[i]) {
                i += 1;
            }
            // optional whitespace then `(`
            let mut j = i;
            while j < bytes.len()
                && (bytes[j] == b' ' || bytes[j] == b'\n' || bytes[j] == b'\t' || bytes[j] == b'\r')
            {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'(' {
                // Not a method call (`.ident(`) and not a path tail (`::ident(`): require the char
                // before the ident to be neither `.` nor `:`.
                let before = if start == 0 { b' ' } else { bytes[start - 1] };
                if before != b'.' && before != b':' {
                    let ident = &body[start..i];
                    if ident != own && !is_keyword(ident) {
                        out.insert(ident.to_owned());
                    }
                }
            }
            continue;
        }
        i += 1;
    }
    out.into_iter().collect()
}

/// Rust keywords / control-flow idents that look like calls (`if (`, `match (`, `while (`) but are
/// never function delegates. Excluded so the one-hop follow does not chase control flow.
fn is_keyword(ident: &str) -> bool {
    matches!(
        ident,
        "if" | "while"
            | "for"
            | "match"
            | "loop"
            | "return"
            | "let"
            | "fn"
            | "async"
            | "await"
            | "move"
            | "in"
            | "as"
            | "ref"
            | "mut"
            | "Some"
            | "Ok"
            | "Err"
            | "None"
    )
}

/// From offset `from`, find the first `{` and return the slice of the brace-balanced body up to its
/// matching `}`. String/char literals and line comments are skipped so a brace inside them does not
/// throw off the balance. Returns None if no opening brace or no balanced close is found.
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

/// If a raw-string literal opens at offset `start` (`r"`, `r#"`, `r##"`..., or byte-raw `br"`,
/// `br#"`...), return `(content_start, end)` where `content_start` is the offset of the first
/// content byte (just past the opening `"`) and `end` is the offset just past the closing
/// delimiter (`"` + the same number of `#`). Returns None if `start` is not a raw-string opener.
///
/// The opener must be a whole token: the byte before `r`/`b` must not be an ident byte (so `for"`
/// or `super` does not parse as a raw string). A raw string has NO escapes; its terminator is the
/// FIRST `"` followed by exactly the opener's `#` count. MINOR-1: matching this exactly is what
/// keeps the [`mask_non_code`] offset alignment from desyncing on `r#"..."#` content.
fn raw_string_open(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    // Left boundary: not part of a longer ident.
    if start > 0 && is_ident_byte(bytes[start - 1]) {
        return None;
    }
    let mut i = start;
    if i < bytes.len() && bytes[i] == b'b' {
        i += 1; // byte-raw prefix
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
        return None; // `r` not followed by `#`*`"` is an ident (e.g. `route`), not a raw string.
    }
    i += 1; // past opening quote
    let content_start = i;
    // Terminator: a `"` followed by exactly `hashes` `#`.
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
    // Unterminated raw string: consume to EOF (fail-closed — never leaks structure past it).
    Some((content_start, bytes.len()))
}

/// Skip a `"..."` string literal starting at `start` (the opening quote); return the offset just
/// past the closing quote (or EOF). Handles escapes.
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

/// Skip a `'c'` char literal or a `'a` lifetime starting at `start`. Returns the offset just past
/// it. A lifetime (`'a`) has no closing quote, so we stop at the first non-ident char.
fn skip_char_or_lifetime(bytes: &[u8], start: usize) -> usize {
    // char literal: '\'' or 'x' or '\n'
    if start + 1 < bytes.len() && bytes[start + 1] == b'\\' {
        // escaped char literal '\x'
        let mut i = start + 2;
        while i < bytes.len() && bytes[i] != b'\'' {
            i += 1;
        }
        return i + 1;
    }
    if start + 2 < bytes.len() && bytes[start + 2] == b'\'' {
        // simple char literal 'x'
        return start + 3;
    }
    // lifetime: skip the ident
    let mut i = start + 1;
    while i < bytes.len() && is_ident_byte(bytes[i]) {
        i += 1;
    }
    i
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

/// Whether `path` is exempt (an unauthenticated read like `/healthz`) via the policy
/// `exempt_path_substrings` allowlist.
///
/// Matching is segment-boundary aware: an exempt substring must appear at a path-segment boundary
/// (preceded by `/` or start-of-string, followed by `/`, `?`, end-of-string, or end of a path
/// parameter `}`). This prevents `/version` from matching `/policies/{id}/versions/{v}` — the
/// `version` there is not the exempt health-probe segment `/version`. Plain contains() would silently
/// exempt a real mutating control-plane surface whose path contains a non-exempt segment that
/// happens to share a prefix with an exempt one (the BLOCKER root-cause for the Cedar publish port).
fn path_exempt(path: &str, exempt_substrings: &[String]) -> bool {
    exempt_substrings.iter().any(|s| {
        let sub = s.as_str();
        let mut from = 0usize;
        let path_b = path.as_bytes();
        while from < path.len() {
            let Some(rel) = path[from..].find(sub) else {
                break;
            };
            let at = from + rel;
            // Preceded by '/' or start-of-string.
            let before_ok = at == 0 || path_b.get(at - 1) == Some(&b'/');
            // Followed by '/', '?', end-of-string, or '}' (end of path param like {version}).
            let after = at + sub.len();
            let after_ok = after >= path.len() || matches!(path_b[after], b'/' | b'?' | b'}');
            if before_ok && after_ok {
                return true;
            }
            from = at + 1;
        }
        false
    })
}

/// Whether a route's path carries a per-resource path param (`{...}`).
fn has_path_param(path: &str) -> bool {
    path.contains('{') && path.contains('}')
}

/// The stable SIGNATURE key for a surface finding: `<file>#<scope>::router[<m1 p1; m2 p2; ..>]` where
/// the `(method, route-path)` tuples are sorted (M2) and `<scope>` is the enclosing-fn name (so two
/// route-introduction scopes in one file get distinct stable keys). Independent of line numbers and
/// route-declaration order, so an unrelated edit that shifts the router's line does NOT spuriously
/// re-RED a baselined surface. A route's tuple uses its resolved path when known, its raw path arg
/// (`const NAME`) when unresolved, prefixed by the method-class discriminant for an unclassified
/// method-router and tagged `surface:` for a structurally-unclassified route call. Composition
/// `unresolved_subrouters` are appended so a merge/nest fail-closed finding keys stably. Handler
/// names are excluded so a handler rename keeps the key stable.
fn surface_signature_key(
    file: &str,
    scope: &str,
    routes: &[Value],
    subrouters: &[Value],
) -> String {
    let mut tuples: Vec<String> = routes
        .iter()
        .map(|r| {
            let method = r.get("method").and_then(Value::as_str).unwrap_or("?");
            let class = r.get("method_class").and_then(Value::as_str).unwrap_or("?");
            let surface_unclassified = r
                .get("surface_unclassified")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let path = match r.get("path").and_then(Value::as_str) {
                Some(p) => p.to_owned(),
                None => format!(
                    "<unresolved:{}>",
                    r.get("path_raw").and_then(Value::as_str).unwrap_or("?")
                ),
            };
            if surface_unclassified {
                format!("surface:{path}")
            } else if class == "unclassified" {
                format!("{class}:{method} {path}")
            } else {
                format!("{method} {path}")
            }
        })
        .collect();
    for sub in subrouters {
        if let Some(s) = sub.as_str() {
            tuples.push(format!("subrouter:{s}"));
        }
    }
    tuples.sort();
    let scope = if scope.is_empty() {
        "<file-scope>"
    } else {
        scope
    };
    format!("{file}#{scope}::router[{}]", tuples.join("; "))
}

/// Pure evaluator. `policy` is DATA (`authz-coverage-policy.json`); `observed` is the collected
/// surface graph shaped by [`collect_surfaces`].
///
/// FAIL-CLOSED spine: a route makes its surface a CONTROL PLANE iff it is (a) a recognized mutating
/// method on a non-exempt path, (b) an UNCLASSIFIED method-router (the engine could not prove it
/// non-mutating), or (c) an UNRESOLVED path (the engine could not prove what surface it is). The
/// surface is COVERED iff it has a recognized auth layer OR every such route's handler body invokes
/// a recognized authz guard. An uncovered control-plane surface whose SIGNATURE key is NOT in the
/// frozen baseline → a blocking finding: `AC-UNRESOLVED-ROUTE-PATH` if it has any unresolved path,
/// `AC-UNCLASSIFIED-METHOD` if it has any unclassified method, else `AC-UNAUTHENTICATED-CONTROL-PLANE`.
/// Baseline keys with no live finding → `AC-STALE-BASELINE`.
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if policy.get("gate_id").and_then(Value::as_str) != Some(GATE_ID) {
        findings.insert(Finding::new(
            "AC-POLICY-GATE-ID-MISMATCH",
            POLICY_KEY,
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    // Fail CLOSED on a structurally invalid policy: the guard-ident list is the gate's whole
    // recognition vocabulary; an empty/absent one would mark every surface unauthenticated, but a
    // MISSING (null/non-array) list signals a corrupt policy — fail closed loudly rather than
    // silently flag the whole repo.
    if policy
        .get("authz_guard_idents")
        .and_then(Value::as_array)
        .is_none()
    {
        findings.insert(Finding::new(
            "AC-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `authz_guard_idents` must be an array of recognized authz-guard ident strings; correct the policy before the gate can evaluate",
        ));
        return findings;
    }
    if policy.get("scan_roots").and_then(Value::as_array).is_none() {
        findings.insert(Finding::new(
            "AC-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `scan_roots` must be an array of repo-relative scan-root strings; correct the policy before the gate can evaluate",
        ));
        return findings;
    }

    let exempt_substrings = string_list(policy, "exempt_path_substrings");
    let frozen_baseline: BTreeSet<String> = string_list(policy, "frozen_unauthenticated_surfaces")
        .into_iter()
        .collect();

    let min_expected = policy
        .get("min_expected_surfaces")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let surfaces_found = observed
        .get("surfaces_found")
        .and_then(Value::as_u64)
        .or_else(|| {
            observed
                .get("surfaces")
                .and_then(Value::as_array)
                .map(|s| s.len() as u64)
        })
        .unwrap_or(0);
    if surfaces_found < min_expected {
        findings.insert(Finding::new(
            "AC-EMPTY-SCAN",
            POLICY_KEY,
            format!(
                "scan found {surfaces_found} router surfaces, below the policy floor of {min_expected}; the scan roots, CWD, or collection is likely broken (fail-closed against a silent false-green)"
            ),
        ));
    }

    // The set of LIVE unauthenticated-surface keys, used to detect stale baseline entries.
    let mut live_unauth_keys: BTreeSet<String> = BTreeSet::new();

    let surfaces = observed
        .get("surfaces")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for surface in &surfaces {
        let Some(file) = surface.get("file").and_then(Value::as_str) else {
            continue;
        };
        let router_line = surface
            .get("router_line")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let scope = surface.get("scope").and_then(Value::as_str).unwrap_or("");
        let routes = surface
            .get("routes")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let has_auth_layer = surface
            .get("has_auth_layer")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let handler_authz = surface.get("handler_authz").and_then(Value::as_object);
        let subrouters = surface
            .get("unresolved_subrouters")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        // Walk the routes, fail-closed. A route makes the surface a CONTROL PLANE if it is mutating
        // (on a non-exempt resolved path), unclassified (unknown method), structurally unclassifiable
        // (the whole call matched no route grammar), or has an unresolved path.
        let mut is_control_plane = false;
        let mut has_unresolved_path = false;
        let mut has_unclassified_method = false;
        let mut has_unclassified_surface = false;
        // (label, path-display, handler) of every uncovered control-plane route.
        let mut uncovered_handlers: Vec<(String, String, String)> = Vec::new();
        for route in &routes {
            let class = route
                .get("method_class")
                .and_then(Value::as_str)
                .unwrap_or("");
            let method = route.get("method").and_then(Value::as_str).unwrap_or("");
            let handler = route.get("handler").and_then(Value::as_str).unwrap_or("");
            let path_opt = route.get("path").and_then(Value::as_str);
            let path_raw = route.get("path_raw").and_then(Value::as_str).unwrap_or("");
            let surface_unclassified = route
                .get("surface_unclassified")
                .and_then(Value::as_bool)
                .unwrap_or(false);

            let is_mutating = class == "mutating";
            let is_unclassified = class == "unclassified";
            let path_unresolved = path_opt.is_none();

            // A non-mutating, classified, resolved-path route that is NOT a structurally-unclassified
            // call is the only safe (skippable) case.
            if !is_mutating && !is_unclassified && !path_unresolved && !surface_unclassified {
                continue;
            }
            // A resolved exempt-path read is exempt even if mutating (e.g. a `/metrics` push). An
            // UNRESOLVED path cannot be proven exempt → never exempt (fail-closed). An unclassified
            // method/surface on an exempt resolved path is still potentially mutating, not exempt.
            if let Some(path) = path_opt
                && !is_unclassified
                && !surface_unclassified
                && path_exempt(path, &exempt_substrings)
            {
                continue;
            }

            is_control_plane = true;
            if path_unresolved && !surface_unclassified {
                has_unresolved_path = true;
            }
            if is_unclassified && !surface_unclassified {
                has_unclassified_method = true;
            }
            if surface_unclassified {
                has_unclassified_surface = true;
            }
            let _ = path_opt.map(has_path_param); // documented signal; mutating/unclassified qualifies.

            let covered = handler_authz
                .and_then(|m| m.get(handler))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if !covered {
                let path_disp = match path_opt {
                    Some(p) => p.to_owned(),
                    None => format!("<unresolved path: {path_raw}>"),
                };
                let label = if surface_unclassified {
                    "UNCLASSIFIED-SURFACE".to_owned()
                } else if is_unclassified {
                    format!("UNCLASSIFIED-METHOD({method})")
                } else {
                    method.to_uppercase()
                };
                uncovered_handlers.push((label, path_disp, handler.to_owned()));
            }
        }

        // Composition fail-closed (item 4): an unresolved `.merge(X)` / `.nest(p, X)` sub-router this
        // file did not scan-and-clear can carry unauthenticated mutating routes the gate cannot see.
        let has_unresolved_subrouter = !subrouters.is_empty();

        if !is_control_plane && !has_unresolved_subrouter {
            continue;
        }
        // The surface is COVERED (skippable) iff it carries NO unresolved-path / unclassified-method /
        // unclassified-surface route AND NO unresolved sub-router AND (a router-level auth layer
        // guards the whole scope OR every control-plane route is individually covered). A router-level
        // auth layer does NOT excuse an unresolved/unclassified route or an unresolved sub-router —
        // those are recognition failures, not coverage facts — so the structural finding still fires.
        if !has_unresolved_path
            && !has_unclassified_method
            && !has_unclassified_surface
            && !has_unresolved_subrouter
            && (has_auth_layer || uncovered_handlers.is_empty())
        {
            continue;
        }

        // An uncovered (or unparseable) control-plane surface. Key it by stable signature (M2).
        let key = surface_signature_key(file, scope, &routes, &subrouters);
        live_unauth_keys.insert(key.clone());

        // Frozen-baseline ratchet: a known pre-existing surface is ACCEPTED (no block).
        if frozen_baseline.contains(&key) {
            continue;
        }

        let holes = uncovered_handlers
            .iter()
            .map(|(m, p, h)| format!("{m} {p} -> {h}()"))
            .collect::<Vec<_>>()
            .join("; ");
        let subrouter_list = subrouters
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join("; ");
        // Pick the most specific structural code: an unresolved sub-router (composition we cannot see
        // into) and a structurally-unclassified call are the deepest discovery failures, then an
        // unresolved path, then an unclassified method, else the plain unauthenticated control plane.
        let (code, detail) = if has_unresolved_subrouter {
            (
                "AC-UNRESOLVED-SUBROUTER",
                format!(
                    "UNRESOLVED sub-router composition (fail-closed): the router scope `{scope}` at {file}:{router_line} composes a sub-router via `.merge(...)`/`.nest(...)`/`.nest_service(...)` the gate could not resolve to a router it scanned-and-cleared (a call into another crate/module, a function-returned router, or a macro) — so its routes' authz coverage is unknown. The gate does not assume merged/nested content is covered. Resolve the sub-router inline in a scanned scope, or add a router-level auth layer covering the whole scope, and ensure its mutating routes carry fail-closed authz — see {REMEDIATION_DOCTRINE}. Unresolved composition(s): [{subrouter_list}].{}",
                    if holes.is_empty() {
                        String::new()
                    } else {
                        format!(" Uncovered direct route(s): [{holes}].")
                    }
                ),
            )
        } else if has_unclassified_surface {
            (
                "AC-UNCLASSIFIED-SURFACE",
                format!(
                    "UNCLASSIFIED route-introduction call (fail-closed): the router scope `{scope}` at {file}:{router_line} mounts a `.route(...)`/`.route_service(...)` whose call shape matched NEITHER the axum `(path, METHOD(handler))` grammar NOR the owned-kernel `(HttpMethod::X, path, handler)` grammar (a macro-generated or structurally-novel route shape). The gate cannot extract its method/path/authz, so it fails closed as a potentially-mutating control plane. Use a recognized route-introduction shape, or add a router-level auth layer + per-handler authz — see {REMEDIATION_DOCTRINE}. Uncovered route(s): [{holes}]."
                ),
            )
        } else if has_unresolved_path {
            (
                "AC-UNRESOLVED-ROUTE-PATH",
                format!(
                    "UNRESOLVED route path (fail-closed): the router scope `{scope}` at {file}:{router_line} mounts a `.route(...)` whose path argument the gate could not resolve to a concrete string (a `const`/`static` it could not substitute, or a non-literal path expression). The gate cannot prove this surface is non-mutating or authz-covered, so it is treated as an unknown-authz control plane. Make the path a literal or a resolvable `const NAME: &str = \"...\";`, and add fail-closed authz — see {REMEDIATION_DOCTRINE}. Uncovered route(s): [{holes}]."
                ),
            )
        } else if has_unclassified_method {
            (
                "AC-UNCLASSIFIED-METHOD",
                format!(
                    "UNCLASSIFIED method-router (fail-closed): the router scope `{scope}` at {file}:{router_line} mounts a `.route(...)` whose method-router the gate could not classify (a `let`-bound MethodRouter it could not resolve, or an unrecognized call shape). It is treated as a potentially-mutating control plane requiring authz. Use an inline `get/post/.../on(MethodFilter::X, h)` shape or ensure the binding is resolvable, and add fail-closed authz — see {REMEDIATION_DOCTRINE}. Uncovered route(s): [{holes}]."
                ),
            )
        } else {
            (
                "AC-UNAUTHENTICATED-CONTROL-PLANE",
                format!(
                    "NEW unauthenticated HTTP control plane: the router scope `{scope}` at {file}:{router_line} mounts mutating route(s) [{holes}] whose handler(s) derive no caller identity (no recognized authz guard in the handler body and no router-level auth layer). Any network caller can invoke these writes. Add fail-closed authz before merge — see {REMEDIATION_DOCTRINE}. If a route is a genuinely unauthenticated read, declare its path in `exempt_path_substrings` (DATA)."
                ),
            )
        };
        findings.insert(Finding::new(code, &key, detail));
    }

    // Shrink-only self-clean: a frozen-baseline key with no live finding is stale.
    for key in &frozen_baseline {
        if !live_unauth_keys.contains(key) {
            findings.insert(Finding::new(
                "AC-STALE-BASELINE",
                key,
                format!(
                    "frozen-baseline surface key `{key}` matched no live unauthenticated finding (the surface was fixed, removed, or moved). Remove it from `frozen_unauthenticated_surfaces` in the policy — the baseline is shrink-only."
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

/// The per-surface finding codes whose keys are the BASELINE vocabulary (file+signature keys). The
/// policy-level codes (`AC-EMPTY-SCAN`, `AC-POLICY-*`, `AC-STALE-BASELINE`) are NOT baseline keys.
pub const SURFACE_FINDING_CODES: [&str; 5] = [
    "AC-UNAUTHENTICATED-CONTROL-PLANE",
    "AC-UNRESOLVED-ROUTE-PATH",
    "AC-UNCLASSIFIED-METHOD",
    "AC-UNCLASSIFIED-SURFACE",
    "AC-UNRESOLVED-SUBROUTER",
];

/// Regenerate the frozen-baseline signature keys from the live observation (the AUTOMATED property:
/// re-baselining is mechanical, not hand-edited). Returns the sorted set of per-surface finding keys
/// that the gate WOULD block against an EMPTY baseline — i.e. every currently-detected uncovered /
/// unparseable control-plane surface. `--write` substitutes these into
/// `frozen_unauthenticated_surfaces`, freezing today's surfaces so only NEW ones block.
pub fn baseline_keys(policy: &Value, observed: &Value) -> Vec<String> {
    live_surface_keys(policy, observed).into_iter().collect()
}

/// The set of per-surface finding keys the gate WOULD block against an EMPTY baseline — every
/// currently-detected uncovered / unparseable control-plane surface.
fn live_surface_keys(policy: &Value, observed: &Value) -> BTreeSet<String> {
    let mut p = policy.clone();
    p["frozen_unauthenticated_surfaces"] = json!([]);
    let mut keys: BTreeSet<String> = BTreeSet::new();
    for finding in evaluate_keyed(&p, observed) {
        if SURFACE_FINDING_CODES.contains(&finding.code.as_str()) {
            keys.insert(finding.key);
        }
    }
    keys
}

/// The SHRINK-ONLY re-baseline result for `--write` (MAJOR-1). Given the PRIOR committed baseline,
/// the regenerated baseline:
/// - DROPS any prior key with no live finding (a fixed/removed surface — shrink).
/// - KEEPS every prior key that is still live.
/// - ABSORBS a NEW live key (one absent from the prior baseline) ONLY when `allow_new` is true.
///
/// Without `--allow-new`, a `--write` therefore can only ever SHRINK the baseline; it refuses to
/// silently grow it (the old `--write` regenerated from ALL live findings with no new-vs-existing
/// discrimination, so a brand-new unauthenticated control plane could be absorbed by a careless
/// re-baseline). Returns `(next_baseline_sorted, new_keys_sorted)` — `new_keys` is the set of live
/// keys absent from the prior baseline (printed by the binary so a grower is always reviewed).
pub fn shrink_only_baseline(
    policy: &Value,
    observed: &Value,
    allow_new: bool,
) -> (Vec<String>, Vec<String>) {
    let prior: BTreeSet<String> = string_list(policy, "frozen_unauthenticated_surfaces")
        .into_iter()
        .collect();
    let live = live_surface_keys(policy, observed);

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
        return "authz-coverage gate passed: every NEW HTTP control-plane surface carries fail-closed authz (router-level auth layer or per-handler authz guard); no new unauthenticated mutating router".to_owned();
    }
    let mut out = String::from("authz-coverage gate failed (issue #770 / AUTH-005 class):\n");
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
            "min_expected_surfaces": 0,
            "scan_roots": ["src"],
            "excluded_dir_names": ["target", "third-party"],
            "auth_layer_idents": ["RequireAuth", "AuthLayer", "require_principal"],
            "authz_guard_idents": [
                "authorize", "admin_tenant_allowed", "authenticate_caller",
                ".decide(", "require_data_plane_bearer", "authorize_token_for",
                "authorize_with_token", "constant_time_eq"
            ],
            "exempt_path_substrings": ["/healthz", "/livez", "/readyz", "/metrics"],
            "frozen_unauthenticated_surfaces": []
        })
    }

    // A synthetic unauthenticated mutating router (the RED exhibit): a DELETE/POST with no authz
    // call in either handler body, no auth layer.
    const RED_FIXTURE: &str = r#"
        use axum::{Router, routing::{post, delete, get}};

        async fn create_thing() -> StatusCode { StatusCode::OK }
        async fn delete_thing() -> StatusCode { StatusCode::NO_CONTENT }
        async fn healthz() -> StatusCode { StatusCode::OK }

        pub fn build_router() -> Router {
            Router::new()
                .route("/things", post(create_thing))
                .route("/things/{id}", delete(delete_thing))
                .route("/healthz", get(healthz))
                .with_state(())
        }
    "#;

    // A GREEN exhibit mirroring tenancy: every mutating handler calls authorize(...).
    const GREEN_PER_HANDLER: &str = r#"
        use axum::{Router, routing::{post, delete, get}};

        fn authorize(s: &S, h: &H, a: A) -> Result<(), E> { Ok(()) }

        async fn register_tenant() -> StatusCode {
            authorize(&state, &headers, Action::Register)?;
            StatusCode::OK
        }
        async fn retire_tenant() -> StatusCode {
            authorize(&state, &headers, Action::Retire)?;
            StatusCode::NO_CONTENT
        }
        async fn healthz() -> StatusCode { StatusCode::OK }

        pub fn build_router() -> Router {
            Router::new()
                .route("/v1/tenants", post(register_tenant))
                .route("/v1/tenants/{id}", delete(retire_tenant))
                .route("/healthz", get(healthz))
                .with_state(state)
        }
    "#;

    // A GREEN exhibit mirroring intelligence/adapters/rest: admin_tenant_allowed / bearer guard
    // in the mutating handler bodies, plus a non-auth .layer(DefaultBodyLimit) that must NOT be
    // mistaken for an auth layer.
    const GREEN_GUARD_AND_BODY_LIMIT_LAYER: &str = r#"
        use axum::{Router, routing::{post, get}};

        fn admin_tenant_allowed(h: &HeaderMap, t: &str) -> bool { true }
        fn require_data_plane_bearer(s: &S, h: &HeaderMap) -> Result<(), R> { Ok(()) }

        async fn handle_proxy() -> Response {
            if let Err(r) = require_data_plane_bearer(&state, &headers) { return r; }
            ok()
        }
        async fn handle_admin_resume(headers: HeaderMap) -> Response {
            if !admin_tenant_allowed(&headers, state.tenant_id.as_str()) { return deny(); }
            ok()
        }

        pub fn build_router() -> Router {
            Router::new()
                .route("/v1/messages", post(handle_proxy))
                .route("/admin/v1/resume", post(handle_admin_resume))
                .route("/healthz", get(handle_healthz))
                .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        }
    "#;

    fn observe(text: &str) -> Value {
        let p = policy();
        let auth_layers = string_list(&p, "auth_layer_idents");
        let guards = string_list(&p, "authz_guard_idents");
        let surfaces = extract_surfaces("fixture.rs", text, &auth_layers, &guards);
        json!({ "surfaces_found": surfaces.len(), "surfaces": surfaces })
    }

    #[test]
    fn red_on_unauthenticated_mutating_router() {
        let observed = observe(RED_FIXTURE);
        let findings = evaluate_keyed(&policy(), &observed);
        assert!(
            findings
                .iter()
                .any(|f| f.code == "AC-UNAUTHENTICATED-CONTROL-PLANE"),
            "an unauthenticated POST/DELETE router must produce AC-UNAUTHENTICATED-CONTROL-PLANE: {findings:?}"
        );
        let finding = findings
            .iter()
            .find(|f| f.code == "AC-UNAUTHENTICATED-CONTROL-PLANE")
            .unwrap();
        assert!(
            finding.detail.contains("DELETE /things/{id}"),
            "names the delete hole: {finding:?}"
        );
        assert!(
            finding.detail.contains("POST /things"),
            "names the post hole: {finding:?}"
        );
        assert!(
            finding.detail.contains("intelligence/adapters/rest"),
            "remediation must point at the doctrine: {finding:?}"
        );
        assert_eq!(evaluate(&policy(), &observed).verdict, Verdict::Red);
    }

    #[test]
    fn green_on_per_handler_authorize_pattern() {
        let observed = observe(GREEN_PER_HANDLER);
        let findings = evaluate_keyed(&policy(), &observed);
        assert!(
            findings.is_empty(),
            "every mutating handler calls authorize() ⇒ no finding: {findings:?}"
        );
        assert_eq!(evaluate(&policy(), &observed).verdict, Verdict::Green);
    }

    #[test]
    fn green_on_guard_handlers_despite_non_auth_body_limit_layer() {
        let observed = observe(GREEN_GUARD_AND_BODY_LIMIT_LAYER);
        // The .layer(DefaultBodyLimit) must NOT be treated as an auth layer; coverage comes from
        // the per-handler guards (require_data_plane_bearer / admin_tenant_allowed).
        let surface = observed["surfaces"].as_array().unwrap().first().unwrap();
        assert_eq!(
            surface["has_auth_layer"],
            Value::from(false),
            "DefaultBodyLimit is not an auth layer"
        );
        let findings = evaluate_keyed(&policy(), &observed);
        assert!(
            findings.is_empty(),
            "per-handler guards cover the surface: {findings:?}"
        );
    }

    #[test]
    fn health_only_router_is_not_a_control_plane() {
        let text = r#"
            pub fn r() -> Router {
                Router::new()
                    .route("/healthz", get(healthz))
                    .route("/metrics", get(metrics))
                    .with_state(())
            }
        "#;
        let observed = observe(text);
        // It has no mutating route at all ⇒ never a control plane.
        let findings = evaluate_keyed(&policy(), &observed);
        assert!(
            findings.is_empty(),
            "a read-only health router is not a control plane: {findings:?}"
        );
    }

    #[test]
    fn router_level_auth_layer_covers_the_surface() {
        let text = r#"
            async fn create_thing() -> StatusCode { StatusCode::OK }
            pub fn r() -> Router {
                Router::new()
                    .route("/things", post(create_thing))
                    .layer(RequireAuth::new(verifier))
                    .with_state(())
            }
        "#;
        let observed = observe(text);
        let surface = observed["surfaces"].as_array().unwrap().first().unwrap();
        assert_eq!(
            surface["has_auth_layer"],
            Value::from(true),
            "RequireAuth is an auth layer"
        );
        let findings = evaluate_keyed(&policy(), &observed);
        assert!(
            findings.is_empty(),
            "a router-level auth layer covers all routes: {findings:?}"
        );
    }

    #[test]
    fn frozen_baseline_accepts_known_surface_blocks_new_one() {
        // Two unauthenticated surfaces: one in the baseline (accepted), one not (blocked).
        let text = r#"
            async fn a() -> StatusCode { StatusCode::OK }
            pub fn r() -> Router {
                Router::new().route("/a", post(a)).with_state(())
            }
        "#;
        let observed = observe(text);
        // The signature key is derived from the finding itself (M2: file + sorted (method,path)
        // tuples, line-independent), not a `router@<line>` literal.
        let blocked: Vec<Finding> = evaluate_keyed(&policy(), &observed)
            .into_iter()
            .filter(|f| f.code == "AC-UNAUTHENTICATED-CONTROL-PLANE")
            .collect();
        assert_eq!(
            blocked.len(),
            1,
            "an un-baselined unauthenticated surface blocks: {blocked:?}"
        );
        let key = blocked[0].key.clone();
        assert_eq!(
            key, "fixture.rs#r::router[post /a]",
            "signature key is line-independent + scope-keyed: {key}"
        );

        // With the key frozen ⇒ accepted (no block), no stale-baseline (it is live).
        let mut p = policy();
        p["frozen_unauthenticated_surfaces"] = json!([key]);
        let findings = evaluate_keyed(&p, &observed);
        assert!(
            findings.is_empty(),
            "a baselined surface is accepted and not stale: {findings:?}"
        );
    }

    #[test]
    fn stale_baseline_entry_self_cleans() {
        let observed = observe(GREEN_PER_HANDLER); // no live unauth findings
        let mut p = policy();
        p["frozen_unauthenticated_surfaces"] = json!(["some/old/file.rs::router@10"]);
        let findings = evaluate_keyed(&p, &observed);
        assert!(
            findings
                .iter()
                .any(|f| f.code == "AC-STALE-BASELINE" && f.key == "some/old/file.rs::router@10"),
            "a baseline key with no live finding must self-clean: {findings:?}"
        );
    }

    #[test]
    fn empty_scan_fails_closed() {
        let mut p = policy();
        p["min_expected_surfaces"] = json!(5);
        let observed = json!({ "surfaces_found": 0, "surfaces": [] });
        let findings = evaluate_keyed(&p, &observed);
        assert!(
            findings.iter().any(|f| f.code == "AC-EMPTY-SCAN"),
            "a below-floor surface census must trip AC-EMPTY-SCAN: {findings:?}"
        );
    }

    #[test]
    fn policy_gate_id_mismatch_fails_closed() {
        let mut p = policy();
        p["gate_id"] = Value::from("wrong-id");
        let observed = observe(GREEN_PER_HANDLER);
        let findings = evaluate_keyed(&p, &observed);
        assert!(
            findings
                .iter()
                .any(|f| f.code == "AC-POLICY-GATE-ID-MISMATCH")
        );
    }

    #[test]
    fn malformed_policy_fails_closed() {
        let observed = observe(GREEN_PER_HANDLER);
        let p = json!({ "gate_id": GATE_ID, "scan_roots": ["src"] }); // no authz_guard_idents
        let findings = evaluate_keyed(&p, &observed);
        assert!(
            findings.iter().any(|f| f.code == "AC-POLICY-MALFORMED"),
            "a missing authz_guard_idents must fail closed: {findings:?}"
        );
    }

    #[test]
    fn parses_multiline_route_with_turbofish_handler() {
        // The intelligence/adapters/rest + tenancy real shape: multiline .route() and ::<S> handler.
        let text = r#"
            async fn register_tenant() -> R { authorize(&s,&h,A)?; ok() }
            pub fn r() -> Router {
                Router::new()
                    .route(
                        "/v1/tenants/{id}/suspend",
                        post(register_tenant::<S>),
                    )
                    .with_state(state)
            }
        "#;
        let observed = observe(text);
        let surface = observed["surfaces"].as_array().unwrap().first().unwrap();
        let routes = surface["routes"].as_array().unwrap();
        assert_eq!(routes.len(), 1, "one route parsed: {routes:?}");
        assert_eq!(routes[0]["method"], "post");
        assert_eq!(
            routes[0]["handler"], "register_tenant",
            "turbofish stripped"
        );
        assert_eq!(routes[0]["path"], "/v1/tenants/{id}/suspend");
        // Handler calls authorize ⇒ covered ⇒ green.
        assert!(evaluate_keyed(&policy(), &observed).is_empty());
    }

    #[test]
    fn evaluate_is_bare_projection() {
        let observed = observe(RED_FIXTURE);
        let projected: BTreeSet<String> = evaluate_keyed(&policy(), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert_eq!(evaluate(&policy(), &observed).violations, projected);
    }

    // =====================================================================
    // Review B1/B2/B3 fail-closed fixtures. Each RED fixture is the adversarial
    // bypass; each must now produce a finding (RED). Each GREEN fixture is the
    // authenticated / resolvable counterpart and must PASS (no finding).
    // =====================================================================

    fn has_code(text: &str, code: &str) -> bool {
        let observed = observe(text);
        evaluate_keyed(&policy(), &observed)
            .iter()
            .any(|f| f.code == code)
    }

    fn is_green(text: &str) -> BTreeSet<Finding> {
        let observed = observe(text);
        evaluate_keyed(&policy(), &observed)
    }

    // ---- B1: const/static route path ----------------------------------------

    // RED: a const-path unauthenticated mutating DELETE — the exact reproduced bypass. The path is
    // a `const`, the engine resolves it, classifies the surface as a control plane, finds no guard.
    const RED_B1_CONST_PATH: &str = r#"
        const NUKE: &str = "/tenants/{id}";
        async fn nuke_tenant() -> StatusCode { StatusCode::NO_CONTENT }
        pub fn build_router() -> Router {
            Router::new().route(NUKE, delete(nuke_tenant)).with_state(())
        }
    "#;

    #[test]
    fn b1_const_path_unauthenticated_mutating_route_is_red() {
        // Const resolves to a concrete path ⇒ classified as a control plane ⇒ uncovered ⇒ RED.
        assert!(
            has_code(RED_B1_CONST_PATH, "AC-UNAUTHENTICATED-CONTROL-PLANE"),
            "a const-path unauthenticated DELETE must be RED: {:?}",
            is_green(RED_B1_CONST_PATH)
        );
        // The resolved path appears in the route observation (proving const substitution).
        let observed = observe(RED_B1_CONST_PATH);
        let route = &observed["surfaces"][0]["routes"][0];
        assert_eq!(
            route["path"], "/tenants/{id}",
            "const NUKE substituted to its value"
        );
        assert_eq!(route["method"], "delete");
    }

    // RED: an UNRESOLVABLE path (a const with no string-literal initializer) ⇒ fail-closed.
    const RED_B1_UNRESOLVED_PATH: &str = r#"
        const NUKE: &str = some_other_const();
        async fn nuke_tenant() -> StatusCode { StatusCode::NO_CONTENT }
        pub fn build_router() -> Router {
            Router::new().route(NUKE, delete(nuke_tenant)).with_state(())
        }
    "#;

    #[test]
    fn b1_unresolvable_path_fails_closed() {
        assert!(
            has_code(RED_B1_UNRESOLVED_PATH, "AC-UNRESOLVED-ROUTE-PATH"),
            "an unresolvable route path must fail closed with AC-UNRESOLVED-ROUTE-PATH: {:?}",
            is_green(RED_B1_UNRESOLVED_PATH)
        );
        let observed = observe(RED_B1_UNRESOLVED_PATH);
        assert_eq!(observed["surfaces"][0]["routes"][0]["path"], Value::Null);
    }

    // GREEN: a resolvable-const path WITH a real guard in the handler body.
    const GREEN_B1_CONST_PATH_GUARDED: &str = r#"
        const NUKE: &str = "/tenants/{id}";
        async fn nuke_tenant(headers: HeaderMap) -> StatusCode {
            authorize(&state, &headers, Action::Retire)?;
            StatusCode::NO_CONTENT
        }
        pub fn build_router() -> Router {
            Router::new().route(NUKE, delete(nuke_tenant)).with_state(())
        }
    "#;

    #[test]
    fn b1_const_path_with_guard_is_green() {
        assert!(
            is_green(GREEN_B1_CONST_PATH_GUARDED).is_empty(),
            "a resolvable-const path with a real authorize() guard must PASS: {:?}",
            is_green(GREEN_B1_CONST_PATH_GUARDED)
        );
    }

    // ---- B2: non-METHOD(handler) method-router shapes ------------------------

    // RED: a MethodRouter bound to a variable, unauthenticated. The engine resolves the binding to
    // delete(h) (mutating) AND to its handler for the guard probe; the handler has no guard ⇒ RED.
    const RED_B2_METHOD_VAR: &str = r#"
        async fn delete_thing() -> StatusCode { StatusCode::NO_CONTENT }
        pub fn build_router() -> Router {
            let m = delete(delete_thing);
            Router::new().route("/x/{id}", m).with_state(())
        }
    "#;

    #[test]
    fn b2_method_router_variable_unauthenticated_is_red() {
        assert!(
            has_code(RED_B2_METHOD_VAR, "AC-UNAUTHENTICATED-CONTROL-PLANE"),
            "a variable-bound delete() with no guard must be RED: {:?}",
            is_green(RED_B2_METHOD_VAR)
        );
        // The binding resolved the method to `delete` (mutating), not unclassified.
        let observed = observe(RED_B2_METHOD_VAR);
        assert_eq!(
            observed["surfaces"][0]["routes"][0]["method_class"],
            "mutating"
        );
    }

    // RED: a TRULY unresolvable method-router variable (bound to a non-method-router expression) ⇒
    // fail-closed AC-UNCLASSIFIED-METHOD.
    const RED_B2_UNCLASSIFIED_METHOD: &str = r#"
        pub fn build_router() -> Router {
            let m = build_method_router();
            Router::new().route("/x/{id}", m).with_state(())
        }
    "#;

    #[test]
    fn b2_unresolvable_method_router_fails_closed() {
        assert!(
            has_code(RED_B2_UNCLASSIFIED_METHOD, "AC-UNCLASSIFIED-METHOD"),
            "an unresolvable method-router variable must fail closed: {:?}",
            is_green(RED_B2_UNCLASSIFIED_METHOD)
        );
        assert_eq!(
            observe(RED_B2_UNCLASSIFIED_METHOD)["surfaces"][0]["routes"][0]["method_class"],
            "unclassified"
        );
    }

    // RED: on(MethodFilter::DELETE, h) — the documented axum API — unauthenticated.
    const RED_B2_ON_FILTER: &str = r#"
        async fn delete_thing() -> StatusCode { StatusCode::NO_CONTENT }
        pub fn build_router() -> Router {
            Router::new()
                .route("/x/{id}", on(MethodFilter::DELETE, delete_thing))
                .with_state(())
        }
    "#;

    #[test]
    fn b2_on_method_filter_delete_unauthenticated_is_red() {
        assert!(
            has_code(RED_B2_ON_FILTER, "AC-UNAUTHENTICATED-CONTROL-PLANE"),
            "on(MethodFilter::DELETE, h) with no guard must be RED: {:?}",
            is_green(RED_B2_ON_FILTER)
        );
        let observed = observe(RED_B2_ON_FILTER);
        assert_eq!(
            observed["surfaces"][0]["routes"][0]["method_class"],
            "mutating"
        );
        assert_eq!(
            observed["surfaces"][0]["routes"][0]["handler"],
            "delete_thing"
        );
    }

    // GREEN: on(MethodFilter::GET, h) is non-mutating ⇒ not a control plane.
    const GREEN_B2_ON_FILTER_GET: &str = r#"
        async fn read_thing() -> StatusCode { StatusCode::OK }
        pub fn build_router() -> Router {
            Router::new()
                .route("/x/{id}", on(MethodFilter::GET, read_thing))
                .with_state(())
        }
    "#;

    #[test]
    fn b2_on_method_filter_get_is_green() {
        assert!(
            is_green(GREEN_B2_ON_FILTER_GET).is_empty(),
            "on(MethodFilter::GET, h) is a read, not a control plane: {:?}",
            is_green(GREEN_B2_ON_FILTER_GET)
        );
    }

    // GREEN: a variable-bound delete WITH a guard in the handler.
    const GREEN_B2_METHOD_VAR_GUARDED: &str = r#"
        async fn delete_thing(headers: HeaderMap) -> StatusCode {
            authorize(&state, &headers, Action::Delete)?;
            StatusCode::NO_CONTENT
        }
        pub fn build_router() -> Router {
            let m = delete(delete_thing);
            Router::new().route("/x/{id}", m).with_state(())
        }
    "#;

    #[test]
    fn b2_method_router_variable_with_guard_is_green() {
        assert!(
            is_green(GREEN_B2_METHOD_VAR_GUARDED).is_empty(),
            "a variable-bound delete() whose handler calls authorize() must PASS: {:?}",
            is_green(GREEN_B2_METHOD_VAR_GUARDED)
        );
    }

    // ---- B3: comment/string-stripped coverage probe -------------------------

    // RED: a handler whose ONLY "guard" is in a comment and a string literal. The code-only view
    // strips both ⇒ no real guard ⇒ RED.
    const RED_B3_COMMENT_ONLY_GUARD: &str = r#"
        async fn delete_thing() -> StatusCode {
            // TODO: call authorize() later once the PDP is wired
            let msg = "should authorize before deleting";
            StatusCode::NO_CONTENT
        }
        pub fn build_router() -> Router {
            Router::new().route("/x/{id}", delete(delete_thing)).with_state(())
        }
    "#;

    #[test]
    fn b3_comment_only_guard_fails_closed() {
        assert!(
            has_code(
                RED_B3_COMMENT_ONLY_GUARD,
                "AC-UNAUTHENTICATED-CONTROL-PLANE"
            ),
            "a guard ident only in a comment/string must NOT false-cover (must be RED): {:?}",
            is_green(RED_B3_COMMENT_ONLY_GUARD)
        );
    }

    // GREEN: the same handler with a REAL authorize() call in code ⇒ covered.
    const GREEN_B3_REAL_GUARD: &str = r#"
        async fn delete_thing(headers: HeaderMap) -> StatusCode {
            // authorize first
            authorize(&state, &headers, Action::Delete)?;
            StatusCode::NO_CONTENT
        }
        pub fn build_router() -> Router {
            Router::new().route("/x/{id}", delete(delete_thing)).with_state(())
        }
    "#;

    #[test]
    fn b3_real_guard_in_code_is_green() {
        assert!(
            is_green(GREEN_B3_REAL_GUARD).is_empty(),
            "a real authorize() call in code (not just a comment) must PASS: {:?}",
            is_green(GREEN_B3_REAL_GUARD)
        );
    }

    // M2: the signature key is line-independent — inserting blank lines above the router must NOT
    // change the finding key (the old router@<line> key would have changed).
    #[test]
    fn m2_signature_key_is_line_independent() {
        let base = r#"
            async fn a() -> StatusCode { StatusCode::OK }
            pub fn r() -> Router { Router::new().route("/a", post(a)).with_state(()) }
        "#;
        let shifted = format!("\n\n\n\n{base}");
        let k1 = evaluate_keyed(&policy(), &observe(base))
            .into_iter()
            .find(|f| f.code == "AC-UNAUTHENTICATED-CONTROL-PLANE")
            .map(|f| f.key);
        let k2 = evaluate_keyed(&policy(), &observe(&shifted))
            .into_iter()
            .find(|f| f.code == "AC-UNAUTHENTICATED-CONTROL-PLANE")
            .map(|f| f.key);
        assert_eq!(
            k1, k2,
            "the signature key must not depend on the router's line number"
        );
        assert_eq!(k1.as_deref(), Some("fixture.rs#r::router[post /a]"));
    }

    // =====================================================================
    // PR #780 SECOND-PASS review: surface-DISCOVERY fail-closed fixtures. The
    // first pass anchored discovery on the literal `Router::new()` token + a
    // chain truncated at `.with_state(`; an adversarial re-review bypassed the
    // gate FIVE ways. Each RED fixture below is one reproduced bypass and must
    // now be RED; each CONTROL proves no false-cover; each GREEN proves a legit
    // covered router still passes.
    // =====================================================================

    // ---- BLOCKER-1: discovery anchored on the route-introduction call, not Router::new() ----------

    // RED: `Router::default().route(...)` — never produced by `Router::new()`, was invisible before.
    const RED_S1_ROUTER_DEFAULT: &str = r#"
        async fn h() -> StatusCode { StatusCode::NO_CONTENT }
        pub fn build() -> Router {
            Router::default().route("/v1/tenants/{id}", delete(h)).with_state(())
        }
    "#;

    #[test]
    fn s1_router_default_unauthenticated_is_red() {
        assert!(
            has_code(RED_S1_ROUTER_DEFAULT, "AC-UNAUTHENTICATED-CONTROL-PLANE"),
            "a Router::default().route(...) unauthenticated DELETE must be RED: {:?}",
            is_green(RED_S1_ROUTER_DEFAULT)
        );
    }

    // RED: a route declared on a `Router` PARAMETER in a helper fn (no constructor in this fn at all).
    const RED_S1_ROUTER_PARAM: &str = r#"
        async fn h() -> StatusCode { StatusCode::OK }
        pub fn add(r: Router) -> Router {
            r.route("/admin/v1/create", post(h))
        }
    "#;

    #[test]
    fn s1_router_parameter_helper_unauthenticated_is_red() {
        assert!(
            has_code(RED_S1_ROUTER_PARAM, "AC-UNAUTHENTICATED-CONTROL-PLANE"),
            "a `fn add(r: Router)->Router {{ r.route(...) }}` unauthenticated POST must be RED: {:?}",
            is_green(RED_S1_ROUTER_PARAM)
        );
        // The surface keys to the helper fn scope.
        let observed = observe(RED_S1_ROUTER_PARAM);
        assert_eq!(observed["surfaces"][0]["scope"], "add");
    }

    // ---- BLOCKER-3: routes declared AFTER the `.with_state(` constructor chain ----------------------

    // RED: a `.with_state` chain followed by a SECOND `.route(` on the bound variable. The old chain
    // truncated at `.with_state(` and dropped the later DELETE; discovery now collects ALL route calls.
    const RED_S3_ROUTE_AFTER_WITH_STATE: &str = r#"
        async fn hz() -> StatusCode { StatusCode::OK }
        async fn nuke() -> StatusCode { StatusCode::NO_CONTENT }
        pub fn build() -> Router {
            let b = Router::new().route("/healthz", get(hz)).with_state(());
            b.route("/tenants/{id}", delete(nuke))
        }
    "#;

    #[test]
    fn s3_route_after_with_state_is_red() {
        assert!(
            has_code(
                RED_S3_ROUTE_AFTER_WITH_STATE,
                "AC-UNAUTHENTICATED-CONTROL-PLANE"
            ),
            "an unauthenticated DELETE declared AFTER .with_state(...) must be RED: {:?}",
            is_green(RED_S3_ROUTE_AFTER_WITH_STATE)
        );
        // Both routes (the healthz read AND the delete) are in the one scope's signature.
        let f = is_green(RED_S3_ROUTE_AFTER_WITH_STATE)
            .into_iter()
            .find(|f| f.code == "AC-UNAUTHENTICATED-CONTROL-PLANE")
            .unwrap();
        assert!(
            f.detail.contains("DELETE /tenants/{id}"),
            "the post-with_state DELETE is seen: {f:?}"
        );
    }

    // ---- BLOCKER-4: auth-layer whole-token match (no substring false-cover) -------------------------

    // CONTROL (must be RED): a layer named `RequireAuthMetricsRecorder` substring-matches the
    // `RequireAuth` auth-layer ident but does NOT authenticate. Its routes must NOT be covered.
    const RED_S4_AUTH_LAYER_SUBSTRING: &str = r#"
        async fn create_thing() -> StatusCode { StatusCode::OK }
        pub fn build() -> Router {
            Router::new()
                .route("/v1/things", post(create_thing))
                .layer(RequireAuthMetricsRecorder::new())
                .with_state(())
        }
    "#;

    #[test]
    fn s4_auth_layer_substring_does_not_false_cover() {
        assert!(
            has_code(
                RED_S4_AUTH_LAYER_SUBSTRING,
                "AC-UNAUTHENTICATED-CONTROL-PLANE"
            ),
            "RequireAuthMetricsRecorder must NOT satisfy the RequireAuth auth-layer ident: {:?}",
            is_green(RED_S4_AUTH_LAYER_SUBSTRING)
        );
        let observed = observe(RED_S4_AUTH_LAYER_SUBSTRING);
        assert_eq!(
            observed["surfaces"][0]["has_auth_layer"],
            Value::from(false),
            "the substring layer is not an auth layer"
        );
    }

    // GREEN: a REAL `RequireAuth` layer (whole token) covers the surface.
    const GREEN_S4_REAL_AUTH_LAYER: &str = r#"
        async fn create_thing() -> StatusCode { StatusCode::OK }
        pub fn build() -> Router {
            Router::new()
                .route("/v1/things", post(create_thing))
                .layer(RequireAuth::new(verifier))
                .with_state(())
        }
    "#;

    #[test]
    fn s4_real_auth_layer_is_green() {
        assert!(
            is_green(GREEN_S4_REAL_AUTH_LAYER).is_empty(),
            "a real RequireAuth layer (whole token) must cover the surface: {:?}",
            is_green(GREEN_S4_REAL_AUTH_LAYER)
        );
    }

    // ---- BLOCKER-5: guard whole-token + call-shape match (no substring false-cover) -----------------

    // CONTROL (must be RED): a handler whose only authz-ish reference is `unauthorized_response()`,
    // which substring-matches the `authorize` guard ident but does NO authz.
    const RED_S5_GUARD_SUBSTRING: &str = r#"
        fn unauthorized_response() -> StatusCode { StatusCode::UNAUTHORIZED }
        async fn nuke() -> StatusCode {
            let _ = unauthorized_response();
            StatusCode::NO_CONTENT
        }
        pub fn build() -> Router {
            Router::new().route("/v1/tenants/{id}", delete(nuke)).with_state(())
        }
    "#;

    #[test]
    fn s5_guard_substring_does_not_false_cover() {
        assert!(
            has_code(RED_S5_GUARD_SUBSTRING, "AC-UNAUTHENTICATED-CONTROL-PLANE"),
            "unauthorized_response() must NOT satisfy the `authorize` guard ident (substring): {:?}",
            is_green(RED_S5_GUARD_SUBSTRING)
        );
    }

    // GREEN: a real `authorize(...)` CALL (whole token + `(`) covers the handler.
    const GREEN_S5_REAL_GUARD_CALL: &str = r#"
        async fn nuke(headers: HeaderMap) -> StatusCode {
            authorize(&state, &headers, Action::Retire)?;
            StatusCode::NO_CONTENT
        }
        pub fn build() -> Router {
            Router::new().route("/v1/tenants/{id}", delete(nuke)).with_state(())
        }
    "#;

    #[test]
    fn s5_real_guard_call_is_green() {
        assert!(
            is_green(GREEN_S5_REAL_GUARD_CALL).is_empty(),
            "a real authorize() call must cover the handler: {:?}",
            is_green(GREEN_S5_REAL_GUARD_CALL)
        );
    }

    // ---- Composition fail-closed (item 4): unresolved .merge(...) / .nest(...) ----------------------

    // RED: a `.merge(X)` whose X is a function-returned sub-router this scope did not scan-and-clear.
    const RED_COMPOSITION_MERGE: &str = r#"
        async fn hz() -> StatusCode { StatusCode::OK }
        pub fn build() -> Router {
            Router::new()
                .route("/healthz", get(hz))
                .merge(admin_subrouter())
                .with_state(())
        }
    "#;

    #[test]
    fn composition_unresolved_merge_fails_closed() {
        assert!(
            has_code(RED_COMPOSITION_MERGE, "AC-UNRESOLVED-SUBROUTER"),
            "an unresolved .merge(subrouter()) must fail closed AC-UNRESOLVED-SUBROUTER: {:?}",
            is_green(RED_COMPOSITION_MERGE)
        );
    }

    // RED: a `.nest("/admin", X)` whose X is an unresolved sub-router.
    const RED_COMPOSITION_NEST: &str = r#"
        async fn hz() -> StatusCode { StatusCode::OK }
        pub fn build() -> Router {
            Router::new()
                .route("/healthz", get(hz))
                .nest("/admin", admin_subrouter())
                .with_state(())
        }
    "#;

    #[test]
    fn composition_unresolved_nest_fails_closed() {
        assert!(
            has_code(RED_COMPOSITION_NEST, "AC-UNRESOLVED-SUBROUTER"),
            "an unresolved .nest(path, subrouter()) must fail closed AC-UNRESOLVED-SUBROUTER: {:?}",
            is_green(RED_COMPOSITION_NEST)
        );
    }

    // ---- item 5: macro-generated / structurally-unclassifiable route shapes -------------------------

    // RED: a `.route(` with a STRING path (so it IS a recognized route-introduction) but a method
    // router the gate cannot classify (a macro expansion) → fail-closed AC-UNCLASSIFIED-METHOD, never
    // silently green.
    const RED_UNCLASSIFIED_METHOD_MACRO: &str = r#"
        pub fn build() -> Router {
            Router::new().route("/x/{id}", method_router_macro!(h)).with_state(())
        }
    "#;

    #[test]
    fn unclassified_method_macro_fails_closed() {
        let findings = is_green(RED_UNCLASSIFIED_METHOD_MACRO);
        assert!(
            findings
                .iter()
                .any(|f| SURFACE_FINDING_CODES.contains(&f.code.as_str())),
            "a string-path route with a macro method-router must fail closed: {findings:?}"
        );
    }

    // RED: a truncated/unbounded `.route(` (no balanced close paren) → fail-closed
    // AC-UNCLASSIFIED-SURFACE (a real builder route the parser could not bound — never dropped).
    const RED_UNCLASSIFIED_SURFACE_TRUNCATED: &str = "
        pub fn build() -> Router {
            Router::new().route(generated_macro_expansion
        }
    ";

    #[test]
    fn unclassified_surface_truncated_fails_closed() {
        assert!(
            has_code(
                RED_UNCLASSIFIED_SURFACE_TRUNCATED,
                "AC-UNCLASSIFIED-SURFACE"
            ),
            "an unbounded/truncated .route( must fail closed AC-UNCLASSIFIED-SURFACE: {:?}",
            is_green(RED_UNCLASSIFIED_SURFACE_TRUNCATED)
        );
    }

    // ---- Owned `oya-http-router-kernel` grammar `(HttpMethod::X, path, handler)` --------------------

    // RED: an owned-kernel POST with no guard.
    const RED_OWNED_KERNEL_POST: &str = r#"
        fn build(router: &mut Router<SyncHandler>) -> Result<(), RouterError> {
            router.route(HttpMethod::Post, "/admin/v1/provision", provision_handler)?;
            Ok(())
        }
    "#;

    #[test]
    fn owned_kernel_post_unauthenticated_is_red() {
        assert!(
            has_code(RED_OWNED_KERNEL_POST, "AC-UNAUTHENTICATED-CONTROL-PLANE"),
            "an owned-kernel (HttpMethod::Post, path, handler) with no guard must be RED: {:?}",
            is_green(RED_OWNED_KERNEL_POST)
        );
        let observed = observe(RED_OWNED_KERNEL_POST);
        assert_eq!(
            observed["surfaces"][0]["routes"][0]["method_class"],
            "mutating"
        );
        assert_eq!(
            observed["surfaces"][0]["routes"][0]["path"],
            "/admin/v1/provision"
        );
    }

    // GREEN: an owned-kernel GET-only router is a read, not a control plane.
    const GREEN_OWNED_KERNEL_GET: &str = r#"
        fn build(router: &mut Router<SyncHandler>) -> Result<(), RouterError> {
            router.route(HttpMethod::Get, "/healthz", health_handler)?;
            Ok(())
        }
    "#;

    #[test]
    fn owned_kernel_get_only_is_green() {
        assert!(
            is_green(GREEN_OWNED_KERNEL_GET).is_empty(),
            "an owned-kernel GET-only health router is not a control plane: {:?}",
            is_green(GREEN_OWNED_KERNEL_GET)
        );
    }

    // GREEN: an owned-kernel POST using the repo-owned typed Handler bridge. The route stores
    // `handler_to_sync(PolicyAdmissionHandler)`, so the detector must unwrap the bridge and inspect
    // the typed `impl Handler for PolicyAdmissionHandler { fn call(...) { ... } }` body. The handler
    // delegates to a real fail-closed tenant/body guard that performs a constant-time comparison.
    const GREEN_OWNED_KERNEL_HANDLER_TO_SYNC_TYPED_HANDLER: &str = r#"
        fn build(router: &mut Router<SyncHandler>) -> Result<(), RouterError> {
            router.route(
                HttpMethod::Post,
                "/tenant-rbac/v1/policy-admissions",
                handler_to_sync(PolicyAdmissionHandler),
            )?;
            Ok(())
        }

        struct PolicyAdmissionHandler;

        impl oya_http_middleware_kernel::Handler for PolicyAdmissionHandler {
            type Error = HttpResponse;

            fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
                let request: ServiceWriteAdmissionRequest = parse_json(&req.body)?;
                enforce_body_tenant_matches_verified(&req, &request.tenant_id)?;
                Ok(HttpResponse::new(202))
            }
        }

        fn enforce_body_tenant_matches_verified(
            req: &HttpRequest,
            body_tenant_id: &str,
        ) -> Result<(), HttpResponse> {
            if constant_time_eq(req.path.as_bytes(), body_tenant_id.as_bytes()) {
                Ok(())
            } else {
                Err(HttpResponse::new(403))
            }
        }
    "#;

    #[test]
    fn owned_kernel_handler_to_sync_typed_handler_impl_is_green_when_impl_calls_guard() {
        let observed = observe(GREEN_OWNED_KERNEL_HANDLER_TO_SYNC_TYPED_HANDLER);
        assert_eq!(
            observed["surfaces"][0]["routes"][0]["handler"], "PolicyAdmissionHandler",
            "handler_to_sync(HandlerStruct) must resolve to the typed handler struct"
        );
        assert!(
            is_green(GREEN_OWNED_KERNEL_HANDLER_TO_SYNC_TYPED_HANDLER).is_empty(),
            "a typed Handler impl with a real guard must cover an owned-kernel POST: {:?}",
            is_green(GREEN_OWNED_KERNEL_HANDLER_TO_SYNC_TYPED_HANDLER)
        );
    }

    // RED control: unwrapping handler_to_sync must not itself count as authz.
    const RED_OWNED_KERNEL_HANDLER_TO_SYNC_TYPED_HANDLER_WITHOUT_GUARD: &str = r#"
        fn build(router: &mut Router<SyncHandler>) -> Result<(), RouterError> {
            router.route(HttpMethod::Post, "/tenant-rbac/v1/policy-admissions", handler_to_sync(PolicyAdmissionHandler))?;
            Ok(())
        }

        struct PolicyAdmissionHandler;

        impl oya_http_middleware_kernel::Handler for PolicyAdmissionHandler {
            type Error = HttpResponse;

            fn call(&self, _req: HttpRequest) -> Result<HttpResponse, Self::Error> {
                Ok(HttpResponse::new(202))
            }
        }
    "#;

    #[test]
    fn owned_kernel_handler_to_sync_typed_handler_without_guard_remains_red() {
        assert!(
            has_code(
                RED_OWNED_KERNEL_HANDLER_TO_SYNC_TYPED_HANDLER_WITHOUT_GUARD,
                "AC-UNAUTHENTICATED-CONTROL-PLANE"
            ),
            "handler_to_sync is only a typed-handler bridge, not an authz marker"
        );
    }

    // RED: an owned-kernel route whose method is a `const NAME: HttpMethod = HttpMethod::Post;` — the
    // const-method must resolve (a fail-OPEN hole if it dropped). No guard ⇒ RED.
    const RED_OWNED_KERNEL_CONST_METHOD: &str = r#"
        const REST_METHOD: HttpMethod = HttpMethod::Post;
        fn build(router: &mut Router<SyncHandler>) -> Result<(), RouterError> {
            router.route(REST_METHOD, "/admin/v1/apply", apply_handler)?;
            Ok(())
        }
    "#;

    #[test]
    fn owned_kernel_const_method_post_is_red() {
        assert!(
            has_code(
                RED_OWNED_KERNEL_CONST_METHOD,
                "AC-UNAUTHENTICATED-CONTROL-PLANE"
            ),
            "a const-method (= HttpMethod::Post) owned-kernel route must resolve + be RED: {:?}",
            is_green(RED_OWNED_KERNEL_CONST_METHOD)
        );
        assert_eq!(
            observe(RED_OWNED_KERNEL_CONST_METHOD)["surfaces"][0]["routes"][0]["method_class"],
            "mutating"
        );
    }

    // RED: owned-kernel-SHAPED 3-arg `.route(method_var, field.path, handler)` where the path
    // arg is a field access (contains `.`) — the engine cannot classify it into either grammar.
    // Must fail-CLOSED as AC-UNCLASSIFIED-SURFACE rather than silently dropping (fail-open).
    // This is the MAJOR fix: libs/oya-shared-backbone-rest-runtime-adapter uses exactly this shape.
    const RED_OWNED_KERNEL_FIELD_PATH: &str = r#"
        fn register(router: &mut Router<SyncHandler>, route: &RouteSpec, handler: SyncHandler) {
            router.route(method, route.path, handler).expect("route");
        }
    "#;

    #[test]
    fn owned_kernel_field_path_fails_closed_as_unclassified_surface() {
        assert!(
            has_code(RED_OWNED_KERNEL_FIELD_PATH, "AC-UNCLASSIFIED-SURFACE"),
            "a 3-arg .route(method_ident, field.path, handler) must fail-CLOSED as \
             AC-UNCLASSIFIED-SURFACE, not silently drop (fail-open): {:?}",
            is_green(RED_OWNED_KERNEL_FIELD_PATH)
        );
    }

    // ---- non-route-introduction `.route(` calls (domain/dispatch) must NOT register ---------------

    #[test]
    fn domain_route_method_is_not_a_surface() {
        // A `.route(struct{..})`, a `usecase.route(input)`, an enum `.route()` — none are HTTP route
        // introductions; they must NOT be discovered as surfaces (no findings, no surface emitted).
        let text = r#"
            fn dispatch(usecase: &U) -> Receipt {
                usecase.route(base_input("idem", vec![]))
            }
            fn classify(intent: &Intent) -> DriveRoute { intent.route() }
            fn route_req(r: &Router) -> Response { r.route(RouteRequest { model: "x" }) }
        "#;
        let observed = observe(text);
        assert_eq!(
            observed["surfaces"].as_array().map(|a| a.len()),
            Some(0),
            "domain/dispatch route() methods must not register as router surfaces: {observed}"
        );
        assert!(is_green(text).is_empty());
    }

    // ---- MINOR-1: raw-string path mask alignment ---------------------------------------------------

    #[test]
    fn raw_string_path_is_resolved_and_red() {
        // A raw-string route path `r"/v1/x/{id}"` must resolve (the mask must not desync on `r"..."`).
        let text = r##"
            async fn nuke() -> StatusCode { StatusCode::NO_CONTENT }
            pub fn build() -> Router {
                Router::new().route(r"/v1/x/{id}", delete(nuke)).with_state(())
            }
        "##;
        let observed = observe(text);
        assert_eq!(observed["surfaces"][0]["routes"][0]["path"], "/v1/x/{id}");
        assert!(
            evaluate_keyed(&policy(), &observed)
                .iter()
                .any(|f| f.code == "AC-UNAUTHENTICATED-CONTROL-PLANE"),
            "a raw-string-path unauthenticated DELETE must be RED"
        );
    }

    #[test]
    fn raw_string_with_embedded_quotes_does_not_desync_mask() {
        // A raw string containing `"` and `\` must not break offset alignment so a following real
        // route is still parsed correctly (the MINOR-1 desync regression).
        let text = r###"
            async fn nuke() -> StatusCode {
                let _sql = r#"SELECT "x" FROM t WHERE p = "/route(\\""#;
                StatusCode::NO_CONTENT
            }
            pub fn build() -> Router {
                Router::new().route("/after/{id}", delete(nuke)).with_state(())
            }
        "###;
        let observed = observe(text);
        assert_eq!(
            observed["surfaces"][0]["routes"][0]["path"], "/after/{id}",
            "the route after a raw string with embedded quotes must still parse: {observed}"
        );
    }

    // ---- MAJOR-1: --write shrink-only baseline -----------------------------------------------------

    #[test]
    fn shrink_only_baseline_drops_fixed_keeps_live_refuses_new() {
        // Prior baseline = {fixed_key (no longer live), live_key (still live)}. Live findings =
        // {live_key, new_key}. Without --allow-new: next = {live_key} (fixed dropped, new refused);
        // new_keys = {new_key}. With --allow-new: next = {live_key, new_key}.
        let text = r#"
            async fn a() -> StatusCode { StatusCode::OK }
            pub fn r() -> Router { Router::new().route("/a", post(a)).with_state(()) }
        "#;
        let observed = observe(text);
        let live_key = "fixture.rs#r::router[post /a]";
        let fixed_key = "gone/old.rs#r::router[post /x]";
        let mut p = policy();
        p["frozen_unauthenticated_surfaces"] = json!([fixed_key, live_key]);

        let (next, new_keys) = shrink_only_baseline(&p, &observed, false);
        assert_eq!(
            next,
            vec![live_key.to_owned()],
            "fixed dropped, live kept, new refused: {next:?}"
        );
        assert!(
            new_keys.is_empty(),
            "no NEW key here (live_key was already baselined): {new_keys:?}"
        );

        // Now a baseline that does NOT contain the live key ⇒ it is a NEW key, refused unless allowed.
        let mut p2 = policy();
        p2["frozen_unauthenticated_surfaces"] = json!([fixed_key]);
        let (next2, new2) = shrink_only_baseline(&p2, &observed, false);
        assert!(
            next2.is_empty(),
            "without --allow-new a new key is NOT absorbed: {next2:?}"
        );
        assert_eq!(
            new2,
            vec![live_key.to_owned()],
            "the new key is reported: {new2:?}"
        );
        let (next3, _) = shrink_only_baseline(&p2, &observed, true);
        assert_eq!(
            next3,
            vec![live_key.to_owned()],
            "with --allow-new the new key is absorbed: {next3:?}"
        );
    }
}
