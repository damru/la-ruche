ALTER TABLE levy_call
    ADD COLUMN charge_amount NUMERIC(12, 2) NOT NULL DEFAULT 0,
    ADD COLUMN works_fund_amount NUMERIC(12, 2) NOT NULL DEFAULT 0;

UPDATE levy_call
SET charge_amount = amount;
