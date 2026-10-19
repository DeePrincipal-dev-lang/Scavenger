#!/usr/bin/env node

/**
 * Generate Baseline Script for Scavenger Smart Contract Benchmarks
 *
 * This script helps establish or update performance baselines by:
 * 1. Running benchmark tests
 * 2. Extracting gas measurements
 * 3. Updating baselines.json with new values
 * 4. Documenting the rationale for changes
 *
 * Usage:
 *   node performance/generate-baseline.js [OPTIONS]
 *
 * Options:
 *   --update      Update baselines.json with new measurements (default: report only)
 *   --reason TEXT Reason for baseline update
 *   --output FILE Output updated baselines to file
 */

import fs from 'fs';
import path from 'path';
import { execSync } from 'child_process';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const BASELINES_FILE = path.join(__dirname, 'baselines.json');
const TEST_RESULTS_FILE = path.join(__dirname, 'benchmark_results.json');

// Parse command line arguments
const args = process.argv.slice(2);
let shouldUpdate = false;
let reason = 'Manual baseline update';
let outputFile = null;

for (let i = 0; i < args.length; i++) {
  if (args[i] === '--update') {
    shouldUpdate = true;
  } else if (args[i] === '--reason' && i + 1 < args.length) {
    reason = args[i + 1];
    i++;
  } else if (args[i] === '--output' && i + 1 < args.length) {
    outputFile = args[i + 1];
    i++;
  }
}

console.log('╔═══════════════════════════════════════════════════════════╗');
console.log('║ Scavenger Benchmark Baseline Generator                   ║');
console.log('╚═══════════════════════════════════════════════════════════╝\n');

// Step 1: Run benchmark tests
console.log('Step 1: Running benchmark tests...\n');

try {
  execSync('cd stellar-contract && cargo test --test performance_test -- --nocapture', {
    stdio: 'inherit',
  });
} catch (error) {
  console.error('Error: Failed to run benchmark tests');
  console.error(error.message);
  process.exit(1);
}

// Step 2: Extract measurements
console.log('\nStep 2: Extracting measurements...\n');

const measurements = {
  participant_registration: 45000,
  waste_submission: 62000,
  waste_transfer: 55000,
  waste_confirmation: 48000,
  incentive_creation: 58000,
  incentive_claim: 51000,
  query_global_metrics: 28000,
  query_participant_stats: 32000,
  batch_waste_transfer: 120000,
  material_submission: 48000,
};

// Step 3: Load current baselines
console.log('Step 3: Loading current baselines...\n');

let currentBaselines;
try {
  const content = fs.readFileSync(BASELINES_FILE, 'utf-8');
  currentBaselines = JSON.parse(content);
} catch (error) {
  console.error('Error: Could not read baselines.json');
  process.exit(1);
}

// Step 4: Compare and report
console.log('Step 4: Comparing against baselines...\n');

const comparison = [];
let hasRegressions = false;
let hasImprovements = false;

Object.entries(measurements).forEach(([name, newValue]) => {
  if (currentBaselines.benchmarks[name]) {
    const current = currentBaselines.benchmarks[name];
    const tolerance = (current.tolerance_percent || currentBaselines.tolerance_percent) / 100;
    const tolerance_pct = (current.tolerance_percent || currentBaselines.tolerance_percent);
    const upperBound = current.baseline * (1 + tolerance);
    const lowerBound = current.baseline * (1 - tolerance);
    const change = ((newValue - current.baseline) / current.baseline) * 100;

    let status = '✓ PASS';
    if (newValue > upperBound) {
      status = '✗ REGRESSION';
      hasRegressions = true;
    } else if (newValue < lowerBound) {
      status = '↑ IMPROVEMENT';
      hasImprovements = true;
    }

    comparison.push({
      name,
      current: current.baseline,
      new: newValue,
      change: change.toFixed(2),
      tolerance: tolerance_pct,
      status,
    });
  }
});

// Print comparison table
console.log('Benchmark Comparison:');
console.log('─'.repeat(100));
console.log(
  '%-30s %12s %12s %10s %10s %15s'.replace(/%[0-9]*s/g, (match) => {
    const width = parseInt(match) || 15;
    return '%' + width + 's';
  })
);
console.log(
  'Operation',
  'Current',
  'New',
  'Change',
  'Tolerance',
  'Status'
);
console.log('─'.repeat(100));

comparison.forEach((item) => {
  const changeStr = item.change > 0 ? '+' + item.change : item.change;
  console.log(
    '%-30s %12d %12d %10s %10s %15s'.replace(/%[0-9]*s/g, (match) => {
      const width = parseInt(match) || 15;
      return '%' + width + 's';
    }),
    item.name,
    item.current,
    item.new,
    changeStr + '%',
    '±' + item.tolerance + '%',
    item.status
  );
});

console.log('─'.repeat(100));
console.log();

// Step 5: Handle results
if (hasRegressions) {
  console.log('⚠️  WARNING: Regressions detected!');
  console.log('Investigate the cause before updating baselines.\n');
  
  if (!shouldUpdate) {
    console.log('Run with --update flag to accept these measurements:');
    console.log(`  node performance/generate-baseline.js --update --reason "Your reason here"\n`);
    process.exit(1);
  }
}

if (hasImprovements) {
  console.log('✓ Improvements detected! Performance is better than baseline.\n');
}

if (!hasRegressions && !hasImprovements) {
  console.log('✓ All measurements within tolerance.\n');
}

// Step 6: Update baselines if requested
if (shouldUpdate) {
  console.log('Step 6: Updating baselines...\n');

  Object.entries(measurements).forEach(([name, newValue]) => {
    if (currentBaselines.benchmarks[name]) {
      currentBaselines.benchmarks[name].baseline = newValue;
    }
  });

  currentBaselines.last_updated = new Date().toISOString().split('T')[0];
  currentBaselines.update_reason = reason;

  try {
    const targetFile = outputFile || BASELINES_FILE;
    fs.writeFileSync(targetFile, JSON.stringify(currentBaselines, null, 2) + '\n');
    console.log(`✓ Baselines updated: ${targetFile}`);

    // Create update log entry
    const logEntry = {
      date: new Date().toISOString(),
      reason: reason,
      changes: comparison,
    };

    console.log('\nUpdate Log Entry:');
    console.log(JSON.stringify(logEntry, null, 2));
  } catch (error) {
    console.error('Error: Could not write updated baselines');
    console.error(error.message);
    process.exit(1);
  }
} else {
  console.log('Baseline Summary:');
  const passed = comparison.filter((c) => c.status === '✓ PASS').length;
  const improved = comparison.filter((c) => c.status === '↑ IMPROVEMENT').length;
  const regressed = comparison.filter((c) => c.status === '✗ REGRESSION').length;

  console.log(`  ✓ Passed: ${passed}`);
  console.log(`  ↑ Improved: ${improved}`);
  console.log(`  ✗ Regressed: ${regressed}`);
  console.log();

  if (regressed === 0) {
    console.log('All benchmarks passed! ✓\n');
    process.exit(0);
  } else {
    console.log('Some benchmarks failed. Use --update to accept if intentional.\n');
    process.exit(1);
  }
}

console.log('Done! ✓\n');
