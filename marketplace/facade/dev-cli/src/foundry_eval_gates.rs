use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use application_app::{
    AdversarialKind, EvalCaseInput, EvalGate, EvalMetric, EvalRunInput, EvalSetInput,
};

use crate::{
    insert_scalar_field, parse_bool_field, parse_u8_percent, required_field, required_scalar, usage,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FoundryEvalValidateArgs {
    capabilities_dir: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FoundryCapabilityEvalRecord {
    capability_id: String,
    eval_set_path: PathBuf,
    eval_run_path: PathBuf,
}

pub(crate) fn parse_foundry_eval_validate_args(
    args: Vec<String>,
) -> Result<FoundryEvalValidateArgs, String> {
    let mut parsed = FoundryEvalValidateArgs {
        capabilities_dir: PathBuf::from("registry/capability-templates"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(value) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--capabilities-dir" => parsed.capabilities_dir = PathBuf::from(value),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_foundry_eval_gate(
    args: FoundryEvalValidateArgs,
) -> Result<(usize, usize, usize), String> {
    let records = read_foundry_capability_eval_records(&args.capabilities_dir)?;
    let mut eval_gate = EvalGate::default();
    let mut case_count = 0usize;
    let mut run_count = 0usize;

    for record in &records {
        let eval_set_path = args.capabilities_dir.join(&record.eval_set_path);
        let eval_set_contents = fs::read_to_string(&eval_set_path)
            .map_err(|error| format!("eval set unreadable {}: {error}", eval_set_path.display()))?;
        let eval_set = parse_eval_set_input(&eval_set_path, &eval_set_contents)?;
        if eval_set.capability_id != record.capability_id {
            return Err(format!(
                "{}: eval set capability_id {} does not match capability record {}",
                eval_set_path.display(),
                eval_set.capability_id,
                record.capability_id
            ));
        }
        case_count += eval_set.cases.len();
        eval_gate
            .register_eval_set(eval_set)
            .map_err(|error| format!("eval set invalid {}: {error:?}", eval_set_path.display()))?;

        let eval_run_path = args.capabilities_dir.join(&record.eval_run_path);
        let eval_run_contents = fs::read_to_string(&eval_run_path)
            .map_err(|error| format!("eval run unreadable {}: {error}", eval_run_path.display()))?;
        let eval_run = parse_eval_run_input(&eval_run_path, &eval_run_contents)?;
        if eval_run.capability_id != record.capability_id {
            return Err(format!(
                "{}: eval run capability_id {} does not match capability record {}",
                eval_run_path.display(),
                eval_run.capability_id,
                record.capability_id
            ));
        }
        eval_gate
            .record_run(eval_run)
            .map_err(|error| format!("eval run invalid {}: {error:?}", eval_run_path.display()))?;
        eval_gate
            .assert_publish_ready(&record.capability_id)
            .map_err(|error| {
                format!(
                    "capability not publish-ready {}: {error:?}",
                    record.capability_id
                )
            })?;
        run_count += 1;
    }

    Ok((records.len(), case_count, run_count))
}

fn read_foundry_capability_eval_records(
    capabilities_dir: &Path,
) -> Result<Vec<FoundryCapabilityEvalRecord>, String> {
    let mut records = Vec::new();
    for entry in fs::read_dir(capabilities_dir)
        .map_err(|error| format!("capabilities directory unreadable: {error}"))?
    {
        let entry = entry.map_err(|error| format!("capability entry unreadable: {error}"))?;
        let path = entry.path();
        if path.is_dir()
            || path.extension().and_then(|extension| extension.to_str()) != Some("yaml")
        {
            continue;
        }
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("capability record unreadable {}: {error}", path.display()))?;
        records.push(parse_foundry_capability_eval_record(&path, &contents)?);
    }
    records.sort_by(|left, right| left.capability_id.cmp(&right.capability_id));
    if records.is_empty() {
        Err(format!(
            "capabilities directory contains no root capability .yaml records: {}",
            capabilities_dir.display()
        ))
    } else {
        Ok(records)
    }
}

fn parse_foundry_capability_eval_record(
    path: &Path,
    contents: &str,
) -> Result<FoundryCapabilityEvalRecord, String> {
    let capability_id = required_scalar(path, contents, "id")?;
    let status = required_scalar(path, contents, "status")?;
    if status != "published" {
        return Err(format!(
            "{}: capability status must be published for eval validation",
            path.display()
        ));
    }
    Ok(FoundryCapabilityEvalRecord {
        capability_id,
        eval_set_path: PathBuf::from(required_scalar(path, contents, "eval_set")?),
        eval_run_path: PathBuf::from(required_scalar(path, contents, "eval_run")?),
    })
}

fn parse_eval_set_input(path: &Path, contents: &str) -> Result<EvalSetInput, String> {
    let mut top_level = BTreeMap::new();
    let mut cases = Vec::new();
    let mut current_case: Option<BTreeMap<String, String>> = None;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed == "cases:" {
            continue;
        }
        if let Some(after_marker) = trimmed.strip_prefix("- ") {
            if let Some(case) = current_case.take() {
                cases.push(eval_case_from_fields(path, case)?);
            }
            let mut fields = BTreeMap::new();
            if !after_marker.trim().is_empty() {
                insert_scalar_field(path, &mut fields, after_marker)?;
            }
            current_case = Some(fields);
            continue;
        }
        if let Some(case) = current_case.as_mut() {
            insert_scalar_field(path, case, trimmed)?;
        } else {
            insert_scalar_field(path, &mut top_level, trimmed)?;
        }
    }
    if let Some(case) = current_case.take() {
        cases.push(eval_case_from_fields(path, case)?);
    }

    Ok(EvalSetInput {
        capability_id: required_field(path, &top_level, "capability_id")?,
        version: required_field(path, &top_level, "version")?,
        metric: parse_eval_metric(path, &required_field(path, &top_level, "metric")?)?,
        min_pass_rate_percent: parse_u8_percent(
            path,
            "min_pass_rate_percent",
            &required_field(path, &top_level, "min_pass_rate_percent")?,
        )?,
        min_p95_score_percent: parse_u8_percent(
            path,
            "min_p95_score_percent",
            &required_field(path, &top_level, "min_p95_score_percent")?,
        )?,
        signed: parse_bool_field(path, "signed", &required_field(path, &top_level, "signed")?)?,
        cases,
    })
}

fn parse_eval_run_input(path: &Path, contents: &str) -> Result<EvalRunInput, String> {
    let mut fields = BTreeMap::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        insert_scalar_field(path, &mut fields, trimmed)?;
    }
    Ok(EvalRunInput {
        capability_id: required_field(path, &fields, "capability_id")?,
        eval_set_version: required_field(path, &fields, "eval_set_version")?,
        pass_rate_percent: parse_u8_percent(
            path,
            "pass_rate_percent",
            &required_field(path, &fields, "pass_rate_percent")?,
        )?,
        p95_score_percent: parse_u8_percent(
            path,
            "p95_score_percent",
            &required_field(path, &fields, "p95_score_percent")?,
        )?,
        adversarial_passed: parse_bool_field(
            path,
            "adversarial_passed",
            &required_field(path, &fields, "adversarial_passed")?,
        )?,
        linguistic_passed: parse_bool_field(
            path,
            "linguistic_passed",
            &required_field(path, &fields, "linguistic_passed")?,
        )?,
        signed: parse_bool_field(path, "signed", &required_field(path, &fields, "signed")?)?,
    })
}

fn eval_case_from_fields(
    path: &Path,
    fields: BTreeMap<String, String>,
) -> Result<EvalCaseInput, String> {
    let adversarial_kind = match fields.get("adversarial_kind").map(String::as_str) {
        Some("") | None => None,
        Some(value) => Some(parse_adversarial_kind(path, value)?),
    };
    let deterministic_seed = match fields.get("deterministic_seed").map(String::as_str) {
        Some("") | None => None,
        Some(value) => Some(
            value
                .parse::<u64>()
                .map_err(|_| format!("{}: deterministic_seed must be u64", path.display()))?,
        ),
    };
    Ok(EvalCaseInput {
        case_id: required_field(path, &fields, "case_id")?,
        locale: required_field(path, &fields, "locale")?,
        input_ref: required_field(path, &fields, "input_ref")?,
        expected_ref: required_field(path, &fields, "expected_ref")?,
        adversarial_kind,
        deterministic_seed,
    })
}

fn parse_eval_metric(path: &Path, value: &str) -> Result<EvalMetric, String> {
    match value {
        "ExactMatch" => Ok(EvalMetric::ExactMatch),
        "F1" => Ok(EvalMetric::F1),
        "Bleu" => Ok(EvalMetric::Bleu),
        "Rouge" => Ok(EvalMetric::Rouge),
        "HumanJudged" => Ok(EvalMetric::HumanJudged),
        "Composite" => Ok(EvalMetric::Composite),
        _ => Err(format!("{}: unknown eval metric {value}", path.display())),
    }
}

fn parse_adversarial_kind(path: &Path, value: &str) -> Result<AdversarialKind, String> {
    match value {
        "PromptInjection" => Ok(AdversarialKind::PromptInjection),
        "DataClassViolation" => Ok(AdversarialKind::DataClassViolation),
        "AutonomyBypass" => Ok(AdversarialKind::AutonomyBypass),
        "ToolExfiltration" => Ok(AdversarialKind::ToolExfiltration),
        _ => Err(format!(
            "{}: unknown adversarial kind {value}",
            path.display()
        )),
    }
}
