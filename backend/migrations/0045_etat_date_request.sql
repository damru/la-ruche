-- EP-002-S06: etat date request tracking (LEGAL.md §7.1, 15-day legal deadline)
CREATE TYPE etat_date_status AS ENUM ('requested', 'generated', 'sent', 'cancelled');

CREATE TABLE etat_date_request (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    unit_id UUID NOT NULL REFERENCES unit(id),
    requested_at DATE NOT NULL,                      -- date de la demande du notaire
    due_date DATE NOT NULL,                          -- requested_at + 15 jours (calculé en domain)
    requester_name TEXT NOT NULL,                    -- notaire / étude
    status etat_date_status NOT NULL DEFAULT 'requested',
    snapshot JSONB NULL,                             -- état daté figé à la génération
    generated_at TIMESTAMPTZ NULL,
    created_by UUID NOT NULL REFERENCES user_account(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX etat_date_request_unit_idx ON etat_date_request (unit_id);
