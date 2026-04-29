-- Migration 0010: link report to the task it was converted into
-- • report.task_id  — set when convert_to_task is called, prevents duplicate conversion
-- • report_document — change FK to ON DELETE CASCADE so deleting a report removes its doc links

ALTER TABLE report
    ADD COLUMN task_id UUID REFERENCES task(id) ON DELETE SET NULL;

-- The original FK had no ON DELETE action (defaults to RESTRICT), which would
-- prevent deleting a report that still has attached documents.
ALTER TABLE report_document
    DROP CONSTRAINT report_document_report_id_fkey;

ALTER TABLE report_document
    ADD CONSTRAINT report_document_report_id_fkey
    FOREIGN KEY (report_id) REFERENCES report(id) ON DELETE CASCADE;
