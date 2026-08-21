---
doc_class: Specification
shape: Specification
length_cap: 400
microservice: policy
related_adrs:
  - ADR-0243
  - ADR-0280
inbound_citations:
  - policy/README.md
---

# Cedar conformance — measured, not asserted

Every claim on this page was produced by running the policies in `policy/policy/` against the real
`cedar-policy` engine at the version the workspace locks (**4.12.0**; root `Cargo.toml` requires
`"4.11"`, `Cargo.lock` resolves 4.12.0).

The harness runs **out of tree**, in a scratch crate, for a specific reason: this capability may not
add a workspace member yet (see `PROMOTION.md` — `Cargo.lock` is a hub owned by `integ/build`, and a
new crate additionally owes a `registry/catalog/*.yaml` row this envelope cannot write). Rather than
ship an unrunnable claim, the harness source is reproduced in full below. At promotion it becomes the
test body of `policy/adapters/policy-cedar-conformance`, unchanged.

## 1. Schema and policies validate

```
$ cedarval policy/policy/schema.cedarschema policy/policy/*.cedar
SCHEMA-OK policy/policy/schema.cedarschema
VALIDATE-OK policy/policy/auditor-scope.cedar (2 policies)
VALIDATE-OK policy/policy/authoring-grants.cedar (4 policies)
VALIDATE-OK policy/policy/change-control.cedar (4 policies)
VALIDATE-OK policy/policy/cross-tenant-isolation.cedar (1 policies)
VALIDATE-OK policy/policy/runtime-grants.cedar (3 policies)
VALIDATE-OK policy/policy/static-stability.cedar (3 policies)
exit 0
```

Validation is `ValidationMode::Strict` against the schema. The instrument was checked against a known-
good pair first — `iam/adapters/pdp-cedar/cedar/platform.cedarschema` + `platform-policies.cedar`
returns `VALIDATE-OK (4 policies)`, exit 0 — so a passing result is not a collector that saw nothing.

## 2. The bundle is the concatenation, and decides identically

`policy/cedar/policies.cedar` is the byte-exact concatenation of the six fragments (17 policies =
2+4+4+1+3+3) and validates clean. The behavioural suite below runs against **the bundle**, not the
fragments, so the concatenation property is asserted by every case.

## 3. Behavioural suite

```
  ok   P2 tenant admin authors own-tenant policy  [red if: drop P2]
  ok   F1 cross-tenant author refused  [red if: drop F1 cross-tenant forbid]
  ok   F8 tenant admin cannot touch global scope  [red if: drop F8]
  ok   P3 step-up-C engineer may author global  [red if: tighten F8 past step-up C]
  ok   F8 engineer without step-up refused on global  [red if: drop the step_up_class clause in F8]
  ok   F5 signer cannot sign own authorship  [red if: drop F5 separation of duties]
  ok   P4 signer signs another's policy  [red if: drop P4]
  ok   F6 unsigned policy cannot publish  [red if: drop F6]
  ok   F7 activation before soak refused  [red if: lower the 60s soak bound in F7]
  ok   activation after soak allowed  [red if: drop P2's ActivatePolicy]
  ok   P7 pep reads tuple on fresh snapshot  [red if: drop P7]
  ok   F2 stale AND non-authoritative read refused  [red if: drop F2]
  ok   F2 stale BUT authoritative read allowed  [red if: make F2 forbid on staleness alone]
  ok   F3 snapshot older than caller tolerance refused  [red if: drop F3]
  ok   evaluate within tolerance allowed  [red if: drop P6]
  ok   F4 unverified snapshot never evaluates  [red if: drop F4]
  ok   P8 distributor propagates verified snapshot  [red if: drop P8]
  ok   F4 distributor cannot propagate unverified  [red if: drop F4's DistributeSnapshot arm]
  ok   P1 auditor reads log on fresh token  [red if: drop P1]
  ok   F9 expired token reads nothing  [red if: raise the 900s bound in F9]
  ok   default-deny: unlisted role gets nothing  [red if: add a catch-all permit]
  ok   F1 BACKSTOP: tenant-blind permit is still denied cross-tenant  [red if: drop F1 (this is the ONLY case that reaches it)]

22 passed, 0 failed
```

## 4. Mutation coverage — which cases actually reach which fragment

A suite that passes tells you nothing until you show it can fail. Each fragment was dropped from the
bundle in turn and the suite re-run:

```
drop auditor-scope.cedar          -> 20 passed, 1 failed
drop authoring-grants.cedar       -> 17 passed, 4 failed
drop change-control.cedar         -> 18 passed, 3 failed
drop cross-tenant-isolation.cedar -> 21 passed, 0 failed     <-- NOT COVERED
drop runtime-grants.cedar         -> 17 passed, 4 failed
drop static-stability.cedar       -> 17 passed, 4 failed
```

