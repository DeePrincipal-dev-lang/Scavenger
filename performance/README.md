# Performance Testing Configuration

## Consolidated k6 Load Testing Suite

The performance directory contains a consolidated load testing framework with 5 optimized profiles.

### Quick Start

```bash
# Run default smoke test
./scripts/run-perf-tests.sh

# Run specific profile
./scripts/run-perf-tests.sh --profile load

# Run all profiles
./scripts/run-perf-tests.sh --all
```

### Setup

```bash
# Install k6
brew install k6  # macOS
sudo apt-get install k6  # Linux
choco install k6  # Windows

# Make scripts executable
chmod +x scripts/run-perf-tests.sh
chmod +x scripts/run_fuzz_tests.sh
```

### Load Testing Profiles

| Profile | Duration | Users | Purpose | File |
|---------|----------|-------|---------|------|
| **Smoke** | 50 sec | 1 | Quick sanity check | `k6-smoke.js` |
| **Load** | 6 min | 100 | Baseline performance | `k6-load.js` |
| **Stress** | 15 min | 1k-10k | Breaking point | `k6-stress.js` |
| **Spike** | 6 min | 100→1k | Traffic burst | `k6-spike.js` |
| **Soak** | 40 min | 500 | Degradation detection | `k6-soak.js` |

### Running Tests

#### Command Line

```bash
# Smoke test (default)
k6 run performance/k6-smoke.js

# Load test
BASE_URL=http://localhost:8000/api k6 run performance/k6-load.js

# Stress test
BASE_URL=http://localhost:8000/api k6 run performance/k6-stress.js

# With output
k6 run --out json=results.json performance/k6-load.js
```

#### Master Script

```bash
# Run specific profile
./scripts/run-perf-tests.sh --profile load

# Run with custom configuration
./scripts/run-perf-tests.sh --profile load \
  --base-url https://api.example.com \
  --output ./reports \
  --json

# Run all profiles sequentially
./scripts/run-perf-tests.sh --all
```

### Performance Targets

| Profile | P95 Response | P99 Response | Error Rate | Status |
|---------|-------------|-------------|-----------|--------|
| Smoke   | < 500 ms    | < 1000 ms   | < 1%      | ✓      |
| Load    | < 1000 ms   | < 2000 ms   | < 5%      | ✓      |
| Stress  | < 2000 ms   | < 5000 ms   | < 10%     | ✓      |
| Spike   | < 2000 ms   | < 5000 ms   | < 15%     | ✓      |
| Soak    | < 500 ms    | < 1000 ms   | < 5%      | ✓      |

### Metrics Tracked

Shared custom metrics via `k6-common.js`:

- `error_rate` - Overall error rate
- `http_errors` - HTTP error count
- `api_duration_ms` - API response time
- `contract_call_duration_ms` - Smart contract execution time
- `participant_op_duration_ms` - Participant operations
- `waste_op_duration_ms` - Waste operations
- `query_op_duration_ms` - Query operations
- `success_count` - Successful operations
- `failure_count` - Failed operations
- `active_connections` - Currently active connections

### File Organization

```
performance/
├── k6-common.js              # Shared utilities and configuration
├── k6-smoke.js               # Smoke test profile
├── k6-load.js                # Load test profile
├── k6-stress.js              # Stress test profile
├── k6-spike.js               # Spike test profile
├── k6-soak.js                # Soak test profile
├── LOAD_TESTING_GUIDE.md     # Comprehensive guide
├── performance-benchmarks.md # Baseline metrics
└── README.md                 # This file
```

### Deprecated Scripts

The following overlapping scripts are deprecated:
- ~~`k6-load-test-comprehensive.js`~~
- ~~`k6-load-test.js`~~
- ~~`k6-concurrent-users.js`~~
- ~~`k6-api-response-times.js`~~
- ~~`k6-database-queries.js`~~
- ~~`k6-waste-submission.js`~~
- ~~`k6-contract-gas-usage.js`~~
- ~~`load-test-scenarios.js`~~

Use the consolidated scripts instead.

### CI/CD Integration

Performance tests run on every push:

```yaml
# Example GitHub Actions workflow
- name: Run Performance Tests
  run: |
    ./scripts/run-perf-tests.sh --profile smoke
    ./scripts/run-perf-tests.sh --profile load
```

### Environment Variables

```bash
# API Configuration
BASE_URL=http://localhost:8000/api
CONTRACT_ID=CAAAA...

# Test Options
TIMEOUT=30000  # milliseconds
```

### Example Commands

```bash
# Local development - quick check
./scripts/run-perf-tests.sh

# CI/CD - baseline performance
BASE_URL=http://staging.example.com ./scripts/run-perf-tests.sh --profile load

# Release validation - full suite
./scripts/run-perf-tests.sh --all --output ./reports --json

# Overnight soak test
BASE_URL=http://production.example.com \
  ./scripts/run-perf-tests.sh --profile soak --output ./overnight-results
```

### Performance Budgets

System must maintain:
- API Response Time (p95): < 500ms under normal load
- Contract Call Duration (p95): < 1000ms
- Error Rate: < 5% under normal load
- Throughput: > 100 requests/second

### Thresholds Configuration

Each profile defines thresholds in `k6-common.js`:

```javascript
export const thresholds = {
  smoke: {
    'http_req_duration': ['p(95)<500', 'p(99)<1000'],
    'http_req_failed': ['rate<0.01'],
  },
  load: {
    'http_req_duration': ['p(95)<1000', 'p(99)<2000'],
    'http_req_failed': ['rate<0.05'],
  },
  // ... more profiles
};
```

Thresholds are exit criteria: test fails if any threshold is exceeded.
