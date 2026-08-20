//! # cloud-ci-cedar-deploy-parity (GH #16 / ADR-0608)
//!
//! Deployed-vs-authored Cedar policy parity. Every `<cap>/iac/k8s/helm/templates/cedar.yaml`
//! ConfigMap ships an embedded Cedar policy that the in-cluster PDP loads at runtime. GH #16: those
//! ConfigMaps were stamped from a single byte-identical template carrying a BLANKET, ACTION-AGNOSTIC
//! `permit(principal, action, resource) when { ... }` — over-broad by construction (an unconstrained
//! action head permits EVERY action) and unrelated to each capability's AUTHORED policy. This gate is
//! the recurrence backstop: it refuses any deployed Cedar ConfigMap that carries an over-broad permit
//! or that grants more than the capability authored.
//!
//! ## Two invariants (DATA in `cedar-deploy-parity-policy.json`)
//! 1. **CHECK-A — no unconstrained-head permit.** A deployed ConfigMap permit whose HEAD leaves the
//!    action unconstrained (a bare `action`, not `action == Action::"…"` or `action in [ … ]`) is
//!    action-agnostic. A permit whose HEAD leaves bare `resource` and whose `when` clause carries no
//!    resource/scope predicate is resource-agnostic. Both shapes are over-broad for production Helm
//!    policy; authored PBAC (RBAC + ABAC) policy must name the action and constrain resource/scope.
//! 2. **CHECK-B — deployed allows ⊆ authored allows.** Cedar authorizes only when at least one
//!    `permit` applies, no `forbid` applies, and otherwise defaults to deny. Therefore a deployed
//!    permit must be authored, and any authored forbid that can restrict a deployed permit must also be
//!    deployed. A deployed ConfigMap with NO resolvable authored policy fails closed — parity cannot be
//!    proven against nothing.
//!
//! ## Born-blocking baseline (shrink-only; mirrors ADR-0605 `ignore[]`)
//! The blanket disarm — re-pointing each live service at its real (PBAC-core) authored policy — is a
//! SEQUENCED FOLLOW-UP, not this lane. So the known-blanket ConfigMap paths are recorded in
//! `policy.baseline.paths` plus `policy.baseline.policy_signatures`: each exception is GRANDFATHERED
//! (skipped by CHECK-A/CHECK-B) only when both its path and exact Cedar authorization signature match
//! the known blanket. It remains DOCUMENTED + time-boxed (`remove_by`) and SHRINK-ONLY —
//! `CDP-STALE-BASELINE` flags a baseline path that is gone or whose signature changed so it must be
//! dropped, after which it is fully checked. The baseline never grows by automation: a NEW or CHANGED
//! deployed ConfigMap is checked in full, so the gate blocks regressions from the first commit while
//! the disarm shrinks the baseline to empty.
//!
//! ## Kernel contract
//! - [`collect`] `(root, policy) -> observed` is the ONLY I/O: a hermetic, read-only fs walk for the
//!   deployed ConfigMaps + each capability's authored `.cedar`. No shell, no network, no clock, no VCS.
//! - [`evaluate_keyed`] `(policy, observed) -> BTreeSet<Finding>` is PURE and unit-testable without a
//!   filesystem. [`evaluate`] is its bare-verdict projection.
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `CDP-UNCONSTRAINED-PERMIT`     — a non-baselined deployed ConfigMap carries an action-agnostic permit (CHECK-A).
//! - `CDP-UNCONSTRAINED-RESOURCE`   — a non-baselined deployed ConfigMap carries a permit with bare `resource` and no resource/scope predicate (CHECK-A).
//! - `CDP-DEPLOYED-NOT-SUBSET`      — a deployed permit is absent from the capability's authored policy set (CHECK-B).
//! - `CDP-NO-AUTHORED-BASELINE`     — a non-baselined deployed ConfigMap has no resolvable authored policy (CHECK-B, fail-closed).
//! - `CDP-CEDAR-EXTRACT-FAILED`     — the Cedar policy could not be extracted/parsed from a deployed ConfigMap (fail-closed).
//! - `CDP-STALE-BASELINE`           — a baseline path is no longer blanket / no longer present (shrink-only self-clean).
//! - `CDP-POLICY-GATE-ID-MISMATCH`  — the policy `gate_id` is not [`GATE_ID`] (fail-closed).
//! - `CDP-POLICY-MALFORMED`         — the policy/observed view is structurally invalid or the scan found zero ConfigMaps (fail-closed against a vacuously-green run).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::{Value, json};

/// The gate id, matching the buck2 target stem + the policy `gate_id`.
pub const GATE_ID: &str = "cloud-ci-cedar-deploy-parity";

/// The default repo-relative suffix of a deployed Cedar ConfigMap (overridable via policy DATA).
pub const DEFAULT_DEPLOYED_SUFFIX: &str = "iac/k8s/helm/templates/cedar.yaml";

/// The SECOND place a deployed Cedar policy now lives.
///
/// The shared-microservice-chart cutover collapsed 71 per-service charts onto one chart and moved
/// each service's Cedar policy into its own `values.yaml` under `cedar.policy`. That silently took
/// those services out of this gate's scope: the scan saw 77 deployed policies before the cutover
/// and 7 after, and the ZERO-ConfigMaps fail-closed guard did not trip because 7 is not zero.
/// Coverage, not the baseline list, was the real regression. Scanning both shapes restores it.
pub const DEFAULT_DEPLOYED_VALUES_SUFFIX: &str = "iac/k8s/helm/values.yaml";

/// The blocking + structural violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 8] = [
    "CDP-UNCONSTRAINED-PERMIT",
    "CDP-UNCONSTRAINED-RESOURCE",
    "CDP-DEPLOYED-NOT-SUBSET",
    "CDP-NO-AUTHORED-BASELINE",
    "CDP-CEDAR-EXTRACT-FAILED",
    "CDP-STALE-BASELINE",
    "CDP-POLICY-GATE-ID-MISMATCH",
    "CDP-POLICY-MALFORMED",
];

/// The sentinel key for policy/scan-level (non-per-ConfigMap) findings.
const POLICY_KEY: &str = "<policy>";

// ---------------------------------------------------------------------------
// Collection (the only I/O; read-only, hermetic — no shell / network / VCS)
// ---------------------------------------------------------------------------

