# Contract API reference

Full signatures are in
[`contracts/ticketing/src/lib.rs`](../contracts/ticketing/src/lib.rs);
this is a quick-reference summary.

| Function | Caller | Effect |
|---|---|---|
| `initialize(admin, payment_token)` | admin | One-time setup |
| `create_event(organizer, event_id, params)` | organizer | Registers an event (`params: EventParams`) |
| `issue_ticket(organizer, event_id, to, tier, seat, price)` | organizer | Mints a ticket (off-chain payment already settled); increments `tickets_issued` |
| `purchase_primary(buyer, event_id, tier, seat, price)` | buyer | On-chain primary sale + mint; increments `tickets_issued` |
| `transfer_ticket(from, ticket_id, to)` | owner | Direct transfer |
| `verify_ticket(ticket_id)` | anyone | Read-only lookup |
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

## `tickets_issued` semantics

`Event.tickets_issued` (returned by `get_event`) is a **monotonic, lifetime issuance counter** for the event — **not** the active or circulating ticket supply:

- **Incremented on mint:** `issue_ticket` and `purchase_primary` each increment `tickets_issued` by `1`, and `allocate_lottery` increments it by `winner_count`.
- **Never decremented:** Revoking tickets (`revoke_ticket`, `revoke_with_refund`, `revoke_batch`) or checking them in (`check_in`, `check_in_batch`) transitions the ticket's `status` to `TicketStatus::Revoked` or `TicketStatus::Used`, respectively, without decrementing `tickets_issued`.
- **Why it is monotonic:** The contract uses `tickets_issued > 0` as a one-way guard to lock event settlement parameters (`enable_escrow`, `set_event_payment_token`) once any ticket has ever been issued, even if all issued tickets are later revoked.
- **Computing active supply:** Integrators that need the count of currently usable tickets (`TicketStatus::Valid` or `TicketStatus::Resale`) must not read `tickets_issued` as active supply; instead, index on-chain issuance/check-in events and revocation transitions or inspect ticket statuses via `get_ticket` / `verify_tickets`.

## Error codes

See [`ERRORS.md`](ERRORS.md) for the full table of every `Error` variant
(1–34), which entry points return each one, and whether it is worth
retrying. The enum itself lives in
[`contracts/ticketing/src/error.rs`](../contracts/ticketing/src/error.rs).
