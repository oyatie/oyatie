//! The `ResourceService` API surface (COSI resource access).
//!
//! Mirrors `pkg/machinery/api/resource/resource.proto` (the modern
//! `cosi.resource.State` surface used by `talosctl get`): `Get`, `List`, and
//! `Watch` over the COSI resource store. Backed by an in-memory store keyed by
//! [`os_kernel::ResourcePointer`].

use std::collections::BTreeMap;

use os_kernel::ResourceId;
use os_kernel::resource::{Metadata, Namespace, Phase, ResourceKind, ResourcePointer};
use os_kernel::role::Role;

use crate::common::{ApiError, Code, RequestContext};

/// A COSI resource as returned over the API: its metadata plus an opaque,
/// already-encoded spec payload (the proto `spec` field is a YAML/JSON blob).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    /// The resource metadata (pointer, version, phase, owner, ...).
    pub metadata: Metadata,
    /// The encoded spec bytes.
    pub spec: Vec<u8>,
}

impl Resource {
    /// Construct a resource from metadata and spec bytes.
    pub fn new(metadata: Metadata, spec: impl Into<Vec<u8>>) -> Self {
        Resource {
            metadata,
            spec: spec.into(),
        }
    }

    /// The resource's pointer.
    pub fn pointer(&self) -> &ResourcePointer {
        self.metadata.pointer()
    }
}

/// A `Get` request: a fully-qualified pointer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetRequest {
    /// The target namespace.
    pub namespace: Namespace,
    /// The resource type.
    pub kind: ResourceKind,
    /// The resource id.
    pub id: ResourceId,
}

/// A `List` request: a namespace+kind, with optional label-selector filtering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListRequest {
    /// The target namespace.
    pub namespace: Namespace,
    /// The resource type.
    pub kind: ResourceKind,
    /// Optional label selector key=value pairs that must all match.
    pub label_selector: BTreeMap<String, String>,
}

impl ListRequest {
    /// A bare list with no selector.
    pub fn new(namespace: Namespace, kind: ResourceKind) -> Self {
        ListRequest {
            namespace,
            kind,
            label_selector: BTreeMap::new(),
        }
    }
}

/// The kind of change a `Watch` event represents, mirroring COSI
/// `state.EventType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// Initial snapshot entry / resource created.
    Created,
    /// Resource spec or metadata changed.
    Updated,
    /// Resource destroyed.
    Destroyed,
    /// End-of-snapshot bookmark (the "bootstrapped" marker).
    Bootstrapped,
}

/// A single `Watch` event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchEvent {
    /// The kind of change.
    pub event_type: EventType,
    /// The resource (absent for the `Bootstrapped` bookmark).
    pub resource: Option<Resource>,
}

/// The in-memory COSI resource store backing `ResourceService`.
///
/// Models the parts of `cosi-runtime`'s in-memory state the API touches:
/// create/update/destroy with version bookkeeping and a watch event log.
#[derive(Debug, Default)]
pub struct ResourceStore {
    resources: BTreeMap<ResourcePointer, Resource>,
    events: Vec<WatchEvent>,
}

impl ResourceStore {
    /// An empty store.
    pub fn new() -> Self {
        ResourceStore::default()
    }

    /// Create a resource, erroring if its pointer already exists.
    pub fn create(&mut self, resource: Resource) -> Result<(), ApiError> {
        let ptr = resource.pointer().clone();
        if self.resources.contains_key(&ptr) {
            return Err(ApiError::new(
                Code::AlreadyExists,
                format!("{ptr} already exists"),
            ));
        }
        self.events.push(WatchEvent {
            event_type: EventType::Created,
            resource: Some(resource.clone()),
        });
        self.resources.insert(ptr, resource);
        Ok(())
    }

    /// Update an existing resource's spec, bumping its version.
    pub fn update(
        &mut self,
        ptr: &ResourcePointer,
        spec: impl Into<Vec<u8>>,
    ) -> Result<(), ApiError> {
        let res = self
            .resources
            .get_mut(ptr)
            .ok_or_else(|| ApiError::new(Code::NotFound, format!("{ptr} not found")))?;
        res.metadata.bump_version();
        res.spec = spec.into();
        self.events.push(WatchEvent {
            event_type: EventType::Updated,
            resource: Some(res.clone()),
        });
        Ok(())
    }

