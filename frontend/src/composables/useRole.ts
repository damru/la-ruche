// ── useRole — thin re-export of usePermissions for backward compatibility ──
//
// New code should import from `@/composables/usePermissions` directly.
// This file is kept so existing imports keep working during the migration.

import { usePermissions } from './usePermissions'
import { useAuthStore } from '@/stores/auth'
import type { Role } from '@/types/domain'

export function useRole() {
  const auth = useAuthStore()
  const perms = usePermissions()
  return {
    isManager:             auth.isManager,
    isBoardMember:         auth.isBoardMember,
    isCoOwner:             auth.isCoOwner,
    isTenant:              auth.isTenant,
    // Deprecated — use usePermissions().canVoteGM instead
    canVoteMeeting:        perms.canVoteGM,
    canViewMeetingResults: auth.canViewMeetingResults,
    canViewAccounting:     perms.canViewAccounting,
    hasRole:               (r: Role) => auth.hasRole(r),
    hasAnyRole:            (r: Role[]) => auth.hasAnyRole(r),
  }
}
