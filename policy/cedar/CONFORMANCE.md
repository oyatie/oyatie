---
doc_class: Specification
shape: Specification
length_cap: 600
microservice: policy
related_adrs:
  - ADR-0701
  - ADR-0702
  - ADR-0243
  - ADR-0280
inbound_citations:
  - policy/README.md
---

# Cedar conformance — measured, not asserted

Every claim here was produced by running the policies in `policy/policy/` against the real
`cedar-policy` engine at the version the workspace locks (**4.12.0**; root `Cargo.toml` requires
`"4.11"`, `Cargo.lock` resolves 4.12.0).

The harness runs **out of tree** because this capability may not add a workspace member yet
(`PROMOTION.md` §2). Rather than ship an unrunnable claim, the harness source is reproduced in full
below; at promotion it becomes the test body of `policy/adapters/policy-cedar-conformance`.

**These fragments are version 2.0.0.** Version 1.0.0 was broken by an adversarial review that found
six classes of defect — every safety forbid failing open on an absent attribute, tenant binding by
self-asserted string, a caller-declared staleness bound with no ceiling, global-scope signing
ungated, ReBAC writes bypassing change control, and credential freshness required only on the audit
read. All six are closed and each is a named regression case below.

## 1. Instrument check first

A passing validator proves nothing until it is shown to work on something known good:

```
$ cedarval iam/adapters/pdp-cedar/cedar/platform.cedarschema \
           iam/adapters/pdp-cedar/cedar/platform-policies.cedar
SCHEMA-OK   iam/adapters/pdp-cedar/cedar/platform.cedarschema
VALIDATE-OK iam/adapters/pdp-cedar/cedar/platform-policies.cedar (4 policies)
exit 0
```

## 2. Strict validation

```
SCHEMA-OK policy/policy/schema.cedarschema
VALIDATE-OK policy/policy/auditor-scope.cedar (1 policies)
VALIDATE-OK policy/policy/authoring-grants.cedar (6 policies)
VALIDATE-OK policy/policy/change-control.cedar (7 policies)
VALIDATE-OK policy/policy/credential-freshness.cedar (1 policies)
VALIDATE-OK policy/policy/cross-tenant-isolation.cedar (1 policies)
VALIDATE-OK policy/policy/runtime-attestation.cedar (2 policies)
VALIDATE-OK policy/policy/runtime-grants.cedar (3 policies)
VALIDATE-OK policy/policy/static-stability.cedar (5 policies)
exit 0
```

`policy/cedar/policies.cedar` validates clean at **26 policies**. It is the concatenation of the
fragments **plus** a header and one separator comment per fragment — *not* byte-identical to
`cat policy/policy/*.cedar`, which an earlier revision of this page wrongly claimed. Comment-stripped
they are byte-identical, and the whole suite runs against the bundle, so decisional equivalence is
tested rather than asserted.

## 3. Behavioural suite

