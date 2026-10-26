---
description: 'Roles and permissions: manager/board_member/co_owner/tenant + vendor token, 403 over 404, building_id isolation, single-source permission checks, role constants only.'
applyTo: 'backend/src/**,frontend/src/**'
---

# Roles & permissions

- Application roles: `manager` (syndic), `board_member` (council), `co_owner`, `tenant`. Vendors access through a dedicated **token**, scoped to their intervention.
- Access denial → **403**, not 404. Do not mask the existence of a resource from an authenticated user; `building_id` isolation however is strict.
- Every backend query checks the role **and** the building scope (`building_id`), no exception.
- Any new backend route must declare its minimum role in the auth layer; any new view must declare its access level.
- Frontend hiding never replaces the backend check.

## Single source of truth for permission checks

- **Backend** — every role check lives in `domain/permissions.rs` as `can_<action>_<scope>(user)`. Handlers and DB code call these functions; never re-implement a role test inline.
- **Frontend** — every permission check lives in `composables/usePermissions.ts` as `can<Action><Scope>`. Views and components call `usePermissions()`; never write a local `auth.hasRole(...)` / `auth.isManager` check in a view.
- The auth store exposes only primitives (`hasRole`, `hasAnyRole`, `isAuthenticated`) and user data. Router `meta.roles` and nav filtering consume role **lists** from `utils/roles.ts` — that is navigation config, not a permission check.

## Role constants (DRY)

- Role values come from constants only: frontend `utils/roles.ts` (`ROLE_MANAGER`, `MANAGER_ROLES`, …), backend `domain::Role` enum. Never write the string literals `'manager'` / `'board_member'` / `'co_owner'` / `'tenant'` outside the constant definitions, mock data, and badge variant unions.
- New role groups go in `utils/roles.ts` (frontend) — do not redefine them in composables or components.
