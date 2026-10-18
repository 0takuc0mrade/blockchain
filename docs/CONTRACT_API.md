# Contract API reference

Full signatures are in
[`contracts/ticketing/src/lib.rs`](../contracts/ticketing/src/lib.rs);
this is a quick-reference summary.

`get_ticket` and the legacy `verify_ticket` implementation return the same
`Ticket` record and error codes. `get_ticket` is the canonical Soroban naming
convention; `verify_ticket` is deprecated but remains available so existing
clients do not break. It may be removed only in a future breaking deployment.

| Function                                                                                    | Caller    | Effect                                                                                                       |
| ------------------------------------------------------------------------------------------- | --------- | ------------------------------------------------------------------------------------------------------------ |
| `initialize(admin, payment_token)`                                                          | admin     | One-time setup                                                                                               |
| `create_event(organizer, event_id, name, category, max_resale_multiplier_bps, royalty_bps)` | organizer | Registers an event                                                                                           |
| `issue_ticket(organizer, event_id, to, tier, seat, price)`                                  | organizer | Mints a ticket (off-chain payment already settled)                                                           |
| `purchase_primary(buyer, event_id, tier, seat, price)`                                      | buyer     | On-chain primary sale + mint                                                                                 |
| `transfer_ticket(from, ticket_id, to)`                                                      | owner     | Direct transfer                                                                                              |
| `get_ticket(ticket_id)`                                                                     | anyone    | Canonical read-only ticket lookup                                                                            |
| `verify_ticket(ticket_id)`                                                                  | anyone    | Deprecated compatibility alias for `get_ticket`                                                              |
| `check_in(organizer, ticket_id)`                                                            | organizer | Marks used, one-way                                                                                          |
| `revoke_ticket(organizer, ticket_id)`                                                       | organizer | Permanently voids                                                                                            |
| `revoke_with_refund(organizer, ticket_id, refund)`                                          | organizer | Voids a ticket, optionally paying the original price back to the owner in the event's accepted payment token |
| `set_event_payment_token(organizer, event_id, token)`                                       | organizer | Sets/clears the event's accepted payment token (only before any tickets are issued)                          |
| `event_payment_token(event_id)`                                                             | anyone    | Resolves the event's payment token (per-event override or contract-wide token)                               |
| `token_decimals()`                                                                          | anyone    | Decimals of the active payment token (cached at initialize / token change)                                   |
| `list_for_resale(owner, ticket_id, price)`                                                  | owner     | Lists under the event's price cap                                                                            |
| `cancel_resale(owner, ticket_id)`                                                           | owner     | Pulls a listing before the resale cutoff; rejects with `ResaleClosed` after cutoff                           |
| `buy_resale(buyer, ticket_id)`                                                              | buyer     | Buys a listing, splits royalty                                                                               |
| `get_event(event_id)`                                                                       | anyone    | Read-only event lookup                                                                                       |
| `get_ticket(ticket_id)`                                                                     | anyone    | Read-only ticket lookup                                                                                      |
| `get_organizer_events(organizer)`                                                           | anyone    | Number of events registered by an organizer                                                                  |

> Resale cutoff semantics: once `ledger_timestamp >= starts_at - resale_cutoff_seconds`, all resale actions reject with `Error::ResaleClosed`. That policy is enforced by the contract's `resale_closed` helper and applies to `list_for_resale`, `buy_resale`, and `cancel_resale`.

## Error codes

See [`ERRORS.md`](ERRORS.md) for the full table of every `Error` variant
(1–34), which entry points return each one, and whether it is worth
retrying. The enum itself lives in
[`contracts/ticketing/src/error.rs`](../contracts/ticketing/src/error.rs).
