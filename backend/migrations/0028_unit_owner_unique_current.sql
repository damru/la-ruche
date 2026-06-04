-- Remove duplicate current-owner rows that may have been created without a uniqueness guard,
-- keeping the most recently created row per unit.
DELETE FROM unit_owner
WHERE id NOT IN (
    SELECT DISTINCT ON (unit_id) id
    FROM unit_owner
    WHERE end_date IS NULL
    ORDER BY unit_id, created_at DESC
)
AND end_date IS NULL;

-- Guarantee at the DB level that a unit has at most one current owner (end_date IS NULL).
CREATE UNIQUE INDEX uq_unit_owner_current ON unit_owner(unit_id) WHERE end_date IS NULL;
