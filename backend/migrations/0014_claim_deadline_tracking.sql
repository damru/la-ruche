CREATE TYPE claim_type AS ENUM (
    'current',
    'natural_disaster',
    'theft'
);

ALTER TABLE claim
    ADD COLUMN claim_type claim_type NOT NULL DEFAULT 'current',
    ADD COLUMN published_at DATE,
    ADD COLUMN transmitted_at DATE;

ALTER TABLE claim
    ADD CONSTRAINT claim_published_at_by_type_check
        CHECK (
            (claim_type = 'natural_disaster' AND published_at IS NOT NULL)
            OR (claim_type <> 'natural_disaster' AND published_at IS NULL)
        ),
    ADD CONSTRAINT claim_transmitted_after_declared_check
        CHECK (transmitted_at IS NULL OR transmitted_at >= declared_at);

CREATE INDEX idx_claim_building_status ON claim (building_id, status);
CREATE INDEX idx_claim_building_type ON claim (building_id, claim_type);
CREATE INDEX idx_claim_building_transmitted_at ON claim (building_id, transmitted_at);
