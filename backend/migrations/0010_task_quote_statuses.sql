-- EP-007-S07: quote phase statuses (PRD US-TP-012)
-- ADD VALUE alone in its own migration: the new values are not consumed in the
-- same transaction (precedent: 0030 + 0033 on levy_status).
ALTER TYPE task_status ADD VALUE IF NOT EXISTS 'awaiting_quote' BEFORE 'awaiting_vendor';
ALTER TYPE task_status ADD VALUE IF NOT EXISTS 'quote_received' BEFORE 'awaiting_vendor';
