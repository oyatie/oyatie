use std::path::PathBuf;
use std::process::ExitCode;

use oya_application_app::{
    AutonomyTier, CapabilityAction, CapabilityInvocationPrincipal, CapabilityInvocationRequest,
    CapabilityRegistration, CostBudgetRegistration, DISCOVER_SCOPE, Foundation,
    IdentityRegistration, McpAccessTokenClaims, McpDiscoveryRequest, OutboxPublish, Purpose,
    SubjectClass, TenantCapabilityGrant, TenantRegistration, TokenRequest,
};
use audit_file_adapter::FileAuditLedger;
use messaging_domain::Outbox;
use messaging_file_adapter::FileOutboxStore;
use oya_intelligence_evidence_file_adapter::FileEvidenceChainStore;
use intelligence_run_domain::RunLedger;
use oya_intelligence_run_file_adapter::FileRunLedgerStore;
use intelligence_step_domain::StepLedger;
use oya_intelligence_step_file_adapter::FileStepLedgerStore;
use secrets_domain::{SecretMaterial, SecretRef, SecretVault};
use secrets_file::FileSecretStore;

use crate::foundation_fixture::{
    internal_privacy_data_class, internal_privacy_data_classes,
    publish_capability_invocation_policy, seed_demo_eval,
};

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    // ADR-0083 Tier 1: delegate to `run_inner` that returns `Result<ExitCode, String>`
    // so each fallible `Foundation::*` / `secret_vault.*` call can use `?` propagation
    // instead of `.expect(...)`. Any `Err` reaching here is printed and mapped to
    // `ExitCode::FAILURE`; argument-parse errors keep their dedicated `ExitCode::from(2)`.
    match run_inner(args, usage) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run_inner(args: Vec<String>, usage: &str) -> Result<ExitCode, String> {
    let demo_args = match parse_demo_args(args, usage) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            return Ok(ExitCode::from(2));
        }
    };
    let mut foundation = Foundation::default();
    let tenant = foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: "ten_demo".into(),
            legal_name: "Oyatie Demo Tenant".into(),
            home_region: "region-home".into(),
            residency_class: "strict_home_region".into(),
            regulatory_packs: vec!["oya-pack-alpha".into()],
            autonomy_ceiling: AutonomyTier::T2Advisory,
        })
        .map_err(|error| format!("tenant onboarding failed: {error:?}"))?;
    let _cell = foundation
        .bind_cell(&tenant.id, "region-home-a", "cell-control-a")
        .map_err(|error| format!("demo cell binding failed: {error:?}"))?;
    let user = foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: tenant.id.clone(),
            user_id: "usr_demo_admin".into(),
            primary_identifier: "admin@demo.oyatie.test".into(),
            display_name: "Demo Admin".into(),
            roles: vec!["tenant-admin".into()],
        })
        .map_err(|error| format!("demo identity failed: {error:?}"))?;
    let user_id = user.user_id().as_str().to_string();
    publish_capability_invocation_policy(&mut foundation, &tenant.id, "tenant-admin")
        .map_err(|error| format!("demo capability invocation policy failed: {error:?}"))?;
    let token = foundation
        .issue_token(TokenRequest {
            tenant_id: tenant.id.clone(),
            user_id: user_id.clone(),
            purpose: Purpose::CapabilityInvocation,
            ttl_seconds: 3_600,
            issued_at_epoch_seconds: 0,
        })
        .map_err(|error| format!("demo token failed: {error:?}"))?;
    foundation
        .grant_data_use(
            &tenant.id,
            Purpose::CapabilityInvocation,
            internal_privacy_data_class(),
        )
        .map_err(|error| format!("demo data-use grant failed: {error:?}"))?;
    seed_demo_eval(&mut foundation, "cap.demo.readiness")
        .map_err(|error| format!("demo eval seed failed: {error:?}"))?;
    let capability = foundation
        .register_capability(CapabilityRegistration {
            capability_id: "cap.demo.readiness".into(),
            namespace: "demo".into(),
            action: CapabilityAction::Other,
            required_tier: AutonomyTier::T1ViewOnly,
            touched_privacy_data_classes: internal_privacy_data_classes(),
            evidence_topic: "oya.foundry.capability.invoked".into(),
        })
        .map_err(|error| format!("demo capability failed: {error:?}"))?;
    foundation
        .grant_capability_to_tenant(TenantCapabilityGrant {
            tenant_id: tenant.id.clone(),
            capability_id: capability.id.clone(),
            mcp_visible: true,
        })
        .map_err(|error| format!("demo capability license failed: {error:?}"))?;
    foundation
        .configure_tenant_cost_budget(CostBudgetRegistration {
            tenant_id: tenant.id.clone(),
            capability_id: None,
            window_id: "demo-window".into(),
            monthly_limit_micros: 1_000_000,
            per_invocation_limit_micros: 1_000,
            warning_threshold_percent: 80,
        })
        .map_err(|error| format!("demo cost budget failed: {error:?}"))?;
    let mcp_descriptor = foundation
        .discover_mcp_gateway(McpDiscoveryRequest {
            tenant_id: tenant.id.clone(),
            access_token: McpAccessTokenClaims {
                tenant_id: tenant.id.clone(),
                subject_id: user_id.clone(),
                issuer: "https://auth.oyatie.test/tenants/ten_demo".into(),
                audience: "https://mcp.foundry.region-home.oyatie.test/tenants/ten_demo".into(),
                expires_at_epoch_seconds: 3_600,
                scopes: vec![DISCOVER_SCOPE.into()],
            },
            now_epoch_seconds: 0,
            tld: "test".into(),
            authorization_server: "https://auth.oyatie.test/tenants/ten_demo".into(),
        })
        .map_err(|error| format!("demo MCP discovery failed: {error:?}"))?;
    let receipt = foundation
        .invoke_capability_as_principal(
            CapabilityInvocationPrincipal {
                tenant_id: tenant.id.clone(),
                user_id: user_id.clone(),
                autonomy_ceiling: AutonomyTier::T2Advisory,
            },
            CapabilityInvocationRequest {
                tenant_id: tenant.id.clone(),
                user_id: user_id.clone(),
                capability_id: capability.id.clone(),
                purpose: Purpose::CapabilityInvocation,
                subject_class: SubjectClass::Adult,
                budget_window_id: "demo-window".into(),
                projected_cost_micros: 125,
                started_at_epoch_seconds: 1_000,
            },
        )
        .map_err(|error| format!("demo capability invocation failed: {error:?}"))?;
    let mut secret_vault = SecretVault::default();
    let demo_secret_ref = SecretRef::new(
        tenant.id.clone(),
        capability.id.clone(),
        "provider-api-key".into(),
    )
    .map_err(|error| format!("demo secret ref invalid: {error:?}"))?;
    let demo_secret_material = SecretMaterial::from_bytes(b"sk-demo-provider-key".to_vec())
        .map_err(|error| format!("demo secret material invalid: {error:?}"))?;
    secret_vault
        .put(demo_secret_ref.clone(), demo_secret_material, Some(3_600))
        .map_err(|error| format!("demo secret persist failed: {error:?}"))?;
    if secret_vault.get(&demo_secret_ref, 0).is_err() {
        return Err("demo secret could not be resolved through SecretProvider kernel".to_string());
    }
    foundation
        .publish_outbox(OutboxPublish {
            tenant_id: tenant.id.clone(),
            topic: "oya.demo.readiness.v1".into(),
            idempotency_key: "demo-readiness".into(),
            payload_ref: receipt.evidence_event_hash.clone(),
        })
        .map_err(|error| format!("demo outbox publish failed: {error:?}"))?;
    let audit_persisted = match demo_args.audit_ledger_path {
        Some(path) => persist_audit_ledger(path, &foundation)?,
        None => false,
    };
    let evidence_persisted = match demo_args.evidence_store_path {
        Some(path) => persist_evidence_store(path, &foundation)?,
        None => false,
    };
    let run_persisted = match demo_args.run_ledger_path {
        Some(path) => persist_run_ledger(path, &foundation)?,
        None => false,
    };
    let step_persisted = match demo_args.step_ledger_path {
        Some(path) => persist_step_ledger(path, &foundation)?,
        None => false,
    };
    let outbox_persisted = match demo_args.outbox_store_path {
        Some(path) => persist_outbox_store(path, &foundation)?,
        None => false,
    };
    let secret_persisted = match demo_args.secret_store_path {
        Some(path) => persist_secret_store(path, &secret_vault)?,
        None => false,
    };

    println!(
        "Oyatie foundation ready: tenant={} token_expires={} mcp_tools={} evidence_hash={} audit_events={} audit_verified={} audit_persisted={} evidence_persisted={} run_records={} run_persisted={} step_records={} step_persisted={} outbox_records={} outbox_persisted={} secret_versions={} secret_persisted={}",
        tenant.id,
        token.expires_at_epoch_seconds,
        mcp_descriptor.tools.len(),
        receipt.evidence_event_hash,
        foundation.audit_chain().events().len(),
        foundation.audit_chain().verify(),
        audit_persisted,
        evidence_persisted,
        foundation.foundry_runs().len(),
        run_persisted,
        foundation.foundry_steps().len(),
        step_persisted,
        foundation.outbox_records().len(),
        outbox_persisted,
        secret_vault.records().len(),
        secret_persisted
    );
    Ok(ExitCode::SUCCESS)
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct DemoArgs {
    audit_ledger_path: Option<PathBuf>,
    evidence_store_path: Option<PathBuf>,
    run_ledger_path: Option<PathBuf>,
    step_ledger_path: Option<PathBuf>,
    outbox_store_path: Option<PathBuf>,
    secret_store_path: Option<PathBuf>,
}

