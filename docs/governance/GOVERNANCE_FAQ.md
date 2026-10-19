---
id: faq
title: Governance FAQ
sidebar_position: 5
---

# Governance FAQ

Common questions about how the Scavngr project is run.

---

**Q: Who controls the project?**

Scavngr is governed by its maintainers — a group of contributors with write access to the repository who share responsibility for code review, releases, and governance. There is no single controlling entity. The current maintainers are listed in [`.github/CODEOWNERS`](https://github.com/Xoulomon/Scavenger/blob/main/.github/CODEOWNERS).

---

**Q: How do I become a maintainer?**

By making consistent, high-quality contributions over at least three months. Any existing maintainer can nominate a contributor for maintainership. Nominations are approved via the Type 3 (simple majority) decision process. See the [Charter](charter.md) for details.

---

**Q: How are decisions made?**

Using a four-tier framework based on decision impact — from routine (any maintainer can decide) to constitutional (two-thirds supermajority). See the [Decision-Making Framework](decision-making.md) for full details.

---

**Q: I disagree with a decision. Can I challenge it?**

Yes. Open an issue with the `governance` label and the prefix `[Appeal]` in the title. Uninvolved maintainers will review and respond within 14 days. See the [Decision-Making Framework](decision-making.md) for the appeals process.

---

**Q: How do I report a Code of Conduct violation?**

Send a private message to any maintainer (contact information is in `SECURITY.md`). You will receive a response within 72 hours. Reports are handled confidentially.

---

**Q: Can the project be forked?**

Yes. Scavngr is MIT-licensed. You may fork, modify, and redistribute the codebase under the terms of the [MIT License](https://github.com/Xoulomon/Scavenger/blob/main/LICENSE).

---

**Q: How does the project handle security vulnerabilities?**

Please do **not** open a public GitHub issue for security vulnerabilities. Follow the responsible disclosure process described in [`SECURITY.md`](https://github.com/Xoulomon/Scavenger/blob/main/SECURITY.md). Maintainers will acknowledge receipt within 48 hours and aim to release a fix within 30 days for critical issues.

---

**Q: Is this project affiliated with the Stellar Development Foundation?**

Scavngr is built on Stellar and participates in the Stellar Wave program, but it is an independent open-source project and is not affiliated with, endorsed by, or sponsored by the Stellar Development Foundation.

---

**Q: How transparent is the project?**

Very. All governance discussions happen on GitHub (public issues with the `governance` label). Quarterly transparency reports are published in `docs/governance/transparency-reports/` covering project metrics, issue statistics, and aggregate moderation statistics (no personal details).

---

**Q: Where can I follow project announcements?**

Watch the [GitHub repository](https://github.com/Xoulomon/Scavenger) for releases and governance decisions. Subscribe to notifications on issues tagged `governance` or `release`.

---

**Q: How do I propose a new feature?**

Open a GitHub issue with the `enhancement` label. Describe the feature, its motivation, and any relevant technical details. The community and maintainers will discuss it. Once there's consensus, a contributor can implement it and open a pull request.

---

**Q: How are releases scheduled?**

Scavngr follows a loose "release when ready" cadence rather than a fixed calendar schedule. Significant features and security fixes are released as they're ready. Patch releases for bug fixes are made promptly. All releases are tagged on GitHub and announced in the repository's Releases page.
