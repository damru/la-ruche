-- New claims start as draft, visible to the manager only.
-- declared_at becomes nullable: it is set at declaration time, not at creation.
-- Existing rows keep their status and declared_at: they were all created pre-draft-concept.
ALTER TABLE claim ALTER COLUMN status SET DEFAULT 'draft';
ALTER TABLE claim ALTER COLUMN declared_at DROP NOT NULL;
