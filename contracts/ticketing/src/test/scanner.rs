use super::*;
use soroban_sdk::testutils::storage::{Instance as _, Persistent as _};

fn scan(env: &Env, client: &TicketingContractClient, id: u64, owner: &Address) -> bool {
    env.mock_auths(&[]);
    let valid = client.is_valid(&id, owner);
    assert!(env.auths().is_empty());
    env.mock_all_auths();
    valid
}

fn scan_status(env: &Env, client: &TicketingContractClient, id: u64) -> bool {
    env.mock_auths(&[]);
    let valid = client.is_ticket_valid(&id);
    assert!(env.auths().is_empty());
    env.mock_all_auths();
    valid
}

#[test]
fn scanner_tracks_listing_transfer_revocation_and_check_in() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let owner = Address::generate(&env);
    let recipient = Address::generate(&env);
    let id = issue_sample_ticket(&env, &client, &organizer, 1, &owner, 1_000);

    assert!(scan(&env, &client, id, &owner));
    assert!(scan_status(&env, &client, id));
    assert!(!scan(&env, &client, id, &recipient));
    client.list_for_resale(&owner, &id, &1_100);
    assert!(!scan(&env, &client, id, &owner));
    assert!(!scan_status(&env, &client, id));
    client.cancel_resale(&owner, &id);
    assert!(scan(&env, &client, id, &owner));
    assert!(scan_status(&env, &client, id));
    client.transfer_ticket(&owner, &id, &recipient);
    assert!(!scan(&env, &client, id, &owner));
    assert!(scan(&env, &client, id, &recipient));
    assert!(scan_status(&env, &client, id));
    client.revoke_ticket(&organizer, &id);
    assert!(!scan(&env, &client, id, &recipient));
    assert!(!scan_status(&env, &client, id));

    let used = issue_sample_ticket(&env, &client, &organizer, 1, &owner, 1_000);
    client.check_in(&organizer, &used);
    assert!(!scan(&env, &client, used, &owner));
    assert!(!scan_status(&env, &client, used));
}

#[test]
fn scanner_preserves_lookup_errors_and_owner_check_compatibility() {
    let (env, client, _token, _admin, _organizer) = setup_uninitialized();
    let owner = Address::generate(&env);
    assert!(!scan(&env, &client, 999, &owner));
    env.mock_auths(&[]);
    assert_eq!(
        client.try_is_ticket_valid(&999),
        Err(Ok(Error::NotInitialized))
    );

    let (env, client, _token, _token_asset, _admin, _organizer) = setup();
    let owner = Address::generate(&env);
    assert!(!scan(&env, &client, 999, &owner));
    env.mock_auths(&[]);
    assert_eq!(
        client.try_is_ticket_valid(&999),
        Err(Ok(Error::TicketNotFound))
    );
}

#[test]
fn scanner_preserves_invalid_lifecycle_errors() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let owner = Address::generate(&env);
    let id = issue_sample_ticket(&env, &client, &organizer, 1, &owner, 1_000);
    env.as_contract(&client.address, || {
        let key = DataKey::Ticket(id);
        let mut stored: StoredTicket = env.storage().persistent().get(&key).unwrap();
        stored.lifecycle = 255;
        env.storage().persistent().set(&key, &stored);
    });

    env.mock_auths(&[]);
    assert_eq!(
        client.try_is_ticket_valid(&id),
        Err(Ok(Error::InvalidTicketLifecycle))
    );
    assert!(!client.is_valid(&id, &owner));
}

#[test]
fn scanner_reads_leave_ticket_state_and_storage_ttl_unchanged() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let owner = Address::generate(&env);
    let id = issue_sample_ticket(&env, &client, &organizer, 1, &owner, 1_000);
    env.ledger().set_sequence_number(40_000);
    let before = client.get_ticket(&id);
    let ttls = || {
        env.as_contract(&client.address, || {
            (
                env.storage().persistent().get_ttl(&DataKey::Ticket(id)),
                env.storage().instance().get_ttl(),
            )
        })
    };
    let before_ttls = ttls();

    assert!(scan(&env, &client, id, &owner));
    assert!(scan_status(&env, &client, id));
    assert!(!scan(&env, &client, 999, &owner));
    assert_eq!(client.get_ticket(&id), before);
    assert_eq!(ttls(), before_ttls);
}
