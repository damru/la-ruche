-- EP-005-S06: founding document types + version chain (LEGAL §1.2)

CREATE TYPE document_category AS ENUM ('reglement_copropriete', 'edd', 'autre');

ALTER TABLE document
    ADD COLUMN category document_category NOT NULL DEFAULT 'autre',
    ADD COLUMN supersedes_id UUID REFERENCES document(id),
    ADD COLUMN is_current BOOLEAN NOT NULL DEFAULT TRUE;

-- Linear version chain: a version has at most one successor.
CREATE UNIQUE INDEX document_supersedes_unique
    ON document (supersedes_id) WHERE supersedes_id IS NOT NULL;

-- At most one current version per founding category per building.
CREATE UNIQUE INDEX document_current_founding_unique
    ON document (building_id, category)
    WHERE is_current AND category IN ('reglement_copropriete', 'edd');
