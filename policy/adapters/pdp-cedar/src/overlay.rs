use std::collections::HashSet;
use std::str::FromStr;

use cedar_policy::{Effect, PolicyId, PolicySet, Schema, ValidationMode, Validator};
use shared_pdp_kernel::PdpError;

/// Separator joining an overlay's owning tenant to its authored policy id when
/// the overlay policy is merged into the per-tenant set. A global `@id` is a
/// Cedar id which never contains `/` in the seed corpus; namespacing overlay
/// ids this way keeps them disjoint from global ids and from each other, and a
/// residual collision still fails closed via `PolicySet::add`.
const OVERLAY_ID_SEP: &str = "/";

/// Compile one tenant overlay into a MERGED policy set `global ∪ overlay`.
/// Cross-tenant isolation in the merged set is enforced at RUNTIME by the
/// global `structural-tenant-isolation` forbid (cloned in below) over the
/// schema-required `tenant_id` attribute — forbid-overrides-permit makes that
/// boundary unconditional for any overlay permit shape. The load-time
/// `reject_cross_tenant_overlay_policy` check is defense-in-depth hygiene only,
/// not the boundary. Overlays are still strict-validated as a merged set so a
/// schema-inconsistent overlay fails closed. Overlay policy ids are namespaced
/// by the owning tenant so they cannot collide with the global set or another
/// overlay; a residual collision still fails closed via `PolicySet::add`.
pub(super) fn compile_tenant_overlay(
    tenant_id: &str,
    overlay_src: &str,
    global: &PolicySet,
    schema: &Schema,
    known_tenants: &HashSet<&str>,
) -> Result<PolicySet, PdpError> {
    let parsed = PolicySet::from_str(overlay_src).map_err(|e| PdpError::BundleRejected {
        detail: format!("tenant {tenant_id} overlay rejected: {e}"),
    })?;
    // Start from a clone of the global set so the structural forbid (and every
    // global permit) is present in the per-tenant decision — ONE algorithm.
    let mut merged = global.clone();
    for policy in parsed.policies() {
        // Re-key by the authored @id (stable attribution), then namespace by
        // the owning tenant so an overlay id can never collide with a global
        // id or another tenant's overlay id.
        let authored = match policy.annotation("id") {
            Some(id) => policy.new_id(PolicyId::new(id)),
            None => policy.clone(),
        };
        // Defense-in-depth hygiene only — NOT the isolation boundary. The
        // global `structural-tenant-isolation` forbid (cloned into this merged
        // set just above) plus the schema-required `tenant_id` attribute are
        // the sole, formally-verified tenant-isolation boundary
        // (forbid-overrides-permit). This load-time check merely rejects
        // obviously-misauthored overlays early; it adds nothing to that runtime
        // guarantee and must never be relied on in its place.
        reject_cross_tenant_overlay_policy(tenant_id, &authored, known_tenants)?;
        let namespaced_id = format!("{tenant_id}{OVERLAY_ID_SEP}{}", authored.id());
        let namespaced = authored.new_id(PolicyId::new(&namespaced_id));
        merged
            .add(namespaced)
            .map_err(|e| PdpError::BundleRejected {
                detail: format!("tenant {tenant_id} overlay policy {namespaced_id} rejected: {e}"),
            })?;
    }
    // Strict-validate the MERGED set so an overlay that is individually
    // parseable but inconsistent with the schema (or the global set) fails
    // closed before it can serve.
    let validation = Validator::new(schema.clone()).validate(&merged, ValidationMode::Strict);
    if !validation.validation_passed() {
        let errors: Vec<String> = validation
            .validation_errors()
            .map(|e| e.to_string())
            .collect();
        return Err(PdpError::BundleRejected {
            detail: format!(
                "tenant {tenant_id} overlay strict validation failed: {}",
                errors.join("; ")
            ),
        });
    }
    Ok(merged)
}

