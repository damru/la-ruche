-- Migration 0011: fix report_document.document_id FK
-- The original FK had no ON DELETE action (defaults to RESTRICT), which causes a
-- DB error when trying to delete a document that is linked to a report.
-- Change it to ON DELETE CASCADE so deleting a document automatically removes its
-- report_document links.

ALTER TABLE report_document
    DROP CONSTRAINT report_document_document_id_fkey;

ALTER TABLE report_document
    ADD CONSTRAINT report_document_document_id_fkey
    FOREIGN KEY (document_id) REFERENCES document(id) ON DELETE CASCADE;
