// Cedar v4.2 LTS — Notes PCI payments-adjacent overlay
// Authority: ADR-0251 compliance pack primitive (PCI-DSS)

forbid (
  principal,
  action == Action::"NotesNoteSave",
  resource
) when {
  // Reject save if note body contains cardholder data per PCI-DSS scope reduction
  resource.tenant.compliance_packs.contains("pci")
  && action has pan_detected
  && action.pan_detected == true
  && !(action has pan_redacted && action.pan_redacted == true)
};

forbid (
  principal,
  action == Action::"NotesShareLinkIssue",
  resource
) when {
  // PCI scope: notes carrying cardholder data refs cannot be shared externally
  resource.tenant.compliance_packs.contains("pci")
  && resource has pci_scope_in
  && resource.pci_scope_in == true
  && action has share_audience
  && action.share_audience in ["PUBLIC_SHARE_VIEW", "EXTERNAL_PARTNER"]
};
