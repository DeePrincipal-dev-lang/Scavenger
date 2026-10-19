---
id: decision-making
title: Decision-Making Framework
sidebar_position: 2
---

# Decision-Making Framework

This document describes how decisions are made in the Scavngr project.

---

## Decision Types

### Type 1 — Routine (No vote required)

Day-to-day decisions made by any maintainer without requiring consensus:

- Merging a pull request with ≥ 1 maintainer approval and passing CI.
- Closing stale issues (no activity > 90 days with a 7-day warning comment).
- Updating dependencies (non-breaking, patch or minor semver).
- Fixing typos and formatting in documentation.

### Type 2 — Standard (Lazy consensus)

Decisions that affect a wider scope but are not breaking changes. Any maintainer may proceed unless another maintainer raises an objection within **7 days**:

- Adding a new feature to the contract or frontend.
- Changing CI/CD configuration.
- Adding or removing a dependency.
- Creating a new branch protection rule.
- Merging a pull request that touches core contract logic.

**Lazy consensus process**: Post the proposal as a GitHub issue or PR comment with the `governance` label. If no maintainer objects within 7 days, the proposer may proceed.

### Type 3 — Significant (Simple majority vote)

Decisions with meaningful impact on contributors, users, or project direction:

- New maintainer nominations.
- Deprecating a feature or API endpoint.
- Changing the project's license.
- Adding a new supported language (i18n).
- Major architectural changes.

**Voting**: Maintainers vote via a `+1` / `-1` comment on the governance issue. A proposal passes with a simple majority of active maintainers. Voting is open for **14 days** or until all active maintainers have voted, whichever is first.

### Type 4 — Constitutional (Two-thirds supermajority)

Changes to foundational governance documents:

- Amending this charter.
- Changes to the Code of Conduct.
- Transferring project ownership.
- Archiving or discontinuing the project.

**Voting**: 14-day public comment period followed by a vote of active maintainers. Requires ≥ two-thirds of active maintainers voting in favour.

---

## Proposal Process

1. **Open an issue** with the `governance` label.
2. **Describe the proposal**: context, motivation, proposed change, and alternatives considered.
3. **Community discussion**: anyone may comment during the comment period.
4. **Maintainer decision**: follow the appropriate decision type above.
5. **Record the outcome**: close the issue with a comment documenting the result and reasoning.

---

## Conflict of Interest

A maintainer must recuse themselves from voting on a proposal if they have a direct personal or financial interest in the outcome. Recusal is declared by commenting "Recused — conflict of interest" on the issue.

---

## Appeal Process

Any contributor who believes a decision was made unfairly may appeal by:

1. Opening a new issue with the `governance` label and title prefix `[Appeal]`.
2. Describing why the decision was unfair and what outcome they seek.
3. Maintainers who were not involved in the original decision will review and respond within 14 days.
