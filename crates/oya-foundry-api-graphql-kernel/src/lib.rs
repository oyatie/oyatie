//! M02-P04-IP-001 — GraphQL transport kernel.
//!
//! Schema-shape types + `UseCaseRequest`/`UseCaseResponse` projection.
//! Reuses canonical `AuditEvent` from the REST kernel.

use std::collections::BTreeMap;

use oya_foundry_api_rest_kernel::{ResponseStatus, UseCaseRequest, UseCaseResponse};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphqlField {
    pub name: String, // data_class: INTERNAL_ONLY
    pub ty: String,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphqlType {
    pub name: String,              // data_class: INTERNAL_ONLY
    pub fields: Vec<GraphqlField>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphqlSchema {
    pub types: Vec<GraphqlType>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphqlRequest {
    pub operation: String,                   // data_class: INTERNAL_ONLY
    pub use_case_id: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub variables: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
}

impl UseCaseRequest for GraphqlRequest {
    fn use_case_id(&self) -> &str {
        &self.use_case_id
    }
    fn tenant_id(&self) -> &str {
        &self.tenant_id
    }
    fn payload(&self) -> &BTreeMap<String, String> {
        &self.variables
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphqlResponse {
    pub use_case_id: String,            // data_class: INTERNAL_ONLY
    pub status: ResponseStatus,         // data_class: INTERNAL_ONLY
    pub data: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
}

impl UseCaseResponse for GraphqlResponse {
    fn use_case_id(&self) -> &str {
        &self.use_case_id
    }
    fn status(&self) -> ResponseStatus {
        self.status
    }
    fn body(&self) -> &BTreeMap<String, String> {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_round_trip() {
        let schema = GraphqlSchema {
            types: vec![GraphqlType {
                name: "Account".into(),
                fields: vec![GraphqlField {
                    name: "id".into(),
                    ty: "ID".into(),
                }],
            }],
        };
        assert_eq!(schema.types.len(), 1);
        assert_eq!(schema.types[0].fields[0].name, "id");
    }

    #[test]
    fn graphql_request_projects_use_case_request() {
        let req = GraphqlRequest {
            operation: "query Account { account { id } }".into(),
            use_case_id: "foundry.account.view".into(),
            tenant_id: "tenant-alpha".into(),
            variables: BTreeMap::new(),
        };
        assert_eq!(req.use_case_id(), "foundry.account.view");
    }

    #[test]
    fn graphql_response_status() {
        let res = GraphqlResponse {
            use_case_id: "foundry.account.view".into(),
            status: ResponseStatus::Ok,
            data: BTreeMap::new(),
        };
        assert_eq!(res.status(), ResponseStatus::Ok);
    }
}
