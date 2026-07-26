//! The COSI `resource` gRPC API modeled as a Rust trait + method enum.
//!
//! Talos exposes its runtime state (COSI resources) through a read-mostly gRPC
//! service that `talosctl get` talks to. `apid` proxies it the same way it does
//! `MachineService`.

use crate::error::ApiError;
use crate::request::Request;
use crate::response::Response;

/// One RPC of the COSI `resource.ResourceService` / `state.State` API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceMethod {
    /// `Get` — fetch a single resource by namespace/type/id. Read-only.
    Get,
    /// `List` — list resources of a type within a namespace. Read-only.
    List,
    /// `Watch` — stream resource change events. Read-only, streaming.
    Watch,
}

impl ResourceMethod {
    /// The fully-qualified gRPC method name.
    pub fn grpc_name(self) -> &'static str {
        match self {
            ResourceMethod::Get => "/cosi.resource.State/Get",
            ResourceMethod::List => "/cosi.resource.State/List",
            ResourceMethod::Watch => "/cosi.resource.State/Watch",
        }
    }

    /// The short method name.
    pub fn short_name(self) -> &'static str {
        self.grpc_name().rsplit('/').next().unwrap_or("")
    }

    /// Whether the method is server-streaming.
    pub fn is_streaming(self) -> bool {
        matches!(self, ResourceMethod::List | ResourceMethod::Watch)
    }

    /// The resource API is read-only; mutations go through machine config.
    pub fn is_mutating(self) -> bool {
        false
    }

    /// Parse a method from its fully-qualified or short name.
    pub fn parse(name: &str) -> Result<Self, ApiError> {
        let short = name.rsplit('/').next().unwrap_or(name);
        match short {
            "Get" => Ok(ResourceMethod::Get),
            "List" => Ok(ResourceMethod::List),
            "Watch" => Ok(ResourceMethod::Watch),
            other => Err(ApiError::unimplemented(format!(
                "unknown resource method '{other}'"
            ))),
        }
    }
}

/// A single addressed COSI resource fetch: `namespace/type/id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceRef {
    /// The COSI namespace (e.g. `runtime`, `network`, `k8s`).
    pub namespace: String,
    /// The resource type (e.g. `MachineStatus`, `NodeAddress`).
    pub typ: String,
    /// The resource id within the type; empty for a `List`.
    pub id: String,
}

impl ResourceRef {
    /// Construct and validate a reference. Namespace and type are required.
    pub fn new(
        namespace: impl Into<String>,
        typ: impl Into<String>,
        id: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let namespace = namespace.into();
        let typ = typ.into();
        if namespace.is_empty() {
            return Err(ApiError::invalid("resource namespace is empty"));
        }
        if typ.is_empty() {
            return Err(ApiError::invalid("resource type is empty"));
        }
        Ok(ResourceRef {
            namespace,
            typ,
            id: id.into(),
        })
    }

    /// Whether this reference addresses a single resource (has an id).
    pub fn is_singular(&self) -> bool {
        !self.id.is_empty()
    }

    /// Parse a `namespace/type[/id]` reference string (the addressing form
    /// `talosctl get` builds before calling the resource API). A missing id is
    /// allowed (yields a list reference); a missing namespace or type is an
    /// error.
    pub fn parse(s: &str) -> Result<Self, ApiError> {
        let mut parts = s.splitn(3, '/');
        let namespace = parts.next().unwrap_or("");
        let typ = parts.next().unwrap_or("");
        let id = parts.next().unwrap_or("");
        ResourceRef::new(namespace, typ, id)
    }

    /// The canonical `namespace/type/id` string (id omitted when empty).
    pub fn to_path(&self) -> String {
        if self.id.is_empty() {
            format!("{}/{}", self.namespace, self.typ)
        } else {
            format!("{}/{}/{}", self.namespace, self.typ, self.id)
        }
    }
}

/// An in-memory COSI resource store implementing [`ResourceService`].
///
/// Resources are keyed by their `namespace/type/id` path. `Get` returns the
/// single resource (or [`ApiError::NotFound`]); `List`/`Watch` return every
/// resource whose namespace+type match, in id order. This mirrors how the COSI
/// state backend answers the resource gRPC API that apid proxies.
#[derive(Debug, Clone, Default)]
pub struct ResourceStore {
    items: std::collections::BTreeMap<String, String>,
}

impl ResourceStore {
    /// An empty store.
    pub fn new() -> Self {
        ResourceStore {
            items: std::collections::BTreeMap::new(),
        }
    }

    /// Insert or replace a resource at `ref`'s path with `body`.
    pub fn put(&mut self, r: &ResourceRef, body: impl Into<String>) {
        self.items.insert(r.to_path(), body.into());
    }

    /// Remove a resource by reference, returning whether it existed.
    pub fn delete(&mut self, r: &ResourceRef) -> bool {
        self.items.remove(&r.to_path()).is_some()
    }

    /// Number of stored resources.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn prefix(r: &ResourceRef) -> String {
        format!("{}/{}/", r.namespace, r.typ)
    }
}

impl ResourceService for ResourceStore {
    fn get(&self, _req: &Request, r: &ResourceRef) -> Result<Response, ApiError> {
        if !r.is_singular() {
            return Err(ApiError::invalid("Get requires a resource id"));
        }
        match self.items.get(&r.to_path()) {
            Some(v) => Ok(Response::ok(v.clone())),
            None => Err(ApiError::NotFound(format!(
                "resource '{}' not found",
                r.to_path()
            ))),
        }
    }

