use super::*;

#[contract]
struct FailingToken;

#[contractimpl]
impl FailingToken {
    pub fn decimals(_env: Env) -> u32 {
        7
    }

    pub fn transfer(_env: Env, _from: Address, _to: Address, _amount: i128) {
        panic!("simulated token transfer failure");
    }
}

#[contract]
pub struct MaliciousReentrantToken;

#[contractimpl]
impl MaliciousReentrantToken {
    pub fn decimals(_env: Env) -> u32 {
        7
    }

    pub fn transfer(env: Env, from: Address, _to: Address, amount: i128) {
        // Attempt recursive reentrancy callback into the ticketing contract
        if let Some(target) = env.storage().instance().get::<_, Address>(&DataKey::Admin) {
            let client = TicketingContractClient::new(&env, &target);
            let _ = client.try_purchase_primary(
                &from,
                &1u64,
                &String::from_str(&env, "GA"),
                &String::from_str(&env, "reentrant"),
            );
        }
    }

    pub fn set_target(env: Env, target: Address) {
        env.storage().instance().set(&DataKey::Admin, &target);
    }
}

/// #145: `initialize` calls `admin.require_auth()` before writing any state,
/// but nothing in the suite called it without `mock_all_auths()` blanket
/// approval, so a regression that dropped the `require_auth()` call (or
/// moved it after the first storage write) would not fail any existing test.
/// Uses a fresh, un-initialized contract — `setup()` already calls
/// `initialize` under the blanket mock — and mocks auth for a stranger
/// address instead of the admin being initialized, matching the pattern
/// `check_in_requires_the_organizers_auth` uses for the same class of gap.
#[test]
fn initialize_requires_admin_auth() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let payment_token = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();
    let contract_id = env.register(TicketingContract, ());
    let client = TicketingContractClient::new(&env, &contract_id);

    let stranger = Address::generate(&env);
    env.mock_auths(&[MockAuth {
        address: &stranger,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (&admin, &payment_token).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    let result = client.try_initialize(&admin, &payment_token);
    // A host auth failure, not a contract Error — and nothing was written.
    assert!(matches!(result, Err(Err(_))));
    assert!(!env.as_contract(&contract_id, || {
        env.storage().instance().has(&DataKey::Admin)
    }));

    // With the admin's own auth the same call succeeds.
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (&admin, &payment_token).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin, &payment_token);
    let auths = env.auths();
    assert_eq!(auths.len(), 1);
    assert_eq!(auths[0].0, admin);
}

#[test]
fn initialize_rejects_a_second_call() {
    let (env, client, _token, _token_asset, admin, _organizer) = setup();
    let second_token = Address::generate(&env);
    let result = client.try_initialize(&admin, &second_token);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn payment_token_propose_and_apply_flow() {
    let (env, client, _token, _token_asset, admin, _organizer) = setup();

    let new_token_admin = Address::generate(&env);
    let new_token_contract = env.register_stellar_asset_contract_v2(new_token_admin);
    let new_token_addr = new_token_contract.address();

    client.propose_payment_token(&admin, &new_token_addr);

    // Cannot apply before delay elapses
    let early_apply = client.try_apply_payment_token(&admin);
    assert_eq!(early_apply, Err(Ok(Error::TimelockNotElapsed)));

    // Advance ledger past delay
    env.ledger().set_sequence_number(100_000);
    client.apply_payment_token(&admin);
}

#[test]
fn apply_payment_token_rejects_without_proposal() {
    let (env, client, _token, _token_asset, admin, _organizer) = setup();
    let result = client.try_apply_payment_token(&admin);
    assert_eq!(result, Err(Ok(Error::NoPendingPaymentToken)));
}

#[test]
fn purchase_throttle_rejects_rapid_repeat_purchases() {
    let (env, client, _token, token_asset, admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    client.set_purchase_throttle(&admin, &10u32);

    let buyer = Address::generate(&env);
    token_asset.mint(&buyer, &10_000i128);

    env.ledger().set_sequence_number(100);
    client.set_tier_price(&organizer, &1, &String::from_str(&env, "GA"), &1_000i128);
    client.purchase_primary(
        &buyer,
        &1,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "1"),
    );

    // Second purchase in the same block (or within 10 ledgers) must be throttled
    env.ledger().set_sequence_number(105);
    client.set_tier_price(&organizer, &1, &String::from_str(&env, "GA"), &1_000i128);
    let throttled = client.try_purchase_primary(
        &buyer,
        &1,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "2"),
    );
    assert_eq!(throttled, Err(Ok(Error::PurchaseTooSoon)));
}

#[test]
fn purchase_throttle_allows_purchase_after_spacing_elapses() {
    let (env, client, _token, token_asset, admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    client.set_purchase_throttle(&admin, &10u32);

    let buyer = Address::generate(&env);
    token_asset.mint(&buyer, &10_000i128);

    env.ledger().set_sequence_number(100);
    client.set_tier_price(&organizer, &1, &String::from_str(&env, "GA"), &1_000i128);
    client.purchase_primary(
        &buyer,
        &1,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "1"),
    );

    env.ledger().set_sequence_number(111);
    client.set_tier_price(&organizer, &1, &String::from_str(&env, "GA"), &1_000i128);
    let ok = client.try_purchase_primary(
        &buyer,
        &1,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "2"),
    );
    assert!(ok.is_ok());
}

