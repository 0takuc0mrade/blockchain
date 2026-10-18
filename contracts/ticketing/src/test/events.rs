use super::*;

#[test]
fn get_event_reports_not_found_for_an_unknown_id() {
    let (env, client, _token, _token_asset, _admin, _organizer) = setup();
    let result = client.try_get_event(&999);
    assert_eq!(result, Err(Ok(Error::EventNotFound)));
}

#[test]
fn create_event_rejects_a_duplicate_event_id() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let duplicate = client.try_create_event(
        &organizer,
        &1,
        &EventParams {
            name: String::from_str(&env, "Other Event"),
            category: String::from_str(&env, "concert"),
            max_resale_multiplier_bps: 10_000,
            royalty_bps: 0,
            starts_at: 10_000,
            transfer_freeze_seconds: 0,
            resale_cutoff_seconds: 0,
        },
    );
    assert_eq!(duplicate, Err(Ok(Error::EventAlreadyExists)));
}

#[test]
fn create_event_rejects_a_royalty_above_10_000_bps() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    let bad_royalty = client.try_create_event(
        &organizer,
        &1,
        &EventParams {
            name: String::from_str(&env, "Event"),
            category: String::from_str(&env, "concert"),
            max_resale_multiplier_bps: 12_000,
            royalty_bps: 10_001, // >100%
            starts_at: 10_000,
            transfer_freeze_seconds: 0,
            resale_cutoff_seconds: 0,
        },
    );
    assert_eq!(bad_royalty, Err(Ok(Error::InvalidRoyalty)));
}

#[test]
fn create_event_rejects_past_or_current_timestamp() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    env.ledger().set_timestamp(1_000);

    let in_the_past = client.try_create_event(
        &organizer,
        &1,
        &EventParams {
            name: String::from_str(&env, "Past Event"),
            category: String::from_str(&env, "concert"),
            max_resale_multiplier_bps: 10_000,
            royalty_bps: 0,
            starts_at: 999,
            transfer_freeze_seconds: 0,
            resale_cutoff_seconds: 0,
        },
    );
    assert_eq!(in_the_past, Err(Ok(Error::InvalidEventTime)));

    let right_now = client.try_create_event(
        &organizer,
        &2,
        &EventParams {
            name: String::from_str(&env, "Now Event"),
            category: String::from_str(&env, "concert"),
            max_resale_multiplier_bps: 10_000,
            royalty_bps: 0,
            starts_at: 1_000,
            transfer_freeze_seconds: 0,
            resale_cutoff_seconds: 0,
        },
    );
    assert_eq!(right_now, Err(Ok(Error::InvalidEventTime)));
}

#[test]
fn organizer_can_run_multiple_independent_events() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    make_event(&env, &client, &organizer, 2);

    let buyer = Address::generate(&env);
    client.issue_ticket(
        &organizer,
        &1,
        &buyer,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "A1"),
        &1_000i128,
    );

    let ev1 = client.get_event(&1);
    let ev2 = client.get_event(&2);
    assert_eq!(ev1.tickets_issued, 1);
    assert_eq!(ev2.tickets_issued, 0);
}