/// Errors collecting the observed view. Returned instead of panicking so the caller decides how to
/// surface them — an unreadable tree is a fail-closed error, never a silently empty scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    /// A read-only filesystem operation failed.
    Io(String),
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::Io(message) => write!(f, "cedar-deploy-parity io: {message}"),
        }
    }
}

impl std::error::Error for CollectError {}

/// Dirs never worth walking (heavy/irrelevant); keeps the scan deterministic and fast.
const SKIP_DIRS: [&str; 5] = ["target", ".git", "node_modules", "third-party", ".omc"];

/// Collect the observed view: every deployed Cedar ConfigMap + each capability's authored policy set.
///
/// The ONLY I/O. Walks the tree for files whose repo-relative path ends with the policy
/// `deployed_suffix`; for each, extracts the embedded Cedar policy, reduces every `permit` to
/// `{ normalized, action_unconstrained }`, records deployed `forbid` statements, computes the Cedar
/// authorization signature, and resolves + normalizes the capability's authored `permit`/`forbid` set
/// from `<cap>/<authored_subdir>/*.cedar`.
///
/// Emits `{ "configmaps": [ { "path", "capability", "extract_error"?, "permits": [{normalized,
/// action_unconstrained}], "forbids": [{normalized}], "policy_signature", "authored_found": bool,
/// "authored_permits": [normalized…], "authored_forbids": [normalized…] } ] }`.
pub fn collect(root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let suffix = policy
        .get("deployed_suffix")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_DEPLOYED_SUFFIX);
    let authored_subdirs = str_array(policy.get("authored_subdirs"));
    let authored_subdirs: Vec<&str> = if authored_subdirs.is_empty() {
        vec!["policy", "cedar"]
    } else {
        authored_subdirs.iter().map(String::as_str).collect()
    };
    let baseline_adr_exists =
        baseline_adr(policy).is_some_and(|adr| baseline_adr_file_exists(root, adr));

    let values_suffix = policy
        .get("deployed_values_suffix")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_DEPLOYED_VALUES_SUFFIX);

    let mut rel_paths: Vec<String> = Vec::new();
    walk_for_suffix(root, root, suffix, &mut rel_paths)?;
    let template_capabilities: BTreeSet<String> = rel_paths
        .iter()
        .map(|rel| capability_of(rel, suffix))
        .collect();
    // A service that still ships templates/cedar.yaml is covered by it; its values.yaml would be
    // the same policy counted twice. The template is the deployed artifact and wins.
    let mut values_paths: Vec<String> = Vec::new();
    walk_for_suffix(root, root, values_suffix, &mut values_paths)?;
    for rel in values_paths {
        if template_capabilities.contains(&capability_of(&rel, values_suffix)) {
            continue;
        }
        // A values.yaml is only a policy carrier if it actually carries one. Most do not — a
        // service with no Cedar policy has no `cedar.policy` block — and treating those as failed
        // extractions would turn "this service has no policy" into a gate finding, which is noise
        // rather than signal.
        let text = fs::read_to_string(root.join(&rel))
            .map_err(|e| CollectError::Io(format!("read {rel}: {e}")))?;
        if !extract_cedar_blocks(&text).is_empty() {
            rel_paths.push(rel);
        }
    }
    rel_paths.sort();

    let mut configmaps = Vec::new();
    for rel in &rel_paths {
        let abs = root.join(rel);
        let text = fs::read_to_string(&abs)
            .map_err(|e| CollectError::Io(format!("read {}: {e}", abs.display())))?;

        let mut permits: Vec<Value> = Vec::new();
        let mut forbids: Vec<Value> = Vec::new();
        let mut permit_set: BTreeSet<String> = BTreeSet::new();
        let mut forbid_set: BTreeSet<String> = BTreeSet::new();
        let mut extract_error: Option<String> = None;

        let blocks = extract_cedar_blocks(&text);
        if blocks.is_empty() {
            extract_error =
                Some("no `*.cedar` block-scalar key found under the ConfigMap".to_owned());
        }
        for block in &blocks {
            for stmt in split_statements(block) {
                match statement_effect(&stmt) {
                    Some(PolicyEffect::Permit) => match permit_head_constraints(&stmt) {
                        Ok(Some(constraints)) => {
                            let normalized = normalize_statement(&stmt);
                            permit_set.insert(normalized.clone());
                            permits.push(json!({
                                "normalized": normalized,
                                "action_unconstrained": constraints.action_unconstrained,
                                "resource_scope_unconstrained": constraints.resource_scope_unconstrained,
                            }));
                        }
                        Ok(None) => {}
                        Err(e) => {
                            // Fail closed: an unparseable permit could hide an over-broad grant.
                            extract_error = Some(e);
                        }
                    },
                    Some(PolicyEffect::Forbid) => {
                        let normalized = normalize_statement(&stmt);
                        forbid_set.insert(normalized.clone());
                        forbids.push(json!({ "normalized": normalized }));
                    }
                    None => {}
                }
            }
        }

        let policy_signature = if extract_error.is_none() {
            Value::String(authorization_signature(&permit_set, &forbid_set))
        } else {
            Value::Null
        };

        let capability = if rel.ends_with(suffix) {
            capability_of(rel, suffix)
        } else {
            capability_of(rel, values_suffix)
        };
        let authored = read_authored_policy(root, &capability, &authored_subdirs)?;

        configmaps.push(json!({
            "path": rel,
            "capability": capability,
            "extract_error": extract_error,
            "permits": permits,
            "forbids": forbids,
            "policy_signature": policy_signature,
            "authored_found": !authored.permits.is_empty() || !authored.forbids.is_empty(),
            "authored_permits": authored.permits.into_iter().collect::<Vec<_>>(),
            "authored_forbids": authored.forbids.into_iter().collect::<Vec<_>>(),
        }));
    }

    Ok(json!({ "configmaps": configmaps, "baseline_adr_exists": baseline_adr_exists }))
}

