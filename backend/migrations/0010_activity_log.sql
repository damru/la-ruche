-- Polymorphic business activity log feeding the report/task lifecycle timelines.
-- Distinct from audit_log: audit_log is the accounting-grade evidentiary trail;
-- activity_log is the user-facing lifecycle feed. Append-only by convention:
-- no UPDATE/DELETE path is exposed by the application.

CREATE TABLE activity_log (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type TEXT NOT NULL,
    entity_id   UUID NOT NULL,
    action      TEXT NOT NULL,
    actor_id    UUID REFERENCES user_account(id),
    payload     JSONB,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX activity_log_entity_idx
    ON activity_log (entity_type, entity_id, created_at DESC);

-- Retro-fill of pre-existing rows. Only maintenance_log and vendor_submission
-- transitions are reconstructible from durable data; every other historical
-- event (report creation, past status changes, photos, conversions, ...) is
-- not recoverable and is intentionally not backfilled.
-- The pre-closure status is not recoverable; only the log entry id is kept.
INSERT INTO activity_log (entity_type, entity_id, action, actor_id, payload, created_at)
SELECT 'task', ml.task_id, 'closed', ml.created_by,
       jsonb_build_object('log_entry_id', ml.id),
       ml.created_at
FROM maintenance_log ml
WHERE ml.source = 'automatic' AND ml.task_id IS NOT NULL;

INSERT INTO activity_log (entity_type, entity_id, action, actor_id, payload, created_at)
SELECT 'task', vs.task_id, 'vendor_submission_received', NULL,
       jsonb_build_object('submission_id', vs.id),
       vs.submitted_at
FROM vendor_submission vs
WHERE vs.task_id IS NOT NULL;