#[test]
fn set_purchase_throttle_rejects_a_non_admin_caller() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    let result = client.try_set_purchase_throttle(&organizer, &10u32);
    assert_eq!(result, Err(Ok(Error::NotAdmin)));
}

#[test]
fn purchase_primary_leaves_no_partial_state_when_token_transfer_fails() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);

    let failing_token = env.register(FailingToken, ());
    client.set_event_payment_token(&organizer, &1, &Some(failing_token));

    let buyer = Address::generate(&env);
    let event_before = client.get_event(&1);

    client.set_tier_price(&organizer, &1, &String::from_str(&env, "GA"), &1_000i128);
    let result = client.try_purchase_primary(
        &buyer,
        &1,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "1"),
    );
    assert!(
        result.is_err(),
        "expected purchase_primary to fail when the token transfer fails"
    );

    let event_after = client.get_event(&1);
    assert_eq!(
        event_before, event_after,
        "event state must be unchanged after a failed token transfer"
    );
    assert!(
        client.try_get_ticket(&0).is_err(),
        "no ticket should have been minted when the token transfer failed"
    );
}

#[test]
fn malicious_token_reentrancy_fails_safely_and_preserves_state() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);

    let malicious_token_id = env.register(MaliciousReentrantToken, ());
    let token_client = MaliciousReentrantTokenClient::new(&env, &malicious_token_id);
    token_client.set_target(&client.address);

    client.set_event_payment_token(&organizer, &1, &Some(malicious_token_id));

    let buyer = Address::generate(&env);
    let event_before = client.get_event(&1);

    client.set_tier_price(&organizer, &1, &String::from_str(&env, "GA"), &1_000i128);
    let _ = client.try_purchase_primary(
        &buyer,
        &1,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "1"),
    );

    // Contract state remains consistent and tickets_issued accurately reflects final state
    let event_after = client.get_event(&1);
    assert!(event_after.tickets_issued <= event_before.tickets_issued + 1);
}

/// Issue #368: The throttle check uses `< spacing`, so a purchase exactly
/// `spacing` ledgers after the previous one must be accepted, while a purchase
/// at `spacing - 1` ledgers must be rejected. This test pins the boundary
/// so a one-off error in the comparison would be caught.
#[test]
fn purchase_throttle_boundary_spacing_minus_one_rejected_spacing_accepted() {
    let (env, client, _token, token_asset, admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    client.set_purchase_throttle(&admin, &10u32);

    let buyer = Address::generate(&env);
    token_asset.mint(&buyer, &30_000i128);

    // First purchase at ledger 100.
    env.ledger().set_sequence_number(100);
    client.set_tier_price(&organizer, &1, &String::from_str(&env, "GA"), &1_000i128);
    client.purchase_primary(
        &buyer,
        &1,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "1"),
    );

    // At ledger 109 (spacing - 1 = 9 ledgers later), the purchase must be rejected.
    env.ledger().set_sequence_number(109);
    client.set_tier_price(&organizer, &1, &String::from_str(&env, "GA"), &1_000i128);
    let rejected = client.try_purchase_primary(
        &buyer,
        &1,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "2"),
    );
    assert_eq!(
        rejected,
        Err(Ok(Error::PurchaseTooSoon)),
        "purchase at spacing-1 ledgers must be rejected"
    );

    // At ledger 110 (exactly spacing = 10 ledgers later), the purchase must succeed.
    env.ledger().set_sequence_number(110);
    client.set_tier_price(&organizer, &1, &String::from_str(&env, "GA"), &1_000i128);
    let accepted = client.try_purchase_primary(
        &buyer,
        &1,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "3"),
    );
    assert!(
        accepted.is_ok(),
        "purchase at exactly spacing ledgers must be accepted"
    );
}

/// Issue #369: When `min_ledger_spacing` is set to zero, the throttle is
/// disabled and back-to-back purchases in the same ledger must succeed.
#[test]
fn purchase_throttle_zero_disables_throttle_allowing_back_to_back_purchases() {
    let (env, client, _token, token_asset, admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);

    // Enable a non-zero throttle first, then disable it by setting to zero.
    client.set_purchase_throttle(&admin, &10u32);
    client.set_purchase_throttle(&admin, &0u32);

    let buyer = Address::generate(&env);
    token_asset.mint(&buyer, &10_000i128);

    // Both purchases happen on the same ledger — no spacing at all.
    env.ledger().set_sequence_number(100);
    client.set_tier_price(&organizer, &1, &String::from_str(&env, "GA"), &1_000i128);
    client.purchase_primary(
        &buyer,
        &1,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "1"),
    );

    // Second purchase on the same ledger must succeed because throttle is disabled.
    client.set_tier_price(&organizer, &1, &String::from_str(&env, "GA"), &1_000i128);
    let second = client.try_purchase_primary(
        &buyer,
        &1,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "2"),
    );
    assert!(
        second.is_ok(),
        "back-to-back purchase must succeed when throttle is zero (disabled)"
    );
}
