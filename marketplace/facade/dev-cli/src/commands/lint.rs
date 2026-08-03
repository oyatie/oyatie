//! `oya lint` — Rust-owned compatibility targets for retired Node validators.
//!
//! These commands replace the logic previously embedded in small `.mjs`
//! validators. Retired root-level Node shims are intentionally not preserved.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const REQUIRED_ACCOUNT_CRATES: &[&str] = &[
    "oya-intelligence-account-kernel",
    "intelligence-account-domain",
    "oya-intelligence-account-app",
    "oya-intelligence-account-adapter-codex-cli",
    "oya-intelligence-account-adapter-claude-code",
    "oya-intelligence-account-adapter-gemini-cli",
    "oya-intelligence-account-adapter-openbao",
    "oya-intelligence-account-runtime",
];

const REQUIRED_P03_KERNEL_CRATES: &[&str] = &[
    "oya-governance-claim-ceiling-kernel",
    "oya-governance-bypass-kernel",
    "oya-governance-pr-traceability-kernel",
    "oya-governance-pre-push-kernel",
    "oya-governance-quality-lane-kernel",
    "oya-governance-cohesion-kernel",
    "oya-intelligence-bypass-ledger-kernel",
];

const REQUIRED_PHASE00_IPS: &[&str] = &[
    ".omc/plans/milestones/M02-foundry-preview/phases/P03-gates-validators-evidence/IP-001-phase00-evidence-validator.md",
    ".omc/plans/milestones/M02-foundry-preview/phases/P03-gates-validators-evidence/IP-002-foundry-fitness-lane-ratchet.md",
    ".omc/plans/milestones/M02-foundry-preview/phases/P03-gates-validators-evidence/IP-003-adr-template-bypass-ledger.md",
];

const ADR_STATUSES: &[&str] = &[
    "Proposed",
    "Accepted",
    "Superseded",
    "Deprecated",
    "Retracted",
];
const ADR_SECTIONS: &[&str] = &["Context", "Decision", "Consequences"];

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let mut iter = args.into_iter();
    match iter.next().as_deref() {
        Some("proto") => run_one_path(
            iter.collect(),
            usage,
            "oya lint proto <proto-file-or-directory>",
            lint_proto,
        ),
        Some("asyncapi") => run_one_path(
            iter.collect(),
            usage,
            "oya lint asyncapi <asyncapi-yaml>",
            lint_asyncapi,
        ),
        Some("adr-shape") => run_one_path(
            iter.collect(),
            usage,
            "oya lint adr-shape <docs/decisions/ADR-NNNN-slug.md>",
            lint_adr_shape,
        ),
        Some("foundry-phase00-evidence") => {
            let rest = iter.collect::<Vec<_>>();
            if rest.len() > 1 {
                eprintln!("{usage}");
                return ExitCode::from(2);
            }
            let root = rest
                .first()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."));
            match lint_foundry_phase00_evidence(&root) {
                Ok(()) => {
                    println!("Phase 00 evidence validator: OK");
                    ExitCode::SUCCESS
                }
                Err(errors) => {
                    eprintln!("Phase 00 evidence validator: FAIL");
                    for error in errors {
                        eprintln!("  - {error}");
                    }
                    ExitCode::FAILURE
                }
            }
        }
        _ => {
            eprintln!("{usage}");
            ExitCode::from(2)
        }
    }
}

