ALTER TABLE levy_call
    ADD COLUMN call_label TEXT NOT NULL DEFAULT 'Trimestriel';

ALTER TABLE levy_call
    ADD CONSTRAINT levy_call_call_label_not_blank
    CHECK (length(trim(call_label)) > 0);
