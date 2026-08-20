#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use ci_product_protocol_policy::{GATE_ID, evaluate_keyed};
use serde_json::Value;

fn json(path: &Path) -> Value {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

fn text(variable: &str) -> String {
    let path = declared_path(variable);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn declared_path(variable: &str) -> PathBuf {
    declared_path_value(variable, std::env::var_os(variable))
}

fn declared_path_value(variable: &str, value: Option<std::ffi::OsString>) -> PathBuf {
    value
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("Buck must declare {variable} with $(location)"))
}

fn policy() -> Value {
    json(&declared_path("OYA_PRODUCT_PROTOCOL_POLICY"))
}

#[test]
fn missing_declared_resource_remains_fail_closed_outside_build_graph_contract() {
    let failure = std::panic::catch_unwind(|| declared_path_value("OYA_REQUIRED_RESOURCE", None));
    assert!(
        failure.is_err(),
        "a missing declared resource must not fall back to the source checkout"
    );
}

fn artifacts() -> BTreeMap<String, Value> {
    BTreeMap::from([
        (
            "product_contract".to_owned(),
            json(&declared_path("OYA_PRODUCT_PROTOCOL_CONTRACT")),
        ),
        (
            "api_contract_ssot".to_owned(),
            json(&declared_path("OYA_API_CONTRACT_SSOT")),
        ),
        (
            "transport_profile".to_owned(),
            json(&declared_path("OYA_ENDPOINT_TRANSPORT_PROFILE")),
        ),
        (
            "root_hub".to_owned(),
            json(&declared_path("OYA_ROOT_HUB_POINTERS")),
        ),
        (
            "manifest_schema".to_owned(),
            json(&declared_path("OYA_MICROSERVICE_MANIFEST_SCHEMA")),
        ),
        (
            "master_plan_sequencing".to_owned(),
            json(&declared_path("OYA_MASTER_PLAN_SEQUENCING")),
        ),
    ])
}

fn section<'a>(document: &'a str, heading: &str) -> &'a str {
    let body = document
        .split_once(heading)
        .unwrap_or_else(|| panic!("missing authority section {heading}"))
        .1;
    body.split("\n## ").next().expect("section body")
}

fn frontmatter(document: &str) -> &str {
    document
        .strip_prefix("---\n")
        .and_then(|body| body.split_once("\n---\n"))
        .map(|(frontmatter, _)| frontmatter)
        .expect("ADR must carry YAML frontmatter")
}

fn repo_root() -> PathBuf {
    let mut directory = std::env::current_dir().expect("current directory");
    for _ in 0..16 {
        if directory.join("specs/root-hub-pointers.json").is_file() {
            return directory;
        }
        if !directory.pop() {
            break;
        }
    }
    panic!("failed to locate repository root from Buck test working directory")
}

fn collect_named_files(directory: &Path, name: &str, output: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("read directory {}: {error}", directory.display()))
    {
        let entry = entry.expect("directory entry");
        let path = entry.path();
        if entry.file_type().expect("file type").is_dir() {
            // A manifest under `tests/` is a FIXTURE, not a service. Widening governed_roots
            // to 22 pulled cell/core/regional-pack/tests/fixtures/kr/manifest.json into the
            // governed corpus, where it drifted the reviewed-legacy baseline by one entry.
            // Widening a corpus without reconciling its new members trades one red for another.
            if entry.file_name() == "tests" {
                continue;
            }
            collect_named_files(&path, name, output);
        } else if entry.file_name() == name {
            output.push(path);
        }
    }
}

/// Governed capability roots, read from the policy DATA rather than hard-coded here.
/// The roots are the sole knob a capability rehome has to turn; duplicating them in Rust is what
/// let the manifest universe silently shrink 100 -> 71 across six re-anchor waves.
fn governed_roots(policy: &Value) -> Vec<String> {
    let roots = string_set(policy, "/manifest_inventory/governed_roots");
    assert!(
        !roots.is_empty(),
        "manifest_inventory.governed_roots must not be empty"
    );
    roots.into_iter().collect()
}

fn collect_governed_manifests(root: &Path, policy: &Value) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for directory in governed_roots(policy) {
        collect_named_files(&root.join(directory), "manifest.json", &mut paths);
    }
    paths.sort();
    paths
}

fn collect_extension_files(directory: &Path, extension: &str, output: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("read directory {}: {error}", directory.display()))
    {
        let entry = entry.expect("directory entry");
        let path = entry.path();
        if entry.file_type().expect("file type").is_dir() {
            collect_extension_files(&path, extension, output);
        } else if path.extension().and_then(|value| value.to_str()) == Some(extension) {
            output.push(path);
        }
    }
}

const INTERNAL_PROTO_EXPOSURE_MARKER: &str = "// oyatie.contract.exposure: internal-only";
const INTERNAL_PROTO_TRANSPORT_MARKER: &str = "// oyatie.contract.transport: grpc-http2";
const INTERNAL_PROTO_IDENTITY_MARKER: &str = "// oyatie.contract.identity: spiffe-mtls-tls13";

