import { defineStore } from 'pinia'
import { usePreferredDark } from '@vueuse/core'
import { ref, computed, watch } from 'vue'
import { runtimeApi } from '@/api'

type ThemePreference = 'light' | 'dark' | 'auto'

const THEME_STORAGE_KEY = 'laruche-theme'

// ── Notification preferences ────────────────────────────────────
// ponytail: no notification dispatch exists yet, so prefs live client-side
// (localStorage, like the theme). Move server-side when notifications ship.
const NOTIF_STORAGE_KEY = 'laruche-notif-prefs'

export interface NotifEvent {
  id: string
  /** Mandatory events are always on and cannot be disabled (legal / security). */
  mandatory: boolean
  /** First-visit default for optional events (S06). */
  default: boolean
}

// Catalogue of notification events (order = display order).
// Labels live under i18n `settings.notifications.events.<id>` / `<id>Desc`.
export const NOTIF_EVENTS: NotifEvent[] = [
  { id: 'ag_convocation', mandatory: true, default: true },
  { id: 'levy_call_issued', mandatory: true, default: true },
  { id: 'account_security', mandatory: true, default: true },
  { id: 'ag_minutes', mandatory: false, default: true },
  { id: 'ag_vote_reminder', mandatory: false, default: true },
  { id: 'levy_overdue', mandatory: false, default: true },
  { id: 'consultation_opened', mandatory: false, default: true },
  { id: 'report_published', mandatory: false, default: true },
  { id: 'document_published', mandatory: false, default: true },
  { id: 'contract_expiring', mandatory: false, default: true },
  { id: 'claim_created', mandatory: false, default: true },
  { id: 'claim_resolved', mandatory: false, default: false },
  { id: 'task_assigned', mandatory: false, default: true },
  { id: 'task_due', mandatory: false, default: true },
  { id: 'maintenance_due', mandatory: false, default: true },
  { id: 'weekly_digest', mandatory: false, default: false },
]

function defaultNotifPrefs(): Record<string, boolean> {
  return Object.fromEntries(NOTIF_EVENTS.map((e) => [e.id, e.mandatory ? true : e.default]))
}

function readNotifPrefs(): Record<string, boolean> {
  const base = defaultNotifPrefs()
  if (typeof window === 'undefined') {
    return base
  }
  try {
    const raw = localStorage.getItem(NOTIF_STORAGE_KEY)
    if (raw) {
      const saved = JSON.parse(raw) as Record<string, boolean>
      for (const event of NOTIF_EVENTS) {
        if (!event.mandatory && typeof saved[event.id] === 'boolean') {
          base[event.id] = saved[event.id]
        }
      }
    }
  } catch {
    // Corrupt storage → fall back to defaults.
  }
  return base
}

function persistNotifPrefs(prefs: Record<string, boolean>) {
  if (typeof window === 'undefined') {
    return
  }
  localStorage.setItem(NOTIF_STORAGE_KEY, JSON.stringify(prefs))
}

function readThemePreference(): ThemePreference {
  if (typeof window === 'undefined') {
    return 'auto'
  }

  const saved = localStorage.getItem(THEME_STORAGE_KEY)
  return saved === 'light' || saved === 'dark' ? saved : 'auto'
}

function persistThemePreference(preference: ThemePreference) {
  if (typeof window === 'undefined') {
    return
  }

  if (preference === 'auto') {
    localStorage.removeItem(THEME_STORAGE_KEY)
  } else {
    localStorage.setItem(THEME_STORAGE_KEY, preference)
  }
}

export const useUiStore = defineStore('ui', () => {
  const prefersDark = usePreferredDark()

  // ── Theme ────────────────────────────────────────────────────────
  const themePreference = ref<ThemePreference>(readThemePreference())

  const theme = computed<'light' | 'dark'>(() =>
    themePreference.value === 'auto'
      ? (prefersDark.value ? 'dark' : 'light')
      : themePreference.value
  )

  const isDark = computed(() => theme.value === 'dark')

  // ── Runtime flags ───────────────────────────────────────────────
  const demoMode = ref(false)
  const runtimeLoaded = ref(false)

  // Apply data-theme whenever the resolved theme changes (including system changes)
  watch(theme, (resolvedTheme) => {
    if (typeof document !== 'undefined') {
      document.documentElement.setAttribute('data-theme', resolvedTheme)
    }
  }, { immediate: true })

  function setThemePreference(preference: ThemePreference) {
    themePreference.value = preference
    persistThemePreference(preference)
  }

  // ── Notification preferences ───────────────────────────────────
  const notifPrefs = ref<Record<string, boolean>>(readNotifPrefs())

  function setNotifPref(id: string, value: boolean) {
    const event = NOTIF_EVENTS.find((e) => e.id === id)
    if (!event || event.mandatory) {
      return
    }
    notifPrefs.value[id] = value
    persistNotifPrefs(notifPrefs.value)
  }

  function resetNotifPrefs() {
    notifPrefs.value = defaultNotifPrefs()
    persistNotifPrefs(notifPrefs.value)
  }

  async function loadRuntimeStatus() {
    if (runtimeLoaded.value) {
      return
    }

    runtimeLoaded.value = true
    try {
      const { data } = await runtimeApi.status()
      demoMode.value = data.demo_mode
    } catch {
      demoMode.value = false
    }
  }

  // ── Sidebar mobile (bottom nav) ────────────────────────────────
  const activeNavItem = ref<string>('dashboard')

  function setActiveNavItem(item: string) {
    activeNavItem.value = item
  }

  // ── Toast notifications (post-MVP: voir DDR future) ────────────
  // Placeholder — implemented when notifications are added

  return {
    themePreference,
    theme,
    isDark,
    demoMode,
    runtimeLoaded,
    setThemePreference,
    notifPrefs,
    setNotifPref,
    resetNotifPrefs,
    loadRuntimeStatus,
    activeNavItem,
    setActiveNavItem,
  }
})
