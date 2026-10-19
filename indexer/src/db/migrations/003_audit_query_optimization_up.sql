-- Migration 003 (up): Audit logging and query optimization
-- Merges 003_audit_and_query_optimization.sql + 003_query_indexes.sql (issue #915)

-- Audit log table for on-chain sensitive operations
CREATE TABLE IF NOT EXISTS audit_logs (
  id BIGSERIAL PRIMARY KEY,
  action TEXT NOT NULL,
  actor_address TEXT NOT NULL,
  target TEXT NOT NULL,
  details TEXT,
  ledger_sequence BIGINT,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_actor ON audit_logs(actor_address);
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_actor_action ON audit_logs(actor_address, action);

-- Composite indexes for common query patterns
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_wastes_type_registered
  ON wastes(waste_type, registered_at DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_wastes_recycler_active_time
  ON wastes(recycler_address, is_active, registered_at DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_transfers_waste_time
  ON waste_transfers(waste_id, transferred_at DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_rewards_recipient_time
  ON token_rewards(recipient_address, rewarded_at DESC);

-- Partial indexes for filtered queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_wastes_active_only
  ON wastes(registered_at DESC) WHERE is_active = true;

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_wastes_confirmed_active
  ON wastes(recycler_address, registered_at DESC) WHERE is_confirmed = true AND is_active = true;

-- Composite index for raw_events range + type queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_raw_events_ledger_type
  ON raw_events(ledger_sequence DESC, event_type);

-- Covering index for paginated event listing (avoids heap fetch)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_raw_events_ledger_id_event_type
  ON raw_events (ledger_sequence DESC, id DESC, event_type, contract_id, created_at);

-- Per-recycler active-waste queries (N+1 fix)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_wastes_recycler_active_registered
  ON wastes (recycler_address, is_active, registered_at DESC);

-- Transfer history by time (supply-chain view)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_waste_transfers_waste_id_time
  ON waste_transfers (waste_id, transferred_at ASC);

-- Partial: only active participants
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_participants_active
  ON participants (address) WHERE is_active = true;

-- Token reward aggregation by recipient
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_token_rewards_recipient_amount
  ON token_rewards (recipient_address, amount);

-- Date-range event queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_raw_events_created_at_event_type
  ON raw_events (created_at, event_type);
