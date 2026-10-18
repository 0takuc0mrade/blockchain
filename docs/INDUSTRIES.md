# Supported industries

`Event.category` is a free-text field on-chain; the backend enforces
a closed set of twelve values so the marketplace and dashboard can
filter and label consistently:

Concerts, Flights, Sports, Festivals, Conferences, Bus companies,
Movie theaters, Museums, Tourist attractions, Public transport,
Universities, Corporate events.

Adding a thirteenth industry requires no contract change — only a
backend enum update (`Industry` in
[`StellarTickets/backend`](https://github.com/StellarTickets/backend)'s
Prisma schema) and a frontend label.

## Optional on-chain vocabulary

The contract exports the typed `Category` enum with an `Other` variant and
the constants `CATEGORY_CONCERT`, `CATEGORY_FLIGHT`, `CATEGORY_SPORTS`,
`CATEGORY_FESTIVAL`, `CATEGORY_CONFERENCE`, and `CATEGORY_OTHER`. These are
optional client/indexer conventions; `Event.category` remains free text for
ABI and migration compatibility. Applications should map known strings to a
`Category` value and use `Other` for labels outside the shared vocabulary.
