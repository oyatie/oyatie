//! Source-controlled proto3 contract registry for backbone write services.
//!
//! This kernel embeds the hand-authored `.proto` files for messenger, mail,
//! social, and community write RPCs and validates their expected proto3 syntax,
//! package, service, RPC, request/response message, and common authorization
//! metadata fields. It does not generate Rust stubs, serialize protobuf bytes,
//! run a gRPC server/client, or prove transport delivery.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub const MESSENGER_MESSAGE_STREAM_PROTO: &str =
    include_str!("../contracts/backbone/messenger/message_stream.proto");
pub const MAIL_PROTO: &str = include_str!("../contracts/backbone/mail/mail.proto");
pub const SOCIAL_POST_COMPOSITION_PROTO: &str =
    include_str!("../contracts/backbone/social/social_post_composition.proto");
pub const COMMUNITY_POST_STORE_PROTO: &str =
    include_str!("../contracts/backbone/community/community_post_store.proto");

const COMMON_METADATA_FIELDS: &[&str] = &[
    "tenant_scope_ref",
    "principal_ref",
    "idempotency_key",
    "policy_decision_ref",
    "audit_correlation_id",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackboneProtoRpc {
    pub service: &'static str,          // data_class: INTERNAL_ONLY
    pub rpc: &'static str,              // data_class: INTERNAL_ONLY
    pub request_message: &'static str,  // data_class: INTERNAL_ONLY
    pub response_message: &'static str, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackboneProtoContract {
    pub path: &'static str,                       // data_class: INTERNAL_ONLY
    pub package: &'static str,                    // data_class: INTERNAL_ONLY
    pub source: &'static str,                     // data_class: INTERNAL_ONLY
    pub rpcs: &'static [BackboneProtoRpc],        // data_class: INTERNAL_ONLY
    pub required_fields: &'static [&'static str], // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BackboneProtoContractError {
    MissingProto3Syntax {
        path: &'static str,
    },
    MissingPackage {
        path: &'static str,
        expected_package: &'static str,
    },
    MissingService {
        path: &'static str,
        service: &'static str,
    },
    MissingRpc {
        path: &'static str,
        service: &'static str,
        rpc: &'static str,
        request_message: &'static str,
        response_message: &'static str,
    },
    MissingMessage {
        path: &'static str,
        message: &'static str,
    },
    MissingField {
        path: &'static str,
        field: &'static str,
    },
    ForbiddenKeyword {
        path: &'static str,
        keyword: &'static str,
    },
}

const MESSENGER_RPCS: &[BackboneProtoRpc] = &[BackboneProtoRpc {
    service: "MessageStream",
    rpc: "PostMessage",
    request_message: "PostMessageRequest",
    response_message: "PostMessageResponse",
}];

const MAIL_RPCS: &[BackboneProtoRpc] = &[BackboneProtoRpc {
    service: "Mail",
    rpc: "SendMessage",
    request_message: "SendMessageRequest",
    response_message: "SendMessageResponse",
}];

const SOCIAL_RPCS: &[BackboneProtoRpc] = &[BackboneProtoRpc {
    service: "PostComposition",
    rpc: "PublishPost",
    request_message: "PublishPostRequest",
    response_message: "PublishPostResponse",
}];

const COMMUNITY_RPCS: &[BackboneProtoRpc] = &[
    BackboneProtoRpc {
        service: "PostStoreService",
        rpc: "CreatePost",
        request_message: "CreatePostRequest",
        response_message: "CreatePostResponse",
    },
    BackboneProtoRpc {
        service: "VotingEngineService",
        rpc: "CastVote",
        request_message: "CastVoteRequest",
        response_message: "CastVoteResponse",
    },
    BackboneProtoRpc {
        service: "ModerationQueueService",
        rpc: "ApplyAction",
        request_message: "ApplyActionRequest",
        response_message: "ApplyActionResponse",
    },
];

pub const MESSENGER_PROTO_CONTRACT: BackboneProtoContract = BackboneProtoContract {
    path: "specs/proto/backbone/messenger/message_stream.proto",
    package: "oya.messenger.v1",
    source: MESSENGER_MESSAGE_STREAM_PROTO,
    rpcs: MESSENGER_RPCS,
    required_fields: COMMON_METADATA_FIELDS,
};

