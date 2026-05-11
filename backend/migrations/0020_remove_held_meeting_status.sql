-- Remove 'held' from meeting_status enum (lifecycle is now draft → called → ongoing → closed).
-- PostgreSQL does not support DROP VALUE on enums; the workaround is to recreate the type.

-- 1. Rename the old type so we can create the new one with the same name.
ALTER TYPE meeting_status RENAME TO meeting_status_old;

-- 2. Create the clean enum (draft, called, ongoing, closed).
CREATE TYPE meeting_status AS ENUM ('draft', 'called', 'ongoing', 'closed');

-- 3. Migrate the column to the new type.
--    Any residual 'held' row should not exist in production, but guard just in case.
ALTER TABLE general_meeting
    ALTER COLUMN status DROP DEFAULT,
    ALTER COLUMN status TYPE meeting_status
        USING CASE status::text
            WHEN 'held' THEN 'ongoing'::meeting_status
            ELSE status::text::meeting_status
        END,
    ALTER COLUMN status SET DEFAULT 'draft';

-- 4. Drop the old type.
DROP TYPE meeting_status_old;
