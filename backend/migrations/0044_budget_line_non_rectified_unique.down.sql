-- EP-018-S06: Pattern B — budget_line non-rectified uniqueness constraint (down)

-- Drop the partial unique index for non-rectified lines
DROP INDEX IF EXISTS idx_budget_line_non_rectified_unique;