/// Best-effort, DEFENSE-IN-DEPTH load-time hygiene check on overlay policies.
///
/// NOT THE TENANT-ISOLATION BOUNDARY. The sole, formally-verified
/// tenant-isolation boundary is the global `structural-tenant-isolation`
/// `forbid` (forbid-overrides-permit; arXiv 2403.04651) backed by the
/// schema-required `tenant_id` attribute on every principal/resource. That
/// forbid is cloned into EVERY per-tenant merged set, so it denies any
/// cross-tenant grant whatever shape an overlay permit takes — this check
/// contributes NOTHING to that runtime guarantee and must never be relied on
/// in its place. It exists only to reject obviously-misauthored overlays
/// early (fail-closed) and to keep a foreign-tenant-literal smell out of the
/// corpus.
///
/// What it rejects (fail-closed `BundleRejected`):
/// - any policy whose EST names a KNOWN foreign tenant id as a string literal
///   (an unambiguous cross-tenant authoring smell);
/// - any `permit` that is not CONSERVATIVELY confined to its tenant by the
///   canonical same-tenant guard `principal.tenant_id == resource.tenant_id`
///   appearing as a TOP-LEVEL CONJUNCT of a `when` clause (see
///   [`permit_is_tenant_confined`] for the exact, sound acceptance rule).
///
/// A `forbid` can only ever DENY, so it is always safe.
fn reject_cross_tenant_overlay_policy(
    tenant_id: &str,
    policy: &cedar_policy::Policy,
    known_tenants: &HashSet<&str>,
) -> Result<(), PdpError> {
    let json = policy.to_json().map_err(|e| PdpError::BundleRejected {
        detail: format!(
            "tenant {tenant_id} overlay policy {} not introspectable: {e}",
            policy.id()
        ),
    })?;
    // A foreign tenant id appearing as a string literal is an unambiguous
    // cross-tenant authoring smell — reject regardless of shape.
    if let Some(foreign) = first_foreign_tenant_literal(&json, tenant_id, known_tenants) {
        return Err(PdpError::BundleRejected {
            detail: format!(
                "tenant {tenant_id} overlay policy {} names foreign tenant {foreign:?} — \
                 an overlay may never reference another tenant",
                policy.id()
            ),
        });
    }
    // A forbid can only ever DENY; it cannot grant across (or within) a tenant.
    if policy.effect() == Effect::Forbid {
        return Ok(());
    }
    // A permit must be CONSERVATIVELY tenant-confined: the same-tenant guard
    // must bind unconditionally as a top-level conjunct of a `when` clause and
    // no `unless`/`||`/`!` may weaken or invert it. A non-binding occurrence
    // (in an `unless`, behind `||`/`!`, or absent) is rejected fail-closed.
    if permit_is_tenant_confined(&json) {
        Ok(())
    } else {
        Err(PdpError::BundleRejected {
            detail: format!(
                "tenant {tenant_id} overlay permit {} is not tenant-confined: every overlay \
                 permit must carry the same-tenant guard \
                 `principal.tenant_id == resource.tenant_id` as a top-level conjunct of a \
                 `when` clause, with no `unless` clause and no enclosing `||`/`!`",
                policy.id()
            ),
        })
    }
}

/// Sound, conservative, fail-closed check that a `permit` EST is confined to a
/// single tenant by the same-tenant guard. Operates on the policy EST that
/// cedar-policy emits via `Policy::to_json`:
///
/// ```text
/// { "effect": "permit", ..., "conditions": [ { "kind": "when"|"unless", "body": <expr> }, ... ] }
/// ```
///
/// A permit is accepted ONLY when BOTH hold:
/// 1. it has NO `unless` clause (an `unless` can defeat any `when` guard), and
/// 2. at least one `when` clause has the same-tenant equality as a member of
///    its TOP-LEVEL CONJUNCTIVE SPINE — the leaves reached by descending only
///    through `&&` nodes. The walk NEVER descends into `||`, `!`, or any other
///    operator, so a guard buried under a disjunction/negation (where it does
///    not unconditionally bind) is NOT accepted.
///
/// Cedar AND-s all `when` clauses together, so a guard that is an unconditional
/// conjunct of any one `when` clause confines the whole permit. This rejects
/// every known evasion shape (`|| true`, `unless { true }`, guard-in-`unless`,
/// `!(guard)`, `guard || (1==1)`) while accepting the canonical, operand-
/// swapped, parenthesized (parens are transparent in the EST), and
/// `&&`-nested legitimate forms.
fn permit_is_tenant_confined(policy_json: &serde_json::Value) -> bool {
    let Some(conditions) = policy_json
        .get("conditions")
        .and_then(serde_json::Value::as_array)
    else {
        return false;
    };
    let mut has_binding_when = false;
    for condition in conditions {
        let kind = condition.get("kind").and_then(serde_json::Value::as_str);
        let Some(body) = condition.get("body") else {
            return false;
        };
        match kind {
            // Any `unless` can flip the decision back to deny-by-omission for a
            // matching principal, so we cannot soundly accept a permit that
            // carries one. Fail closed.
            Some("unless") => return false,
            Some("when") => {
                if conjunctive_spine_has_same_tenant_guard(body) {
                    has_binding_when = true;
                }
            }
            // Unknown condition kind — fail closed.
            _ => return false,
        }
    }
    has_binding_when
}

