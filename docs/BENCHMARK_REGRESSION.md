# Benchmark Regression Gate Documentation

## Overview

The Scavenger smart contract uses an enforced benchmark regression gate to prevent unintended performance degradation. This gate:

1. Establishes baseline gas costs for critical operations
2. Tracks performance across contract versions
3. Fails CI/CD if measured costs exceed acceptable thresholds
4. Forces investigation before accepting regressions

## Architecture

### Baseline Configuration

Baselines are stored in `performance/baselines.json` with structure:

```json
{
  "version": "1.0.0",
  "tolerance_percent": 5,
  "benchmarks": {
    "participant_registration": {
      "baseline": 45000,
      "tolerance_percent": 5,
      "description": "Gas cost for participant registration",
      "operation": "register_participant()"
    }
  }
}
```

### Measured Operations

Current benchmarked operations:

| Operation | Baseline | Tolerance | Measurement |
|-----------|----------|-----------|-------------|
| `participant_registration` | 45,000 | ±5% | CPU instructions |
| `waste_submission` | 62,000 | ±5% | CPU instructions |
| `waste_transfer` | 55,000 | ±5% | CPU instructions |
| `waste_confirmation` | 48,000 | ±5% | CPU instructions |
| `incentive_creation` | 58,000 | ±5% | CPU instructions |
| `incentive_claim` | 51,000 | ±5% | CPU instructions |
| `query_global_metrics` | 28,000 | ±5% | CPU instructions |
| `query_participant_stats` | 32,000 | ±5% | CPU instructions |
| `batch_waste_transfer` | 120,000 | ±8% | CPU instructions |
| `material_submission` | 48,000 | ±5% | CPU instructions |

## Running Benchmarks

### Generate New Baselines

Use when establishing baselines for new operations or environments:

```bash
# Run benchmarks and report results (no changes)
node performance/generate-baseline.js

# Update baselines with new measurements
node performance/generate-baseline.js --update --reason "Optimized waste transfer algorithm"

# Save to alternate file
node performance/generate-baseline.js --update --output baseline-v2.json
```

### Analyze Results

Check measured results against baselines:

```bash
# Analyze results from benchmark run
node performance/analyze-results.js performance_results.json

# Use custom baselines
node performance/analyze-results.js results.json --baselines custom-baselines.json

# Strict mode: fail on any deviation from baseline
node performance/analyze-results.js results.json --strict

# Save detailed report
node performance/analyze-results.js results.json --report report.json
```

### Integration Test

Run as part of CI/CD pipeline:

```bash
# In GitHub Actions or similar
cd stellar-contract
cargo test --test performance_test -- --nocapture
node ../performance/analyze-results.js benchmark_results.json || exit 1
```

## Handling Regressions

### When a Regression is Detected

1. **Local Development**
   ```bash
   node performance/generate-baseline.js
   # Output shows regression details
   ```

2. **Review the Cause**
   - Examine contract code changes
   - Profile the operation using Soroban SDK tools
   - Identify algorithmic or storage changes
   - Calculate expected cost increase

3. **Decision: Accept or Reject**

   **Reject (revert change):**
   ```bash
   git checkout stellar-contract/src/lib.rs
   ```

   **Accept (if justified):**
   ```bash
   # Update baseline with documented reason
   node performance/generate-baseline.js \
     --update \
     --reason "Added validation for X, trade-off worth Y% cost increase"
   
   git add performance/baselines.json
   git commit -m "perf: Update benchmark baseline - validation improvement"
   ```

### Injecting a Regression (For Testing)

To verify the gate works:

```rust
// In stellar-contract/src/lib.rs
pub fn expensive_operation() {
  // Add intentional inefficiency
  for _ in 0..1000 {
    // Waste CPU cycles
  }
}
```

Then run:
```bash
node performance/generate-baseline.js
# Should show regression exceeding tolerance
```

## Baseline Update Procedure

### Step 1: Measure

```bash
cd stellar-contract
cargo test --test performance_test -- --nocapture
```

### Step 2: Validate

```bash
node ../performance/generate-baseline.js
# Review the comparison report
```

### Step 3: Document

```bash
node ../performance/generate-baseline.js \
  --update \
  --reason "Clear, specific reason for change"
```

### Step 4: Review

