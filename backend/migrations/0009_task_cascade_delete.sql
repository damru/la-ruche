-- Migration 0009: cascade delete maintenance_log entries when a task is deleted
-- The original FK had no ON DELETE action (defaults to RESTRICT), causing a DB error
-- when deleting a closed task that was auto-linked to the carnet d'entretien.

ALTER TABLE maintenance_log
    DROP CONSTRAINT maintenance_log_task_id_fkey;

ALTER TABLE maintenance_log
    ADD CONSTRAINT maintenance_log_task_id_fkey
    FOREIGN KEY (task_id) REFERENCES task(id) ON DELETE CASCADE;
