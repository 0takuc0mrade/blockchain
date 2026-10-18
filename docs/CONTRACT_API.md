# Contract API reference

Full signatures are in
[`contracts/ticketing/src/lib.rs`](../contracts/ticketing/src/lib.rs);
this is a quick-reference summary.

`get_ticket` and the legacy `verify_ticket` implementation return the same
`Ticket` record and error codes. `get_ticket` is the canonical Soroban naming
convention; `verify_ticket` is deprecated but remains available so existing
clients do not break. It may be removed only in a future breaking deployment.

| Function | Caller | Effect |
|---|---|---|
| `initialize(admin, payment_token)` | admin | One-time setup |
| `approve_organizer(admin, organizer)` | admin | Adds an organizer to the event-creation allowlist (issue #135) |
| `revoke_organizer(admin, organizer)` | admin | Removes an organizer from the allowlist; existing events are unaffected |
| `is_approved_organizer(organizer)` | anyone | Whether an organizer may create events |
| `create_event(organizer, event_id, params)` | approved organizer | Registers an event (`params: EventParams`); see [ORGANIZER_ALLOWLIST.md](ORGANIZER_ALLOWLIST.md) |
| `issue_ticket(organizer, event_id, to, tier, seat, price)` | organizer | Mints a ticket (off-chain payment already settled); increments `tickets_issued` |
| `set_tier_price(organizer, event_id, tier, price)` | organizer | Sets the primary sale price for a tier (issue #127) |
| `purchase_primary(buyer, event_id, tier, seat)` | buyer | On-chain primary sale at the tier price + mint; increments `tickets_issued` |
| `transfer_ticket(from, ticket_id, to)` | owner | Direct transfer |
| `transfer_batch(from, ticket_ids, to)` | owner | Batch direct transfer (max 100 tickets) |
| `create_gift_claim(owner, ticket_id, secret_hash, expires_at)` | owner | Creates a hash-locked gift claim; the owner shares the preimage off-chain. See [`STELLAR_CLI_EXAMPLES.md`](STELLAR_CLI_EXAMPLES.md#create_gift_claim) for the SHA-256 workflow |
| `claim_gift(recipient, ticket_id, secret)` | recipient | Claims a gifted ticket by presenting the preimage before expiry. See [`STELLAR_CLI_EXAMPLES.md`](STELLAR_CLI_EXAMPLES.md#claim_gift) |
| `get_ticket(ticket_id)` | anyone | Canonical read-only ticket lookup |
| `verify_ticket(ticket_id)` | anyone | Deprecated compatibility alias for `get_ticket` |
| `check_in(organizer, ticket_id)` | organizer | Marks used, one-way |
| `revoke_ticket(organizer, ticket_id)` | organizer | Permanently voids (does not decrement `tickets_issued`) |
| `revoke_with_refund(organizer, ticket_id, refund)` | organizer | Voids a ticket, optionally paying the original price back to the owner in the event's accepted payment token (does not decrement `tickets_issued`) |
| `set_event_payment_token(organizer, event_id, token)` | organizer | Sets/clears the event's accepted payment token (only before any tickets are issued) |
| `event_payment_token(event_id)` | anyone | Resolves the event's payment token (per-event override or contract-wide token) |
| `token_decimals()` | anyone | Decimals of the active payment token (cached at initialize / token change) |
| `list_for_resale(owner, ticket_id, price)` | owner | Lists under the event's price cap |
| `cancel_resale(owner, ticket_id)` | owner | Pulls a listing |
| `buy_resale(buyer, ticket_id)` | buyer | Buys a listing, splits royalty |
| `get_event(event_id)` | anyone | Read-only event lookup |
| `get_ticket(ticket_id)` | anyone | Read-only ticket lookup |
| `get_organizer_events(organizer)` | anyone | Number of events registered by an organizer |

## `tickets_issued` semantics

`Event.tickets_issued` (returned by `get_event`) is a **monotonic, lifetime issuance counter** for the event — **not** the active or circulating ticket supply:

- **Incremented on mint:** `issue_ticket` and `purchase_primary` each increment `tickets_issued` by `1`, and `allocate_lottery` increments it by `winner_count`.
- **Never decremented:** Revoking tickets (`revoke_ticket`, `revoke_with_refund`, `revoke_batch`) or checking them in (`check_in`, `check_in_batch`) transitions the ticket's `status` to `TicketStatus::Revoked` or `TicketStatus::Used`, respectively, without decrementing `tickets_issued`.
- **Why it is monotonic:** The contract uses `tickets_issued > 0` as a one-way guard to lock event settlement parameters (`enable_escrow`, `set_event_payment_token`) once any ticket has ever been issued, even if all issued tickets are later revoked.
- **Computing active supply:** Integrators that need the count of currently usable tickets (`TicketStatus::Valid` or `TicketStatus::Resale`) must not read `tickets_issued` as active supply; instead, index on-chain issuance/check-in events and revocation transitions or inspect ticket statuses via `get_ticket` / `verify_tickets`.

## Error codes

See [`ERRORS.md`](ERRORS.md) for the full table of every `Error` variant
(1–41), which entry points return each one, and whether it is worth
retrying. The enum itself lives in
[`contracts/ticketing/src/error.rs`](../contracts/ticketing/src/error.rs).
