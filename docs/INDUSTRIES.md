# Supported industries

`Event.category` is stored as a `String` for ABI compatibility, but the
backend's source of truth is the shared industry enum used by the service and
its dashboard. The accepted values are the canonical labels documented by the
backend and mirrored in the contract's `Category` enum / `CATEGORY_*`
constants.

The backend-enforced vocabulary includes: Concerts, Flights, Sports,
Festivals, Conferences, Bus companies, Movie theaters, Museums, Tourist
attractions, Public transport, Universities, and Corporate events.

Applications should use those canonical strings (or the `Category` enum values
exported by the contract) and fall back to `Other` only when a label falls
outside the shared vocabulary. The contract and backend keep this list in sync;
no new label should be invented ad hoc.

## Contract-side vocabulary

The contract exports the typed `Category` enum and the constants
`CATEGORY_CONCERT`, `CATEGORY_FLIGHT`, `CATEGORY_SPORTS`,
`CATEGORY_FESTIVAL`, `CATEGORY_CONFERENCE`, and `CATEGORY_OTHER`. These are
the canonical in-contract labels used by clients, and they mirror the backend
source-of-truth list described above.
