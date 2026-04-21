-- ============================================================
-- La Ruche — Dev Seed Data
-- ⚠️  Development / testing only — remove before v1.0
-- ============================================================
-- Creates a demo condominium with realistic data:
--   • 1 building  : Résidence Les Cerisiers (Lyon)
--   • 10 units    : 8 apartments, 1 cellar, 1 parking space
--   • 3 alloc keys: Charges Générales, Ascenseur, Eau Froide
--   • 10 users    : 1 manager · 2 board members · 5 co-owners · 2 tenants
--
-- All accounts share the same password: Demo1234!
-- (Argon2id hash, m=4096 t=1 p=1 — dev-only lightweight params)
--
-- Usage:
--   psql $DATABASE_URL -f backend/seeds/seed_dev.sql
--   make seed-dev
-- ============================================================

-- ============================================================
-- RESET: truncate all tables in dependency order
-- ============================================================
TRUNCATE TABLE
    maintenance_log_document, maintenance_log,
    report_document, report,
    vendor_submission_document, vendor_submission,
    task,
    contract_document, contract_reminder, vendor_contract,
    claim_document, claim,
    vote, resolution, general_meeting,
    levy_call, budget_line, budget, works_fund,
    consultation,
    unit_allocation_key,
    allocation_key,
    unit_owner,
    user_building,
    user_token,
    user_account,
    unit,
    document,
    building
CASCADE;

DO $$ BEGIN RAISE NOTICE '[seed] All tables truncated — inserting fresh dev data'; END $$;

-- ============================================================
-- SHARED PASSWORD HASH  (password: Demo1234!)
-- Argon2id  m=4096, t=1, p=1  — verified by Argon2::default()
-- ============================================================
-- All INSERTs below use this literal for password_hash.

-- ============================================================
-- 1. BUILDING
-- ============================================================
INSERT INTO building (id, name, address_line1, postal_code, city, shares_total, bylaws_date)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'Résidence Les Cerisiers',
    '14 allée des Cerisiers',
    '69003',
    'Lyon',
    8000,
    '2005-03-15'
) ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 2. ALLOCATION KEYS
-- ============================================================

-- 2a. Charges Générales  (type = general — unique per building)
INSERT INTO allocation_key (id, building_id, name, description, key_type)
VALUES (
    '00000000-0000-0000-0000-000000000010',
    '00000000-0000-0000-0000-000000000001',
    'Charges Générales',
    'Clé générale applicable à l''ensemble des charges communes',
    'general'
) ON CONFLICT (building_id, name) DO NOTHING;

-- 2b. Ascenseur
INSERT INTO allocation_key (id, building_id, name, description, key_type)
VALUES (
    '00000000-0000-0000-0000-000000000011',
    '00000000-0000-0000-0000-000000000001',
    'Ascenseur',
    'Charges liées à l''entretien et à l''utilisation de l''ascenseur',
    'special'
) ON CONFLICT (building_id, name) DO NOTHING;

-- 2c. Eau Froide
INSERT INTO allocation_key (id, building_id, name, description, key_type)
VALUES (
    '00000000-0000-0000-0000-000000000012',
    '00000000-0000-0000-0000-000000000001',
    'Eau Froide',
    'Clé de répartition pour la consommation d''eau froide collective',
    'special'
) ON CONFLICT (building_id, name) DO NOTHING;

-- ============================================================
-- 3. UNITS  (10 lots)
--    Lots 1–8  : apartments
--    Lot  9    : cellar
--    Lot  10   : parking space
-- ============================================================
INSERT INTO unit (id, building_id, unit_number, unit_type, block, floor, description)
VALUES
    ('00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000001', '1',  'apartment', 'A', 'RDC', 'Appartement 3 pièces — rez-de-chaussée'),
    ('00000000-0000-0000-0000-000000000102', '00000000-0000-0000-0000-000000000001', '2',  'apartment', 'A', '1',   'Appartement 2 pièces — 1er étage'),
    ('00000000-0000-0000-0000-000000000103', '00000000-0000-0000-0000-000000000001', '3',  'apartment', 'A', '1',   'Appartement 2 pièces — 1er étage'),
    ('00000000-0000-0000-0000-000000000104', '00000000-0000-0000-0000-000000000001', '4',  'apartment', 'A', '2',   'Appartement 3 pièces — 2e étage'),
    ('00000000-0000-0000-0000-000000000105', '00000000-0000-0000-0000-000000000001', '5',  'apartment', 'A', '2',   'Appartement 2 pièces — 2e étage'),
    ('00000000-0000-0000-0000-000000000106', '00000000-0000-0000-0000-000000000001', '6',  'apartment', 'A', '3',   'Appartement 4 pièces — 3e étage'),
    ('00000000-0000-0000-0000-000000000107', '00000000-0000-0000-0000-000000000001', '7',  'apartment', 'A', '3',   'Appartement 3 pièces — 3e étage'),
    ('00000000-0000-0000-0000-000000000108', '00000000-0000-0000-0000-000000000001', '8',  'apartment', 'A', '4',   'Appartement 4 pièces duplex — 4e étage'),
    ('00000000-0000-0000-0000-000000000109', '00000000-0000-0000-0000-000000000001', '9',  'cellar',    'S', NULL,  'Cave n°1 — sous-sol'),
    ('00000000-0000-0000-0000-000000000110', '00000000-0000-0000-0000-000000000001', '10', 'parking',   'P', NULL,  'Place de parking n°1')
