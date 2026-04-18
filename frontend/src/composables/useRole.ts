// ── useRole — reactive role access without importing the store everywhere
import { useAuthStore } from '@/stores/auth'
import type { Role } from '@/types/domain'

export function useRole() {
  const auth = useAuthStore()
  return {
    isManager:             auth.isManager,
    isBoardMember:         auth.isBoardMember,
    isCoOwner:             auth.isCoOwner,
    isTenant:              auth.isTenant,
    canVoteMeeting:        auth.canVoteMeeting,
    canViewMeetingResults: auth.canViewMeetingResults,
    canViewAccounting:     auth.canViewAccounting,
    hasRole:               (r: Role) => auth.hasRole(r),
    hasAnyRole:            (r: Role[]) => auth.hasAnyRole(r),
  }
}
