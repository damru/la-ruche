import { defineStore } from 'pinia'
import { usePreferredDark } from '@vueuse/core'
import { ref, computed, watch } from 'vue'

type ThemePreference = 'light' | 'dark' | 'auto'

const THEME_COOKIE_NAME = 'laruche-theme'

function readThemePreference(): ThemePreference {
  if (typeof document === 'undefined') {
    return 'auto'
  }

  const cookieValue = document.cookie
    .split('; ')
    .find(entry => entry.startsWith(`${THEME_COOKIE_NAME}=`))
    ?.split('=')[1]

  return cookieValue === 'light' || cookieValue === 'dark' ? cookieValue : 'auto'
}

function persistThemePreference(preference: ThemePreference) {
  if (typeof document === 'undefined') {
    return
  }

  if (preference === 'auto') {
    document.cookie = `${THEME_COOKIE_NAME}=; Path=/; Max-Age=0; SameSite=Lax`
    return
  }

  document.cookie = `${THEME_COOKIE_NAME}=${preference}; Path=/; Max-Age=31536000; SameSite=Lax`
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
