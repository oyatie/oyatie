use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use audit_file_adapter::FileAuditLedger;
use intelligence_bypass_domain::{
    AutonomyBreakGlassInput, AutonomyTier, BypassLedger, BypassLedgerRecord, FoundationBypassInput,
};

use crate::{current_epoch_days, parse_u32_field, parse_u64_field, usage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FoundationBypassValidateArgs {
    ledger_dir: PathBuf,
    now_epoch_days: u64,
}

pub(crate) fn parse_foundation_bypass_validate_args(
    args: Vec<String>,
) -> Result<FoundationBypassValidateArgs, String> {
    let mut parsed = FoundationBypassValidateArgs {
        ledger_dir: PathBuf::from("registry/foundation-bypasses"),
        now_epoch_days: current_epoch_days()?,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(value) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--ledger" => parsed.ledger_dir = PathBuf::from(value),
            "--now-epoch-days" => {
                parsed.now_epoch_days = value
                    .parse::<u64>()
                    .map_err(|_| "--now-epoch-days must be an unsigned integer".to_string())?;
            }
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_foundation_bypass_gate(
    args: FoundationBypassValidateArgs,
) -> Result<(usize, usize), String> {
    let records = read_foundation_bypasses(&args.ledger_dir)?;
    let ledger = BypassLedger::from_ledger_records(records)
        .map_err(|error| format!("gate exception ledger invalid: {error:?}"))?;
    ledger
        .validate_windows(args.now_epoch_days)
        .map_err(|error| format!("gate exception expiry invalid: {error:?}"))?;
    Ok((ledger.len(), ledger.open_count()))
}

// Grounded domain path: foundation bypass records are explicit, expiring
// gate-exception ledger entries. Loading fails closed and every entry is
// validated by oya-intelligence-bypass-kernel; this is not recovery-path fallback.
fn read_foundation_bypasses(ledger_dir: &Path) -> Result<Vec<BypassLedgerRecord>, String> {
    let entries = fs::read_dir(ledger_dir)
        .map_err(|error| format!("foundation gate exception directory unreadable: {error}"))?;
    let mut records = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!("foundation gate exception ledger entry unreadable: {error}")
        })?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("yaml") {
            continue;
        }
        let contents = fs::read_to_string(&path).map_err(|error| {
            format!(
                "foundation gate exception record unreadable {}: {error}",
                path.display()
            )
        })?;
        let record = parse_foundation_bypass_record(&contents)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        records.push(record);
    }
    Ok(records)
}

fn parse_foundation_bypass_record(contents: &str) -> Result<BypassLedgerRecord, String> {
    let mut fields = parse_scalar_fields(contents)?;
    let entry_class = fields
        .remove("entry_class")
        .unwrap_or_else(|| "foundation-bypass".to_string());
    match entry_class.as_str() {
        "foundation-bypass" => parse_foundation_bypass_fields(fields)?
            .build()
            .map(BypassLedgerRecord::from)
            .map_err(|error| format!("{error:?}")),
        "autonomy-break-glass" => parse_autonomy_break_glass_fields(fields)?
            .build()
            .map(BypassLedgerRecord::from)
            .map_err(|error| format!("{error:?}")),
        other => Err(format!("unknown entry_class {other}")),
    }
}

fn parse_scalar_fields(contents: &str) -> Result<BTreeMap<String, String>, String> {
    let mut fields = BTreeMap::new();
    let mut seen_fields = BTreeSet::new();
    for (line_index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            return Err(format!(
                "malformed line {}: expected key: value",
                line_index + 1
            ));
        };
        let key = key.trim();
        if !seen_fields.insert(key.to_string()) {
            return Err(format!("duplicate field {key}"));
        }
        let value = value.trim();
        fields.insert(key.to_string(), value.to_string());
    }
    Ok(fields)
}

fn parse_foundation_bypass_fields(
    mut fields: BTreeMap<String, String>,
) -> Result<FoundationBypassInput, String> {
    let id = take_required_field(&mut fields, "id")?;
    let pr_ref = take_required_field(&mut fields, "pr_ref")?;
    let crate_ref = take_required_field(&mut fields, "crate_ref")?;
    let gate_bypassed = take_required_field(&mut fields, "gate_bypassed")?;
    let bypassing_actor = take_required_field(&mut fields, "bypassing_actor")?;
    let rationale = take_required_field(&mut fields, "rationale")?;
    let regression_window_days = parse_u32_field(
        &take_required_field(&mut fields, "regression_window_days")?,
        "regression_window_days",
    )?;
    let created_at_epoch_days = parse_u64_field(
        &take_required_field(&mut fields, "created_at_epoch_days")?,
        "created_at_epoch_days",
    )?;
    let remediated_at_epoch_days = take_optional_field(&mut fields, "remediated_at_epoch_days")
        .map(|value| parse_u64_field(&value, "remediated_at_epoch_days"))
        .transpose()?;
    reject_unknown_fields(fields)?;
    Ok(FoundationBypassInput {
        id,
        pr_ref,
        crate_ref,
        gate_bypassed,
        bypassing_actor,
        rationale,
        regression_window_days,
        created_at_epoch_days,
        remediated_at_epoch_days,
    })
}

