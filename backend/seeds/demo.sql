-- ============================================================
-- La Ruche — Dev Seed Data
-- ⚠️  Development / testing only — remove before v1.0
-- ============================================================
-- Creates a demo condominium with realistic data:
--   • 1 building  : Résidence Les Cerisiers (Lyon)
--   • 10 units    : 8 apartments, 1 cellar, 1 parking space
--   • 3 alloc keys: Charges Générales, Ascenseur, Eau Froide
--   • 10 users    : 1 manager · 2 board members · 5 co-owners · 2 tenants
--   • 3 contracts : all contract statuses covered
--   • 4 contractors
--   • 6 tasks     : all task statuses covered
--   • 3 reports   : all report statuses covered
--   • 3 claims    : all claim statuses covered
--   • 4 assemblies: draft, called, closed, ongoing
--   • 2 consults  : open + closed with seeded choices/responses
--
-- All accounts share the same password: Demo1234!
-- (Argon2id hash, m=4096 t=1 p=1 — dev-only lightweight params)
--
-- Usage:
--   psql $DATABASE_URL -f backend/seeds/demo.sql
--   set LA_RUCHE__APP__DEMO_MODE=true to auto-initialize the database at startup
-- ============================================================

-- ============================================================
-- RESET: truncate all tables in dependency order
-- ============================================================
TRUNCATE TABLE
    maintenance_log_document, maintenance_log,
    report_document, report,
    vendor_submission_document, vendor_submission,
    task_document,
    task,
    contract_document, contract_reminder, vendor_contract,
    claim_document, claim,
    vote, resolution, general_meeting,
    levy_call, budget_line, budget, works_fund,
    consultation_response_choice, consultation_response, consultation_choice, consultation,
    contractor,
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

-- EP-003-S07: lot 10 (parking, 440 tantièmes) archived after meeting 855 started.
-- Frozen attendance for meetings 853 and 855 still includes it (locked at start time).
-- compute_active_shares_total (draft/called) → 7 560 ; SUM(attendance) for locked meetings → 8 000.
UPDATE unit SET active = false
WHERE id = '00000000-0000-0000-0000-000000000110';

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
-- 7b. BUDGETS, LINES, WORKS FUND AND LEVIES (past + current)
-- ============================================================
INSERT INTO budget (id, building_id, fiscal_year, status, created_at, updated_at)
VALUES
    (
        '00000000-0000-0000-0000-000000000421',
        '00000000-0000-0000-0000-000000000001',
        EXTRACT(YEAR FROM CURRENT_DATE)::int - 1,
        'closed',
        NOW() - INTERVAL '420 days',
        NOW() - INTERVAL '120 days'
    ),
    (
        '00000000-0000-0000-0000-000000000422',
        '00000000-0000-0000-0000-000000000001',
        EXTRACT(YEAR FROM CURRENT_DATE)::int,
        'draft',
        NOW() - INTERVAL '120 days',
        NOW() - INTERVAL '2 days'
    )
ON CONFLICT (id) DO NOTHING;

INSERT INTO budget_line (id, budget_id, label, key_id, budgeted_amount, actual_amount, created_at)
VALUES
    ('00000000-0000-0000-0000-000000000423', '00000000-0000-0000-0000-000000000421', 'Entretien parties communes', '00000000-0000-0000-0000-000000000010', 9800.00, 9620.00, NOW() - INTERVAL '420 days'),
    ('00000000-0000-0000-0000-000000000424', '00000000-0000-0000-0000-000000000421', 'Ascenseur', '00000000-0000-0000-0000-000000000011', 5200.00, 5170.00, NOW() - INTERVAL '420 days'),
    ('00000000-0000-0000-0000-000000000425', '00000000-0000-0000-0000-000000000422', 'Entretien parties communes', '00000000-0000-0000-0000-000000000010', 10400.00, 5110.00, NOW() - INTERVAL '120 days'),
    ('00000000-0000-0000-0000-000000000426', '00000000-0000-0000-0000-000000000422', 'Ascenseur', '00000000-0000-0000-0000-000000000011', 5400.00, 2510.00, NOW() - INTERVAL '120 days'),
    ('00000000-0000-0000-0000-000000000427', '00000000-0000-0000-0000-000000000422', 'Eau froide collective', '00000000-0000-0000-0000-000000000012', 3200.00, 980.00, NOW() - INTERVAL '120 days')
ON CONFLICT (id) DO NOTHING;

INSERT INTO works_fund (id, building_id, fiscal_year, levied_amount, available_amount, created_at, updated_at)
VALUES
    (
        '00000000-0000-0000-0000-000000000428',
        '00000000-0000-0000-0000-000000000001',
        EXTRACT(YEAR FROM CURRENT_DATE)::int - 1,
        2100.00,
        380.00,
        NOW() - INTERVAL '420 days',
        NOW() - INTERVAL '120 days'
    ),
    (
        '00000000-0000-0000-0000-000000000429',
        '00000000-0000-0000-0000-000000000001',
        EXTRACT(YEAR FROM CURRENT_DATE)::int,
        2450.00,
        1640.00,
        NOW() - INTERVAL '120 days',
        NOW() - INTERVAL '2 days'
    )
ON CONFLICT (id) DO NOTHING;

INSERT INTO levy_call (
    id,
    budget_id,
    unit_id,
    amount,
    due_date,
    status,
    key_id,
    shares_snapshot,
    shares_total_snapshot,
    call_label,
    issued_at,
    paid_at,
    created_at
)
VALUES
    (
        '00000000-0000-0000-0000-000000000430',
        '00000000-0000-0000-0000-000000000422',
        '00000000-0000-0000-0000-000000000101',
        2150.00,
        CURRENT_DATE - 45,
        'paid',
        '00000000-0000-0000-0000-000000000010',
        850,
        8000,
        'Trimestriel T1',
        NOW() - INTERVAL '60 days',
        NOW() - INTERVAL '20 days',
        NOW() - INTERVAL '60 days'
    ),
    (
        '00000000-0000-0000-0000-000000000431',
        '00000000-0000-0000-0000-000000000422',
        '00000000-0000-0000-0000-000000000104',
        2410.00,
        CURRENT_DATE - 20,
        'overdue',
        '00000000-0000-0000-0000-000000000010',
        950,
        8000,
        'Trimestriel T1',
        NOW() - INTERVAL '35 days',
        NULL,
        NOW() - INTERVAL '35 days'
    ),
    (
        '00000000-0000-0000-0000-000000000432',
        '00000000-0000-0000-0000-000000000422',
        '00000000-0000-0000-0000-000000000106',
        2520.00,
        CURRENT_DATE + 15,
        'issued',
        '00000000-0000-0000-0000-000000000010',
        980,
        8000,
        'Appel exceptionnel AG 2026-04',
        NOW() - INTERVAL '5 days',
        NULL,
        NOW() - INTERVAL '5 days'
    )
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 8. CONTRACTS
-- One fixture per contract status: active, terminated, expired.
-- Reminder rows also cover all reminder statuses.
-- ============================================================
INSERT INTO vendor_contract (
    id,
    building_id,
    vendor_name,
    vendor_email,
    vendor_phone,
    subject,
    contract_type,
    start_date,
    end_date,
    amount,
    payment_frequency,
    auto_renewal,
    notice_days,
    status,
    created_by,
    created_at,
    updated_at
)
VALUES
    (
        '00000000-0000-0000-0000-000000000451',
        '00000000-0000-0000-0000-000000000001',
        'Alpha Ascenseurs',
        'contact@alpha-ascenseurs.test',
        '+33 4 72 10 00 01',
        'Lift preventive maintenance',
        'maintenance',
        CURRENT_DATE - 380,
        CURRENT_DATE + 350,
        2800.00,
        'annual',
        TRUE,
        90,
        'active',
        '00000000-0000-0000-0000-000000000201',
        NOW() - INTERVAL '380 days',
        NOW() - INTERVAL '3 days'
    ),
    (
        '00000000-0000-0000-0000-000000000452',
        '00000000-0000-0000-0000-000000000001',
        'Hydro Services Lyon',
        'interventions@hydro-services.test',
        '+33 4 72 10 00 02',
        'Water network emergency support',
        'water',
        CURRENT_DATE - 520,
        CURRENT_DATE + 25,
        450.00,
        'quarterly',
        FALSE,
        30,
        'terminated',
        '00000000-0000-0000-0000-000000000202',
        NOW() - INTERVAL '520 days',
        NOW() - INTERVAL '40 days'
    ),
    (
        '00000000-0000-0000-0000-000000000453',
        '00000000-0000-0000-0000-000000000001',
        'Nettoyage Presqu''ile',
        'planning@nettoyage-presquile.test',
        '+33 4 72 10 00 05',
        'Common areas cleaning service',
        'cleaning',
        CURRENT_DATE - 760,
        CURRENT_DATE - 20,
        620.00,
        'monthly',
        FALSE,
        15,
        'expired',
        '00000000-0000-0000-0000-000000000201',
        NOW() - INTERVAL '760 days',
        NOW() - INTERVAL '20 days'
    )