fn run_one_path(
    args: Vec<String>,
    usage: &str,
    command_usage: &str,
    lint: fn(&Path) -> Result<String, String>,
) -> ExitCode {
    if args.len() != 1 {
        eprintln!("usage: {command_usage}");
        return ExitCode::from(2);
    }
    match lint(Path::new(&args[0])) {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(message) if message == "USAGE" => {
            eprintln!("{usage}");
            ExitCode::from(2)
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn lint_proto(input_path: &Path) -> Result<String, String> {
    let proto_files = collect_proto_files(input_path)?;
    if proto_files.is_empty() {
        return Err(format!(
            "proto-lint: no .proto files found under {}",
            input_path.display()
        ));
    }
    let audit_proto = proto_files
        .iter()
        .find(|path| path.ends_with("audit-event-v1.proto"))
        .ok_or_else(|| "proto-lint: audit-event-v1.proto not found".to_string())?;
    let source = read_to_string(audit_proto)?;

    if !has_syntax_proto3(&source) {
        return Err("proto-lint: syntax must be proto3".to_string());
    }
    if !has_package(&source, "platform.audit.v1") {
        return Err("proto-lint: package must be platform.audit.v1".to_string());
    }
    if find_message_body(&source, "AuditEventEd25519Signature").is_err() {
        return Err("proto-lint: AuditEventEd25519Signature message missing".to_string());
    }
    if find_message_body(&source, "AuditEvent").is_err() {
        return Err("proto-lint: AuditEvent message missing".to_string());
    }
    if find_message_body(&source, "AuditEventEmit").is_ok() {
        return Err(
            "proto-lint: legacy AuditEventEmit message must not remain as a second source schema"
                .to_string(),
        );
    }

    let signature_fields = parse_fields(&find_message_body(&source, "AuditEventEd25519Signature")?);
    require_field(
        "proto-lint",
        &signature_fields,
        "key_id",
        "string",
        1,
        false,
    )?;
    require_field(
        "proto-lint",
        &signature_fields,
        "public_key_hex",
        "string",
        2,
        false,
    )?;
    require_field(
        "proto-lint",
        &signature_fields,
        "signature_hex",
        "string",
        3,
        false,
    )?;

    let audit_fields = parse_fields(&find_message_body(&source, "AuditEvent")?);
    for (name, field_type, number, repeated) in [
        ("id", "string", 1, false),
        ("tenant_id", "string", 2, false),
        ("surface", "string", 3, false),
        ("plane", "string", 4, false),
        ("purpose", "string", 5, false),
        ("data_classes_touched", "string", 6, true),
        ("decision", "string", 7, false),
        ("idempotency_key", "string", 8, false),
        ("emitted_at_epoch_seconds", "uint64", 9, false),
        ("tenant_shard", "string", 10, false),
        ("sequence", "uint64", 11, false),
        ("previous_hash", "string", 12, false),
        ("hash", "string", 13, false),
        ("merkle_root", "string", 14, false),
        ("ed25519_signature", "AuditEventEd25519Signature", 15, false),
    ] {
        require_field(
            "proto-lint",
            &audit_fields,
            name,
            field_type,
            number,
            repeated,
        )?;
    }
    reject_duplicate_tags(
        "proto-lint",
        "AuditEventEd25519Signature",
        &signature_fields,
    )?;
    reject_duplicate_tags("proto-lint", "AuditEvent", &audit_fields)?;

    Ok(format!("proto-lint: ok {}", audit_proto.display()))
}

fn lint_asyncapi(contract_path: &Path) -> Result<String, String> {
    if !contract_path.exists() {
        return Err(format!(
            "asyncapi-lint: contract does not exist: {}",
            contract_path.display()
        ));
    }
    let source = read_to_string(contract_path)?;
    for (needle, label) in [
        ("asyncapi: 3.1.0", "asyncapi: 3.1.0"),
        (
            "defaultContentType: application/cloudevents+protobuf",
            "defaultContentType: application/cloudevents+protobuf",
        ),
        (
            "contentType: application/cloudevents+protobuf",
            "contentType: application/cloudevents+protobuf",
        ),
        (
            "schemaFormat: application/vnd.google.protobuf;version=3",
            "schemaFormat: application/vnd.google.protobuf;version=3",
        ),
    ] {
        require_includes("asyncapi-lint", &source, needle, label)?;
    }

    let proto_refs = find_proto_refs(&source);
    if proto_refs.is_empty() {
        return Err("asyncapi-lint: missing local protobuf payload $ref".to_string());
    }
    for proto_ref in &proto_refs {
        validate_local_proto_ref(contract_path, proto_ref)?;
    }

    if is_platform_audit_asyncapi(contract_path, &source) {
        lint_platform_audit_asyncapi(&source, &proto_refs)?;
    }

    Ok(format!("asyncapi-lint: ok {}", contract_path.display()))
}

fn lint_platform_audit_asyncapi(source: &str, proto_refs: &[String]) -> Result<(), String> {
    for (needle, label) in [
        ("address: oya.platform.audit", "address: oya.platform.audit"),
        ("action: send", "action: send"),
        ("name: audit.event.emit.v1", "name: audit.event.emit.v1"),
        ("additionalProperties: false", "additionalProperties: false"),
        (
            "required: [specversion, id, source, type, subject, time, datacontenttype]",
            "required: [specversion, id, source, type, subject, time, datacontenttype]",
        ),
        ("const: '1.0'", "CloudEvents specversion 1.0 header"),
        (
            "const: oyatie://platform/audit-chain",
            "const: oyatie://platform/audit-chain",
        ),
        ("const: audit.event.emit.v1", "const: audit.event.emit.v1"),
        ("const: application/protobuf", "const: application/protobuf"),
    ] {
        require_includes("asyncapi-lint", source, needle, label)?;
    }
    for header in [
        "specversion",
        "id",
        "source",
        "type",
        "subject",
        "time",
        "datacontenttype",
    ] {
        require_includes(
            "asyncapi-lint",
            source,
            &format!("          {header}:"),
            &format!("CloudEvents header property {header}"),
        )?;
    }

    let payload_ref = proto_refs
        .iter()
        .find(|proto_ref| proto_ref.contains("audit-event-v1.proto#"))
        .ok_or_else(|| "asyncapi-lint: missing audit-event-v1.proto payload $ref".to_string())?;
    let (_, message_ref) = payload_ref
        .split_once('#')
        .ok_or_else(|| "asyncapi-lint: missing audit-event-v1.proto payload $ref".to_string())?;
    if message_ref != "/platform.audit.v1.AuditEvent" {
        return Err(format!(
            "asyncapi-lint: payload $ref must target /platform.audit.v1.AuditEvent, got {message_ref}"
        ));
    }

    Ok(())
}

fn lint_adr_shape(file: &Path) -> Result<String, String> {
    let text = read_to_string(file)?;
    let base = file
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if !is_adr_file_name(base) {
        return Err(format!(
            "{file}: expected ADR-NNNN-slug.md",
            file = file.display()
        ));
    }
    if !text.lines().any(|line| line.starts_with("# ADR-")) {
        return Err(format!("{file}: missing ADR title", file = file.display()));
    }
    let status = read_adr_status(&text);
    let Some(valid_status) = status
        .as_ref()
        .filter(|value| ADR_STATUSES.contains(&value.as_str()))
    else {
        return Err(format!(
            "{file}: invalid or missing status ({status})",
            file = file.display(),
            status = status.unwrap_or_else(|| "missing".to_string())
        ));
    };
    let found = adr_headings(&text);
    for section in ADR_SECTIONS {
        if !found
            .iter()
            .any(|heading| heading == &section.to_lowercase())
        {
            return Err(format!(
                "{file}: missing required section ## {section}",
                file = file.display()
            ));
        }
    }
    let positions = ADR_SECTIONS
        .iter()
        .map(|section| {
            found
                .iter()
                .position(|heading| heading == &section.to_lowercase())
                .unwrap_or(usize::MAX)
        })
        .collect::<Vec<_>>();
    for pair in positions.windows(2) {
        if pair[1] < pair[0] {
            return Err(format!(
                "{file}: required sections out of order ({})",
                ADR_SECTIONS.join(" -> "),
                file = file.display()
            ));
        }
    }
    Ok(format!(
        "adr-shape ok: file={} status={} sections={}",
        file.display(),
        valid_status,
        ADR_SECTIONS.len()
    ))
}

fn lint_foundry_phase00_evidence(root: &Path) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for crate_name in REQUIRED_ACCOUNT_CRATES {
        if check_crate(root, crate_name, &mut errors) {
            scan_for_secrets(
                &root.join("crates").join(crate_name).join("src"),
                &mut errors,
            );
        }
    }
    for crate_name in REQUIRED_P03_KERNEL_CRATES {
        check_crate(root, crate_name, &mut errors);
    }
    for ip in REQUIRED_PHASE00_IPS {
        if check_file(root, ip, &mut errors) {
            status_complete(root, ip, &mut errors);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProtoField {
    repeated: bool,
    field_type: String,
    name: String,
    number: u32,
}

fn collect_proto_files(path: &Path) -> Result<Vec<PathBuf>, String> {
    if !path.exists() {
        return Err(format!(
            "proto-lint: path does not exist: {}",
            path.display()
        ));
    }
    if path.is_file() {
        return Ok(if path.extension().is_some_and(|ext| ext == "proto") {
            vec![path.to_path_buf()]
        } else {
            Vec::new()
        });
    }
    let mut files = Vec::new();
    collect_proto_files_recursive(path, &mut files)
        .map_err(|error| format!("proto-lint: could not read {}: {error}", path.display()))?;
    files.sort();
    Ok(files)
}

fn collect_proto_files_recursive(
    path: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_proto_files_recursive(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "proto") {
            files.push(path);
        }
    }
    Ok(())
}

fn read_to_string(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn has_syntax_proto3(source: &str) -> bool {
    source.lines().any(|line| {
        let compact = line.split_whitespace().collect::<String>();
        compact == "syntax=\"proto3\";"
    })
}

fn has_package(source: &str, package: &str) -> bool {
    source.lines().any(|line| {
        let compact = line.split_whitespace().collect::<String>();
        compact == format!("package{package};")
    })
}

fn find_message_body(source: &str, message_name: &str) -> Result<String, String> {
    let bytes = source.as_bytes();
    let mut cursor = 0;
    while let Some(relative) = source[cursor..].find("message") {
        let start = cursor + relative;
        let before_ok = start == 0
            || !source[..start]
                .chars()
                .next_back()
                .is_some_and(is_identifier_char);
        let mut idx = start + "message".len();
        idx = skip_whitespace(bytes, idx);
        if before_ok && source[idx..].starts_with(message_name) {
            let after_name = idx + message_name.len();
            let after_ok = source[after_name..]
                .chars()
                .next()
                .is_none_or(|character| !is_identifier_char(character));
            if after_ok {
                let brace = source[after_name..]
                    .find('{')
                    .map(|offset| after_name + offset)
                    .ok_or_else(|| format!("{message_name} message missing"))?;
                let end = matching_brace(source, brace)
                    .ok_or_else(|| format!("{message_name} message has unbalanced braces"))?;
                return Ok(source[brace + 1..end].to_string());
            }
        }
        cursor = start + "message".len();
    }
    Err(format!("{message_name} message missing"))
}

fn skip_whitespace(bytes: &[u8], mut idx: usize) -> usize {
    while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
        idx += 1;
    }
    idx
}

fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0_u32;
    for (idx, character) in source[open..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(open + idx);
                }
            }
            _ => {}
        }
    }
    None
}

