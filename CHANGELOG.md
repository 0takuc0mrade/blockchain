# Changelog

All notable changes to the `ticketing` contract are documented here.
This project follows [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Changed
- Upgrade `soroban-sdk` from 26 to 28. See [CONTRIBUTING.md](CONTRIBUTING.md)
  for the dependency versioning policy and upgrade procedure.

### Added
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
