ALTER TABLE claim
    ADD COLUMN reimbursement_date DATE,
    ADD COLUMN reimbursement_amount NUMERIC(12,2);

ALTER TABLE claim
    ADD CONSTRAINT claim_reimbursement_amount_non_negative
        CHECK (reimbursement_amount IS NULL OR reimbursement_amount > 0);

ALTER TABLE claim_document
    DROP CONSTRAINT claim_document_claim_id_fkey,
    DROP CONSTRAINT claim_document_document_id_fkey;

ALTER TABLE claim_document
    ADD CONSTRAINT claim_document_claim_id_fkey
        FOREIGN KEY (claim_id) REFERENCES claim(id) ON DELETE CASCADE,
    ADD CONSTRAINT claim_document_document_id_fkey
        FOREIGN KEY (document_id) REFERENCES document(id) ON DELETE CASCADE;
