-- EP-005-S07: rollback - revert to previous document_category enum

-- First add back the old column
ALTER TABLE document ADD COLUMN category_old TEXT;

-- Copy data from current category to old column
UPDATE document SET category_old = category::text;

-- Drop the new enum and indexes
DROP INDEX IF EXISTS document_current_founding_unique;
DROP INDEX IF EXISTS document_supersedes_unique;

-- Drop the category column
ALTER TABLE document DROP COLUMN IF EXISTS category;

-- Recreate the old enum
CREATE TYPE document_category AS ENUM ('reglement_copropriete', 'edd', 'autre');

-- Add the category column back with old enum
ALTER TABLE document ADD COLUMN category document_category;

-- Migrate data back
UPDATE document SET category = 
    CASE category_old
        WHEN 'reglement_copropriete' THEN 'reglement_copropriete'::document_category
        WHEN 'edd' THEN 'edd'::document_category
        ELSE 'autre'::document_category
    END;

-- Drop the temporary column
ALTER TABLE document DROP COLUMN IF EXISTS category_old;

-- Recreate the indexes
CREATE UNIQUE INDEX document_supersedes_unique
    ON document (supersedes_id) WHERE supersedes_id IS NOT NULL;

CREATE UNIQUE INDEX document_current_founding_unique
    ON document (building_id, category)
    WHERE is_current AND category IN ('reglement_copropriete', 'edd');
