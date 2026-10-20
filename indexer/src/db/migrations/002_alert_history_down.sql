-- Migration 002 (down): Drop alert history table
DROP INDEX IF EXISTS idx_alert_history_created;
DROP INDEX IF EXISTS idx_alert_history_severity;
DROP INDEX IF EXISTS idx_alert_history_name;
DROP TABLE IF EXISTS alert_history;
