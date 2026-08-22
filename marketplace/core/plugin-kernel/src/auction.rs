//! Ads auction kernel — `Auction` + `Bid` internal-tenant types.
//!
//! Per M06-P01-IP-001. Merge-variant delta-1: smallest net-new type surface
//! added into the existing `saas-plugin-marketplace-kernel` crate
//! (merge-into-existing-crates, decided_by=user-directive-option-2, 2026-05-17).
//!
//! An [`Auction`] is a single-round, internal-tenant-only ad slot contest.
//! Each participating advertiser submits a [`Bid`]; the auction selects the
//! winner by highest `bid_micros` (second-price settlement is a future
//! application-layer concern — this kernel only models identity + validation).
//!
//! ## Tenant isolation
//!
//! `tenant_id` on both [`Auction`] and [`Bid`] must match: the kernel
//! rejects cross-tenant bids at construction time, enforcing the
//! Data Use Boundary required by M06-P01 (preview tier, internal only).
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

const AUCTION_ID_PREFIX: &str = "auc_";
const BID_ID_PREFIX: &str = "bid_";
const MAX_BIDS_PER_AUCTION: usize = 256;

/// Errors raised by [`Auction`] / [`Bid`] validation.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AuctionError {
    /// `auction_id` / `bid_id` does not carry the required prefix.
    InvalidAuctionId,
    InvalidBidId,
    /// `tenant_id` is empty or exceeds 64 bytes.
    InvalidTenantId,
    /// `ad_slot_id` is empty.
    EmptyAdSlotId,
    /// `advertiser_id` is empty.
    EmptyAdvertiserId,
    /// `bid_micros` must be > 0 (floor price enforced at this layer).
    ZeroBidAmount,
    /// Bid's `tenant_id` does not match the auction's `tenant_id`.
    TenantMismatch,
    /// Auction already has [`MAX_BIDS_PER_AUCTION`] bids.
    AuctionFull,
    /// Auction has already been closed; no further bids accepted.
    AuctionClosed,
    /// A bid with the same [`BidId`] was already submitted to this auction.
    DuplicateBidId,
    /// `winner()` was called before the auction was closed.
    AuctionNotClosed,
}

/// Stable identifier for an ad auction slot contest.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AuctionId {
    pub value: String, // data_class: INTERNAL_ONLY
}

impl AuctionId {
    pub fn new(value: impl Into<String>) -> Result<Self, AuctionError> {
        let v = value.into();
        if !v.starts_with(AUCTION_ID_PREFIX) || v.len() <= AUCTION_ID_PREFIX.len() {
            return Err(AuctionError::InvalidAuctionId);
        }
        Ok(Self { value: v })
    }
}

/// Stable identifier for a single advertiser bid within an auction.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BidId {
    pub value: String, // data_class: INTERNAL_ONLY
}

impl BidId {
    pub fn new(value: impl Into<String>) -> Result<Self, AuctionError> {
        let v = value.into();
        if !v.starts_with(BID_ID_PREFIX) || v.len() <= BID_ID_PREFIX.len() {
            return Err(AuctionError::InvalidBidId);
        }
        Ok(Self { value: v })
    }
}

/// Input for constructing an [`Auction`].
#[derive(Clone, Debug)]
pub struct AuctionCreate {
    pub auction_id: String, // data_class: INTERNAL_ONLY
    pub tenant_id: String,  // data_class: INTERNAL_ONLY
    pub ad_slot_id: String, // data_class: INTERNAL_ONLY
}

/// Input for constructing a [`Bid`].
#[derive(Clone, Debug)]
pub struct BidCreate {
    pub bid_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,     // data_class: INTERNAL_ONLY
    pub advertiser_id: String, // data_class: INTERNAL_ONLY
    /// CPM in micro-currency units (e.g. USD micro-cents: 1_000_000 = $1.00).
    pub bid_micros: u64, // data_class: INTERNAL_ONLY
}