```
  ok   P2 admin authors own-tenant policy  [red if: drop P2]
  ok   P2a activation after soak  [red if: drop P2a]
  ok   P3 step-up-C engineer authors global  [red if: tighten F8 past step-up C]
  ok   P3a step-up-C engineer activates global after soak  [red if: drop P3a]
  ok   P4 signer signs another's policy  [red if: drop P4]
  ok   P5 admin writes tuple at step-up C  [red if: drop P5]
  ok   P6 pep evaluates fresh in-cell snapshot  [red if: drop P6]
  ok   P7 pep reads tuple from fresh snapshot  [red if: drop P7]
  ok   P8 distributor propagates verified fresh  [red if: drop P8]
  ok   P1 auditor reads log on fresh token  [red if: drop P1]
  ok   stale BUT authoritative tuple read allowed  [red if: make F2 forbid on staleness alone]
  ok   default-deny: unlisted role/action  [red if: add a catch-all permit]
  ok   S1a activation with NO soak attribute  [red if: stop mirroring the soak bound into P2a/P3a]
  ok   S1b evaluate with NO max_staleness attribute  [red if: stop mirroring the staleness bound into P6]
  ok   S1c author with NO token_age attribute  [red if: stop mirroring token_age into the permits]
  ok   S2a caller declares i64::MAX tolerance on ancient snapshot  [red if: drop F3b, the absolute ceiling]
  ok   S2b tuple read from ancient non-authoritative snapshot  [red if: drop F2]
  ok   S2c distribute ancient non-authoritative snapshot  [red if: drop F3b's DistributeSnapshot arm]
  ok   S2d evaluate unverified snapshot  [red if: drop F4]
  ok   S2e tuple read from unverified snapshot  [red if: drop F4b]
  ok   S2f pep evaluates ANOTHER CELL's snapshot  [red if: drop `principal in resource.cell` from P6]
  ok   S3 no-step-up signer signs GLOBAL policy  [red if: remove SignPolicy from F8's action list]
  ok   S4a principal acts on another tenant's resource  [red if: revert F1 to a tenant_id string compare]
  ok   S4b principal with NO role membership  [red if: revert permits to a principal.role string compare]
  ok   S4c signer sharing the author's spiffe_id  [red if: drop F5b]
  ok   S4d signer signs own authorship  [red if: drop F5]
  ok   S4e junk spiffe_id fails attestation  [red if: weaken the spiffe_id `like` pattern]
  ok   S5 tuple write with no step-up  [red if: drop F12]
  ok   S6a author with a 5000s-old token  [red if: drop F10]
  ok   S6b distribute with a 5000s-old token  [red if: drop F10]
  ok   F6 unsigned policy cannot publish  [red if: drop F6]
  ok   F7 activation before soak  [red if: lower the 60s soak bound]
  ok   F8 tenant admin cannot touch global  [red if: drop F8]
  ok   F8 engineer without step-up on global  [red if: drop F8's step_up_class clause]
  ok   F11 unrecognised scope value denied  [red if: drop F11]
  ok   cross-tenant tuple read denied  [red if: drop F1]

36 passed, 0 failed
```

## 4. Backstop run — every bound is enforced twice

Each bound is written both inside the permit that grants the action and again in a forbid. The
permits alone therefore deny these cases, and the **forbids are never reached** — a forbid nothing
reaches is a forbid nothing tests. This run injects a fixture of deliberately *bound-blind* permits,
each omitting a bound the shipped permits carry, which is precisely the authoring mistake the forbids
exist to survive:

```cedar
// tenant-blind AND token-blind                       // soak-blind activate
permit (principal, action == OyaPolicy::Action::"AuthorPolicy", resource)
when { principal in OyaPolicy::Role::"tenant-policy-admin" };
// ... likewise freshness/signature-blind Evaluate, ReadRebacTuple, DistributeSnapshot,
//     and a step-up-blind WriteRebacTuple
```

```
  ok   BACKSTOP F1: tenant-blind permit, cross-tenant author  [red if: drop F1]
  ok   BACKSTOP F10: token-blind permit, 5000s-old token  [red if: drop F10]
  ok   BACKSTOP F3b: freshness-blind permit, ancient snapshot  [red if: drop F3b]
  ok   BACKSTOP F4: signature-blind permit, unverified snapshot  [red if: drop F4]
  ok   BACKSTOP F2: freshness-blind permit, ancient tuple read  [red if: drop F2]
  ok   BACKSTOP F4b: signature-blind permit, unverified tuple read  [red if: drop F4b]
  ok   BACKSTOP F3b: freshness-blind permit, ancient distribute  [red if: drop F3b's DistributeSnapshot arm]
  ok   BACKSTOP F7: soak-blind permit, 30s soak  [red if: drop F7]
  ok   BACKSTOP F12: step-up-blind permit, no step-up  [red if: drop F12]
45 passed, 0 failed
```

