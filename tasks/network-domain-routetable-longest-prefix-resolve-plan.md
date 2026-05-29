# Plan: network-domain-routetable-longest-prefix-resolve

## Objective

Add `RouteTable::resolve_next_hop(addr: IpAddr) -> Result<Option<&Route>, CloudNetworkError>` implementing longest-prefix-match (LPM) over the route table's `Vec<Route>`. Reuse existing `Ipv4Cidr::contains_ip` and `Ipv6Cidr::contains_ip`. Pure deterministic kernel; no I/O; no new types beyond an inline prefix-length extractor helper.

## Requirements Analysis

### Core algorithm

1. For each route in the table, check whether `addr` is contained in the route's `RouteDestination` CIDR.
2. Among all matching routes, select the one with the **longest** (most specific) prefix length.
3. **Family isolation**: IPv4 addresses only match `RouteDestination::Ipv4` routes; IPv6 addresses only match `RouteDestination::Ipv6` routes. Cross-family matches return `Ok(None)` naturally.
4. **Tie-breaking**: The spec requires determinism when two routes share the same prefix length. Strategy: stable iteration order of `routes` Vec (insertion order) is deterministic for a given `RouteTable`; pick the first among equal-prefix-length matches.
5. **No-match**: return `Ok(None)`.
6. **Error propagation**: `contains_ip` can return `Err(CloudNetworkError::InvalidCidrPrefix)` — propagate via `?`.

### Edge cases

- `0.0.0.0/0` (default route) matches all IPv4; `::/0` matches all IPv6.
- `/32` host route and `/128` IPv6 host route are the most specific.
- Multiple routes with identical prefix length but different network prefixes — only one actually contains `addr`, so ties in prefix length among matching routes need deterministic ordering (first in Vec wins).
- Empty route table → `Ok(None)`.
- Address family mismatch (IPv6 addr vs IPv4 route) → `contains_ip` called on wrong family must be avoided; handle via pattern match on `RouteDestination`.

### Prefix-length extraction helper

A small private function `route_prefix_len(route: &Route) -> Result<u32, CloudNetworkError>` that parses the CIDR value and returns the prefix length. No new public type.

## Subtasks (ordered)

1. [x] Write plan (this file)
2. [ ] Write spec (`docs/specs/task-network-domain-routetable-longest-prefix-resolve.md`)
3. [ ] Write RED tests in `crates/oya-cloud-network-domain/src/lib.rs` `#[cfg(test)]` block — confirm `cargo check` reveals missing method
4. [ ] Implement `RouteTable::resolve_next_hop` + helper `route_prefix_len` — GREEN
5. [ ] Run `cargo nextest run -p oya-cloud-network-domain` — confirm all pass
6. [ ] Self-review (correctness / security / performance / cloud-native)
7. [ ] Simplify: guard clauses, naming, dead code

## Acceptance criteria

- (a) `/32` beats `/24` beats `0.0.0.0/0` for a covered IPv4 addr
- (b) no-match returns `Ok(None)`
- (c) IPv6 dest resolves only against IPv6 routes (family isolation)
- (d) `Local` vs gateway next-hop returned intact
- (e) determinism: same table + addr → same Route pointer every call
