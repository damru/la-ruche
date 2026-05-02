CREATE INDEX IF NOT EXISTS idx_contract_document_document_id
    ON contract_document(document_id);

CREATE INDEX IF NOT EXISTS idx_report_document_document_id
    ON report_document(document_id);

CREATE INDEX IF NOT EXISTS idx_claim_document_document_id
    ON claim_document(document_id);
