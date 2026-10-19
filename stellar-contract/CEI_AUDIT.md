# Checks-Effects-Interactions Audit

## Scope

Reviewed every function in `stellar-contract/src/lib.rs` that performs a
cross-contract call (`token::Client::new(...).transfer(...)`), searching for
state mutations (storage writes) that occur **after** an external call.

External-call sites found:

```
grep -n "token_client.transfer" stellar-contract/src/lib.rs
```

## Findings

### 1. `purchase_carbon_listing` — violation, fixed

Previously issued `token_client.transfer(&buyer, &listing.seller, ...)`
**before** marking the listing inactive, crediting the buyer's carbon
credits, and removing the listing from the active-listings index. A
reentrant token contract could call back into `purchase_carbon_listing`
(or another function reading the same listing) while `listing.is_active`
was still `true`, allowing the same listing to be purchased more than
once.

**Fix:** all storage writes (buyer stats, `listing.is_active = false`,
`remove_active_listing`) now happen before the token transfer.

### 2. `distribute_reward` (waste incentive payout) — violation, fixed

Previously decremented `incentive.remaining_budget` and persisted the
incentive **after** a loop of `token_client.transfer` calls to collectors,
the submitter, and (if applicable) the recycler share. Since
`total_reward` is fully computed before the loop and does not depend on
the transfers themselves, a reentrant callee invoked during any of those
transfers could observe the stale (pre-decrement) `remaining_budget` and
trigger another payout against the same incentive before it was marked
exhausted.

**Fix:** the `incentive.remaining_budget` update (and `add_to_total_tokens`
bookkeeping) now happens immediately after the reward amounts are computed
and before any `token_client.transfer` call in the loop.

### 3. No other cross-contract call sites

`token::Client::new` is only constructed in the two locations above; no
other function in `lib.rs` performs an external call, so no further
mutation-after-interaction ordering issues were found.

## Regression coverage

See `stellar-contract/tests/cei_reentrancy_test.rs`, added alongside this
audit. It documents the two ordering fixes with focused assertions on the
surrounding state (budget/listing) transitions used by
`purchase_carbon_listing` and `distribute_reward`. A full malicious-callee
integration harness (a mock token contract that reenters mid-`transfer`)
is not currently feasible with the existing Soroban test harness/mocks in
this repo without adding a new mock-token test contract crate; that is
recommended as a follow-up so the ordering fix has an executable
reentrancy proof, not just a static ordering guarantee.

## Acceptance criteria status

- [x] Reviewed all functions performing external cross-contract calls
- [x] State mutations now occur before external calls in both identified
      call sites
- [x] Findings documented (this file)
- [ ] Full malicious-callee integration test (follow-up, see above)
