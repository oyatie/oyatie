//! Audit-chain kernel: append-only tamper-evident event chain.

use oya_platform_data_boundary_kernel::{DataClass, DataClassification, Purpose};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Plane {
    Control,
    Data,
    Audit,
    Analytics,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEvent {
    pub sequence: u64,
    pub tenant_id: String,
    pub surface: String,
    pub plane: Plane,
    pub purpose: Purpose,
    pub data_classes: Vec<DataClass>,
    pub decision: String,
    pub previous_hash: String,
    pub hash: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AuditChain {
    events: Vec<AuditEvent>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditChainError {
    InvalidChain,
}

impl AuditChain {
    pub fn from_events(events: Vec<AuditEvent>) -> Result<Self, AuditChainError> {
        let chain = Self { events };
        if chain.verify() {
            Ok(chain)
        } else {
            Err(AuditChainError::InvalidChain)
        }
    }

    fn append_legacy_data_classes(
        &mut self,
        tenant_id: impl Into<String>,
        surface: impl Into<String>,
        plane: Plane,
        purpose: Purpose,
        data_classes: Vec<DataClass>,
        decision: impl Into<String>,
    ) -> &AuditEvent {
        let previous_hash = self
            .events
            .last()
            .map(|event| event.hash.clone())
            .unwrap_or_else(|| "GENESIS".to_string());
        let mut event = AuditEvent {
            sequence: self.events.len() as u64,
            tenant_id: tenant_id.into(),
            surface: surface.into(),
            plane,
            purpose,
            data_classes,
            decision: decision.into(),
            previous_hash,
            hash: String::new(),
        };
        event.hash = event_hash(&event);
        self.events.push(event);
        self.events.last().expect("just pushed")
    }

    /// Append typed field classifications while preserving the legacy
    /// `DataClass` audit payload and hash input. This is the compatibility seam
    /// for append-only ledger replay while callers migrate operational markers
    /// such as `AUDIT` out of privacy-program `DataClass` construction.
    pub fn append_classifications<C>(
        &mut self,
        tenant_id: impl Into<String>,
        surface: impl Into<String>,
        plane: Plane,
        purpose: Purpose,
        data_classifications: impl IntoIterator<Item = C>,
        decision: impl Into<String>,
    ) -> &AuditEvent
    where
        C: Into<DataClassification>,
    {
        self.append_legacy_data_classes(
            tenant_id,
            surface,
            plane,
            purpose,
            data_classifications
                .into_iter()
                .map(|classification| classification.into().compatibility_data_class())
                .collect(),
            decision,
        )
    }

    pub fn events(&self) -> &[AuditEvent] {
        &self.events
    }

    pub fn verify(&self) -> bool {
        let mut previous = "GENESIS".to_string();
        for (index, event) in self.events.iter().enumerate() {
            if event.sequence != index as u64 || event.previous_hash != previous {
                return false;
            }
            if event.hash != event_hash(event) {
                return false;
            }
            previous = event.hash.clone();
        }
        true
    }
}

fn event_hash(event: &AuditEvent) -> String {
    let mut state = 0xcbf29ce484222325_u64;
    fn feed(state: &mut u64, bytes: &[u8]) {
        for byte in bytes {
            *state ^= u64::from(*byte);
            *state = state.wrapping_mul(0x100000001b3);
        }
    }
    feed(&mut state, event.sequence.to_string().as_bytes());
    feed(&mut state, event.tenant_id.as_bytes());
    feed(&mut state, event.surface.as_bytes());
    feed(&mut state, format!("{:?}", event.plane).as_bytes());
    feed(&mut state, format!("{:?}", event.purpose).as_bytes());
    feed(&mut state, format!("{:?}", event.data_classes).as_bytes());
    feed(&mut state, event.decision.as_bytes());
    feed(&mut state, event.previous_hash.as_bytes());
    format!("fnv1a64:{state:016x}")
}

#[cfg(test)]
mod tests {
    use oya_platform_data_boundary_kernel::{DataClassification, OperationalDataClass};

    use super::*;

    #[test]
    fn classified_append_preserves_legacy_audit_hash_payload() {
        let mut legacy = AuditChain::default();
        let legacy_event = legacy
            .append_legacy_data_classes(
                "ten_alpha",
                "foundry.evidence.emit",
                Plane::Audit,
                Purpose::CapabilityInvocation,
                vec![DataClass::InternalOnly, DataClass::Audit],
                "ALLOW",
            )
            .clone();

        let mut classified = AuditChain::default();
        let classified_event = classified
            .append_classifications(
                "ten_alpha",
                "foundry.evidence.emit",
                Plane::Audit,
                Purpose::CapabilityInvocation,
                [
                    DataClassification::from(DataClass::InternalOnly),
                    DataClassification::from(OperationalDataClass::Audit),
                ],
                "ALLOW",
            )
            .clone();

        assert_eq!(classified_event.data_classes, legacy_event.data_classes);
        assert_eq!(classified_event.hash, legacy_event.hash);
        assert_eq!(classified.events(), legacy.events());
        assert!(classified.verify());
    }
}
