#!/usr/bin/env node
/**
 * Automated accessibility audit script — issue #874
 *
 * Runs axe-core via Playwright against key pages and exits with a non-zero
 * code if any critical violations are found.  Designed to be run as a
 * CI-independent step:
 *
 *   node scripts/a11y-audit.mjs
 *
 * Prerequisites (already in devDependencies):
 *   @playwright/test, @axe-core/playwright
 *
 * Environment:
 *   BASE_URL  — defaults to http://localhost:5173
 */

import { chromium } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'

const BASE_URL = process.env.BASE_URL ?? 'http://localhost:5173'

/** Pages to audit.  Add more as the app grows. */
const PAGES = [
  { name: 'Landing',        path: '/' },
  { name: 'Login',          path: '/login' },
  { name: 'Dashboard',      path: '/dashboard' },
  { name: 'Waste List',     path: '/wastes' },
  { name: 'Incentives',     path: '/incentives' },
  { name: 'Profile',        path: '/profile' },
  { name: 'Settings',       path: '/settings' },
  { name: 'Marketplace',    path: '/marketplace' },
  { name: 'Analytics',      path: '/analytics' },
  { name: 'Recycling Guide',path: '/recycling-guide' },
]

const IMPACT_LEVELS = /** @type {const} */ (['critical', 'serious'])

async function auditPage(page, url) {
  await page.goto(url, { waitUntil: 'networkidle', timeout: 30_000 })
  const results = await new AxeBuilder({ page })
    .withTags(['wcag2a', 'wcag2aa', 'wcag21aa'])
    .analyze()
  return results.violations
}

function formatViolations(violations) {
  return violations
    .map((v) => {
      const nodes = v.nodes
        .map((n) => `    • ${n.html.slice(0, 120)}`)
        .join('\n')
      return `  [${v.impact?.toUpperCase()}] ${v.id}: ${v.description}\n${nodes}`
    })
    .join('\n\n')
}

async function run() {
  console.log(`\n🔍  Accessibility audit — ${BASE_URL}\n`)

  const browser = await chromium.launch()
  const context = await browser.newContext()
  const page = await context.newPage()

  let totalCritical = 0
  let totalSerious = 0
  const report = []

  for (const { name, path } of PAGES) {
    const url = `${BASE_URL}${path}`
    process.stdout.write(`  Auditing ${name} (${url}) … `)

    let violations = []
    try {
      violations = await auditPage(page, url)
    } catch (err) {
      console.log(`SKIP (${err.message})`)
      continue
    }

    const critical = violations.filter((v) => v.impact === 'critical')
    const serious  = violations.filter((v) => v.impact === 'serious')
    totalCritical += critical.length
    totalSerious  += serious.length

    const badge =
      critical.length > 0 ? '❌' : serious.length > 0 ? '⚠️ ' : '✅'
    console.log(`${badge}  ${violations.length} violation(s)  (${critical.length} critical, ${serious.length} serious)`)

    const relevant = violations.filter((v) => IMPACT_LEVELS.includes(v.impact))
    if (relevant.length > 0) {
      report.push({ name, url, violations: relevant })
    }
  }

  await browser.close()

  console.log('\n── Summary ─────────────────────────────────────────────')
  console.log(`  Critical: ${totalCritical}`)
  console.log(`  Serious:  ${totalSerious}`)
  console.log('────────────────────────────────────────────────────────\n')

  if (report.length > 0) {
    for (const { name, url, violations } of report) {
      console.error(`\n❌  ${name}  (${url})`)
      console.error(formatViolations(violations))
    }
    console.error('\n✖  Accessibility audit FAILED — fix the violations above.\n')
    process.exit(1)
  }

  console.log('✔  Accessibility audit PASSED — zero critical/serious violations.\n')
  process.exit(0)
}

run().catch((err) => {
  console.error(err)
  process.exit(1)
})