    fn list(&self, _req: &Request, r: &ResourceRef) -> Result<Vec<Response>, ApiError> {
        let prefix = Self::prefix(r);
        let items: Vec<Response> = self
            .items
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(_, v)| Response::ok(v.clone()))
            .collect();
        Ok(items)
    }

    fn type_count(&self) -> usize {
        let mut seen: Vec<String> = Vec::new();
        for key in self.items.keys() {
            // namespace/type/id -> namespace/type
            let mut it = key.splitn(3, '/');
            let ns = it.next().unwrap_or("");
            let ty = it.next().unwrap_or("");
            let nt = format!("{ns}/{ty}");
            if !seen.contains(&nt) {
                seen.push(nt);
            }
        }
        seen.len()
    }
}

/// The COSI resource gRPC service.
pub trait ResourceService {
    /// Fetch a single resource.
    fn get(&self, req: &Request, r: &ResourceRef) -> Result<Response, ApiError>;

    /// List resources of a type within a namespace; one response per item.
    fn list(&self, req: &Request, r: &ResourceRef) -> Result<Vec<Response>, ApiError>;

    /// Number of distinct resource types known to this service.
    fn type_count(&self) -> usize;

    /// Dispatch by method for the router.
    fn dispatch(
        &self,
        method: ResourceMethod,
        req: &Request,
        r: &ResourceRef,
    ) -> Result<Vec<Response>, ApiError> {
        match method {
            ResourceMethod::Get => Ok(vec![self.get(req, r)?]),
            ResourceMethod::List | ResourceMethod::Watch => self.list(req, r),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_flags() {
        assert!(ResourceMethod::List.is_streaming());
        assert!(!ResourceMethod::Get.is_streaming());
        assert!(!ResourceMethod::Watch.is_mutating());
    }

    #[test]
    fn parse_methods() {
        assert_eq!(ResourceMethod::parse("Get").unwrap(), ResourceMethod::Get);
        assert_eq!(
            ResourceMethod::parse("/cosi.resource.State/Watch").unwrap(),
            ResourceMethod::Watch
        );
        assert!(ResourceMethod::parse("Delete").is_err());
    }

    #[test]
    fn resource_ref_validation() {
        let r = ResourceRef::new("runtime", "MachineStatus", "node").unwrap();
        assert!(r.is_singular());
        let listr = ResourceRef::new("network", "NodeAddress", "").unwrap();
        assert!(!listr.is_singular());
        assert!(ResourceRef::new("", "X", "").is_err());
        assert!(ResourceRef::new("ns", "", "").is_err());
    }

    #[test]
    fn resource_ref_parse_and_path() {
        let r = ResourceRef::parse("runtime/MachineStatus/node-1").unwrap();
        assert_eq!(r.namespace, "runtime");
        assert_eq!(r.typ, "MachineStatus");
        assert_eq!(r.id, "node-1");
        assert_eq!(r.to_path(), "runtime/MachineStatus/node-1");

        let list = ResourceRef::parse("network/NodeAddress").unwrap();
        assert!(!list.is_singular());
        assert_eq!(list.to_path(), "network/NodeAddress");

        assert!(ResourceRef::parse("runtime").is_err());
        assert!(ResourceRef::parse("").is_err());
    }

    #[test]
    fn store_get_hit_and_miss() {
        let mut store = ResourceStore::new();
        let r = ResourceRef::parse("runtime/MachineStatus/cp-1").unwrap();
        store.put(&r, "running");
        assert_eq!(store.len(), 1);
        assert_eq!(
            store
                .get(&Request::resource(ResourceMethod::Get), &r)
                .unwrap()
                .body(),
            "running"
        );

        let missing = ResourceRef::parse("runtime/MachineStatus/cp-2").unwrap();
        assert_eq!(
            store
                .get(&Request::resource(ResourceMethod::Get), &missing)
                .unwrap_err()
                .grpc_code(),
            "NotFound"
        );
    }

    #[test]
    fn store_get_requires_id() {
        let store = ResourceStore::new();
        let listref = ResourceRef::parse("runtime/MachineStatus").unwrap();
        assert_eq!(
            store
                .get(&Request::resource(ResourceMethod::Get), &listref)
                .unwrap_err()
                .grpc_code(),
            "InvalidArgument"
        );
    }

    #[test]
    fn store_list_filters_by_namespace_and_type() {
        let mut store = ResourceStore::new();
        store.put(&ResourceRef::parse("runtime/MachineStatus/a").unwrap(), "1");
        store.put(&ResourceRef::parse("runtime/MachineStatus/b").unwrap(), "2");
        store.put(
            &ResourceRef::parse("network/NodeAddress/a").unwrap(),
            "10.0.0.1",
        );
        let lr = ResourceRef::parse("runtime/MachineStatus").unwrap();
        let items = store
            .list(&Request::resource(ResourceMethod::List), &lr)
            .unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(store.type_count(), 2);
    }

    #[test]
    fn store_dispatch_get_and_list() {
        let mut store = ResourceStore::new();
        store.put(&ResourceRef::parse("k8s/Node/cp-1").unwrap(), "Ready");
        let getref = ResourceRef::parse("k8s/Node/cp-1").unwrap();
        let got = store
            .dispatch(
                ResourceMethod::Get,
                &Request::resource(ResourceMethod::Get),
                &getref,
            )
            .unwrap();
        assert_eq!(got[0].body(), "Ready");

        let lr = ResourceRef::parse("k8s/Node").unwrap();
        let listed = store
            .dispatch(
                ResourceMethod::List,
                &Request::resource(ResourceMethod::List),
                &lr,
            )
            .unwrap();
        assert_eq!(listed.len(), 1);
    }

    #[test]
    fn store_delete() {
        let mut store = ResourceStore::new();
        let r = ResourceRef::parse("runtime/MachineStatus/a").unwrap();
        store.put(&r, "x");
        assert!(store.delete(&r));
        assert!(!store.delete(&r));
        assert!(store.is_empty());
    }
}
