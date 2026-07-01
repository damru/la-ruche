-- EP-018-S07: Pattern C — append-only corrections
--
-- 1. levy_call: credit notes reference the levy_call they compensate
-- 2. maintenance_log: corrections reference the entry they supersede
--    Manual entries become immutable; corrections are the only way to amend them.

-- Credit note model for levy_call
ALTER TABLE levy_call
    ADD COLUMN corrects_levy_call_id UUID REFERENCES levy_call(id);

-- Correction model for maintenance_log
ALTER TABLE maintenance_log
    ADD COLUMN corrects_entry_id UUID REFERENCES maintenance_log(id);

-- Manual maintenance_log entries are now immutable (append-only corrections only).
-- Re-use the same trigger function pattern as levy_call_immutable.
CREATE OR REPLACE FUNCTION trg_maintenance_log_manual_immutable()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    -- Automatic entries: already guarded at application layer (EP-018-S04).
    -- Manual entries: immutable after creation; use POST /logbook/{id}/correction instead.
    IF OLD.source = 'manual' THEN
        RAISE EXCEPTION 'maintenance_log % is immutable (source=manual); create a correction instead', OLD.id;
    END IF;
    IF TG_OP = 'DELETE' THEN RETURN OLD; END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER maintenance_log_immutable
    BEFORE UPDATE OR DELETE ON maintenance_log
    FOR EACH ROW EXECUTE FUNCTION trg_maintenance_log_manual_immutable();
