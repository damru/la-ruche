// ── usePermissions — semantic permission checks ──────────────────────────────
//
// Centralised permission logic — every `can<Action><Scope>` function lives here
// so that role-based access rules are defined in exactly one place.
//
// Naming convention mirrors the backend `domain::permissions` module:
//   can<Action><Scope>  where action = View | Manage | Vote | Initiate | Administer
//   and scope = GM | Consultation | Budget | Task | Contract | Document |
//               Logbook | Report | Claim | Unit | User | Accounting |
//               Calendar | EtatDate | Judicial | Governance | Contractor
//
// A user may hold several roles simultaneously (e.g. ['manager', 'co_owner']);
// every check inspects **all** of the user's roles.

import { computed, type ComputedRef } from 'vue'
import { useAuthStore } from '@/stores/auth'
import type { Role } from '@/types/domain'

interface Permissions {
  // ── General Meeting (AG) ──
  /** Anyone authenticated may view AG data. */
  canViewGM: ComputedRef<boolean>
  /** Only co-owners (residents) may vote in a general meeting. */
  canVoteGM: ComputedRef<boolean>
  /** Managers, board members and co-owners may record GM vote entries (session entry flow). */
  canRecordGMVotes: ComputedRef<boolean>
  /** Only the manager may administer AG lifecycle. */
  canManageGM: ComputedRef<boolean>

  // ── Consultation ──
  /** Anyone authenticated may view consultations. */
  canViewConsultation: ComputedRef<boolean>
  /** Only co-owners and tenants may vote in a consultation. */
  canVoteConsultation: ComputedRef<boolean>
  /** Managers and board members manage the consultation lifecycle. */
  canManageConsultation: ComputedRef<boolean>
  /** Anyone authenticated may initiate (create) a consultation. */
  canInitiateConsultation: ComputedRef<boolean>

  // ── Budget ──
  /** Co-owners and tenants may view budgets. */
  canViewBudget: ComputedRef<boolean>
  /** Managers and board members may see draft budgets and levy details. */
  canManageBudget: ComputedRef<boolean>
  /** Only the manager may create / update / close / approve budgets. */
  canAdministerBudget: ComputedRef<boolean>

  // ── Task ──
  canViewTask: ComputedRef<boolean>
  canManageTask: ComputedRef<boolean>

  // ── Contract ──
  canViewContract: ComputedRef<boolean>
  canManageContract: ComputedRef<boolean>
  canAdministerContract: ComputedRef<boolean>

  // ── Document ──
  canViewDocument: ComputedRef<boolean>
  canManageDocument: ComputedRef<boolean>

  // ── Logbook ──
  canViewLogbook: ComputedRef<boolean>
  canManageLogbook: ComputedRef<boolean>

  // ── Report ──
  canViewReport: ComputedRef<boolean>
  canManageReport: ComputedRef<boolean>
  /** Only the manager may change a report's status or convert it to a task. */
  canAdministerReport: ComputedRef<boolean>

  // ── Claim ──
  canViewClaim: ComputedRef<boolean>
  canManageClaim: ComputedRef<boolean>

  // ── Unit ──
  canViewUnit: ComputedRef<boolean>
  canManageUnit: ComputedRef<boolean>

  // ── User ──
  canViewUsers: ComputedRef<boolean>
  canManageUsers: ComputedRef<boolean>

  // ── Accounting / Works fund ──
  canViewAccounting: ComputedRef<boolean>
  canManageAccounting: ComputedRef<boolean>

  // ── Calendar ──
  canViewCalendar: ComputedRef<boolean>
  canManageCalendar: ComputedRef<boolean>

  // ── État daté ──
  canViewEtatDate: ComputedRef<boolean>
  canManageEtatDate: ComputedRef<boolean>

  // ── Judicial proceedings ──
  canViewJudicial: ComputedRef<boolean>
  canManageJudicial: ComputedRef<boolean>

  // ── Governance ──
  canViewGovernance: ComputedRef<boolean>
  canManageGovernance: ComputedRef<boolean>

  // ── Contractor (vendor-facing) ──
  canManageContractor: ComputedRef<boolean>
}

