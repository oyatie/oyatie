# Plan: cloud-resource-domain-transition-graph-introspection

## Objective
Extend `ResourceState` in `oya-cloud-resource-domain` with a pure public introspection
and classification surface without altering existing transition semantics.

## Steps

1. Add `ResourceState::as_str() -> &'static str` const fn  
2. Add `ResourceState::parse(s: &str) -> Option<ResourceState>` fn (fail-closed, no panic)  
3. Add `ResourceState::is_active() -> bool` const fn (Running = active)  
4. Add `ResourceState::is_quiescent() -> bool` const fn (Stopped = quiescent)  
5. Add `ResourceState::allowed_next() -> &'static [ResourceState]` const fn — returns the
   set of legal successor states derived from `state_transition_allowed`; Terminated returns
   empty slice  
6. Add `ResourceState::can_transition_to(self, next: ResourceState) -> bool` public wrapper
   delegating to the existing private `state_transition_allowed`  
7. Write table-driven test: all 5x5 (Pending/Running/Stopped/Terminated/Error) ordered pairs
   verified against `allowed_next()` membership  
8. Write round-trip test: `parse(as_str(x)) == Some(x)` for every variant, `parse("unknown") == None`  
9. `cargo check -p oya-cloud-resource-domain --all-targets` green  
10. `cargo nextest run -p oya-cloud-resource-domain` green  

## Constraints
- No new dependencies  
- No I/O or async  
- Purely additive — existing `state_transition_allowed` and `transition_state` untouched  
- Respects `Classified`/`DataClass` posture and `#![cfg_attr(test,...)]` exemption  
