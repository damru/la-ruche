-- EP-018-S02: Append-only audit log for accounting immutability (ADR-009)
-- Conservation 10 ans (art. L123-22 Code de commerce)
CREATE TABLE audit_log (
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id  UUID NOT NULL REFERENCES building(id),
    entity_type  TEXT NOT NULL,        -- 'task' | 'claim' | 'levy_call' | …
    entity_id    UUID NOT NULL,
    operation    TEXT NOT NULL,        -- 'UPDATE' | 'DELETE'
    changed_by   UUID REFERENCES user_account(id),  -- NULL = system action
    changed_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    old_values   JSONB,                -- modified fields only, values before
    new_values   JSONB                 -- modified fields only, values after (NULL if DELETE)
);

CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);
CREATE INDEX idx_audit_log_building ON audit_log(building_id, changed_at DESC);
