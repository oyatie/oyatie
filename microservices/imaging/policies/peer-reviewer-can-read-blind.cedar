// Cedar policy: Peer reviewer can read studies BLINDED to the primary report.
//
// Authority: ADR-0243 Cedar universal gate; RadPeer methodology.
//
// Principal: User with role "peer-reviewer".
// Action: imaging::Action::"ReadBlinded" | "SubmitPeerReview"
// Resource: imaging::Study | imaging::PeerReviewItem
//
// Critical invariant: the reviewer MUST NOT see the primary structured report
// until they submit their own assessment (FR-RAD-011).

permit (
  principal in imaging::Role::"peer-reviewer",
  action == imaging::Action::"ReadBlinded",
  resource is imaging::Study
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.peer_review_item != nil &&
  resource.peer_review_item.assigned_reviewer == principal &&
  resource.peer_review_item.status == "PENDING" &&
  principal.id != resource.signing_radiologist_id
};

permit (
  principal in imaging::Role::"peer-reviewer",
  action == imaging::Action::"SubmitPeerReview",
  resource is imaging::PeerReviewItem
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.assigned_reviewer == principal &&
  resource.status == "PENDING"
};

// Forbid the reviewer from viewing the primary structured report content
// while the peer review is still PENDING.
forbid (
  principal in imaging::Role::"peer-reviewer",
  action == imaging::Action::"ReadStructuredReport",
  resource is imaging::StructuredReport
) when {
  resource.peer_review_item != nil &&
  resource.peer_review_item.assigned_reviewer == principal &&
  resource.peer_review_item.status == "PENDING"
};

// Forbid the reviewer from being the same person as the signing radiologist.
forbid (
  principal,
  action,
  resource is imaging::PeerReviewItem
) when {
  principal.id == resource.study.signing_radiologist_id
};
