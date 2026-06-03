-- US-GP-008: allow creating residents without email (no_access state).
-- A syndic can register a resident by name+role before having their email address.
-- The NOT NULL constraint is dropped and replaced by a partial unique index so that
-- multiple NULL values are allowed while duplicate emails are still rejected.
ALTER TABLE user_account ALTER COLUMN email DROP NOT NULL;
ALTER TABLE user_account DROP CONSTRAINT IF EXISTS user_account_email_key;
CREATE UNIQUE INDEX IF NOT EXISTS user_account_email_unique
    ON user_account (email)
    WHERE email IS NOT NULL;
