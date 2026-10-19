# Fuzzing Guide for Stellar Scavenger Contract

This document describes how to set up, run, and maintain fuzz tests for the Scavenger Soroban smart contract.

## Quick Start

### Run All Fuzz Tests

```bash
cargo test --test 'fuzz_*'
```

### Run a Specific Fuzz Test

```bash
cargo test --test fuzz_transfer_path_validation
```

### Run Fuzz Tests with Extended Corpus

```bash
PROPTEST_CASES=1000 cargo test --test 'fuzz_*'
```

## Fuzz Testing Framework

We use **proptest 1.x** for property-based fuzz testing:

- **Deterministic:** Tests are repeatable with the same seed
- **Automatic Shrinking:** Finds minimal failing inputs
- **Regression Tracking:** Maintains corpus of failing cases

## Fuzz Targets

### Transfer Path Validation (`tests/fuzz_transfer_path_validation.rs`)

The most critical fuzz target for the contract's core logic.

**What's Tested:**
- All role combinations (Recycler, Collector, Manufacturer)
- Valid transfer paths return true
- Invalid transfer paths return false
- Unregistered and deregistered participant handling
- Multi-hop waste transfers
- Coordinate boundary conditions (lat: -90~90, lon: -180~180)

**Strategies:**
- `valid_role_strategy()`: Generates 0 (Recycler), 1 (Collector), 2 (Manufacturer)
- `coordinate_strategy()`: Generates -180..180
- `latitude_strategy()`: Generates -90..90

**Test Cases:** ~10 property tests, 256 cases each (default)

**Expected Duration:** 2-5 seconds

**Key Invariants:**
1. If from_role=0 and to_role ∈ {1,2}: must return true
2. If from_role=1 and to_role=2: must return true
3. All other combinations: must return false
4. Any unregistered participant: must return false
5. Any deregistered participant: must return false

### Other Fuzz Targets

- `tests/fuzz_waste_transfer.rs` - Waste transfer operations with boundary IDs/notes
- `tests/fuzz_participant_registration.rs` - Participant registration with various roles
- `tests/fuzz_state_transitions.rs` - State machine correctness
- `tests/fuzz_waste_submission.rs` - Waste submission operations
- `tests/fuzz_incentive_creation.rs` - Incentive creation scenarios

## Corpus Management

### Regression Test Cases

Proptest stores failing cases in `.proptest-regressions` files:

```
stellar-contract/tests/reward_fuzz_test.proptest-regressions
stellar-contract/tests/fuzz_transfer_path_validation.proptest-regressions (auto-created on failure)
```

These are **checked into git** to prevent regressions.

### Minimal Reproducible Examples

Example regression file content:

```
# Seeds for the `fuzz_transfer_path_valid_routes` test
prop_assert!(result == true);
cc 3e3e3e3e3e3e00
```

### Updating Regressions

If you intentionally change behavior:

```bash
# Accept new regression cases
PROPTEST_REGRESSIONS=accept cargo test --test fuzz_transfer_path_validation
```

**WARNING:** Always review changes before accepting new regressions.

## Running Fuzz Tests

### Local Development

```bash
# Quick sanity check (50 cases per test)
PROPTEST_CASES=50 cargo test --test 'fuzz_*'

# Standard run (256 cases per test)
cargo test --test 'fuzz_*'

# Extended run (1000 cases per test)
PROPTEST_CASES=1000 cargo test --test 'fuzz_*'

# Single-threaded (for deterministic debugging)
cargo test --test fuzz_transfer_path_validation -- --test-threads=1
```

### Continuous Integration

GitHub Actions runs all fuzz tests:

```bash
cd stellar-contract
cargo test --test 'fuzz_*' -- --nocapture --test-threads=1
```

### Performance Characteristics

| Configuration | Duration | Use Case |
|---|---|---|
| PROPTEST_CASES=50 | ~1-2s per test | Quick feedback, local dev |
| PROPTEST_CASES=256 (default) | ~2-5s per test | Normal testing |
| PROPTEST_CASES=1000 | ~10-30s per test | Before release, regression detection |
| PROPTEST_CASES=10000 | ~2-5 min per test | Deep fuzzing, overnight runs |

## Triaging Failures

### When a Fuzz Test Fails

```
thread 'fuzz_transfer_path_valid_routes' panicked at 'called `Result::unwrap()` on an `Err` value'
 May have been triggered by test case: cc3e3e3e3e3e3e
```

### Steps to Investigate

