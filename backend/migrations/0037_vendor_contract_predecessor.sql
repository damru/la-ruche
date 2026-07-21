-- EP-018-S05: Pattern B — vendor_contract amendments (part 2)
--
-- An amendment is a distinct legal act, not an in-place edit. Adding
-- predecessor_id turns vendor_contract into an explicit temporal chain:
-- each amendment is created in 'draft', pre-filled from its predecessor.
-- Activating the amendment closes the predecessor (status='terminated')
-- atomically. The chain head (original contract) has predecessor_id NULL.
--
-- All pre-migration rows were created directly 'active' (no draft concept
-- existed before this migration) — no backfill needed for status, they
-- simply keep their current status.

ALTER TABLE vendor_contract
    ADD COLUMN predecessor_id UUID REFERENCES vendor_contract(id);

-- One contract can have at most one *pending* (draft) amendment at a time —
-- enforced at the application layer (§6, AC-CF-006-C), not by this index.
-- This index only guarantees a linear chain once activated: a given
-- predecessor can have at most one activated successor recorded against it.
CREATE UNIQUE INDEX vendor_contract_predecessor_unique
    ON vendor_contract (predecessor_id)
    WHERE predecessor_id IS NOT NULL;

CREATE INDEX vendor_contract_predecessor_idx
    ON vendor_contract (predecessor_id);

-- Defensive trigger mirroring domain/immutability.rs: financial/legal fields
-- are frozen once the contract leaves 'draft'. Descriptive fields
-- (vendor_name, vendor_email, vendor_phone, subject) stay mutable via the
-- regular UPDATE path regardless of status — enforced by comparing OLD/NEW
-- only on the frozen columns.
CREATE OR REPLACE FUNCTION trg_vendor_contract_immutable()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF OLD.status <> 'draft' AND (
        NEW.amount IS DISTINCT FROM OLD.amount OR
        NEW.start_date IS DISTINCT FROM OLD.start_date OR
        NEW.end_date IS DISTINCT FROM OLD.end_date OR
        NEW.notice_days IS DISTINCT FROM OLD.notice_days OR
        NEW.contract_type IS DISTINCT FROM OLD.contract_type OR
        NEW.payment_frequency IS DISTINCT FROM OLD.payment_frequency OR
        NEW.auto_renewal IS DISTINCT FROM OLD.auto_renewal
    ) THEN
        RAISE EXCEPTION 'vendor_contract % is frozen on financial/legal fields (status=%)', OLD.id, OLD.status;
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER vendor_contract_immutable
    BEFORE UPDATE ON vendor_contract
    FOR EACH ROW EXECUTE FUNCTION trg_vendor_contract_immutable();