    /// Destroy a resource.
    pub fn destroy(&mut self, ptr: &ResourcePointer) -> Result<(), ApiError> {
        let res = self
            .resources
            .remove(ptr)
            .ok_or_else(|| ApiError::new(Code::NotFound, format!("{ptr} not found")))?;
        self.events.push(WatchEvent {
            event_type: EventType::Destroyed,
            resource: Some(res),
        });
        Ok(())
    }

    /// The recorded watch-event log.
    pub fn events(&self) -> &[WatchEvent] {
        &self.events
    }

    /// Number of stored resources.
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
}

/// The `ResourceService`. Read-only over the wire (gets/lists/watches), gated by
/// the reader role.
pub struct ResourceService {
    store: ResourceStore,
}

impl ResourceService {
    /// Wrap a store.
    pub fn new(store: ResourceStore) -> Self {
        ResourceService { store }
    }

    /// Read access to the underlying store.
    pub fn store(&self) -> &ResourceStore {
        &self.store
    }

    /// Mutable access to the underlying store (for the controllers that
    /// populate it; not part of the read-only wire surface).
    pub fn store_mut(&mut self) -> &mut ResourceStore {
        &mut self.store
    }

    /// `Get` a single resource by pointer.
    pub fn get(&self, ctx: &RequestContext, req: &GetRequest) -> Result<Resource, ApiError> {
        ctx.authorize(Role::Reader)?;
        let ptr = ResourcePointer::new(req.namespace.clone(), req.kind.clone(), req.id.clone());
        self.store
            .resources
            .get(&ptr)
            .cloned()
            .ok_or_else(|| ApiError::new(Code::NotFound, format!("{ptr} not found")))
    }

    /// `List` resources in a namespace/kind, applying any label selector and
    /// returning them in stable pointer order.
    pub fn list(&self, ctx: &RequestContext, req: &ListRequest) -> Result<Vec<Resource>, ApiError> {
        ctx.authorize(Role::Reader)?;
        let mut out: Vec<Resource> = self
            .store
            .resources
            .values()
            .filter(|r| r.pointer().namespace == req.namespace && r.pointer().kind == req.kind)
            .filter(|r| self.matches_selector(r, req))
            .cloned()
            .collect();
        out.sort_by(|a, b| a.pointer().id.as_str().cmp(b.pointer().id.as_str()));
        Ok(out)
    }

    fn matches_selector(&self, r: &Resource, req: &ListRequest) -> bool {
        req.label_selector
            .iter()
            .all(|(k, v)| r.metadata.labels().get(k) == Some(v.as_str()))
    }

    /// `Watch`: produce a snapshot of the namespace/kind followed by a
    /// `Bootstrapped` bookmark, mirroring COSI's initial-state semantics.
    pub fn watch(
        &self,
        ctx: &RequestContext,
        req: &ListRequest,
    ) -> Result<Vec<WatchEvent>, ApiError> {
        ctx.authorize(Role::Reader)?;
        let mut events: Vec<WatchEvent> = self
            .list(ctx, req)?
            .into_iter()
            .map(|r| WatchEvent {
                event_type: EventType::Created,
                resource: Some(r),
            })
            .collect();
        events.push(WatchEvent {
            event_type: EventType::Bootstrapped,
            resource: None,
        });
        Ok(events)
    }
}

