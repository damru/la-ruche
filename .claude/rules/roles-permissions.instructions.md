---
description: 'Roles and permissions: manager/board_member/owner/tenant + vendor token, 403 over 404, building_id isolation, role-based v-if guards.'
applyTo: 'backend/src/**,frontend/src/**'
---

# Roles & permissions

- Application roles: `manager` (syndic), `board_member` (council), `owner`, `tenant`. Vendors access through a dedicated **token**, scoped to their intervention.
- Access denial → **403**, not 404. Do not mask the existence of a resource from an authenticated user; `building_id` isolation however is strict.
- Every backend query checks the role **and** the building scope (`building_id`), no exception.
- On the frontend, privileged UI elements are guarded by role `v-if` (auth composable/store); frontend hiding never replaces the backend check.
- Any new backend route must declare its minimum role in the auth layer; any new view must declare its access level.