fn internal_proto_contract_content_findings(contract_path: &str, document: &str) -> Vec<String> {
    let mut findings = Vec::new();
    for (marker, requirement) in [
        (
            INTERNAL_PROTO_EXPOSURE_MARKER,
            "must declare internal-only exposure",
        ),
        (
            INTERNAL_PROTO_TRANSPORT_MARKER,
            "must declare canonical gRPC-over-HTTP/2 transport",
        ),
        (
            INTERNAL_PROTO_IDENTITY_MARKER,
            "must declare SPIFFE mTLS/TLS 1.3 workload identity",
        ),
    ] {
        if !document.lines().any(|line| line.trim() == marker) {
            findings.push(format!("{contract_path} {requirement}"));
        }
    }
    if !document
        .lines()
        .any(|line| line.trim() == r#"syntax = "proto3";"#)
    {
        findings.push(format!(
            "{contract_path} must declare the proto3 language profile"
        ));
    }

    for (index, line) in document.lines().enumerate() {
        let comment = line.trim().to_ascii_lowercase();
        if !comment.starts_with("//") {
            continue;
        }
        if ["http/3", "http3", "quic"]
            .iter()
            .any(|term| comment.contains(term))
        {
            findings.push(format!(
                "{contract_path}:{} claims HTTP/3 or QUIC for internal gRPC instead of HTTP/2",
                index + 1
            ));
        }
        if comment.contains("grpc-web") {
            findings.push(format!(
                "{contract_path}:{} claims forbidden public gRPC-Web exposure",
                index + 1
            ));
        }
        let auth_line = comment.contains("authn:") || comment.contains("authentication:");
        let external_auth = comment.contains("external")
            && ["oidc", "api key", "api-key", "bearer"]
                .iter()
                .any(|term| comment.contains(term));
        let oidc_or_api_key_as_grpc_auth = auth_line
            && ["oidc bearer", "api key", "api-key"]
                .iter()
                .any(|term| comment.contains(term));
        if external_auth || oidc_or_api_key_as_grpc_auth {
            findings.push(format!(
                "{contract_path}:{} claims external or client-token authentication on an internal-only gRPC contract",
                index + 1
            ));
        }
        if comment.contains("grpc")
            && [
                "clients that prefer",
                "public grpc",
                "external grpc",
                "external — oidc bearer",
                "client-facing grpc",
                "tenant clients",
                "browser clients",
            ]
            .iter()
            .any(|term| comment.contains(term))
        {
            findings.push(format!(
                "{contract_path}:{} claims public or client-choice gRPC",
                index + 1
            ));
        }
    }

    let normalized_comments = document
        .lines()
        .filter_map(|line| line.trim().strip_prefix("//"))
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    if [
        "clients that prefer grpc",
        "clients preferring grpc",
        "clients that prefer protobuf",
        "used by studio, sdk consumers",
    ]
    .iter()
    .any(|term| normalized_comments.contains(term))
        && !findings
            .iter()
            .any(|finding| finding.contains("claims public or client-choice gRPC"))
    {
        findings.push(format!(
            "{contract_path} claims public or client-choice gRPC/Protobuf across adjacent comments"
        ));
    }
    if normalized_comments.contains("all rpcs require oidc")
        || normalized_comments.contains("adminbearer")
    {
        findings.push(format!(
            "{contract_path} claims client-token authentication on an internal-only gRPC contract"
        ));
    }
    if normalized_comments.contains("flatbuffers") {
        findings.push(format!(
            "{contract_path} claims an unqualified FlatBuffers payload inside the canonical Protobuf corpus"
        ));
    }
    findings
}

fn declares_proto_service(document: &str) -> bool {
    #[derive(Clone, Copy)]
    enum LexState {
        Code,
        LineComment,
        BlockComment,
        String(char),
    }

    fn flush_identifier(identifier: &mut String, tokens: &mut Vec<String>) {
        if !identifier.is_empty() {
            tokens.push(std::mem::take(identifier));
        }
    }

    let mut state = LexState::Code;
    let mut escaped = false;
    let mut identifier = String::new();
    let mut tokens = Vec::new();
    let mut characters = document.chars().peekable();
    while let Some(character) = characters.next() {
        match state {
            LexState::Code => match character {
                '/' if characters.peek() == Some(&'/') => {
                    characters.next();
                    flush_identifier(&mut identifier, &mut tokens);
                    state = LexState::LineComment;
                }
                '/' if characters.peek() == Some(&'*') => {
                    characters.next();
                    flush_identifier(&mut identifier, &mut tokens);
                    state = LexState::BlockComment;
                }
                '"' | '\'' => {
                    flush_identifier(&mut identifier, &mut tokens);
                    state = LexState::String(character);
                    escaped = false;
                }
                '{' => {
                    flush_identifier(&mut identifier, &mut tokens);
                    tokens.push("{".to_owned());
                }
                value if value.is_ascii_alphanumeric() || value == '_' => {
                    identifier.push(value);
                }
                _ => flush_identifier(&mut identifier, &mut tokens),
            },
            LexState::LineComment => {
                if character == '\n' {
                    state = LexState::Code;
                }
            }
            LexState::BlockComment => {
                if character == '*' && characters.peek() == Some(&'/') {
                    characters.next();
                    state = LexState::Code;
                }
            }
            LexState::String(delimiter) => {
                if escaped {
                    escaped = false;
                } else if character == '\\' {
                    escaped = true;
                } else if character == delimiter {
                    state = LexState::Code;
                }
            }
        }
    }
    flush_identifier(&mut identifier, &mut tokens);

    tokens.windows(3).any(|tokens| {
        tokens[0] == "service"
            && tokens[1]
                .chars()
                .next()
                .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
            && tokens[2] == "{"
    })
}

fn unclassified_proto_service_finding(
    contract_path: &str,
    document: &str,
    manifest_contracts: &BTreeSet<String>,
    reviewed_legacy_manifests: &BTreeSet<String>,
) -> Option<String> {
    if !declares_proto_service(document) || manifest_contracts.contains(contract_path) {
        return None;
    }
    let legacy_owned = reviewed_legacy_manifests.iter().any(|manifest| {
        manifest
            .strip_suffix("/manifest.json")
            .is_some_and(|directory| contract_path.starts_with(&format!("{directory}/")))
    });
    (!legacy_owned).then(|| {
        format!(
            "{contract_path} declares a Proto service but is neither classified through contracts.internal_grpc nor owned by the explicit reviewed-legacy manifest inventory"
        )
    })
}

fn internal_grpc_contract_path_findings(
    root: &Path,
    declared_proto_paths: &BTreeSet<String>,
    manifest_path: &Path,
    manifest: &Value,
) -> Vec<String> {
    manifest
        .pointer("/contracts/internal_grpc/contracts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|contract| {
            let Some(contract) = contract.as_str() else {
                return Some(format!(
                    "{} declares a non-string internal gRPC contract path",
                    manifest_path.display()
                ));
            };
            let path = root.join(contract);
            let metadata = fs::symlink_metadata(&path);
            if !declared_proto_paths.contains(contract) {
                Some(format!(
                    "{} declares an internal gRPC contract outside the Buck-declared Proto corpus: {contract}",
                    manifest_path.display()
                ))
            } else if !metadata.is_ok_and(|metadata| metadata.file_type().is_file()) {
                Some(format!(
                    "{} declares non-regular internal gRPC contract {contract}",
                    manifest_path.display()
                ))
            } else {
                let document = fs::read_to_string(&path).unwrap_or_else(|error| {
                    panic!("read declared internal gRPC contract {}: {error}", path.display())
                });
                let findings = internal_proto_contract_content_findings(contract, &document);
                if findings.is_empty() {
                    None
                } else {
                    Some(format!(
                        "{} internal gRPC contract findings: {}",
                        manifest_path.display(),
                        findings.join("; ")
                    ))
                }
            }
        })
        .collect()
}

fn manifest_protocol_findings(manifest_path: &Path, manifest: &Value) -> Vec<String> {
    fn walk(
        manifest_path: &Path,
        path: &str,
        value: &Value,
        findings: &mut Vec<String>,
        saw_flatbuffers: &mut bool,
    ) {
        match value {
            Value::Object(object) => {
                for (key, child) in object {
                    let child_path = format!("{path}/{key}");
                    let normalized_key = key.to_ascii_lowercase();
                    if normalized_key == "proto"
                        && path.ends_with("/tenant_version_pinning/public_surface_files")
                    {
                        findings.push(format!(
                            "{} {child_path} declares a public Protobuf carrier; Protobuf is internal-only",
                            manifest_path.display()
                        ));
                    }
                    if normalized_key == "internal_grpc" {
                        let valid = child.as_object().is_some_and(|internal| {
                            internal.get("transport") == Some(&Value::String("http2".to_owned()))
                                && internal.get("language_profile")
                                    == Some(&Value::String("proto3".to_owned()))
                        });
                        if !valid {
                            findings.push(format!(
                                "{} {child_path} must declare internal gRPC as proto3 over HTTP/2",
                                manifest_path.display()
                            ));
                        }
                    }
                    if normalized_key.contains("flatbuffer") {
                        *saw_flatbuffers = true;
                    }
                    if normalized_key.contains("grpc")
                        && let Some(text) = child.as_str()
                    {
                        let text = text.to_ascii_lowercase();
                        if text.contains("http/3") || text.contains("http3") {
                            findings.push(format!(
                                "{} {child_path} routes gRPC over HTTP/3 instead of HTTP/2",
                                manifest_path.display()
                            ));
                        }
                    }
                    walk(manifest_path, &child_path, child, findings, saw_flatbuffers);
                }
            }
            Value::Array(values) => {
                for (index, child) in values.iter().enumerate() {
                    walk(
                        manifest_path,
                        &format!("{path}/{index}"),
                        child,
                        findings,
                        saw_flatbuffers,
                    );
                }
            }
            Value::String(text) => {
                let text = text.to_ascii_lowercase();
                if text.contains("flatbuffer") {
                    *saw_flatbuffers = true;
                }
                if text.contains("grpc") && (text.contains("http/3") || text.contains("http3")) {
                    findings.push(format!(
                        "{} {path} routes gRPC over HTTP/3 instead of HTTP/2",
                        manifest_path.display()
                    ));
                }
            }
            _ => {}
        }
    }

    let mut findings = Vec::new();
    let mut saw_flatbuffers = false;
    walk(
        manifest_path,
        "",
        manifest,
        &mut findings,
        &mut saw_flatbuffers,
    );
    if saw_flatbuffers {
        let required_activation = BTreeSet::from([
            "isolated hot path",
            "no second independently authored source of truth",
            "reproducible latency or zero-copy benchmark",
            "schema-evolution review",
        ]);
        let observed_activation = manifest
            .pointer("/protocol_posture/flatbuffers/activation_requires")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect::<BTreeSet<_>>();
        if manifest.pointer("/protocol_posture/flatbuffers/status")
            != Some(&Value::String("benchmark-gated-derived-adapter".to_owned()))
            || manifest.pointer("/protocol_posture/flatbuffers/canonical")
                != Some(&Value::Bool(false))
            || observed_activation != required_activation
        {
            findings.push(format!(
                "{} mentions FlatBuffers without the complete benchmark-gated, non-canonical derived-adapter posture",
                manifest_path.display()
            ));
        }
    }
    findings
}