/// True iff the same-tenant guard is a member of the top-level conjunctive
/// spine of `expr`: recurse ONLY into `&&` operands, and at every leaf test
/// for the canonical same-tenant equality. Never descends into `||`, `!`, or
/// any other node, so a guard that does not unconditionally bind is not found.
fn conjunctive_spine_has_same_tenant_guard(expr: &serde_json::Value) -> bool {
    if let Some(and) = expr.get("&&").and_then(serde_json::Value::as_object) {
        let left = and
            .get("left")
            .is_some_and(conjunctive_spine_has_same_tenant_guard);
        let right = and
            .get("right")
            .is_some_and(conjunctive_spine_has_same_tenant_guard);
        return left || right;
    }
    is_same_tenant_equality(expr)
}

/// True iff `expr` is exactly the same-tenant equality
/// `principal.tenant_id == resource.tenant_id` (either operand order):
/// an `==` node whose two operands are BOTH a `tenant_id` attribute access,
/// one on the `principal` var and one on the `resource` var.
fn is_same_tenant_equality(expr: &serde_json::Value) -> bool {
    let Some(eq) = expr.get("==").and_then(serde_json::Value::as_object) else {
        return false;
    };
    let (Some(left), Some(right)) = (eq.get("left"), eq.get("right")) else {
        return false;
    };
    let (Some(l), Some(r)) = (
        tenant_id_attr_access_var(left),
        tenant_id_attr_access_var(right),
    ) else {
        return false;
    };
    (l == "principal" && r == "resource") || (l == "resource" && r == "principal")
}

/// If `node` is an EST attribute access of the form
/// `{ ".": { "left": { "Var": V }, "attr": "tenant_id" } }`, return `V`.
fn tenant_id_attr_access_var(node: &serde_json::Value) -> Option<&str> {
    let access = node.get(".").and_then(serde_json::Value::as_object)?;
    if access.get("attr").and_then(serde_json::Value::as_str)? != "tenant_id" {
        return None;
    }
    access
        .get("left")?
        .get("Var")
        .and_then(serde_json::Value::as_str)
}

/// Recursively find the first EST string-literal value (`{ "Value": "<s>" }`)
/// that equals a KNOWN tenant id other than `owning_tenant`. Returns the
/// foreign tenant id when found.
fn first_foreign_tenant_literal<'a>(
    node: &'a serde_json::Value,
    owning_tenant: &str,
    known_tenants: &HashSet<&str>,
) -> Option<&'a str> {
    if let Some(serde_json::Value::String(s)) = node.get("Value")
        && s.as_str() != owning_tenant
        && known_tenants.contains(s.as_str())
    {
        return Some(s.as_str());
    }
    match node {
        serde_json::Value::Object(map) => map
            .values()
            .find_map(|v| first_foreign_tenant_literal(v, owning_tenant, known_tenants)),
        serde_json::Value::Array(items) => items
            .iter()
            .find_map(|v| first_foreign_tenant_literal(v, owning_tenant, known_tenants)),
        _ => None,
    }
}
