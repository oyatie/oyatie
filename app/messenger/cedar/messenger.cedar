// Deny by default: this seed has NO static permits and NO linked grants.
// Bundle assembly for Oyatie Policy:
// - schema_src = messenger.cedarschema.
// - policies_src = the static forbids below (preserve their @id annotations).
// - templates = each unlinked permit below, with template_id = its @id value.
// - template_links = explicitly approved principal + room grants, one action
//   per link. Grant IDs must be unique; removing a link revokes that grant.
// IMPORTANT: Oyatie's admission copies parsed.policies(), not templates, from
// policies_src. Supplying this whole file only as policies_src grants nothing;
// populate bundle.templates separately. No obligations are emitted by this seed.
//
// Exact bundle.action_map (slug -> Cedar EntityUid string):
// messenger.room.create     -> OyatieMessenger::Action::"CreateRoom"
// messenger.room.manage     -> OyatieMessenger::Action::"ManageRoom"
// messenger.message.send    -> OyatieMessenger::Action::"Send"
// messenger.room.invite     -> OyatieMessenger::Action::"Invite"
// messenger.archive.capture -> OyatieMessenger::Action::"Archive"
// messenger.object.read     -> OyatieMessenger::Action::"ReadObject"
// messenger.object.invoke   -> OyatieMessenger::Action::"InvokeAction"
//
// TemplateLink principal.entity_type = "OyatieMessenger::Principal";
// principal.entity_id = JSON.stringify([tenant, authenticated_subject]).
// resource.entity_type = "OyatieMessenger::Room";
// resource.entity_id = JSON.stringify([tenant, exact_resource_argument]).
// For room governance and archival, exact_resource_argument is the immutable policy resource. Matrix
// transport ID is not the grant key: use the immutable dev.oyatie.policy_resource
// chosen from preauthorized business resources when provisioning the room.
// ReadObject/InvokeAction are declared for adapter compatibility; this seed
// does not replace Foundry's separate object authorization or canonical writer.

// Same invariant and annotation as the canonical Oyatie platform seed.
@id("structural-tenant-isolation")
forbid (principal, action, resource)
when {
    principal has tenant_id &&
    resource has tenant_id &&
    principal.tenant_id != resource.tenant_id
};

@id("messenger-request-tenant-isolation")
forbid (principal, action, resource)
when {
    principal.tenant_id != context.caller_tenant ||
    resource.tenant_id != context.caller_tenant
};

@id("messenger-room-create-grant")
permit (
    principal == ?principal,
    action == OyatieMessenger::Action::"CreateRoom",
    resource == ?resource
)
when { principal.tenant_id == resource.tenant_id };

@id("messenger-room-manage-grant")
permit (
    principal == ?principal,
    action == OyatieMessenger::Action::"ManageRoom",
    resource == ?resource
)
when { principal.tenant_id == resource.tenant_id };

@id("messenger-message-send-grant")
permit (
    principal == ?principal,
    action == OyatieMessenger::Action::"Send",
    resource == ?resource
)
when { principal.tenant_id == resource.tenant_id };

@id("messenger-room-invite-grant")
permit (
    principal == ?principal,
    action == OyatieMessenger::Action::"Invite",
    resource == ?resource
)
when { principal.tenant_id == resource.tenant_id };

@id("messenger-archive-capture-grant")
permit (
    principal == ?principal,
    action == OyatieMessenger::Action::"Archive",
    resource == ?resource
)
when { principal.tenant_id == resource.tenant_id };

@id("messenger-object-read-grant")
permit (
    principal == ?principal,
    action == OyatieMessenger::Action::"ReadObject",
    resource == ?resource
)
when { principal.tenant_id == resource.tenant_id };

@id("messenger-object-invoke-grant")
permit (
    principal == ?principal,
    action == OyatieMessenger::Action::"InvokeAction",
    resource == ?resource
)
when { principal.tenant_id == resource.tenant_id };