fn parse_autonomy_break_glass_fields(
    mut fields: BTreeMap<String, String>,
) -> Result<AutonomyBreakGlassInput, String> {
    let id = take_required_field(&mut fields, "id")?;
    let tenant_id = take_required_field(&mut fields, "tenant_id")?;
    let capability_id = take_required_field(&mut fields, "capability_id")?;
    let requested_tier = parse_autonomy_tier(&take_required_field(&mut fields, "requested_tier")?)?;
    let permitted_tier = parse_autonomy_tier(&take_required_field(&mut fields, "permitted_tier")?)?;
    let requesting_actor = take_required_field(&mut fields, "requesting_actor")?;
    let approving_actors = parse_actor_list(&take_required_field(&mut fields, "approving_actors")?);
    let approval_quorum = take_required_field(&mut fields, "approval_quorum")?;
    let rationale = take_required_field(&mut fields, "rationale")?;
    let created_at_epoch_days = parse_u64_field(
        &take_required_field(&mut fields, "created_at_epoch_days")?,
        "created_at_epoch_days",
    )?;
    let expires_at_epoch_days = parse_u64_field(
        &take_required_field(&mut fields, "expires_at_epoch_days")?,
        "expires_at_epoch_days",
    )?;
    let revoked_at_epoch_days = take_optional_field(&mut fields, "revoked_at_epoch_days")
        .map(|value| parse_u64_field(&value, "revoked_at_epoch_days"))
        .transpose()?;
    reject_unknown_fields(fields)?;
    Ok(AutonomyBreakGlassInput {
        id,
        tenant_id,
        capability_id,
        requested_tier,
        permitted_tier,
        requesting_actor,
        approving_actors,
        approval_quorum,
        rationale,
        created_at_epoch_days,
        expires_at_epoch_days,
        revoked_at_epoch_days,
    })
}

fn take_required_field(fields: &mut BTreeMap<String, String>, key: &str) -> Result<String, String> {
    fields.remove(key).ok_or_else(|| format!("missing {key}"))
}

fn take_optional_field(fields: &mut BTreeMap<String, String>, key: &str) -> Option<String> {
    fields.remove(key)
}

fn reject_unknown_fields(fields: BTreeMap<String, String>) -> Result<(), String> {
    if let Some(unknown) = fields.keys().next() {
        Err(format!("unknown field {unknown}"))
    } else {
        Ok(())
    }
}

fn parse_actor_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|actor| actor.trim().to_string())
        .collect()
}

fn parse_autonomy_tier(value: &str) -> Result<AutonomyTier, String> {
    match value {
        "T1" | "T1ViewOnly" => Ok(AutonomyTier::T1ViewOnly),
        "T2" | "T2Advisory" => Ok(AutonomyTier::T2Advisory),
        "T3" | "T3ExecuteWithApproval" => Ok(AutonomyTier::T3ExecuteWithApproval),
        "T4" | "T4AutoExecute" => Ok(AutonomyTier::T4AutoExecute),
        _ => Err(format!("unknown autonomy tier {value}")),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuditChainReplayValidateArgs {
    shards_dir: PathBuf,
}

pub(crate) fn parse_audit_chain_replay_validate_args(
    args: Vec<String>,
) -> Result<AuditChainReplayValidateArgs, String> {
    let mut parsed = AuditChainReplayValidateArgs {
        shards_dir: PathBuf::from("registry/audit-chain/shards"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(value) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--shards-dir" => parsed.shards_dir = PathBuf::from(value),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_audit_chain_replay_gate(
    args: AuditChainReplayValidateArgs,
) -> Result<(usize, usize), String> {
    let shard_paths = list_audit_shard_paths(&args.shards_dir)?;
    let mut event_count = 0usize;
    for path in &shard_paths {
        let ledger = FileAuditLedger::new(path.clone());
        let chain = ledger
            .load_multi_tenant_shards()
            .map_err(|error| format!("audit shard replay failed {}: {error:?}", path.display()))?;
        if chain.events().is_empty() {
            return Err(format!(
                "audit shard contains no events: {}",
                path.display()
            ));
        }
        if !chain.verify() {
            return Err(format!(
                "audit shard hash chain invalid: {}",
                path.display()
            ));
        }
        event_count += chain.events().len();
    }
    Ok((shard_paths.len(), event_count))
}

fn list_audit_shard_paths(shards_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    collect_audit_shard_paths(shards_dir, &mut paths)?;
    paths.sort();
    if paths.is_empty() {
        Err(format!(
            "audit chain shards directory contains no .log or .ledger files: {}",
            shards_dir.display()
        ))
    } else {
        Ok(paths)
    }
}

fn collect_audit_shard_paths(dir: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(dir)
        .map_err(|error| format!("audit chain shards directory unreadable: {error}"))?
    {
        let entry =
            entry.map_err(|error| format!("audit chain shard entry unreadable: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_audit_shard_paths(&path, paths)?;
            continue;
        }
        if let Some("ledger" | "log") = path.extension().and_then(|extension| extension.to_str()) {
            paths.push(path);
        }
    }
    Ok(())
}
