#!/bin/bash
set -e

CASES=${PROPTEST_CASES:-1000}
echo "Running fuzzing suite with PROPTEST_CASES=$CASES"
echo "================================================"

echo ""
echo "[1/3] Running comprehensive fuzzing suite..."
PROPTEST_CASES=$CASES cargo test --package stellar-scavngr-contract --test fuzz_comprehensive -- --nocapture 2>&1 | tee /tmp/fuzz_results.log

echo ""
echo "[2/3] Running regression tests..."
cargo test --package stellar-scavngr-contract --test fuzz_regression -- --nocapture 2>&1 | tee -a /tmp/fuzz_results.log

echo ""
echo "[3/3] Running existing fuzz tests..."
PROPTEST_CASES=$CASES cargo test --package stellar-scavngr-contract fuzz_ -- --nocapture 2>&1 | tee -a /tmp/fuzz_results.log

echo ""
echo "================================================"
echo "Fuzzing complete. Full results in /tmp/fuzz_results.log"
