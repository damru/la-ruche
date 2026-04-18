---
description: 'GED rule: every uploaded file becomes a document row with entity link; never an attachment outside the GED.'
applyTo: 'backend/src/**,frontend/src/**'
---

# Documents & GED

- Every file upload creates a `document` row (GED) in PostgreSQL, then a `{entity}_document` link to the target entity (lot, AG, contract, task, etc.).
- Binary storage lives in RustFS (S3); PostgreSQL stores only metadata.
- No attachment is ever stored outside the GED pipeline (no base64 columns, no file served outside RustFS).
- Deleting an entity does not physically delete documents: the `document` row is unlinked/soft-deleted per the table's soft-delete convention.
- Document access follows the role rules (see the roles-permissions rule).
