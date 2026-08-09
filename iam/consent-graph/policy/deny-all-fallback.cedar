// Deny-by-default fallback policy
// Used when:
//   1. consent-graph cannot resolve an active agreement for (grantor, grantee, entity, action).
//   2. enforcement-app cache is poisoned and re-fetch fails.
//   3. emergency disable: agreement-wide deny override applied by audit-officer.
// Authority: ADR-0214 §2.3 fail-closed.

// Cedar has implicit deny-by-default when no permit applies, but this explicit forbid
// signals intent + supports audit on the deny-class for forensics.

forbid (
  principal,
  action in [
    Action::"project.subscribe",
    Action::"project.read",
    Action::"attested.query",
    Action::"aggregate.read"
  ],
  resource is Resource
)
when {
  true  // unconditional; safest default
}
advice {
  deny_class: "NoAgreement",
  message: "deny-all fallback active for this (grantor, grantee, entity, action) tuple"
};
