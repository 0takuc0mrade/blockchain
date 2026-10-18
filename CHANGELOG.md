# Changelog

All notable changes to the `ticketing` contract are documented here.
This project follows [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Fixed
- Repair merged organizer approval and packed ticket lookup definitions, and
  update a stale event-creation test so the contract builds again.

### Changed
- Centralize organizer authorization and terminal ticket status checks without
  changing operation-specific revocation rules (#348, #349).
- Keep ticket lifecycle encoding with the types in `types.rs`, preserving
  crate-root exports and byte-identical WASM for the isolated move (#347).
- Upgrade `soroban-sdk` from 26 to 28. See [CONTRIBUTING.md](CONTRIBUTING.md)
  for the dependency versioning policy and upgrade procedure.

### Added
- Status-only scanner query `is_ticket_valid(ticket_id) -> Result<bool, Error>`
  with lifecycle, authorization-free read, and TTL tests. The existing
  owner-aware `is_valid(ticket_id, owner)` ABI is preserved (#350).
- Admin-managed organizer allowlist (`approve_organizer`,
  `revoke_organizer`, `is_approved_organizer`). `create_event` and
  `create_event_with_options` now reject callers that are not approved
  with `OrganizerNotApproved`, which stops event-id squatting (#135). See
  [docs/ORGANIZER_ALLOWLIST.md](docs/ORGANIZER_ALLOWLIST.md).
- Authorization test matrix covering every mutating entry point (#136).
- Initial `ticketing` contract: event registration, issuance, primary
  sale, transfer, on-chain verification, check-in, revocation, and a
  resale marketplace with anti-scalping price caps and organizer
  royalties.
