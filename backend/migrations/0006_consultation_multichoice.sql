-- 0006: Multi-choice consultation responses
--
-- Each user can now select one OR multiple choices per consultation.
-- The consultation_response table keeps the "header" (user, comment, timestamp).
-- A new junction table consultation_response_choice stores the selected choices.

CREATE TABLE consultation_response_choice (
    response_id UUID NOT NULL REFERENCES consultation_response(id) ON DELETE CASCADE,
    choice_id   UUID NOT NULL REFERENCES consultation_choice(id),
    PRIMARY KEY (response_id, choice_id)
);

-- Migrate existing single-choice data into the new junction table
INSERT INTO consultation_response_choice (response_id, choice_id)
SELECT id, choice_id FROM consultation_response;

-- Remove the now-redundant single-choice column
ALTER TABLE consultation_response DROP COLUMN choice_id;
