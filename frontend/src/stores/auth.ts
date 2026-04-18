import { defineStore } from 'pinia'
import { authApi, onboardingApi } from '@/api'
import type { AuthUser, Role } from '@/types/domain'

export const useAuthStore = defineStore('auth', () => {
  // ── State ──────────────────────────────────────────────────────
  const user = ref<AuthUser | null>(null)
  const isAuthenticated = computed(() => user.value !== null)  /** null = unknown state (not yet checked) */
  const needsOnboarding = ref<boolean | null>(null)
  // Tracks whether restoreSession() has already been attempted this page load
  let sessionRestored = false
  // ── Roles ──────────────────────────────────────────────────────
  const isManager = computed(() =>
    user.value?.roles.includes('manager') ?? false
  )
  const isBoardMember = computed(() =>
    user.value?.roles.includes('board_member') ?? false
  )
  const isCoOwner = computed(() =>
    user.value?.roles.includes('co_owner') ?? false
  )
  const isTenant = computed(() =>
    user.value?.roles.includes('tenant') ?? false
  )

  function hasRole(role: Role): boolean {
    return user.value?.roles.includes(role) ?? false
  }

  function hasAnyRole(roles: Role[]): boolean {
    return roles.some(r => hasRole(r))
  }
  // ── Can vote at meetings (tenant = consultation only) ──────────
  const canVoteMeeting = computed(() =>
    hasAnyRole(['manager', 'board_member', 'co_owner'])
  )

  // ── Can view meeting results (all roles, including tenant) ─────
  const canViewMeetingResults = computed(() => isAuthenticated.value)
  // ── Can view accounting ────────────────────────────────────────
  const canViewAccounting = computed(() =>
    hasAnyRole(['manager', 'board_member'])
  )

  // ── Actions ────────────────────────────────────────────────────
  async function login(email: string, password: string) {
    const { data } = await authApi.login(email, password)
    user.value = {
      user_id:     data.user_id,
      building_id: data.building_id,
      roles:       data.roles,
      last_name:   data.last_name,
      first_name:  data.first_name,
      email,
    }
    needsOnboarding.value = false
  }

  async function logout() {
    await authApi.logout()
    user.value = null
    needsOnboarding.value = null
    sessionRestored = false
  }

  // ── Restore session from httpOnly cookie (called once on first navigation) ──
  async function restoreSession(): Promise<void> {
    if (sessionRestored) return
    sessionRestored = true

    async function hydrateFromMe(): Promise<boolean> {
      try {
        const { data } = await authApi.me()
        user.value = {
          user_id:     data.user_id,
          building_id: data.building_id,
          roles:       data.roles as Role[],
          last_name:   data.last_name,
          first_name:  data.first_name,
          email:       data.email ?? '',
        }
        return true
      } catch {
        return false
      }
    }

    // Fast path: access_token cookie still valid
    if (await hydrateFromMe()) return

    // access_token may have expired (deleted by browser after Max-Age).
    // Try refreshing via the refresh_token cookie, then retry.
    try {
      await authApi.refresh()
      await hydrateFromMe()
    } catch {
      // No valid refresh_token either → stay unauthenticated
    }
  }

  async function checkOnboardingStatus() {
    try {
      const { data } = await onboardingApi.status()
      needsOnboarding.value = data.needs_onboarding
    } catch {
      // On network error, assume the instance is already initialised
      needsOnboarding.value = false
    }
  }

  function setUser(u: AuthUser) {
    user.value = u
  }

  function completeOnboarding(u: AuthUser) {
    user.value = u
    needsOnboarding.value = false
  }

  function clearUser() {
    user.value = null
  }

  return {
    user,
    isAuthenticated,
    needsOnboarding,
    isManager,
    isBoardMember,
    isCoOwner,
    isTenant,
    canVoteMeeting,
    canViewMeetingResults,
    canViewAccounting,
    hasRole,
    hasAnyRole,
    login,
    logout,
    restoreSession,
    checkOnboardingStatus,
    setUser,
    completeOnboarding,
    clearUser,
  }
})