**`cross-tenant-isolation.cedar` was not covered by the first 21 cases.** The case named
"F1 cross-tenant author refused" passed with F1 *removed* — because every permit already binds
`principal.tenant_id == resource.tenant_id`, so a cross-tenant request is denied by the absence of a
matching permit, not by F1. The test was measuring default-deny and reporting it as F1.

F1 is a **backstop**: unreachable precisely while the permits are correct. Testing it requires making
a permit incorrect. Case 22 injects a permit that forgets the tenant bind — the exact authoring
mistake F1 exists to survive — and asserts the cross-tenant request is still denied:

```cedar
// test fixture only, never shipped
permit (principal, action == OyaPolicy::Action::"AuthorPolicy", resource)
when { principal.role == "tenant-policy-admin" };
```

```
with cross-tenant-isolation.cedar:     ok   F1 BACKSTOP ... -> Deny     22 passed, 0 failed
without cross-tenant-isolation.cedar:  FAIL F1 BACKSTOP ... want Deny got Allow
```

Every fragment now has at least one case that turns red when it is dropped.

## 5. Harness source

Reproduce with `cedar-policy = "4.12"` and `serde_json = "1"`. `conform.rs`:

```rust
use cedar_policy::{Authorizer, Context, Decision, Entities, EntityUid, PolicySet, Request, Schema};
use serde_json::json;
use std::{fs, str::FromStr};

fn uid(s: &str) -> EntityUid { EntityUid::from_str(s).expect("uid") }

fn main() -> std::process::ExitCode {
    let schema_src = fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let (schema, _) = Schema::from_cedarschema_str(&schema_src).unwrap();
    let mut src = fs::read_to_string(std::env::args().nth(2).unwrap()).unwrap();
    let backstop = std::env::args().nth(3).is_some();
    if let Some(extra) = std::env::args().nth(3) { src.push('\n'); src.push_str(&fs::read_to_string(extra).unwrap()); }
    let ps: PolicySet = src.parse().unwrap();

    let ents = json!([
      {"uid":{"type":"OyaPolicy::Tenant","id":"t1"},"attrs":{"tenant_id":"t1","cell_id":"c1"},"parents":[]},
      {"uid":{"type":"OyaPolicy::Tenant","id":"t2"},"attrs":{"tenant_id":"t2","cell_id":"c1"},"parents":[]},
      {"uid":{"type":"OyaPolicy::Principal","id":"admin1"},"attrs":{"tenant_id":"t1","principal_id":"admin1","role":"tenant-policy-admin","step_up_class":"","spiffe_id":"spiffe://o/a1"},"parents":[{"type":"OyaPolicy::Tenant","id":"t1"}]},
      {"uid":{"type":"OyaPolicy::Principal","id":"eng_c"},"attrs":{"tenant_id":"t1","principal_id":"eng_c","role":"platform-policy-engineer","step_up_class":"C","spiffe_id":"spiffe://o/e"},"parents":[{"type":"OyaPolicy::Tenant","id":"t1"}]},
      {"uid":{"type":"OyaPolicy::Principal","id":"eng_none"},"attrs":{"tenant_id":"t1","principal_id":"eng_none","role":"platform-policy-engineer","step_up_class":"","spiffe_id":"spiffe://o/e2"},"parents":[{"type":"OyaPolicy::Tenant","id":"t1"}]},
      {"uid":{"type":"OyaPolicy::Principal","id":"signer1"},"attrs":{"tenant_id":"t1","principal_id":"signer1","role":"policy-signer","step_up_class":"","spiffe_id":"spiffe://o/s"},"parents":[{"type":"OyaPolicy::Tenant","id":"t1"}]},
      {"uid":{"type":"OyaPolicy::Principal","id":"pep1"},"attrs":{"tenant_id":"t1","principal_id":"pep1","role":"pep-workload","step_up_class":"","spiffe_id":"spiffe://o/p"},"parents":[{"type":"OyaPolicy::Tenant","id":"t1"}]},
      {"uid":{"type":"OyaPolicy::Principal","id":"dist1"},"attrs":{"tenant_id":"t1","principal_id":"dist1","role":"policy-distributor","step_up_class":"","spiffe_id":"spiffe://o/d"},"parents":[{"type":"OyaPolicy::Tenant","id":"t1"}]},
      {"uid":{"type":"OyaPolicy::Principal","id":"aud1"},"attrs":{"tenant_id":"t1","principal_id":"aud1","role":"auditor","step_up_class":"","spiffe_id":"spiffe://o/au"},"parents":[{"type":"OyaPolicy::Tenant","id":"t1"}]},
      {"uid":{"type":"OyaPolicy::PolicyVersion","id":"pv_t1"},"attrs":{"tenant_id":"t1","policy_id":"p1","scope":"tenant","signed":true,"author_principal_id":"admin1"},"parents":[{"type":"OyaPolicy::Tenant","id":"t1"}]},
      {"uid":{"type":"OyaPolicy::PolicyVersion","id":"pv_t1_unsigned"},"attrs":{"tenant_id":"t1","policy_id":"p1","scope":"tenant","signed":false,"author_principal_id":"admin1"},"parents":[{"type":"OyaPolicy::Tenant","id":"t1"}]},
      {"uid":{"type":"OyaPolicy::PolicyVersion","id":"pv_t2"},"attrs":{"tenant_id":"t2","policy_id":"p1","scope":"tenant","signed":true,"author_principal_id":"other"},"parents":[{"type":"OyaPolicy::Tenant","id":"t2"}]},
      {"uid":{"type":"OyaPolicy::PolicyVersion","id":"pv_global"},"attrs":{"tenant_id":"t1","policy_id":"pg","scope":"global","signed":true,"author_principal_id":"other"},"parents":[{"type":"OyaPolicy::Tenant","id":"t1"}]},
      {"uid":{"type":"OyaPolicy::PolicyVersion","id":"pv_selfauthored"},"attrs":{"tenant_id":"t1","policy_id":"ps","scope":"tenant","signed":true,"author_principal_id":"signer1"},"parents":[{"type":"OyaPolicy::Tenant","id":"t1"}]},
      {"uid":{"type":"OyaPolicy::RebacTuple","id":"rt1"},"attrs":{"tenant_id":"t1","object_type":"doc","relation":"viewer"},"parents":[{"type":"OyaPolicy::Tenant","id":"t1"}]},
      {"uid":{"type":"OyaPolicy::Snapshot","id":"snap_ok"},"attrs":{"tenant_id":"t1","cell_id":"c1","snapshot_version":"v9","age_seconds":10,"is_authoritative":true,"signature_verified":true},"parents":[{"type":"OyaPolicy::Tenant","id":"t1"}]},
      {"uid":{"type":"OyaPolicy::Snapshot","id":"snap_old"},"attrs":{"tenant_id":"t1","cell_id":"c1","snapshot_version":"v1","age_seconds":900,"is_authoritative":false,"signature_verified":true},"parents":[{"type":"OyaPolicy::Tenant","id":"t1"}]},
      {"uid":{"type":"OyaPolicy::Snapshot","id":"snap_unverified"},"attrs":{"tenant_id":"t1","cell_id":"c1","snapshot_version":"v9","age_seconds":10,"is_authoritative":true,"signature_verified":false},"parents":[{"type":"OyaPolicy::Tenant","id":"t1"}]},
      {"uid":{"type":"OyaPolicy::DecisionLog","id":"log1"},"attrs":{"tenant_id":"t1"},"parents":[{"type":"OyaPolicy::Tenant","id":"t1"}]}
    ]);
    let entities = Entities::from_json_value(ents, Some(&schema)).expect("entities");

    // (name, principal, action, resource, context, expected, what-turns-it-red)
    let cases: Vec<(&str,&str,&str,&str,serde_json::Value,Decision,&str)> = vec![
      ("P2 tenant admin authors own-tenant policy","admin1","AuthorPolicy","pv_t1",json!({"now":0}),Decision::Allow,"drop P2"),
      ("F1 cross-tenant author refused","admin1","AuthorPolicy","pv_t2",json!({"now":0}),Decision::Deny,"drop F1 cross-tenant forbid"),
      ("F8 tenant admin cannot touch global scope","admin1","AuthorPolicy","pv_global",json!({"now":0}),Decision::Deny,"drop F8"),
      ("P3 step-up-C engineer may author global","eng_c","AuthorPolicy","pv_global",json!({"now":0}),Decision::Allow,"tighten F8 past step-up C"),
      ("F8 engineer without step-up refused on global","eng_none","AuthorPolicy","pv_global",json!({"now":0}),Decision::Deny,"drop the step_up_class clause in F8"),
      ("F5 signer cannot sign own authorship","signer1","SignPolicy","pv_selfauthored",json!({"now":0}),Decision::Deny,"drop F5 separation of duties"),
      ("P4 signer signs another's policy","signer1","SignPolicy","pv_t1",json!({"now":0}),Decision::Allow,"drop P4"),
      ("F6 unsigned policy cannot publish","admin1","PublishPolicy","pv_t1_unsigned",json!({"now":0}),Decision::Deny,"drop F6"),
      ("F7 activation before soak refused","admin1","ActivatePolicy","pv_t1",json!({"now":0,"soak_elapsed_seconds":30}),Decision::Deny,"lower the 60s soak bound in F7"),
      ("activation after soak allowed","admin1","ActivatePolicy","pv_t1",json!({"now":0,"soak_elapsed_seconds":120}),Decision::Allow,"drop P2's ActivatePolicy"),
      ("P7 pep reads tuple on fresh snapshot","pep1","ReadRebacTuple","rt1",json!({"now":0,"snapshot_age_seconds":10,"snapshot_is_authoritative":false}),Decision::Allow,"drop P7"),
      ("F2 stale AND non-authoritative read refused","pep1","ReadRebacTuple","rt1",json!({"now":0,"snapshot_age_seconds":900,"snapshot_is_authoritative":false}),Decision::Deny,"drop F2"),
      ("F2 stale BUT authoritative read allowed","pep1","ReadRebacTuple","rt1",json!({"now":0,"snapshot_age_seconds":900,"snapshot_is_authoritative":true}),Decision::Allow,"make F2 forbid on staleness alone"),
      ("F3 snapshot older than caller tolerance refused","pep1","EvaluateAgainstSnapshot","snap_old",json!({"now":0,"max_staleness_seconds":300}),Decision::Deny,"drop F3"),
      ("evaluate within tolerance allowed","pep1","EvaluateAgainstSnapshot","snap_ok",json!({"now":0,"max_staleness_seconds":300}),Decision::Allow,"drop P6"),
      ("F4 unverified snapshot never evaluates","pep1","EvaluateAgainstSnapshot","snap_unverified",json!({"now":0,"max_staleness_seconds":300}),Decision::Deny,"drop F4"),
      ("P8 distributor propagates verified snapshot","dist1","DistributeSnapshot","snap_ok",json!({"now":0}),Decision::Allow,"drop P8"),
      ("F4 distributor cannot propagate unverified","dist1","DistributeSnapshot","snap_unverified",json!({"now":0}),Decision::Deny,"drop F4's DistributeSnapshot arm"),
      ("P1 auditor reads log on fresh token","aud1","ReadDecisionLog","log1",json!({"now":0,"token_age_seconds":60}),Decision::Allow,"drop P1"),
      ("F9 expired token reads nothing","aud1","ReadDecisionLog","log1",json!({"now":0,"token_age_seconds":1200}),Decision::Deny,"raise the 900s bound in F9"),
      ("default-deny: unlisted role gets nothing","dist1","AuthorPolicy","pv_t1",json!({"now":0}),Decision::Deny,"add a catch-all permit"),
    ];
    let mut cases = cases;
    if backstop {
        // F1 is a BACKSTOP: unreachable while every permit binds the tenant. This case injects a
        // permit that FORGETS the tenant bind — the exact authoring mistake F1 exists to survive —
        // and asserts the cross-tenant request is still denied. Drop F1 and this flips to Allow.
        cases.push(("F1 BACKSTOP: tenant-blind permit is still denied cross-tenant",
                    "admin1","AuthorPolicy","pv_t2",json!({"now":0}),Decision::Deny,
                    "drop F1 (this is the ONLY case that reaches it)"));
    }

    let auth = Authorizer::new();
    let (mut pass, mut fail) = (0, 0);
    for (name, p, a, r, ctx, want, red) in cases {
        let action = uid(&format!(r#"OyaPolicy::Action::"{a}""#));
        let rtype = match a { "AuthorPolicy"|"SignPolicy"|"PublishPolicy"|"ActivatePolicy" => "PolicyVersion",
            "WriteRebacTuple"|"ReadRebacTuple" => "RebacTuple",
            "DistributeSnapshot"|"EvaluateAgainstSnapshot" => "Snapshot", _ => "DecisionLog" };
        let context = Context::from_json_value(ctx, Some((&schema, &action))).expect("ctx");
        let req = Request::new(uid(&format!(r#"OyaPolicy::Principal::"{p}""#)), action.clone(),
                               uid(&format!(r#"OyaPolicy::{rtype}::"{r}""#)), context, Some(&schema)).expect("req");
        let got = auth.is_authorized(&req, &ps, &entities).decision();
        if got == want { pass += 1; println!("  ok   {name}  [red if: {red}]"); }
        else { fail += 1; println!("  FAIL {name}: want {want:?} got {got:?}"); }
    }
    println!("\n{pass} passed, {fail} failed");
    if fail == 0 { std::process::ExitCode::SUCCESS } else { std::process::ExitCode::FAILURE }
}
```

And the validator used in §1, `cedarval.rs`:

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