fn is_identifier_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

fn parse_fields(source: &str) -> Vec<ProtoField> {
    let mut fields = Vec::new();
    for line in source.lines() {
        let Some((left, right)) = line.trim().trim_end_matches(';').split_once('=') else {
            continue;
        };
        let number = right.trim().parse::<u32>().ok();
        let tokens = left.split_whitespace().collect::<Vec<_>>();
        let (repeated, field_type, name) = match tokens.as_slice() {
            ["repeated", field_type, name] => (true, *field_type, *name),
            [field_type, name] => (false, *field_type, *name),
            _ => continue,
        };
        if let Some(number) = number {
            fields.push(ProtoField {
                repeated,
                field_type: field_type.to_string(),
                name: name.to_string(),
                number,
            });
        }
    }
    fields
}

fn require_field(
    prefix: &str,
    fields: &[ProtoField],
    name: &str,
    field_type: &str,
    number: u32,
    repeated: bool,
) -> Result<(), String> {
    let Some(field) = fields.iter().find(|field| field.name == name) else {
        return Err(format!("{prefix}: missing field {name}"));
    };
    if field.field_type != field_type || field.number != number || field.repeated != repeated {
        let actual = format!(
            "{}{} {} = {}",
            if field.repeated { "repeated " } else { "" },
            field.field_type,
            name,
            field.number
        );
        let expected = format!(
            "{}{} {} = {}",
            if repeated { "repeated " } else { "" },
            field_type,
            name,
            number
        );
        return Err(format!(
            "{prefix}: field mismatch for {name}: expected {expected}, got {actual}"
        ));
    }
    Ok(())
}