1. **Locate the seed:** Look for "triggered by test case" in output
2. **Reproduce deterministically:**

```bash
PROPTEST_RNG_SEED=0xCC3E3E3E3E3E3E cargo test --test fuzz_transfer_path_validation
```

3. **Review regression file** for minimal input:

```bash
cat stellar-contract/tests/fuzz_transfer_path_validation.proptest-regressions
```

4. **Create unit test** with minimal case:

```rust
#[test]
fn test_regression_transfer_path_case_1() {
    let (env, client) = setup();
    let recycler = register(&client, &env, ParticipantRole::Recycler);
    let collector = register(&client, &env, ParticipantRole::Collector);
    assert!(client.is_valid_transfer(&recycler, &collector));
}
```

5. **Fix contract logic** to handle the edge case
6. **Verify fix:** Run both unit test and fuzz test
7. **Accept regression** once fix is verified:

```bash
PROPTEST_REGRESSIONS=accept cargo test --test fuzz_transfer_path_validation
```

## Adding New Fuzz Targets

### Template for New Fuzz Test

```rust
// stellar-contract/tests/fuzz_new_feature.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn fuzz_my_feature(input in 0u64..1000u64) {
        // Setup
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        // Execute with fuzzy input
        let result = std::panic::catch_unwind(|| {
            // Your contract call here
        });

        // Assert invariant
        prop_assert!(result.is_ok());
    }
}
```

### Best Practices

1. **One test per feature area** - Create `fuzz_feature_name.rs` files
2. **Focus on invariants** - What must always be true?
3. **Use appropriate strategies** - Don't test the entire input space inefficiently
4. **Document expected behavior** - Add comments explaining what's valid
5. **Keep tests fast** - Aim for <100ms per test run

## Debugging Tips

### Print Strategy Distribution

```bash
PROPTEST_VERBOSE=1 cargo test --test fuzz_transfer_path_validation
```

### Disable Shrinking (faster failure analysis)

```bash
PROPTEST_MAX_SHRINK_ITERS=0 cargo test --test fuzz_transfer_path_validation
```

### Fixed Seed for Reproducibility

```bash
PROPTEST_RNG_SEED=42 cargo test --test fuzz_transfer_path_validation
```

## CI/CD Integration

### GitHub Actions Workflow

Add to `.github/workflows/contract-tests.yml`:

```yaml
- name: Run Fuzz Tests
  run: |
    cd stellar-contract
    PROPTEST_CASES=256 cargo test --test 'fuzz_*' -- --nocapture
```

### Required Checks

- All fuzz tests pass locally before PR
- Regression tests pass in CI
- No new panics or crashes

## Common Issues

### Issue: "thread panicked at 'Could not create regression file'"

**Solution:** Ensure write permissions in `stellar-contract/tests/` directory

```bash
chmod +w stellar-contract/tests/
```

### Issue: Test Timeout

**Solution:** Reduce cases or check for infinite loops

```bash
# Reduce test cases
PROPTEST_CASES=50 cargo test --test fuzz_transfer_path_validation

# Check for infinite loops in new code
```

### Issue: Flaky Tests

**Solution:** Identify and fix non-deterministic behavior

```bash
# Run multiple times with different seeds
for i in {1..10}; do
  PROPTEST_RNG_SEED=$i cargo test --test fuzz_transfer_path_validation
done
```

## Performance Analysis

### Profiling Fuzz Tests

```bash
# Time a fuzz test run
time cargo test --test fuzz_transfer_path_validation

# Profile with flamegraph (requires cargo-flamegraph)
cargo flamegraph --test fuzz_transfer_path_validation --bin cargo-test
```

### Expected Corpus Growth

Each fuzz test generates:
- **Initial:** ~1-5 KB (test metadata)
- **With failures:** +1-5 KB per unique regression case
- **Current state:** ~10-50 KB total for all fuzz tests

## Maintenance

### Monthly Tasks

- Review new regression cases
- Update test strategies if contracts change
- Update performance baselines
- Check for unused fuzz targets

### Quarterly Tasks

- Extend corpus size for regression detection
- Analyze and fix slow tests
- Update this documentation

## References

- [proptest documentation](https://docs.rs/proptest/latest/proptest/)
- [Soroban SDK testing](https://github.com/stellar/rs-soroban-sdk/tree/main/soroban-sdk-tests)
- [TESTING_AND_QA.md](../docs/TESTING_AND_QA.md) - General testing guide
- [docs/FUZZ_TESTING.md](../docs/FUZZ_TESTING.md) - User-facing fuzzing guide