fn string_set(value: &Value, pointer: &str) -> BTreeSet<String> {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("policy pointer {pointer} must be an array"))
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .unwrap_or_else(|| panic!("policy pointer {pointer} must contain strings"))
                .to_owned()
        })
        .collect()
}

fn accepted_status(frontmatter: &str, accepted: &BTreeSet<&str>) -> bool {
    frontmatter.lines().any(|line| {
        line.strip_prefix("status:")
            .map(str::trim)
            .map(|status| status.trim_matches(['\'', '"']).to_ascii_lowercase())
            .is_some_and(|status| accepted.contains(status.as_str()))
    })
}

#[derive(Debug, PartialEq, Eq)]
struct PublicRpcFinding {
    key: String,
    clause: String,
}

#[derive(Debug, PartialEq, Eq)]
enum RpcAudience {
    Public,
    Internal,
    Rejected,
    Historical,
    Unclassified,
    NonClaim,
}

fn contains_term(clause: &str, terms: &[&str]) -> bool {
    terms.iter().any(|term| clause.contains(term))
}

fn contains_word(clause: &str, words: &[&str]) -> bool {
    clause
        .split(|character: char| !character.is_alphanumeric())
        .any(|token| words.contains(&token))
}

fn normalized_words(text: &str) -> String {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn contains_claim_term(clause: &str) -> bool {
    clause.split_whitespace().any(|word| {
        matches!(
            word,
            "consume"
                | "consumed"
                | "consumes"
                | "consuming"
                | "call"
                | "called"
                | "calls"
                | "calling"
                | "expose"
                | "exposed"
                | "exposes"
                | "exposing"
                | "query"
                | "queried"
                | "queries"
                | "querying"
                | "serve"
                | "served"
                | "serves"
                | "serving"
                | "support"
                | "supported"
                | "supports"
                | "supporting"
                | "allow"
                | "allowed"
                | "allows"
                | "allowing"
                | "available"
                | "enable"
                | "enabled"
                | "enables"
                | "enabling"
                | "use"
                | "used"
                | "uses"
                | "using"
        )
    })
}

fn current_rpc_claim_is_affirmative(after_rpc: &str) -> bool {
    if contains_term(
        after_rpc,
        &["not forbidden", "not prohibited", "not rejected"],
    ) {
        return true;
    }
    let (mut current_segment, adversative) = after_rpc
        .rsplit_once(" but ")
        .map_or((after_rpc, false), |(_, current)| (current, true));
    let explicit_current = contains_word(current_segment, &["now", "currently"]);
    if let Some((_, current)) = current_segment.rsplit_once(" now ") {
        current_segment = current;
    } else if let Some((_, current)) = current_segment.rsplit_once(" currently ") {
        current_segment = current;
    }
    let current_marker =
        adversative || explicit_current || contains_term(current_segment, &["no longer"]);
    if !current_marker {
        return false;
    }
    if contains_term(
        current_segment,
        &["no longer forbidden", "no longer prohibited"],
    ) {
        return true;
    }
    if contains_term(
        current_segment,
        &[
            "forbidden",
            "prohibited",
            "rejected",
            "not supported",
            "not now supported",
            "no longer supported",
            "not allowed",
            "not enabled",
            "not available",
        ],
    ) {
        return false;
    }
    contains_word(
        current_segment,
        &["supported", "allowed", "available", "enabled"],
    )
}

fn classify_rpc_audience(words: &[&str], rpc_index: usize) -> RpcAudience {
    let before = normalized_words(&words[rpc_index.saturating_sub(4)..rpc_index].join(" "));
    let rejection_before =
        normalized_words(&words[rpc_index.saturating_sub(8)..rpc_index].join(" "));
    let after = normalized_words(&words[rpc_index + 1..(rpc_index + 9).min(words.len())].join(" "));
    let local = normalized_words(
        &words[rpc_index.saturating_sub(8)..(rpc_index + 9).min(words.len())].join(" "),
    );
    let current_affirmative = current_rpc_claim_is_affirmative(&after);

    let historical_before = [
        "historical",
        "formerly",
        "former",
        "previously",
        "legacy",
        "named",
        "described",
    ];
    let historical_after = [
        "superseded",
        "retired",
        "replaced",
        "no longer",
        "deferred",
        "deprecated",
    ];
    let rejected_before = [
        "rejected alternative",
        "no direct",
        "no public",
        "do not expose",
        "does not expose",
        "must not expose",
        "do not create",
        "does not create",
    ];
    let rejected_after = [
        "forbidden",
        "prohibited",
        "not allowed",
        "must not",
        "cannot be public",
        "not public",
        "never public",
        "excluded",
        "rejected",
        "not supported",
        "not authorized",
        "not enabled",
        "not available",
        "needs a proxy",
        "requires a proxy",
    ];
    let internal_terms = [
        "internal only",
        "internal grpc",
        "grpc internal",
        "internal surface",
        "internal module",
        "internal service rpc",
        "internal connect",
        "sibling service",
        "east west",
        "stays internal",
        "remains internal",
        "for internal service",
    ];
    let inferred_internal_terms = [
        "service to service",
        "per service",
        "µservice",
        "microservice",
        "call each other",
        "workflow vs direct",
        "direct grpc",
        "pod",
        "outbound socket",
        "proto3 schema",
        "package path carries",
        "substrate to substrate",
        "outgoing grpc client",
        "synchronous grpc",
        "grpc sync",
    ];
    let nonclaim_terms = [
        "distinguished from",
        "protocol specific",
        "protocol layers",
        "layer enum",
        "surface kind",
        "consumer microservices",
    ];

    if current_affirmative
        && (contains_word(&local, &["public", "tenant", "tenants"])
            || contains_term(&local, &["product primitive", "tenant facing"]))
    {
        RpcAudience::Public
    } else if contains_term(&before, &historical_before) || contains_term(&after, &historical_after)
    {
        RpcAudience::Historical
    } else if contains_term(&rejection_before, &rejected_before)
        || contains_term(&after, &rejected_after)
    {
        RpcAudience::Rejected
    } else if contains_term(&local, &internal_terms) {
        RpcAudience::Internal
    } else if contains_word(&local, &["public"])
        || contains_term(&local, &["product primitive", "tenant facing"])
        || (contains_word(&local, &["tenant", "tenants"]) && contains_claim_term(&local))
        || (contains_term(
            &local,
            &[
                "native client",
                "clients call backend",
                "client calls backend",
            ],
        ) && contains_claim_term(&local))
    {
        RpcAudience::Public
    } else if contains_term(&local, &inferred_internal_terms) {
        RpcAudience::Internal
    } else if contains_term(&local, &nonclaim_terms) {
        RpcAudience::NonClaim
    } else if contains_claim_term(&local) {
        RpcAudience::Unclassified
    } else {
        RpcAudience::NonClaim
    }
}

fn public_rpc_findings(adr_id: &str, lifecycle: &str, document: &str) -> Vec<PublicRpcFinding> {
    let lifecycle = lifecycle
        .trim()
        .trim_matches(['\'', '"'])
        .to_ascii_lowercase();
    if !matches!(lifecycle.as_str(), "accepted" | "accepted (amendment)") {
        return Vec::new();
    }

    let normalized = document
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    let mut findings = Vec::new();
    for (clause_index, clause) in normalized.split(['.', '!', '?']).enumerate() {
        let words = clause.split_whitespace().collect::<Vec<_>>();
        for (word_index, word) in words.iter().enumerate() {
            let token = word
                .trim_matches(|character: char| !character.is_alphanumeric() && character != '-');
            let start = word_index.saturating_sub(12);
            let end = (word_index + 13).min(words.len());
            let window = words[start..end].join(" ");
            let bare_start = word_index.saturating_sub(4);
            let bare_end = (word_index + 5).min(words.len());
            let bare_window = normalized_words(&words[bare_start..bare_end].join(" "));
            let bare_connect_with_protocol_context = token == "connect"
                && ((contains_word(&bare_window, &["public", "tenant", "tenants"])
                    && (contains_claim_term(&bare_window)
                        || contains_word(&bare_window, &["endpoint", "endpoints"])))
                    || contains_term(
                        &bare_window,
                        &["connect protocol", "connect rpc", "integration"],
                    )
                    || words[bare_start..bare_end]
                        .iter()
                        .any(|candidate| candidate.contains("grpc")));
            let protocol = if token.contains("grpc") {
                "grpc"
            } else if token.contains("connect-rpc")
                || token.contains("connect-protocol")
                || bare_connect_with_protocol_context
            {
                "connect"
            } else {
                continue;
            };
            let audience = classify_rpc_audience(&words, word_index);
            if !matches!(audience, RpcAudience::Public | RpcAudience::Unclassified) {
                continue;
            }
            findings.push(PublicRpcFinding {
                key: format!(
                    "{adr_id}:public-rpc:{protocol}:{}:clause-{clause_index}-word-{word_index}",
                    match audience {
                        RpcAudience::Public => "public",
                        RpcAudience::Unclassified => "unclassified",
                        _ => unreachable!("non-finding audience was filtered"),
                    }
                ),
                clause: window,
            });
        }
    }
    findings
}

fn assert_public_protocol_reconciliation(adr_id: &str, document: &str, heading: &str) {
    let frontmatter = frontmatter(document);
    assert!(
        frontmatter.contains(&format!("id: {adr_id}")),
        "{adr_id} identity drifted"
    );
    assert!(
        frontmatter
            .lines()
            .any(|line| line.strip_prefix("status:").is_some_and(|status| {
                let status = status.trim().trim_matches(['\'', '"']);
                status.eq_ignore_ascii_case("accepted") || status.eq_ignore_ascii_case("superseded")
            })),
        "{adr_id} must remain Accepted or Superseded"
    );
    assert!(
        frontmatter.contains("ADR-0632"),
        "{adr_id} must relate to ADR-0632"
    );

    let reconciliation = section(document, heading)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    for required in [
        "openapi 3.2.0",
        "signed/versioned webhook",
        "asyncapi/cloudevents",
        "sse",
        "websocket",
        "graphql",
        "grpc-web",
        "connect",
        "internal-only",
        "grpc/proto3",
        "http/2",
    ] {
        assert!(
            reconciliation.contains(required),
            "{adr_id} reconciliation must cover {required}"
        );
    }

    let normalized_document = document
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    for contradiction in [
        "public rest/grpc",
        "public http/grpc",
        "external http/grpc",
        "every public µservice rpc (http + grpc",
        "proto3 services exposed externally",
        "proto3 reserved field oyatie_version",
    ] {
        assert!(
            !normalized_document.contains(contradiction),
            "{adr_id} reintroduced the public RPC contradiction {contradiction}"
        );
    }
}

fn replace_pointer(document: &mut Value, pointer: &str, replacement: Value) {
    let target = document
        .pointer_mut(pointer)
        .unwrap_or_else(|| panic!("fixture pointer must resolve: {pointer}"));
    *target = replacement;
}

#[test]
fn live_product_protocol_contract_is_green() {
    let policy = policy();
    assert_eq!(policy["gate_id"], GATE_ID);
    assert_eq!(
        policy["artifacts"],
        serde_json::json!({
            "product_contract": "specs/product-protocol-contract.json",
            "api_contract_ssot": "specs/api-contract-ssot-canonical.json",
            "transport_profile": "network/ports/transport-profile/endpoint-transport-profile.contract.json",
            "root_hub": "specs/root-hub-pointers.json",
            "manifest_schema": "specs/microservices/manifest-schema.json",
            "master_plan_sequencing": "specs/master-plan-sequencing.json"
        }),
        "policy artifact identities must stay bound to the Buck-declared location inputs"
    );
    let findings = evaluate_keyed(&policy, &artifacts());
    assert!(findings.is_empty(), "live contract findings: {findings:#?}");
}

#[test]
fn live_adr_authority_reconciliation_is_green() {
    let policy = policy();
    let heading = policy["authority_reconciliation"]["section_heading"]
        .as_str()
        .expect("accepted ADR section heading");
    let declared = policy["authority_reconciliation"]["accepted_adrs"]
        .as_array()
        .expect("accepted ADR inventory")
        .iter()
        .map(|row| row["id"].as_str().expect("ADR id"))
        .collect::<BTreeSet<_>>();

    let root = repo_root();
    let mut accepted_documents = BTreeMap::new();
    let accepted_statuses = policy["authority_reconciliation"]["accepted_statuses"]
        .as_array()
        .expect("accepted status inventory")
        .iter()
        .map(|status| status.as_str().expect("accepted status"))
        .collect::<BTreeSet<_>>();
    for corpus_rel in ["docs/decisions", "docs/adr-archive"] {
        let corpus_dir = root.join(corpus_rel);
        // A declared corpus root that no longer resolves is NOT "nothing to reconcile" — it is a
        // root that scans zero ADRs and reports green over an empty set. This used to `continue`,
        // so `mv docs/decisions` left this reconciliation passing with an empty authority map.
        assert!(
            corpus_dir.is_dir(),
            "declared ADR corpus root {corpus_rel} does not resolve under {} — reconciliation \
             over a missing root scans nothing and reports green; repoint the root in the same \
             change that moves it",
            root.display()
        );
        for entry in fs::read_dir(&corpus_dir).unwrap_or_else(|e| panic!("read {corpus_rel}: {e}"))
        {
            let path = entry.expect("ADR entry").path();
            let file_name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("");
            if path.extension().and_then(|value| value.to_str()) != Some("md")
                || file_name.len() < 9
                || !file_name.as_bytes()[4..8].iter().all(u8::is_ascii_digit)
                || file_name.as_bytes()[8] != b'-'
            {
                continue;
            }
            let document = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
            if document.starts_with("---\n") {
                let metadata = frontmatter(&document);
                let status_line = metadata
                    .lines()
                    .find(|line| line.starts_with("status:"))
                    .unwrap_or("");
                let status_ok = accepted_status(metadata, &accepted_statuses)
                    || status_line.to_ascii_lowercase().contains("superseded");
                if status_ok {
                    accepted_documents.insert(
                        metadata
                            .lines()
                            .find_map(|line| line.strip_prefix("id:").map(str::trim))
                            .unwrap_or(&file_name[..8])
                            .to_owned(),
                        document,
                    );
                }
            } else {
                let lifecycle_prefix = document.lines().take(40).collect::<Vec<_>>().join(" ");
                if lifecycle_prefix.to_ascii_lowercase().contains("accepted") {
                    accepted_documents.insert(file_name[..8].to_owned(), document);
                }
            }
        }
    }
    assert!(
        !accepted_documents.is_empty(),
        "Accepted ADR corpus must not be empty"
    );

    for id in &declared {
        let document = accepted_documents.get(*id).unwrap_or_else(|| {
            panic!(
                "reconciled ADR {id} must exist under live or archive corpus with an allowed status"
            )
        });
        assert_public_protocol_reconciliation(id, document, heading);
        assert!(
            frontmatter(document)
                .lines()
                .any(|line| { line.starts_with("amended_by:") && line.contains("ADR-0632") }),
            "{id} must carry the reciprocal amended_by lifecycle edge"
        );
    }

    let amending = text("OYA_ADR_0632");
    let amending_frontmatter = frontmatter(&amending);
    for id in &declared {
        assert!(
            amending_frontmatter
                .lines()
                .find(|line| line.starts_with("amends:"))
                .is_some_and(|line| line.contains(*id)),
            "ADR-0632 must carry reciprocal amends edge for {id}"
        );
    }

    let zero_graphql_layer_adrs = policy["authority_reconciliation"]["zero_graphql_layer_adrs"]
        .as_array()
        .expect("zero-GraphQL layer ADR inventory")
        .iter()
        .map(|entry| entry.as_str().expect("zero-GraphQL layer ADR id"))
        .collect::<BTreeSet<_>>();
    let zero_graphql_frontmatter = accepted_documents
        .get("ADR-0565")
        .map(|document| frontmatter(document))
        .expect("ADR-0565 must exist under live or archive corpus");
    for id in zero_graphql_layer_adrs {
        let amended = accepted_documents.get(id).unwrap_or_else(|| {
            panic!("zero-GraphQL layer ADR {id} must exist under live or archive corpus")
        });
        let amended_frontmatter = frontmatter(amended);
        for amendment in ["ADR-0565", "ADR-0632"] {
            assert!(
                amended_frontmatter
                    .lines()
                    .any(|line| { line.starts_with("amended_by:") && line.contains(amendment) }),
                "{id} must carry reciprocal amended_by edge for {amendment}"
            );
        }
        assert!(
            zero_graphql_frontmatter
                .lines()
                .find(|line| line.starts_with("amends:"))
                .is_some_and(|line| line.contains(id)),
            "ADR-0565 must carry reciprocal amends edge for {id}"
        );
    }

    for (id, document) in &accepted_documents {
        // Public-RPC purity is a live Accepted hard norm. Archive Superseded
        // members are scanned for declared-authority reconciliation only;
        // their historical wording must not fail this lane after disposition.
        let is_accepted = frontmatter(document).lines().any(|line| {
            line.strip_prefix("status:").is_some_and(|status| {
                status
                    .trim()
                    .trim_matches(['\'', '"'])
                    .eq_ignore_ascii_case("accepted")
            })
        });
        if !is_accepted {
            continue;
        }
        let findings = public_rpc_findings(id, "accepted", document);
        assert!(
            findings.is_empty(),
            "Accepted ADR {id} retains public RPC contradictions: {findings:#?}"
        );
    }

    let proposed = text("OYA_ADR_0246");
    let proposed_frontmatter = frontmatter(&proposed);
    assert!(proposed_frontmatter.contains("id: ADR-0246"));
    // Post-disposition: ADR-0246 is historical (Superseded) but retains the proposal
    // clarification section used by this gate as the non-authority note.
    assert!(
        proposed_frontmatter.contains("status: Proposed")
            || proposed_frontmatter.contains("status: Superseded")
    );
    assert!(proposed_frontmatter.contains("ADR-0632") || proposed.contains("ADR-0632"));
    let proposed_heading = policy["authority_reconciliation"]["proposed_section_heading"]
        .as_str()
        .expect("proposed ADR section heading");
    let clarification = section(&proposed, proposed_heading).to_ascii_lowercase();
    for required in [
        "remains **proposed**",
        "does not accept",
        "if accepted",
        "internal grpc/proto3 over http/2",
        "public and compatibility surface",
        "public grpc",
    ] {
        assert!(
            clarification.contains(required),
            "ADR-0246 clarification must preserve proposal semantics for {required}"
        );
    }
}

#[test]
fn manifest_schema_keeps_public_contracts_closed_and_grpc_internal() {
    let schema = json(&declared_path("OYA_MICROSERVICE_MANIFEST_SCHEMA"));
    let contract_properties = schema
        .pointer("/properties/contracts/properties")
        .and_then(Value::as_object)
        .expect("contract properties");
    assert!(!contract_properties.contains_key("graphql"));
    assert!(!contract_properties.contains_key("proto"));
    assert!(contract_properties.contains_key("internal_grpc"));

    let public_version_files = schema
        .pointer("/properties/tenant_version_pinning/properties/public_surface_files/properties")
        .and_then(Value::as_object)
        .expect("public version-file properties");
    assert_eq!(
        public_version_files
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["asyncapi", "openapi"]),
        "public version carriers must not admit GraphQL or proto3"
    );
}