pub const MAIL_PROTO_CONTRACT: BackboneProtoContract = BackboneProtoContract {
    path: "specs/proto/backbone/mail/mail.proto",
    package: "oya.mail.v1",
    source: MAIL_PROTO,
    rpcs: MAIL_RPCS,
    required_fields: COMMON_METADATA_FIELDS,
};

pub const SOCIAL_PROTO_CONTRACT: BackboneProtoContract = BackboneProtoContract {
    path: "specs/proto/backbone/social/social_post_composition.proto",
    package: "oya.social.v1",
    source: SOCIAL_POST_COMPOSITION_PROTO,
    rpcs: SOCIAL_RPCS,
    required_fields: COMMON_METADATA_FIELDS,
};

pub const COMMUNITY_PROTO_CONTRACT: BackboneProtoContract = BackboneProtoContract {
    path: "specs/proto/backbone/community/community_post_store.proto",
    package: "oya.community.v1",
    source: COMMUNITY_POST_STORE_PROTO,
    rpcs: COMMUNITY_RPCS,
    required_fields: COMMON_METADATA_FIELDS,
};

pub const BACKBONE_PROTO_CONTRACTS: &[BackboneProtoContract] = &[
    MESSENGER_PROTO_CONTRACT,
    MAIL_PROTO_CONTRACT,
    SOCIAL_PROTO_CONTRACT,
    COMMUNITY_PROTO_CONTRACT,
];

pub fn validate_all_backbone_proto_contracts() -> Result<(), BackboneProtoContractError> {
    for contract in BACKBONE_PROTO_CONTRACTS {
        validate_proto_contract(contract)?;
    }
    Ok(())
}

pub fn validate_proto_contract(
    contract: &BackboneProtoContract,
) -> Result<(), BackboneProtoContractError> {
    require_contains(
        contract,
        "syntax = \"proto3\";",
        BackboneProtoContractError::MissingProto3Syntax {
            path: contract.path,
        },
    )?;
    require_absent_keyword(contract, "required")?;
    require_absent_keyword(contract, "syntax = \"proto2\"")?;

    let package_statement = format!("package {};", contract.package);
    require_contains(
        contract,
        &package_statement,
        BackboneProtoContractError::MissingPackage {
            path: contract.path,
            expected_package: contract.package,
        },
    )?;

    for rpc in contract.rpcs {
        require_contains(
            contract,
            &format!("service {} {{", rpc.service),
            BackboneProtoContractError::MissingService {
                path: contract.path,
                service: rpc.service,
            },
        )?;
        require_contains(
            contract,
            &format!(
                "rpc {} ({}) returns ({});",
                rpc.rpc, rpc.request_message, rpc.response_message
            ),
            BackboneProtoContractError::MissingRpc {
                path: contract.path,
                service: rpc.service,
                rpc: rpc.rpc,
                request_message: rpc.request_message,
                response_message: rpc.response_message,
            },
        )?;
        require_contains(
            contract,
            &format!("message {} {{", rpc.request_message),
            BackboneProtoContractError::MissingMessage {
                path: contract.path,
                message: rpc.request_message,
            },
        )?;
        require_contains(
            contract,
            &format!("message {} {{", rpc.response_message),
            BackboneProtoContractError::MissingMessage {
                path: contract.path,
                message: rpc.response_message,
            },
        )?;
    }

    for field in contract.required_fields {
        if !source_contains_token(contract.source, field) {
            return Err(BackboneProtoContractError::MissingField {
                path: contract.path,
                field,
            });
        }
    }

    Ok(())
}

fn require_contains(
    contract: &BackboneProtoContract,
    needle: &str,
    error: BackboneProtoContractError,
) -> Result<(), BackboneProtoContractError> {
    if contract.source.contains(needle) {
        Ok(())
    } else {
        Err(error)
    }
}

fn require_absent_keyword(
    contract: &BackboneProtoContract,
    keyword: &'static str,
) -> Result<(), BackboneProtoContractError> {
    if contract.source.contains(keyword) {
        Err(BackboneProtoContractError::ForbiddenKeyword {
            path: contract.path,
            keyword,
        })
    } else {
        Ok(())
    }
}