/// An internal-tenant ad slot auction.
///
/// Holds the ordered list of [`Bid`]s submitted by tenant advertisers.
/// A fresh auction starts [`AuctionState::Open`]; calling [`Auction::close`]
/// transitions it to [`AuctionState::Closed`] and freezes the bid list.
///
/// `state` and `bids` are private; mutations must go through [`Auction::submit_bid`]
/// and [`Auction::close`] to preserve lifecycle and capacity invariants.
#[derive(Clone, Debug, PartialEq)]
pub struct Auction {
    id: AuctionId,       // data_class: INTERNAL_ONLY
    tenant_id: String,   // data_class: INTERNAL_ONLY
    ad_slot_id: String,  // data_class: INTERNAL_ONLY
    state: AuctionState, // data_class: INTERNAL_ONLY
    bids: Vec<Bid>,      // data_class: INTERNAL_ONLY
}

/// Lifecycle state of an [`Auction`].
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AuctionState {
    Open,
    Closed,
}

impl Auction {
    /// Construct a new open auction.
    pub fn new(input: AuctionCreate) -> Result<Self, AuctionError> {
        let id = AuctionId::new(input.auction_id)?;
        validate_tenant_id(&input.tenant_id)?;
        if input.ad_slot_id.trim().is_empty() {
            return Err(AuctionError::EmptyAdSlotId);
        }
        Ok(Self {
            id,
            tenant_id: input.tenant_id,
            ad_slot_id: input.ad_slot_id,
            state: AuctionState::Open,
            bids: Vec::new(),
        })
    }

    /// Submit a [`Bid`] into this auction.
    ///
    /// Validates bid invariants (`bid_micros > 0`, non-empty `advertiser_id`,
    /// valid `BidId` prefix) at insertion time, rejects cross-tenant bids,
    /// rejects duplicate [`BidId`]s, and rejects bids submitted after close.
    pub fn submit_bid(&mut self, bid: Bid) -> Result<(), AuctionError> {
        if self.state == AuctionState::Closed {
            return Err(AuctionError::AuctionClosed);
        }
        if bid.tenant_id != self.tenant_id {
            return Err(AuctionError::TenantMismatch);
        }
        // Re-validate bid invariants at insertion: guards against direct struct
        // construction that bypasses `Bid::new`.
        if bid.bid_micros == 0 {
            return Err(AuctionError::ZeroBidAmount);
        }
        if bid.advertiser_id.trim().is_empty() {
            return Err(AuctionError::EmptyAdvertiserId);
        }
        if !bid.id.value.starts_with(BID_ID_PREFIX) || bid.id.value.len() <= BID_ID_PREFIX.len() {
            return Err(AuctionError::InvalidBidId);
        }
        // Reject duplicate BidId — a retry must not inflate the bid list.
        if self.bids.iter().any(|b| b.id == bid.id) {
            return Err(AuctionError::DuplicateBidId);
        }
        if self.bids.len() >= MAX_BIDS_PER_AUCTION {
            return Err(AuctionError::AuctionFull);
        }
        self.bids.push(bid);
        Ok(())
    }

    /// Close the auction; no further bids are accepted after this call.
    pub fn close(&mut self) {
        self.state = AuctionState::Closed;
    }

    /// Current lifecycle state of this auction.
    pub fn state(&self) -> &AuctionState {
        &self.state
    }

    /// Slice of bids currently held (read-only).
    pub fn bids(&self) -> &[Bid] {
        &self.bids
    }

    /// Return the winning [`Bid`] (highest `bid_micros`) after the auction is
    /// closed, or `None` if no bids were submitted.
    ///
    /// Returns [`AuctionError::AuctionNotClosed`] when called on an open
    /// auction; premature winner reads before `close()` are rejected to prevent
    /// settlement on an incomplete bid set.
    pub fn winner(&self) -> Result<Option<&Bid>, AuctionError> {
        if self.state != AuctionState::Closed {
            return Err(AuctionError::AuctionNotClosed);
        }
        Ok(self.bids.iter().max_by_key(|b| b.bid_micros))
    }

    /// Count of bids currently held.
    pub fn bid_count(&self) -> usize {
        self.bids.len()
    }
}

/// A single advertiser bid submitted into an [`Auction`].
#[derive(Clone, Debug, PartialEq)]
pub struct Bid {
    pub id: BidId,             // data_class: INTERNAL_ONLY
    pub tenant_id: String,     // data_class: INTERNAL_ONLY
    pub advertiser_id: String, // data_class: INTERNAL_ONLY
    /// CPM in micro-currency units; must be > 0.
    pub bid_micros: u64, // data_class: INTERNAL_ONLY
}

