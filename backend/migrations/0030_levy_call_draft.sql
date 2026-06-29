-- EP-018-S03: Pattern A — levy_call draft/issue lifecycle
--
-- 1. Add 'draft' to levy_status enum (before 'issued' so ordinal is stable)
-- 2. Add status column to levy_call (default 'draft' for new calls)
-- 3. Make issued_at nullable (NULL while draft, set on issue)
-- 4. Backfill existing rows: all pre-migration calls were already issued
-- 5. PG trigger: defensive guard that mirrors domain/immutability.rs

ALTER TYPE levy_status ADD VALUE IF NOT EXISTS 'draft' BEFORE 'issued';

ALTER TABLE levy_call
    ADD COLUMN status levy_status NOT NULL DEFAULT 'draft',
    ALTER COLUMN issued_at DROP NOT NULL;

-- All pre-migration rows were created in 'issued' state (no draft concept existed)
UPDATE levy_call SET status = 'issued';

CREATE OR REPLACE FUNCTION trg_levy_call_immutable()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF OLD.status <> 'draft' THEN
        RAISE EXCEPTION 'levy_call % is frozen (status=%)', OLD.id, OLD.status;
    END IF;
    IF TG_OP = 'DELETE' THEN RETURN OLD; END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER levy_call_immutable
    BEFORE UPDATE OR DELETE ON levy_call
    FOR EACH ROW EXECUTE FUNCTION trg_levy_call_immutable();
