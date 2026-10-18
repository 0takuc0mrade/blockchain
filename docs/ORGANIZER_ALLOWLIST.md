# Organizer allowlist (event-id squatting)

Issue: #135

## Problem

`create_event` and `create_event_with_options` take a caller-chosen
`event_id`. The backend picks the id (typically a ULID cast to `u64`) so the
on-chain event can be correlated with its off-chain record. Before this
change, any address could call `create_event` with any unused id. Because
`EventAlreadyExists` stops a second registration, a squatter who saw the
backend's id (for example in a pending transaction, or by predicting the
allocation scheme) could register it first. The legitimate registration
would then fail, and the event would belong to the squatter.

## Options considered

### 1. Namespace ids by organizer

Key events by `(organizer, event_id)` instead of `event_id` alone, so each
organizer has its own id space and cannot collide with anyone else.

- **Pro:** permissionless; no admin involvement to onboard an organizer.
- **Con:** `Ticket.event_id`, `DataKey::Event`, `TicketsIssued`,
  `TierPrice` and every read entry point (`get_event`, `get_tier_price`,
  `event_payment_token`, ...) are keyed by a bare `u64`. Namespacing changes
  the storage layout and the public ABI of every event-scoped function, and
  existing stored events and tickets would need a migration. The contract
  has no upgrade hook (see [UPGRADES.md](UPGRADES.md)), so this means a
  fresh deployment.
- **Con:** it only moves the problem. Namespacing stops squatting on
  *another organizer's* ids, but it still lets anyone create events. Spam
  events that look legitimate stay possible.

### 2. Admin-managed organizer allowlist (chosen)

Only addresses the admin has approved can create events. Everything else is
unchanged.

- **Pro:** no ABI or storage changes to existing entry points. `event_id`
  stays a global `u64`, so tickets, indexers and clients keep working.
- **Pro:** closes both squatting and event spam. An address that isn't
  approved can't register anything.
- **Pro:** matches how the platform already works. Organizers are onboarded
  by the backend, which already holds a trusted role (it picks the ids).
- **Con:** adds a permissioned step. The admin must approve each organizer
  before its first event. This is an onboarding call that happens once per
  organizer, not once per event.
- **Residual:** approved organizers can still collide with each other. The
  backend allocates ids for all of them, so it must keep allocating without
  collisions (ULIDs already do this). A compromised organizer key can
  squat until the admin revokes it.

## Design

| Item | Detail |
|---|---|
| Storage | `DataKey::ApprovedOrganizer(Address)` in persistent storage. The key exists only for approved organizers. Approval and every successful event creation extend its TTL. |
| `approve_organizer(admin, organizer)` | Admin only (`require_admin`). Idempotent. Emits `OrganizerApproved`. |
| `revoke_organizer(admin, organizer)` | Admin only. Idempotent. Emits `OrganizerRevoked`. Only blocks **new** events. Events the organizer already created, and their tickets, keep working. |
| `is_approved_organizer(organizer) -> bool` | Read-only. |
| Enforcement | `create_event` and `create_event_with_options` call `organizer.require_auth()` first, then check the allowlist. A missing entry returns `OrganizerNotApproved` (error 41). |
| Ordering | The auth check runs before the allowlist check. An unauthenticated caller gets a host auth error, not `OrganizerNotApproved`, so it can't probe who is on the allowlist. This keeps the contract-wide auth-before-validation ordering. |

The allowlist gates only event **creation**. Every other organizer entry
point already compares the caller with `Event.organizer`, which is fixed at
creation. So once creation is gated, no other entry point can be squatted.

## Operational notes

- **Deployment:** after `initialize`, call `approve_organizer` for each
  organizer before its first `create_event`. See
  [STELLAR_CLI_EXAMPLES.md](STELLAR_CLI_EXAMPLES.md).
- **Offboarding:** `revoke_organizer` stops an organizer from creating new
  events. It does not revoke tickets that were already issued.
- **TTL:** an approval entry that nobody touches for longer than the
  persistent TTL can be archived. The organizer then needs a new
  `approve_organizer` (or a restore) before its next event. An organizer
  that creates events regularly keeps its entry alive.

## Tests

[`contracts/ticketing/src/test/organizer_allowlist.rs`](../contracts/ticketing/src/test/organizer_allowlist.rs)
covers:

- a squat attempt by an address that isn't approved, and the id staying
  free for the real organizer;
- the same rejection on `create_event_with_options`;
- approval and idempotency;
- revocation blocking new events while existing events keep working;
- admin-only management;
- the auth check running before the allowlist check.

The `approve_organizer` and `revoke_organizer` auth checks are also covered
by the authorization matrix in
[`test/auth_matrix.rs`](../contracts/ticketing/src/test/auth_matrix.rs).
