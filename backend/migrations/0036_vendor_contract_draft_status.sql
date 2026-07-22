-- EP-018-S05: Pattern B — vendor_contract draft/active/amendment lifecycle (ADR-009, PRD Contrats fournisseurs §4)
--
-- PostgreSQL forbids using a freshly added enum value in the same transaction
-- that adds it. The column/backfill/trigger/predecessor_id that consume
-- 'draft' live in 0037 (a later, separate transaction). Mirrors the
-- levy_call precedent (0030 + 0033).

ALTER TYPE contract_status ADD VALUE IF NOT EXISTS 'draft' BEFORE 'active';
