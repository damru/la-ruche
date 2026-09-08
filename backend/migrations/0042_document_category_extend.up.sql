-- EP-005-S07: extend document_category with all PRD categories (LEGAL §3.4, §6.1-6.3)
-- This migration extends the document_category enum to include all categories
-- defined in docs/product/PRD - Documents.md §4, including technical regulatory
-- documents (DPE, DTG, PPT, PV) with their retention rules.

-- First, rename the existing enum to preserve data
ALTER TYPE document_category RENAME TO document_category_old;

-- Create the new enum with all categories from PRD Documents §4
CREATE TYPE document_category AS ENUM (
    'reglement_copropriete',
    'edd',
    'pv_ag',
    'comptabilite',
    'appel_fonds',
    'budget',
    'contrat',
    'diagnostic',
    'sinistre',
    'tache',
    'libre',
    'signalement',
    'carnet_entretien',
    'prestataire',
    'autre'
);

-- Add the new column with the extended enum type
ALTER TABLE document ADD COLUMN category_new document_category;

-- Migrate existing data from old enum to new enum
-- Map old values to new enum values
UPDATE document SET category_new = 
    CASE category::text
        WHEN 'reglement_copropriete' THEN 'reglement_copropriete'::document_category
        WHEN 'edd' THEN 'edd'::document_category
        WHEN 'autre' THEN 'autre'::document_category
        ELSE 'autre'::document_category
    END;

-- Drop the old column and rename the new one
ALTER TABLE document DROP COLUMN IF EXISTS category;
ALTER TABLE document RENAME COLUMN category_new TO category;

-- Recreate the indexes with the new enum
CREATE UNIQUE INDEX IF NOT EXISTS document_supersedes_unique
    ON document (supersedes_id) WHERE supersedes_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS document_current_founding_unique
    ON document (building_id, category)
    WHERE is_current AND category IN ('reglement_copropriete', 'edd');

-- Drop the old enum type
DROP TYPE IF EXISTS document_category_old;
