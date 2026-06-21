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
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `AC-UNAUTHENTICATED-CONTROL-PLANE` — a control-plane surface (mutating method and/or
//!   per-resource path) has ≥1 mutating handler that derives no caller identity, and its key is not
//!   in the frozen baseline (a NEW unauthenticated control plane).
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
pub const VIOLATION_CODES: [&str; 5] = [
    "AC-UNAUTHENTICATED-CONTROL-PLANE",
    "AC-STALE-BASELINE",
    "AC-EMPTY-SCAN",
    "AC-POLICY-GATE-ID-MISMATCH",
    "AC-POLICY-MALFORMED",
];

/// The sentinel key for codes that are policy-level rather than per-surface.
const POLICY_KEY: &str = "<policy>";

/// The mutating HTTP methods. A route bound to any of these is a write — the class this gate
/// exists to protect. Held as a const (not policy) because the axum method-router fn names are a
/// fixed library fact, not a repo choice.
const MUTATING_METHODS: [&str; 4] = ["post", "put", "patch", "delete"];

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

/// A single parsed route within a builder chain.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Route {
    path: String,
    method: String,
    handler: String,
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
fn extract_surfaces(
    file: &str,
    text: &str,
    auth_layer_idents: &[String],
    authz_guard_idents: &[String],
) -> Vec<Value> {
    let mut out = Vec::new();
    // Spans of `#[cfg(test)]`-gated code (test modules + test fns). A Router built inside one is a
    // TEST FIXTURE, not a production control plane (this gate's own RED/GREEN fixtures live in
    // `#[cfg(test)] mod tests`), so it is skipped — never frozen, never blocked.
    let test_spans = cfg_test_spans(text);
    let mut search_from = 0usize;
    while let Some(rel) = text[search_from..].find("Router::new()") {
        let start = search_from + rel;
        if test_spans.iter().any(|(lo, hi)| start >= *lo && start < *hi) {
            search_from = start + "Router::new()".len();
            continue;
        }
        let router_line = line_of(text, start);
        let chain_end = chain_end_offset(text, start);
        let chain = &text[start..chain_end];

        let routes = parse_routes(chain);
        let has_auth_layer = chain_has_auth_layer(chain, auth_layer_idents);

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
                json!({ "path": r.path, "method": r.method, "handler": r.handler })
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

/// Parse all `.route(...)` calls within a chain slice into `(path, method, handler)` triples.
/// Tolerates whitespace/newlines between `.route(`, the path string, the method-router call, and
/// the handler. Only the FIRST method-router call inside each `.route(` is taken (axum binds one
/// method-router per route; `get(h).post(h2)` chained handlers are rare in this corpus and the
/// first mutating one is sufficient to mark the surface as control-plane).
fn parse_routes(chain: &str) -> Vec<Route> {
    let mut routes = Vec::new();
    let mut from = 0usize;
    let marker = ".route(";
    while let Some(rel) = chain[from..].find(marker) {
        let open = from + rel + marker.len();
        from = open;
        // Find the path string literal: the next `"..."` after `.route(`.
        let Some((path, after_path)) = next_string_literal(chain, open) else {
            continue;
        };
        // Find the method-router call after the path: METHOD(handler).
        if let Some((method, handler)) = next_method_router(chain, after_path) {
            routes.push(Route { path, method, handler });
        }
    }
    routes
}

/// From offset `from`, find the next `"..."` string literal; return its contents and the offset
/// just past the closing quote. Handles escaped quotes inside the literal.
fn next_string_literal(text: &str, from: usize) -> Option<(String, usize)> {
    let bytes = text.as_bytes();
    let mut i = from;
    while i < bytes.len() && bytes[i] != b'"' {
        i += 1;
    }
    if i >= bytes.len() {
        return None;
    }
    i += 1; // past opening quote
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

/// From offset `from`, find the next method-router call `IDENT(handler...)` where IDENT is an HTTP
/// method (get/post/put/patch/delete/head/options/trace/any). Returns `(method, handler_ident)`.
/// The handler ident is the first path-ish token inside the method call, with any `::<...>`
/// turbofish stripped. Stops at the route's own closing — bounded by the next `.route(` or the end
/// of the chain — so a method token from a LATER route is never misattributed.
fn next_method_router(text: &str, from: usize) -> Option<(String, String)> {
    let methods = [
        "get", "post", "put", "patch", "delete", "head", "options", "trace", "any",
    ];
    // Bound the search to this route call: up to the next `.route(` or end of slice.
    let bound = text[from..]
        .find(".route(")
        .map(|i| from + i)
        .unwrap_or(text.len());
    let window = &text[from..bound];
    // Find the earliest method-router call in the window.
    let mut best: Option<(usize, String)> = None;
    for m in methods {
        if let Some(idx) = find_call_ident(window, m) {
            best = match best {
                Some((b, _)) if b <= idx => best,
                _ => Some((idx, m.to_owned())),
            };
        }
    }
    let (idx, method) = best?;
    // The handler is the first token after the method's `(`.
    let after_paren = idx + method.len();
    // skip to `(`
    let rest = &window[after_paren..];
    let paren = rest.find('(')?;
    let handler_start = after_paren + paren + 1;
    let handler = read_path_ident(&window[handler_start..]);
    if handler.is_empty() {
        return None;
    }
    Some((method, handler))
}

/// Find the offset of `ident` used as a CALL (`ident(`), not as a substring of a longer ident.
/// Requires a non-ident char (or start) before `ident` and a `(` (possibly after whitespace)
/// after it. Returns the offset of the ident start.
fn find_call_ident(text: &str, ident: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = text[from..].find(ident) {
        let at = from + rel;
        let before_ok = at == 0 || !is_ident_byte(bytes[at - 1]);
        let after = at + ident.len();
        // allow optional whitespace then `(`
        let mut j = after;
        while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\n' || bytes[j] == b'\t' || bytes[j] == b'\r') {
            j += 1;
        }
        let after_ok = j < bytes.len() && bytes[j] == b'(';
        if before_ok && after_ok {
            return Some(at);
        }
        from = at + ident.len();
    }
    None
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
                if guard_idents.iter().any(|g| body.contains(g.as_str())) {
                    return true;
                }
                // No direct guard: follow up to one local delegate this body calls.
                if depth > 0 {
                    for delegate in delegate_calls_in(body, handler) {
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

/// The stable key for a surface finding: `<file>::router@<line>`. Stable across reorderings of the
/// route list and independent of handler names, so a refactor that renames a handler but keeps the
/// hole still matches its baseline entry (the surface is identified by file+router site).
fn surface_key(file: &str, router_line: u64) -> String {
    format!("{file}::router@{router_line}")
}

/// Pure evaluator. `policy` is DATA (`authz-coverage-policy.json`); `observed` is the collected
/// surface graph shaped by [`collect_surfaces`].
///
/// For each surface: a route is a CONTROL-PLANE write iff it uses a mutating method on a non-exempt
/// path, OR it is a mutating method on a per-resource path param. The surface is COVERED iff it has
/// a recognized auth layer OR every mutating, non-exempt handler's body invokes a recognized authz
/// guard. An uncovered control-plane surface whose key is NOT in the frozen baseline →
/// `AC-UNAUTHENTICATED-CONTROL-PLANE`. Baseline keys with no live finding → `AC-STALE-BASELINE`.
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

        // Collect the mutating, non-exempt handlers + whether the surface is a control plane.
        let mut is_control_plane = false;
        let mut uncovered_handlers: Vec<(String, String, String)> = Vec::new(); // (method, path, handler)
        for route in &routes {
            let path = route.get("path").and_then(Value::as_str).unwrap_or("");
            let method = route.get("method").and_then(Value::as_str).unwrap_or("");
            let handler = route.get("handler").and_then(Value::as_str).unwrap_or("");
            let is_mutating = MUTATING_METHODS.contains(&method);
            // A per-resource path param on a mutating method is a control plane even if the
            // generic mutating-method test already caught it; a per-resource GET is a sensitive
            // read but this gate scopes to WRITES (the AUTH-005 class), so only mutating routes
            // make the surface a control plane.
            if !is_mutating {
                continue;
            }
            if path_exempt(path, &exempt_substrings) {
                continue;
            }
            is_control_plane = true;
            let _ = has_path_param(path); // documented signal; mutating already qualifies.
            let covered = handler_authz
                .and_then(|m| m.get(handler))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if !covered {
                uncovered_handlers.push((method.to_owned(), path.to_owned(), handler.to_owned()));
            }
        }

        if !is_control_plane {
            continue;
        }
        // The surface is COVERED iff a router-level auth layer guards the whole chain OR every
        // mutating handler is individually covered (no uncovered handler).
        if has_auth_layer || uncovered_handlers.is_empty() {
            continue;
        }

        // An uncovered control-plane surface. Key it by file+router site.
        let key = surface_key(file, router_line);
        live_unauth_keys.insert(key.clone());

        // Frozen-baseline ratchet: a known pre-existing surface is ACCEPTED (no block).
        if frozen_baseline.contains(&key) {
            continue;
        }

        let holes = uncovered_handlers
            .iter()
            .map(|(m, p, h)| format!("{} {p} -> {h}()", m.to_uppercase()))
            .collect::<Vec<_>>()
            .join("; ");
        findings.insert(Finding::new(
            "AC-UNAUTHENTICATED-CONTROL-PLANE",
            &key,
            format!(
                "NEW unauthenticated HTTP control plane: the axum router at {file}:{router_line} mounts mutating route(s) [{holes}] whose handler(s) derive no caller identity (no recognized authz guard in the handler body and no router-level auth layer). Any network caller can invoke these writes. Add fail-closed authz before merge — see {REMEDIATION_DOCTRINE}. If a route is a genuinely unauthenticated read, declare its path in `exempt_path_substrings` (DATA)."
            ),
        ));
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
        let surface = observed["surfaces"].as_array().unwrap().first().unwrap();
        let line = surface["router_line"].as_u64().unwrap();
        let key = format!("fixture.rs::router@{line}");

        // Without baseline ⇒ blocked.
        assert!(
            evaluate_keyed(&policy(), &observed)
                .iter()
                .any(|f| f.code == "AC-UNAUTHENTICATED-CONTROL-PLANE" && f.key == key),
            "an un-baselined unauthenticated surface blocks"
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
}