All 45 pass **with the bound-blind permits present**: every safety bound in this bundle survives a
permit that forgets it.

## 5. Mutation coverage — per POLICY, not per fragment

A suite that passes tells you nothing until you show it can fail. Each of the 26 policies carries an
`@id(...)` annotation, and the harness drops exactly one by id (`DROP_POLICY_ID=F7`) and re-runs.

**This measure replaces a per-fragment matrix that was hiding real gaps.** An audit of v1.0.0 showed
three case names asserting they tested a rule that no mutation could reach: `F8` and `F9` were
covered by *nothing*, because the permits independently denied those requests. Fragment granularity
could not see it — a fragment "covered" by one of its six policies reads as covered. Per-policy
granularity is the honest instrument, and applying it to v2.0.0 immediately found **eight** more
uncovered policies.

| policy | plain run: cases red when dropped | backstop run: cases red when dropped |
|---|---|---|
| `F1` | 0 | 2 |
| `F10` | 0 | 4 |
| `F11` | 0 | 1 |
| `F12` | 0 | 2 |
| `F13` | 0 | 1 |
| `F14` | 0 | 1 |
| `F2` | 0 | 2 |
| `F3` | 0 | 1 |
| `F3b` | 0 | 4 |
| `F4` | 0 | 2 |
| `F4b` | 0 | 2 |
| `F5` | 1 | 1 |
| `F5b` | 1 | 1 |
| `F6` | 1 | 1 |
| `F7` | 0 | 3 |
| `F8` | 1 | 2 |
| `P1` | 1 | 1 |
| `P2` | 1 | 0 |
| `P2a` | 1 | 0 |
| `P3` | 1 | 1 |
| `P3a` | 1 | 1 |
| `P4` | 1 | 1 |
| `P5` | 1 | 0 |
| `P6` | 1 | 0 |
| `P7` | 2 | 0 |
| `P8` | 1 | 0 |

Every policy is red in at least one run. Zero are uncovered in both. The split is the design showing
through: the plain run exercises the **permits**, the backstop run the **forbids**; a `0` in one
column is a policy that column's fixture deliberately shadows.

Closing the last four gaps changed the policy set, not just the tests:

- **F5** was uncovered because **F5b subsumed it entirely** — a principal always shares its own
  `spiffe_id`, so the shared-identity rule fired on every case the self-authorship rule would have.
  F5b is now scoped `resource.author != principal`, making the two disjoint and both reachable. Two
  rules no test can tell apart are one rule with a spare.
- **F7** (soak) and **F12** (tuple step-up) needed bound-blind permits added to the fixture.
- **P3a** (step-up-C engineer activating global after soak) had no Allow case at all.

## 6. Harness source

Reproduce with `cedar-policy = "4.12"` and `serde_json = "1"`. Note `Context::from_json_value(ctx,
None)` and `Request::new(..., None)`: the schema is deliberately **not** passed, so omitted context
attributes stay omitted. That is the precondition for the `S1` cases, and passing the schema would
paper over the failure mode they exist to catch.

`conform.rs`:

```rust
// Conformance + attack-regression suite for the policy capability Cedar bundle.
// argv: <schema> <policies> [extra-policies]
use cedar_policy::{Authorizer, Context, Decision, Entities, EntityUid, PolicySet, Request, Schema};
use serde_json::{json, Value};
use std::{fs, str::FromStr};

fn uid(s: &str) -> EntityUid { EntityUid::from_str(s).expect("uid") }
fn ent(t: &str, i: &str) -> Value { json!({"type": format!("OyaPolicy::{t}"), "id": i}) }
fn eref(t: &str, i: &str) -> Value { json!({"__entity": ent(t, i)}) }

fn principal(id: &str, tenant: &str, role: &str, cell: &str, step: &str, spiffe: &str) -> Value {
    json!({"uid": ent("Principal", id),
           "attrs": {"step_up_class": step, "spiffe_id": spiffe},
           "parents": [ent("Tenant", tenant), ent("Role", role), ent("Cell", cell)]})
}
fn snapshot(id: &str, tenant: &str, cell: &str, age: i64, auth: bool, sig: bool) -> Value {
    json!({"uid": ent("Snapshot", id),
           "attrs": {"owner_tenant": eref("Tenant", tenant), "cell": eref("Cell", cell),
                     "snapshot_version": "v1", "age_seconds": age,
                     "is_authoritative": auth, "signature_verified": sig},
           "parents": [ent("Tenant", tenant)]})
}
fn pv(id: &str, tenant: &str, scope: &str, signed: bool, author: &str) -> Value {
    json!({"uid": ent("PolicyVersion", id),
           "attrs": {"owner_tenant": eref("Tenant", tenant), "policy_id": "p1", "scope": scope,
                     "signed": signed, "author": eref("Principal", author)},
           "parents": [ent("Tenant", tenant)]})
}
fn tuple(id: &str, tenant: &str, snap: &str) -> Value {
    json!({"uid": ent("RebacTuple", id),
           "attrs": {"owner_tenant": eref("Tenant", tenant), "object_type": "doc",
                     "relation": "viewer", "served_from": eref("Snapshot", snap)},
           "parents": [ent("Tenant", tenant)]})
}

pub fn run(drop_id: Option<&str>, quiet: bool) -> (usize, usize, Vec<String>) {
    let a: Vec<String> = std::env::args().collect();
    let (schema, _) = Schema::from_cedarschema_str(&fs::read_to_string(&a[1]).unwrap()).unwrap();
    let mut src = fs::read_to_string(&a[2]).unwrap();
    let backstop = a.len() > 3;
    if backstop { src.push('\n'); src.push_str(&fs::read_to_string(&a[3]).unwrap()); }
    let parsed: PolicySet = src.parse().unwrap();
    let ps: PolicySet = if let Some(d) = drop_id {
        PolicySet::from_policies(parsed.policies()
            .filter(|pol| pol.annotation("id").map(|a| a != d).unwrap_or(true))
            .cloned()).expect("subset")
    } else { parsed };

    let mut e = vec![
        json!({"uid": ent("Tenant","t1"), "attrs": {"tenant_id":"t1"}, "parents": []}),
        json!({"uid": ent("Tenant","t2"), "attrs": {"tenant_id":"t2"}, "parents": []}),
        json!({"uid": ent("Cell","c1"), "attrs": {"cell_id":"c1"}, "parents": []}),
        json!({"uid": ent("Cell","c99"), "attrs": {"cell_id":"c99"}, "parents": []}),
        json!({"uid": ent("DecisionLog","log1"), "attrs": {"owner_tenant": eref("Tenant","t1")}, "parents": [ent("Tenant","t1")]}),
    ];
    for r in ["tenant-policy-admin","platform-policy-engineer","policy-signer","policy-distributor","auditor","pep-workload"] {
        e.push(json!({"uid": ent("Role", r), "attrs": {}, "parents": []}));
    }
    e.push(principal("admin1","t1","tenant-policy-admin","c1","C","spiffe://oyatie/a1"));
    e.push(principal("admin_nostep","t1","tenant-policy-admin","c1","","spiffe://oyatie/a2"));
    e.push(principal("eng_c","t1","platform-policy-engineer","c1","C","spiffe://oyatie/e1"));
    e.push(principal("eng_none","t1","platform-policy-engineer","c1","","spiffe://oyatie/e2"));
    e.push(principal("signer1","t1","policy-signer","c1","C","spiffe://oyatie/s1"));
    e.push(principal("signer_same_spiffe","t1","policy-signer","c1","C","spiffe://oyatie/a1"));
    e.push(principal("dist1","t1","policy-distributor","c1","C","spiffe://oyatie/d1"));
    e.push(principal("aud1","t1","auditor","c1","","spiffe://oyatie/au1"));
    e.push(principal("pep1","t1","pep-workload","c1","","spiffe://oyatie/p1"));
    e.push(principal("pep_c99","t1","pep-workload","c99","","spiffe://oyatie/p2"));
    e.push(principal("pep_junk","t1","pep-workload","c1","","  "));
    // S4: parent tenant t1, but every check is membership now, so it can never reach t2.
    e.push(principal("spoofer","t1","tenant-policy-admin","c1","C","spiffe://oyatie/sp"));
    // no Role parent at all
    e.push(json!({"uid": ent("Principal","roleless"), "attrs": {"step_up_class":"C","spiffe_id":"spiffe://oyatie/r"},
                  "parents": [ent("Tenant","t1"), ent("Cell","c1")]}));
    e.push(pv("pv_t1","t1","tenant",true,"admin1"));
    e.push(pv("pv_t1_unsigned","t1","tenant",false,"admin1"));
    e.push(pv("pv_t2","t2","tenant",true,"admin1"));
    e.push(pv("pv_global","t1","global",true,"eng_c"));
    e.push(pv("pv_global_unsigned","t1","global",false,"eng_c"));
    e.push(pv("pv_selfauthored","t1","tenant",true,"signer1"));
    e.push(pv("pv_badscope","t1","GLOBAL",true,"admin1"));
    e.push(snapshot("snap_ok","t1","c1",10,true,true));
    e.push(snapshot("snap_old","t1","c1",900,false,true));
    e.push(snapshot("snap_old_auth","t1","c1",900,true,true));
    e.push(snapshot("snap_ancient","t1","c1",2_592_000,false,true));
    e.push(snapshot("snap_unverified","t1","c1",10,true,false));
    e.push(snapshot("snap_c99","t1","c99",10,true,true));
    e.push(tuple("rt_fresh","t1","snap_ok"));
    e.push(tuple("rt_stale","t1","snap_old"));
    e.push(tuple("rt_stale_auth","t1","snap_old_auth"));
    e.push(tuple("rt_unverified","t1","snap_unverified"));
    e.push(tuple("rt_t2","t2","snap_ok"));
    let entities = Entities::from_json_value(Value::Array(e), Some(&schema)).expect("entities");

    let t = |extra: Value| -> Value {
        let mut m = json!({"now":0,"token_age_seconds":60});
        if let (Some(o), Some(x)) = (m.as_object_mut(), extra.as_object()) {
            for (k,v) in x { o.insert(k.clone(), v.clone()); } }
        m
    };
    type C<'a> = (&'a str,&'a str,&'a str,&'a str,Value,Decision,&'a str);
    let mut cases: Vec<C> = vec![
      // ── baseline behaviour ──
      ("P2 admin authors own-tenant policy","admin1","AuthorPolicy","pv_t1",t(json!({})),Decision::Allow,"drop P2"),
      ("P2a activation after soak","admin1","ActivatePolicy","pv_t1",t(json!({"soak_elapsed_seconds":120})),Decision::Allow,"drop P2a"),
      ("P3 step-up-C engineer authors global","eng_c","AuthorPolicy","pv_global",t(json!({})),Decision::Allow,"tighten F8 past step-up C"),
      ("P3a step-up-C engineer activates global after soak","eng_c","ActivatePolicy","pv_global",t(json!({"soak_elapsed_seconds":120})),Decision::Allow,"drop P3a"),
      ("P4 signer signs another's policy","signer1","SignPolicy","pv_t1",t(json!({})),Decision::Allow,"drop P4"),
      ("P5 admin writes tuple at step-up C","admin1","WriteRebacTuple","rt_fresh",t(json!({})),Decision::Allow,"drop P5"),
      ("P6 pep evaluates fresh in-cell snapshot","pep1","EvaluateAgainstSnapshot","snap_ok",t(json!({"max_staleness_seconds":300})),Decision::Allow,"drop P6"),
      ("P7 pep reads tuple from fresh snapshot","pep1","ReadRebacTuple","rt_fresh",t(json!({})),Decision::Allow,"drop P7"),
      ("P8 distributor propagates verified fresh","dist1","DistributeSnapshot","snap_ok",t(json!({})),Decision::Allow,"drop P8"),
      ("P1 auditor reads log on fresh token","aud1","ReadDecisionLog","log1",t(json!({})),Decision::Allow,"drop P1"),
      ("stale BUT authoritative tuple read allowed","pep1","ReadRebacTuple","rt_stale_auth",t(json!({})),Decision::Allow,"make F2 forbid on staleness alone"),
      ("default-deny: unlisted role/action","dist1","AuthorPolicy","pv_t1",t(json!({})),Decision::Deny,"add a catch-all permit"),
      // ── S1: forbids must not fail open on an absent context attribute ──
      ("S1a activation with NO soak attribute","admin1","ActivatePolicy","pv_t1",json!({"now":0,"token_age_seconds":60}),Decision::Deny,"stop mirroring the soak bound into P2a/P3a"),
      ("S1b evaluate with NO max_staleness attribute","pep1","EvaluateAgainstSnapshot","snap_ok",json!({"now":0,"token_age_seconds":60}),Decision::Deny,"stop mirroring the staleness bound into P6"),
      ("S1c author with NO token_age attribute","admin1","AuthorPolicy","pv_t1",json!({"now":0}),Decision::Deny,"stop mirroring token_age into the permits"),
      // ── S2: the static-stability invariant ──
      ("S2a caller declares i64::MAX tolerance on ancient snapshot","pep1","EvaluateAgainstSnapshot","snap_ancient",t(json!({"max_staleness_seconds":9223372036854775807i64})),Decision::Deny,"drop F3b, the absolute ceiling"),
      ("S2b tuple read from ancient non-authoritative snapshot","pep1","ReadRebacTuple","rt_stale",t(json!({})),Decision::Deny,"drop F2"),
      ("S2c distribute ancient non-authoritative snapshot","dist1","DistributeSnapshot","snap_ancient",t(json!({})),Decision::Deny,"drop F3b's DistributeSnapshot arm"),
      ("S2d evaluate unverified snapshot","pep1","EvaluateAgainstSnapshot","snap_unverified",t(json!({"max_staleness_seconds":300})),Decision::Deny,"drop F4"),
      ("S2e tuple read from unverified snapshot","pep1","ReadRebacTuple","rt_unverified",t(json!({})),Decision::Deny,"drop F4b"),
      ("S2f pep evaluates ANOTHER CELL's snapshot","pep_c99","EvaluateAgainstSnapshot","snap_ok",t(json!({"max_staleness_seconds":300})),Decision::Deny,"drop `principal in resource.cell` from P6"),
      // ── S3: global-scope signing ──
      ("S3 no-step-up signer signs GLOBAL policy","signer1","SignPolicy","pv_global_unsigned",t(json!({})),Decision::Deny,"remove SignPolicy from F8's action list"),
      // ── S4: authority by membership, not by string ──
      ("S4a principal acts on another tenant's resource","spoofer","AuthorPolicy","pv_t2",t(json!({})),Decision::Deny,"revert F1 to a tenant_id string compare"),
      ("S4b principal with NO role membership","roleless","AuthorPolicy","pv_t1",t(json!({})),Decision::Deny,"revert permits to a principal.role string compare"),
      ("S4c signer sharing the author's spiffe_id","signer_same_spiffe","SignPolicy","pv_t1",t(json!({})),Decision::Deny,"drop F5b"),
      ("S4d signer signs own authorship","signer1","SignPolicy","pv_selfauthored",t(json!({})),Decision::Deny,"drop F5"),
      ("S4e junk spiffe_id fails attestation","pep_junk","EvaluateAgainstSnapshot","snap_ok",t(json!({"max_staleness_seconds":300})),Decision::Deny,"weaken the spiffe_id `like` pattern"),
      // ── S5: tuple writes are governed ──
      ("S5 tuple write with no step-up","admin_nostep","WriteRebacTuple","rt_fresh",t(json!({})),Decision::Deny,"drop F12"),
      // ── S6: credential freshness on mutating paths ──
      ("S6a author with a 5000s-old token","admin1","AuthorPolicy","pv_t1",t(json!({"token_age_seconds":5000})),Decision::Deny,"drop F10"),
      ("S6b distribute with a 5000s-old token","dist1","DistributeSnapshot","snap_ok",t(json!({"token_age_seconds":5000})),Decision::Deny,"drop F10"),
      // ── misc ──
      ("F6 unsigned policy cannot publish","admin1","PublishPolicy","pv_t1_unsigned",t(json!({})),Decision::Deny,"drop F6"),
      ("F7 activation before soak","admin1","ActivatePolicy","pv_t1",t(json!({"soak_elapsed_seconds":30})),Decision::Deny,"lower the 60s soak bound"),
      ("F8 tenant admin cannot touch global","admin1","AuthorPolicy","pv_global",t(json!({})),Decision::Deny,"drop F8"),
      ("F8 engineer without step-up on global","eng_none","AuthorPolicy","pv_global",t(json!({})),Decision::Deny,"drop F8's step_up_class clause"),
      ("F11 unrecognised scope value denied","admin1","AuthorPolicy","pv_badscope",t(json!({})),Decision::Deny,"drop F11"),
      ("cross-tenant tuple read denied","pep1","ReadRebacTuple","rt_t2",t(json!({})),Decision::Deny,"drop F1"),
    ];
    if backstop {
        // BACKSTOP CASES. Every bound is enforced twice: once inside the permit (so an absent
        // attribute denies) and once in a forbid (so a permit that FORGETS the bound is survivable).
        // The permits alone make the forbids unreachable, so these cases inject deliberately
        // bound-blind permits — the exact authoring mistakes the forbids exist for — and assert the
        // request is still denied. Each is the ONLY case that reaches its forbid.
        cases.extend(vec![
          ("BACKSTOP F1: tenant-blind permit, cross-tenant author","spoofer","AuthorPolicy","pv_t2",t(json!({})),Decision::Deny,"drop F1"),
          ("BACKSTOP F10: token-blind permit, 5000s-old token","admin1","AuthorPolicy","pv_t1",t(json!({"token_age_seconds":5000})),Decision::Deny,"drop F10"),
          ("BACKSTOP F3b: freshness-blind permit, ancient snapshot","pep1","EvaluateAgainstSnapshot","snap_ancient",t(json!({"max_staleness_seconds":9223372036854775807i64})),Decision::Deny,"drop F3b"),
          ("BACKSTOP F4: signature-blind permit, unverified snapshot","pep1","EvaluateAgainstSnapshot","snap_unverified",t(json!({"max_staleness_seconds":300})),Decision::Deny,"drop F4"),
          ("BACKSTOP F2: freshness-blind permit, ancient tuple read","pep1","ReadRebacTuple","rt_stale",t(json!({})),Decision::Deny,"drop F2"),
          ("BACKSTOP F4b: signature-blind permit, unverified tuple read","pep1","ReadRebacTuple","rt_unverified",t(json!({})),Decision::Deny,"drop F4b"),
          ("BACKSTOP F3b: freshness-blind permit, ancient distribute","dist1","DistributeSnapshot","snap_ancient",t(json!({})),Decision::Deny,"drop F3b's DistributeSnapshot arm"),
          ("BACKSTOP F7: soak-blind permit, 30s soak","admin1","ActivatePolicy","pv_t1",t(json!({"soak_elapsed_seconds":30})),Decision::Deny,"drop F7"),
          ("BACKSTOP F12: step-up-blind permit, no step-up","admin_nostep","WriteRebacTuple","rt_fresh",t(json!({})),Decision::Deny,"drop F12"),
        ]);
    }

    let auth = Authorizer::new();
    let (mut pass, mut fail) = (0usize, 0usize);
    let mut flipped: Vec<String> = Vec::new();
    for (name, p, act, r, ctx, want, red) in cases {
        let action = uid(&format!(r#"OyaPolicy::Action::"{act}""#));
        let rtype = match act { "AuthorPolicy"|"SignPolicy"|"PublishPolicy"|"ActivatePolicy" => "PolicyVersion",
            "WriteRebacTuple"|"ReadRebacTuple" => "RebacTuple",
            "DistributeSnapshot"|"EvaluateAgainstSnapshot" => "Snapshot", _ => "DecisionLog" };
        // Context deliberately built WITHOUT the schema so absent attributes stay absent — that is
        // the S1 precondition and the whole point of those cases.
        let context = Context::from_json_value(ctx, None).expect("ctx");
        let req = Request::new(uid(&format!(r#"OyaPolicy::Principal::"{p}""#)), action,
                               uid(&format!(r#"OyaPolicy::{rtype}::"{r}""#)), context, None).expect("req");
        let got = auth.is_authorized(&req, &ps, &entities).decision();
        if got == want { pass += 1; if !quiet { println!("  ok   {name}  [red if: {red}]"); } }
        else { fail += 1; flipped.push(name.to_string()); if !quiet { println!("  FAIL {name}: want {want:?} got {got:?}"); } }
    }
    if !quiet { println!("\n{pass} passed, {fail} failed"); }
    (pass, fail, flipped)
}

fn main() -> std::process::ExitCode {
    let drop = std::env::var("DROP_POLICY_ID").ok();
    let (_p, fail, _f) = run(drop.as_deref(), false);
    if fail == 0 { std::process::ExitCode::SUCCESS } else { std::process::ExitCode::FAILURE }
}
```

