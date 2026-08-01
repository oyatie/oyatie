#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    std::env::var_os(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("Buck must declare {variable} with $(location)"))
}

fn policy() -> Value {
    json(&declared_path("OYA_PRODUCT_PROTOCOL_POLICY"))
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
            collect_named_files(&path, name, output);
        } else if entry.file_name() == name {
            output.push(path);
        }
    }
}

fn tracked_paths(root: &Path) -> BTreeSet<String> {
    let output = Command::new("git")
        .args([
            "-C",
            root.to_str().expect("UTF-8 repository root"),
            "ls-files",
            "-z",
        ])
        .output()
        .expect("run git ls-files for the tracked contract universe");
    assert!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("git ls-files output must be UTF-8")
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(str::to_owned)
        .collect()
}

fn internal_grpc_contract_path_findings(
    root: &Path,
    tracked: &BTreeSet<String>,
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
            let metadata = fs::symlink_metadata(root.join(contract));
            if !tracked.contains(contract) {
                Some(format!(
                    "{} declares untracked internal gRPC contract {contract}",
                    manifest_path.display()
                ))
            } else if !metadata.is_ok_and(|metadata| metadata.file_type().is_file()) {
                Some(format!(
                    "{} declares non-regular internal gRPC contract {contract}",
                    manifest_path.display()
                ))
            } else {
                None
            }
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
    let (current_segment, adversative) = after_rpc
        .rsplit_once(" but ")
        .map_or((after_rpc, false), |(_, current)| (current, true));
    let current_marker = adversative
        || contains_word(current_segment, &["now", "currently"])
        || contains_term(current_segment, &["no longer"]);
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
        "not enabled",
        "not available",
        "needs a proxy",
    ];
    let internal_terms = [
        "internal only",
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
    for (clause_index, clause) in normalized.split(['.', '!', '?', ';']).enumerate() {
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
                status
                    .trim()
                    .trim_matches(['\'', '"'])
                    .eq_ignore_ascii_case("accepted")
            })),
        "{adr_id} must remain Accepted"
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
    for entry in fs::read_dir(root.join("docs/decisions")).expect("read declared ADR corpus") {
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
            if accepted_status(metadata, &accepted_statuses) {
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
    assert!(
        !accepted_documents.is_empty(),
        "Accepted ADR corpus must not be empty"
    );

    for id in &declared {
        let document = accepted_documents
            .get(*id)
            .unwrap_or_else(|| panic!("reconciled ADR {id} must exist and remain Accepted"));
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

    for (id, document) in &accepted_documents {
        let findings = public_rpc_findings(id, "accepted", document);
        assert!(
            findings.is_empty(),
            "Accepted ADR {id} retains public RPC contradictions: {findings:#?}"
        );
    }

    let proposed = text("OYA_ADR_0246");
    let proposed_frontmatter = frontmatter(&proposed);
    assert!(proposed_frontmatter.contains("id: ADR-0246"));
    assert!(proposed_frontmatter.contains("status: Proposed"));
    assert!(proposed_frontmatter.contains("ADR-0632"));
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
fn every_declared_internal_grpc_contract_is_a_regular_tracked_file() {
    let root = repo_root();
    let tracked = tracked_paths(&root);
    let mut paths = Vec::new();
    collect_named_files(&root.join("oya"), "manifest.json", &mut paths);
    collect_named_files(&root.join("cloud"), "manifest.json", &mut paths);
    paths.sort();

    let mut declared_contracts = 0;
    let mut findings = Vec::new();
    for path in paths {
        let manifest = json(&path);
        declared_contracts += manifest
            .pointer("/contracts/internal_grpc/contracts")
            .and_then(Value::as_array)
            .map_or(0, Vec::len);
        findings.extend(internal_grpc_contract_path_findings(
            &root, &tracked, &path, &manifest,
        ));
    }
    assert!(
        declared_contracts > 0,
        "internal gRPC contract corpus must not be empty"
    );
    assert!(
        findings.is_empty(),
        "internal gRPC contract path findings: {findings:#?}"
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
        &tracked,
        Path::new("fixture/manifest.json"),
        &mutation,
    );
    assert_eq!(mutation_findings.len(), 1, "dangling path must fail closed");
    assert!(mutation_findings[0].contains("untracked internal gRPC contract"));
}

#[test]
fn entire_live_v1_manifest_corpus_is_protocol_schema_compatible() {
    let root = repo_root();
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

    let mut paths = Vec::new();
    collect_named_files(&root.join("oya"), "manifest.json", &mut paths);
    collect_named_files(&root.join("cloud"), "manifest.json", &mut paths);
    paths.sort();

    let mut live_count = 0;
    for path in paths {
        let manifest = json(&path);
        if manifest["schema_version"] != "1.0" || !manifest["contracts"].is_object() {
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
    assert_eq!(
        live_count, 60,
        "the Buck-declared live v1.0 service manifest corpus changed; classify and migrate every new match"
    );
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
