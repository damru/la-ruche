import { defineStore } from 'pinia'
import { usePreferredDark } from '@vueuse/core'
import { ref, computed, watch } from 'vue'
import { notificationPreferencesApi, runtimeApi } from '@/api'

type ThemePreference = 'light' | 'dark' | 'auto'

const THEME_STORAGE_KEY = 'laruche-theme'

// ── Notification preferences ────────────────────────────────────
// ponytail: no notification dispatch exists yet, so prefs have no sending effect.
// Values are persisted server-side; localStorage is only a fallback cache
// for offline/network-error situations.
const NOTIF_STORAGE_KEY = 'laruche-notif-prefs'
const NOTIF_MIGRATED_KEY = 'laruche-notif-prefs-migrated'

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

// ── Modal auto-close delay ────────────────────────────────────────────────────
const AUTO_CLOSE_DELAY_STORAGE_KEY = 'laruche-auto-close-delay'
export const AUTO_CLOSE_DELAY_DEFAULT_SEC = 3
export const AUTO_CLOSE_DELAY_MIN_SEC = 0
export const AUTO_CLOSE_DELAY_MAX_SEC = 10

function readAutoCloseDelay(): number {
  if (typeof window === 'undefined') return AUTO_CLOSE_DELAY_DEFAULT_SEC
  const raw = Number(localStorage.getItem(AUTO_CLOSE_DELAY_STORAGE_KEY))
  if (!Number.isInteger(raw) || raw < AUTO_CLOSE_DELAY_MIN_SEC || raw > AUTO_CLOSE_DELAY_MAX_SEC) {
    return AUTO_CLOSE_DELAY_DEFAULT_SEC
  }
  return raw
}

function persistAutoCloseDelay(seconds: number) {
  if (typeof window === 'undefined') return
  if (seconds === AUTO_CLOSE_DELAY_DEFAULT_SEC) {
    localStorage.removeItem(AUTO_CLOSE_DELAY_STORAGE_KEY)
  } else {
    localStorage.setItem(AUTO_CLOSE_DELAY_STORAGE_KEY, String(seconds))
  }
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
  const notifPrefsLoaded = ref(false)

  function applyNotifPrefs(events: { key: string; enabled: boolean }[]) {
    // Intersection: only events known to the frontend catalog are applied;
    // unknown server keys are ignored.
    const next = defaultNotifPrefs()
    for (const event of events) {
      const known = NOTIF_EVENTS.find((e) => e.id === event.key)
      if (known && !known.mandatory) {
        next[event.key] = event.enabled
      }
    }
    notifPrefs.value = next
    persistNotifPrefs(next)
  }

  /**
   * Hydrates notification preferences from the server.
   * On success, also refreshes the localStorage cache. One-shot migration:
   * local values differing from server defaults are pushed to the server
   * when no server override exists yet. On network error, falls back to the
   * localStorage cache without breaking the UI.
   */
  async function loadNotifPrefs() {
    if (notifPrefsLoaded.value) {
      return
    }
    notifPrefsLoaded.value = true
    let serverEvents: { key: string; enabled: boolean }[]
    let serverHasOverrides: boolean
    try {
      const { data } = await notificationPreferencesApi.get()
      serverEvents = data.events
      serverHasOverrides = data.events.some((e) => {
        const known = NOTIF_EVENTS.find((ev) => ev.id === e.key)
        return known != null && !known.mandatory && e.enabled !== known.default
      })
    } catch {
      // Network error → keep the localStorage fallback values.
      return
    }
    applyNotifPrefs(serverEvents)

    // One-shot migration of pre-existing localStorage values.
    if (typeof window !== 'undefined' && !localStorage.getItem(NOTIF_MIGRATED_KEY) && !serverHasOverrides) {
      const local = readNotifPrefs()
      const defaults = defaultNotifPrefs()
      const diff: Record<string, boolean> = {}
      for (const event of NOTIF_EVENTS) {
        if (!event.mandatory && local[event.id] !== defaults[event.id]) {
          diff[event.id] = local[event.id]
        }
      }
      if (Object.keys(diff).length > 0) {
        try {
          const { data } = await notificationPreferencesApi.update(diff)
          applyNotifPrefs(data.events)
        } catch {
          // Migration failed — retried on next visit (flag not set).
          return
        }
      }
      localStorage.setItem(NOTIF_MIGRATED_KEY, '1')
    }
  }

  function setNotifPref(id: string, value: boolean) {
    const event = NOTIF_EVENTS.find((e) => e.id === id)
    if (!event || event.mandatory) {
      return
    }
    const previous = notifPrefs.value[id]
    // Optimistic local update + cache write.
    notifPrefs.value[id] = value
    persistNotifPrefs(notifPrefs.value)
    notificationPreferencesApi.update({ [id]: value }).catch(() => {
      // Rollback on failure.
      notifPrefs.value[id] = previous
      persistNotifPrefs(notifPrefs.value)
    })
  }

  async function resetNotifPrefs() {
    const defaults = defaultNotifPrefs()
    notifPrefs.value = defaults
    persistNotifPrefs(defaults)
    // Push optional-event defaults to the server (multi-device coherence).
    const prefs: Record<string, boolean> = {}
    for (const event of NOTIF_EVENTS) {
      if (!event.mandatory) {
        prefs[event.id] = defaults[event.id]
      }
    }
    try {
      const { data } = await notificationPreferencesApi.update(prefs)
      applyNotifPrefs(data.events)
    } catch {
      // Network error → local reset stands (localStorage fallback).
    }
  }

  // ── Global preferences reset ──────────────────────────────────────────────────

  /** Restores appearance defaults: theme preference back to 'auto'. */
  function resetAppearance() {
    setThemePreference('auto')
    setAutoCloseDelay(AUTO_CLOSE_DELAY_DEFAULT_SEC)
  }

  /** Global reset: notifications + appearance in one action. */
  async function resetAllPrefs() {
    await resetNotifPrefs()
    resetAppearance()
  }

  // ── Modal auto-close delay ────────────────────────────────────────────────
  const autoCloseDelaySec = ref<number>(readAutoCloseDelay())

  function setAutoCloseDelay(seconds: number) {
    if (!Number.isInteger(seconds) || seconds < AUTO_CLOSE_DELAY_MIN_SEC || seconds > AUTO_CLOSE_DELAY_MAX_SEC) {
      return
    }
    autoCloseDelaySec.value = seconds
    persistAutoCloseDelay(seconds)
  }

  /** Delay in ms for timers/animations; 0 disables the success window entirely. */
  const autoCloseDelayMs = computed(() => autoCloseDelaySec.value * 1000)

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
    notifPrefsLoaded,
    setNotifPref,
    resetNotifPrefs,
    loadNotifPrefs,
    resetAppearance,
    resetAllPrefs,
    autoCloseDelaySec,
    autoCloseDelayMs,
    setAutoCloseDelay,
    loadRuntimeStatus,
    activeNavItem,
    setActiveNavItem,
  }
})
