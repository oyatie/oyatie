// foundry-supervisor.cedar — autonomy policy for foundry.supervisor capabilities
// Extends docs/policies/autonomy-ceiling.cedar (M02-P05-IP-002 seed; DO NOT modify that file here).
// Existing seed (verbatim):
//   // M02-P05-IP-002 autonomy ceiling — T4 actuation disabled by default
//   forbid (principal, action == Action::"actuate-t4", resource);
//   permit (principal, action in [Action::"read-t1", Action::"suggest-t2"], resource);
//
// This file adds per-capability policies for foundry.supervisor.{inject_message,idle_tick,
// restart_session,dead_letter,read} loaded alongside autonomy-ceiling.cedar by
// autonomy-ceiling-app::enforce_for_tenant.

// inject_message: permitted when resource autonomy_tier <= account ceiling
permit (
  principal in Capability::"foundry.supervisor.inject_message",
  action == Action::"invoke",
  resource
) when {
  resource.autonomy_tier <= principal.autonomy_tier_ceiling
};

// idle_tick: always permitted (read-only heartbeat; no payload mutation)
permit (
  principal in Capability::"foundry.supervisor.idle_tick",
  action == Action::"invoke",
  resource
);

// read: read-only surface; permitted at T1 threshold
permit (
  principal in Capability::"foundry.supervisor.read",
  action == Action::"read",
  resource
);

// restart_session: T4 actuation — forbidden (inherits from autonomy-ceiling.cedar T4 blanket
// forbid, reinforced here for explicitness)
forbid (
  principal in Capability::"foundry.supervisor.restart_session",
  action == Action::"actuate-t4",
  resource
);

// dead_letter: T4 actuation — forbidden (dead-letter routing mutates global state irreversibly;
// requires human approval outside Cedar; explicit forbid below)
forbid (
  principal in Capability::"foundry.supervisor.dead_letter",
  action == Action::"actuate-t4",
  resource
);
