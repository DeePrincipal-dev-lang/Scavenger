-- =============================================================================
-- Test data seeding script for the Scavngr indexer database.
--
-- Issue #956 – Add test data seeding scripts.
--
-- Usage (psql):
--   psql "$DATABASE_URL" -f scripts/seed-test-data.sql
--
-- Or via the TypeScript helper:
--   npx ts-node scripts/seed-test-data.ts
--
-- This script creates reproducible datasets suitable for:
--   * Integration tests
--   * Local development
--   * CI test environments
--
-- Safe to re-run: uses INSERT … ON CONFLICT DO NOTHING / DO UPDATE so that
-- re-seeding is idempotent.
-- =============================================================================

-- Ensure the schema exists (mirrors indexer/src/db/migrate.ts)
CREATE TABLE IF NOT EXISTS sync_status (
  id         SERIAL PRIMARY KEY,
  last_ledger BIGINT NOT NULL DEFAULT 0,
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS participants (
  address              VARCHAR(64)  PRIMARY KEY,
  role                 VARCHAR(20)  NOT NULL,
  name                 VARCHAR(255) NOT NULL,
  latitude             DECIMAL(10, 6),
  longitude            DECIMAL(10, 6),
  registered_at_ledger BIGINT       NOT NULL DEFAULT 0,
  registered_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
  is_active            BOOLEAN      NOT NULL DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS raw_events (
  id               SERIAL PRIMARY KEY,
  ledger_sequence  BIGINT       NOT NULL,
  transaction_hash VARCHAR(128) NOT NULL,
  contract_id      VARCHAR(64)  NOT NULL,
  event_type       VARCHAR(128) NOT NULL,
  topic            JSONB,
  value            JSONB,
  created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS token_rewards (
  id                SERIAL PRIMARY KEY,
  recipient_address VARCHAR(64)  NOT NULL,
  amount            BIGINT       NOT NULL DEFAULT 0,
  waste_id          BIGINT,
  rewarded_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- ---------------------------------------------------------------------------
-- sync_status – start indexing from ledger 0
-- ---------------------------------------------------------------------------
INSERT INTO sync_status (id, last_ledger, updated_at)
  VALUES (1, 0, NOW())
  ON CONFLICT (id) DO UPDATE SET last_ledger = EXCLUDED.last_ledger;

-- ---------------------------------------------------------------------------
-- participants – 10 seed accounts covering all three roles
-- ---------------------------------------------------------------------------
INSERT INTO participants
  (address, role, name, latitude, longitude, registered_at_ledger, registered_at, is_active)
VALUES
  -- Recyclers (4)
  ('GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN',
   'Recycler', 'Alice Kamau',      -1.2921,  36.8219, 1000, '2024-01-15 10:30:00+00', TRUE),
  ('GBKGJTSMPLC54YDKYZPAWKQ4HFSJCLB6PWDX36AFZDLXO3YLQAZFXBO',
   'Recycler', 'Bob Omondi',       -1.3031,  36.7073, 1050, '2024-01-16 08:00:00+00', TRUE),
  ('GCUGB2S2WETD2VBUAV2CKUGKYY63JTFNOXIN2XRXHEQNLSOMKGXNB3T',
   'Recycler', 'Carol Wanjiru',    -1.2707,  36.8126, 1100, '2024-01-17 09:15:00+00', TRUE),
  ('GDC3W2X5KUTZPURLOQNHLDWYFNZV26Y7IZIUAASYGXQQAIVVPZP6GK4',
   'Recycler', 'David Mwangi',     -1.2880,  36.8240, 1150, '2024-01-18 11:00:00+00', FALSE),

  -- Collectors (3)
  ('GCITNMB4RRXQHBOPVV42LH2T5NHPD6L5PO23XCJZSKJJBR5Z3GIHKZF',
   'Collector', 'Eve Collector',   -1.3040,  36.7200, 1200, '2024-01-20 14:00:00+00', TRUE),
  ('GDYMQMZJUVXMHF4MJKLZPCRJPPOPVKGKQDBVPQ6HDWLZAXOFZ5TKM52',
   'Collector', 'Frank Njoroge',   -1.2600,  36.7900, 1250, '2024-01-22 12:30:00+00', TRUE),
  ('GBQPMBZLGKL2GV47SFKLFYIMHBBYUVDHDSJG46SRO3RMYPKYXRGJPB3',
   'Collector', 'Grace Achieng',   -1.2990,  36.8300, 1300, '2024-01-25 09:00:00+00', TRUE),

  -- Manufacturers (3)
  ('GCMF7MH5YYVKYUIBBIGZR3OEBBSLSTMKPDBV5AQMG63O6I5QRS3C3QQ',
   'Manufacturer', 'Acme Recycling Ltd',  -1.2850,  36.8200, 1400, '2024-02-01 08:00:00+00', TRUE),
  ('GBKLFASDL2NLKJDSF3AAAGDVNMKJZXBCSDGABD3NAKJDSFH4NLKJASDF',
   'Manufacturer', 'GreenPak Industries', -1.2700,  36.8100, 1450, '2024-02-05 10:00:00+00', TRUE),
  ('GBNAIKJDSLAKJSDFLKJASDFKLJ3NKJ2BSDFASDFKLJ3NKJSDFLAKJDSF',
   'Manufacturer', 'EcoMelt Corp',        -1.2650,  36.8050, 1500, '2024-02-10 13:00:00+00', FALSE)
ON CONFLICT (address) DO UPDATE SET
  role                 = EXCLUDED.role,
  name                 = EXCLUDED.name,
  latitude             = EXCLUDED.latitude,
  longitude            = EXCLUDED.longitude,
  registered_at_ledger = EXCLUDED.registered_at_ledger,
  registered_at        = EXCLUDED.registered_at,
  is_active            = EXCLUDED.is_active;

-- ---------------------------------------------------------------------------
-- raw_events – 20 seed events covering key event types
-- ---------------------------------------------------------------------------
INSERT INTO raw_events
  (ledger_sequence, transaction_hash, contract_id, event_type, topic, value, created_at)
VALUES
  -- Participant registration events
  (1000, 'tx_hash_001_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'ParticipantRegistered',
   '["ParticipantRegistered"]'::jsonb,
   '{"address":"GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN","role":0}'::jsonb,
   '2024-01-15 10:30:00+00'),

  (1050, 'tx_hash_002_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'ParticipantRegistered',
   '["ParticipantRegistered"]'::jsonb,
   '{"address":"GBKGJTSMPLC54YDKYZPAWKQ4HFSJCLB6PWDX36AFZDLXO3YLQAZFXBO","role":0}'::jsonb,
   '2024-01-16 08:00:00+00'),

  (1200, 'tx_hash_003_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'ParticipantRegistered',
   '["ParticipantRegistered"]'::jsonb,
   '{"address":"GCITNMB4RRXQHBOPVV42LH2T5NHPD6L5PO23XCJZSKJJBR5Z3GIHKZF","role":1}'::jsonb,
   '2024-01-20 14:00:00+00'),

  (1400, 'tx_hash_004_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'ParticipantRegistered',
   '["ParticipantRegistered"]'::jsonb,
   '{"address":"GCMF7MH5YYVKYUIBBIGZR3OEBBSLSTMKPDBV5AQMG63O6I5QRS3C3QQ","role":2}'::jsonb,
   '2024-02-01 08:00:00+00'),

  -- Waste submission events
  (2000, 'tx_hash_005_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'WasteRegistered',
   '["WasteRegistered"]'::jsonb,
   '{"waste_id":1,"waste_type":2,"weight":5000,"submitter":"GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN"}'::jsonb,
   '2024-02-15 10:00:00+00'),

  (2010, 'tx_hash_006_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'WasteRegistered',
   '["WasteRegistered"]'::jsonb,
   '{"waste_id":2,"waste_type":0,"weight":3000,"submitter":"GBKGJTSMPLC54YDKYZPAWKQ4HFSJCLB6PWDX36AFZDLXO3YLQAZFXBO"}'::jsonb,
   '2024-02-15 10:30:00+00'),

  (2020, 'tx_hash_007_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'WasteRegistered',
   '["WasteRegistered"]'::jsonb,
   '{"waste_id":3,"waste_type":3,"weight":8000,"submitter":"GCUGB2S2WETD2VBUAV2CKUGKYY63JTFNOXIN2XRXHEQNLSOMKGXNB3T"}'::jsonb,
   '2024-02-16 09:00:00+00'),

  (2030, 'tx_hash_008_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'WasteRegistered',
   '["WasteRegistered"]'::jsonb,
   '{"waste_id":4,"waste_type":5,"weight":12000,"submitter":"GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN"}'::jsonb,
   '2024-02-16 11:00:00+00'),

  -- Waste transfer events
  (3000, 'tx_hash_009_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'WasteTransferred',
   '["WasteTransferred"]'::jsonb,
   '{"waste_id":1,"from":"GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN","to":"GCITNMB4RRXQHBOPVV42LH2T5NHPD6L5PO23XCJZSKJJBR5Z3GIHKZF"}'::jsonb,
   '2024-02-20 10:00:00+00'),

  (3010, 'tx_hash_010_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'WasteTransferred',
   '["WasteTransferred"]'::jsonb,
   '{"waste_id":1,"from":"GCITNMB4RRXQHBOPVV42LH2T5NHPD6L5PO23XCJZSKJJBR5Z3GIHKZF","to":"GCMF7MH5YYVKYUIBBIGZR3OEBBSLSTMKPDBV5AQMG63O6I5QRS3C3QQ"}'::jsonb,
   '2024-02-20 14:00:00+00'),

  (3020, 'tx_hash_011_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'WasteTransferred',
   '["WasteTransferred"]'::jsonb,
   '{"waste_id":2,"from":"GBKGJTSMPLC54YDKYZPAWKQ4HFSJCLB6PWDX36AFZDLXO3YLQAZFXBO","to":"GDYMQMZJUVXMHF4MJKLZPCRJPPOPVKGKQDBVPQ6HDWLZAXOFZ5TKM52"}'::jsonb,
   '2024-02-21 09:30:00+00'),

  -- Waste confirmation events
  (3500, 'tx_hash_012_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'WasteConfirmed',
   '["WasteConfirmed"]'::jsonb,
   '{"waste_id":1,"confirmer":"GCMF7MH5YYVKYUIBBIGZR3OEBBSLSTMKPDBV5AQMG63O6I5QRS3C3QQ"}'::jsonb,
   '2024-02-22 10:00:00+00'),

  (3510, 'tx_hash_013_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'WasteConfirmed',
   '["WasteConfirmed"]'::jsonb,
   '{"waste_id":2,"confirmer":"GDYMQMZJUVXMHF4MJKLZPCRJPPOPVKGKQDBVPQ6HDWLZAXOFZ5TKM52"}'::jsonb,
   '2024-02-22 11:00:00+00'),

  -- Incentive events
  (4000, 'tx_hash_014_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'IncentiveCreated',
   '["IncentiveCreated"]'::jsonb,
   '{"incentive_id":1,"rewarder":"GCMF7MH5YYVKYUIBBIGZR3OEBBSLSTMKPDBV5AQMG63O6I5QRS3C3QQ","waste_type":2,"reward_points":100,"budget":10000}'::jsonb,
   '2024-03-01 08:00:00+00'),

  (4010, 'tx_hash_015_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'IncentiveCreated',
   '["IncentiveCreated"]'::jsonb,
   '{"incentive_id":2,"rewarder":"GCMF7MH5YYVKYUIBBIGZR3OEBBSLSTMKPDBV5AQMG63O6I5QRS3C3QQ","waste_type":0,"reward_points":50,"budget":5000}'::jsonb,
   '2024-03-01 09:00:00+00'),

  -- Reward distribution events
  (5000, 'tx_hash_016_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'RewardDistributed',
   '["RewardDistributed"]'::jsonb,
   '{"waste_id":1,"recipient":"GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN","amount":500}'::jsonb,
   '2024-03-10 10:00:00+00'),

  (5010, 'tx_hash_017_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'RewardDistributed',
   '["RewardDistributed"]'::jsonb,
   '{"waste_id":1,"recipient":"GCITNMB4RRXQHBOPVV42LH2T5NHPD6L5PO23XCJZSKJJBR5Z3GIHKZF","amount":300}'::jsonb,
   '2024-03-10 10:01:00+00'),

  -- Participant deregistration event
  (6000, 'tx_hash_018_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'ParticipantDeregistered',
   '["ParticipantDeregistered"]'::jsonb,
   '{"address":"GDC3W2X5KUTZPURLOQNHLDWYFNZV26Y7IZIUAASYGXQQAIVVPZP6GK4"}'::jsonb,
   '2024-03-15 12:00:00+00'),

  -- Role update event
  (6010, 'tx_hash_019_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'RoleUpdated',
   '["RoleUpdated"]'::jsonb,
   '{"address":"GCUGB2S2WETD2VBUAV2CKUGKYY63JTFNOXIN2XRXHEQNLSOMKGXNB3T","old_role":0,"new_role":1}'::jsonb,
   '2024-03-20 09:00:00+00'),

  -- Admin transfer event
  (7000, 'tx_hash_020_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
   'CONTRACT_SEED_ID_001',
   'AdminTransferred',
   '["AdminTransferred"]'::jsonb,
   '{"old_admin":"GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN","new_admin":"GCMF7MH5YYVKYUIBBIGZR3OEBBSLSTMKPDBV5AQMG63O6I5QRS3C3QQ"}'::jsonb,
   '2024-04-01 10:00:00+00')
ON CONFLICT DO NOTHING;

-- ---------------------------------------------------------------------------
-- token_rewards – 5 reward records
-- ---------------------------------------------------------------------------
INSERT INTO token_rewards
  (recipient_address, amount, waste_id, rewarded_at)
VALUES
  ('GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN', 500,  1, '2024-03-10 10:00:00+00'),
  ('GCITNMB4RRXQHBOPVV42LH2T5NHPD6L5PO23XCJZSKJJBR5Z3GIHKZF', 300,  1, '2024-03-10 10:01:00+00'),
  ('GBKGJTSMPLC54YDKYZPAWKQ4HFSJCLB6PWDX36AFZDLXO3YLQAZFXBO', 250,  2, '2024-03-11 09:00:00+00'),
  ('GCUGB2S2WETD2VBUAV2CKUGKYY63JTFNOXIN2XRXHEQNLSOMKGXNB3T', 400,  3, '2024-03-12 11:00:00+00'),
  ('GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN', 600,  4, '2024-03-13 14:00:00+00')
ON CONFLICT DO NOTHING;