ON CONFLICT (id) DO NOTHING;

INSERT INTO contract_reminder (
    id,
    contract_id,
    reminder_type,
    reminder_date,
    status,
    created_at,
    updated_at
)
VALUES
    (
        '00000000-0000-0000-0000-000000000461',
        '00000000-0000-0000-0000-000000000451',
        'renewal',
        CURRENT_DATE + 260,
        'pending',
        NOW() - INTERVAL '10 days',
        NOW() - INTERVAL '10 days'
    ),
    (
        '00000000-0000-0000-0000-000000000462',
        '00000000-0000-0000-0000-000000000452',
        'termination',
        CURRENT_DATE - 15,
        'sent',
        NOW() - INTERVAL '50 days',
        NOW() - INTERVAL '12 days'
    ),
    (
        '00000000-0000-0000-0000-000000000463',
        '00000000-0000-0000-0000-000000000453',
        'expiration',
        CURRENT_DATE - 35,
        'acknowledged',
        NOW() - INTERVAL '60 days',
        NOW() - INTERVAL '18 days'
    ),
    (
        '00000000-0000-0000-0000-000000000464',
        '00000000-0000-0000-0000-000000000451',
        'payment_due',
        CURRENT_DATE + 20,
        'pending',
        NOW() - INTERVAL '5 days',
        NOW() - INTERVAL '5 days'
    )
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 9. CONTRACTORS
-- Contractors do not have a status enum in the current schema.
-- ============================================================
INSERT INTO contractor (id, building_id, name, email, phone, domains, notes)
VALUES
    (
        '00000000-0000-0000-0000-000000000501',
        '00000000-0000-0000-0000-000000000001',
        'Alpha Ascenseurs',
        'contact@alpha-ascenseurs.test',
        '+33 4 72 10 00 01',
        ARRAY['maintenance', 'service'],
        'Entretien annuel des ascenseurs et interventions d''urgence en cas de panne.'
    ),
    (
        '00000000-0000-0000-0000-000000000502',
        '00000000-0000-0000-0000-000000000001',
        'Hydro Services Lyon',
        'interventions@hydro-services.test',
        '+33 4 72 10 00 02',
        ARRAY['water', 'maintenance'],
        'Prestataire plomberie intervenant sur les réseaux d''eau, fuites et colonnes montantes.'
    ),
    (
        '00000000-0000-0000-0000-000000000503',
        '00000000-0000-0000-0000-000000000001',
        'Serrurerie du Parc',
        'atelier@serrurerie-parc.test',
        '+33 4 72 10 00 03',
        ARRAY['service', 'other'],
        'Serrurier de référence pour les accès aux parties communes et changements de cylindres urgents.'
    ),
    (
        '00000000-0000-0000-0000-000000000504',
        '00000000-0000-0000-0000-000000000001',
        'Toitures Lyonnaises',
        'support@toitures-lyonnaises.test',
        '+33 4 72 10 00 04',
        ARRAY['maintenance', 'other'],
        'Couvreur intervenant sur l''étanchéité des terrasses et la remise en état après tempête.'
    )
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 10. TASKS
-- One fixture per task status: open, in_progress, awaiting_vendor,
-- awaiting_validation, closed, cancelled.
-- ============================================================
INSERT INTO task (
    id,
    building_id,
    title,
    description,
    status,
    priority,
    created_by,
    assigned_to,
    contractor_id,
    contractor_name,
    vendor_token,
    token_expires_at,
    closed_at,
    completion_cost,
    created_at,
    updated_at
)
VALUES
    (
        '00000000-0000-0000-0000-000000000601',
        '00000000-0000-0000-0000-000000000001',
        'Replace broken hallway light on level 2',
        'A common-area light fixture near lot 4 has failed and needs replacement.',
        'open',
        'normal',
        '00000000-0000-0000-0000-000000000201',
        '00000000-0000-0000-0000-000000000202',
        NULL,
        'Electricien de quartier',
        NULL,
        NULL,
        NULL,
        NULL,
        NOW() - INTERVAL '14 days',
        NOW() - INTERVAL '2 days'
    ),
    (
        '00000000-0000-0000-0000-000000000602',
        '00000000-0000-0000-0000-000000000001',
        'Repair leak on the riser above lot 2',
        'The board opened an intervention after repeated humidity reports in the stairwell.',
        'in_progress',
        'high',
        '00000000-0000-0000-0000-000000000202',
        '00000000-0000-0000-0000-000000000201',
        '00000000-0000-0000-0000-000000000502',
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NOW() - INTERVAL '10 days',
        NOW() - INTERVAL '1 day'
    ),
    (
        '00000000-0000-0000-0000-000000000603',
        '00000000-0000-0000-0000-000000000001',
        'Replace basement service door lock',
        'The old cylinder is failing; awaiting contractor visit confirmation.',
        'awaiting_vendor',
        'urgent',
        '00000000-0000-0000-0000-000000000201',
        '00000000-0000-0000-0000-000000000203',
        '00000000-0000-0000-0000-000000000503',
        NULL,
        '00000000-0000-0000-0000-000000009603',
        NOW() + INTERVAL '5 days',
        NULL,
        NULL,
        NOW() - INTERVAL '7 days',
        NOW() - INTERVAL '6 hours'
    ),
    (
        '00000000-0000-0000-0000-000000000604',
        '00000000-0000-0000-0000-000000000001',
        'Validate glazing replacement quote for lobby window',
        'The contractor submitted completion details; the manager must validate the final amount.',
        'awaiting_validation',
        'high',
        '00000000-0000-0000-0000-000000000202',
        '00000000-0000-0000-0000-000000000201',
        NULL,
        'Vitrier du Rhone',
        NULL,
        NULL,
        NULL,
        680.00,
        NOW() - INTERVAL '9 days',
        NOW() - INTERVAL '12 hours'
    ),
    (
        '00000000-0000-0000-0000-000000000605',
        '00000000-0000-0000-0000-000000000001',
        'Repair terrace waterproofing above common corridor',
        'Storm infiltration was fixed after a short roof intervention on the terrace slab.',
        'closed',
        'normal',
        '00000000-0000-0000-0000-000000000201',
        '00000000-0000-0000-0000-000000000202',
        '00000000-0000-0000-0000-000000000504',
        NULL,
        NULL,
        NULL,
        NOW() - INTERVAL '5 days',
        3200.00,
        NOW() - INTERVAL '25 days',
        NOW() - INTERVAL '5 days'
    ),
    (
        '00000000-0000-0000-0000-000000000606',
        '00000000-0000-0000-0000-000000000001',
        'Refresh old notice-board signage',
        'Initial request cancelled after the board decided to redesign all common signage later.',
        'cancelled',
        'low',
        '00000000-0000-0000-0000-000000000203',
        NULL,
        NULL,
        'Signaletique Lyon Centre',
        NULL,
        NULL,
        NULL,
        NULL,
        NOW() - INTERVAL '18 days',
        NOW() - INTERVAL '8 days'
    )
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 11. REPORTS
-- One fixture per report status: open, in_progress, closed.
-- ============================================================
INSERT INTO report (
    id,
    building_id,
    unit_id,
    author_id,
    title,
    description,
    status,
    task_id,
    created_at,
    updated_at
)
VALUES
    (
        '00000000-0000-0000-0000-000000000701',
        '00000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000104',
        '00000000-0000-0000-0000-000000000209',
        'Motion sensor no longer works in the lobby',
        'The entrance light turns on too late at night and leaves the hall dark for several seconds.',
        'open',
        NULL,
        NOW() - INTERVAL '4 days',
        NOW() - INTERVAL '4 days'
    ),
    (
        '00000000-0000-0000-0000-000000000702',
        '00000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000102',
        '00000000-0000-0000-0000-000000000205',
        'Humidity marks around the riser in stair A',
        'The issue was converted into a task and is being handled by the plumbing contractor.',
        'in_progress',
        '00000000-0000-0000-0000-000000000602',
        NOW() - INTERVAL '11 days',
        NOW() - INTERVAL '1 day'
    ),
    (
        '00000000-0000-0000-0000-000000000703',
        '00000000-0000-0000-0000-000000000001',
        NULL,
        '00000000-0000-0000-0000-000000000204',
        'Terrace infiltration above the corridor',
        'The leak was reported after a storm and is now resolved after the roofing intervention.',
        'closed',
        '00000000-0000-0000-0000-000000000605',
        NOW() - INTERVAL '27 days',
        NOW() - INTERVAL '5 days'
    )
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 12. CLAIMS
-- One fixture per claim status: open, in_progress, closed.
-- ============================================================
INSERT INTO claim (
    id,
    building_id,
    title,
    description,
    declared_at,
    status,
    unit_id,
    created_by,
    claim_type,
    published_at,
    transmitted_at,
    reimbursement_date,
    reimbursement_amount,
    created_at,
    updated_at
)
VALUES
    (
        '00000000-0000-0000-0000-000000000801',
        '00000000-0000-0000-0000-000000000001',
        'Water ingress in the cellar corridor',
        'Minor water infiltration observed after heavy rain near the cellar access ramp.',
        CURRENT_DATE - 6,
        'open',
        '00000000-0000-0000-0000-000000000109',
        '00000000-0000-0000-0000-000000000203',
        'current',
        NULL,
        NULL,
        NULL,
        NULL,
        NOW() - INTERVAL '6 days',
        NOW() - INTERVAL '6 days'
    ),
    (
        '00000000-0000-0000-0000-000000000802',
        '00000000-0000-0000-0000-000000000001',
        'Break-in attempt on parking access door',
        'The insurer has been notified after damage to the shared parking entrance lock.',
        CURRENT_DATE - 8,
        'in_progress',
        '00000000-0000-0000-0000-000000000110',
        '00000000-0000-0000-0000-000000000208',
        'theft',
        NULL,
        CURRENT_DATE - 7,
        NULL,
        NULL,
        NOW() - INTERVAL '8 days',
        NOW() - INTERVAL '3 days'
    ),
    (
        '00000000-0000-0000-0000-000000000803',
        '00000000-0000-0000-0000-000000000001',
        'Storm damage on the roof membrane',
        'Natural disaster claim closed after reimbursement for emergency waterproofing work.',
        CURRENT_DATE - 45,
        'closed',
        NULL,
        '00000000-0000-0000-0000-000000000201',
        'natural_disaster',
        CURRENT_DATE - 38,
        CURRENT_DATE - 34,
        CURRENT_DATE - 20,
        4200.00,
        NOW() - INTERVAL '45 days',
        NOW() - INTERVAL '20 days'
    )
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 13. ASSEMBLIES (GENERAL MEETINGS)
-- - annual draft: created, not convoked, no agenda, no resolutions
-- - extraordinary called: convoked with multi-line agenda and resolutions, no votes yet
-- - annual closed: convoked, closed, with multi-line agenda, attendance and votes
-- ============================================================
INSERT INTO general_meeting (
    id,
    building_id,
    meeting_type,
    status,
    notice_date,
    meeting_date,
    location,
    agenda,
    created_by,
    created_at,
    updated_at
)
VALUES
    (
        '00000000-0000-0000-0000-000000000851',
        '00000000-0000-0000-0000-000000000001',
        'annual',
        'draft',
        NULL,
        NOW() + INTERVAL '45 days',
        'Salle commune - RDC',
        NULL,
        '00000000-0000-0000-0000-000000000201',
        NOW() - INTERVAL '3 days',
        NOW() - INTERVAL '3 days'
    ),
    (
        '00000000-0000-0000-0000-000000000852',
        '00000000-0000-0000-0000-000000000001',
        'extraordinary',
        'called',
        CURRENT_DATE - 5,
        NOW() + INTERVAL '20 days',
        'Cabinet du syndic - Salle 2',
        'Vote de travaux urgents sur l''etancheite
    Validation du financement exceptionnel',
        '00000000-0000-0000-0000-000000000201',
        NOW() - INTERVAL '6 days',
        NOW() - INTERVAL '5 days'
    ),
    (
        '00000000-0000-0000-0000-000000000853',
        '00000000-0000-0000-0000-000000000001',
        'annual',
        'closed',
        CURRENT_DATE - 60,
        NOW() - INTERVAL '45 days',
        'Salle commune - RDC',
        'Approbation des comptes de l''exercice
    Augmentation du budget entretien de 12%
    Renouvellement du contrat d''entretien ascenseur
    Reprise partielle des garde-corps en facade
    Lancement du programme d''isolation thermique des pignons',
        '00000000-0000-0000-0000-000000000201',
        NOW() - INTERVAL '75 days',
        NOW() - INTERVAL '44 days'
    )
