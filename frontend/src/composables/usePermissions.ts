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
import {
  MANAGER_ROLES,
  RESIDENT_ROLES,
  ROLE_BOARD_MEMBER,
  ROLE_CO_OWNER,
  ROLE_MANAGER,
} from '@/utils/roles'
import type { CalendarSource, Role } from '@/types/domain'

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
  /** Only co-owners may see their own levy calls (My charges). */
  canViewMyCharges: ComputedRef<boolean>

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
  /** Managers, board members and co-owners may view the logbook. Tenants are excluded. */
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
  /** Managers and board members may view and add claim follow-up notes. */
  canManageClaimNotes: ComputedRef<boolean>

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
  /** Whether the caller's role can see the given calendar event source (mirrors the backend `visible_sources` rule). */
  canViewCalendarSource: (source: CalendarSource) => boolean

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
  const hasAnyRole = (roles: readonly Role[]): boolean => auth.hasAnyRole([...roles])
  const isAuthenticated = computed(() => auth.isAuthenticated)

  // ── General Meeting (AG) ──

  const canViewGM = computed(() => isAuthenticated.value)
  const canVoteGM = computed(() => hasRole(ROLE_CO_OWNER))
  const canRecordGMVotes = computed(() =>
    hasAnyRole([ROLE_MANAGER, ROLE_BOARD_MEMBER, ROLE_CO_OWNER]),
  )
  const canManageGM = computed(() => hasRole(ROLE_MANAGER))

  // ── Consultation ──

  const canViewConsultation = computed(() => isAuthenticated.value)
  const canVoteConsultation = computed(() => hasAnyRole(RESIDENT_ROLES))
  const canManageConsultation = computed(() => hasAnyRole(MANAGER_ROLES))
  const canInitiateConsultation = computed(() => isAuthenticated.value)

  // ── Budget ──

  const canViewBudget = computed(() => isAuthenticated.value)
  const canManageBudget = computed(() => hasAnyRole(MANAGER_ROLES))
  const canAdministerBudget = computed(() => hasRole(ROLE_MANAGER))
  const canViewMyCharges = computed(() => hasRole(ROLE_CO_OWNER))

  // ── Task ──

  const canViewTask = computed(() => isAuthenticated.value)
  const canManageTask = computed(() => hasAnyRole(MANAGER_ROLES))

  // ── Contract ──

  const canViewContract = computed(() => hasAnyRole(MANAGER_ROLES))
  const canManageContract = computed(() => hasAnyRole(MANAGER_ROLES))
  const canAdministerContract = computed(() => hasRole(ROLE_MANAGER))

  // ── Document ──

  const canViewDocument = computed(() => isAuthenticated.value)
  const canManageDocument = computed(() => hasAnyRole(MANAGER_ROLES))

  // ── Logbook ──

  /** Managers, board members and co-owners may view the logbook. Tenants are excluded. */
  const canViewLogbook = computed(() =>
    hasAnyRole([ROLE_MANAGER, ROLE_BOARD_MEMBER, ROLE_CO_OWNER]),
  )
  const canManageLogbook = computed(() => hasAnyRole(MANAGER_ROLES))

  // ── Report ──

  const canViewReport = computed(() => isAuthenticated.value)
  const canManageReport = computed(() => hasAnyRole(MANAGER_ROLES))
  const canAdministerReport = computed(() => hasRole(ROLE_MANAGER))

  // ── Claim ──

  const canViewClaim = computed(() => isAuthenticated.value)
  const canManageClaim = computed(() => hasRole(ROLE_MANAGER))
  const canManageClaimNotes = computed(() => hasAnyRole(MANAGER_ROLES))

  // ── Unit ──

  const canViewUnit = computed(() => isAuthenticated.value)
  const canManageUnit = computed(() => hasRole(ROLE_MANAGER))

  // ── User ──

  const canViewUsers = computed(() => hasAnyRole(MANAGER_ROLES))
  const canManageUsers = computed(() => hasRole(ROLE_MANAGER))

  // ── Accounting / Works fund ──

  const canViewAccounting = computed(() => hasAnyRole(MANAGER_ROLES))
  const canManageAccounting = computed(() => hasRole(ROLE_MANAGER))

  // ── Calendar ──

  const canViewCalendar = computed(() => isAuthenticated.value)
  const canManageCalendar = computed(() => hasAnyRole(MANAGER_ROLES))
  // Mirrors the backend `visible_sources` rule: Manager/BoardMember see every
  // source; CoOwner/Tenant only see meetings and manual events.
  const canViewCalendarSource = (source: CalendarSource): boolean =>
    source === 'meeting' || source === 'manual'
      ? isAuthenticated.value
      : hasAnyRole(MANAGER_ROLES)


  // ── État daté ──

  const canViewEtatDate = computed(() => isAuthenticated.value)
  const canManageEtatDate = computed(() => hasRole(ROLE_MANAGER))

  // ── Judicial proceedings ──

  const canViewJudicial = computed(() => isAuthenticated.value)
  const canManageJudicial = computed(() => hasRole(ROLE_MANAGER))

  // ── Governance ──

  const canViewGovernance = computed(() => isAuthenticated.value)
  const canManageGovernance = computed(() => hasRole(ROLE_MANAGER))

  // ── Contractor (vendor-facing) ──

  const canManageContractor = computed(() => hasAnyRole(MANAGER_ROLES))

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
    canViewMyCharges,
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
    canManageClaimNotes,
    canViewUnit,
    canManageUnit,
    canViewUsers,
    canManageUsers,
    canViewAccounting,
    canManageAccounting,
    canViewCalendar,
    canManageCalendar,
    canViewCalendarSource,
    canViewEtatDate,
    canManageEtatDate,
    canViewJudicial,
    canManageJudicial,
    canViewGovernance,
    canManageGovernance,
    canManageContractor,
  }
}