#[test]
fn events_with_identical_names_have_independent_state() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();

    let event_name = String::from_str(&env, "Summer Music Festival 2026");
    let category = String::from_str(&env, "concert");

    client.create_event(
        &organizer,
        &101u64,
        &EventParams {
            name: event_name.clone(),
            category: category.clone(),
            max_resale_multiplier_bps: 12_000,
            royalty_bps: 500,
            starts_at: 10_000,
            transfer_freeze_seconds: 100,
            resale_cutoff_seconds: 200,
        },
    );

    client.create_event(
        &organizer,
        &102u64,
        &EventParams {
            name: event_name,
            category,
            max_resale_multiplier_bps: 15_000,
            royalty_bps: 1_000,
            starts_at: 20_000,
            transfer_freeze_seconds: 150,
            resale_cutoff_seconds: 300,
        },
    );

    let event1 = client.get_event(&101u64);
    let event2 = client.get_event(&102u64);

    assert_eq!(event1.name, event2.name);
    assert_eq!(event1.category, event2.category);
    assert_ne!(event1.starts_at, event2.starts_at);
    assert_ne!(
        event1.max_resale_multiplier_bps,
        event2.max_resale_multiplier_bps
    );
    assert_ne!(event1.royalty_bps, event2.royalty_bps);

    let buyer1 = Address::generate(&env);
    let buyer2 = Address::generate(&env);

    let ticket1 = client.issue_ticket(
        &organizer,
        &101u64,
        &buyer1,
        &String::from_str(&env, "VIP"),
        &String::from_str(&env, "Row 1"),
        &10_000i128,
    );

    let ticket2 = client.issue_ticket(
        &organizer,
        &102u64,
        &buyer2,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "Standing"),
        &5_000i128,
    );

    assert_eq!(client.get_event(&101u64).tickets_issued, 1);
    assert_eq!(client.get_event(&102u64).tickets_issued, 1);

    client.check_in(&organizer, &ticket1);
    assert_eq!(client.verify_ticket(&ticket1).status, TicketStatus::Used);
    assert_eq!(client.verify_ticket(&ticket2).status, TicketStatus::Valid);
}

#[test]
fn events_and_tickets_with_unicode_names_and_categories() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();

    let unicode_name = String::from_str(&env, "東京ライブ 2026 🎵 (Tokyo Live)");
    let unicode_category = String::from_str(&env, "音楽・コンサート / Festival ✨");

    client.create_event(
        &organizer,
        &200u64,
        &EventParams {
            name: unicode_name.clone(),
            category: unicode_category.clone(),
            max_resale_multiplier_bps: 12_000,
            royalty_bps: 500,
            starts_at: 10_000,
            transfer_freeze_seconds: 100,
            resale_cutoff_seconds: 200,
        },
    );

    let event = client.get_event(&200u64);
    assert_eq!(event.name, unicode_name);
    assert_eq!(event.category, unicode_category);

    let buyer = Address::generate(&env);
    let unicode_tier = String::from_str(&env, "特別席 🌟 VIP");
    let unicode_seat = String::from_str(&env, "中央列-12番 🎟️");

    let ticket_id = client.issue_ticket(
        &organizer,
        &200u64,
        &buyer,
        &unicode_tier,
        &unicode_seat,
        &8_888i128,
    );

    let ticket = client.verify_ticket(&ticket_id);
    assert_eq!(ticket.tier, unicode_tier);
    assert_eq!(ticket.seat, unicode_seat);
    assert_eq!(ticket.owner, buyer);
    assert_eq!(ticket.status, TicketStatus::Valid);
}

#[test]
fn lottery_allocates_requested_number_of_tickets() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);

    let e1 = Address::generate(&env);
    let e2 = Address::generate(&env);
    let e3 = Address::generate(&env);
    let mut entrants = Vec::new(&env);
    entrants.push_back(e1);
    entrants.push_back(e2);
    entrants.push_back(e3);

    let winners = client.allocate_lottery(
        &organizer,
        &1,
        &entrants,
        &2u32,
        &String::from_str(&env, "VIP"),
        &0i128,
    );
    assert_eq!(winners.len(), 2);
    assert_eq!(client.get_event(&1).tickets_issued, 2);
}

#[test]
fn lottery_rejects_more_winners_than_entrants() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);

    let mut entrants = Vec::new(&env);
    entrants.push_back(Address::generate(&env));

    let result = client.try_allocate_lottery(
        &organizer,
        &1,
        &entrants,
        &2u32,
        &String::from_str(&env, "VIP"),
        &0i128,
    );
    assert_eq!(result, Err(Ok(Error::InvalidLottery)));
}

#[test]
fn lottery_rejects_duplicate_entrants() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);

    let dup = Address::generate(&env);
    let mut entrants = Vec::new(&env);
    entrants.push_back(dup.clone());
    entrants.push_back(dup);

    let result = client.try_allocate_lottery(
        &organizer,
        &1,
        &entrants,
        &1u32,
        &String::from_str(&env, "VIP"),
        &0i128,
    );
    assert_eq!(result, Err(Ok(Error::InvalidLottery)));
}

