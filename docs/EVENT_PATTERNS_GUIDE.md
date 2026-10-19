# Event Patterns Guide

_Issue #814 — Scavngr Soroban Contract_

## Overview

All contract events are emitted via functions in `src/events.rs`.  The new
`src/event_builder.rs` module adds:

| Utility | Purpose |
|---------|---------|
| `EventCategory` | Logical grouping for events |
| `EventBuilder` | Fluent builder to publish events with ≤ 4 topics |
| `EventFormatter` | Static labels for known event symbols |
| `EventFilter` | Predicate-based filter for off-chain consumers |

---

## Event anatomy

A Soroban event has two parts:

```
topics  — tuple of Symbols / scalar values  (max 4 elements)
data    — arbitrary Val-encodable value
```

The indexer (`indexer/src/handlers/eventHandlers.ts`) listens for these and
stores them in the database.

---

## Existing event reference

| Function | Topic 0 | Topic 1 | Data |
|----------|---------|---------|------|
| `emit_waste_registered` | `"recycled"` | `waste_id` | `(waste_type, weight, recycler, lat, lon)` |
| `emit_waste_transferred` | `"transfer"` | `waste_id` | `(from, to)` |
| `emit_waste_confirmed` | `"confirmed"` | `waste_id` | `confirmer` |
| `emit_participant_registered` | `"reg"` | `address` | `(role, name, lat, lon)` |
| `emit_tokens_rewarded` | `"rewarded"` | `recipient` | `(amount, waste_id)` |
| `emit_donation_made` | `"donated"` | `donor` | `(amount, charity)` |
| `emit_waste_deactivated` | `"deactive"` | `waste_id` | `(admin, timestamp)` |
| `emit_waste_expired` | `"expired"` | `waste_id` | `timestamp` |
| `emit_admin_transferred` | `"adm_xfr"` | _(none)_ | `previous_admin` |
| `emit_contract_paused` | `"paused"` | _(none)_ | `admin` |
| `emit_contract_unpaused` | `"unpaused"` | _(none)_ | `admin` |

> Symbols are limited to **9 ASCII characters** by `symbol_short!`.

---

## Adding a new event

### 1. Define a `Symbol` constant (events.rs)

```rust
const MY_EVENT: Symbol = symbol_short!("my_evt");
```

### 2. Write an emit function

```rust
pub fn emit_my_event(env: &Env, id: u64, actor: &Address) {
    env.events().publish((MY_EVENT, id), actor);
}
```

### 3. Call it from the contract function

```rust
events::emit_my_event(&env, thing_id, &caller);
```

### 4. Optional — use EventBuilder for complex data

```rust
use crate::event_builder::EventBuilder;

// 2-topic event
EventBuilder::new(&env).publish2(MY_EVENT, thing_id, (actor, extra_field));

// 1-topic event
EventBuilder::new(&env).publish1(MY_EVENT, actor);
```

---

## Event filtering (off-chain)

Use `EventFilter` to decide which events the indexer should process:

```rust
use crate::event_builder::{EventFilter, EventCategory};

// Only waste events
let f = EventFilter::category(EventCategory::Waste);

// Only security-relevant events
let f = EventFilter::security();

// Everything
let f = EventFilter::all();

// Use it
if f.matches(&event_symbol) {
    // process event
}
```

---

## EventCategory prefix convention

New events **may** carry the category prefix as their first topic element for
easy indexer-side filtering:

```
topics: ("waste", "recycled", waste_id)
         ↑ category   ↑ action   ↑ id
```

Existing events do **not** use this convention to preserve backward
compatibility.  The `EventBuilder::with_category` constructor is available for
new events that want to opt into category-prefixed topics.

---

## Security-relevant events

The following events indicate high-impact state changes and should always be
monitored / alerted on:

- `adm_xfr` — admin transfer
- `paused` — contract paused
- `upg_exec` — upgrade executed
- `perm_gr` — permission granted
- `perm_rv` — permission revoked

Use `EventFormatter::is_security_event(sym)` or `EventFilter::security()` to
detect them.

---

## Testing events

The Soroban test SDK records all emitted events.  Retrieve them with:

```rust
let events = env.events().all();
// events is a Vec<(Address, Vec<Val>, Val)> — (contract, topics, data)
```

See `stellar-contract/tests/waste_registered_event_test.rs` for examples.
