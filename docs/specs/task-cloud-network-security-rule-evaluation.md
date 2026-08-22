# Spec: cloud-network-security-rule-evaluation

**Vertical**: cloud  
**Crate**: `cloud-network-domain`  
**Task slug**: `cloud-network-security-rule-evaluation`  
**ADR authority**: ADR-0509 (single-crate-per-service, mod-based subsystems)  
**Layout authority**: ADR-0131 (per-microservice flat layout)

---

## Objective

Extend `cloud-network-domain` with deterministic, adapter-free security-group rule
evaluation and conflict/shadow detection. The implementation is a pure in-crate domain
extension of `crates/cloud-network-domain/src/lib.rs` — no new workspace member, no
OVN/Envoy adapter work, no root `Cargo.toml` edit.

---

## Vertical context

The `cloud.network.*` surface owns VPC, subnet, load-balancer, DNS-zone, CDN, interconnect,
DDoS, and mesh invariants. Security groups (`SecurityGroup` + `SecurityRule`) are already
first-class primitives in the domain kernel. This slice adds the evaluation engine — the
domain logic that answers "does this flow pass?" and "which rules are redundant?" — without
touching any adapter or infrastructure layer.

---

## Existing primitives (lib.rs)

| Type | Role |
|---|---|
| `Ipv4Cidr` / `Ipv6Cidr` | CIDR value objects with `new()` validation |
| `RouteDestination` | IPv4 or IPv6 CIDR discriminated union |
| `RuleDirection` | `Ingress` / `Egress` |
| `IpProtocol` | `Tcp` / `Udp` / `Icmp` / `Any` |
| `SecurityRule` | `direction`, `protocol`, `port_range`, `cidr`, `description` |
| `SecurityGroup` | `id` + `rules: Vec<SecurityRule>` |
| `CloudNetworkError` | existing error enum — new variant added here |
| `parse_ipv4_cidr` / `parse_ipv6_cidr` | private helpers returning `(addr_bits, prefix_len)` |
| `ipv4_contains` / `ipv6_contains` | private containment helpers (parent, child) |
| `ipv4_overlaps` / `ipv6_overlaps` | private overlap helpers |

---

## New domain surface

### ST1 — CIDR containment + overlap on value objects

New `CloudNetworkError` variant:
```rust
InvalidCidrPrefix,
```

New public methods (pure, no I/O, no alloc beyond what Rust requires):

```rust
impl Ipv4Cidr {
    /// True if `other` is fully contained within `self`.
    pub fn contains_cidr(&self, other: &Self) -> Result<bool, CloudNetworkError>
    /// True if `self` and `other` share at least one address.
    pub fn overlaps_cidr(&self, other: &Self) -> Result<bool, CloudNetworkError>
    /// True if `addr` falls within the `self` prefix.
    pub fn contains_ip(&self, addr: Ipv4Addr) -> Result<bool, CloudNetworkError>
}

impl Ipv6Cidr {
    pub fn contains_cidr(&self, other: &Self) -> Result<bool, CloudNetworkError>
    pub fn overlaps_cidr(&self, other: &Self) -> Result<bool, CloudNetworkError>
    pub fn contains_ip(&self, addr: Ipv6Addr) -> Result<bool, CloudNetworkError>
}
```

Implementation delegates to the existing private `parse_ipv4_cidr` / `parse_ipv6_cidr`
helpers and the same bitmask arithmetic used in `ipv4_contains` / `ipv4_overlaps`.

---

### ST2 — `SecurityGroup::evaluate(flow)` → `Decision`

```rust
/// The typed description of a network flow to be evaluated.
pub struct FlowMatch {
    pub direction: RuleDirection,
    pub protocol:  IpProtocol,
    /// None for protocols that carry no port (ICMP, raw IP).
    pub port:      Option<u16>,
    /// Peer address expressed as a host CIDR (/32 for IPv4, /128 for IPv6)
    /// or a CIDR block when evaluating aggregate flows.
    pub peer_cidr: RouteDestination,
}

/// Outcome of evaluating a flow against a security group.
pub enum Decision<'a> {
    Allow { matched_rule: &'a SecurityRule },
    Deny  { matched_rule: Option<&'a SecurityRule> },
}

impl SecurityGroup {
    pub fn evaluate(&self, flow: &FlowMatch) -> Result<Decision<'_>, CloudNetworkError>
}
```

**Matching algorithm** (first-match-wins, matching only on the explicit rule fields):

