-- Migration 003 (down): Reverse audit logging and query optimization
-- Drops everything added in 003_audit_query_optimization_up.sql

-- Remove covering / composite indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_raw_events_created_at_event_type;
DROP INDEX CONCURRENTLY IF EXISTS idx_token_rewards_recipient_amount;
DROP INDEX CONCURRENTLY IF EXISTS idx_participants_active;
DROP INDEX CONCURRENTLY IF EXISTS idx_waste_transfers_waste_id_time;
DROP INDEX CONCURRENTLY IF EXISTS idx_wastes_recycler_active_registered;
DROP INDEX CONCURRENTLY IF EXISTS idx_raw_events_ledger_id_event_type;
DROP INDEX CONCURRENTLY IF EXISTS idx_raw_events_ledger_type;
DROP INDEX CONCURRENTLY IF EXISTS idx_wastes_confirmed_active;
DROP INDEX CONCURRENTLY IF EXISTS idx_wastes_active_only;
DROP INDEX CONCURRENTLY IF EXISTS idx_rewards_recipient_time;
DROP INDEX CONCURRENTLY IF EXISTS idx_transfers_waste_time;
DROP INDEX CONCURRENTLY IF EXISTS idx_wastes_recycler_active_time;
DROP INDEX CONCURRENTLY IF EXISTS idx_wastes_type_registered;

-- Remove audit log indexes and table
DROP INDEX IF EXISTS idx_audit_actor_action;
DROP INDEX IF EXISTS idx_audit_timestamp;
DROP INDEX IF EXISTS idx_audit_action;
DROP INDEX IF EXISTS idx_audit_actor;
DROP TABLE IF EXISTS audit_logs;