ON CONFLICT (id) DO NOTHING;

INSERT INTO resolution (id, meeting_id, number, title, description, vote_type, status, created_at)
VALUES
    -- Extraordinary called meeting (no votes yet)
    (
        '00000000-0000-0000-0000-000000000861',
        '00000000-0000-0000-0000-000000000852',
        1,
        'Lancement des travaux de reprise d''etancheite',
        'Autoriser le syndic a engager les travaux d''urgence sur la toiture terrasse.',
        'absolute_majority',
        'pending',
        NOW() - INTERVAL '5 days'
    ),
    (
        '00000000-0000-0000-0000-000000000862',
        '00000000-0000-0000-0000-000000000852',
        2,
        'Appel de fonds exceptionnel',
        'Valider un appel de fonds exceptionnel pour couvrir les travaux urgents.',
        'simple_majority',
        'pending',
        NOW() - INTERVAL '5 days'
    ),
    -- Annual closed meeting (votes seeded below)
    (
        '00000000-0000-0000-0000-000000000863',
        '00000000-0000-0000-0000-000000000853',
        1,
        'Approbation des comptes de l''exercice',
        'Valider les comptes de copropriete de l''exercice ecoule.',
        'simple_majority',
        'passed',
        NOW() - INTERVAL '46 days'
    ),
    (
        '00000000-0000-0000-0000-000000000864',
        '00000000-0000-0000-0000-000000000853',
        2,
        'Augmentation du budget entretien de 12%',
        'Proposition d''augmentation du budget entretien pour l''exercice suivant.',
        'absolute_majority',
        'rejected',
        NOW() - INTERVAL '46 days'
    ),
    (
        '00000000-0000-0000-0000-000000000865',
        '00000000-0000-0000-0000-000000000853',
        3,
        'Renouvellement du contrat d''entretien ascenseur',
        'Valider le renouvellement du contrat de maintenance de l''ascenseur pour trois ans.',
        'simple_majority',
        'passed',
        NOW() - INTERVAL '46 days'
    ),
    (
        '00000000-0000-0000-0000-000000000868',
        '00000000-0000-0000-0000-000000000853',
        4,
        'Reprise partielle des garde-corps en facade',
        'Resolution relevant de l''article 25 : rejetee a la majorite absolue, mais avec plus d''un tiers des tantiemes pour.',
        'absolute_majority',
        'rejected',
        NOW() - INTERVAL '46 days'
    ),
    (
        '00000000-0000-0000-0000-000000000869',
        '00000000-0000-0000-0000-000000000853',
        5,
        'Lancement du programme d''isolation thermique des pignons',
        'Validation des travaux structurants d''isolation en double majorite.',
        'double_majority',
        'passed',
        NOW() - INTERVAL '46 days'
    )
