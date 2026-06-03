-- EP-002-S03: Add unit_id to user_token for tenant invitation association (AC-GP-002-H).
-- A tenant invitation may be linked to a specific unit so that on acceptance the
-- unit_owner record is created automatically.
ALTER TABLE user_token ADD COLUMN unit_id UUID REFERENCES unit(id);
