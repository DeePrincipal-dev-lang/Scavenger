-- Migration 001 (down): Drop initial schema
DROP TABLE IF EXISTS carbon_credits;
DROP TABLE IF EXISTS auctions;
DROP TABLE IF EXISTS token_rewards;
DROP TABLE IF EXISTS waste_transfers;
DROP TABLE IF EXISTS wastes;
DROP TABLE IF EXISTS participants;
DROP TABLE IF EXISTS raw_events;
DROP TABLE IF EXISTS sync_status;
