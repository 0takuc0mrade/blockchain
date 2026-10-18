# Contributing to StellarTickets/blockchain

Thanks for considering a contribution to the `ticketing` contract.

## Development setup

```bash
rustup target add wasm32v1-none
cargo test -p stellar-tickets-ticketing
```

## Before opening a PR

- `cargo fmt --all`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --workspace`
- `stellar contract build` to confirm the wasm still builds

## Dependency versioning

### soroban-sdk pinning policy

The workspace pins `soroban-sdk` to a major version in [`Cargo.toml`](Cargo.toml):

```toml
[workspace.dependencies]
soroban-sdk = "26"  # major version only, allows patch/minor updates
```

**Why:** Version bumps can change behavior, so updates should be intentional and
reviewed. Pinning to major version allows automatic patch updates (e.g., 26.0 →
26.1.1) while requiring explicit review for minor versions (26 → 27).

**When upgrading soroban-sdk:**
1. Bump the version in `Cargo.toml`
2. Run `cargo test --workspace` to verify all tests pass
3. Run `cargo clippy --all-targets -- -D warnings` to check for lint breaks
4. Run `stellar contract build` to confirm WASM still builds
5. Add a changelog entry documenting the upgrade and any fixes needed
6. Open a PR describing what changed and why the upgrade was necessary

## Commit style

Keep commits scoped to one logical change. Prefer imperative subject
lines ("Add resale price cap test" not "Added" or "Adding").

## Writing contract tests

Tests are organized in [`contracts/ticketing/src/test/`](contracts/ticketing/src/test/) and follow these patterns:

### Setup function
Use a `setup()` helper to initialize the test environment:

```rust
use soroban_sdk::testutils::{Address as _, Ledger, MockAuth};

fn setup<'a>() -> (Env, TicketingContractClient<'a>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();  // allows any address to authorize
    
    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    
    let contract_id = env.register(TicketingContract, ());
    let client = TicketingContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_contract.address());
    
    (env, client, admin, organizer)
}
```

### Testing authorization with mock_auths
For functions requiring specific signatures, use `MockAuth`:

```rust
#[test]
fn only_organizer_can_create_event() {
    let (env, client, _admin, organizer) = setup();
    let unauthorized = Address::generate(&env);
    
    // This succeeds — organizer is authorized
    env.set_auths(&[MockAuth {
        address: organizer.clone(),
        invoke: MockAuthInvoke {
            contract: contract_id,
            fn_name: "create_event",
            args: (...),
            sub_invokes: vec![],
        },
    }]);
    client.create_event(&organizer, ...);
    
    // This fails — unauthorized caller
    env.set_auths(&[MockAuth {
        address: unauthorized.clone(),
        invoke: MockAuthInvoke { ... },
    }]);
    let result = client.try_create_event(&unauthorized, ...);
    assert_eq!(result, Err(Ok(Error::NotOrganizer)));
}
```

### Testing error cases
Use `try_*` methods to capture errors:

```rust
#[test]
fn revoked_ticket_cannot_transfer() {
    let (env, client, _admin, organizer) = setup();
    let owner = Address::generate(&env);
    
    let ticket_id = client.issue_ticket(&organizer, &1, &owner, ...);
    client.revoke_ticket(&organizer, &ticket_id);
    
    let result = client.try_transfer_ticket(&owner, &ticket_id, &Address::generate(&env));
    assert_eq!(result, Err(Ok(Error::Revoked)));
}
```

### Ledger and time-based tests
For tests involving ledger height or timestamps:

```rust
#[test]
fn event_respects_ledger_ttl() {
    let env = Env::default();
    // Advance ledger
    env.ledger().with_mut(|l| {
        l.sequence_number = 1000;
    });
    // ... your test
}
```

Each test should be focused, use generated addresses for isolation, and include assertions that verify both happy paths and error boundaries.

## Reporting issues

Open a GitHub issue with a minimal reproduction — for contract bugs,
a failing test case is the most useful thing you can attach.