fn reject_duplicate_tags(
    prefix: &str,
    message_name: &str,
    fields: &[ProtoField],
) -> Result<(), String> {
    let mut seen = Vec::new();
    for field in fields {
        if seen.contains(&field.number) {
            return Err(format!(
                "{prefix}: duplicate protobuf field tags detected in {message_name}"
            ));
        }
        seen.push(field.number);
    }
    Ok(())
}

fn require_includes(prefix: &str, source: &str, needle: &str, label: &str) -> Result<(), String> {
    if source.contains(needle) {
        Ok(())
    } else {
        Err(format!("{prefix}: missing {label}"))
    }
}

fn find_proto_refs(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let (_, value) = line.split_once("$ref:")?;
            let cleaned = yaml_inline_scalar(value);
            if cleaned.contains(".proto#") {
                Some(cleaned)
            } else {
                None
            }
        })
        .collect()
}

fn yaml_inline_scalar(value: &str) -> String {
    let trimmed = value.trim();
    if let Some(quote) = trimmed.as_bytes().first().copied()
        && (quote == b'\'' || quote == b'"')
        && let Some(end) = trimmed[1..].find(quote as char)
    {
        return trimmed[1..1 + end].to_string();
    }
    trimmed
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string()
}

fn validate_local_proto_ref(contract_path: &Path, proto_ref: &str) -> Result<(), String> {
    let (proto_rel_path, message_ref) = proto_ref.split_once('#').ok_or_else(|| {
        format!("asyncapi-lint: protobuf payload $ref missing fragment: {proto_ref}")
    })?;
    if proto_rel_path.is_empty()
        || proto_rel_path.starts_with('/')
        || proto_rel_path.contains('\\')
        || proto_rel_path.contains('\0')
        || proto_rel_path.starts_with("http://")
        || proto_rel_path.starts_with("https://")
    {
        return Err(format!(
            "asyncapi-lint: protobuf payload $ref must be a local relative path: {proto_ref}"
        ));
    }
    let proto_path = contract_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(proto_rel_path);
    if !proto_path.exists() {
        return Err(format!(
            "asyncapi-lint: payload $ref target does not exist: {}",
            proto_path.display()
        ));
    }
    let proto = read_to_string(&proto_path)?;
    let message_path = message_ref.strip_prefix('/').ok_or_else(|| {
        format!("asyncapi-lint: protobuf payload $ref fragment must start with '/': {proto_ref}")
    })?;
    let (package, message) = message_path.rsplit_once('.').ok_or_else(|| {
        format!("asyncapi-lint: protobuf payload $ref fragment must include package and message: {proto_ref}")
    })?;
    if !has_package(&proto, package) {
        return Err(format!(
            "asyncapi-lint: payload proto package {package} not found"
        ));
    }
    if find_message_body(&proto, message).is_err() {
        return Err(format!(
            "asyncapi-lint: payload proto message {message_path} not found"
        ));
    }
    Ok(())
}

