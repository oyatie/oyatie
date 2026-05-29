// identity µservice — operator-recovery policy
// Authority: ADR-0189 step-up ACR + runbooks/passkey-reset.md operator flow.
//
// Allows ops-security operators to perform mediated account-recovery actions
// after out-of-band identity-proofing. Every grant emits a sealed audit event.

// =============================================================================
// Operator recovery actions (each must include an open ticket reference + acr=critical).
// =============================================================================

permit (
  principal,
  action in [
    IdentityAction::"OperatorRecoveryRevokeCredential",
    IdentityAction::"OperatorRecoveryPinAcr",
    IdentityAction::"OperatorRecoveryForceReauth",
    IdentityAction::"OperatorRecoveryProvisionOneTimeLink"
  ],
  resource is User
) when {
  principal.role == "ops-security" &&
  principal.acr_level == AcrLevel::"critical" &&
  context.operator_ticket_id != "" &&
  context.operator_ticket_state == "open" &&
  context.operator_id_proof_verified == true &&
  context.it_approval_token_present == true
};

// =============================================================================
// Two-operator-required rule for mass operations.
// =============================================================================

permit (
  principal,
  action in [
    IdentityAction::"OperatorMassPasskeyReset",
    IdentityAction::"OperatorMassCredentialRevoke"
  ],
  resource
) when {
  principal.role == "ops-security" &&
  principal.acr_level == AcrLevel::"critical" &&
  context.second_operator_approval_present == true &&
  context.second_operator_id != principal.id &&
  context.operator_ticket_state == "open"
};

// =============================================================================
// Default deny for everything operator-class not explicitly permitted.
// =============================================================================

forbid (
  principal,
  action in [
    IdentityAction::"OperatorRecoveryRevokeCredential",
    IdentityAction::"OperatorRecoveryPinAcr",
    IdentityAction::"OperatorRecoveryForceReauth",
    IdentityAction::"OperatorRecoveryProvisionOneTimeLink",
    IdentityAction::"OperatorMassPasskeyReset",
    IdentityAction::"OperatorMassCredentialRevoke"
  ],
  resource
) when {
  principal.role != "ops-security"
};
