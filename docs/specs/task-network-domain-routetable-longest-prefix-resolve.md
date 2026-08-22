# Spec: RouteTable Longest-Prefix-Match Resolver

**Slug**: `network-domain-routetable-longest-prefix-resolve`
**Crate**: `cloud-network-domain`
**Lane**: infra | Priority: high | Effort: M

---

## Objective

Add a pure, deterministic longest-prefix-match (LPM) resolver to `RouteTable`. Given a single `IpAddr`, select the most-specific `Route` whose destination CIDR contains that address.

## Method signature

```rust
impl RouteTable {
    pub fn resolve_next_hop(
        &self,
        addr: IpAddr,
    ) -> Result<Option<&Route>, CloudNetworkError>
}
```

## Contracts

| Input | Behaviour |
|---|---|
| `IpAddr::V4(a)` | Match only `RouteDestination::Ipv4` routes |
| `IpAddr::V6(a)` | Match only `RouteDestination::Ipv6` routes |
| Address in multiple routes | Return the route with the longest prefix length |
| No matching route | `Ok(None)` |
| Equal-prefix-length tie | First (lowest Vec index) among tied matches wins (deterministic) |
| Invalid CIDR in stored route | Propagate `Err(CloudNetworkError::InvalidCidrPrefix)` |

## Mod layout (flat-clean-arch)

All logic lives in `src/lib.rs` — the crate is a flat single-file kernel following ADR-0509. No new modules.

Private helper:
```rust
fn route_prefix_len(route: &Route) -> Result<u32, CloudNetworkError>
```
Parses the CIDR string and returns the numeric prefix length. Used only by `resolve_next_hop`.

## Testing strategy

Inline `#[cfg(test)]` module in `src/lib.rs`. All tests are hermetic unit tests (no I/O, no network, no clock).

### Test cases required

| ID | Description |
|---|---|
| lpm_v4_most_specific_wins | /32 beats /24 beats 0.0.0.0/0 for a contained IPv4 addr |
| lpm_v4_no_match | Address not covered by any route → Ok(None) |
| lpm_v6_family_isolation | IPv6 addr only matches IPv6 routes; IPv4-only table → Ok(None) |
| lpm_next_hop_intact | Local and gateway next_hop kinds returned without mutation |
| lpm_determinism | Two calls on same table+addr return the same route (ptr equality / field equality) |
| lpm_default_route | 0.0.0.0/0 matches everything when no more-specific route exists |
| lpm_empty_table | Empty route table → Ok(None) |

## Observability / SLO

This is a pure in-process computation kernel (no network I/O, no async). No OTel instrumentation required at this layer. Callers that invoke this from the control plane path are responsible for span attribution.

## Crate boundary

- Modifies: `crates/cloud-network-domain/src/lib.rs` only
- New dependencies: none
- Existing helpers used: `Ipv4Cidr::contains_ip`, `Ipv6Cidr::contains_ip`, `parse_ipv4_cidr`, `parse_ipv6_cidr`