fn parse_demo_args(args: Vec<String>, usage: &str) -> Result<DemoArgs, String> {
    let mut parsed = DemoArgs::default();
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage.to_string());
        };
        match flag.as_str() {
            "--audit-ledger" => parsed.audit_ledger_path = Some(PathBuf::from(path)),
            "--evidence-store" => parsed.evidence_store_path = Some(PathBuf::from(path)),
            "--run-ledger" => parsed.run_ledger_path = Some(PathBuf::from(path)),
            "--step-ledger" => parsed.step_ledger_path = Some(PathBuf::from(path)),
            "--outbox-store" => parsed.outbox_store_path = Some(PathBuf::from(path)),
            "--secret-store" => parsed.secret_store_path = Some(PathBuf::from(path)),
            _ => return Err(usage.to_string()),
        }
    }
    Ok(parsed)
}

fn persist_audit_ledger(path: PathBuf, foundation: &Foundation) -> Result<bool, String> {
    let ledger = FileAuditLedger::new(path);
    ledger
        .append_chain(foundation.audit_chain())
        .map_err(|error| format!("audit ledger persist failed: {error:?}"))?;
    let replayed = ledger
        .load_multi_tenant_shards()
        .map_err(|error| format!("audit ledger replay failed: {error:?}"))?;
    if replayed.events() == foundation.audit_chain().events() && replayed.verify() {
        Ok(true)
    } else {
        Err("audit ledger replay diverged from in-memory chain".to_string())
    }
}

