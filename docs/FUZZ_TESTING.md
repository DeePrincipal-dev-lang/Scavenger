# Fuzz Testing Guide for Scavenger Contract

## Overview

Fuzz testing helps discover edge cases and vulnerabilities by generating random or semi-random inputs to exercise code paths. The Scavenger smart contract uses **proptest** for property-based testing and fuzzing.

## Current Fuzz Targets

### 1. Transfer Path Validation (`fuzz_transfer_path_validation.rs`)

**Purpose:** Validates the core waste transfer logic by fuzzing all possible role combinations and transfer paths.

**What It Tests:**
- Valid transfer paths: Recycler→Collector, Recycler→Manufacturer, Collector→Manufacturer
- Invalid transfer paths: All other role combinations
- Unregistered and deregistered participant handling
- Multi-hop transfer sequences
- Coordinate boundary conditions

**Key Properties:**
1. Valid routes return `true` from `is_valid_transfer()`
2. Invalid routes return `false` from `is_valid_transfer()`
3. Unregistered participants are rejected
4. Deregistered participants are rejected
5. Transfers on invalid paths fail without changing state
6. Multi-hop transfers maintain correct ownership state

### 2. Waste Transfer (`fuzz_waste_transfer.rs`)

**Purpose:** Fuzzes waste transfer operations with boundary conditions and coordinate variations.

**Coverage:**
- Valid waste IDs and boundary values
- Coordinate ranges (latitude: -90 to 90, longitude: -180 to 180)
- Variable-length transfer notes

### 3. Participant Registration (`fuzz_participant_registration.rs`)

**Purpose:** Tests participant registration with various role combinations and data.

### 4. State Transitions (`fuzz_state_transitions.rs`)

**Purpose:** Validates state machine correctness through fuzzing.

### 5. Waste Submission (`fuzz_waste_submission.rs`)

**Purpose:** Fuzzes waste submission operations and waste type handling.

### 6. Incentive Creation (`fuzz_incentive_creation.rs`)

**Purpose:** Tests incentive creation with various configurations.

## Running Fuzz Tests

### Run All Fuzz Tests

```bash
cd stellar-contract
cargo test --test 'fuzz_*' -- --nocapture
```

### Run Specific Fuzz Test

```bash
# Run transfer path validation fuzz test
cargo test --test fuzz_transfer_path_validation -- --nocapture
```

### Run with Custom Fuzz Configuration

```bash
# Set number of test cases (default: 256)
PROPTEST_CASES=1000 cargo test --test fuzz_transfer_path_validation

# Set maximum number of shrink iterations
PROPTEST_MAX_SHRINK_ITERS=5000 cargo test --test fuzz_transfer_path_validation

# Write regression test cases to file
cargo test --test fuzz_transfer_path_validation
```

### Interpret Fuzz Test Output

```
running 10 tests
test fuzz_transfer_path_valid_routes ... ok
test fuzz_transfer_path_invalid_routes ... ok
test fuzz_transfer_path_unregistered_participants ... ok
test fuzz_transfer_path_with_coordinates ... ok
test fuzz_waste_transfer_valid_path ... ok
test fuzz_transfer_path_deregistered_participants ... ok
test fuzz_transfer_path_multi_hop ... ok
test fuzz_waste_transfer_invalid_path ... ok

test result: ok. 5 passed; 0 failed; 0 ignored
```

## Test Corpus and Regression Cases

Proptest maintains a **regression corpus** in `stellar-contract/tests/` directory with `.proptest-regressions` files.

**Example:** `reward_fuzz_test.proptest-regressions`

These files store minimal failing inputs to ensure regressions are caught:

```
# Seeds for the `reward_fuzz_test` test
cc 3e3e3e3e3e3e00
fa 05000000000000
```

### Updating Regression Cases

If you intentionally change behavior:

1. Run tests and review failures
2. Verify the new behavior is correct
3. Run with `PROPTEST_REGRESSIONS=accept`:

```bash
PROPTEST_REGRESSIONS=accept cargo test --test fuzz_transfer_path_validation
```

