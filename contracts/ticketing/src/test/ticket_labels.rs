//! Tier/seat length bounds for `issue_ticket` and `purchase_primary`
//! (Issue #126).
//!
//! Unbounded labels inflate per-ticket storage cost and rent: every issued
//! ticket persists `tier` and `seat`, so an organizer could mint arbitrarily
//! large strings at the protocol's expense. Both labels are now rejected when
//! empty or longer than `MAX_TICKET_LABEL_LEN`.

use super::*;

#[test]
fn issue_ticket_rejects_empty_tier() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let buyer = Address::generate(&env);

    let result = client.try_issue_ticket(
        &organizer,
        &1,
        &buyer,
        &String::from_str(&env, ""),
        &String::from_str(&env, "A12"),
        &1_000i128,
    );
    assert_eq!(result, Err(Ok(Error::EmptyNameOrCategory)));
}

#[test]
fn issue_ticket_rejects_empty_seat() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let buyer = Address::generate(&env);

    let result = client.try_issue_ticket(
        &organizer,
        &1,
        &buyer,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, ""),
        &1_000i128,
    );
    assert_eq!(result, Err(Ok(Error::EmptyNameOrCategory)));
}

#[test]
fn issue_ticket_rejects_oversized_tier() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let buyer = Address::generate(&env);

    let tier = "t".repeat(MAX_TICKET_LABEL_LEN as usize + 1);
    let result = client.try_issue_ticket(
        &organizer,
        &1,
        &buyer,
        &String::from_str(&env, &tier),
        &String::from_str(&env, "A12"),
        &1_000i128,
    );
    assert_eq!(result, Err(Ok(Error::StringTooLong)));
}

#[test]
fn issue_ticket_rejects_oversized_seat() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let buyer = Address::generate(&env);

    let seat = "s".repeat(MAX_TICKET_LABEL_LEN as usize + 1);
    let result = client.try_issue_ticket(
        &organizer,
        &1,
        &buyer,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, &seat),
        &1_000i128,
    );
    assert_eq!(result, Err(Ok(Error::StringTooLong)));
}

#[test]
fn issue_ticket_accepts_labels_at_the_maximum_length() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let buyer = Address::generate(&env);

    let tier = "t".repeat(MAX_TICKET_LABEL_LEN as usize);
    let seat = "s".repeat(MAX_TICKET_LABEL_LEN as usize);

    let ticket_id = client.issue_ticket(
        &organizer,
        &1,
        &buyer,
        &String::from_str(&env, &tier),
        &String::from_str(&env, &seat),
        &1_000i128,
    );

    let ticket = client.get_ticket(&ticket_id);
    assert_eq!(ticket.tier.len(), MAX_TICKET_LABEL_LEN);
    assert_eq!(ticket.seat.len(), MAX_TICKET_LABEL_LEN);
}

#[test]
fn purchase_primary_rejects_oversized_tier() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let buyer = Address::generate(&env);

    let tier = "t".repeat(MAX_TICKET_LABEL_LEN as usize + 1);
    let result = client.try_purchase_primary(
        &buyer,
        &1,
        &String::from_str(&env, &tier),
        &String::from_str(&env, "A12"),
    );
    assert_eq!(result, Err(Ok(Error::StringTooLong)));
}

#[test]
fn purchase_primary_rejects_empty_seat() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let buyer = Address::generate(&env);

    let result = client.try_purchase_primary(
        &buyer,
        &1,
        &String::from_str(&env, "GA"),
        &String::from_str(&env, ""),
    );
    assert_eq!(result, Err(Ok(Error::EmptyNameOrCategory)));
}

#[test]
fn purchase_primary_accepts_labels_at_the_maximum_length() {
    let (env, client, _token, token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let buyer = Address::generate(&env);
    token_asset.mint(&buyer, &10_000i128);

    let tier = "t".repeat(MAX_TICKET_LABEL_LEN as usize);
    let seat = "s".repeat(MAX_TICKET_LABEL_LEN as usize);
    client.set_tier_price(&organizer, &1, &String::from_str(&env, &tier), &1_000i128);

    let ticket_id = client.purchase_primary(
        &buyer,
        &1,
        &String::from_str(&env, &tier),
        &String::from_str(&env, &seat),
    );

    let ticket = client.get_ticket(&ticket_id);
    assert_eq!(ticket.tier.len(), MAX_TICKET_LABEL_LEN);
    assert_eq!(ticket.seat.len(), MAX_TICKET_LABEL_LEN);
}
