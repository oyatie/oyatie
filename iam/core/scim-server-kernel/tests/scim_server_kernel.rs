#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Integration tests for `shared-scim-server-kernel`.

use iam_scim_server_kernel::{
    CounterIdGen, Email, FilterExpr, Group, GroupMembership, InMemoryGroupStore, InMemoryUserStore,
    ListQuery, NewGroup, NewUser, OyatieExtension, PatchOp, PatchOpKind, PatchOperation,
    ReferenceScimServer, ScimError, ScimId, ScimServer, ScimStoreError, ScimType, TenantId, User,
    UserName, UserStore, parse_filter,
};
use serde_json::json;

fn srv() -> ReferenceScimServer<InMemoryUserStore, InMemoryGroupStore, CounterIdGen> {
    ReferenceScimServer::new(
        InMemoryUserStore::default(),
        InMemoryGroupStore::default(),
        CounterIdGen::new(),
        "https://identity.oyatie.com/scim/v2",
    )
}

fn tenant() -> TenantId {
    TenantId("tenant-acme".into())
}

fn new_user(name: &str) -> NewUser {
    NewUser {
        user_name: name.into(),
        external_id: Some(format!("ext-{name}")),
        name: Some(UserName {
            given_name: Some("First".into()),
            family_name: Some("Last".into()),
            ..Default::default()
        }),
        display_name: Some(name.into()),
        active: true,
        emails: vec![Email {
            value: format!("{name}@example.com"),
            r#type: Some("work".into()),
            primary: Some(true),
            display: None,
        }],
        enterprise: None,
        oyatie: None,
    }
}

#[path = "scim_server_kernel/filters.rs"]
mod filters;
#[path = "scim_server_kernel/groups.rs"]
mod groups;
#[path = "scim_server_kernel/native_authority.rs"]
mod native_authority;
#[path = "scim_server_kernel/users.rs"]
mod users;
#[path = "scim_server_kernel/wire.rs"]
mod wire;