export function usePermissions(): Permissions {
  const auth = useAuthStore()

  const hasRole = (role: Role): boolean => auth.hasRole(role)
  const hasAnyRole = (roles: Role[]): boolean => auth.hasAnyRole(roles)
  const isAuthenticated = computed(() => auth.isAuthenticated)

  // ── General Meeting (AG) ──

  const canViewGM = computed(() => isAuthenticated.value)
  const canVoteGM = computed(() => hasRole('co_owner'))
  const canRecordGMVotes = computed(() =>
    hasAnyRole(['manager', 'board_member', 'co_owner'])
  )
  const canManageGM = computed(() => hasRole('manager'))

  // ── Consultation ──

  const canViewConsultation = computed(() => isAuthenticated.value)
  const canVoteConsultation = computed(() => hasRole('co_owner') || hasRole('tenant'))
  const canManageConsultation = computed(() =>
    hasAnyRole(['manager', 'board_member'])
  )
  const canInitiateConsultation = computed(() => isAuthenticated.value)

  // ── Budget ──

  const canViewBudget = computed(() => isAuthenticated.value)
  const canManageBudget = computed(() =>
    hasAnyRole(['manager', 'board_member'])
  )
  const canAdministerBudget = computed(() => hasRole('manager'))

  // ── Task ──

  const canViewTask = computed(() => isAuthenticated.value)
  const canManageTask = computed(() =>
    hasAnyRole(['manager', 'board_member'])
  )

  // ── Contract ──

  const canViewContract = computed(() =>
    hasAnyRole(['manager', 'board_member'])
  )
  const canManageContract = computed(() =>
    hasAnyRole(['manager', 'board_member'])
  )
  const canAdministerContract = computed(() => hasRole('manager'))

  // ── Document ──

  const canViewDocument = computed(() => isAuthenticated.value)
  const canManageDocument = computed(() =>
    hasAnyRole(['manager', 'board_member'])
  )

  // ── Logbook ──

  const canViewLogbook = computed(() => isAuthenticated.value)
  const canManageLogbook = computed(() =>
    hasAnyRole(['manager', 'board_member'])
  )

  // ── Report ──

  const canViewReport = computed(() => isAuthenticated.value)
  const canManageReport = computed(() =>
    hasAnyRole(['manager', 'board_member'])
  )
  const canAdministerReport = computed(() => hasRole('manager'))

  // ── Claim ──

  const canViewClaim = computed(() => isAuthenticated.value)
  const canManageClaim = computed(() =>
    hasAnyRole(['manager', 'board_member'])
  )

  // ── Unit ──

  const canViewUnit = computed(() => isAuthenticated.value)
  const canManageUnit = computed(() => hasRole('manager'))

  // ── User ──

  const canViewUsers = computed(() =>
    hasAnyRole(['manager', 'board_member'])
  )
  const canManageUsers = computed(() => hasRole('manager'))

  // ── Accounting / Works fund ──

  const canViewAccounting = computed(() =>
    hasAnyRole(['manager', 'board_member'])
  )
  const canManageAccounting = computed(() => hasRole('manager'))

  // ── Calendar ──

  const canViewCalendar = computed(() => isAuthenticated.value)
  const canManageCalendar = computed(() =>
    hasAnyRole(['manager', 'board_member'])
  )

  // ── État daté ──

  const canViewEtatDate = computed(() => isAuthenticated.value)
  const canManageEtatDate = computed(() => hasRole('manager'))

  // ── Judicial proceedings ──

  const canViewJudicial = computed(() => isAuthenticated.value)
  const canManageJudicial = computed(() => hasRole('manager'))

  // ── Governance ──

  const canViewGovernance = computed(() => isAuthenticated.value)
  const canManageGovernance = computed(() => hasRole('manager'))

  // ── Contractor (vendor-facing) ──

  const canManageContractor = computed(() =>
    hasAnyRole(['manager', 'board_member'])
  )

  return {
    canViewGM,
    canVoteGM,
    canRecordGMVotes,
    canManageGM,
    canViewConsultation,
    canVoteConsultation,
    canManageConsultation,
    canInitiateConsultation,
    canViewBudget,
    canManageBudget,
    canAdministerBudget,
    canViewTask,
    canManageTask,
    canViewContract,
    canManageContract,
    canAdministerContract,
    canViewDocument,
    canManageDocument,
    canViewLogbook,
    canManageLogbook,
    canViewReport,
    canManageReport,
    canAdministerReport,
    canViewClaim,
    canManageClaim,
    canViewUnit,
    canManageUnit,
    canViewUsers,
    canManageUsers,
    canViewAccounting,
    canManageAccounting,
    canViewCalendar,
    canManageCalendar,
    canViewEtatDate,
    canManageEtatDate,
    canViewJudicial,
    canManageJudicial,
    canViewGovernance,
    canManageGovernance,
    canManageContractor,
  }
}
