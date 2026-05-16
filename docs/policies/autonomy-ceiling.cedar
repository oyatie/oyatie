// M02-P05-IP-002 autonomy ceiling — T4 actuation disabled by default
forbid (principal, action == Action::"actuate-t4", resource);
permit (principal, action in [Action::"read-t1", Action::"suggest-t2"], resource);
