-- EP-018-S06: Add version column for optimistic locking on budget_line
--
-- Adds an integer version column (default 1) to budget_line for optimistic
-- concurrency control. The version is incremented on each UPDATE via a trigger.
-- Update operations must include the current version in their WHERE clause
-- and will fail with a 409 Conflict if the version has changed since read.
--
-- This is separate from the temporal versioning (valid_from/valid_to) added
-- in migration 0039. Temporal versioning tracks historical states, while
-- this version column prevents concurrent modification conflicts.

ALTER TABLE budget_line ADD COLUMN version INTEGER NOT NULL DEFAULT 1;

-- Trigger to auto-increment version on any UPDATE to budget_line
CREATE OR REPLACE FUNCTION increment_budget_line_version()
RETURNS TRIGGER AS $$
BEGIN
    NEW.version = OLD.version + 1;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER budget_line_version_trigger
    BEFORE UPDATE ON budget_line
    FOR EACH ROW
    EXECUTE FUNCTION increment_budget_line_version();

-- Backfill existing rows to version 1 (though DEFAULT 1 already covers this)
UPDATE budget_line SET version = 1 WHERE version IS NULL;
