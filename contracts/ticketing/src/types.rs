//! Contract storage types shared across the ticketing contract.
#![allow(missing_docs)]

use crate::Error;
use soroban_sdk::{contracttype, Address, BytesN, String};

/// Optional common category labels for indexers and client applications.
///
/// `Event.category` intentionally remains a `String` so deployments can add
/// industry-specific labels without a contract migration. Use `Other` when a
/// value does not fit the common vocabulary.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Category {
    Concert,
    Flight,
    Sports,
    Festival,
    Conference,
    Bus,
    Cinema,
    Museum,
    TouristAttraction,
    PublicTransport,
    University,
    Corporate,
    Other,
}

/// Lifecycle status of a ticket.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TicketStatus {
    /// Usable for entry and transfer.
    Valid,
    /// Checked in at the gate; no longer usable.
    Used,
    /// Voided by the organizer; permanently unusable.
    Revoked,
    /// Listed on the resale marketplace.
    Resale,
}

/// Configuration parameters for registering an event via `create_event`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventParams {
    /// Display name of the event.
    pub name: String,
    /// Category such as "concert", "flight", "sports", "conference", etc.
    pub category: String,
    /// Basis points cap on resale price relative to original sale price
    /// (e.g. 12000 = 120%). Anti-scalping enforcement.
    pub max_resale_multiplier_bps: u32,
    /// Basis points of every resale price paid to the organizer as royalty.
    pub royalty_bps: u32,
    /// Ledger timestamp at which the event starts.
    pub starts_at: u64,
    /// Seconds before `starts_at` after which direct transfers are frozen.
    pub transfer_freeze_seconds: u64,
    /// Seconds before `starts_at` after which resale listings are closed.
    pub resale_cutoff_seconds: u64,
}

/// An event registered by an organizer.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    /// Address of the event organizer, the only account allowed to issue
    /// tickets, check them in, revoke them, and enable escrow.
    pub organizer: Address,
    /// Display name of the event.
    pub name: String,
    /// Category such as "concert", "flight", "sports", "conference", etc.
    /// Source of truth: the backend enforces the shared industry list in
    /// [docs/INDUSTRIES.md](../../docs/INDUSTRIES.md) and the `Category` enum/
    /// `CATEGORY_*` constants exported by this contract. The storage field is a
    /// `String` for ABI compatibility, but callers must use the canonical
    /// backend vocabulary rather than inventing new labels.
    pub category: String,
    /// Basis points cap on resale price relative to original sale price
    /// (e.g. 12000 = 120%). Anti-scalping enforcement.
    pub max_resale_multiplier_bps: u32,
    /// Optional floor on resale price relative to original price.
    pub min_resale_multiplier_bps: Option<u32>,
    /// Optional maximum number of ownership transfers for tickets in this event.
    pub max_transfers_per_ticket: Option<u32>,
    /// Basis points of every resale price paid to the organizer as royalty.
    pub royalty_bps: u32,
    /// Cumulative number of tickets ever minted for the event (`issue_ticket`,
    /// `purchase_primary`, `allocate_lottery`). Monotonic — never decremented
    /// when a ticket is revoked or checked in, so revoked tickets still count
    /// toward this total. Do not read this field as active supply.
    pub tickets_issued: u64,
    /// Ledger timestamp at which the event starts.
    pub starts_at: u64,
    /// Seconds before `starts_at` after which direct transfers are frozen.
    pub transfer_freeze_seconds: u64,
    /// Seconds before `starts_at` after which resale listings are closed.
    pub resale_cutoff_seconds: u64,
    /// When true, primary sale proceeds are held by the contract instead of
    /// paid to the organizer immediately, and can only be released once the
    /// ledger sequence reaches `escrow_release_ledger`.
    pub escrow_enabled: bool,
    /// Ledger sequence after which escrowed proceeds may be released.
    /// Ignored when `escrow_enabled` is false.
    pub escrow_release_ledger: u32,
    /// Primary sale proceeds currently held in escrow for this event.
    pub escrow_balance: i128,
    /// Per-event accepted payment token (issue #235). `None` means the
    /// event settles in the contract-wide payment token set at
    /// initialization. Lockable only while no tickets have been issued so
    /// existing sales stay denominated in the token they were paid in.
    pub payment_token: Option<Address>,
}

/// A ticket minted for an event.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ticket {
    /// Id of the event this ticket admits entry to.
    pub event_id: u64,
    /// Current owner of the ticket.
    pub owner: Address,
    /// Ticket tier name, e.g. "VIP" or "GA".
    pub tier: String,
    /// Assigned seat, or "unassigned" for general admission.
    pub seat: String,
    /// Current lifecycle status of the ticket.
    pub status: TicketStatus,
    /// Price paid at primary sale; used for resale caps and refunds.
    pub original_price: i128,
    /// Asking price while the ticket is listed for resale; 0 otherwise.
    pub resale_price: i128,
    /// Number of ownership transfers completed for this ticket.
    pub transfers: u32,
}

