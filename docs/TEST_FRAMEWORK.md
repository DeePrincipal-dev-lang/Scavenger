# Test Framework Helpers

Reusable utilities in `stellar-contract/tests/test_helpers.rs` to reduce boilerplate across the test suite.

## Available Helpers

### `setup_full(env) -> (client, admin, recycler, collector, manufacturer)`

Registers a contract, initializes admin, and pre-registers one participant of each role. This is the standard starting point for most tests.

```rust
let env = Env::default();
let (client, admin, recycler, collector, manufacturer) = test_helpers::setup_full(&env);
```

### `create_recyclers(env, client, n) -> Vec<Address>`

Generates and registers `n` recycler addresses. Useful for bulk/load tests.

```rust
let recyclers = test_helpers::create_recyclers(&env, &client, 10);
```

### `submit_waste(client, owner, waste_type, weight) -> u64`

Calls `recycle_waste` and returns the waste ID.

```rust
let waste_id = test_helpers::submit_waste(&client, &recycler, WasteType::Plastic, 5000);
```

### `assert_waste_owner(client, waste_id, expected_owner)`

Fetches the waste and asserts its `submitter` field matches `expected_owner`. Panics with a clear message on mismatch.

```rust
test_helpers::assert_waste_owner(&client, waste_id, &recycler);
```

### `create_test_incentive(client, manufacturer, waste_type, reward, budget) -> u64`

Creates an incentive and returns its ID.

```rust
let incentive_id = test_helpers::create_test_incentive(&client, &manufacturer, WasteType::Metal, 100, 5000);
```

## Example Usage

```rust
#![cfg(test)]
mod test_helpers;
use soroban_sdk::Env;
use stellar_scavngr_contract::WasteType;

#[test]
fn test_full_flow() {
    let env = Env::default();
    let (client, _admin, recycler, collector, manufacturer) = test_helpers::setup_full(&env);

    let waste_id = test_helpers::submit_waste(&client, &recycler, WasteType::Plastic, 5000);
    test_helpers::assert_waste_owner(&client, waste_id, &recycler);

    client.transfer_waste_v2(&waste_id, &recycler, &collector, &0, &0);
    client.transfer_waste_v2(&waste_id, &collector, &manufacturer, &0, &0);

    let incentive_id = test_helpers::create_test_incentive(&client, &manufacturer, WasteType::Plastic, 100, 10000);
    assert!(incentive_id > 0);
}
```

## Adding New Helpers

1. Add the function to `stellar-contract/tests/test_helpers.rs` with `pub fn`.
2. Keep the `#![cfg(test)]` attribute at the top of the file — helpers are test-only.
3. Use `soroban_sdk` types (`Address`, `Env`, `Vec`) rather than std equivalents where the contract API requires them.
4. Use `mod test_helpers;` in any test file that needs the helpers (the file must be in the same `tests/` directory).