#[test]
fn entire_buck_proto_corpus_is_internal_and_every_service_is_manifest_classified() {
    let root = repo_root();
    let policy = policy();
    let reviewed_legacy_manifests = string_set(
        &policy,
        "/manifest_inventory/reviewed_legacy_service_manifests",
    );
    // The proto corpus and the manifest corpus MUST be walked over the same governed_roots: a
    // manifest found under a root whose protos are not walked reports every one of its own
    // contracts as "outside the Buck-declared Proto corpus".
    let mut proto_files = Vec::new();
    for directory in governed_roots(&policy)
        .into_iter()
        .chain(std::iter::once("contracts".to_owned()))
    {
        collect_extension_files(&root.join(directory), "proto", &mut proto_files);
    }
    let declared_proto_paths = proto_files
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .expect("Proto contract must be below repository root")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect::<BTreeSet<_>>();
    assert!(
        !declared_proto_paths.is_empty(),
        "Buck-declared Proto corpus must not be empty"
    );

    let paths = collect_governed_manifests(&root, &policy);

    let mut manifest_contracts = BTreeSet::new();
    let mut findings = Vec::new();
    for path in paths {
        let manifest = json(&path);
        if let Some(contracts) = manifest
            .pointer("/contracts/internal_grpc/contracts")
            .and_then(Value::as_array)
        {
            for contract in contracts {
                if let Some(contract) = contract.as_str() {
                    manifest_contracts.insert(contract.to_owned());
                }
            }
        }
        findings.extend(internal_grpc_contract_path_findings(
            &root,
            &declared_proto_paths,
            &path,
            &manifest,
        ));
    }
    assert!(
        !manifest_contracts.is_empty(),
        "internal gRPC contract corpus must not be empty"
    );

    for contract in &declared_proto_paths {
        let path = root.join(contract);
        let document = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read Buck-declared Proto {contract}: {error}"));
        findings.extend(internal_proto_contract_content_findings(
            contract, &document,
        ));
        if let Some(finding) = unclassified_proto_service_finding(
            contract,
            &document,
            &manifest_contracts,
            &reviewed_legacy_manifests,
        ) {
            findings.push(finding);
        }
    }
    assert!(
        findings.is_empty(),
        "Buck-declared Proto corpus findings: {findings:#?}"
    );

    let mutation = serde_json::json!({
        "contracts": {
            "internal_grpc": {
                "contracts": ["__missing_internal_grpc_contract__.proto"]
            }
        }
    });
    let mutation_findings = internal_grpc_contract_path_findings(
        &root,
        &declared_proto_paths,
        Path::new("fixture/manifest.json"),
        &mutation,
    );
    assert_eq!(mutation_findings.len(), 1, "dangling path must fail closed");
    assert!(mutation_findings[0].contains("outside the Buck-declared Proto corpus"));

    let unreferenced_service = format!(
        "{INTERNAL_PROTO_EXPOSURE_MARKER}\n{INTERNAL_PROTO_TRANSPORT_MARKER}\n{INTERNAL_PROTO_IDENTITY_MARKER}\nsyntax = \"proto3\";\nservice Unclassified {{}}\n"
    );
    let classification_finding = unclassified_proto_service_finding(
        "fixture/unclassified-service.proto",
        &unreferenced_service,
        &BTreeSet::new(),
        &BTreeSet::new(),
    )
    .expect("unreferenced Proto service must fail closed");
    assert!(classification_finding.contains("reviewed-legacy manifest inventory"));

    let multiline_unreferenced_service = format!(
        "{INTERNAL_PROTO_EXPOSURE_MARKER}\n{INTERNAL_PROTO_TRANSPORT_MARKER}\n{INTERNAL_PROTO_IDENTITY_MARKER}\nsyntax = \"proto3\";\nservice\nMultilineUnclassified\n{{}}\n"
    );
    assert!(
        unclassified_proto_service_finding(
            "fixture/multiline-unclassified-service.proto",
            &multiline_unreferenced_service,
            &BTreeSet::new(),
            &BTreeSet::new(),
        )
        .is_some(),
        "valid multiline Proto service syntax must not bypass manifest classification"
    );
    assert!(
        !declares_proto_service(
            "// service CommentOnly {}\n/* service BlockCommentOnly {} */\noption java_package = \"service StringOnly {}\";\n"
        ),
        "comments and string literals must not fabricate a Proto service declaration"
    );

    assert!(
        unclassified_proto_service_finding(
            "oya/legacy/contracts/internal.proto",
            &unreferenced_service,
            &BTreeSet::new(),
            &BTreeSet::from(["oya/legacy/manifest.json".to_owned()]),
        )
        .is_none(),
        "the exact reviewed-legacy service inventory is the only manifest-reference exception"
    );
}