ON CONFLICT (building_id, unit_number) DO NOTHING;

-- ============================================================
-- 4. UNIT ALLOCATION KEY SHARES  (active records, valid_to = NULL)
-- ============================================================

-- 4a. Charges Générales  (total = 8 000 tantièmes)
INSERT INTO unit_allocation_key (unit_id, key_id, share_value, valid_from)
VALUES
    ('00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000010',  850, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000102', '00000000-0000-0000-0000-000000000010',  900, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000103', '00000000-0000-0000-0000-000000000010',  850, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000104', '00000000-0000-0000-0000-000000000010',  950, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000105', '00000000-0000-0000-0000-000000000010',  900, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000106', '00000000-0000-0000-0000-000000000010',  980, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000107', '00000000-0000-0000-0000-000000000010',  960, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000108', '00000000-0000-0000-0000-000000000010', 1010, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000109', '00000000-0000-0000-0000-000000000010',  160, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000110', '00000000-0000-0000-0000-000000000010',  440, CURRENT_DATE)
ON CONFLICT DO NOTHING;

-- 4b. Ascenseur  (lots 2–8 only — RDC and parking/cave excluded, total = 1 110)
INSERT INTO unit_allocation_key (unit_id, key_id, share_value, valid_from)
VALUES
    ('00000000-0000-0000-0000-000000000102', '00000000-0000-0000-0000-000000000011', 150, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000103', '00000000-0000-0000-0000-000000000011', 140, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000104', '00000000-0000-0000-0000-000000000011', 160, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000105', '00000000-0000-0000-0000-000000000011', 150, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000106', '00000000-0000-0000-0000-000000000011', 170, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000107', '00000000-0000-0000-0000-000000000011', 165, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000108', '00000000-0000-0000-0000-000000000011', 175, CURRENT_DATE)
ON CONFLICT DO NOTHING;

-- 4c. Eau Froide  (lots 1–9, parking excluded, total = 750)
INSERT INTO unit_allocation_key (unit_id, key_id, share_value, valid_from)
VALUES
    ('00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000012',  80, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000102', '00000000-0000-0000-0000-000000000012',  90, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000103', '00000000-0000-0000-0000-000000000012',  85, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000104', '00000000-0000-0000-0000-000000000012',  95, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000105', '00000000-0000-0000-0000-000000000012',  90, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000106', '00000000-0000-0000-0000-000000000012',  95, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000107', '00000000-0000-0000-0000-000000000012',  95, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000108', '00000000-0000-0000-0000-000000000012', 100, CURRENT_DATE),
    ('00000000-0000-0000-0000-000000000109', '00000000-0000-0000-0000-000000000012',  20, CURRENT_DATE)
ON CONFLICT DO NOTHING;

-- ============================================================
-- 5. USER ACCOUNTS  (password: Demo1234!)
-- ============================================================
-- Hash: Argon2id  m=4096, t=1, p=1
-- Using the same hash for all accounts is intentional for dev seeds
-- (each account still validates independently via its embedded salt)

INSERT INTO user_account (id, email, last_name, first_name, password_hash)
VALUES
    -- Syndic (manager)
    ('00000000-0000-0000-0000-000000000201', 'alice.martin@laruche.test',       'Martin',    'Alice',     '$argon2id$v=19$m=4096,t=1,p=1$zxo1I5suprzObq/w2QsR2w$6X1sj0+dgFtUrecTy7CpgdH966Yc8TIp8MLvIJ8j16k'),
    -- Conseil syndical
    ('00000000-0000-0000-0000-000000000202', 'bernard.dupont@laruche.test',     'Dupont',    'Bernard',   '$argon2id$v=19$m=4096,t=1,p=1$zxo1I5suprzObq/w2QsR2w$6X1sj0+dgFtUrecTy7CpgdH966Yc8TIp8MLvIJ8j16k'),
    ('00000000-0000-0000-0000-000000000203', 'camille.rousseau@laruche.test',   'Rousseau',  'Camille',   '$argon2id$v=19$m=4096,t=1,p=1$zxo1I5suprzObq/w2QsR2w$6X1sj0+dgFtUrecTy7CpgdH966Yc8TIp8MLvIJ8j16k'),
    -- Copropriétaires résidents
    ('00000000-0000-0000-0000-000000000204', 'david.laurent@laruche.test',      'Laurent',   'David',     '$argon2id$v=19$m=4096,t=1,p=1$zxo1I5suprzObq/w2QsR2w$6X1sj0+dgFtUrecTy7CpgdH966Yc8TIp8MLvIJ8j16k'),
    ('00000000-0000-0000-0000-000000000205', 'emma.lefebvre@laruche.test',      'Lefebvre',  'Emma',      '$argon2id$v=19$m=4096,t=1,p=1$zxo1I5suprzObq/w2QsR2w$6X1sj0+dgFtUrecTy7CpgdH966Yc8TIp8MLvIJ8j16k'),
    ('00000000-0000-0000-0000-000000000206', 'francois.moreau@laruche.test',    'Moreau',    'François',  '$argon2id$v=19$m=4096,t=1,p=1$zxo1I5suprzObq/w2QsR2w$6X1sj0+dgFtUrecTy7CpgdH966Yc8TIp8MLvIJ8j16k'),
    ('00000000-0000-0000-0000-000000000207', 'gisele.petit@laruche.test',       'Petit',     'Gisèle',    '$argon2id$v=19$m=4096,t=1,p=1$zxo1I5suprzObq/w2QsR2w$6X1sj0+dgFtUrecTy7CpgdH966Yc8TIp8MLvIJ8j16k'),
    ('00000000-0000-0000-0000-000000000208', 'henri.chevalier@laruche.test',    'Chevalier', 'Henri',     '$argon2id$v=19$m=4096,t=1,p=1$zxo1I5suprzObq/w2QsR2w$6X1sj0+dgFtUrecTy7CpgdH966Yc8TIp8MLvIJ8j16k'),
    -- Locataires
    ('00000000-0000-0000-0000-000000000209', 'isabelle.fontaine@laruche.test',  'Fontaine',  'Isabelle',  '$argon2id$v=19$m=4096,t=1,p=1$zxo1I5suprzObq/w2QsR2w$6X1sj0+dgFtUrecTy7CpgdH966Yc8TIp8MLvIJ8j16k'),
    ('00000000-0000-0000-0000-000000000210', 'jules.martin@laruche.test',       'Martin',    'Jules',     '$argon2id$v=19$m=4096,t=1,p=1$zxo1I5suprzObq/w2QsR2w$6X1sj0+dgFtUrecTy7CpgdH966Yc8TIp8MLvIJ8j16k')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 6. USER ↔ BUILDING ROLES
-- ============================================================
-- Roles are cumulative (PG array). Board members also carry co_owner.

INSERT INTO user_building (id, user_id, building_id, roles)
VALUES
    ('00000000-0000-0000-0000-000000000301', '00000000-0000-0000-0000-000000000201', '00000000-0000-0000-0000-000000000001', '{manager}'),
    ('00000000-0000-0000-0000-000000000302', '00000000-0000-0000-0000-000000000202', '00000000-0000-0000-0000-000000000001', '{board_member,co_owner}'),
    ('00000000-0000-0000-0000-000000000303', '00000000-0000-0000-0000-000000000203', '00000000-0000-0000-0000-000000000001', '{board_member,co_owner}'),
    ('00000000-0000-0000-0000-000000000304', '00000000-0000-0000-0000-000000000204', '00000000-0000-0000-0000-000000000001', '{co_owner}'),
    ('00000000-0000-0000-0000-000000000305', '00000000-0000-0000-0000-000000000205', '00000000-0000-0000-0000-000000000001', '{co_owner}'),
    ('00000000-0000-0000-0000-000000000306', '00000000-0000-0000-0000-000000000206', '00000000-0000-0000-0000-000000000001', '{co_owner}'),
    ('00000000-0000-0000-0000-000000000307', '00000000-0000-0000-0000-000000000207', '00000000-0000-0000-0000-000000000001', '{co_owner}'),
    ('00000000-0000-0000-0000-000000000308', '00000000-0000-0000-0000-000000000208', '00000000-0000-0000-0000-000000000001', '{co_owner}'),
    ('00000000-0000-0000-0000-000000000309', '00000000-0000-0000-0000-000000000209', '00000000-0000-0000-0000-000000000001', '{tenant}'),
    ('00000000-0000-0000-0000-000000000310', '00000000-0000-0000-0000-000000000210', '00000000-0000-0000-0000-000000000001', '{tenant}')
ON CONFLICT (user_id, building_id) DO NOTHING;

-- ============================================================
-- 7. UNIT OWNERSHIP HISTORY
-- ============================================================
-- Current owners (end_date = NULL).
-- Emma owns lots 2 & 8 (she rents out lot 2 to Isabelle).
-- Henri owns lots 3, 9, 10 (he rents out lot 3 to Jules).

INSERT INTO unit_owner (id, unit_id, user_id, start_date)
VALUES
    -- Lot 1 → David Laurent
    ('00000000-0000-0000-0000-000000000401', '00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000204', '2018-06-01'),
    -- Lot 2 → Emma Lefebvre (tenant: Isabelle)
    ('00000000-0000-0000-0000-000000000402', '00000000-0000-0000-0000-000000000102', '00000000-0000-0000-0000-000000000205', '2019-09-01'),
    -- Lot 3 → Henri Chevalier (tenant: Jules)
    ('00000000-0000-0000-0000-000000000403', '00000000-0000-0000-0000-000000000103', '00000000-0000-0000-0000-000000000208', '2015-03-15'),
    -- Lot 4 → François Moreau
    ('00000000-0000-0000-0000-000000000404', '00000000-0000-0000-0000-000000000104', '00000000-0000-0000-0000-000000000206', '2020-02-01'),
    -- Lot 5 → Gisèle Petit
    ('00000000-0000-0000-0000-000000000405', '00000000-0000-0000-0000-000000000105', '00000000-0000-0000-0000-000000000207', '2017-11-15'),
    -- Lot 6 → Bernard Dupont
    ('00000000-0000-0000-0000-000000000406', '00000000-0000-0000-0000-000000000106', '00000000-0000-0000-0000-000000000202', '2010-05-01'),
    -- Lot 7 → Camille Rousseau
    ('00000000-0000-0000-0000-000000000407', '00000000-0000-0000-0000-000000000107', '00000000-0000-0000-0000-000000000203', '2012-07-01'),
    -- Lot 8 → Emma Lefebvre (second unit)
    ('00000000-0000-0000-0000-000000000408', '00000000-0000-0000-0000-000000000108', '00000000-0000-0000-0000-000000000205', '2021-04-01'),
    -- Lot 9 (cellar) → Henri Chevalier
    ('00000000-0000-0000-0000-000000000409', '00000000-0000-0000-0000-000000000109', '00000000-0000-0000-0000-000000000208', '2015-03-15'),
    -- Lot 10 (parking) → Henri Chevalier
    ('00000000-0000-0000-0000-000000000410', '00000000-0000-0000-0000-000000000110', '00000000-0000-0000-0000-000000000208', '2015-03-15')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- SUMMARY
-- ============================================================
-- Role          | Email                              | Units owned
-- --------------|------------------------------------|-----------------
-- manager       | alice.martin@laruche.test          | —
-- board+co_owner| bernard.dupont@laruche.test        | Lot 6 (A-3e)
-- board+co_owner| camille.rousseau@laruche.test      | Lot 7 (A-3e)
-- co_owner      | david.laurent@laruche.test         | Lot 1 (A-RDC)
-- co_owner      | emma.lefebvre@laruche.test         | Lots 2 & 8
-- co_owner      | francois.moreau@laruche.test       | Lot 4 (A-2e)
-- co_owner      | gisele.petit@laruche.test          | Lot 5 (A-2e)
-- co_owner      | henri.chevalier@laruche.test       | Lots 3, 9, 10
-- tenant        | isabelle.fontaine@laruche.test     | (occupies Lot 2)
-- tenant        | jules.martin@laruche.test          | (occupies Lot 3)
--
-- Password (all accounts): Demo1234!