ON CONFLICT (id) DO NOTHING;

INSERT INTO general_meeting_attendance (
    meeting_id,
    unit_id,
    mode,
    representative_name,
    shares_represented
)
VALUES
    ('00000000-0000-0000-0000-000000000853', '00000000-0000-0000-0000-000000000101', 'present',        NULL,                 850),
    ('00000000-0000-0000-0000-000000000853', '00000000-0000-0000-0000-000000000102', 'proxy',          'Bernard Dupont',     900),
    ('00000000-0000-0000-0000-000000000853', '00000000-0000-0000-0000-000000000103', 'correspondence', NULL,                 850),
    ('00000000-0000-0000-0000-000000000853', '00000000-0000-0000-0000-000000000104', 'present',        NULL,                 950),
    ('00000000-0000-0000-0000-000000000853', '00000000-0000-0000-0000-000000000105', 'proxy',          'Camille Rousseau',   900),
    ('00000000-0000-0000-0000-000000000853', '00000000-0000-0000-0000-000000000106', 'present',        NULL,                 980),
    ('00000000-0000-0000-0000-000000000853', '00000000-0000-0000-0000-000000000107', 'absent',         NULL,                 960),
    ('00000000-0000-0000-0000-000000000853', '00000000-0000-0000-0000-000000000108', 'present',        NULL,                1010),
    ('00000000-0000-0000-0000-000000000853', '00000000-0000-0000-0000-000000000109', 'correspondence', NULL,                 160),
    ('00000000-0000-0000-0000-000000000853', '00000000-0000-0000-0000-000000000110', 'absent',         NULL,                 440)
ON CONFLICT (meeting_id, unit_id) DO NOTHING;