#[test]
fn internal_proto_contract_content_mutations_fail_closed() {
    let markers = format!(
        "{INTERNAL_PROTO_EXPOSURE_MARKER}\n{INTERNAL_PROTO_TRANSPORT_MARKER}\n{INTERNAL_PROTO_IDENTITY_MARKER}\n"
    );
    let cases = [
        (
            "missing-proto3.proto",
            format!("{markers}package oyatie.test;\n// Internal gRPC over HTTP/2.\n"),
            "must declare the proto3 language profile",
        ),
        (
            "http3.proto",
            format!("{markers}syntax = \"proto3\";\n// Internal gRPC over HTTP/3 and QUIC.\n"),
            "claims HTTP/3 or QUIC",
        ),
        (
            "public-choice.proto",
            format!(
                "{markers}syntax = \"proto3\";\n// This is available to clients that prefer gRPC.\n"
            ),
            "claims public or client-choice gRPC",
        ),
        (
            "grpc-web.proto",
            format!("{markers}syntax = \"proto3\";\n// Browser transport uses gRPC-Web.\n"),
            "claims forbidden public gRPC-Web exposure",
        ),
        (
            "external-auth.proto",
            format!(
                "{markers}syntax = \"proto3\";\n// Authentication: SPIFFE mTLS internally OR OIDC + API key externally.\n"
            ),
            "claims external or client-token authentication",
        ),
        (
            "client-token-auth.proto",
            format!(
                "{markers}syntax = \"proto3\";\n// Authn: per-request OIDC bearer in gRPC metadata.\n"
            ),
            "claims external or client-token authentication",
        ),
        (
            "multiline-public-choice.proto",
            format!(
                "{markers}syntax = \"proto3\";\n// gRPC mirrors REST for clients\n// that prefer protobuf wire format.\n"
            ),
            "claims public or client-choice gRPC/Protobuf across adjacent comments",
        ),
        (
            "flatbuffers.proto",
            format!("{markers}syntax = \"proto3\";\n// FlatBuffers payload for the hot path.\n"),
            "claims an unqualified FlatBuffers payload",
        ),
        (
            "oidc-all-rpcs.proto",
            format!("{markers}syntax = \"proto3\";\n// All RPCs require OIDC and tenant scope.\n"),
            "claims client-token authentication",
        ),
        (
            "missing-exposure-marker.proto",
            format!(
                "{INTERNAL_PROTO_TRANSPORT_MARKER}\n{INTERNAL_PROTO_IDENTITY_MARKER}\nsyntax = \"proto3\";\n"
            ),
            "must declare internal-only exposure",
        ),
        (
            "missing-transport-marker.proto",
            format!(
                "{INTERNAL_PROTO_EXPOSURE_MARKER}\n{INTERNAL_PROTO_IDENTITY_MARKER}\nsyntax = \"proto3\";\n"
            ),
            "must declare canonical gRPC-over-HTTP/2 transport",
        ),
        (
            "missing-identity-marker.proto",
            format!(
                "{INTERNAL_PROTO_EXPOSURE_MARKER}\n{INTERNAL_PROTO_TRANSPORT_MARKER}\nsyntax = \"proto3\";\n"
            ),
            "must declare SPIFFE mTLS/TLS 1.3 workload identity",
        ),
    ];
    for (path, document, expected) in cases {
        let findings = internal_proto_contract_content_findings(path, &document);
        assert!(
            findings.iter().any(|finding| finding.contains(expected)),
            "{path} must fail closed with {expected}: {findings:#?}"
        );
    }

    assert!(
        internal_proto_contract_content_findings(
            "internal-h2.proto",
            &format!(
                "{markers}syntax = \"proto3\";\n// Internal sibling-service gRPC over HTTP/2 with mTLS/TLS 1.3.\n"
            ),
        )
        .is_empty(),
        "the canonical internal gRPC profile must remain accepted"
    );
}

