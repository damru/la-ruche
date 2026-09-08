DROP INDEX IF EXISTS document_deleted_idx;

ALTER TABLE document
    DROP COLUMN IF EXISTS replacement_reason,
    DROP COLUMN IF EXISTS deletion_reason,
    DROP COLUMN IF EXISTS deleted_by,
    DROP COLUMN IF EXISTS deleted_at;
