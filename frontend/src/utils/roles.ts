// ── Roles ──
// Single source of truth for role values and role groups — mirrors the backend
// `domain::permissions` module and the PG `user_role` enum.
//
// Permission checks live in composables/usePermissions.ts — import the groups
// from here, do not redefine them.

import type { Role } from '@/types/domain'

export const ROLES = ['manager', 'board_member', 'co_owner', 'tenant'] as const satisfies readonly Role[]

export const ROLE_MANAGER: Role = 'manager'
export const ROLE_BOARD_MEMBER: Role = 'board_member'
export const ROLE_CO_OWNER: Role = 'co_owner'
export const ROLE_TENANT: Role = 'tenant'

/** Syndic + conseil syndical — management access. */
export const MANAGER_ROLES: readonly Role[] = [ROLE_MANAGER, ROLE_BOARD_MEMBER]

/** Co-owners + tenants — the residents. */
export const RESIDENT_ROLES: readonly Role[] = [ROLE_CO_OWNER, ROLE_TENANT]

/** All roles that can see documents (i.e. every authenticated role). */
export const DOCUMENT_ROLES: readonly Role[] = ROLES

/** Default roles that can see a document. */
export const DEFAULT_DOCUMENT_VISIBLE_ROLES: Role[] = [ROLE_MANAGER, ROLE_BOARD_MEMBER, ROLE_CO_OWNER]