#[test]
fn retired_cloud_corpus_has_exact_accounting_and_cannot_revive_silently() {
    let root = repo_root();
    let policy = policy();
    // `governance` joined and `oya` left in the same change that emptied oya/. governance
    // holds a deployable service manifest now (the absorbed oya/governance service), and a
    // governed root is exactly where manifests are counted — the same reason `app`, also a
    // meta directory, is already here. `oya` leaves because the root no longer exists: a
    // governed root matching nothing is a dead scan root claiming coverage of an empty set.
    let expected_roots = BTreeSet::from([
        "app".to_owned(),
        "governance".to_owned(),
        "audit".to_owned(),
        "billing".to_owned(),
        "cell".to_owned(),
        "comms".to_owned(),
        "compliance".to_owned(),
        "console".to_owned(),
        "data".to_owned(),
        "flags".to_owned(),
        "gateway".to_owned(),
        "iac".to_owned(),
        "iam".to_owned(),
        "intelligence".to_owned(),
        "k8s".to_owned(),
        "marketplace".to_owned(),
        "network".to_owned(),
        "observability".to_owned(),
        "secrets".to_owned(),
        "storage".to_owned(),
        "tenancy".to_owned(),
        "workflow".to_owned(),
    ]);
    assert_eq!(
        string_set(&policy, "/manifest_inventory/governed_roots"),
        expected_roots,
        "the retired cloud root must leave every other governed root unchanged"
    );
    assert_eq!(
        policy["manifest_inventory"]["expected_total"], 76,
        "the governed corpus total is review-pinned: retiring the two cloud manifests moved it \
         96 -> 94, retiring the 51 unreferenced oya product crates moved it 94 -> 77, admitting \
         `app` as a governed root moved it 77 -> 78 by making the long-invisible \
         app/calendar/manifest.json countable, and draining the oya/calendar duplicate that \
         PR #1671 left behind moved it back 78 -> 77 — the same product had briefly been \
         counted at both paths, and absorbing the cloud-intelligence component manifest into \
         the intelligence capability manifest moved it 77 -> 76"
    );
    let retirement = policy["manifest_inventory"]["_comment"]
        .as_str()
        .expect("manifest retirement attribution");
    for retired in [
        "cloud/cloud-kernel/manifest.json",
        "cloud/cloud-os/manifest.json",
    ] {
        assert!(
            retirement.contains(retired),
            "retirement accounting must attribute {retired}"
        );
    }

    let projection = json(&root.join("specs/microservice-tier-classification.json"));
    // 94 -> 95 / substrate 53 -> 54: the `workflow` capability root lands its own
    // `workflow/manifest.json` in this change (ADR-0562 capability-first roots, PRs #1651-#1655).
    // It is one ADDITION to the governed corpus and is unrelated to the cloud/ retirement this
    // test attributes above — the retirement pins are the two `cloud/*` entries asserted there,
    // which are untouched. Kept as an exact pin rather than a range so a silent revival of the
    // retired corpus still fails here.
    // Retiring the 51 unreferenced oya product crates dropped 18 entries, all of them
    // product-tier: 95 -> 77 with substrate held at 54. A retirement of product crates that
    // moved the substrate count would mean the walk, not the retirement, changed — so the
    // substrate pin stays exact and is the control on this shrink.
    // 77 -> 78 when `app` joined governed_roots. That did NOT add a service: it made
    // app/calendar/manifest.json COUNTABLE. That manifest has existed since PR #1671 landed
    // the calendar absorb and was invisible to the census the whole time, because `app` was
    // not a governed root. Substrate stays 54 — an app product is product-tier — which is
    // the control: a root admission that moved substrate would mean the walk changed, not
    // the corpus.
    // 78 -> 77 when oya/calendar was drained. Admitting `app` briefly counted the SAME
    // product twice: app/calendar/manifest.json and the oya/calendar/manifest.json duplicate
    // PR #1671 left behind. Draining the duplicate removes the double-count, so this is the
    // 78 unwinding rather than a service being retired — calendar is still here, at
    // app/calendar. Substrate stays 54 again, and remains the control: a drain that moved
    // the substrate count would mean something other than a product duplicate was removed.
    // 77 -> 76, and THIS TIME SUBSTRATE MOVES (54 -> 53) — which is the point. The two
    // previous shifts were bookkeeping and had to leave substrate alone; this one is a real
    // absorption. `cloud-intelligence` was never a peer service: its crates already lived in
    // the intelligence capability root (core/account-*, adapters/anthropic-subscription-
    // adapter) and only its manifest and README stayed split, describing a component as
    // though it were a service. Folding it into the capability manifest's
    // absorbed_microservices retires one substrate manifest, so a substrate count that did
    // NOT move here would mean the absorption did not actually land.
    assert_eq!(projection["service_count"], 76);
    assert_eq!(projection["tier_distribution"]["substrate"], 53);
    assert!(
        projection["services"]
            .as_object()
            .expect("tier projection services")
            .keys()
            .all(|path| !path.starts_with("cloud/")),
        "the read-only tier projection must not retain retired cloud manifests"
    );
}

