//! Cloud marketplace kernel (M-CC-M03-P03-IP-005 minimum viable kernel).
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
    pub offer_id: String,
    pub publisher_tenant_id: String,
    pub kind: OfferKind,
    pub state: OfferState,
    pub valid_until_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entitlement {
    pub entitlement_id: String,
    pub offer_id: String,
    pub buyer_tenant_id: String,
    pub state: EntitlementState,
    pub seats: u32,
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
}
