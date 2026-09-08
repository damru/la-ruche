DROP INDEX IF EXISTS document_current_founding_unique;
DROP INDEX IF EXISTS document_supersedes_unique;

ALTER TABLE document
    DROP COLUMN IF EXISTS is_current,
    DROP COLUMN IF EXISTS supersedes_id,
    DROP COLUMN IF EXISTS category;

DROP TYPE IF EXISTS document_category;