fn source_contains_token(source: &str, token: &str) -> bool {
    source
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .any(|part| part == token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_backbone_proto_contracts_validate() {
        validate_all_backbone_proto_contracts().unwrap();
    }

    #[test]
    fn contracts_register_expected_packages_and_rpcs() {
        let packages: Vec<&str> = BACKBONE_PROTO_CONTRACTS
            .iter()
            .map(|contract| contract.package)
            .collect();
        assert_eq!(
            packages,
            vec![
                "oya.messenger.v1",
                "oya.mail.v1",
                "oya.social.v1",
                "oya.community.v1",
            ]
        );
        assert_eq!(COMMUNITY_PROTO_CONTRACT.rpcs.len(), 3);
        assert_eq!(SOCIAL_PROTO_CONTRACT.rpcs[0].rpc, "PublishPost");
    }

    #[test]
    fn validator_rejects_missing_proto3_syntax() {
        let contract = BackboneProtoContract {
            source: "package oya.messenger.v1; service MessageStream { rpc PostMessage (PostMessageRequest) returns (PostMessageResponse); } message PostMessageRequest {} message PostMessageResponse {} tenant_scope_ref principal_ref idempotency_key policy_decision_ref audit_correlation_id",
            ..MESSENGER_PROTO_CONTRACT
        };

        assert_eq!(
            validate_proto_contract(&contract),
            Err(BackboneProtoContractError::MissingProto3Syntax {
                path: MESSENGER_PROTO_CONTRACT.path,
            })
        );
    }

    #[test]
    fn validator_rejects_missing_rpc() {
        let contract = BackboneProtoContract {
            source: "syntax = \"proto3\"; package oya.mail.v1; service Mail {} message SendMessageRequest {} message SendMessageResponse {} tenant_scope_ref principal_ref idempotency_key policy_decision_ref audit_correlation_id",
            ..MAIL_PROTO_CONTRACT
        };

        assert_eq!(
            validate_proto_contract(&contract),
            Err(BackboneProtoContractError::MissingRpc {
                path: MAIL_PROTO_CONTRACT.path,
                service: "Mail",
                rpc: "SendMessage",
                request_message: "SendMessageRequest",
                response_message: "SendMessageResponse",
            })
        );
    }

    #[test]
    fn validator_rejects_missing_common_field() {
        let contract = BackboneProtoContract {
            source: "syntax = \"proto3\"; package oya.social.v1; service PostComposition { rpc PublishPost (PublishPostRequest) returns (PublishPostResponse); } message PublishPostRequest {} message PublishPostResponse {} principal_ref idempotency_key policy_decision_ref audit_correlation_id",
            ..SOCIAL_PROTO_CONTRACT
        };

        assert_eq!(
            validate_proto_contract(&contract),
            Err(BackboneProtoContractError::MissingField {
                path: SOCIAL_PROTO_CONTRACT.path,
                field: "tenant_scope_ref",
            })
        );
    }

    #[test]
    fn validator_rejects_required_keyword() {
        let contract = BackboneProtoContract {
            source: "syntax = \"proto3\"; package oya.community.v1; service PostStoreService { rpc CreatePost (CreatePostRequest) returns (CreatePostResponse); } service VotingEngineService { rpc CastVote (CastVoteRequest) returns (CastVoteResponse); } service ModerationQueueService { rpc ApplyAction (ApplyActionRequest) returns (ApplyActionResponse); } message CreatePostRequest { required string tenant_scope_ref = 1; } message CreatePostResponse {} message CastVoteRequest {} message CastVoteResponse {} message ApplyActionRequest {} message ApplyActionResponse {} principal_ref idempotency_key policy_decision_ref audit_correlation_id",
            ..COMMUNITY_PROTO_CONTRACT
        };

        assert_eq!(
            validate_proto_contract(&contract),
            Err(BackboneProtoContractError::ForbiddenKeyword {
                path: COMMUNITY_PROTO_CONTRACT.path,
                keyword: "required",
            })
        );
    }
}