Git shows the changes to `performance/baselines.json`:

```diff
- "baseline": 45000,
+ "baseline": 47500,
```

Include in commit message:
- What changed (new feature, optimization, etc.)
- Why it impacts performance
- Whether the change is acceptable
- Links to issues/PRs

### Step 5: Commit

```bash
git add performance/baselines.json
git commit -m "perf: Update benchmark baseline

- Updated participant_registration baseline from 45K to 47K instructions
- Reason: Added extra validation for coordinate bounds checking (Issue #XXX)
- Impact: 5.5% increase, justified by security improvement
- Verified: All other operations unchanged"
```

## Tolerance Levels

Default tolerance is 5% for most operations, with exceptions:

| Operation | Tolerance | Rationale |
|-----------|-----------|-----------|
| Batch operations | 8% | Higher variability due to batch sizes |
| Query operations | 5% | Stable, predictable cost |
| State-modifying ops | 5% | Consistent behavior |

### Why Tolerances Exist

1. **Soroban Runtime Variations**: Different optimization levels
2. **Storage State**: VM state affects instruction counts
3. **Environmental Factors**: Ledger snapshots, cache effects

Tolerances are conservative to catch meaningful regressions while allowing for expected variation.

## Performance Interpretation

### Gas and Costs

Measurements are in **CPU instruction counts** (not stellar lumens):

- 1 instruction ≈ small CPU operation
- 100K instructions ≈ ~1ms on modern CPUs
- Costs convert to XLM using Stellar chain parameters

### Optimization Opportunities

Regressions indicate:

1. **Added Logic**: New validation, error handling
2. **Inefficient Algorithm**: O(n) loops, repeated work
3. **Storage Overhead**: More data persisted per operation
4. **Dependencies**: External calls or complex types

### Investigation Checklist

When regression detected:

- [ ] Is this change intentional?
- [ ] Does it provide value (security, functionality)?
- [ ] Can it be optimized further?
- [ ] What's the actual cost impact to users?
- [ ] Are there alternatives with better cost/benefit?

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Benchmark Gate

on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Run Benchmarks
        run: |
          cd stellar-contract
          cargo test --test performance_test -- --nocapture
          
      - name: Check Regression Gate
        run: |
          node performance/analyze-results.js \
            stellar-contract/performance_results.json \
            --report benchmark-report.json || exit 1
            
      - name: Comment PR with Results
        if: always()
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = JSON.parse(fs.readFileSync('benchmark-report.json'));
            const comment = `## Benchmark Results
            - Passed: ${report.passed}
            - Improved: ${report.improved}
            - Regressed: ${report.regressed}
            `;
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: comment
            });
```

## Troubleshooting

### Issue: High Variance in Measurements

**Cause**: Soroban runtime variations, background processes

**Solution**: 
- Run multiple times: `cargo test --test performance_test -- --nocapture --test-threads=1`
- Use isolated environment
- Compare recent runs for trend

### Issue: Baseline Update Stale

**Cause**: Contract changed but baseline not updated

**Solution**:
- Always regenerate after contract changes
- Use `--update` flag to accept new baseline
- Document reason for update

### Issue: Tolerance Too Strict

**Cause**: Minor environmental variations trigger regression

**Solution**:
- Review actual change (use `--strict` mode)
- Consider adjusting tolerance if justified
- Document rationale in baselines.json

## Best Practices

1. **Update Early**: Regenerate baselines immediately after contract changes
2. **Document Always**: Include reason in commit messages
3. **Review Changes**: Understand why performance changed
4. **Monitor Trends**: Track baseline growth over releases
5. **Investigate Improvements**: Don't assume improvements are free
6. **Communicate**: PR descriptions should mention performance impact

## Related Files

- `stellar-contract/tests/performance_test.rs` - Benchmark test implementations
- `performance/baselines.json` - Baseline measurements
- `performance/generate-baseline.js` - Baseline generation utility
- `performance/analyze-results.js` - Regression detection utility
- `.github/workflows/` - CI/CD integration

## References

- [Soroban Documentation](https://developers.stellar.org/soroban)
- [Smart Contract Performance Best Practices](https://developers.stellar.org/soroban/learn/storing-data)
- [Stellar Asset Limits and Costs](https://developers.stellar.org/learn/building-blocks/stellar-asset)