impl Bid {
    /// Construct a validated bid.
    pub fn new(input: BidCreate) -> Result<Self, AuctionError> {
        let id = BidId::new(input.bid_id)?;
        validate_tenant_id(&input.tenant_id)?;
        if input.advertiser_id.trim().is_empty() {
            return Err(AuctionError::EmptyAdvertiserId);
        }
        if input.bid_micros == 0 {
            return Err(AuctionError::ZeroBidAmount);
        }
        Ok(Self {
            id,
            tenant_id: input.tenant_id,
            advertiser_id: input.advertiser_id,
            bid_micros: input.bid_micros,
        })
    }
}

fn validate_tenant_id(id: &str) -> Result<(), AuctionError> {
    if id.trim().is_empty() || id.len() > 64 {
        return Err(AuctionError::InvalidTenantId);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn auction(id: &str, tenant: &str, slot: &str) -> Auction {
        Auction::new(AuctionCreate {
            auction_id: id.to_string(),
            tenant_id: tenant.to_string(),
            ad_slot_id: slot.to_string(),
        })
        .expect("auction")
    }

    fn bid(id: &str, tenant: &str, advertiser: &str, micros: u64) -> Bid {
        Bid::new(BidCreate {
            bid_id: id.to_string(),
            tenant_id: tenant.to_string(),
            advertiser_id: advertiser.to_string(),
            bid_micros: micros,
        })
        .expect("bid")
    }

    #[test]
    fn auction_id_requires_prefix() {
        assert_eq!(
            AuctionId::new("no_prefix").expect_err("prefix enforced"),
            AuctionError::InvalidAuctionId
        );
        assert_eq!(
            AuctionId::new("auc_").expect_err("empty suffix rejected"),
            AuctionError::InvalidAuctionId
        );
        AuctionId::new("auc_slot_001").expect("valid");
    }

    #[test]
    fn bid_id_requires_prefix() {
        assert_eq!(
            BidId::new("no_prefix").expect_err("prefix enforced"),
            AuctionError::InvalidBidId
        );
        BidId::new("bid_adv_001").expect("valid");
    }

    #[test]
    fn zero_bid_rejected() {
        let err = Bid::new(BidCreate {
            bid_id: "bid_z".to_string(),
            tenant_id: "ten_alpha".to_string(),
            advertiser_id: "adv_1".to_string(),
            bid_micros: 0,
        })
        .expect_err("zero bid floor");
        assert_eq!(err, AuctionError::ZeroBidAmount);
    }

    #[test]
    fn cross_tenant_bid_rejected() {
        let mut a = auction("auc_s1", "ten_alpha", "slot_home");
        let b = bid("bid_adv1", "ten_beta", "adv_1", 1_000_000);
        assert_eq!(
            a.submit_bid(b).expect_err("tenant mismatch"),
            AuctionError::TenantMismatch
        );
    }

    #[test]
    fn winner_is_highest_bid() {
        let mut a = auction("auc_s2", "ten_alpha", "slot_feed");
        a.submit_bid(bid("bid_low", "ten_alpha", "adv_1", 500_000))
            .unwrap();
        a.submit_bid(bid("bid_high", "ten_alpha", "adv_2", 2_000_000))
            .unwrap();
        a.submit_bid(bid("bid_mid", "ten_alpha", "adv_3", 1_000_000))
            .unwrap();
        a.close();
        let w = a.winner().expect("closed ok").expect("winner exists");
        assert_eq!(w.advertiser_id, "adv_2");
        assert_eq!(w.bid_micros, 2_000_000);
    }

    #[test]
    fn winner_on_open_auction_returns_error() {
        let mut a = auction("auc_s2b", "ten_alpha", "slot_feed");
        a.submit_bid(bid("bid_early", "ten_alpha", "adv_1", 500_000))
            .unwrap();
        assert_eq!(
            a.winner().expect_err("open auction rejects winner call"),
            AuctionError::AuctionNotClosed
        );
    }

    #[test]
    fn closed_auction_rejects_further_bids() {
        let mut a = auction("auc_s3", "ten_alpha", "slot_sidebar");
        a.close();
        let err = a
            .submit_bid(bid("bid_late", "ten_alpha", "adv_1", 1_000_000))
            .expect_err("closed auction");
        assert_eq!(err, AuctionError::AuctionClosed);
    }

    #[test]
    fn empty_auction_has_no_winner() {
        let mut a = auction("auc_s4", "ten_alpha", "slot_empty");
        a.close();
        assert!(a.winner().expect("closed ok").is_none());
    }

    #[test]
    fn bid_count_tracks_submissions() {
        let mut a = auction("auc_s5", "ten_alpha", "slot_count");
        assert_eq!(a.bid_count(), 0);
        a.submit_bid(bid("bid_a", "ten_alpha", "adv_1", 100_000))
            .unwrap();
        a.submit_bid(bid("bid_b", "ten_alpha", "adv_2", 200_000))
            .unwrap();
        assert_eq!(a.bid_count(), 2);
    }

    #[test]
    fn duplicate_bid_id_rejected() {
        let mut a = auction("auc_s6", "ten_alpha", "slot_dup");
        a.submit_bid(bid("bid_dup", "ten_alpha", "adv_1", 500_000))
            .unwrap();
        let err = a
            .submit_bid(bid("bid_dup", "ten_alpha", "adv_1", 600_000))
            .expect_err("duplicate bid id");
        assert_eq!(err, AuctionError::DuplicateBidId);
    }

    #[test]
    fn direct_struct_zero_bid_rejected_at_submit() {
        let mut a = auction("auc_s7", "ten_alpha", "slot_inv");
        // Bypass Bid::new by constructing directly — submit_bid must still catch it.
        let bad_bid = Bid {
            id: BidId {
                value: "bid_bad1".to_string(),
            },
            tenant_id: "ten_alpha".to_string(),
            advertiser_id: "adv_1".to_string(),
            bid_micros: 0,
        };
        assert_eq!(
            a.submit_bid(bad_bid).expect_err("zero bid at submit"),
            AuctionError::ZeroBidAmount
        );
    }

    #[test]
    fn direct_struct_empty_advertiser_rejected_at_submit() {
        let mut a = auction("auc_s8", "ten_alpha", "slot_inv2");
        let bad_bid = Bid {
            id: BidId {
                value: "bid_bad2".to_string(),
            },
            tenant_id: "ten_alpha".to_string(),
            advertiser_id: "  ".to_string(),
            bid_micros: 1_000_000,
        };
        assert_eq!(
            a.submit_bid(bad_bid)
                .expect_err("empty advertiser at submit"),
            AuctionError::EmptyAdvertiserId
        );
    }

    #[test]
    fn state_accessor_reflects_lifecycle() {
        let mut a = auction("auc_s9", "ten_alpha", "slot_state");
        assert_eq!(a.state(), &AuctionState::Open);
        a.close();
        assert_eq!(a.state(), &AuctionState::Closed);
    }

    #[test]
    fn bids_accessor_returns_slice() {
        let mut a = auction("auc_s10", "ten_alpha", "slot_slice");
        assert!(a.bids().is_empty());
        a.submit_bid(bid("bid_x", "ten_alpha", "adv_1", 100_000))
            .unwrap();
        assert_eq!(a.bids().len(), 1);
    }

    #[test]
    fn empty_tenant_id_rejected_on_auction() {
        let err = Auction::new(AuctionCreate {
            auction_id: "auc_t1".to_string(),
            tenant_id: "".to_string(),
            ad_slot_id: "slot_x".to_string(),
        })
        .expect_err("empty tenant");
        assert_eq!(err, AuctionError::InvalidTenantId);
    }

    #[test]
    fn empty_ad_slot_id_rejected() {
        let err = Auction::new(AuctionCreate {
            auction_id: "auc_t2".to_string(),
            tenant_id: "ten_alpha".to_string(),
            ad_slot_id: "  ".to_string(),
        })
        .expect_err("empty slot");
        assert_eq!(err, AuctionError::EmptyAdSlotId);
    }

    #[test]
    fn empty_advertiser_id_rejected_on_bid() {
        let err = Bid::new(BidCreate {
            bid_id: "bid_e1".to_string(),
            tenant_id: "ten_alpha".to_string(),
            advertiser_id: "".to_string(),
            bid_micros: 1_000_000,
        })
        .expect_err("empty advertiser");
        assert_eq!(err, AuctionError::EmptyAdvertiserId);
    }
}
