-- Migration 0012: allow multiple logbook entries per task and add task completion pre-validation

ALTER TYPE task_status ADD VALUE IF NOT EXISTS 'awaiting_validation';

ALTER TABLE task
    ADD COLUMN completion_cost NUMERIC(12, 2);

DROP INDEX IF EXISTS idx_maintenance_log_task;

ALTER TABLE maintenance_log
    DROP CONSTRAINT maintenance_log_task_id_fkey;

ALTER TABLE maintenance_log
    ADD CONSTRAINT maintenance_log_task_id_fkey
    FOREIGN KEY (task_id) REFERENCES task(id) ON DELETE SET NULL;

ALTER TABLE vendor_submission
    ALTER COLUMN task_id DROP NOT NULL;

ALTER TABLE vendor_submission
    DROP CONSTRAINT vendor_submission_task_id_fkey;

ALTER TABLE vendor_submission
    ADD CONSTRAINT vendor_submission_task_id_fkey
    FOREIGN KEY (task_id) REFERENCES task(id) ON DELETE SET NULL;
