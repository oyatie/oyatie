/// Product-owned intent required when a candidate changes the declared Rust floor.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DeclaredMsrvChangeIntentV1 {
    product_owner: Box<str>,
    current_msrv: RustVersionV1,
    proposed_msrv: RustVersionV1,
    semantic_intent_sha256: DigestV1,
    postconditions_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl DeclaredMsrvChangeIntentV1 {
    pub fn try_new(
        product_owner: impl Into<String>,
        current_msrv: RustVersionV1,
        proposed_msrv: RustVersionV1,
        semantic_intent_sha256: DigestV1,
        postconditions_sha256: DigestV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if current_msrv == proposed_msrv {
            return Err(toolchain_intent_mismatch());
        }
        let product_owner = lifecycle_identity(product_owner.into())?;
        let mut hash = CanonicalHasherV1::new(b"build.declared-msrv-change-intent.v1\0");
        lifecycle_hash_string(&mut hash, &product_owner)?;
        current_msrv.encode(&mut hash);
        proposed_msrv.encode(&mut hash);
        hash.digest(semantic_intent_sha256);
        hash.digest(postconditions_sha256);
        Ok(Self {
            product_owner,
            current_msrv,
            proposed_msrv,
            semantic_intent_sha256,
            postconditions_sha256,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub fn product_owner(&self) -> &str {
        &self.product_owner
    }

    #[must_use]
    pub const fn current_msrv(&self) -> RustVersionV1 {
        self.current_msrv
    }

    #[must_use]
    pub const fn proposed_msrv(&self) -> RustVersionV1 {
        self.proposed_msrv
    }

    #[must_use]
    pub const fn semantic_intent_sha256(&self) -> DigestV1 {
        self.semantic_intent_sha256
    }

    #[must_use]
    pub const fn postconditions_sha256(&self) -> DigestV1 {
        self.postconditions_sha256
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Declared-MSRV effect computed from exact current and proposed profiles.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ToolchainMsrvEffectV1 {
    Unchanged,
    QualificationRefresh,
    FloorChange { intent: DeclaredMsrvChangeIntentV1 },
}

impl ToolchainMsrvEffectV1 {
    fn encode(&self, hash: &mut CanonicalHasherV1) {
        match self {
            Self::Unchanged => hash.tag(0),
            Self::QualificationRefresh => hash.tag(1),
            Self::FloorChange { intent } => {
                hash.tag(2);
                hash.digest(intent.identity_sha256());
            }
        }
    }
}

/// Nonbinding transition between two exact, admitted toolchain matrices.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainCandidateV1 {
    current: ToolchainMatrixV1,
    proposed: ToolchainMatrixV1,
    delta: ToolchainCandidateDeltaV1,
    msrv_effect: ToolchainMsrvEffectV1,
    discovery_receipt_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl ToolchainCandidateV1 {
    pub fn try_new(
        current: ToolchainMatrixV1,
        proposed: ToolchainMatrixV1,
        msrv_intent: Option<DeclaredMsrvChangeIntentV1>,
        discovery_receipt_sha256: DigestV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if matrix_host(&current) != matrix_host(&proposed) {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::ToolchainTargetMismatch,
            ));
        }
        for (current_profile, proposed_profile) in [
            (current.stable(), proposed.stable()),
            (current.beta(), proposed.beta()),
            (current.nightly(), proposed.nightly()),
        ] {
            if proposed_profile.version() < current_profile.version() {
                return Err(LifecycleFailureV1::new(
                    LifecycleFailureClassV1::UnsupportedVersionRelation,
                ));
            }
        }

        let delta = ToolchainCandidateDeltaV1::between(&current, &proposed);
        if delta.changed_roles().is_empty() {
            return Err(invalid_toolchain_candidate());
        }
        let msrv_effect = msrv_effect(&current, &proposed, msrv_intent)?;

        let mut hash = CanonicalHasherV1::new(b"build.toolchain-candidate.v1\0");
        hash.digest(current.identity_sha256());
        hash.digest(proposed.identity_sha256());
        delta.encode(&mut hash)?;
        msrv_effect.encode(&mut hash);
        hash.digest(discovery_receipt_sha256);
        Ok(Self {
            current,
            proposed,
            delta,
            msrv_effect,
            discovery_receipt_sha256,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn current(&self) -> &ToolchainMatrixV1 {
        &self.current
    }

    #[must_use]
    pub const fn proposed(&self) -> &ToolchainMatrixV1 {
        &self.proposed
    }

    #[must_use]
    pub fn changed_roles(&self) -> &[ToolchainRoleV1] {
        self.delta.changed_roles()
    }

    #[must_use]
    pub const fn delta(&self) -> &ToolchainCandidateDeltaV1 {
        &self.delta
    }

    #[must_use]
    pub const fn msrv_effect(&self) -> &ToolchainMsrvEffectV1 {
        &self.msrv_effect
    }

    #[must_use]
    pub const fn discovery_receipt_sha256(&self) -> DigestV1 {
        self.discovery_receipt_sha256
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

fn msrv_effect(
    current: &ToolchainMatrixV1,
    proposed: &ToolchainMatrixV1,
    intent: Option<DeclaredMsrvChangeIntentV1>,
) -> Result<ToolchainMsrvEffectV1, LifecycleFailureV1> {
    let current_msrv = current.msrv();
    let proposed_msrv = proposed.msrv();
    if current_msrv.version() != proposed_msrv.version() {
        let intent = intent.ok_or_else(|| {
            LifecycleFailureV1::new(LifecycleFailureClassV1::MissingToolchainIntent)
        })?;
        if intent.current_msrv() != current_msrv.version()
            || intent.proposed_msrv() != proposed_msrv.version()
        {
            return Err(toolchain_intent_mismatch());
        }
        return Ok(ToolchainMsrvEffectV1::FloorChange { intent });
    }
    if intent.is_some() {
        return Err(invalid_toolchain_candidate());
    }
    if current_msrv.identity_sha256() == proposed_msrv.identity_sha256() {
        Ok(ToolchainMsrvEffectV1::Unchanged)
    } else {
        Ok(ToolchainMsrvEffectV1::QualificationRefresh)
    }
}

fn matrix_host(matrix: &ToolchainMatrixV1) -> &str {
    matrix.msrv().tools().rustc().host_triple()
}

const fn invalid_toolchain_candidate() -> LifecycleFailureV1 {
    LifecycleFailureV1::new(LifecycleFailureClassV1::InvalidToolchainCandidate)
}

const fn toolchain_intent_mismatch() -> LifecycleFailureV1 {
    LifecycleFailureV1::new(LifecycleFailureClassV1::ToolchainIntentMismatch)
}
