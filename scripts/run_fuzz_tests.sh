#!/bin/bash

# Fuzz Testing Script for Scavenger Smart Contract
# This script runs all fuzz targets with configurable parameters
# Usage: ./run_fuzz_tests.sh [OPTIONS]
# Options:
#   -c, --cases NUM        Number of test cases per fuzz test (default: 256)
#   -t, --test NAME        Run specific fuzz test (e.g., fuzz_transfer_path_validation)
#   -e, --extended         Run extended suite (1000 cases, verbose output)
#   -q, --quick            Quick check (50 cases, faster feedback)
#   -s, --seed SEED        Use fixed seed for reproducibility
#   -h, --help             Show this help message

set -e

# Default configuration
PROPTEST_CASES=256
VERBOSE=""
SPECIFIC_TEST=""
SEED=""
COLOR="\033[0;36m"
RESET="\033[0m"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -c|--cases)
            PROPTEST_CASES="$2"
            shift 2
            ;;
        -t|--test)
            SPECIFIC_TEST="$2"
            shift 2
            ;;
        -e|--extended)
            PROPTEST_CASES=1000
            VERBOSE="--nocapture"
            shift
            ;;
        -q|--quick)
            PROPTEST_CASES=50
            shift
            ;;
        -s|--seed)
            SEED="$2"
            shift 2
            ;;
        -h|--help)
            echo "Fuzz Testing Script for Scavenger Contract"
            echo ""
            echo "Usage: ./run_fuzz_tests.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -c, --cases NUM        Number of test cases per fuzz test (default: 256)"
            echo "  -t, --test NAME        Run specific fuzz test (e.g., fuzz_transfer_path_validation)"
            echo "  -e, --extended         Run extended suite (1000 cases, verbose output)"
            echo "  -q, --quick            Quick check (50 cases, faster feedback)"
            echo "  -s, --seed SEED        Use fixed seed for reproducibility"
            echo "  -h, --help             Show this help message"
            echo ""
            echo "Examples:"
            echo "  ./run_fuzz_tests.sh                    # Run all fuzz tests (256 cases each)"
            echo "  ./run_fuzz_tests.sh --quick            # Quick check (50 cases each)"
            echo "  ./run_fuzz_tests.sh --extended         # Extended run (1000 cases each)"
            echo "  ./run_fuzz_tests.sh -t fuzz_transfer_path_validation   # Specific test"
            echo "  ./run_fuzz_tests.sh --cases 500 --seed 0xcafe         # Custom config"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Navigate to stellar-contract directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONTRACT_DIR="$(dirname "$SCRIPT_DIR")/stellar-contract"

if [ ! -d "$CONTRACT_DIR" ]; then
    echo "Error: stellar-contract directory not found at $CONTRACT_DIR"
    exit 1
fi

cd "$CONTRACT_DIR"

echo -e "${COLOR}═══════════════════════════════════════════════════════════${RESET}"
echo -e "${COLOR}Scavenger Smart Contract - Fuzz Testing Suite${RESET}"
echo -e "${COLOR}═══════════════════════════════════════════════════════════${RESET}"
echo ""

# Show configuration
echo "Configuration:"
echo "  Test Cases per Test: $PROPTEST_CASES"
if [ -n "$SEED" ]; then
    echo "  Random Seed: $SEED"
fi
if [ -n "$SPECIFIC_TEST" ]; then
    echo "  Specific Test: $SPECIFIC_TEST"
fi
echo ""

# Set up environment
export PROPTEST_CASES
if [ -n "$SEED" ]; then
    export PROPTEST_RNG_SEED="$SEED"
fi

# Determine which tests to run
if [ -n "$SPECIFIC_TEST" ]; then
    TESTS="$SPECIFIC_TEST"
    TEST_PATTERN="--test $SPECIFIC_TEST"
else
    TEST_PATTERN="--test 'fuzz_*'"
fi

# Run tests with timing
START_TIME=$(date +%s)

echo "Running fuzz tests..."
echo ""

if [ -n "$VERBOSE" ]; then
    cargo test $TEST_PATTERN -- $VERBOSE --test-threads=1
else
    cargo test $TEST_PATTERN
fi

EXIT_CODE=$?

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
echo -e "${COLOR}═══════════════════════════════════════════════════════════${RESET}"

if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${COLOR}✓ All fuzz tests passed!${RESET}"
    echo "Duration: ${DURATION}s"
else
    echo -e "${COLOR}✗ Fuzz tests failed (exit code: $EXIT_CODE)${RESET}"
fi

echo -e "${COLOR}═══════════════════════════════════════════════════════════${RESET}"
echo ""

# Print regression test file locations
echo "Regression test cases (minimal failing inputs):"
if ls tests/*.proptest-regressions 1> /dev/null 2>&1; then
    ls -lh tests/*.proptest-regressions
    echo ""
    echo "Review regressions:"
    echo "  cat tests/*.proptest-regressions"
else
    echo "  No regression cases recorded yet"
fi

echo ""
echo "For more information, see stellar-contract/FUZZING.md"
echo ""

exit $EXIT_CODE
