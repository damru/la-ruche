-- EP-018-S03: Pattern A — levy_call draft/issue lifecycle (part 1: enum value)
--
-- PostgreSQL forbids using a freshly added enum value in the same transaction
-- that adds it. sqlx runs each migration file in its own transaction, so the
-- ADD VALUE must stay alone here; the column/backfill/trigger that consume
-- 'draft' live in 0033 (a later, separate transaction).

ALTER TYPE levy_status ADD VALUE IF NOT EXISTS 'draft' BEFORE 'issued';