fn persist_outbox_store(path: PathBuf, foundation: &Foundation) -> Result<bool, String> {
    let store = FileOutboxStore::new(path);
    let outbox = Outbox::from_records(foundation.outbox_records().to_vec())
        .map_err(|error| format!("outbox snapshot invalid: {error:?}"))?;
    store
        .append_outbox(&outbox)
        .map_err(|error| format!("outbox store persist failed: {error:?}"))?;
    let replayed = store
        .load()
        .map_err(|error| format!("outbox store replay failed: {error:?}"))?;
    if replayed.records() == foundation.outbox_records() {
        Ok(true)
    } else {
        Err("outbox store replay diverged from in-memory records".to_string())
    }
}

fn persist_evidence_store(path: PathBuf, foundation: &Foundation) -> Result<bool, String> {
    let store = FileEvidenceChainStore::new(path);
    store
        .append_chain(foundation.foundry_evidence_chain())
        .map_err(|error| format!("evidence store persist failed: {error:?}"))?;
    let replayed = store
        .load()
        .map_err(|error| format!("evidence store replay failed: {error:?}"))?;
    if replayed.records() == foundation.foundry_evidence_chain().records() && replayed.verify() {
        Ok(true)
    } else {
        Err("evidence store replay diverged from in-memory chain".to_string())
    }
}

