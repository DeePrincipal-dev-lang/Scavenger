#!/usr/bin/env node

/**
 * Analyze Benchmark Results Against Baselines
 *
 * This script compares test results against established baselines and:
 * 1. Detects regressions exceeding tolerance
 * 2. Reports performance metrics
 * 3. Exits non-zero on threshold breach (for CI/CD gates)
 *
 * Usage:
 *   node performance/analyze-results.js <results_file> [OPTIONS]
 *
 * Options:
 *   --baselines FILE  Path to baselines.json (default: performance/baselines.json)
 *   --strict          Fail on any regression (no tolerance)
 *   --report FILE     Save detailed report to file
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const BASELINES_FILE = path.join(__dirname, 'baselines.json');

// Parse arguments
const args = process.argv.slice(2);
if (args.length === 0) {
  console.error('Usage: node analyze-results.js <results_file> [--baselines FILE] [--strict] [--report FILE]');
  process.exit(1);
}

const resultsFile = args[0];
let baselinesFile = BASELINES_FILE;
let strictMode = false;
let reportFile = null;

for (let i = 1; i < args.length; i++) {
  if (args[i] === '--baselines' && i + 1 < args.length) {
    baselinesFile = args[i + 1];
    i++;
  } else if (args[i] === '--strict') {
    strictMode = true;
  } else if (args[i] === '--report' && i + 1 < args.length) {
    reportFile = args[i + 1];
    i++;
  }
}

console.log('╔═══════════════════════════════════════════════════════════╗');
console.log('║ Benchmark Results Analysis                                ║');
console.log('╚═══════════════════════════════════════════════════════════╝\n');

// Load files
let results;
let baselines;

try {
  console.log(`Loading results from: ${resultsFile}`);
  const resultsContent = fs.readFileSync(resultsFile, 'utf-8');
  results = JSON.parse(resultsContent);
} catch (error) {
  console.error(`✗ Error: Could not read results file: ${resultsFile}`);
  console.error(error.message);
  process.exit(1);
}

try {
  console.log(`Loading baselines from: ${baselinesFile}`);
  const baselinesContent = fs.readFileSync(baselinesFile, 'utf-8');
  baselines = JSON.parse(baselinesContent);
} catch (error) {
  console.error(`✗ Error: Could not read baselines file: ${baselinesFile}`);
  console.error(error.message);
  process.exit(1);
}

console.log(`Tolerance: ${baselines.tolerance_percent}%\n`);

// Analyze results
const analysis = {
  timestamp: new Date().toISOString(),
  totalTests: 0,
  passed: 0,
  improved: 0,
  regressed: 0,
  failed: 0,
  details: [],
};

console.log('Analysis Results:');
console.log('─'.repeat(110));
console.log(
  '%-30s %15s %15s %12s %12s %15s'.replace(/%[0-9]*s/g, (match) => {
    const width = parseInt(match) || 15;
    return '%' + width + 's';
  })
);
console.log(
  'Benchmark',
  'Baseline',
  'Measured',
  'Change',
  'Tolerance',
  'Result'
);
console.log('─'.repeat(110));

// Compare each benchmark
Object.entries(results.measurements || results.benchmarks || {}).forEach(([name, value]) => {
  const baseline = baselines.benchmarks[name];
  
  if (!baseline) {
    console.log(`⚠️  Unknown benchmark: ${name}`);
    return;
  }

  analysis.totalTests++;
  
  const tolerance = (baseline.tolerance_percent || baselines.tolerance_percent) / 100;
  const tolerance_pct = (baseline.tolerance_percent || baselines.tolerance_percent);
  const upperBound = baseline.baseline * (1 + tolerance);
  const lowerBound = baseline.baseline * (1 - tolerance);
  const change = value - baseline.baseline;
  const changePct = (change / baseline.baseline) * 100;
  
  let result = 'PASS';
  let status = '✓';
  
  if (value > upperBound) {
    result = 'REGRESSION';
    status = '✗';
    analysis.regressed++;
    analysis.failed++;
  } else if (value < lowerBound) {
    result = 'IMPROVEMENT';
    status = '↑';
    analysis.improved++;
    analysis.passed++;
  } else {
    analysis.passed++;
  }

  // In strict mode, any deviation is a failure
  if (strictMode && (value > baseline.baseline || value < baseline.baseline)) {
    result = 'STRICT_FAIL';
    status = '⚠️ ';
    if (value > baseline.baseline) {
      analysis.failed++;
    }
  }

  const changeStr = change >= 0 ? '+' + changePct.toFixed(2) : changePct.toFixed(2);
  
  console.log(
    '%-30s %15d %15d %12s %12s %15s'.replace(/%[0-9]*s/g, (match) => {
      const width = parseInt(match) || 15;
      return '%' + width + 's';
    }),
    name,
    baseline.baseline,
    Math.round(value),
    changeStr + '%',
    `±${tolerance_pct}%`,
    status + ' ' + result
  );

  analysis.details.push({
    name,
    baseline: baseline.baseline,
    measured: Math.round(value),
    change: Math.round(change),
    changePct: parseFloat(changePct.toFixed(2)),
    tolerance: tolerance_pct,
    result,
  });
});

console.log('─'.repeat(110));
console.log();

// Summary
console.log('Summary:');
console.log(`  Total Tests:    ${analysis.totalTests}`);
console.log(`  ✓ Passed:       ${analysis.passed}`);
console.log(`  ↑ Improved:     ${analysis.improved}`);
console.log(`  ✗ Regressed:    ${analysis.regressed}`);
console.log(`  ⚠️  Failed:       ${analysis.failed}`);
console.log();

// Generate report if requested
if (reportFile) {
  try {
    fs.writeFileSync(reportFile, JSON.stringify(analysis, null, 2) + '\n');
    console.log(`Report saved to: ${reportFile}\n`);
  } catch (error) {
    console.error(`Error: Could not save report to ${reportFile}`);
    console.error(error.message);
  }
}

// Exit codes
if (analysis.failed > 0) {
  console.log('❌ Benchmark regression gate FAILED\n');
  process.exit(1);
} else if (analysis.regressed > 0) {
  console.log('⚠️  Regressions detected but within tolerance\n');
  process.exit(0); // Don't fail, but log warning
} else if (analysis.improved > 0) {
  console.log('✅ All benchmarks improved! Performance is better than baseline\n');
  process.exit(0);
} else {
  console.log('✅ Benchmark regression gate PASSED\n');
  process.exit(0);
}