/// Recursive read-only walk collecting repo-relative paths ending with `suffix`.
fn walk_for_suffix(
    root: &Path,
    dir: &Path,
    suffix: &str,
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
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if SKIP_DIRS.contains(&name.as_ref()) {
                continue;
            }
            walk_for_suffix(root, &path, suffix, out)?;
        } else if file_type.is_file()
            && let Ok(rel) = path.strip_prefix(root)
        {
            let rel = rel.to_string_lossy().replace('\\', "/");
            if rel == suffix || rel.ends_with(&format!("/{suffix}")) {
                out.push(rel);
            }
        }
    }
    Ok(())
}

/// The capability prefix of a deployed ConfigMap path: the path with the `suffix` (and the joining
/// `/`) removed (`oya/identity/iac/k8s/helm/templates/cedar.yaml` -> `oya/identity`).
fn capability_of(rel: &str, suffix: &str) -> String {
    rel.strip_suffix(suffix)
        .map(|head| head.trim_end_matches('/').to_owned())
        .unwrap_or_default()
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct CedarStatementSets {
    permits: BTreeSet<String>,
    forbids: BTreeSet<String>,
}

/// Read + normalize the authored `permit`/`forbid` sets for a capability from
/// `<cap>/<subdir>/*.cedar`.
fn read_authored_policy(
    root: &Path,
    capability: &str,
    subdirs: &[&str],
) -> Result<CedarStatementSets, CollectError> {
    let mut out = CedarStatementSets::default();
    if capability.is_empty() {
        return Ok(out);
    }
    for subdir in subdirs {
        let dir = root.join(capability).join(subdir);
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(CollectError::Io(format!("read dir {}: {e}", dir.display()))),
        };
        for entry in entries {
            let entry = entry
                .map_err(|e| CollectError::Io(format!("read entry in {}: {e}", dir.display())))?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("cedar") {
                continue;
            }
            let text = fs::read_to_string(&path)
                .map_err(|e| CollectError::Io(format!("read {}: {e}", path.display())))?;
            for stmt in split_statements(&text) {
                let normalized = normalize_statement(&stmt);
                match statement_effect(&stmt) {
                    Some(PolicyEffect::Permit) => {
                        if matches!(permit_action_unconstrained(&stmt), Ok(Some(_))) {
                            out.permits.insert(normalized);
                        }
                    }
                    Some(PolicyEffect::Forbid) => {
                        out.forbids.insert(normalized);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Cedar text helpers (pure; exposed for tests)
// ---------------------------------------------------------------------------

/// Extract every Cedar block scalar from a Helm-templated ConfigMap: the body of each mapping key
/// ending in `.cedar` declared with a `|` block-scalar indicator. The Helm `{{- if … }}` / `{{- end }}`
/// wrapper lives at the document indentation, so it is naturally excluded (it dedents out of the block).
pub fn extract_cedar_blocks(yaml_text: &str) -> Vec<String> {
    let lines: Vec<&str> = yaml_text.lines().collect();
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if let Some(key_indent) = block_scalar_key_indent(lines[i]) {
            let mut body = String::new();
            let mut block_indent: Option<usize> = None;
            let mut j = i + 1;
            while j < lines.len() {
                let line = lines[j];
                if line.trim().is_empty() {
                    body.push('\n');
                    j += 1;
                    continue;
                }
                let ind = indent_of(line);
                if ind <= key_indent {
                    break;
                }
                let strip = *block_indent.get_or_insert(ind);
                let content = line.get(strip..).unwrap_or_else(|| line.trim_start());
                body.push_str(content);
                body.push('\n');
                j += 1;
            }
            blocks.push(body);
            i = j;
        } else {
            i += 1;
        }
    }
    blocks
}

/// `Some(indent)` if a line is a mapping key ending in `.cedar` with a `|` block-scalar indicator
/// (`policies.cedar: |`, `foo.cedar: |-`, …).
fn block_scalar_key_indent(line: &str) -> Option<usize> {
    let indent = indent_of(line);
    let trimmed = line.trim();
    let (key, rest) = trimmed.split_once(':')?;
    let key = key.trim_end();
    // `policies.cedar: |` in a rendered ConfigMap, or `policy: |` under the `cedar:` mapping of a
    // service values.yaml. Both carry the same authored policy text; only the container differs.
    if !key.ends_with(".cedar") && key != "policy" {
        return None;
    }
    rest.trim().starts_with('|').then_some(indent)
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// Strip `//` line comments from Cedar text, leaving string literals intact.
fn strip_line_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let mut in_str = false;
        let mut esc = false;
        let mut prev_slash = false;
        let mut cut = None;
        for (idx, ch) in line.char_indices() {
            if in_str {
                if esc {
                    esc = false;
                } else if ch == '\\' {
                    esc = true;
                } else if ch == '"' {
                    in_str = false;
                }
                prev_slash = false;
                continue;
            }
            match ch {
                '"' => {
                    in_str = true;
                    prev_slash = false;
                }
                '/' if prev_slash => {
                    cut = Some(idx - 1);
                    break;
                }
                '/' => prev_slash = true,
                _ => prev_slash = false,
            }
        }
        match cut {
            Some(at) => out.push_str(line.get(..at).unwrap_or("")),
            None => out.push_str(line),
        }
        out.push('\n');
    }
    out
}

/// Split Cedar text into statements on top-level `;` (respecting strings + `() [] {}` nesting).
pub fn split_statements(text: &str) -> Vec<String> {
    let cleaned = strip_line_comments(text);
    let mut stmts = Vec::new();
    let mut cur = String::new();
    let (mut paren, mut brace, mut brack) = (0i32, 0i32, 0i32);
    let mut in_str = false;
    let mut esc = false;
    for ch in cleaned.chars() {
        if in_str {
            cur.push(ch);
            if esc {
                esc = false;
            } else if ch == '\\' {
                esc = true;
            } else if ch == '"' {
                in_str = false;
            }
            continue;
        }
        match ch {
            '"' => {
                in_str = true;
                cur.push(ch);
            }
            '(' => {
                paren += 1;
                cur.push(ch);
            }
            ')' => {
                paren -= 1;
                cur.push(ch);
            }
            '{' => {
                brace += 1;
                cur.push(ch);
            }
            '}' => {
                brace -= 1;
                cur.push(ch);
            }
            '[' => {
                brack += 1;
                cur.push(ch);
            }
            ']' => {
                brack -= 1;
                cur.push(ch);
            }
            ';' if paren == 0 && brace == 0 && brack == 0 => {
                let trimmed = cur.trim();
                if !trimmed.is_empty() {
                    stmts.push(trimmed.to_owned());
                }
                cur.clear();
            }
            _ => cur.push(ch),
        }
    }
    let trimmed = cur.trim();
    if !trimmed.is_empty() {
        stmts.push(trimmed.to_owned());
    }
    stmts
}

/// Strip leading `@annotation(...)` heads (e.g. `@id("…")`) from a statement.
fn strip_annotations(stmt: &str) -> String {
    let mut rest = stmt.trim_start();
    while let Some(after_at) = rest.strip_prefix('@') {
        match after_at.find('(') {
            Some(open) => {
                let from_open = &after_at[open..];
                match matching_paren(from_open) {
                    Some(close) => rest = from_open[close + 1..].trim_start(),
                    None => break,
                }
            }
            None => break,
        }
    }
    rest.to_owned()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PolicyEffect {
    Permit,
    Forbid,
}

fn statement_effect(stmt: &str) -> Option<PolicyEffect> {
    let body = strip_annotations(stmt);
    let body = body.trim_start();
    if starts_with_keyword(body, "permit") {
        Some(PolicyEffect::Permit)
    } else if starts_with_keyword(body, "forbid") {
        Some(PolicyEffect::Forbid)
    } else {
        None
    }
}

/// Index (into `s`, which must start with `(`) of the `)` matching the opening `(`.
fn matching_paren(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_str = false;
    let mut esc = false;
    for (idx, ch) in s.char_indices() {
        if in_str {
            if esc {
                esc = false;
            } else if ch == '\\' {
                esc = true;
            } else if ch == '"' {
                in_str = false;
            }
            continue;
        }
        match ch {
            '"' => in_str = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

/// Parsed broadness facts for the Cedar permit head plus its resource/scope predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermitHeadConstraints {
    pub action_unconstrained: bool,
    pub resource_scope_unconstrained: bool,
}

/// For a Cedar statement: `Ok(None)` if it is not a `permit`; otherwise returns the permit's
/// broadness facts. `Err` if it is a `permit` whose head cannot be parsed (fail-closed).
pub fn permit_head_constraints(stmt: &str) -> Result<Option<PermitHeadConstraints>, String> {
    let body = strip_annotations(stmt);
    let body = body.trim_start();
    if !starts_with_keyword(body, "permit") {
        return Ok(None);
    }
    let after = body["permit".len()..].trim_start();
    let (head, after_head) = match after.strip_prefix('(') {
        Some(_) => {
            let close = matching_paren(after)
                .ok_or_else(|| format!("permit head has no closing paren: {stmt}"))?;
            (
                after.get(1..close).unwrap_or(""),
                after.get(close + 1..).unwrap_or(""),
            )
        }
        None => return Err(format!("permit not followed by a head paren: {stmt}")),
    };
    let slots = split_top_level_commas(head);
    let action = slots
        .get(1)
        .ok_or_else(|| format!("permit head has fewer than 3 scope slots: {stmt}"))?;
    let resource = slots
        .get(2)
        .ok_or_else(|| format!("permit head has fewer than 3 scope slots: {stmt}"))?;
    let action = collapse_ws(action);
    let resource = collapse_ws(resource);
    Ok(Some(PermitHeadConstraints {
        action_unconstrained: !head_slot_is_constrained(&action),
        resource_scope_unconstrained: !head_slot_is_constrained(&resource)
            && !has_resource_scope_predicate(after_head),
    }))
}

/// Back-compat helper for tests and callers that only need the action-axis result.
pub fn permit_action_unconstrained(stmt: &str) -> Result<Option<bool>, String> {
    Ok(permit_head_constraints(stmt)?.map(|facts| facts.action_unconstrained))
}

fn head_slot_is_constrained(slot: &str) -> bool {
    slot.contains("==")
        || slot.contains(" in ")
        || slot.contains(" in[")
        || slot.contains(" is ")
        || slot.contains(" is(")
}

fn has_resource_scope_predicate(after_head: &str) -> bool {
    let lower = after_head.to_ascii_lowercase();
    lower.contains("resource.")
        || lower.contains("resource[")
        || lower.contains("context.scope")
        || lower.contains("context.tenant")
        || lower.contains("scope_id")
}

/// Split a Cedar policy-head on top-level commas (respecting `[]`, `()`, and string literals).
fn split_top_level_commas(head: &str) -> Vec<String> {
    let mut slots = Vec::new();
    let mut cur = String::new();
    let (mut paren, mut brack) = (0i32, 0i32);
    let mut in_str = false;
    let mut esc = false;
    for ch in head.chars() {
        if in_str {
            cur.push(ch);
            if esc {
                esc = false;
            } else if ch == '\\' {
                esc = true;
            } else if ch == '"' {
                in_str = false;
            }
            continue;
        }
        match ch {
            '"' => {
                in_str = true;
                cur.push(ch);
            }
            '(' => {
                paren += 1;
                cur.push(ch);
            }
            ')' => {
                paren -= 1;
                cur.push(ch);
            }
            '[' => {
                brack += 1;
                cur.push(ch);
            }
            ']' => {
                brack -= 1;
                cur.push(ch);
            }
            ',' if paren == 0 && brack == 0 => {
                slots.push(cur.clone());
                cur.clear();
            }
            _ => cur.push(ch),
        }
    }
    slots.push(cur);
    slots
}

/// Canonicalize a statement for set comparison: drop annotations, then collapse all whitespace.
pub fn normalize_statement(stmt: &str) -> String {
    collapse_ws(&strip_annotations(stmt))
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn authorization_signature(permits: &BTreeSet<String>, forbids: &BTreeSet<String>) -> String {
    let mut material = Vec::new();
    for permit in permits {
        material.extend_from_slice(b"permit\0");
        material.extend_from_slice(permit.as_bytes());
        material.push(0);
    }
    for forbid in forbids {
        material.extend_from_slice(b"forbid\0");
        material.extend_from_slice(forbid.as_bytes());
        material.push(0);
    }
    format!("cedar-authz-fnv1a64:{:016x}", fnv1a64(&material))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn starts_with_keyword(s: &str, kw: &str) -> bool {
    match s.strip_prefix(kw) {
        Some(rest) => match rest.chars().next() {
            Some(c) => !c.is_ascii_alphanumeric() && c != '_',
            None => true,
        },
        None => false,
    }
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

/// The grandfathered known-blanket ConfigMap paths (`policy.baseline.paths`).
fn baseline_paths(policy: &Value) -> BTreeSet<String> {
    policy
        .get("baseline")
        .and_then(|b| b.get("paths"))
        .map(|v| str_array(Some(v)).into_iter().collect())
        .unwrap_or_default()
}

/// The exact Cedar authorization signatures allowed to use the baseline path exception.
fn baseline_policy_signatures(policy: &Value) -> BTreeSet<String> {
    policy
        .get("baseline")
        .and_then(|b| b.get("policy_signatures"))
        .map(|v| str_array(Some(v)).into_iter().collect())
        .unwrap_or_default()
}

fn baseline_text_field(policy: &Value, key: &str) -> Option<String> {
    policy
        .get("baseline")
        .and_then(|b| b.get(key))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

fn baseline_adr(policy: &Value) -> Option<&str> {
    policy
        .get("baseline")
        .and_then(|b| b.get("adr"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn baseline_adr_file_exists(root: &Path, adr: &str) -> bool {
    let Ok(entries) = fs::read_dir(root.join("docs/decisions")) else {
        return false;
    };
    let prefix = format!("{adr}-");
    entries.flatten().any(|entry| {
        entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with(&prefix) && name.ends_with(".md"))
    })
}

fn strict_adr_ref(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 8 && bytes.starts_with(b"ADR-") && bytes[4..].iter().all(u8::is_ascii_digit)
}

fn strict_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes[..4].iter().all(u8::is_ascii_digit)
        || !bytes[5..7].iter().all(u8::is_ascii_digit)
        || !bytes[8..].iter().all(u8::is_ascii_digit)
    {
        return false;
    }
    let Ok(year) = value[..4].parse::<u32>() else {
        return false;
    };
    let Ok(month) = value[5..7].parse::<u32>() else {
        return false;
    };
    let Ok(day) = value[8..].parse::<u32>() else {
        return false;
    };
    if month == 0 || month > 12 || day == 0 {
        return false;
    }
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => return false,
    };
    day <= max_day
}

fn is_leap_year(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}
fn baseline_text_field_present(policy: &Value, key: &str) -> bool {
    policy
        .get("baseline")
        .and_then(|b| b.get(key))
        .and_then(Value::as_str)
        .is_some_and(|s| !s.trim().is_empty())
}

fn baseline_signature_matches(cm: &Value, signatures: &BTreeSet<String>) -> bool {
    cm.get("policy_signature")
        .and_then(Value::as_str)
        .is_some_and(|signature| signatures.contains(signature))
}

fn str_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|seq| {
            seq.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// Whether a ConfigMap observed view carries at least one action-unconstrained permit.
fn has_unconstrained_permit(cm: &Value) -> bool {
    cm.get("permits")
        .and_then(Value::as_array)
        .is_some_and(|permits| {
            permits.iter().any(|p| {
                p.get("action_unconstrained")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            })
        })
}

/// Pure evaluator. `policy` is DATA (`cedar-deploy-parity-policy.json`); `observed` is the view shaped
/// by [`collect`]. RED iff a non-baselined deployed ConfigMap carries an action-agnostic permit or
/// bare-resource permit with no resource/scope predicate (CHECK-A), grants more than the capability
/// authored (CHECK-B), or fails closed (extract failure / missing authored baseline / stale baseline / structural).
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if policy.get("gate_id").and_then(Value::as_str) != Some(GATE_ID) {
        findings.insert(Finding::new(
            "CDP-POLICY-GATE-ID-MISMATCH",
            POLICY_KEY,
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    let Some(configmaps) = observed.get("configmaps").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "CDP-POLICY-MALFORMED",
            POLICY_KEY,
            "observed view has no `configmaps` array; the collector did not run — fail closed",
        ));
        return findings;
    };
    if configmaps.is_empty() {
        findings.insert(Finding::new(
            "CDP-POLICY-MALFORMED",
            POLICY_KEY,
            "scan found ZERO deployed Cedar ConfigMaps; the `deployed_suffix`/scan root is likely wrong — fail closed against a vacuously-green run",
        ));
        return findings;
    }

    let baseline = baseline_paths(policy);
    let baseline_signatures = baseline_policy_signatures(policy);
    if !baseline.is_empty() && baseline_signatures.is_empty() {
        findings.insert(Finding::new(
            "CDP-POLICY-MALFORMED",
            POLICY_KEY,
            "baseline paths require baseline.policy_signatures; path-only grandfathering is not allowed",
        ));
    }
    if !baseline.is_empty() {
        for key in ["reason", "remove_by", "adr"] {
            if !baseline_text_field_present(policy, key) {
                findings.insert(Finding::new(
                    "CDP-POLICY-MALFORMED",
                    POLICY_KEY,
                    format!("baseline paths require non-empty baseline.{key}; broad-policy exceptions must be audited, time-boxed, and decision-linked"),
                ));
            }
        }
        let remove_by = baseline_text_field(policy, "remove_by");
        if remove_by
            .as_deref()
            .is_none_or(|date| !strict_iso_date(date))
        {
            findings.insert(Finding::new(
                "CDP-POLICY-MALFORMED",
                POLICY_KEY,
                "baseline.remove_by must be a strict valid YYYY-MM-DD date",
            ));
        }

        let adr = baseline_text_field(policy, "adr");
        if adr.as_deref().is_none_or(|value| !strict_adr_ref(value)) {
            findings.insert(Finding::new(
                "CDP-POLICY-MALFORMED",
                POLICY_KEY,
                "baseline.adr must be a strict ADR-NNNN reference",
            ));
        } else if observed.get("baseline_adr_exists").and_then(Value::as_bool) != Some(true) {
            findings.insert(Finding::new(
                "CDP-POLICY-MALFORMED",
                POLICY_KEY,
                "baseline.adr must resolve to an existing docs/decisions/ADR-NNNN-*.md record",
            ));
        }
    }
    let observed_paths: BTreeSet<&str> = configmaps
        .iter()
        .filter_map(|cm| cm.get("path").and_then(Value::as_str))
        .collect();

    // Shrink-only self-clean: a baseline path that is gone or whose exact authz signature changed
    // must be dropped; changed ConfigMaps are then fully checked below instead of blindly skipped.
    for path in &baseline {
        match configmaps
            .iter()
            .find(|cm| cm.get("path").and_then(Value::as_str) == Some(path.as_str()))
        {
            None => {
                if !observed_paths.contains(path.as_str()) {
                    findings.insert(Finding::new(
                        "CDP-STALE-BASELINE",
                        path,
                        "baseline path no longer present in the scan — drop it from policy.baseline.paths",
                    ));
                }
            }
            Some(cm) => {
                let extract_ok = cm.get("extract_error").map(Value::is_null).unwrap_or(true);
                if !extract_ok || !baseline_signature_matches(cm, &baseline_signatures) {
                    findings.insert(Finding::new(
                        "CDP-STALE-BASELINE",
                        path,
                        "baseline path Cedar authorization signature no longer matches the grandfathered blanket — drop it from policy.baseline.paths so it is fully checked",
                    ));
                } else if !has_unconstrained_permit(cm) {
                    findings.insert(Finding::new(
                        "CDP-STALE-BASELINE",
                        path,
                        "baseline path no longer carries a blanket (action-agnostic) permit — drop it from policy.baseline.paths so it is fully checked",
                    ));
                }
            }
        }
    }

    for cm in configmaps {
        let Some(path) = cm.get("path").and_then(Value::as_str) else {
            findings.insert(Finding::new(
                "CDP-POLICY-MALFORMED",
                POLICY_KEY,
                "a configmap observed entry is missing its `path`",
            ));
            continue;
        };
        if baseline.contains(path) && baseline_signature_matches(cm, &baseline_signatures) {
            continue; // grandfathered exact known-blanket signature; tracked + shrink-only above.
        }

        // Fail closed: an un-extractable Cedar ConfigMap could hide an over-broad grant.
        if let Some(err) = cm.get("extract_error").and_then(Value::as_str) {
            findings.insert(Finding::new(
                "CDP-CEDAR-EXTRACT-FAILED",
                path,
                format!("could not extract/parse the deployed Cedar policy: {err}"),
            ));
            continue;
        }

        let permits = cm
            .get("permits")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        // CHECK-A: no action-agnostic permit and no bare-resource permit without a resource/scope predicate.
        for permit in &permits {
            if permit
                .get("action_unconstrained")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                findings.insert(Finding::new(
                    "CDP-UNCONSTRAINED-PERMIT",
                    path,
                    "deployed permit leaves the action unconstrained (action-agnostic blanket grant); constrain it with `action == Action::\"…\"` or `action in [ … ]`",
                ));
            }
            if permit
                .get("resource_scope_unconstrained")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                findings.insert(Finding::new(
                    "CDP-UNCONSTRAINED-RESOURCE",
                    path,
                    "deployed permit leaves `resource` unconstrained and has no resource/scope predicate; constrain it with `resource is …`, `resource in …`, or an explicit resource/scope `when` clause",
                ));
            }
        }

        // CHECK-B: Cedar semantics require deployed allows ⊆ authored allows. A deployed permit must
        // be authored, and authored forbids must be present whenever deployed permits could otherwise
        // allow a request that authored policy would deny. No deployed permits means Cedar default-deny,
        // so missing forbids cannot widen authorization in that case.
        let authored_found = cm
            .get("authored_found")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if !authored_found {
            findings.insert(Finding::new(
                "CDP-NO-AUTHORED-BASELINE",
                path,
                "deployed ConfigMap has no resolvable authored policy (<cap>/{policy,cedar}/*.cedar) to prove parity against — fail closed",
            ));
            continue;
        }
        let authored: BTreeSet<&str> = cm
            .get("authored_permits")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        for permit in &permits {
            let normalized = permit
                .get("normalized")
                .and_then(Value::as_str)
                .unwrap_or("");
            if !authored.contains(normalized) {
                findings.insert(Finding::new(
                    "CDP-DEPLOYED-NOT-SUBSET",
                    path,
                    "a deployed permit is absent from the capability's authored policy set (deployed grants more than authored)",
                ));
            }
        }

        if !permits.is_empty() {
            let deployed_forbids: BTreeSet<&str> = cm
                .get("forbids")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|forbid| forbid.get("normalized").and_then(Value::as_str))
                        .collect()
                })
                .unwrap_or_default();
            let authored_forbids: BTreeSet<&str> = cm
                .get("authored_forbids")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();

            for forbid in authored_forbids {
                if !deployed_forbids.contains(forbid) {
                    findings.insert(Finding::new(
                        "CDP-DEPLOYED-NOT-SUBSET",
                        path,
                        "an authored forbid is absent from the deployed policy; Cedar forbid/default-deny semantics cannot prove deployed allows are a subset of authored allows",
                    ));
                }
            }
        }
    }

    findings
}

/// Bare-code projection of [`evaluate_keyed`] — the single source of truth for the verdict.
pub fn evaluate(policy: &Value, observed: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(policy, observed))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLANKET: &str = r#"
permit(
  principal,
  action,
  resource
) when {
  resource.microservice == "{{ .Values.microservice.id }}" &&
  principal.tenant_class == "{{ .Values.microservice.tenantClass }}"
};
forbid(
  principal,
  action,
  resource
) when {
  resource.microservice == "{{ .Values.microservice.id }}" &&
  principal.tenant_class != "{{ .Values.microservice.tenantClass }}"
};
"#;

    const BLANKET_SIGNATURE: &str = "cedar-authz-fnv1a64:bb36847225f2f7ff";

    fn configmap_yaml(cedar: &str) -> String {
        let indented = cedar
            .lines()
            .map(|l| format!("    {l}"))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "{{{{- if .Values.cedar.enabled }}}}\napiVersion: v1\nkind: ConfigMap\ndata:\n  policies.cedar: |\n{indented}\n{{{{- end }}}}\n"
        )
    }

    #[test]
    fn extracts_cedar_block_from_helm_configmap() {
        let yaml = configmap_yaml(BLANKET);
        let blocks = extract_cedar_blocks(&yaml);
        assert_eq!(blocks.len(), 1, "one .cedar block expected");
        assert!(blocks[0].contains("permit("));
        assert!(
            !blocks[0].contains("{{- end }}"),
            "wrapper must not leak in"
        );
    }

    #[test]
    fn blanket_permit_is_action_unconstrained() {
        let stmts = split_statements(BLANKET);
        let permit = stmts
            .iter()
            .find(|s| s.trim_start().starts_with("permit"))
            .unwrap();
        assert_eq!(permit_action_unconstrained(permit), Ok(Some(true)));
    }

    #[test]
    fn forbid_is_not_a_permit() {
        let forbid = "forbid ( principal, action, resource ) when { true }";
        assert_eq!(permit_action_unconstrained(forbid), Ok(None));
    }

    #[test]
    fn action_equals_is_constrained() {
        let p = r#"@id("x") permit ( principal == W::"a", action == Action::"kms.Decrypt", resource is Secret ) when { true }"#;
        assert_eq!(permit_action_unconstrained(p), Ok(Some(false)));
    }

    #[test]
    fn action_in_list_is_constrained_even_with_commas() {
        let p = r#"permit ( principal, action in [Action::"a", Action::"b"], resource )"#;
        assert_eq!(permit_action_unconstrained(p), Ok(Some(false)));
    }

    #[test]
    fn bare_resource_with_resource_scope_predicate_is_constrained() {
        let p = r#"permit ( principal, action == Action::"doc.Read", resource ) when { principal.tenant_id == resource.tenant_id }"#;
        assert_eq!(
            permit_head_constraints(p),
            Ok(Some(PermitHeadConstraints {
                action_unconstrained: false,
                resource_scope_unconstrained: false,
            }))
        );
    }

    #[test]
    fn bare_resource_without_scope_predicate_is_unconstrained() {
        let p = r#"permit ( principal, action == Action::"doc.Read", resource );"#;
        assert_eq!(
            permit_head_constraints(p),
            Ok(Some(PermitHeadConstraints {
                action_unconstrained: false,
                resource_scope_unconstrained: true,
            }))
        );
    }

    #[test]
    fn malformed_permit_head_fails_closed() {
        assert!(permit_action_unconstrained("permit ( principal, action").is_err());
        assert!(permit_action_unconstrained("permit principal, action, resource").is_err());
    }

    #[test]
    fn blanket_authorization_signature_is_stable() {
        let mut permits = BTreeSet::new();
        let mut forbids = BTreeSet::new();
        for stmt in split_statements(BLANKET) {
            match statement_effect(&stmt) {
                Some(PolicyEffect::Permit) => {
                    permits.insert(normalize_statement(&stmt));
                }
                Some(PolicyEffect::Forbid) => {
                    forbids.insert(normalize_statement(&stmt));
                }
                None => {}
            }
        }
        assert_eq!(
            authorization_signature(&permits, &forbids),
            BLANKET_SIGNATURE
        );
    }

    fn policy(baseline: &[&str]) -> Value {
        json!({
            "gate_id": GATE_ID,
            "deployed_suffix": DEFAULT_DEPLOYED_SUFFIX,
            "authored_subdirs": ["policy", "cedar"],
            "baseline": {
                "reason": "unit-test audited blanket exception",
                "remove_by": "2026-12-31",
                "adr": "ADR-0608",
                "paths": baseline,
                "policy_signatures": [BLANKET_SIGNATURE]
            }
        })
    }

    fn cm(path: &str, action_unconstrained: bool, authored_found: bool) -> Value {
        let normalized = if action_unconstrained {
            "permit ( principal, action, resource )".to_owned()
        } else {
            "permit ( principal, action == Action::\"a\", resource is Doc )".to_owned()
        };
        let signature = if action_unconstrained {
            BLANKET_SIGNATURE
        } else {
            "cedar-authz-fnv1a64:changed"
        };
        json!({
            "path": path,
            "capability": "oya/x",
            "extract_error": Value::Null,
            "permits": [ {
                "normalized": normalized,
                "action_unconstrained": action_unconstrained,
                "resource_scope_unconstrained": action_unconstrained
            } ],
            "forbids": [],
            "policy_signature": signature,
            "authored_found": authored_found,
            "authored_permits": ["permit ( principal, action == Action::\"a\", resource is Doc )"],
            "authored_forbids": [],
        })
    }

    #[test]
    fn unconstrained_permit_outside_baseline_is_red() {
        let observed =
            json!({ "configmaps": [ cm("oya/x/iac/k8s/helm/templates/cedar.yaml", true, true) ] });
        let codes: BTreeSet<String> = evaluate_keyed(&policy(&[]), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("CDP-UNCONSTRAINED-PERMIT"), "{codes:?}");
    }

    #[test]
    fn baselined_blanket_is_grandfathered_green() {
        let path = "oya/x/iac/k8s/helm/templates/cedar.yaml";
        let observed =
            json!({ "configmaps": [ cm(path, true, true) ], "baseline_adr_exists": true });
        let report = evaluate(&policy(&[path]), &observed);
        assert_eq!(report.verdict, Verdict::Green, "{:?}", report.violations);
    }

    #[test]
    fn baseline_paths_require_audited_exception_metadata() {
        let path = "oya/x/iac/k8s/helm/templates/cedar.yaml";
        let observed = json!({ "configmaps": [ cm(path, true, true) ] });
        let malformed_policy = json!({
            "gate_id": GATE_ID,
            "deployed_suffix": DEFAULT_DEPLOYED_SUFFIX,
            "authored_subdirs": ["policy", "cedar"],
            "baseline": { "paths": [path], "policy_signatures": [BLANKET_SIGNATURE] }
        });
        let codes: BTreeSet<String> = evaluate_keyed(&malformed_policy, &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("CDP-POLICY-MALFORMED"), "{codes:?}");
    }

    #[test]
    fn baseline_remove_by_requires_strict_valid_date() {
        let path = "oya/x/iac/k8s/helm/templates/cedar.yaml";
        let mut malformed_policy = policy(&[path]);
        malformed_policy["baseline"]["remove_by"] = json!("2026-02-31");
        let observed =
            json!({ "configmaps": [ cm(path, true, true) ], "baseline_adr_exists": true });
        let codes: BTreeSet<String> = evaluate_keyed(&malformed_policy, &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("CDP-POLICY-MALFORMED"), "{codes:?}");
    }

    #[test]
    fn baseline_adr_requires_strict_existing_reference() {
        let path = "oya/x/iac/k8s/helm/templates/cedar.yaml";
        let mut malformed_policy = policy(&[path]);
        malformed_policy["baseline"]["adr"] = json!("ADR-608");
        let observed =
            json!({ "configmaps": [ cm(path, true, true) ], "baseline_adr_exists": true });
        let codes: BTreeSet<String> = evaluate_keyed(&malformed_policy, &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("CDP-POLICY-MALFORMED"), "{codes:?}");

        let observed_without_adr =
            json!({ "configmaps": [ cm(path, true, true) ], "baseline_adr_exists": false });
        let codes: BTreeSet<String> = evaluate_keyed(&policy(&[path]), &observed_without_adr)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("CDP-POLICY-MALFORMED"), "{codes:?}");
    }

    #[test]
    fn baseline_for_now_clean_configmap_is_stale() {
        let path = "oya/x/iac/k8s/helm/templates/cedar.yaml";
        // Constrained permit that IS in authored -> no CHECK-A/B finding, so the baseline is stale.
        let observed =
            json!({ "configmaps": [ cm(path, false, true) ], "baseline_adr_exists": true });
        let codes: BTreeSet<String> = evaluate_keyed(&policy(&[path]), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("CDP-STALE-BASELINE"), "{codes:?}");
    }

    #[test]
    fn changed_baselined_path_is_fully_checked_not_blindly_skipped() {
        let path = "oya/x/iac/k8s/helm/templates/cedar.yaml";
        let mut changed = cm(path, true, true);
        changed["policy_signature"] = json!("cedar-authz-fnv1a64:changed");
        let observed = json!({ "configmaps": [ changed ], "baseline_adr_exists": true });
        let codes: BTreeSet<String> = evaluate_keyed(&policy(&[path]), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("CDP-STALE-BASELINE"), "{codes:?}");
        assert!(codes.contains("CDP-UNCONSTRAINED-PERMIT"), "{codes:?}");
    }

    #[test]
    fn deployed_not_subset_of_authored_is_red() {
        // Constrained-action permit (passes CHECK-A) but absent from authored set.
        let path = "oya/x/iac/k8s/helm/templates/cedar.yaml";
        let observed = json!({ "configmaps": [ {
            "path": path,
            "capability": "oya/x",
            "extract_error": Value::Null,
            "permits": [ { "normalized": "permit ( principal, action == Action::\"unauthored\", resource )", "action_unconstrained": false } ],
            "authored_found": true,
            "authored_permits": ["permit ( principal, action == Action::\"a\", resource )"],
        } ] });
        let codes: BTreeSet<String> = evaluate_keyed(&policy(&[]), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("CDP-DEPLOYED-NOT-SUBSET"), "{codes:?}");
        assert!(!codes.contains("CDP-UNCONSTRAINED-PERMIT"), "{codes:?}");
    }

    #[test]
    fn missing_authored_forbid_is_deployed_not_subset() {
        let path = "oya/x/iac/k8s/helm/templates/cedar.yaml";
        let permit = "permit ( principal, action == Action::\"a\", resource )";
        let forbid =
            "forbid ( principal, action == Action::\"a\", resource ) when { resource.blocked }";
        let observed = json!({ "configmaps": [ {
            "path": path,
            "capability": "oya/x",
            "extract_error": Value::Null,
            "permits": [ { "normalized": permit, "action_unconstrained": false } ],
            "forbids": [],
            "policy_signature": "cedar-authz-fnv1a64:deployed",
            "authored_found": true,
            "authored_permits": [permit],
            "authored_forbids": [forbid],
        } ] });
        let codes: BTreeSet<String> = evaluate_keyed(&policy(&[]), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("CDP-DEPLOYED-NOT-SUBSET"), "{codes:?}");
    }

    #[test]
    fn deployed_default_deny_without_permits_does_not_widen_authored_policy() {
        let path = "oya/x/iac/k8s/helm/templates/cedar.yaml";
        let observed = json!({ "configmaps": [ {
            "path": path,
            "capability": "oya/x",
            "extract_error": Value::Null,
            "permits": [],
            "forbids": [],
            "policy_signature": "cedar-authz-fnv1a64:deny-all",
            "authored_found": true,
            "authored_permits": ["permit ( principal, action == Action::\"a\", resource )"],
            "authored_forbids": ["forbid ( principal, action == Action::\"a\", resource ) when { resource.blocked }"],
        } ] });
        let report = evaluate(&policy(&[]), &observed);
        assert_eq!(report.verdict, Verdict::Green, "{:?}", report.violations);
    }

    #[test]
    fn missing_authored_fails_closed() {
        let path = "oya/x/iac/k8s/helm/templates/cedar.yaml";
        let observed = json!({ "configmaps": [ cm(path, false, false) ] });
        let codes: BTreeSet<String> = evaluate_keyed(&policy(&[]), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("CDP-NO-AUTHORED-BASELINE"), "{codes:?}");
    }

    #[test]
    fn extract_failure_fails_closed() {
        let path = "oya/x/iac/k8s/helm/templates/cedar.yaml";
        let observed = json!({ "configmaps": [ {
            "path": path, "capability": "oya/x",
            "extract_error": "no .cedar block found",
            "permits": [], "authored_found": true, "authored_permits": [],
        } ] });
        let codes: BTreeSet<String> = evaluate_keyed(&policy(&[]), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("CDP-CEDAR-EXTRACT-FAILED"), "{codes:?}");
    }

    #[test]
    fn zero_configmaps_fails_closed() {
        let observed = json!({ "configmaps": [] });
        let codes: BTreeSet<String> = evaluate_keyed(&policy(&[]), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("CDP-POLICY-MALFORMED"), "{codes:?}");
    }

    #[test]
    fn wrong_gate_id_fails_closed() {
        let observed =
            json!({ "configmaps": [ cm("oya/x/iac/k8s/helm/templates/cedar.yaml", false, true) ] });
        let mut p = policy(&[]);
        p["gate_id"] = json!("wrong");
        let codes: BTreeSet<String> = evaluate_keyed(&p, &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("CDP-POLICY-GATE-ID-MISMATCH"), "{codes:?}");
    }
}
