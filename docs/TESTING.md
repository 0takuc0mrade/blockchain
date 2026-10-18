# Testing

The test suite in
[`contracts/ticketing/src/test/`](../contracts/ticketing/src/test/)
uses `soroban_sdk::testutils` with `env.mock_all_auths()`, so every
`require_auth()` call succeeds without needing real signatures — this
is standard practice for Soroban unit tests and keeps focus on
contract logic rather than signature mechanics.

Coverage includes:

- Happy paths for every entry point (issue, purchase, transfer,
  check-in, revoke, list/cancel/buy resale)
- Authorization failures (wrong organizer, non-owner)
- A signature matrix
  ([`test/auth_matrix.rs`](../contracts/ticketing/src/test/auth_matrix.rs))
  that calls every mutating entry point with no auths mocked, asserts it
  fails with a host auth error, then repeats the call signed and asserts
  the expected signer's authorization was recorded. Add a row there
  whenever a mutating entry point is added.
- The organizer allowlist for event creation
  ([`test/organizer_allowlist.rs`](../contracts/ticketing/src/test/organizer_allowlist.rs))
- State-machine violations (double check-in, revoked-ticket actions,
  listing/buying when not eligible)
- Boundary conditions (resale price cap, royalty bounds, negative and
  zero prices)
- Multi-event and multi-ticket scenarios

Run the full suite with:

```bash
cargo test -p stellar-tickets-ticketing
```

## Snapshot policy

Soroban test snapshots live in
`contracts/ticketing/test_snapshots/test/`. They record authorization and
storage state and are reviewed source artifacts, not disposable build output.
Keep snapshots that correspond to active tests; remove a snapshot only when
its test is intentionally removed or renamed. After changing contract state
or authorization behavior, run the relevant test and review the generated
snapshot diff before committing it. Do not delete the snapshot directory or
regenerate every snapshot as a cleanup shortcut.

Use a focused test command while updating a snapshot:

```bash
cargo test -p stellar-tickets-ticketing <test_name>
```
