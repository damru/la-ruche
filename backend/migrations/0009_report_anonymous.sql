-- Optional reporter anonymity: when TRUE, the author identity (author_id,
-- first/last name) is masked in every API serialization except for the
-- reporter themself. author_id stays NOT NULL in DB (accountability);
-- anonymity is a serialization rule, not data deletion. Immutable after
-- creation: no endpoint ever updates this flag.
ALTER TABLE report
    ADD COLUMN is_anonymous BOOLEAN NOT NULL DEFAULT FALSE;
