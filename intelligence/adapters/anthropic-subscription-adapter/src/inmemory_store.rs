//! In-memory implementations of CredentialStorePort and OperatorAlertPort for tests.
// data_class: INTERNAL_ONLY throughout this module.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ports::{AlertKind, CredentialStorePort, OperatorAlertPort, SeatId, TokenBytes};

/// Thread-safe in-memory credential store. Used in tests and single-node bring-up.
#[derive(Default, Clone)]
pub struct InMemoryCredentialStore {
    data: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl InMemoryCredentialStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of stored entries (test helper).
    pub fn len(&self) -> usize {
        self.data.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl CredentialStorePort for InMemoryCredentialStore {
    fn store(&self, seat_id: &SeatId, bytes: TokenBytes) -> Result<(), String> {
        self.data
            .lock()
            .map_err(|e| e.to_string())?
            .insert(seat_id.0.clone(), bytes.0);
        Ok(())
    }

    fn load(&self, seat_id: &SeatId) -> Option<TokenBytes> {
        self.data
            .lock()
            .ok()?
            .get(&seat_id.0)
            .map(|v| TokenBytes(v.clone()))
    }

    fn delete(&self, seat_id: &SeatId) {
        if let Ok(mut map) = self.data.lock() {
            map.remove(&seat_id.0);
        }
    }
}

/// Thread-safe in-memory operator alert collector. Records emitted alerts for assertion in tests.
#[derive(Default, Clone)]
pub struct InMemoryAlertPort {
    alerts: Arc<Mutex<Vec<(SeatId, AlertKind)>>>,
}

impl InMemoryAlertPort {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return all collected alerts (test helper).
    pub fn collected(&self) -> Vec<(SeatId, AlertKind)> {
        self.alerts.lock().unwrap().clone()
    }

    /// Number of collected alerts (test helper).
    pub fn count(&self) -> usize {
        self.alerts.lock().unwrap().len()
    }
}

impl OperatorAlertPort for InMemoryAlertPort {
    fn alert(&self, seat_id: &SeatId, kind: AlertKind) {
        self.alerts.lock().unwrap().push((seat_id.clone(), kind));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_store_roundtrip() {
        let store = InMemoryCredentialStore::new();
        let seat = SeatId("seat-1".into());
        assert!(store.load(&seat).is_none());

        store.store(&seat, TokenBytes(b"hello".to_vec())).unwrap();
        assert_eq!(store.load(&seat).unwrap().0, b"hello");

        store.delete(&seat);
        assert!(store.load(&seat).is_none());
    }

    #[test]
    fn alert_port_collects() {
        let port = InMemoryAlertPort::new();
        assert_eq!(port.count(), 0);

        port.alert(&SeatId("s1".into()), AlertKind::RefreshTokenExpired);
        port.alert(&SeatId("s2".into()), AlertKind::RefreshTokenReused);

        let collected = port.collected();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0].1, AlertKind::RefreshTokenExpired);
        assert_eq!(collected[1].1, AlertKind::RefreshTokenReused);
    }
}