/// Helper to build a [`Resource`] for the `runtime` namespace with the given
/// kind, id, labels, and spec — convenient for seeding stores in tests and
/// controller code.
pub fn runtime_resource(
    kind: &str,
    id: &str,
    labels: &[(&str, &str)],
    spec: impl Into<Vec<u8>>,
) -> Result<Resource, ApiError> {
    let mut meta = Metadata::new(
        Namespace::runtime(),
        ResourceKind::new(kind).map_err(ApiError::from)?,
        ResourceId::new(id).map_err(ApiError::from)?,
    );
    for (k, v) in labels {
        meta.labels_mut().insert(*k, *v).map_err(ApiError::from)?;
    }
    // Default phase is Running; surface it via debug assertion of invariant.
    debug_assert_eq!(meta.phase(), Phase::Running);
    Ok(Resource::new(meta, spec))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_with_two() -> ResourceStore {
        let mut store = ResourceStore::new();
        store
            .create(
                runtime_resource(
                    "Hostnames.net.talos.dev",
                    "hostname",
                    &[("zone", "a")],
                    b"node-1".to_vec(),
                )
                .unwrap(),
            )
            .unwrap();
        store
            .create(
                runtime_resource(
                    "Hostnames.net.talos.dev",
                    "alias",
                    &[("zone", "b")],
                    b"node-1-alias".to_vec(),
                )
                .unwrap(),
            )
            .unwrap();
        store
    }

    fn list_req() -> ListRequest {
        ListRequest::new(
            Namespace::runtime(),
            ResourceKind::new("Hostnames.net.talos.dev").unwrap(),
        )
    }

    #[test]
    fn create_is_idempotent_guarded() {
        let mut store = ResourceStore::new();
        let r = runtime_resource("Foo", "a", &[], b"x".to_vec()).unwrap();
        store.create(r.clone()).unwrap();
        assert_eq!(store.create(r).unwrap_err().code, Code::AlreadyExists);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn update_bumps_version_and_logs_event() {
        let mut store = ResourceStore::new();
        let r = runtime_resource("Foo", "a", &[], b"v1".to_vec()).unwrap();
        let ptr = r.pointer().clone();
        store.create(r).unwrap();
        store.update(&ptr, b"v2".to_vec()).unwrap();

        let svc = ResourceService::new(store);
        let got = svc
            .get(
                &RequestContext::admin_local(),
                &GetRequest {
                    namespace: Namespace::runtime(),
                    kind: ResourceKind::new("Foo").unwrap(),
                    id: ResourceId::new("a").unwrap(),
                },
            )
            .unwrap();
        assert_eq!(got.spec, b"v2");
        assert_eq!(got.metadata.version(), 2);

        let events = svc.store().events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, EventType::Created);
        assert_eq!(events[1].event_type, EventType::Updated);
    }

    #[test]
    fn get_missing_is_not_found() {
        let svc = ResourceService::new(ResourceStore::new());
        let err = svc
            .get(
                &RequestContext::admin_local(),
                &GetRequest {
                    namespace: Namespace::runtime(),
                    kind: ResourceKind::new("Foo").unwrap(),
                    id: ResourceId::new("a").unwrap(),
                },
            )
            .unwrap_err();
        assert_eq!(err.code, Code::NotFound);
    }

    #[test]
    fn list_is_sorted_and_filtered() {
        let svc = ResourceService::new(store_with_two());
        let all = svc
            .list(&RequestContext::admin_local(), &list_req())
            .unwrap();
        // Sorted by id: "alias" before "hostname".
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].pointer().id.as_str(), "alias");
        assert_eq!(all[1].pointer().id.as_str(), "hostname");

        let mut req = list_req();
        req.label_selector.insert("zone".into(), "a".into());
        let filtered = svc.list(&RequestContext::admin_local(), &req).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].pointer().id.as_str(), "hostname");
    }

    #[test]
    fn watch_snapshot_ends_with_bootstrapped() {
        let svc = ResourceService::new(store_with_two());
        let events = svc
            .watch(&RequestContext::admin_local(), &list_req())
            .unwrap();
        assert_eq!(events.len(), 3);
        assert!(
            events[..2]
                .iter()
                .all(|e| e.event_type == EventType::Created)
        );
        assert_eq!(events[2].event_type, EventType::Bootstrapped);
        assert!(events[2].resource.is_none());
    }

    #[test]
    fn read_endpoints_require_read_role() {
        let svc = ResourceService::new(store_with_two());
        let nobody = RequestContext::with_roles(os_kernel::role::RoleSet::new());
        assert_eq!(
            svc.list(&nobody, &list_req()).unwrap_err().code,
            Code::PermissionDenied
        );
    }

    #[test]
    fn destroy_removes_and_logs() {
        let mut store = store_with_two();
        let ptr = ResourcePointer::new(
            Namespace::runtime(),
            ResourceKind::new("Hostnames.net.talos.dev").unwrap(),
            ResourceId::new("alias").unwrap(),
        );
        store.destroy(&ptr).unwrap();
        assert_eq!(store.len(), 1);
        assert_eq!(
            store.events().last().unwrap().event_type,
            EventType::Destroyed
        );
        assert_eq!(store.destroy(&ptr).unwrap_err().code, Code::NotFound);
    }
}
