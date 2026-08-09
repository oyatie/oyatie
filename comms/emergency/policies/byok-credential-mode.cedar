// Policy: byok-credential-mode
// Authority: ADR-0332 (in flight) | ADR-0243 (Cedar universal gate) | ADR-0255 §D-4 (BYOK)
// Effect: enforce provider_credential_mode at the credential-use boundary.
//         platform_default = OK; byok = require tenant credentials present;
//         byok_required_by_pack = block unless tenant credentials present.

permit (
    principal,
    action == Oyatie::Action::"externalCredentialUse",
    resource is Oyatie::Emergency::ExternalCredential
)
when {
    resource.tenantCredentialMode == "platform_default"
};

permit (
    principal,
    action == Oyatie::Action::"externalCredentialUse",
    resource is Oyatie::Emergency::ExternalCredential
)
when {
    resource.tenantCredentialMode == "byok" &&
    resource.tenantCredentialPresent == true
};

forbid (
    principal,
    action == Oyatie::Action::"externalCredentialUse",
    resource is Oyatie::Emergency::ExternalCredential
)
when {
    resource.tenantCredentialMode == "byok_required_by_pack" &&
    resource.tenantCredentialPresent == false
};