1. Filter rules by `direction == flow.direction`.
2. Protocol match: `rule.protocol == IpProtocol::Any` OR `rule.protocol == flow.protocol`.
3. Port match: if `rule.port_range` is `Some((start, end))` AND `flow.port` is `Some(p)`,
   require `start <= p <= end`; if rule has no port_range, any port matches; if rule has a
   port_range but flow has no port (ICMP/Any), the rule does not match on port.
4. CIDR match: `rule.cidr.contains_cidr(flow.peer_cidr)` — i.e., the rule's CIDR must
   contain the flow's peer CIDR (ST1).
5. First passing rule → `Allow { matched_rule }` (all rules currently permit; deny-by-rule
   extension is deferred to a future slice).
6. No match → `Deny { matched_rule: None }`.

Note: the existing `SecurityRule` carries only allow-semantics (no `action` field). The
default-deny posture is expressed via the no-match path. A future slice may add
`SecurityRuleAction::Deny` but that is explicitly out of scope here.

---

### ST3 — `SecurityGroup::detect_shadowed_rules()`

```rust
impl SecurityGroup {
    /// Returns pairs `(shadowing, shadowed)` where `shadowing` fully subsumes `shadowed`
    /// in evaluation order: any flow that matches `shadowed` would also match `shadowing`
    /// before reaching it, making `shadowed` unreachable.
    pub fn detect_shadowed_rules(
        &self,
    ) -> Result<Vec<(&SecurityRule, &SecurityRule)>, CloudNetworkError>
}
```

**Shadow predicate** (rule A shadows rule B, A appears before B):
- `A.direction == B.direction`
- `A.protocol == B.protocol` OR `A.protocol == IpProtocol::Any`
- Port superset: `A.port_range` is `None` (matches all ports), OR both have ranges and
  `A.start <= B.start && A.end >= B.end`
- CIDR superset: `A.cidr.contains_cidr(B.cidr)` — A's CIDR fully contains B's CIDR (ST1)

Returns all such pairs in discovery order (outer index i < j, checking all pairs O(n²)).
Returns an empty vec if no shadowed rules exist. Never mutates the group.

---

## Mod layout (flat-clean-arch per ADR-0509)

All code is added inline to `src/lib.rs` — no sub-modules. The file already owns all
domain types; this slice adds methods to `Ipv4Cidr`, `Ipv6Cidr`, `SecurityGroup`, and
two new types `FlowMatch` and `Decision`. Tests live in the existing `#[cfg(test)] mod tests`
block.

---

## Error surface

| Variant (new) | Trigger |
|---|---|
| `CloudNetworkError::InvalidCidrPrefix` | Callers pass a malformed CIDR string through the new public containment/overlap API |

All existing variants are untouched.

---

## OpenAPI / proto3 / AsyncAPI

This slice is a pure domain kernel extension — no HTTP, gRPC, or event surface is added.
The evaluation engine is consumed by adapter layers (REST / gRPC) in a future slice.
No OpenAPI 3.2.0, proto3, or AsyncAPI 3.1.0 contract is introduced here.

---

## Testing strategy

All tests are inline `#[cfg(test)]` inside `lib.rs`, following the established pattern.

| Test group | What is verified |
|---|---|
| ST1 CIDR containment | contained, disjoint, overlapping, equal pairs (IPv4 + IPv6); invalid-prefix rejection |
| ST2 evaluate — allow | ingress allow, egress allow |
| ST2 evaluate — deny | no-match deny, port-mismatch deny, protocol-mismatch deny |
| ST2 evaluate — precedence | first-rule wins when multiple rules match |
| ST3 shadow detection | shadowed pair found; redundant (identical) pair found; non-conflicting → empty vec |

Clippy exemption `#![cfg_attr(test, allow(clippy::unwrap_used, ...))]` is already present.

---

## Boundaries

| In scope | Out of scope |
|---|---|
| `Ipv4Cidr` / `Ipv6Cidr` containment + overlap methods | OVN/Envoy adapter implementation |
| `FlowMatch` + `Decision` domain types | Explicit deny-action `SecurityRule` variant |
| `SecurityGroup::evaluate` | REST / gRPC handler layer |
| `SecurityGroup::detect_shadowed_rules` | Cross-group policy evaluation |
| `CloudNetworkError::InvalidCidrPrefix` | Root `Cargo.toml` edits |
| Lane-namespaced docs | Any other crate |
| OpenSLO files untouched | New workspace member |

---

## Acceptance gate

```bash
cargo check -p cloud-network-domain --all-targets
cargo nextest run -p cloud-network-domain
```

Both must exit 0 with zero errors and all tests green.
