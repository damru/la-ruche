-- EP-005-S08: soft-delete with metadata retention (PRD US-DOC-004/US-DOC-007).
-- The DB row and the S3 object are both kept after deletion; physical purge
-- is deferred to the S3 cleanup job (EP-005-S09) once the category's legal
-- retention has elapsed.

ALTER TABLE document
    ADD COLUMN deleted_at         TIMESTAMPTZ,
    ADD COLUMN deleted_by         UUID REFERENCES user_account(id),
    ADD COLUMN deletion_reason    TEXT,
    ADD COLUMN replacement_reason TEXT;

CREATE INDEX document_deleted_idx ON document (building_id) WHERE deleted_at IS NOT NULL;
