//! Cloud marketplace kernel (M03-P03-IP-005 minimum viable kernel).
//!
//! Pure I/O-free model for marketplace listings (ISV-published SaaS
//! offers), entitlement state, and the admission rule that an
//! entitlement cannot be activated without a non-expired offer +
//! a buyer tenant in good standing.

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum OfferKind {
    Saas,
    Container,
    Vm,
    DataProduct,
    Ml,
}

impl OfferKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Saas => "saas",
            Self::Container => "container",
            Self::Vm => "vm",
            Self::DataProduct => "data-product",
            Self::Ml => "ml",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OfferState {
    Draft,
    Published,
    Deprecated,
    Removed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntitlementState {
    Pending,
    Active,
    Suspended,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Offer {
    // data_class: INTERNAL_ONLY
    pub offer_id: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub publisher_tenant_id: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub kind: OfferKind, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub state: OfferState, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub valid_until_unix_ms: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entitlement {
    // data_class: INTERNAL_ONLY
    pub entitlement_id: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub offer_id: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub buyer_tenant_id: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub state: EntitlementState, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub seats: u32, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MarketplaceError {
    EmptyOfferId,
    EmptyEntitlementId,
    EmptyPublisher,
    EmptyBuyer,
    ZeroSeats,
    OfferNotPublished {
        offer_id: String,
        state: OfferState,
    },
    OfferExpired {
        offer_id: String,
    },
    BuyerSuspended,
    InvalidEntitlementTransition {
        from: EntitlementState,
        to: EntitlementState,
    },
}

impl MarketplaceError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyOfferId => "offer id is empty".to_owned(),
            Self::EmptyEntitlementId => "entitlement id is empty".to_owned(),
            Self::EmptyPublisher => "publisher_tenant_id is empty".to_owned(),
            Self::EmptyBuyer => "buyer_tenant_id is empty".to_owned(),
            Self::ZeroSeats => "entitlement requests zero seats".to_owned(),
            Self::OfferNotPublished { offer_id, state } => {
                format!("offer {offer_id} not Published (state={state:?})")
            }
            Self::OfferExpired { offer_id } => format!("offer {offer_id} expired"),
            Self::BuyerSuspended => "buyer tenant is suspended".to_owned(),
            Self::InvalidEntitlementTransition { from, to } => {
                format!("invalid entitlement transition: {from:?} -> {to:?}")
            }
        }
    }
}

pub fn validate_offer(o: &Offer) -> Result<(), MarketplaceError> {
    if o.offer_id.is_empty() {
        return Err(MarketplaceError::EmptyOfferId);
    }
    if o.publisher_tenant_id.is_empty() {
        return Err(MarketplaceError::EmptyPublisher);
    }
    Ok(())
}

pub fn activate_entitlement(
    ent: &mut Entitlement,
    offer: &Offer,
    now_unix_ms: u64,
    buyer_in_good_standing: bool,
) -> Result<(), MarketplaceError> {
    if ent.entitlement_id.is_empty() {
        return Err(MarketplaceError::EmptyEntitlementId);
    }
    if ent.buyer_tenant_id.is_empty() {
        return Err(MarketplaceError::EmptyBuyer);
    }
    if ent.seats == 0 {
        return Err(MarketplaceError::ZeroSeats);
    }
    if offer.state != OfferState::Published {
        return Err(MarketplaceError::OfferNotPublished {
            offer_id: offer.offer_id.clone(),
            state: offer.state,
        });
    }
    if now_unix_ms >= offer.valid_until_unix_ms {
        return Err(MarketplaceError::OfferExpired {
            offer_id: offer.offer_id.clone(),
        });
    }
    if !buyer_in_good_standing {
        return Err(MarketplaceError::BuyerSuspended);
    }
    if ent.state != EntitlementState::Pending {
        return Err(MarketplaceError::InvalidEntitlementTransition {
            from: ent.state,
            to: EntitlementState::Active,
        });
    }
    ent.state = EntitlementState::Active;
    Ok(())
}

pub fn cancel(ent: &mut Entitlement) -> Result<(), MarketplaceError> {
    if matches!(ent.state, EntitlementState::Cancelled) {
        return Err(MarketplaceError::InvalidEntitlementTransition {
            from: ent.state,
            to: EntitlementState::Cancelled,
        });
    }
    ent.state = EntitlementState::Cancelled;
    Ok(())
}

/// Transition an entitlement from `Active` to `Suspended`.
///
/// Returns `Err(InvalidEntitlementTransition)` for any source state other than
/// `Active`.
pub fn suspend(ent: &mut Entitlement) -> Result<(), MarketplaceError> {
    if ent.state != EntitlementState::Active {
        return Err(MarketplaceError::InvalidEntitlementTransition {
            from: ent.state,
            to: EntitlementState::Suspended,
        });
    }
    ent.state = EntitlementState::Suspended;
    Ok(())
}

/// Transition an entitlement from `Suspended` back to `Active`.
///
/// Re-validates that the offer is still `Published`, not yet expired, and that
/// the buyer tenant remains in good standing.  Returns the first applicable
/// error variant when any precondition fails.
pub fn reinstate(
    ent: &mut Entitlement,
    offer: &Offer,
    now_unix_ms: u64,
    buyer_in_good_standing: bool,
) -> Result<(), MarketplaceError> {
    if ent.state != EntitlementState::Suspended {
        return Err(MarketplaceError::InvalidEntitlementTransition {
            from: ent.state,
            to: EntitlementState::Active,
        });
    }
    if offer.state != OfferState::Published {
        return Err(MarketplaceError::OfferNotPublished {
            offer_id: offer.offer_id.clone(),
            state: offer.state,
        });
    }
    if now_unix_ms >= offer.valid_until_unix_ms {
        return Err(MarketplaceError::OfferExpired {
            offer_id: offer.offer_id.clone(),
        });
    }
    if !buyer_in_good_standing {
        return Err(MarketplaceError::BuyerSuspended);
    }
    ent.state = EntitlementState::Active;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offer(state: OfferState, valid_until: u64) -> Offer {
        Offer {
            offer_id: "O1".into(),
            publisher_tenant_id: "isv-1".into(),
            kind: OfferKind::Saas,
            state,
            valid_until_unix_ms: valid_until,
        }
    }

    fn ent(seats: u32) -> Entitlement {
        Entitlement {
            entitlement_id: "E1".into(),
            offer_id: "O1".into(),
            buyer_tenant_id: "buyer-1".into(),
            state: EntitlementState::Pending,
            seats,
        }
    }

    #[test]
    fn kind_names_distinct() {
        use std::collections::HashSet;
        let s: HashSet<_> = [
            OfferKind::Saas,
            OfferKind::Container,
            OfferKind::Vm,
            OfferKind::DataProduct,
            OfferKind::Ml,
        ]
        .iter()
        .map(|k| k.name())
        .collect();
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn activate_valid_passes() {
        let mut e = ent(5);
        let o = offer(OfferState::Published, 2000);
        assert!(activate_entitlement(&mut e, &o, 1000, true).is_ok());
        assert_eq!(e.state, EntitlementState::Active);
    }

    #[test]
    fn activate_draft_offer_rejected() {
        let mut e = ent(5);
        let o = offer(OfferState::Draft, 2000);
        assert!(matches!(
            activate_entitlement(&mut e, &o, 1000, true),
            Err(MarketplaceError::OfferNotPublished { .. })
        ));
    }

    #[test]
    fn activate_expired_offer_rejected() {
        let mut e = ent(5);
        let o = offer(OfferState::Published, 1000);
        assert!(matches!(
            activate_entitlement(&mut e, &o, 2000, true),
            Err(MarketplaceError::OfferExpired { .. })
        ));
    }

    #[test]
    fn activate_suspended_buyer_rejected() {
        let mut e = ent(5);
        let o = offer(OfferState::Published, 2000);
        assert!(matches!(
            activate_entitlement(&mut e, &o, 1000, false),
            Err(MarketplaceError::BuyerSuspended)
        ));
    }

    #[test]
    fn zero_seats_rejected() {
        let mut e = ent(0);
        let o = offer(OfferState::Published, 2000);
        assert!(matches!(
            activate_entitlement(&mut e, &o, 1000, true),
            Err(MarketplaceError::ZeroSeats)
        ));
    }

    #[test]
    fn already_active_rejected() {
        let mut e = ent(5);
        e.state = EntitlementState::Active;
        let o = offer(OfferState::Published, 2000);
        assert!(matches!(
            activate_entitlement(&mut e, &o, 1000, true),
            Err(MarketplaceError::InvalidEntitlementTransition { .. })
        ));
    }

    #[test]
    fn cancel_pending_succeeds() {
        let mut e = ent(5);
        assert!(cancel(&mut e).is_ok());
        assert_eq!(e.state, EntitlementState::Cancelled);
    }

    #[test]
    fn cancel_already_cancelled_rejected() {
        let mut e = ent(5);
        e.state = EntitlementState::Cancelled;
        assert!(matches!(
            cancel(&mut e),
            Err(MarketplaceError::InvalidEntitlementTransition { .. })
        ));
    }

    #[test]
    fn validate_offer_empty_id_rejected() {
        let mut o = offer(OfferState::Published, 2000);
        o.offer_id = String::new();
        assert!(matches!(
            validate_offer(&o),
            Err(MarketplaceError::EmptyOfferId)
        ));
    }

    #[test]
    fn validate_offer_empty_publisher_rejected() {
        let mut o = offer(OfferState::Published, 2000);
        o.publisher_tenant_id = String::new();
        assert!(matches!(
            validate_offer(&o),
            Err(MarketplaceError::EmptyPublisher)
        ));
    }

    // --- suspend tests ---

    #[test]
    fn suspend_active_succeeds() {
        let mut e = ent(5);
        e.state = EntitlementState::Active;
        assert!(suspend(&mut e).is_ok());
        assert_eq!(e.state, EntitlementState::Suspended);
    }

    #[test]
    fn suspend_pending_rejected() {
        let mut e = ent(5);
        // state is Pending by default
        assert!(matches!(
            suspend(&mut e),
            Err(MarketplaceError::InvalidEntitlementTransition {
                from: EntitlementState::Pending,
                to: EntitlementState::Suspended,
            })
        ));
    }

    #[test]
    fn suspend_already_suspended_rejected() {
        let mut e = ent(5);
        e.state = EntitlementState::Suspended;
        assert!(matches!(
            suspend(&mut e),
            Err(MarketplaceError::InvalidEntitlementTransition {
                from: EntitlementState::Suspended,
                to: EntitlementState::Suspended,
            })
        ));
    }

    #[test]
    fn suspend_cancelled_rejected() {
        let mut e = ent(5);
        e.state = EntitlementState::Cancelled;
        assert!(matches!(
            suspend(&mut e),
            Err(MarketplaceError::InvalidEntitlementTransition {
                from: EntitlementState::Cancelled,
                to: EntitlementState::Suspended,
            })
        ));
    }

    // --- reinstate tests ---

    #[test]
    fn reinstate_suspended_succeeds() {
        let mut e = ent(5);
        e.state = EntitlementState::Suspended;
        let o = offer(OfferState::Published, 2000);
        assert!(reinstate(&mut e, &o, 1000, true).is_ok());
        assert_eq!(e.state, EntitlementState::Active);
    }

    #[test]
    fn reinstate_from_active_rejected() {
        let mut e = ent(5);
        e.state = EntitlementState::Active;
        let o = offer(OfferState::Published, 2000);
        assert!(matches!(
            reinstate(&mut e, &o, 1000, true),
            Err(MarketplaceError::InvalidEntitlementTransition {
                from: EntitlementState::Active,
                to: EntitlementState::Active,
            })
        ));
    }

    #[test]
    fn reinstate_from_pending_rejected() {
        let mut e = ent(5);
        // state is Pending by default
        let o = offer(OfferState::Published, 2000);
        assert!(matches!(
            reinstate(&mut e, &o, 1000, true),
            Err(MarketplaceError::InvalidEntitlementTransition {
                from: EntitlementState::Pending,
                to: EntitlementState::Active,
            })
        ));
    }

    #[test]
    fn reinstate_from_cancelled_rejected() {
        let mut e = ent(5);
        e.state = EntitlementState::Cancelled;
        let o = offer(OfferState::Published, 2000);
        assert!(matches!(
            reinstate(&mut e, &o, 1000, true),
            Err(MarketplaceError::InvalidEntitlementTransition {
                from: EntitlementState::Cancelled,
                to: EntitlementState::Active,
            })
        ));
    }

    #[test]
    fn reinstate_expired_offer_rejected() {
        let mut e = ent(5);
        e.state = EntitlementState::Suspended;
        let o = offer(OfferState::Published, 1000);
        assert!(matches!(
            reinstate(&mut e, &o, 2000, true),
            Err(MarketplaceError::OfferExpired { .. })
        ));
    }

    #[test]
    fn reinstate_unpublished_offer_rejected() {
        let mut e = ent(5);
        e.state = EntitlementState::Suspended;
        let o = offer(OfferState::Deprecated, 2000);
        assert!(matches!(
            reinstate(&mut e, &o, 1000, true),
            Err(MarketplaceError::OfferNotPublished { .. })
        ));
    }

    #[test]
    fn reinstate_suspended_buyer_rejected() {
        let mut e = ent(5);
        e.state = EntitlementState::Suspended;
        let o = offer(OfferState::Published, 2000);
        assert!(matches!(
            reinstate(&mut e, &o, 1000, false),
            Err(MarketplaceError::BuyerSuspended)
        ));
    }
}
