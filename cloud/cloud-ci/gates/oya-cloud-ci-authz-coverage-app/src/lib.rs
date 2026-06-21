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
//! A Rust file that constructs an axum `Router` (`Router::new()....route(path, METHOD(handler))`)
//! is a surface. A surface is a CONTROL PLANE when any of its routes is either
//! - a MUTATING method (`post`/`put`/`patch`/`delete`) on a non-exempt path, or
//! - a per-resource path param (`{id}`/`{tenant_id}`/...) on a mutating method.
//! `/healthz`-style unauthenticated reads are exempt via an explicit DATA allowlist
//! (`exempt_path_substrings` in `authz-coverage-policy.json`) — never code.
//!
//! ## Required authz coverage
//! A control-plane surface is COVERED iff
//! - its builder chain carries a recognized router-level auth `.layer(...)` (a verified-principal
//!   extractor / auth middleware named in policy `auth_layer_idents`), OR
//! - every MUTATING handler bound in the chain invokes a recognized authz decision in its function
//!   body — an `admin_tenant_allowed`-style guard, the tenancy `authorize(...)` pattern, a PDP
//!   `decide(...)` port call, or a bearer/peer authentication guard, all named in policy
//!   `authz_guard_idents`.
//! A mutating handler that derives no caller identity → the surface is UNAUTHENTICATED.
//!
//! ## Conservative in the SAFE direction
//! Like the kernel-purity gate's src-ident liveness probe, handler-body authz detection is a
//! token over-approximation: a guard ident anywhere in the handler's `fn` body counts as covered.
//! This can only ever mark a surface as COVERED when it is in fact covered or close to it; it never
//! invents a false UNAUTHENTICATED finding for a handler that genuinely calls a guard. The
//! risk it trades away (a handler that names a guard ident in a comment but never calls it) is
//! acceptable: this gate's job is to stop the ZERO-authz class (the AUTH-005 exhibit had no guard
//! token anywhere), not to prove call-graph reachability — the audit-coverage gate (AC-W-13) and
//! human review own the deeper proof.
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
//! ## Fail-closed spine
//! A textual matcher need not be perfect IF its failure mode is fail-closed. Any `.route(...)` the
//! engine cannot PROVE is (a) non-mutating OR (b) authorization-covered produces a FINDING (RED),
//! never a silent skip. Two backstops enforce this:
//! - a `.route(` path the engine cannot resolve to a concrete string (a `const`/`static` it cannot
//!   substitute, or any non-literal it does not understand) → `AC-UNRESOLVED-ROUTE-PATH`.
//! - a `.route(`'s method-router the engine cannot classify (a `let`-bound variable it cannot
//!   resolve, an unrecognized call) → `AC-UNCLASSIFIED-METHOD`, treated as potentially-mutating.
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
pub const REMEDIATION_DOCTRINE: &str =
    "intelligence/adapters/rest/src/lib.rs (admin_tenant_allowed + PDP gate.decide + constant_time_eq) \
     and tenancy/facade/tenant-lifecycle-app/src/lib.rs (authenticate_caller + authorize() per route)";

/// The blocking + structural violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 7] = [
    "AC-UNAUTHENTICATED-CONTROL-PLANE",
    "AC-UNRESOLVED-ROUTE-PATH",
    "AC-UNCLASSIFIED-METHOD",
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
        for surface in
            extract_surfaces(rel_path, &text, &auth_layer_idents, &authz_guard_idents)
        {
            surfaces.push(surface);
        }
    }
    surfaces.sort_by(|a, b| surface_sort_key(a).cmp(&surface_sort_key(b)));

    Ok(json!({
        "surfaces_found": surfaces.len(),
        "surfaces": surfaces,
    }))
}