/// Compact persistent representation for a ticket.
///
/// The public `Ticket` ABI stays unchanged for clients, while storage packs
/// status and transfer count into one lifecycle word to reduce the persistent
/// footprint of every ticket record.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredTicket {
    /// Id of the event this ticket admits entry to.
    pub event_id: u64,
    /// Current owner of the ticket.
    pub owner: Address,
    /// Ticket tier name, e.g. "VIP" or "GA".
    pub tier: String,
    /// Assigned seat, or "unassigned" for general admission.
    pub seat: String,
    /// Packed lifecycle state: low byte is `TicketStatus`, high bits are transfers.
    pub lifecycle: u64,
    /// Price paid at primary sale; used for resale caps and refunds.
    pub original_price: i128,
    /// Asking price while the ticket is listed for resale; 0 otherwise.
    pub resale_price: i128,
}

/// A pending gift-claim link for a ticket.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GiftClaim {
    /// Owner the ticket must still belong to when the claim is accepted.
    pub from: Address,
    /// SHA-256 digest of the claim secret; the preimage is shared off-chain.
    pub secret_hash: BytesN<32>,
    /// Ledger timestamp after which the claim can no longer be accepted.
    pub expires_at: u64,
}

/// A payment token change that has been proposed but not yet applied.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingPaymentToken {
    /// Token contract address proposed as the new payment token.
    pub token: Address,
    /// Ledger sequence after which the change may be applied.
    pub apply_after_ledger: u32,
}

/// Storage keys for all contract state.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Contract admin address.
    Admin,
    /// Contract-wide payment token address.
    PaymentToken,
    /// Decimals of the active payment token, cached for client display.
    TokenDecimals,
    /// Timelocked payment token change awaiting application.
    PendingPaymentToken,
    /// Event record, keyed by event id.
    Event(u64),
    /// Ticket record, keyed by ticket id.
    Ticket(u64),
    /// Pending gift claim, keyed by ticket id.
    GiftClaim(u64),
    /// Monotonic counter used to allocate new ticket ids.
    NextTicketId,
    /// Number of tickets issued for an event. Kept separate from `Event` so
    /// minting does not rewrite the complete event record.
    TicketsIssued(u64),
    /// Primary sale price for one of an event's tiers, keyed by event id and
    /// tier name (issue #127). Stored alongside the event record rather than
    /// inside it so changing a tier price never rewrites the whole `Event`
    /// struct, which also keeps already-stored event records readable.
    TierPrice(u64, String),
    /// Number of events registered by an organizer.
    OrganizerEvents(Address),
    /// Ledger sequence of a buyer's most recent primary purchase.
    LastPurchaseLedger(Address),
    /// Minimum ledger spacing enforced between a buyer's primary purchases.
    MinPurchaseSpacing,
    /// Presence marks an organizer the admin has approved to create events
    /// (issue #135).
    ApprovedOrganizer(Address),
}

// Keep the persistent encoding beside the public and stored ticket types.
impl StoredTicket {
    pub(crate) fn pack(ticket: &Ticket) -> StoredTicket {
        StoredTicket {
            event_id: ticket.event_id,
            owner: ticket.owner.clone(),
            tier: ticket.tier.clone(),
            seat: ticket.seat.clone(),
            lifecycle: ((ticket.transfers as u64) << 8) | Self::status_code(&ticket.status),
            original_price: ticket.original_price,
            resale_price: ticket.resale_price,
        }
    }

    pub(crate) fn unpack(stored: StoredTicket) -> Result<Ticket, Error> {
        let status_code = stored.lifecycle & 0xff;
        let transfers = (stored.lifecycle >> 8) as u32;
        Ok(Ticket {
            event_id: stored.event_id,
            owner: stored.owner,
            tier: stored.tier,
            seat: stored.seat,
            status: Self::status_from_code(status_code)?,
            original_price: stored.original_price,
            resale_price: stored.resale_price,
            transfers,
        })
    }

    fn status_code(status: &TicketStatus) -> u64 {
        match status {
            TicketStatus::Valid => 0,
            TicketStatus::Used => 1,
            TicketStatus::Revoked => 2,
            TicketStatus::Resale => 3,
        }
    }

    fn status_from_code(code: u64) -> Result<TicketStatus, Error> {
        match code {
            0 => Ok(TicketStatus::Valid),
            1 => Ok(TicketStatus::Used),
            2 => Ok(TicketStatus::Revoked),
            3 => Ok(TicketStatus::Resale),
            _ => Err(Error::InvalidTicketLifecycle),
        }
    }
}