The validator used in §1 and §2, `cedarval.rs`:

```rust
use cedar_policy::{PolicySet, Schema, Validator, ValidationMode};
use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let a: Vec<String> = env::args().collect();
    if a.len() < 3 { eprintln!("usage: cedarval <schema.cedarschema> <policy.cedar>..."); return ExitCode::from(2); }
    let schema_src = match fs::read_to_string(&a[1]) { Ok(s) => s, Err(e) => { eprintln!("READ-FAIL {}: {e}", a[1]); return ExitCode::from(2); } };
    let (schema, warns) = match Schema::from_cedarschema_str(&schema_src) {
        Ok(v) => v, Err(e) => { eprintln!("SCHEMA-PARSE-FAIL {}: {e}", a[1]); return ExitCode::from(1); }
    };
    for w in warns { eprintln!("schema-warning: {w}"); }
    println!("SCHEMA-OK {}", a[1]);
    let mut rc = 0u8;
    for p in &a[2..] {
        let src = match fs::read_to_string(p) { Ok(s) => s, Err(e) => { eprintln!("READ-FAIL {p}: {e}"); rc = 2; continue } };
        let ps = match src.parse::<PolicySet>() { Ok(v) => v, Err(e) => { eprintln!("PARSE-FAIL {p}: {e}"); rc = 1; continue } };
        let n = ps.policies().count();
        let res = Validator::new(schema.clone()).validate(&ps, ValidationMode::Strict);
        if res.validation_passed() {
            println!("VALIDATE-OK {p} ({n} policies)");
        } else {
            rc = 1;
            println!("VALIDATE-FAIL {p} ({n} policies)");
            for e in res.validation_errors() { println!("   error: {e}"); }
        }
        for w in res.validation_warnings() { println!("   warning: {w}"); }
    }
    ExitCode::from(rc)
}
```
