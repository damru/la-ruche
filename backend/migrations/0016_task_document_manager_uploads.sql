CREATE TABLE task_document (
    task_id      UUID NOT NULL REFERENCES task(id) ON DELETE CASCADE,
    document_id  UUID NOT NULL REFERENCES document(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id, document_id)
);

-- Align vendor-submission document links with the rest of the document model:
-- deleting a vendor submission should drop its link rows, and deleting a document
-- from the manager-facing task modal must also clean existing vendor-submission links.
ALTER TABLE vendor_submission_document
    DROP CONSTRAINT vendor_submission_document_submission_id_fkey,
    DROP CONSTRAINT vendor_submission_document_document_id_fkey;

ALTER TABLE vendor_submission_document
    ADD CONSTRAINT vendor_submission_document_submission_id_fkey
        FOREIGN KEY (submission_id) REFERENCES vendor_submission(id) ON DELETE CASCADE,
    ADD CONSTRAINT vendor_submission_document_document_id_fkey
        FOREIGN KEY (document_id) REFERENCES document(id) ON DELETE CASCADE;