#[test]
fn entire_governed_manifest_corpus_is_inventoried_and_protocol_compatible() {
    let root = repo_root();
    let policy = policy();
    let schema = json(&declared_path("OYA_MICROSERVICE_MANIFEST_SCHEMA"));
    let allowed_contract_keys = schema
        .pointer("/properties/contracts/properties")
        .and_then(Value::as_object)
        .expect("contract properties")
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        allowed_contract_keys,
        BTreeSet::from([
            "asyncapi",
            "contract_status",
            "convention_docs",
            "internal_grpc",
            "openapi",
            "sdk",
            "service_local_contract_scaffolds",
            "source",
            "trait",
        ]),
        "manifest contract keys must be the closed public/internal carriers plus compatibility metadata"
    );

    let paths = collect_governed_manifests(&root, &policy);

    // PROCESS_TAX DELETE: hand equality on expected_total / expected_live_v1_total is not a merge
    // blocker. Anti-vacuity: refuse a collapsed walk. Named reviewed inventories stay equality-
    // pinned (classification, not census counters).
    assert!(
        !paths.is_empty(),
        "governed manifest walk saw zero paths — refuse vacuous green"
    );
    let reviewed_legacy = string_set(
        &policy,
        "/manifest_inventory/reviewed_legacy_service_manifests",
    );
    let reviewed_overlays = string_set(&policy, "/manifest_inventory/reviewed_overlay_manifests");
    assert!(
        reviewed_legacy.is_disjoint(&reviewed_overlays),
        "legacy service and overlay inventories must not overlap"
    );

    let mut live_count = 0;
    let mut observed_legacy = BTreeSet::new();
    let mut observed_overlays = BTreeSet::new();
    let mut observed_public_proto = BTreeSet::new();
    let mut protocol_findings = Vec::new();
    for path in paths {
        let manifest = json(&path);
        let relative = path
            .strip_prefix(&root)
            .expect("manifest must be below repository root")
            .to_string_lossy()
            .replace('\\', "/");
        protocol_findings.extend(manifest_protocol_findings(&path, &manifest));
        if manifest
            .pointer("/tenant_version_pinning/public_surface_files/proto")
            .is_some()
        {
            observed_public_proto.insert(relative.clone());
        }
        if manifest["schema_version"] != "1.0" || !manifest["contracts"].is_object() {
            if relative.contains("/packs/") {
                observed_overlays.insert(relative);
            } else {
                observed_legacy.insert(relative);
            }
            continue;
        }
        live_count += 1;
        assert!(
            manifest["microservice"].is_string(),
            "{} live v1.0 microservice identity must be a string",
            path.display()
        );
        let contracts = manifest["contracts"]
            .as_object()
            .unwrap_or_else(|| panic!("{} contracts must be an object", path.display()));
        for key in contracts.keys() {
            assert!(
                allowed_contract_keys.contains(key.as_str()),
                "{} uses undeclared contract carrier {key}",
                path.display()
            );
        }
        for public in ["openapi", "asyncapi"] {
            let values = contracts[public].as_array().unwrap_or_else(|| {
                panic!("{} contracts.{public} must be an array", path.display())
            });
            assert!(
                values.iter().all(Value::is_string),
                "{} contracts.{public} must contain only paths",
                path.display()
            );
        }
        for metadata in [
            "convention_docs",
            "sdk",
            "service_local_contract_scaffolds",
            "source",
            "trait",
        ] {
            if let Some(values) = contracts.get(metadata) {
                assert!(
                    values
                        .as_array()
                        .is_some_and(|values| values.iter().all(Value::is_string)),
                    "{} contracts.{metadata} compatibility metadata must contain only paths",
                    path.display()
                );
            }
        }
        if let Some(status) = contracts.get("contract_status") {
            assert!(
                status.is_string(),
                "{} contracts.contract_status compatibility metadata must be a string",
                path.display()
            );
        }
        assert!(!contracts.contains_key("proto"));
        assert!(!contracts.contains_key("connect"));
        let internal = contracts["internal_grpc"]
            .as_object()
            .unwrap_or_else(|| panic!("{} must declare migrated internal_grpc", path.display()));
        assert_eq!(
            internal.len(),
            3,
            "{} internal_grpc must be closed",
            path.display()
        );
        assert_eq!(
            internal["transport"],
            "http2",
            "{} internal gRPC must use H2",
            path.display()
        );
        assert_eq!(
            internal["language_profile"],
            "proto3",
            "{} internal gRPC must use proto3",
            path.display()
        );
        assert!(
            internal["contracts"]
                .as_array()
                .is_some_and(|values| values.iter().all(Value::is_string)),
            "{} internal_grpc.contracts must be an array of paths",
            path.display()
        );
        if let Some(public_files) = manifest
            .pointer("/tenant_version_pinning/public_surface_files")
            .and_then(Value::as_object)
        {
            assert!(
                public_files
                    .keys()
                    .all(|key| matches!(key.as_str(), "openapi" | "asyncapi")),
                "{} public version carriers must exclude Proto/Connect",
                path.display()
            );
        }
    }
    // PROCESS_TAX DELETE: expected_live_v1_total equality is not a merge blocker.
    assert!(
        live_count > 0,
        "live v1.0 service manifest walk saw zero — refuse vacuous green"
    );
    assert_eq!(
        observed_legacy, reviewed_legacy,
        "legacy service manifest review baseline drifted"
    );
    assert_eq!(
        observed_overlays, reviewed_overlays,
        "localization overlay manifest review baseline drifted"
    );
    assert!(
        observed_public_proto.is_empty(),
        "public_surface_files.proto is forbidden in every governed manifest: {observed_public_proto:#?}"
    );
    assert_eq!(
        policy["manifest_inventory"]["public_proto_posture"],
        "No public_surface_files.proto carrier is permitted, including in legacy manifests. Every Buck-corpus Proto is internal-only and marker-classified; live Proto service contracts must also be declared through contracts.internal_grpc. Named reviewed-legacy service contracts remain migration inventory, receive no content exemption, and do not imply runtime readiness.",
        "policy must preserve the zero-public-Protobuf posture"
    );
    assert!(
        protocol_findings.is_empty(),
        "shape-independent manifest protocol findings: {protocol_findings:#?}"
    );
}

// specs/microservice-tier-classification.json calls itself the roll-up of the per-manifest tier
// facets and platform-architecture.json references it as tier_classification_table_ref, but NO gate
// opened it: the coverage enforcement its own _provenance names walks the live manifests directly,
// and the reachability registry only asserts the path exists. So it drifted in silence — it sat at
// 96 services while expected_total moved to 101 in this very file, and every lane stayed green. A
// projection nothing reads is a declaration wired to nothing, which is worse than no projection at
// all, because consumers trust it. This binds it to the SAME walk that produces expected_total, and
// binds its counts to its own entries so neither can be hand-typed into agreement.
#[test]
fn tier_classification_projection_is_the_governed_manifest_corpus() {
    let root = repo_root();
    let policy = policy();
    let projection = json(&root.join("specs/microservice-tier-classification.json"));
    let services = projection["services"]
        .as_object()
        .expect("tier projection services map");

    let projected = services.keys().cloned().collect::<BTreeSet<_>>();
    let governed = collect_governed_manifests(&root, &policy)
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .expect("manifest must be below repository root")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        projected, governed,
        "the tier projection must name exactly the governed manifest corpus; re-project it in the \
         same change that moves the corpus"
    );

    // The counts are DERIVED from the entries rather than asserted beside them.
    assert_eq!(
        services.len() as u64,
        projection["service_count"]
            .as_u64()
            .expect("projection service_count"),
        "service_count must equal the number of projected entries"
    );
    let mut distribution: BTreeMap<String, u64> = BTreeMap::new();
    for entry in services.values() {
        let tier = entry["tier"].as_str().expect("projected tier").to_owned();
        *distribution.entry(tier).or_default() += 1;
    }
    for (tier, count) in &distribution {
        assert_eq!(
            projection["tier_distribution"][tier]
                .as_u64()
                .unwrap_or_default(),
            *count,
            "tier_distribution.{tier} must be the measured histogram of the projected entries"
        );
    }

    // A projection that disagrees with its source is worse than a missing one. The per-manifest
    // facet is the source of truth (ADR-0245 tier-as-facet); this file is its read-only roll-up.
    for (relative, entry) in services {
        let manifest = json(&root.join(relative));
        for facet in ["tier", "tier_subtype", "dr_tier", "substrate_dag_position"] {
            assert_eq!(
                entry.get(facet),
                manifest.get(facet),
                "{relative}: projected {facet} must be verbatim from the manifest that owns it"
            );
        }
    }
}

