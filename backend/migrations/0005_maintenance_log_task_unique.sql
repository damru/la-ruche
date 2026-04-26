-- Ensure at most one automatic maintenance log entry per closed task (idempotency).
-- Partial index: only applies when task_id IS NOT NULL.
CREATE UNIQUE INDEX IF NOT EXISTS idx_maintenance_log_task
    ON maintenance_log(task_id) WHERE task_id IS NOT NULL;