fn surface_sort_key(surface: &Value) -> (String, u64) {
    (
        surface.get("file").and_then(Value::as_str).unwrap_or("").to_owned(),
        surface.get("router_line").and_then(Value::as_u64).unwrap_or(0),
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
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
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
            MethodClass::Mutating(s) | MethodClass::NonMutating(s) | MethodClass::Unclassified(s) => {
                s
            }
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

/// A single parsed route within a builder chain.
///
/// `path` is the RESOLVED concrete route string (a literal, or a `const`/`static` ident substituted
/// from the file's declarations) — `None` when the path argument cannot be resolved to a concrete
/// string (fail-closed: such a route is an unknown-authz control-plane surface). `path_raw` is the
/// raw argument text (the ident name or the literal) kept for the finding detail and stable keying.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Route {
    path: Option<String>,
    path_raw: String,
    method: MethodClass,
    handler: String,
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

/// Extract every `Router::new()....route(...)` surface from one file's source text.
///
/// The parser is line/char based (not a full Rust AST — the repo has no Rust-source AST kernel),
/// but it is robust to the shapes the corpus uses: multiline `.route(\n  "path",\n
/// METHOD(handler),\n)` calls, generic handler turbofish (`handler::<S>`), and a `.layer(...)`
/// anywhere in the chain. A chain begins at a `Router::new()` occurrence and runs until the
/// builder terminator (`.with_state(`, `.into_make_service`, or a `;`/blank-structural boundary).
/// We scan the WHOLE remaining buffer for `.route(`/`.layer(` calls belonging to the chain; this
/// over-includes at worst (a later sibling router's routes folded in), which only ever makes a
/// surface look MORE covered or adds routes — never hides an unauthenticated mutating route.
///
/// Structure is searched against a length-preserving [`mask_non_code`] view (so comment/string
/// mentions of `Router::new()`/`.route(` never register), while literal VALUES (paths, consts) are
/// read from the original `text` at the aligned offsets.
fn extract_surfaces(
    file: &str,
    text: &str,
    auth_layer_idents: &[String],
    authz_guard_idents: &[String],
) -> Vec<Value> {
    let mut out = Vec::new();
    // Length-preserving code-structure mask: comment + string/char content blanked (quotes kept),
    // offsets aligned with `text`. ALL structural `.find()` runs against `masked`; literal VALUES are
    // read from `text` at the aligned offsets. This stops comment/string mentions of `Router::new()`
    // / `.route(` (including this gate scanning its OWN source) from registering as surfaces.
    let masked = mask_non_code(text);
    let masked = masked.as_str();
    // Spans of `#[cfg(test)]`-gated code (test modules + test fns). A Router built inside one is a
    // TEST FIXTURE, not a production control plane (this gate's own RED/GREEN fixtures live in
    // `#[cfg(test)] mod tests`), so it is skipped — never frozen, never blocked.
    let test_spans = cfg_test_spans(masked);
    // File-level `const NAME: &str = "...";` / `static NAME: &str = "...";` map for B1 const-path
    // resolution, and a `let m = METHOD(h);` binding map for B2 variable-method resolution. Both are
    // computed once per file; route parsing substitutes from them and FAILS CLOSED when a path ident
    // or method-router variable cannot be resolved. Decls are LOCATED in `masked`, VALUES read from `text`.
    let str_consts = collect_str_consts(masked, text);
    let method_bindings = collect_method_bindings(masked);
    let mut search_from = 0usize;
    while let Some(rel) = masked[search_from..].find("Router::new()") {
        let start = search_from + rel;
        if test_spans.iter().any(|(lo, hi)| start >= *lo && start < *hi) {
            search_from = start + "Router::new()".len();
            continue;
        }
        let router_line = line_of(text, start);
        let chain_end = chain_end_offset(masked, start);
        let chain_masked = &masked[start..chain_end];
        let chain_text = &text[start..chain_end];

        let routes = parse_routes(chain_masked, chain_text, &str_consts, &method_bindings);
        let has_auth_layer = chain_has_auth_layer(chain_masked, auth_layer_idents);

        // Per-handler authz: for each handler bound in this surface, does its `fn` body anywhere in
        // the file invoke a recognized authz guard ident? Computed for all handlers (the evaluator
        // only consults the mutating ones, but emitting all keeps the observation self-describing).
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
                })
            })
            .collect();

        out.push(json!({
            "file": file,
            "router_line": router_line as u64,
            "routes": routes_json,
            "has_auth_layer": has_auth_layer,
            "handler_authz": Value::Object(handler_authz),
        }));

        // Advance past this Router::new() token so a chain with its own nested Router::new() (rare)
        // is still progressed; the next chain starts at the next occurrence.
        search_from = start + "Router::new()".len();
    }
    out
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
        let attr_end = text[at..].find(']').map(|i| at + i + 1).unwrap_or(text.len());
        let attr = &text[at..attr_end];
        if attr_contains_test_token(attr) {
            if let Some(body) = brace_body(text, attr_end) {
                let body_start = body.as_ptr() as usize - text.as_ptr() as usize;
                spans.push((at, body_start + body.len()));
            }
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

/// Find the end offset of a builder chain starting at `start` (`Router::new()`). The chain ends at
/// the first builder terminator AFTER the last `.route(`/`.layer(`/`.with_state(`/`.merge(`/
/// `.nest(`/`.fallback(` method call: in practice that is the statement-terminating boundary. We
/// approximate it as the next blank line that follows a `.with_state(` / `.into_make_service` /
/// `.layer(` tail, or — failing those — the next top-level `;` from `start`. Over-running to a
/// later sibling chain only ever ADDS routes/coverage signal, never removes the protected mutating
/// route, so the safe direction holds.
fn chain_end_offset(text: &str, start: usize) -> usize {
    let rest = &text[start..];
    // Prefer the explicit builder terminators; take the EARLIEST that appears, then run to the end
    // of that call/statement.
    let terminators = [".with_state(", ".into_make_service"];
    let mut best: Option<usize> = None;
    for term in terminators {
        if let Some(idx) = rest.find(term) {
            best = Some(best.map_or(idx, |b| b.min(idx)));
        }
    }
    if let Some(term_idx) = best {
        // Run to the end of the line that statement closes on (next newline after a `;` or after
        // the terminator's matching context). Use the next `;` after the terminator, else next \n.
        let after = start + term_idx;
        if let Some(semi) = text[after..].find(';') {
            return after + semi + 1;
        }
        if let Some(nl) = text[after..].find('\n') {
            return after + nl + 1;
        }
        return text.len();
    }
    // No explicit terminator (e.g. a function that `return`s the Router directly): run to the next
    // top-level `;` from start, else to EOF.
    if let Some(semi) = rest.find(';') {
        return start + semi + 1;
    }
    text.len()
}

/// Parse all `.route(...)` calls within a chain slice into [`Route`]s, FAIL-CLOSED.
///
/// For each `.route(`, the call's balanced-paren argument list is split at the top-level comma into
/// the PATH arg and the METHOD-ROUTER arg:
/// - The path arg is a `"..."` literal (taken verbatim) or an ident resolved from the file's
///   `const`/`static &str` declarations (`str_consts`). An ident that resolves to no concrete
///   string yields `path: None` (B1 fail-closed: an unresolvable control-plane path).
/// - The method-router arg is classified via [`classify_method_router`], resolving a `let m =
///   METHOD(h);` binding (`method_bindings`) when the arg is a bare ident (B2). A shape that cannot
///   be classified yields `MethodClass::Unclassified` (B2 fail-closed).
///
/// A `.route(` whose arg list cannot be bounded (no balanced close) still emits an
/// unresolved-path/unclassified route (fail-closed) so an invisible surface cannot result from a
/// truncated chain.
fn parse_routes(
    chain_masked: &str,
    chain_text: &str,
    str_consts: &std::collections::BTreeMap<String, String>,
    method_bindings: &std::collections::BTreeMap<String, MethodBinding>,
) -> Vec<Route> {
    let mut routes = Vec::new();
    let mut from = 0usize;
    let marker = ".route(";
    while let Some(rel) = chain_masked[from..].find(marker) {
        let open = from + rel + marker.len(); // just past the `(`
        from = open;
        // Bound the route call to its balanced closing paren on the MASKED chain (so a `)` inside a
        // string/comment does not mis-bound), then read the args from both views: structure from
        // masked, the path literal VALUE from the original text (offsets align).
        let Some(args_masked) = balanced_paren_body(chain_masked, open) else {
            // A truncated `.route(` we cannot bound: fail closed — record an unresolved, potentially
            // mutating route so the surface is never silently dropped.
            routes.push(Route {
                path: None,
                path_raw: "<unparsed-route-args>".to_owned(),
                method: MethodClass::Unclassified("<unparsed>".to_owned()),
                handler: String::new(),
            });
            continue;
        };
        // Offset of the args within the chain (masked and text are byte-aligned).
        let args_off = args_masked.as_ptr() as usize - chain_masked.as_ptr() as usize;
        let args_len = args_masked.len();
        let args_text = &chain_text[args_off..args_off + args_len];

        let (path_arg_masked, method_arg_masked) = split_top_level_comma(args_masked);
        let path_off = path_arg_masked.as_ptr() as usize - chain_masked.as_ptr() as usize;
        let path_arg_text = &chain_text[path_off..path_off + path_arg_masked.len()];
        // Resolve the path: structure (is it a literal vs ident) from masked, VALUE from text.
        let (path, path_raw) = resolve_path_arg(path_arg_masked, path_arg_text, str_consts);
        let _ = args_text; // method classification uses the masked arg (handler/method are idents).
        let (method, handler) =
            classify_method_router(method_arg_masked.unwrap_or(""), method_bindings);
        routes.push(Route {
            path,
            path_raw,
            method,
            handler,
        });
    }
    routes
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
    (None, if raw.is_empty() { "<empty-path-arg>".to_owned() } else { raw })
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
            return (MethodClass::NonMutating(m.to_owned()), read_path_ident(rest));
        }
    }
    for m in mutmeth {
        if let Some(rest) = strip_call(trimmed, m) {
            return (MethodClass::Mutating(m.to_owned()), read_path_ident(rest));
        }
    }
    if let Some(rest) = strip_call(trimmed, "any") {
        // `any` accepts every verb, writes included.
        return (MethodClass::Mutating("any".to_owned()), read_path_ident(rest));
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
        return (MethodClass::Unclassified(format!("var:{ident}")), String::new());
    }
    // 4) anything else -> fail closed.
    (MethodClass::Unclassified("<unrecognized>".to_owned()), String::new())
}