fn persist_run_ledger(path: PathBuf, foundation: &Foundation) -> Result<bool, String> {
    let store = FileRunLedgerStore::new(path);
    let ledger = RunLedger::from_runs(foundation.foundry_runs().to_vec())
        .map_err(|error| format!("run ledger snapshot invalid: {error:?}"))?;
    store
        .save_ledger(&ledger)
        .map_err(|error| format!("run ledger persist failed: {error:?}"))?;
    let replayed = store
        .load()
        .map_err(|error| format!("run ledger replay failed: {error:?}"))?;
    if replayed.runs() == foundation.foundry_runs() {
        Ok(true)
    } else {
        Err("run ledger replay diverged from in-memory runs".to_string())
    }
}

fn persist_step_ledger(path: PathBuf, foundation: &Foundation) -> Result<bool, String> {
    let store = FileStepLedgerStore::new(path);
    let ledger = StepLedger::from_steps(foundation.foundry_steps().to_vec())
        .map_err(|error| format!("step ledger snapshot invalid: {error:?}"))?;
    store
        .save_ledger(&ledger)
        .map_err(|error| format!("step ledger persist failed: {error:?}"))?;
    let replayed = store
        .load()
        .map_err(|error| format!("step ledger replay failed: {error:?}"))?;
    if replayed.steps() == foundation.foundry_steps() {
        Ok(true)
    } else {
        Err("step ledger replay diverged from in-memory steps".to_string())
    }
}

fn persist_secret_store(path: PathBuf, secret_vault: &SecretVault) -> Result<bool, String> {
    let store = FileSecretStore::new(path);
    store
        .append_vault(secret_vault)
        .map_err(|error| format!("secret store persist failed: {error:?}"))?;
    if store
        .matches_vault_metadata(secret_vault)
        .map_err(|error| format!("secret store metadata validation failed: {error:?}"))?
    {
        Ok(true)
    } else {
        Err("secret store metadata diverged from in-memory vault".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_demo_args_accepts_all_persistence_paths() {
        let args = parse_demo_args(
            vec![
                "--audit-ledger".into(),
                "audit.jsonl".into(),
                "--evidence-store".into(),
                "evidence.jsonl".into(),
                "--run-ledger".into(),
                "runs.json".into(),
                "--step-ledger".into(),
                "steps.json".into(),
                "--outbox-store".into(),
                "outbox.jsonl".into(),
                "--secret-store".into(),
                "secrets.jsonl".into(),
            ],
            "usage text",
        )
        .expect("demo args parse");

        assert_eq!(args.audit_ledger_path, Some(PathBuf::from("audit.jsonl")));
        assert_eq!(
            args.evidence_store_path,
            Some(PathBuf::from("evidence.jsonl"))
        );
        assert_eq!(args.run_ledger_path, Some(PathBuf::from("runs.json")));
        assert_eq!(args.step_ledger_path, Some(PathBuf::from("steps.json")));
        assert_eq!(args.outbox_store_path, Some(PathBuf::from("outbox.jsonl")));
        assert_eq!(args.secret_store_path, Some(PathBuf::from("secrets.jsonl")));
    }

    #[test]
    fn parse_demo_args_returns_usage_for_dangling_flag() {
        assert_eq!(
            parse_demo_args(vec!["--audit-ledger".into()], "usage text"),
            Err("usage text".to_string())
        );
    }
}