INSERT INTO vote (
    id,
    resolution_id,
    user_id,
    unit_id,
    position,
    shares_represented,
    proxy_for,
    created_at
)
VALUES
    -- Resolution 863 (passed, attendance total = 6 600 tantiemes)
    (
        '00000000-0000-0000-0000-000000000871',
        '00000000-0000-0000-0000-000000000863',
        '00000000-0000-0000-0000-000000000204',
        '00000000-0000-0000-0000-000000000101',
        'in_favor',
        850,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000872',
        '00000000-0000-0000-0000-000000000863',
        '00000000-0000-0000-0000-000000000202',
        '00000000-0000-0000-0000-000000000102',
        'in_favor',
        900,
        '00000000-0000-0000-0000-000000000205',
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000873',
        '00000000-0000-0000-0000-000000000863',
        '00000000-0000-0000-0000-000000000208',
        '00000000-0000-0000-0000-000000000103',
        'abstain',
        850,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000874',
        '00000000-0000-0000-0000-000000000863',
        '00000000-0000-0000-0000-000000000206',
        '00000000-0000-0000-0000-000000000104',
        'in_favor',
        950,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000875',
        '00000000-0000-0000-0000-000000000863',
        '00000000-0000-0000-0000-000000000203',
        '00000000-0000-0000-0000-000000000105',
        'against',
        900,
        '00000000-0000-0000-0000-000000000207',
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000876',
        '00000000-0000-0000-0000-000000000863',
        '00000000-0000-0000-0000-000000000202',
        '00000000-0000-0000-0000-000000000106',
        'in_favor',
        980,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000877',
        '00000000-0000-0000-0000-000000000863',
        '00000000-0000-0000-0000-000000000205',
        '00000000-0000-0000-0000-000000000108',
        'in_favor',
        1010,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000878',
        '00000000-0000-0000-0000-000000000863',
        '00000000-0000-0000-0000-000000000208',
        '00000000-0000-0000-0000-000000000109',
        'in_favor',
        160,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    -- Resolution 864 (rejected, for = 2 890 / 8 000)
    (
        '00000000-0000-0000-0000-000000000879',
        '00000000-0000-0000-0000-000000000864',
        '00000000-0000-0000-0000-000000000204',
        '00000000-0000-0000-0000-000000000101',
        'against',
        850,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000880',
        '00000000-0000-0000-0000-000000000864',
        '00000000-0000-0000-0000-000000000202',
        '00000000-0000-0000-0000-000000000102',
        'in_favor',
        900,
        '00000000-0000-0000-0000-000000000205',
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000881',
        '00000000-0000-0000-0000-000000000864',
        '00000000-0000-0000-0000-000000000208',
        '00000000-0000-0000-0000-000000000103',
        'abstain',
        850,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000882',
        '00000000-0000-0000-0000-000000000864',
        '00000000-0000-0000-0000-000000000206',
        '00000000-0000-0000-0000-000000000104',
        'against',
        950,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000883',
        '00000000-0000-0000-0000-000000000864',
        '00000000-0000-0000-0000-000000000203',
        '00000000-0000-0000-0000-000000000105',
        'against',
        900,
        '00000000-0000-0000-0000-000000000207',
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000884',
        '00000000-0000-0000-0000-000000000864',
        '00000000-0000-0000-0000-000000000202',
        '00000000-0000-0000-0000-000000000106',
        'against',
        980,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000885',
        '00000000-0000-0000-0000-000000000864',
        '00000000-0000-0000-0000-000000000205',
        '00000000-0000-0000-0000-000000000108',
        'in_favor',
        1010,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000886',
        '00000000-0000-0000-0000-000000000864',
        '00000000-0000-0000-0000-000000000208',
        '00000000-0000-0000-0000-000000000109',
        'abstain',
        160,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    -- Resolution 865 (passed, renewal contract)
    (
        '00000000-0000-0000-0000-000000000887',
        '00000000-0000-0000-0000-000000000865',
        '00000000-0000-0000-0000-000000000204',
        '00000000-0000-0000-0000-000000000101',
        'in_favor',
        850,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000888',
        '00000000-0000-0000-0000-000000000865',
        '00000000-0000-0000-0000-000000000202',
        '00000000-0000-0000-0000-000000000102',
        'in_favor',
        900,
        '00000000-0000-0000-0000-000000000205',
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000889',
        '00000000-0000-0000-0000-000000000865',
        '00000000-0000-0000-0000-000000000208',
        '00000000-0000-0000-0000-000000000103',
        'abstain',
        850,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000890',
        '00000000-0000-0000-0000-000000000865',
        '00000000-0000-0000-0000-000000000206',
        '00000000-0000-0000-0000-000000000104',
        'in_favor',
        950,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000891',
        '00000000-0000-0000-0000-000000000865',
        '00000000-0000-0000-0000-000000000203',
        '00000000-0000-0000-0000-000000000105',
        'in_favor',
        900,
        '00000000-0000-0000-0000-000000000207',
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000892',
        '00000000-0000-0000-0000-000000000865',
        '00000000-0000-0000-0000-000000000202',
        '00000000-0000-0000-0000-000000000106',
        'in_favor',
        980,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000893',
        '00000000-0000-0000-0000-000000000865',
        '00000000-0000-0000-0000-000000000205',
        '00000000-0000-0000-0000-000000000108',
        'against',
        1010,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000894',
        '00000000-0000-0000-0000-000000000865',
        '00000000-0000-0000-0000-000000000208',
        '00000000-0000-0000-0000-000000000109',
        'in_favor',
        160,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    -- Resolution 868 (rejected under article 25, but > 33% of total shares in favor)
    (
        '00000000-0000-0000-0000-000000000895',
        '00000000-0000-0000-0000-000000000868',
        '00000000-0000-0000-0000-000000000204',
        '00000000-0000-0000-0000-000000000101',
        'in_favor',
        850,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000896',
        '00000000-0000-0000-0000-000000000868',
        '00000000-0000-0000-0000-000000000202',
        '00000000-0000-0000-0000-000000000102',
        'in_favor',
        900,
        '00000000-0000-0000-0000-000000000205',
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000897',
        '00000000-0000-0000-0000-000000000868',
        '00000000-0000-0000-0000-000000000208',
        '00000000-0000-0000-0000-000000000103',
        'abstain',
        850,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000898',
        '00000000-0000-0000-0000-000000000868',
        '00000000-0000-0000-0000-000000000206',
        '00000000-0000-0000-0000-000000000104',
        'against',
        950,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000899',
        '00000000-0000-0000-0000-000000000868',
        '00000000-0000-0000-0000-000000000203',
        '00000000-0000-0000-0000-000000000105',
        'against',
        900,
        '00000000-0000-0000-0000-000000000207',
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000900',
        '00000000-0000-0000-0000-000000000868',
        '00000000-0000-0000-0000-000000000202',
        '00000000-0000-0000-0000-000000000106',
        'in_favor',
        980,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000901',
        '00000000-0000-0000-0000-000000000868',
        '00000000-0000-0000-0000-000000000205',
        '00000000-0000-0000-0000-000000000108',
        'against',
        1010,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000902',
        '00000000-0000-0000-0000-000000000868',
        '00000000-0000-0000-0000-000000000208',
        '00000000-0000-0000-0000-000000000109',
        'abstain',
        160,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    -- Resolution 869 (passed under article 26 double majority)
    (
        '00000000-0000-0000-0000-000000000903',
        '00000000-0000-0000-0000-000000000869',
        '00000000-0000-0000-0000-000000000204',
        '00000000-0000-0000-0000-000000000101',
        'in_favor',
        850,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000904',
        '00000000-0000-0000-0000-000000000869',
        '00000000-0000-0000-0000-000000000202',
        '00000000-0000-0000-0000-000000000102',
        'in_favor',
        900,
        '00000000-0000-0000-0000-000000000205',
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000905',
        '00000000-0000-0000-0000-000000000869',
        '00000000-0000-0000-0000-000000000208',
        '00000000-0000-0000-0000-000000000103',
        'against',
        850,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000906',
        '00000000-0000-0000-0000-000000000869',
        '00000000-0000-0000-0000-000000000206',
        '00000000-0000-0000-0000-000000000104',
        'in_favor',
        950,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000907',
        '00000000-0000-0000-0000-000000000869',
        '00000000-0000-0000-0000-000000000203',
        '00000000-0000-0000-0000-000000000105',
        'in_favor',
        900,
        '00000000-0000-0000-0000-000000000207',
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000908',
        '00000000-0000-0000-0000-000000000869',
        '00000000-0000-0000-0000-000000000202',
        '00000000-0000-0000-0000-000000000106',
        'in_favor',
        980,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000909',
        '00000000-0000-0000-0000-000000000869',
        '00000000-0000-0000-0000-000000000205',
        '00000000-0000-0000-0000-000000000108',
        'in_favor',
        1010,
        NULL,
        NOW() - INTERVAL '45 days'
    ),
    (
        '00000000-0000-0000-0000-000000000910',
        '00000000-0000-0000-0000-000000000869',
        '00000000-0000-0000-0000-000000000208',
        '00000000-0000-0000-0000-000000000109',
        'in_favor',
        160,
        NULL,
        NOW() - INTERVAL '45 days'
    )
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 13b. ONGOING ASSEMBLY
-- Extraordinary meeting currently in session (status = ongoing).
-- Attendance: 8 lots present/represented/correspondence (6 660 / 8 000 tantièmes locked at start).
-- Lot 10 (parking, 440) archived after start → still in frozen attendance (EP-003-S07).
-- Resolutions (all thresholds computed on 8 000 = SUM attendance, not building.shares_total):
--   870  Art. 24 (simple_majority)  — passed     for=4560 > against=1940
--   871  Art. 24 (simple_majority)  — rejected   for=2890 < against=3610
--   872  Art. 25 (absolute_majority)— passed     for=5540 > 4000 (50% of 8000)
--   873  Art. 25 (absolute_majority)— rejected   for=3550 (44.4% ≥ 1/3) → art. 25-1 eligible
--   874  Art. 26 (double_majority)  — rejected   for=4530 (56.6% > 50%) → art. 26-1 eligible
-- ============================================================
INSERT INTO general_meeting (
    id, building_id, meeting_type, status,
    notice_date, meeting_date, location, agenda,
    created_by, created_at, updated_at
)
VALUES (
    '00000000-0000-0000-0000-000000000855',
    '00000000-0000-0000-0000-000000000001',
    'extraordinary',
    'ongoing',
    CURRENT_DATE - 22,
    NOW(),
    'Salle commune - RDC',
    'Approbation du contrat d''entretien de la chaudière
Réfection des canalisations d''eau chaude sanitaire
Modification de la fréquence de nettoyage des parties communes
Installation de bornes de recharge véhicules électriques en parking',
    '00000000-0000-0000-0000-000000000201',
    NOW() - INTERVAL '3 hours',
    NOW()
) ON CONFLICT (id) DO NOTHING;

INSERT INTO resolution (id, meeting_id, number, title, description, vote_type, status, created_at)
VALUES
    -- Résolution 1 — Art. 24 (simple_majority) — PASSED
    -- for=4560 (lots 1+2+3+4+8) > against=1940 (lots 6+7) ; abstain=160 (lot 9)
    (
        '00000000-0000-0000-0000-000000000870',
        '00000000-0000-0000-0000-000000000855',
        1,
        'Approbation du contrat d''entretien de la chaudière',
        'Autoriser le syndic à signer le contrat de maintenance préventive de la chaudière collective avec Therma Services pour une durée de 3 ans.',
        'simple_majority',
        'passed',
        NOW() - INTERVAL '90 minutes'
    ),
    -- Résolution 2 — Art. 24 (simple_majority) — REJECTED
    -- for=2890 (lots 4+6+7) < against=3610 (lots 1+2+3+8) ; non_voter=160 (lot 9)
    (
        '00000000-0000-0000-0000-000000000871',
        '00000000-0000-0000-0000-000000000855',
        2,
        'Modification de la fréquence de nettoyage des parties communes',
        'Passer le nettoyage des parties communes de deux fois par semaine à trois fois par semaine et augmenter le budget annuel en conséquence.',
        'simple_majority',
        'rejected',
        NOW() - INTERVAL '60 minutes'
    ),
    -- Résolution 3 — Art. 25 (absolute_majority) — PASSED
    -- for=5540 (lots 1+2+3+4+6+8) > 4000 (50% of 8000) ; against=960 (lot 7) ; abstain=160 (lot 9)
    (
        '00000000-0000-0000-0000-000000000872',
        '00000000-0000-0000-0000-000000000855',
        3,
        'Réfection des canalisations d''eau chaude sanitaire',
        'Voter les travaux de remplacement des canalisations d''eau chaude sanitaire des colonnes A et B — devis Hydro Services retenu pour 42 000 €.',
        'absolute_majority',
        'passed',
        NOW() - INTERVAL '30 minutes'
    ),
    -- Résolution 4 — Art. 25 (absolute_majority) — REJECTED, art. 25-1 eligible
    -- for=3550 (lots 1+2+3+4 = 44.4% ≥ 1/3 of 8000) — not absolute majority but re-submission possible
    -- against=2950 (lots 6+7+8) ; non_voter=160 (lot 9)
    (
        '00000000-0000-0000-0000-000000000873',
        '00000000-0000-0000-0000-000000000855',
        4,
        'Installation de bornes de recharge véhicules électriques en parking',
        'Voter l''installation de deux bornes de recharge VE dans le parking souterrain. Rejetée à la majorité absolue (art. 25) mais 3 550 tantièmes en faveur (44.4 % ≥ 1/3 des 8 000) — éligible à une seconde délibération immédiate à la majorité simple (art. 25-1).',
        'absolute_majority',
        'rejected',
        NOW() - INTERVAL '10 minutes'
    ),
    -- Résolution 5 — Art. 26 (double_majority) — REJECTED, art. 26-1 eligible
    -- for=4530 (lots 1+2+3+4+6 = 56.6% > 50% of 8000) but < 5334 (2/3) — double maj fails
    -- against=1970 (lots 7+8) ; non_voter=160 (lot 9)
    (
        '00000000-0000-0000-0000-000000000874',
        '00000000-0000-0000-0000-000000000855',
        5,
        'Réfection de la cage d''escalier et rénovation du hall d''entrée',
        'Autoriser les travaux de rénovation complète du hall d''entrée et de la cage d''éscalier (revêtements sol, peinture, éclairage LED) pour un montant de 68 000 € TTC. Rejetée à la double majorité (art. 26) — 4 530 tantièmes favorables (56,6 % > 50 %) — éligible à une seconde délibération à la majorité absolue (art. 26-1).',
        'double_majority',
        'rejected',
        NOW() - INTERVAL '2 minutes'
    )
ON CONFLICT (id) DO NOTHING;

-- Ongoing meeting attendance (EP-003-S07: locked at start time, includes now-archived lot 10)
-- get_meeting_total_shares → SUM(shares_represented) = 8 000 (not compute_active_shares_total = 7 560)
-- Present/represented: lots 1–4, 6–9  (6 660 tantièmes)
-- Absent: lot 5 (900), lot 10 (440) — lot 10 archived after meeting started, still frozen here
INSERT INTO general_meeting_attendance (
    meeting_id, unit_id, mode, representative_name, shares_represented
)
VALUES
    ('00000000-0000-0000-0000-000000000855', '00000000-0000-0000-0000-000000000101', 'present',        NULL,             850),
    ('00000000-0000-0000-0000-000000000855', '00000000-0000-0000-0000-000000000102', 'present',        NULL,             900),
    ('00000000-0000-0000-0000-000000000855', '00000000-0000-0000-0000-000000000103', 'proxy',          'Sophie Bernard', 850),
    ('00000000-0000-0000-0000-000000000855', '00000000-0000-0000-0000-000000000104', 'present',        NULL,             950),
    ('00000000-0000-0000-0000-000000000855', '00000000-0000-0000-0000-000000000105', 'absent',         NULL,             900),
    ('00000000-0000-0000-0000-000000000855', '00000000-0000-0000-0000-000000000106', 'present',        NULL,             980),
    ('00000000-0000-0000-0000-000000000855', '00000000-0000-0000-0000-000000000107', 'present',        NULL,             960),
    ('00000000-0000-0000-0000-000000000855', '00000000-0000-0000-0000-000000000108', 'present',        NULL,            1010),
    ('00000000-0000-0000-0000-000000000855', '00000000-0000-0000-0000-000000000109', 'correspondence', NULL,             160),
    ('00000000-0000-0000-0000-000000000855', '00000000-0000-0000-0000-000000000110', 'absent',         NULL,             440)
ON CONFLICT (meeting_id, unit_id) DO NOTHING;

-- Votes — ongoing meeting
-- Lot 3 (Henri, proxy holder Sophie Bernard) cast by Bernard Dupont (202) as representative
INSERT INTO vote (
    id, resolution_id, user_id, unit_id, position, shares_represented, proxy_for, created_at
)
VALUES
    -- Resolution 870 (Art. 24 — passed: for=4560, against=1940, abstain=160)
    ('00000000-0000-0000-0000-000000000951', '00000000-0000-0000-0000-000000000870', '00000000-0000-0000-0000-000000000204', '00000000-0000-0000-0000-000000000101', 'in_favor',  850, NULL,                                       NOW() - INTERVAL '85 minutes'),
    ('00000000-0000-0000-0000-000000000952', '00000000-0000-0000-0000-000000000870', '00000000-0000-0000-0000-000000000205', '00000000-0000-0000-0000-000000000102', 'in_favor',  900, NULL,                                       NOW() - INTERVAL '85 minutes'),
    ('00000000-0000-0000-0000-000000000953', '00000000-0000-0000-0000-000000000870', '00000000-0000-0000-0000-000000000202', '00000000-0000-0000-0000-000000000103', 'in_favor',  850, '00000000-0000-0000-0000-000000000208',     NOW() - INTERVAL '85 minutes'),
    ('00000000-0000-0000-0000-000000000954', '00000000-0000-0000-0000-000000000870', '00000000-0000-0000-0000-000000000206', '00000000-0000-0000-0000-000000000104', 'in_favor',  950, NULL,                                       NOW() - INTERVAL '85 minutes'),
    ('00000000-0000-0000-0000-000000000955', '00000000-0000-0000-0000-000000000870', '00000000-0000-0000-0000-000000000202', '00000000-0000-0000-0000-000000000106', 'against',   980, NULL,                                       NOW() - INTERVAL '85 minutes'),
    ('00000000-0000-0000-0000-000000000956', '00000000-0000-0000-0000-000000000870', '00000000-0000-0000-0000-000000000203', '00000000-0000-0000-0000-000000000107', 'against',   960, NULL,                                       NOW() - INTERVAL '85 minutes'),
    ('00000000-0000-0000-0000-000000000957', '00000000-0000-0000-0000-000000000870', '00000000-0000-0000-0000-000000000205', '00000000-0000-0000-0000-000000000108', 'in_favor', 1010, NULL,                                       NOW() - INTERVAL '85 minutes'),
    ('00000000-0000-0000-0000-000000000958', '00000000-0000-0000-0000-000000000870', '00000000-0000-0000-0000-000000000208', '00000000-0000-0000-0000-000000000109', 'abstain',   160, NULL,                                       NOW() - INTERVAL '85 minutes'),
    -- Resolution 871 (Art. 24 — rejected: for=2890, against=3610, non_voter=160)
    ('00000000-0000-0000-0000-000000000959', '00000000-0000-0000-0000-000000000871', '00000000-0000-0000-0000-000000000204', '00000000-0000-0000-0000-000000000101', 'against',   850, NULL,                                       NOW() - INTERVAL '55 minutes'),
    ('00000000-0000-0000-0000-000000000960', '00000000-0000-0000-0000-000000000871', '00000000-0000-0000-0000-000000000205', '00000000-0000-0000-0000-000000000102', 'against',   900, NULL,                                       NOW() - INTERVAL '55 minutes'),
    ('00000000-0000-0000-0000-000000000961', '00000000-0000-0000-0000-000000000871', '00000000-0000-0000-0000-000000000202', '00000000-0000-0000-0000-000000000103', 'against',   850, '00000000-0000-0000-0000-000000000208',     NOW() - INTERVAL '55 minutes'),
    ('00000000-0000-0000-0000-000000000962', '00000000-0000-0000-0000-000000000871', '00000000-0000-0000-0000-000000000206', '00000000-0000-0000-0000-000000000104', 'in_favor',  950, NULL,                                       NOW() - INTERVAL '55 minutes'),
    ('00000000-0000-0000-0000-000000000963', '00000000-0000-0000-0000-000000000871', '00000000-0000-0000-0000-000000000202', '00000000-0000-0000-0000-000000000106', 'in_favor',  980, NULL,                                       NOW() - INTERVAL '55 minutes'),
    ('00000000-0000-0000-0000-000000000964', '00000000-0000-0000-0000-000000000871', '00000000-0000-0000-0000-000000000203', '00000000-0000-0000-0000-000000000107', 'in_favor',  960, NULL,                                       NOW() - INTERVAL '55 minutes'),
    ('00000000-0000-0000-0000-000000000965', '00000000-0000-0000-0000-000000000871', '00000000-0000-0000-0000-000000000205', '00000000-0000-0000-0000-000000000108', 'against',  1010, NULL,                                       NOW() - INTERVAL '55 minutes'),
    ('00000000-0000-0000-0000-000000000966', '00000000-0000-0000-0000-000000000871', '00000000-0000-0000-0000-000000000208', '00000000-0000-0000-0000-000000000109', 'non_voter', 160, NULL,                                       NOW() - INTERVAL '55 minutes'),
    -- Resolution 872 (Art. 25 — passed: for=5540 > 4000; against=960; abstain=160)
    ('00000000-0000-0000-0000-000000000967', '00000000-0000-0000-0000-000000000872', '00000000-0000-0000-0000-000000000204', '00000000-0000-0000-0000-000000000101', 'in_favor',  850, NULL,                                       NOW() - INTERVAL '25 minutes'),
    ('00000000-0000-0000-0000-000000000968', '00000000-0000-0000-0000-000000000872', '00000000-0000-0000-0000-000000000205', '00000000-0000-0000-0000-000000000102', 'in_favor',  900, NULL,                                       NOW() - INTERVAL '25 minutes'),
    ('00000000-0000-0000-0000-000000000969', '00000000-0000-0000-0000-000000000872', '00000000-0000-0000-0000-000000000202', '00000000-0000-0000-0000-000000000103', 'in_favor',  850, '00000000-0000-0000-0000-000000000208',     NOW() - INTERVAL '25 minutes'),
    ('00000000-0000-0000-0000-000000000970', '00000000-0000-0000-0000-000000000872', '00000000-0000-0000-0000-000000000206', '00000000-0000-0000-0000-000000000104', 'in_favor',  950, NULL,                                       NOW() - INTERVAL '25 minutes'),
    ('00000000-0000-0000-0000-000000000971', '00000000-0000-0000-0000-000000000872', '00000000-0000-0000-0000-000000000202', '00000000-0000-0000-0000-000000000106', 'in_favor',  980, NULL,                                       NOW() - INTERVAL '25 minutes'),
    ('00000000-0000-0000-0000-000000000972', '00000000-0000-0000-0000-000000000872', '00000000-0000-0000-0000-000000000203', '00000000-0000-0000-0000-000000000107', 'against',   960, NULL,                                       NOW() - INTERVAL '25 minutes'),
    ('00000000-0000-0000-0000-000000000973', '00000000-0000-0000-0000-000000000872', '00000000-0000-0000-0000-000000000205', '00000000-0000-0000-0000-000000000108', 'in_favor', 1010, NULL,                                       NOW() - INTERVAL '25 minutes'),
    ('00000000-0000-0000-0000-000000000974', '00000000-0000-0000-0000-000000000872', '00000000-0000-0000-0000-000000000208', '00000000-0000-0000-0000-000000000109', 'abstain',   160, NULL,                                       NOW() - INTERVAL '25 minutes'),
    -- Resolution 873 (Art. 25 — rejected, art. 25-1 eligible: for=3550 = 44.4% ≥ 1/3; against=2950; non_voter=160)
    ('00000000-0000-0000-0000-000000000975', '00000000-0000-0000-0000-000000000873', '00000000-0000-0000-0000-000000000204', '00000000-0000-0000-0000-000000000101', 'in_favor',  850, NULL,                                       NOW() - INTERVAL '8 minutes'),
    ('00000000-0000-0000-0000-000000000976', '00000000-0000-0000-0000-000000000873', '00000000-0000-0000-0000-000000000205', '00000000-0000-0000-0000-000000000102', 'in_favor',  900, NULL,                                       NOW() - INTERVAL '8 minutes'),
    ('00000000-0000-0000-0000-000000000977', '00000000-0000-0000-0000-000000000873', '00000000-0000-0000-0000-000000000202', '00000000-0000-0000-0000-000000000103', 'in_favor',  850, '00000000-0000-0000-0000-000000000208',     NOW() - INTERVAL '8 minutes'),
    ('00000000-0000-0000-0000-000000000978', '00000000-0000-0000-0000-000000000873', '00000000-0000-0000-0000-000000000206', '00000000-0000-0000-0000-000000000104', 'in_favor',  950, NULL,                                       NOW() - INTERVAL '8 minutes'),
    ('00000000-0000-0000-0000-000000000979', '00000000-0000-0000-0000-000000000873', '00000000-0000-0000-0000-000000000202', '00000000-0000-0000-0000-000000000106', 'against',   980, NULL,                                       NOW() - INTERVAL '8 minutes'),
    ('00000000-0000-0000-0000-000000000980', '00000000-0000-0000-0000-000000000873', '00000000-0000-0000-0000-000000000203', '00000000-0000-0000-0000-000000000107', 'against',   960, NULL,                                       NOW() - INTERVAL '8 minutes'),
    ('00000000-0000-0000-0000-000000000981', '00000000-0000-0000-0000-000000000873', '00000000-0000-0000-0000-000000000205', '00000000-0000-0000-0000-000000000108', 'against',  1010, NULL,                                       NOW() - INTERVAL '8 minutes'),
    ('00000000-0000-0000-0000-000000000982', '00000000-0000-0000-0000-000000000873', '00000000-0000-0000-0000-000000000208', '00000000-0000-0000-0000-000000000109', 'non_voter', 160, NULL,                                       NOW() - INTERVAL '8 minutes'),
    -- Resolution 874 (Art. 26 — rejected, art. 26-1 eligible: for=4530 = 56.6% > 50%; against=1970; non_voter=160)
    -- favorable lots: 1+2+3+4+6 ; against: 7+8 ; non_voter: 9
    ('00000000-0000-0000-0000-000000000983', '00000000-0000-0000-0000-000000000874', '00000000-0000-0000-0000-000000000204', '00000000-0000-0000-0000-000000000101', 'in_favor',  850, NULL,                                       NOW() - INTERVAL '1 minutes'),
    ('00000000-0000-0000-0000-000000000984', '00000000-0000-0000-0000-000000000874', '00000000-0000-0000-0000-000000000205', '00000000-0000-0000-0000-000000000102', 'in_favor',  900, NULL,                                       NOW() - INTERVAL '1 minutes'),
    ('00000000-0000-0000-0000-000000000985', '00000000-0000-0000-0000-000000000874', '00000000-0000-0000-0000-000000000202', '00000000-0000-0000-0000-000000000103', 'in_favor',  850, '00000000-0000-0000-0000-000000000208',     NOW() - INTERVAL '1 minutes'),
    ('00000000-0000-0000-0000-000000000986', '00000000-0000-0000-0000-000000000874', '00000000-0000-0000-0000-000000000206', '00000000-0000-0000-0000-000000000104', 'in_favor',  950, NULL,                                       NOW() - INTERVAL '1 minutes'),
    ('00000000-0000-0000-0000-000000000987', '00000000-0000-0000-0000-000000000874', '00000000-0000-0000-0000-000000000202', '00000000-0000-0000-0000-000000000106', 'in_favor',  980, NULL,                                       NOW() - INTERVAL '1 minutes'),
    ('00000000-0000-0000-0000-000000000988', '00000000-0000-0000-0000-000000000874', '00000000-0000-0000-0000-000000000203', '00000000-0000-0000-0000-000000000107', 'against',   960, NULL,                                       NOW() - INTERVAL '1 minutes'),
    ('00000000-0000-0000-0000-000000000989', '00000000-0000-0000-0000-000000000874', '00000000-0000-0000-0000-000000000205', '00000000-0000-0000-0000-000000000108', 'against',  1010, NULL,                                       NOW() - INTERVAL '1 minutes'),
    ('00000000-0000-0000-0000-000000000990', '00000000-0000-0000-0000-000000000874', '00000000-0000-0000-0000-000000000208', '00000000-0000-0000-0000-000000000109', 'non_voter', 160, NULL,                                       NOW() - INTERVAL '1 minutes')
ON CONFLICT (id) DO NOTHING;

    -- ============================================================
    -- 14. CONSULTATIONS
    -- Two fixtures: one open, one closed. Includes seeded choices and
    -- responses using the multi-choice schema.
    -- ============================================================
    INSERT INTO consultation (
        id,
        building_id,
        title,
        description,
        status,
        visible_to_roles,
        created_by,
        closed_at,
        created_at,
        updated_at
    )
    VALUES
        (
            '00000000-0000-0000-0000-000000000901',
            '00000000-0000-0000-0000-000000000001',
            'Priorité d''aménagement de la cour intérieure',
            'Consultation informelle sur les prochains usages à privilégier pour la cour commune avant arbitrage budgétaire.',
            'open',
            '{manager,board_member,co_owner,tenant}',
            '00000000-0000-0000-0000-000000000201',
            NOW() + INTERVAL '10 days',
            NOW() - INTERVAL '2 days',
            NOW() - INTERVAL '4 hours'
        ),
        (
            '00000000-0000-0000-0000-000000000902',
            '00000000-0000-0000-0000-000000000001',
            'Choix du prochain chantier prioritaire dans le hall',
            'Synthèse des avis exprimés sur la priorité de travaux dans le hall d''entrée.',
            'closed',
            '{manager,board_member,co_owner}',
            '00000000-0000-0000-0000-000000000202',
            NOW() - INTERVAL '3 days',
            NOW() - INTERVAL '18 days',
            NOW() - INTERVAL '3 days'
        )
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO consultation_choice (id, consultation_id, label, order_index)
    VALUES
        ('00000000-0000-0000-0000-000000000911', '00000000-0000-0000-0000-000000000901', 'Végétaliser davantage la cour', 0),
        ('00000000-0000-0000-0000-000000000912', '00000000-0000-0000-0000-000000000901', 'Ajouter un stationnement vélos sécurisé', 1),
        ('00000000-0000-0000-0000-000000000913', '00000000-0000-0000-0000-000000000901', 'Créer un coin assise / détente', 2),
        ('00000000-0000-0000-0000-000000000921', '00000000-0000-0000-0000-000000000902', 'Repeindre le hall et reprendre les plinthes', 0),
        ('00000000-0000-0000-0000-000000000922', '00000000-0000-0000-0000-000000000902', 'Remplacer les luminaires par des LED', 1),
        ('00000000-0000-0000-0000-000000000923', '00000000-0000-0000-0000-000000000902', 'Changer le tapis et la signalétique', 2)
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO consultation_response (id, consultation_id, user_id, comment, created_at)
    VALUES
        (
            '00000000-0000-0000-0000-000000000931',
            '00000000-0000-0000-0000-000000000901',
            '00000000-0000-0000-0000-000000000204',
            'La cour manque surtout de verdure pour l''été.',
            NOW() - INTERVAL '30 hours'
        ),
        (
            '00000000-0000-0000-0000-000000000932',
            '00000000-0000-0000-0000-000000000901',
            '00000000-0000-0000-0000-000000000209',
            'Un espace vélo fermé serait très utile au quotidien.',
            NOW() - INTERVAL '20 hours'
        ),
        (
            '00000000-0000-0000-0000-000000000933',
            '00000000-0000-0000-0000-000000000901',
            '00000000-0000-0000-0000-000000000202',
            'Le projet peut combiner végétalisation et coin assise si le budget suit.',
            NOW() - INTERVAL '8 hours'
        ),
        (
            '00000000-0000-0000-0000-000000000934',
            '00000000-0000-0000-0000-000000000902',
            '00000000-0000-0000-0000-000000000205',
            'La peinture donnerait un vrai coup de propre immédiatement.',
            NOW() - INTERVAL '12 days'
        ),
        (
            '00000000-0000-0000-0000-000000000935',
            '00000000-0000-0000-0000-000000000902',
            '00000000-0000-0000-0000-000000000206',
            'Les LED amélioreraient le confort et réduiraient la consommation.',
            NOW() - INTERVAL '11 days'
        ),
        (
            '00000000-0000-0000-0000-000000000936',
            '00000000-0000-0000-0000-000000000902',
            '00000000-0000-0000-0000-000000000203',
            'Signalétique et tapis peuvent venir dans un second temps.',
            NOW() - INTERVAL '10 days'
        )
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO consultation_response_choice (response_id, choice_id)
    VALUES
        ('00000000-0000-0000-0000-000000000931', '00000000-0000-0000-0000-000000000911'),
        ('00000000-0000-0000-0000-000000000931', '00000000-0000-0000-0000-000000000913'),
        ('00000000-0000-0000-0000-000000000932', '00000000-0000-0000-0000-000000000912'),
        ('00000000-0000-0000-0000-000000000933', '00000000-0000-0000-0000-000000000911'),
        ('00000000-0000-0000-0000-000000000933', '00000000-0000-0000-0000-000000000913'),
        ('00000000-0000-0000-0000-000000000934', '00000000-0000-0000-0000-000000000921'),
        ('00000000-0000-0000-0000-000000000935', '00000000-0000-0000-0000-000000000922'),
        ('00000000-0000-0000-0000-000000000936', '00000000-0000-0000-0000-000000000922'),
        ('00000000-0000-0000-0000-000000000936', '00000000-0000-0000-0000-000000000923')
    ON CONFLICT DO NOTHING;

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
-- Contracts     | 3 fixtures                         | active, terminated, expired
-- Reminders     | 4 fixtures                         | pending, sent, acknowledged
-- Contractors   | 4 referential entries              | maintenance, water, service, other
-- Tasks         | 6 fixtures                         | open, in_progress, awaiting_vendor, awaiting_validation, closed, cancelled
-- Reports       | 3 fixtures                         | open, in_progress, closed
-- Claims        | 3 fixtures                         | open, in_progress, closed
-- Assemblies    | 4 fixtures                         | draft (empty), called (multi-line agenda + pending resolutions), closed (multi-line agenda + mixed attendance + votes, incl. art. 25 and art. 26 cases), ongoing (extraordinary — art. 24 ✓/✗, art. 25 ✓/✗ with art. 25-1 eligible case)
-- Consultations | 2 fixtures                         | open, closed (+ choices & responses)
--
-- Password (all accounts): Demo1234!