fn is_platform_audit_asyncapi(contract_path: &Path, source: &str) -> bool {
    contract_path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "audit-events-v1.yaml")
        || source.contains("audit.event.emit.v1")
}

fn is_adr_file_name(base: &str) -> bool {
    let Some(rest) = base.strip_prefix("ADR-") else {
        return false;
    };
    let Some((digits, slug)) = rest.split_once('-') else {
        return false;
    };
    digits.len() == 4
        && digits.chars().all(|character| character.is_ascii_digit())
        && slug.ends_with(".md")
        && slug.len() > ".md".len()
}

fn read_adr_status(text: &str) -> Option<String> {
    let lines = text.lines().collect::<Vec<_>>();
    for prefix in ["> **Status:**", "- > **Status:**"] {
        if let Some(status) = lines.iter().find_map(|line| {
            let trimmed = line.trim();
            trimmed.strip_prefix(prefix).and_then(clean_status)
        }) {
            return Some(status);
        }
    }
    for prefix in ["**Status:**", "- **Status:**"] {
        if let Some(status) = lines.iter().find_map(|line| {
            let trimmed = line.trim();
            trimmed.strip_prefix(prefix).and_then(clean_status)
        }) {
            return Some(status);
        }
    }
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.to_ascii_lowercase().starts_with("status:") {
            return clean_status(&trimmed["status:".len()..]);
        }
        if trimmed.eq_ignore_ascii_case("## Status")
            && let Some(next) = lines.get(idx + 1)
        {
            return clean_status(next);
        }
    }
    None
}

fn clean_status(value: &str) -> Option<String> {
    let cleaned = value
        .replace(['`', '*', '.'], "")
        .trim()
        .split(|character: char| {
            character.is_whitespace() || character == '-' || character == '—' || character == '('
        })
        .next()
        .unwrap_or_default()
        .to_string();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn adr_headings(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| line.strip_prefix("## "))
        .map(|line| line.trim().to_ascii_lowercase())
        .collect()
}

fn check_crate(root: &Path, crate_name: &str, errors: &mut Vec<String>) -> bool {
    let cargo = root.join("crates").join(crate_name).join("Cargo.toml");
    if cargo.exists() {
        true
    } else {
        errors.push(format!("missing crate: {crate_name}"));
        false
    }
}

fn check_file(root: &Path, rel: &str, errors: &mut Vec<String>) -> bool {
    let path = root.join(rel);
    if path.exists() {
        true
    } else {
        errors.push(format!("missing file: {rel}"));
        false
    }
}

