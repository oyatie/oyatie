// Policy: ama-disposition
// Authority: ADR-0332 (in flight) | ADR-0243 (Cedar universal gate)
// Effect: AMA disposition requires signed AMA form + provider attestation
//         + patient acknowledgement.

permit (
    principal in Oyatie::Role::"attending",
    action == Oyatie::Action::"amaDispositionFinalize",
    resource is Oyatie::Emergency::Disposition
)
when {
    resource.outcome == "ama" &&
    resource.amaFormSigned == true &&
    resource.providerAttestationProvided == true &&
    resource.patientAcknowledgementProvided == true
};

forbid (
    principal,
    action == Oyatie::Action::"amaDispositionFinalize",
    resource
)
unless {
    resource.amaFormSigned == true &&
    resource.providerAttestationProvided == true &&
    resource.patientAcknowledgementProvided == true
};
