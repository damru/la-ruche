import { defineStore } from 'pinia'
import { usePreferredDark } from '@vueuse/core'
import { ref, computed, watch } from 'vue'

type ThemePreference = 'light' | 'dark' | 'auto'

const THEME_STORAGE_KEY = 'laruche-theme'

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
    setThemePreference,
    activeNavItem,
    setActiveNavItem,
  }
})
