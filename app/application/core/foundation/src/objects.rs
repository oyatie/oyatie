//! Regional packs, object entities, the outbox, and read accessors.

use crate::*;

use crate::Foundation;

impl Foundation {
    pub fn register_regional_pack(
        &mut self,
        registration: RegionalPackRegistration,
    ) -> Result<RegionalPack, FoundationError> {
        if self.regional_packs.contains_key(&registration.pack_id) {
            return Err(FoundationError::RegionalPackAlreadyExists);
        }
        let pack = RegionalPack::new(
            registration.pack_id,
            registration.region,
            registration.residency_class,
            registration.controls,
        )
        .map_err(map_regional_pack_error)?;
        self.regional_packs.insert(pack.id.clone(), pack.clone());
        self.audit_chain.append_classifications(
            "ten_system",
            "regulatory-pack.bind",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(pack)
    }

    pub fn upsert_object_entity(
        &mut self,
        upsert: ObjectEntityUpsert,
    ) -> Result<ObjectEntity, FoundationError> {
        self.require_tenant(&upsert.tenant_id)?;
        let properties = upsert
            .properties
            .into_iter()
            .map(|input| {
                ObjectProperty::new_with_privacy_data_class(
                    input.name,
                    input.value,
                    input.tier,
                    input.privacy_data_class,
                )
            })
            .collect::<Vec<_>>();
        let entity = ObjectEntity::new(
            upsert.tenant_id.clone(),
            upsert.entity_id,
            upsert.entity_type,
            properties,
        )
        .map_err(map_object_graph_error)?;
        self.object_entities.insert(
            (upsert.tenant_id.clone(), entity.id.clone()),
            entity.clone(),
        );
        self.audit_chain.append_classifications(
            upsert.tenant_id,
            "object-graph.entity.upsert",
            Plane::Data,
            Purpose::CoreService,
            entity
                .properties
                .values()
                .map(|property| property.value.data_class.compatibility_data_class())
                .collect::<Vec<_>>(),
            "ALLOW",
        )?;
        Ok(entity)
    }

    pub fn publish_outbox(
        &mut self,
        publish: OutboxPublish,
    ) -> Result<OutboxRecord, FoundationError> {
        self.require_tenant(&publish.tenant_id)?;
        let record = self
            .outbox
            .publish(
                publish.tenant_id.clone(),
                publish.topic,
                publish.idempotency_key,
                publish.payload_ref,
            )
            .map_err(map_eventing_error)?;
        self.audit_chain.append_classifications(
            publish.tenant_id,
            "eventing.outbox.publish",
            Plane::Data,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(record)
    }

    pub fn mark_outbox_published(
        &mut self,
        tenant_id: &str,
        sequence: u64,
    ) -> Result<OutboxRecord, FoundationError> {
        self.require_tenant(tenant_id)?;
        match self.outbox.mark_published(tenant_id, sequence) {
            Ok(record) => {
                self.audit_chain.append_classifications(
                    tenant_id,
                    "eventing.outbox.mark-published",
                    Plane::Data,
                    Purpose::CoreService,
                    vec![DataClass::InternalOnly],
                    "ALLOW",
                )?;
                Ok(record)
            }
            Err(EventingError::OutboxRecordNotFound) => {
                self.audit_chain.append_classifications(
                    tenant_id,
                    "eventing.outbox.mark-published",
                    Plane::Data,
                    Purpose::CoreService,
                    vec![DataClass::InternalOnly],
                    "DENY",
                )?;
                Err(FoundationError::OutboxRecordNotFound)
            }
            Err(error) => Err(map_eventing_error(error)),
        }
    }

    pub fn outbox_records(&self) -> &[OutboxRecord] {
        self.outbox.records()
    }

    pub fn foundry_runs(&self) -> &[Run] {
        self.foundry_runs.runs()
    }

    pub fn foundry_steps(&self) -> &[Step] {
        self.foundry_steps.steps()
    }

    pub fn foundry_evidence_chain(&self) -> &EvidenceChain {
        &self.foundry_evidence
    }

    pub fn audit_chain(&self) -> &AuditChain {
        &self.audit_chain
    }

    pub(crate) fn require_tenant(&self, tenant_id: &str) -> Result<&Tenant, FoundationError> {
        self.tenants
            .get(tenant_id)
            .ok_or(FoundationError::TenantNotFound)
    }

    pub(crate) fn require_user(
        &self,
        tenant_id: &str,
        user_id: &str,
    ) -> Result<&User, FoundationError> {
        self.require_tenant(tenant_id)?;
        self.users
            .get(&(tenant_id.to_string(), user_id.to_string()))
            .ok_or(FoundationError::UserNotFound)
    }

    pub(crate) fn mcp_principal(
        &self,
        endpoint: &McpTenantEndpoint,
        access_token: McpAccessTokenClaims,
        now_epoch_seconds: u64,
    ) -> Result<McpPrincipal, FoundationError> {
        let policy = self
            .tenant_policies
            .get(&endpoint.tenant_id.value)
            .ok_or(FoundationError::TenantNotFound)?;
        validate_access_token(
            endpoint,
            access_token,
            now_epoch_seconds,
            policy.autonomy_ceiling,
        )
        .map_err(map_mcp_error)
    }
}
