# Scavenger Smart Contract — Internal Notes

This page is for contributors working on `stellar-contract/`. It is not the
consumer reference. External SDK users should read
[sdk-api-reference.md](./sdk-api-reference.md), which lists every public
entrypoint with its arguments, auth requirements and error codes.

- **Source of truth:** `stellar-contract/src/lib.rs` (entrypoints) and
  `stellar-contract/src/errors.rs` (error codes).
- **Module layout:** [stellar-contract/ARCHITECTURE.md](../stellar-contract/ARCHITECTURE.md)
  and [ADR-0013](./adr/0013-contract-module-layout.md).
- **Previous full text:** the long-form function reference was retired in favour of
  the generated tables. It is kept at
  [archive/CONTRACT_DOCUMENTATION-pre-1445.md](./archive/CONTRACT_DOCUMENTATION-pre-1445.md).

## Internal-only entrypoints

<!-- BEGIN GENERATED: internal-entrypoints -->

Generated from `stellar-contract/src/lib.rs`. 1 entrypoints are documented as internal-only and are excluded from the consumer reference in [sdk-api-reference.md](./sdk-api-reference.md). Do not edit this table by hand.

| Entrypoint | Arguments | Returns | Requires auth | Errors | Note |
|---|---|---|---|---|---|
| `record_performance_metric` | `admin: Address`, `gas_used: u64`, `is_success: bool` | `()` | `admin` | — | Record a performance metric (internal helper or admin-only). |

<!-- END GENERATED: internal-entrypoints -->

## Regenerating the reference

```bash
python3 scripts/generate-contract-reference.py           # rewrite both docs
python3 scripts/generate-contract-reference.py --check   # fail if they are stale (CI)
```
