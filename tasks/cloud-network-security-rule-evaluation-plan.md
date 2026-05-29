# Plan: cloud-network-security-rule-evaluation

**Crate**: `oya-cloud-network-domain`  
**Lane**: cloud  
**Branch**: `feat/task-cloud-network-security-rule-evaluation-2026-05-28`

## Objective

Extend the cloud-network domain kernel with deterministic, adapter-free security-group
rule evaluation and conflict/shadow detection. All logic lives inside
`crates/oya-cloud-network-domain/src/lib.rs` as methods on existing types — no new crate,
no root `Cargo.toml` edit, no OVN/Envoy adapter work.

---

## Subtasks

### ST1 — CIDR containment + overlap helpers (public methods on `Ipv4Cidr` / `Ipv6Cidr`)

**What**: Add `contains(ip_or_cidr)` and `overlaps(other)` public methods to both `Ipv4Cidr`
and `Ipv6Cidr`, reusing the existing private `parse_ipv4_cidr` / `parse_ipv6_cidr` helpers.
Add a new `CloudNetworkError::InvalidCidrPrefix` variant for malformed inputs surfaced through
the public API.

**New surface**:
```
impl Ipv4Cidr {
    pub fn contains_cidr(&self, other: &Self) -> Result<bool, CloudNetworkError>
    pub fn overlaps_cidr(&self, other: &Self) -> Result<bool, CloudNetworkError>
    pub fn contains_ip(&self, addr: Ipv4Addr) -> Result<bool, CloudNetworkError>
}

impl Ipv6Cidr {
    pub fn contains_cidr(&self, other: &Self) -> Result<bool, CloudNetworkError>
    pub fn overlaps_cidr(&self, other: &Self) -> Result<bool, CloudNetworkError>
    pub fn contains_ip(&self, addr: Ipv6Addr) -> Result<bool, CloudNetworkError>
}
```

**Acceptance**:
- Unit tests cover: contained pair, disjoint pair, overlapping pair, equal pair — for both
  IPv4 and IPv6.
- Invalid-prefix rejection test (returns `CloudNetworkError::InvalidCidrPrefix`).
- `cargo check -p oya-cloud-network-domain --all-targets` passes.
- `cargo nextest run -p oya-cloud-network-domain` green.

---

### ST2 — `SecurityGroup::evaluate(flow)` → `Decision`

**What**: Add types `FlowMatch` (direction, protocol, port, peer CIDR) and `Decision`
(Allow/Deny with matched rule reference), and implement `SecurityGroup::evaluate` using ST1
CIDR helpers and existing `SecurityRule` fields. First matching rule wins; no-match → Deny.

**New surface**:
```rust
pub enum Decision<'a> {
    Allow { matched_rule: &'a SecurityRule },
    Deny  { matched_rule: Option<&'a SecurityRule> },
}

pub struct FlowMatch {
    pub direction: RuleDirection,
    pub protocol:  IpProtocol,
    pub port:      Option<u16>,
    pub peer_cidr: RouteDestination,
}

impl SecurityGroup {
    pub fn evaluate(&self, flow: &FlowMatch) -> Result<Decision<'_>, CloudNetworkError>
}
```

**Matching semantics**:
- Direction must match exactly.
- Protocol: rule `Any` matches any flow protocol; exact match otherwise.
- Port: if rule `port_range` is `None` (protocol `Any` or ICMP), no port check; otherwise
  the flow's `port` must fall within `[start, end]`.
- CIDR: rule's `cidr` must contain the flow's `peer_cidr` (uses ST1 `contains_cidr` / `contains_ip`).
- First matching rule in slice order wins.
- No match → `Deny { matched_rule: None }`.

**Acceptance**:
- Unit tests: allow on matching ingress rule; allow on matching egress rule; deny on no-match;
  correct first-match precedence when multiple rules match; port mismatch → deny; protocol
  mismatch → deny.
- `cargo nextest run -p oya-cloud-network-domain` green.

---

### ST3 — `SecurityGroup::detect_shadowed_rules()` + lane docs

**What**: Implement `SecurityGroup::detect_shadowed_rules()` returning pairs of
`(&SecurityRule, &SecurityRule)` where the first rule fully subsumes the second (same
direction, compatible protocol, port range superset, CIDR superset). Document the slice
in lane-namespaced docs.

**New surface**:
```rust
impl SecurityGroup {
    /// Returns pairs (shadowing, shadowed) where `shadowing` fully subsumes `shadowed`.
    pub fn detect_shadowed_rules(&self) -> Result<Vec<(&SecurityRule, &SecurityRule)>, CloudNetworkError>
}
```

**Shadow predicate** (all conditions must hold for rule A to shadow rule B):
- Same `direction`.
- A's `protocol` == B's `protocol`, OR A's `protocol` == `Any`.
- A's `port_range` is a superset of B's (or A has no port restriction).
- A's `cidr` contains B's `cidr` (uses ST1 `contains_cidr`).
- A appears before B in the slice (earlier evaluation order subsumes later).

**Lane docs** (created, not edited):
- `docs/specs/task-cloud-network-security-rule-evaluation.md`
- `tasks/cloud-network-security-rule-evaluation-plan.md` (this file)

**Constraints**:
- `crates/oya-cloud-network-domain/slos/*.openslo.yaml` untouched.
- Root `Cargo.toml` untouched.
- No other crate touched.

**Acceptance**:
- Unit tests: shadowed pair detected; redundant (identical) pair detected; non-conflicting
  set returns empty vec.
- Both lane-namespaced docs exist on disk.
- `cargo check -p oya-cloud-network-domain --all-targets` passes.
- `cargo nextest run -p oya-cloud-network-domain` green.

---

## Constraints (global)

| Constraint | Status |
|---|---|
| Single crate (`oya-cloud-network-domain`) | enforced |
| Root `Cargo.toml` not edited | enforced |
| No new workspace member | enforced |
| No OVN/Envoy adapter | enforced |
| OpenSLO files untouched | enforced |
| Pure domain logic, adapter-free | enforced |

---

## Test strategy

All tests live inside `lib.rs` under `#[cfg(test)]` following the existing inline-test
pattern. No new integration test file is needed for ST1–ST3.

Pattern evidence: existing test helpers (`security_group_create()`, `vpc_create()`, etc.)
live in `mod tests` at the bottom of `lib.rs` — new tests follow the same shape.