## Corpus Structure

### Transfer Path Validation Corpus

Located implicitly via proptest regression files. The test uses strategies for:

- **Role Strategy:** Generates valid participant roles (0=Recycler, 1=Collector, 2=Manufacturer)
- **Coordinate Strategy:** Generates valid latitude (-90 to 90) and longitude (-180 to 180)
- **Registration Status:** Generates combinations of registered/unregistered and deregistered states

### Adding Custom Seeds

To seed the corpus with specific test cases, add them to regression files or modify test strategies:

```rust
// Example: Always test a specific scenario in addition to fuzzing
#[test]
fn test_transfer_path_specific_case() {
    let (env, client) = setup();
    let recycler = register(&client, &env, ParticipantRole::Recycler);
    let collector = register(&client, &env, ParticipantRole::Collector);
    assert!(client.is_valid_transfer(&recycler, &collector));
}
```

## Performance and Timing

### Expected Execution Times

- **Transfer Path Validation:** ~2-5 seconds (256 test cases)
- **All Fuzz Tests:** ~10-20 seconds
- **Extended Run (1000 cases):** ~10-30 seconds per test

### Performance Optimization

For faster feedback during development:

```bash
# Run fewer cases during development
PROPTEST_CASES=50 cargo test --test fuzz_transfer_path_validation

# Run full suite for CI/CD
cargo test --test 'fuzz_*'
```

## Triaging Fuzz Test Failures

### 1. Identify the Failure

```
thread 'fuzz_transfer_path_valid_routes' panicked at 'assertion failed: client.is_valid_transfer(&from, &to)'
```

### 2. Find the Seed Value

Look for the seed in error output:

```
Error: Seed for the failing input: 0xabcd1234
Shrunk example: from_role: 0, to_role: 1
```

### 3. Create Minimal Reproduction

Add a regression test with the minimal input:

```rust
#[test]
fn test_regression_case_xyz() {
    let (env, client) = setup();
    // Use the minimal input from the fuzz failure
    let recycler = register(&client, &env, ParticipantRole::Recycler);
    let collector = register(&client, &env, ParticipantRole::Collector);
    assert!(client.is_valid_transfer(&recycler, &collector));
}
```

### 4. Fix and Verify

1. Fix the underlying issue in the contract code
2. Run the regression test
3. Update proptest regressions once verified

## Integration with CI/CD

Add to GitHub Actions workflow (`.github/workflows/`):

```yaml
- name: Run Fuzz Tests
  run: |
    cd stellar-contract
    cargo test --test 'fuzz_*' -- --nocapture --test-threads=1
```

## Best Practices

1. **Keep Tests Hermetic:** Each test should be independent and not rely on global state
2. **Use Appropriate Strategies:** Choose strategies that explore the relevant input space
3. **Validate Properties:** Focus on invariants that should always hold true
4. **Review Regressions:** Check regression files to understand edge cases
5. **Performance:** Balance test coverage with execution time
6. **Documentation:** Document what each test validates and why

## Troubleshooting

### Issue: Tests Timeout

**Solution:** Reduce test case count

```bash
PROPTEST_CASES=50 cargo test --test fuzz_transfer_path_validation
```

### Issue: Flaky Tests

**Solution:** Investigate random failures by running with fixed seed

```bash
PROPTEST_RNG_SEED=0 cargo test --test fuzz_transfer_path_validation
```

### Issue: Contract Panics

**Solution:** Review error message and check regression files for minimal reproduction

## Related Documentation

- [TESTING_AND_QA.md](./TESTING_AND_QA.md) - General testing guide
- [stellar-contract/FUZZING.md](../stellar-contract/FUZZING.md) - Contract-specific fuzzing details
- [proptest Documentation](https://docs.rs/proptest/)

## Contributing Fuzz Tests

When adding new fuzz targets:

1. Place test file in `stellar-contract/tests/fuzz_*.rs`
2. Use proptest strategies for input generation
3. Add property comments explaining what's being tested
4. Document expected runtime
5. Update this guide with new targets