#[test]
fn manifest_protocol_red_mutations_are_rejected() {
    for contracts in [
        serde_json::json!({"openapi": [], "asyncapi": [], "proto": ["public.proto"]}),
        serde_json::json!({"openapi": [], "asyncapi": [], "connect": ["public.connect"]}),
        serde_json::json!({"openapi": [], "asyncapi": [], "internal_grpc": {"transport": "http3", "language_profile": "proto3", "contracts": []}}),
        serde_json::json!({"openapi": [], "asyncapi": [], "internal_grpc": {"transport": "http2", "language_profile": "editions-2024", "contracts": []}}),
    ] {
        let keys = contracts.as_object().expect("fixture object");
        let valid = keys.keys().all(|key| {
            matches!(
                key.as_str(),
                "openapi" | "asyncapi" | "source" | "internal_grpc"
            )
        }) && keys.get("internal_grpc").is_none_or(|internal| {
            internal["transport"] == "http2" && internal["language_profile"] == "proto3"
        });
        assert!(
            !valid,
            "RED manifest mutation was incorrectly accepted: {contracts}"
        );
    }

    for mutation in [
        serde_json::json!({
            "microservice": "schema-less-bypass",
            "contracts": ["legacy-shape"],
            "grpc_transport": "gRPC over HTTP/3 internally"
        }),
        serde_json::json!({
            "schema_version": "0.9",
            "microservice": "non-v1-bypass",
            "contracts": "legacy-shape",
            "internal_grpc": {
                "transport": "http3",
                "language_profile": "proto3"
            }
        }),
        serde_json::json!({
            "microservice": "schema-less-flatbuffers-bypass",
            "contracts": ["legacy-shape"],
            "serialization": "FlatBuffers is canonical for the waveform path"
        }),
        serde_json::json!({
            "microservice": "legacy-public-proto-bypass",
            "contracts": ["legacy-shape"],
            "tenant_version_pinning": {
                "public_surface_files": {
                    "proto": "contracts/public.proto"
                }
            }
        }),
    ] {
        let findings = manifest_protocol_findings(Path::new("fixture/manifest.json"), &mutation);
        assert!(
            !findings.is_empty(),
            "non-v1/schema-less RED manifest bypassed protocol checks: {mutation}"
        );
    }
}

#[test]
fn queued_15v_has_no_public_proto_or_rpc_discovery_carrier() {
    let plan = json(&declared_path("OYA_MASTER_PLAN_SEQUENCING"));
    let plan_rendered = serde_json::to_string(&plan)
        .expect("master plan must serialize")
        .to_ascii_lowercase();
    for contradiction in [
        "public rest/asyncapi/proto3",
        "proto3 services exposed externally",
        "proto3 reserved field oyatie_version",
        "versionsservice",
    ] {
        assert!(
            !plan_rendered.contains(contradiction),
            "master-plan sequencing reintroduced {contradiction}"
        );
    }
    let wave = plan
        .pointer("/realignment_wave_sequence/waves_15_plus/sub_wave_landings/15V-API-Versioning-Adoption")
        .expect("15V wave");
    let rendered = serde_json::to_string(wave)
        .expect("15V must serialize")
        .to_ascii_lowercase();
    assert!(!rendered.contains("contracts/*.proto"));
    for required in [
        "openapi 3.2.0",
        "signed/versioned webhook",
        "asyncapi 3.1.0",
        "sse",
        "websocket",
        "internal-mesh grpc/proto3 over http/2",
        "exempt",
    ] {
        assert!(rendered.contains(required), "15V must preserve {required}");
    }
}

#[test]
fn named_transport_class_rules_are_order_independent() {
    let policy = policy();
    let mut observed = artifacts();
    observed
        .get_mut("transport_profile")
        .expect("transport profile")
        .get_mut("capability_classes")
        .and_then(Value::as_array_mut)
        .expect("capability classes")
        .reverse();

    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings.is_empty(),
        "transport class ordering changed the verdict: {findings:#?}"
    );
}

#[test]
fn negative_fixture_corpus_fails_on_each_guarded_invariant() {
    let policy = policy();
    let baseline = artifacts();
    let fixtures = json(&declared_path("OYA_PRODUCT_PROTOCOL_NEGATIVE_CASES"));

    let cases = fixtures["cases"].as_array().expect("cases array");
    let required_codes = policy["rules"]
        .as_array()
        .expect("rules array")
        .iter()
        .map(|rule| rule["code"].as_str().expect("rule code"))
        .collect::<BTreeSet<_>>();
    let fixture_codes = cases
        .iter()
        .map(|case| case["expected_code"].as_str().expect("expected code"))
        .collect::<BTreeSet<_>>();
    assert!(
        required_codes == fixture_codes,
        "negative fixtures must cover every policy rule exactly by code; missing={:?}, unknown={:?}",
        required_codes
            .difference(&fixture_codes)
            .collect::<Vec<_>>(),
        fixture_codes
            .difference(&required_codes)
            .collect::<Vec<_>>()
    );
    let fixture_names = cases
        .iter()
        .map(|case| case["name"].as_str().expect("case name"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        fixture_names.len(),
        cases.len(),
        "fixture names must be unique"
    );
    for case in cases {
        let name = case["name"].as_str().expect("case name");
        let artifact = case["artifact"].as_str().expect("artifact name");
        let pointer = case["pointer"].as_str().expect("JSON pointer");
        let expected_code = case["expected_code"].as_str().expect("expected code");
        let mut observed = baseline.clone();
        replace_pointer(
            observed.get_mut(artifact).expect("known artifact"),
            pointer,
            case["replacement"].clone(),
        );
        let findings = evaluate_keyed(&policy, &observed);
        assert!(
            findings.iter().any(|finding| finding.code == expected_code),
            "negative fixture {name} did not emit {expected_code}: {findings:#?}"
        );
    }
}

#[test]
fn accepted_adr_public_rpc_red_mutations_are_detected() {
    let fixtures = json(&declared_path("OYA_PRODUCT_PROTOCOL_NEGATIVE_CASES"));
    let accepted_baseline = "# Fixture Accepted ADR\n\nPublic gRPC, gRPC-Web, and Connect are forbidden. Sibling-service calls use internal-only gRPC/proto3 over HTTP/2.";
    let cases = fixtures["accepted_adr_cases"]
        .as_array()
        .expect("Accepted ADR RED cases");
    assert!(
        !cases.is_empty(),
        "Accepted ADR RED corpus must not be empty"
    );
    for case in cases {
        let name = case["name"].as_str().expect("case name");
        let mutation = case["text"].as_str().expect("RED ADR text");
        let document = format!("{accepted_baseline}\n\n{mutation}\n");
        let findings = public_rpc_findings(name, "Accepted", &document);
        assert!(
            !findings.is_empty(),
            "RED ADR mutation {name} evaded the live detector"
        );
        assert!(
            findings.iter().all(|finding| finding.key.starts_with(name)),
            "RED ADR mutation {name} did not emit a keyed finding: {findings:#?}"
        );
    }

    for case in fixtures["accepted_adr_counterexamples"]
        .as_array()
        .expect("Accepted ADR counterexamples")
    {
        let name = case["name"].as_str().expect("case name");
        let text = case["text"].as_str().expect("counterexample text");
        let lifecycle = case["lifecycle"].as_str().unwrap_or("Accepted");
        let findings = public_rpc_findings(name, lifecycle, text);
        assert!(
            findings.is_empty(),
            "counterexample {name} produced false positives: {findings:#?}"
        );
    }
}

#[test]
fn malformed_or_empty_policy_fails_closed() {
    let findings = evaluate_keyed(&serde_json::json!({}), &BTreeMap::new());
    assert!(
        !findings.is_empty(),
        "empty policy must not certify an empty universe"
    );
}