fn status_complete(root: &Path, rel: &str, errors: &mut Vec<String>) {
    match fs::read_to_string(root.join(rel)) {
        Ok(text)
            if text
                .lines()
                .any(|line| line.trim().starts_with("status: complete")) => {}
        Ok(_) => errors.push(format!("IP not marked complete: {rel}")),
        Err(error) => errors.push(format!("could not read {rel}: {error}")),
    }
}

fn scan_for_secrets(src_dir: &Path, errors: &mut Vec<String>) {
    if !src_dir.exists() {
        return;
    }
    let mut stack = vec![src_dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let Ok(text) = fs::read_to_string(&path) else {
                    continue;
                };
                if let Some(pattern) = secret_pattern_hit(&text) {
                    errors.push(format!(
                        "raw secret heuristic match in {}: {pattern}",
                        path.display()
                    ));
                }
            }
        }
    }
}

fn secret_pattern_hit(text: &str) -> Option<&'static str> {
    if contains_akia_key(text) {
        return Some("AKIA[0-9A-Z]{16}");
    }
    if text.contains("-----BEGIN ") && text.contains("PRIVATE KEY-----") {
        return Some("-----BEGIN PRIVATE KEY-----");
    }
    if contains_prefixed_token(text, &["xoxa-", "xoxb-", "xoxr-", "xoxs-"], 10) {
        return Some("xox[abrs]-[0-9A-Za-z-]{10,}");
    }
    if contains_prefixed_alnum(text, "ghp_", 36) {
        return Some("ghp_[0-9A-Za-z]{36,}");
    }
    if contains_prefixed_alnum(text, "sk-", 32) {
        return Some("sk-[A-Za-z0-9]{32,}");
    }
    None
}

fn contains_akia_key(text: &str) -> bool {
    let bytes = text.as_bytes();
    for idx in 0..bytes.len().saturating_sub(20) {
        if &bytes[idx..idx + 4] == b"AKIA"
            && bytes[idx + 4..idx + 20]
                .iter()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
        {
            return true;
        }
    }
    false
}

fn contains_prefixed_token(text: &str, prefixes: &[&str], min_suffix_len: usize) -> bool {
    prefixes
        .iter()
        .any(|prefix| contains_prefixed_token_chars(text, prefix, min_suffix_len))
}

fn contains_prefixed_token_chars(text: &str, prefix: &str, min_suffix_len: usize) -> bool {
    let mut cursor = 0;
    while let Some(offset) = text[cursor..].find(prefix) {
        let start = cursor + offset + prefix.len();
        let len = text[start..]
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric() || *character == '-')
            .count();
        if len >= min_suffix_len {
            return true;
        }
        cursor = start;
    }
    false
}

fn contains_prefixed_alnum(text: &str, prefix: &str, min_suffix_len: usize) -> bool {
    let mut cursor = 0;
    while let Some(offset) = text[cursor..].find(prefix) {
        let start = cursor + offset + prefix.len();
        let len = text[start..]
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric())
            .count();
        if len >= min_suffix_len {
            return true;
        }
        cursor = start;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adr_status_parses_status_variants() {
        assert_eq!(
            read_adr_status("> **Status:** Accepted — active").as_deref(),
            Some("Accepted")
        );
        assert_eq!(
            read_adr_status("status: Proposed").as_deref(),
            Some("Proposed")
        );
        assert_eq!(
            read_adr_status("## Status\nDeprecated").as_deref(),
            Some("Deprecated")
        );
    }

    #[test]
    fn proto_body_and_fields_are_parsed() {
        let body = find_message_body(
            r#"message AuditEvent {
              string id = 1;
              repeated string data_classes_touched = 6;
            }"#,
            "AuditEvent",
        )
        .expect("body parsed");
        let fields = parse_fields(&body);
        require_field("proto-lint", &fields, "id", "string", 1, false).expect("id field");
        require_field(
            "proto-lint",
            &fields,
            "data_classes_touched",
            "string",
            6,
            true,
        )
        .expect("repeated field");
    }

    #[test]
    fn secret_heuristics_detect_known_prefixes() {
        assert_eq!(
            secret_pattern_hit("token = \"ghp_abcdefghijklmnopqrstuvwxyzABCDEFGHIJ\""),
            Some("ghp_[0-9A-Za-z]{36,}")
        );
        assert_eq!(
            secret_pattern_hit("key = \"AKIAABCDEFGHIJKLMNOP\""),
            Some("AKIA[0-9A-Z]{16}")
        );
    }
}
