# FAQ

**Why not one contract per industry?**
See "Why one contract for every industry" in the root README —
category is metadata, not a separate code path.

**Can an organizer change the resale cap after tickets are sold?**
No — `max_resale_multiplier_bps` and `royalty_bps` are set once in
`create_event` and apply to every ticket under that event for its
lifetime.

**What is the accepted range for `max_resale_multiplier_bps`?**
10,000 bps (100%, face value) and above. 12,000 bps — the 120% used
throughout the docs and examples — allows a 20% markup. A value below
10,000 is rejected with `InvalidMultiplier`, because the resulting cap
would sit below face value and no resale could ever clear it.

**What are the byte limits on the name, category, tier and seat strings?**
`name` up to 64 bytes, `category` up to 32 bytes, and both `tier` and
`seat` up to 32 bytes. Empty labels are rejected with
`EmptyNameOrCategory` and oversized ones with `StringTooLong`, bounding
the per-ticket storage rent an organizer can incur.

**What happens to a ticket if the organizer account is compromised?**
Whoever controls the organizer's signing key can revoke or check in
tickets for that event — the contract has no separate recovery path.
Treat the organizer key with the same care as any other high-value
signing key.

**Is there a maximum number of tickets per event?**
No hard cap in the contract; `tickets_issued` is a `u64` counter.