#[test]
fn escrowed_primary_sale_holds_funds_in_the_contract() {
    let (env, client, token, token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    client.enable_escrow(&organizer, &1, &500u32);

    let buyer = Address::generate(&env);
    token_asset.mint(&buyer, &10_000i128);

    client.purchase_primary(
        &buyer,
        &1,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "1"),
        &2_000i128,
    );

    let event = client.get_event(&1);
    assert_eq!(event.escrow_balance, 2_000);
    assert_eq!(token.balance(&organizer), 0);
    assert_eq!(token.balance(&client.address), 2_000);
}

#[test]
fn release_escrow_rejects_before_the_event_ends() {
    let (env, client, _token, token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    client.enable_escrow(&organizer, &1, &500u32);

    let buyer = Address::generate(&env);
    token_asset.mint(&buyer, &10_000i128);
    client.purchase_primary(
        &buyer,
        &1,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "1"),
        &2_000i128,
    );

    env.ledger().set_sequence_number(499);
    let result = client.try_release_escrow(&organizer, &1);
    assert_eq!(result, Err(Ok(Error::EventNotEnded)));
}

#[test]
fn release_escrow_pays_the_organizer_after_the_event_ends() {
    let (env, client, token, token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    client.enable_escrow(&organizer, &1, &500u32);

    let buyer = Address::generate(&env);
    token_asset.mint(&buyer, &10_000i128);
    client.purchase_primary(
        &buyer,
        &1,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "1"),
        &2_000i128,
    );

    env.ledger().set_sequence_number(500);
    client.release_escrow(&organizer, &1);

    assert_eq!(token.balance(&organizer), 2_000);
    assert_eq!(token.balance(&client.address), 0);
    assert_eq!(client.get_event(&1).escrow_balance, 0);
}

#[test]
fn release_escrow_rejects_a_non_escrow_event() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let result = client.try_release_escrow(&organizer, &1);
    assert_eq!(result, Err(Ok(Error::EscrowNotEnabled)));
}

#[test]
fn enable_escrow_rejects_once_tickets_have_been_sold() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let buyer = Address::generate(&env);
    client.issue_ticket(
        &organizer,
        &1,
        &buyer,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "1"),
        &1_000i128,
    );
    let result = client.try_enable_escrow(&organizer, &1, &500u32);
    assert_eq!(result, Err(Ok(Error::EventAlreadyStarted)));
}

#[test]
fn per_event_payment_token_routes_settlement() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);

    let token2_admin = Address::generate(&env);
    let token2_contract = env.register_stellar_asset_contract_v2(token2_admin);
    let token2 = TokenClient::new(&env, &token2_contract.address());
    let token2_asset = StellarAssetClient::new(&env, &token2_contract.address());

    client.set_event_payment_token(&organizer, &1, &Some(token2_contract.address()));
    assert_eq!(client.event_payment_token(&1), token2_contract.address());

    let buyer = Address::generate(&env);
    token2_asset.mint(&buyer, &5_000i128);

    client.purchase_primary(
        &buyer,
        &1,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "1"),
        &2_000i128,
    );

    assert_eq!(token2.balance(&organizer), 2_000);
    assert_eq!(token2.balance(&buyer), 3_000);
}

#[test]
fn per_event_payment_token_rejects_change_after_tickets_issued() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);

    let buyer = Address::generate(&env);
    client.issue_ticket(
        &organizer,
        &1,
        &buyer,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, "1"),
        &1_000i128,
    );

    let token2_admin = Address::generate(&env);
    let token2_contract = env.register_stellar_asset_contract_v2(token2_admin);
    let err = client.try_set_event_payment_token(&organizer, &1, &Some(token2_contract.address()));
    assert_eq!(err, Err(Ok(Error::TicketsAlreadyIssued)));
}