/// If `text` begins with `ident` immediately followed (after optional whitespace) by `(`, return the
/// slice just past that `(`; else None. Ensures `ident` is a whole call ident, not a prefix.
fn strip_call<'a>(text: &'a str, ident: &str) -> Option<&'a str> {
    let t = text.trim_start();
    let rest = t.strip_prefix(ident)?;
    // The char right after the ident must not be an ident char (so `on` != `onfoo`).
    if let Some(c) = rest.chars().next() {
        if is_ident_char(c) {
            return None;
        }
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
fn collect_str_consts(
    masked: &str,
    text: &str,
) -> std::collections::BTreeMap<String, String> {
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
            let Some(colon) = decl_masked.find(':') else { continue };
            let name = decl_masked[..colon].trim();
            // `static mut` / generics make the name non-simple — require a plain ident.
            let name = name.trim_start_matches("mut ").trim();
            if name.is_empty() || !name.chars().all(is_ident_char) {
                continue;
            }
            // Find the `=` (masked) then the first string literal before the terminating `;`. The
            // initializer VALUE is read from the ORIGINAL text at the aligned offset.
            let Some(eq) = decl_masked[colon..].find('=') else { continue };
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
    let empty: std::collections::BTreeMap<String, MethodBinding> = std::collections::BTreeMap::new();
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
        let semi = decl[init_start..].find(';').map(|i| init_start + i).unwrap_or(decl.len());
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
            if auth_layer_idents.iter().any(|ident| arg.contains(ident.as_str())) {
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
/// tests whether any `authz_guard_idents` token appears in that body. A guard ident may be a plain
/// ident (`authorize`, `admin_tenant_allowed`) or a method tail (`.decide(`) — both are simple
/// substring probes, sound for the over-approximation this gate intends (a guard token in the body
/// ⇒ the handler derives caller identity). Async handlers are covered because we anchor on
/// `fn <handler>` regardless of the `async`/`pub` prefix.
///
/// If the body names no guard directly, it is probed for a SINGLE local-function delegate it calls
/// (a thin wrapper like `handle_openai_compatible_proxy(state, headers, body, ..).await`); the gate
/// recurses one hop into that delegate's body. This recognizes the real intelligence/adapters/rest
/// data-plane wrappers as COVERED without a full call graph.
fn handler_body_has_guard(text: &str, handler: &str, guard_idents: &[String]) -> bool {
    has_guard_rec(text, handler, guard_idents, MAX_DELEGATE_DEPTH, &mut BTreeSet::new())
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
        if before_ok && after_ok {
            if let Some(body) = brace_body(text, after_name) {
                // B3 FIX: the guard probe runs on a CODE-ONLY view of the body — comments and
                // string/char literals elided — so a `// TODO: authorize()` comment or a
                // `"authorize"` string literal can NEVER false-cover a handler that does no real
                // authz. The guard ident must appear in genuine code.
                let code = code_only(body);
                if guard_idents.iter().any(|g| code.contains(g.as_str())) {
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
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\n' || bytes[j] == b'\t' || bytes[j] == b'\r') {
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
        "if" | "while" | "for" | "match" | "loop" | "return" | "let" | "fn" | "async"
            | "await" | "move" | "in" | "as" | "ref" | "mut" | "Some" | "Ok" | "Err" | "None"
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
fn path_exempt(path: &str, exempt_substrings: &[String]) -> bool {
    exempt_substrings.iter().any(|s| path.contains(s.as_str()))
}

/// Whether a route's path carries a per-resource path param (`{...}`).
fn has_path_param(path: &str) -> bool {
    path.contains('{') && path.contains('}')
}

/// The stable SIGNATURE key for a surface finding: `<file>::router[<m1 p1; m2 p2; ..>]` where the
/// `(method, route-path)` tuples are sorted (M2). Independent of line numbers and route-declaration
/// order, so an unrelated edit that shifts the router's line does NOT spuriously re-RED a baselined
/// surface (the old `router@<line>` key did). A route's tuple uses its resolved path when known, its
/// raw path arg (`const NAME`) when unresolved, prefixed by the method-class discriminant for an
/// unclassified method-router. Handler names are excluded so a handler rename keeps the key stable.
fn surface_signature_key(file: &str, routes: &[Value]) -> String {
    let mut tuples: Vec<String> = routes
        .iter()
        .map(|r| {
            let method = r.get("method").and_then(Value::as_str).unwrap_or("?");
            let class = r.get("method_class").and_then(Value::as_str).unwrap_or("?");
            let path = match r.get("path").and_then(Value::as_str) {
                Some(p) => p.to_owned(),
                None => format!(
                    "<unresolved:{}>",
                    r.get("path_raw").and_then(Value::as_str).unwrap_or("?")
                ),
            };
            if class == "unclassified" {
                format!("{class}:{method} {path}")
            } else {
                format!("{method} {path}")
            }
        })
        .collect();
    tuples.sort();
    format!("{file}::router[{}]", tuples.join("; "))
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
    if policy.get("authz_guard_idents").and_then(Value::as_array).is_none() {
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
    let frozen_baseline: BTreeSet<String> =
        string_list(policy, "frozen_unauthenticated_surfaces").into_iter().collect();

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
        let router_line = surface.get("router_line").and_then(Value::as_u64).unwrap_or(0);
        let routes = surface.get("routes").and_then(Value::as_array).cloned().unwrap_or_default();
        let has_auth_layer = surface.get("has_auth_layer").and_then(Value::as_bool).unwrap_or(false);
        let handler_authz = surface.get("handler_authz").and_then(Value::as_object);

        // Walk the routes, fail-closed. A route makes the surface a CONTROL PLANE if it is mutating
        // (on a non-exempt resolved path), unclassified (unknown method), or has an unresolved path.
        let mut is_control_plane = false;
        let mut has_unresolved_path = false;
        let mut has_unclassified_method = false;
        // (label, path-display, handler) of every uncovered control-plane route.
        let mut uncovered_handlers: Vec<(String, String, String)> = Vec::new();
        for route in &routes {
            let class = route.get("method_class").and_then(Value::as_str).unwrap_or("");
            let method = route.get("method").and_then(Value::as_str).unwrap_or("");
            let handler = route.get("handler").and_then(Value::as_str).unwrap_or("");
            let path_opt = route.get("path").and_then(Value::as_str);
            let path_raw = route.get("path_raw").and_then(Value::as_str).unwrap_or("");

            let is_mutating = class == "mutating";
            let is_unclassified = class == "unclassified";
            let path_unresolved = path_opt.is_none();

            // A non-mutating, classified route on a RESOLVED path is the only safe (skippable) case.
            if !is_mutating && !is_unclassified && !path_unresolved {
                continue;
            }
            // A resolved exempt-path read is exempt even if mutating (e.g. a `/metrics` push). An
            // UNRESOLVED path cannot be proven exempt → never exempt (fail-closed). An unclassified
            // method on an exempt resolved path is still potentially mutating, so do not exempt it.
            if let Some(path) = path_opt
                && !is_unclassified
                && path_exempt(path, &exempt_substrings)
            {
                continue;
            }

            is_control_plane = true;
            if path_unresolved {
                has_unresolved_path = true;
            }
            if is_unclassified {
                has_unclassified_method = true;
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
                let label = if is_unclassified {
                    format!("UNCLASSIFIED-METHOD({method})")
                } else {
                    method.to_uppercase()
                };
                uncovered_handlers.push((label, path_disp, handler.to_owned()));
            }
        }

        if !is_control_plane {
            continue;
        }
        // The surface is COVERED (skippable) iff it carries NO unresolved-path / unclassified-method
        // route AND (a router-level auth layer guards the whole chain OR every control-plane route is
        // individually covered). A router-level auth layer does NOT excuse an UNRESOLVED-PATH or
        // UNCLASSIFIED-METHOD route — those are recognition failures, not coverage facts — so the
        // structural fail-closed finding still fires.
        if !has_unresolved_path
            && !has_unclassified_method
            && (has_auth_layer || uncovered_handlers.is_empty())
        {
            continue;
        }

        // An uncovered (or unparseable) control-plane surface. Key it by stable signature (M2).
        let key = surface_signature_key(file, &routes);
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
        // Pick the most specific structural code: an unresolved path is the deepest recognition
        // failure, then an unclassified method, else the plain unauthenticated control plane.
        let (code, detail) = if has_unresolved_path {
            (
                "AC-UNRESOLVED-ROUTE-PATH",
                format!(
                    "UNRESOLVED route path (fail-closed): the axum router at {file}:{router_line} mounts a `.route(...)` whose path argument the gate could not resolve to a concrete string (a `const`/`static` it could not substitute, or a non-literal path expression). The gate cannot prove this surface is non-mutating or authz-covered, so it is treated as an unknown-authz control plane. Make the path a literal or a resolvable `const NAME: &str = \"...\";`, and add fail-closed authz — see {REMEDIATION_DOCTRINE}. Uncovered route(s): [{holes}]."
                ),
            )
        } else if has_unclassified_method {
            (
                "AC-UNCLASSIFIED-METHOD",
                format!(
                    "UNCLASSIFIED method-router (fail-closed): the axum router at {file}:{router_line} mounts a `.route(...)` whose method-router the gate could not classify (a `let`-bound MethodRouter it could not resolve, or an unrecognized call shape). It is treated as a potentially-mutating control plane requiring authz. Use an inline `get/post/.../on(MethodFilter::X, h)` shape or ensure the binding is resolvable, and add fail-closed authz — see {REMEDIATION_DOCTRINE}. Uncovered route(s): [{holes}]."
                ),
            )
        } else {
            (
                "AC-UNAUTHENTICATED-CONTROL-PLANE",
                format!(
                    "NEW unauthenticated HTTP control plane: the axum router at {file}:{router_line} mounts mutating route(s) [{holes}] whose handler(s) derive no caller identity (no recognized authz guard in the handler body and no router-level auth layer). Any network caller can invoke these writes. Add fail-closed authz before merge — see {REMEDIATION_DOCTRINE}. If a route is a genuinely unauthenticated read, declare its path in `exempt_path_substrings` (DATA)."
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
pub const SURFACE_FINDING_CODES: [&str; 3] = [
    "AC-UNAUTHENTICATED-CONTROL-PLANE",
    "AC-UNRESOLVED-ROUTE-PATH",
    "AC-UNCLASSIFIED-METHOD",
];

/// Regenerate the frozen-baseline signature keys from the live observation (the AUTOMATED property:
/// re-baselining is mechanical, not hand-edited). Returns the sorted set of per-surface finding keys
/// that the gate WOULD block against an EMPTY baseline — i.e. every currently-detected uncovered /
/// unparseable control-plane surface. `--write` substitutes these into
/// `frozen_unauthenticated_surfaces`, freezing today's surfaces so only NEW ones block.
pub fn baseline_keys(policy: &Value, observed: &Value) -> Vec<String> {
    // Evaluate against an empty baseline so every live surface produces its finding (and no
    // stale-baseline noise). Then keep only the per-surface finding keys.
    let mut p = policy.clone();
    p["frozen_unauthenticated_surfaces"] = json!([]);
    let mut keys: BTreeSet<String> = BTreeSet::new();
    for finding in evaluate_keyed(&p, observed) {
        if SURFACE_FINDING_CODES.contains(&finding.code.as_str()) {
            keys.insert(finding.key);
        }
    }
    keys.into_iter().collect()
}

/// Human-readable render of the findings. Never a bare FAIL — every finding prints its detail.
pub fn render_findings(findings: &BTreeSet<Finding>) -> String {
    if findings.is_empty() {
        return "authz-coverage gate passed: every NEW HTTP control-plane surface carries fail-closed authz (router-level auth layer or per-handler authz guard); no new unauthenticated mutating router".to_owned();
    }
    let mut out = String::from("authz-coverage gate failed (issue #770 / AUTH-005 class):\n");
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
            "min_expected_surfaces": 0,
            "scan_roots": ["src"],
            "excluded_dir_names": ["target", "third-party"],
            "auth_layer_idents": ["RequireAuth", "AuthLayer", "require_principal"],
            "authz_guard_idents": [
                "authorize", "admin_tenant_allowed", "authenticate_caller",
                ".decide(", "require_data_plane_bearer", "authorize_token_for",
                "authorize_with_token"
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
            findings.iter().any(|f| f.code == "AC-UNAUTHENTICATED-CONTROL-PLANE"),
            "an unauthenticated POST/DELETE router must produce AC-UNAUTHENTICATED-CONTROL-PLANE: {findings:?}"
        );
        let finding = findings
            .iter()
            .find(|f| f.code == "AC-UNAUTHENTICATED-CONTROL-PLANE")
            .unwrap();
        assert!(finding.detail.contains("DELETE /things/{id}"), "names the delete hole: {finding:?}");
        assert!(finding.detail.contains("POST /things"), "names the post hole: {finding:?}");
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
            surface["has_auth_layer"], Value::from(false),
            "DefaultBodyLimit is not an auth layer"
        );
        let findings = evaluate_keyed(&policy(), &observed);
        assert!(findings.is_empty(), "per-handler guards cover the surface: {findings:?}");
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
        assert!(findings.is_empty(), "a read-only health router is not a control plane: {findings:?}");
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
        assert_eq!(surface["has_auth_layer"], Value::from(true), "RequireAuth is an auth layer");
        let findings = evaluate_keyed(&policy(), &observed);
        assert!(findings.is_empty(), "a router-level auth layer covers all routes: {findings:?}");
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
        assert_eq!(blocked.len(), 1, "an un-baselined unauthenticated surface blocks: {blocked:?}");
        let key = blocked[0].key.clone();
        assert_eq!(key, "fixture.rs::router[post /a]", "signature key is line-independent: {key}");

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
            findings.iter().any(|f| f.code == "AC-STALE-BASELINE" && f.key == "some/old/file.rs::router@10"),
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
        assert!(findings.iter().any(|f| f.code == "AC-POLICY-GATE-ID-MISMATCH"));
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
        assert_eq!(routes[0]["handler"], "register_tenant", "turbofish stripped");
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
        evaluate_keyed(&policy(), &observed).iter().any(|f| f.code == code)
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
        assert_eq!(route["path"], "/tenants/{id}", "const NUKE substituted to its value");
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
        assert_eq!(observed["surfaces"][0]["routes"][0]["method_class"], "mutating");
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
        assert_eq!(observed["surfaces"][0]["routes"][0]["method_class"], "mutating");
        assert_eq!(observed["surfaces"][0]["routes"][0]["handler"], "delete_thing");
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
            has_code(RED_B3_COMMENT_ONLY_GUARD, "AC-UNAUTHENTICATED-CONTROL-PLANE"),
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
        assert_eq!(k1, k2, "the signature key must not depend on the router's line number");
        assert_eq!(k1.as_deref(), Some("fixture.rs::router[post /a]"));
    }
}
