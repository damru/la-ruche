import { defineStore } from 'pinia'
import { usePreferredDark } from '@vueuse/core'
import { ref, computed, watch } from 'vue'

export const useUiStore = defineStore('ui', () => {
  const prefersDark = usePreferredDark()

  // ── Theme ────────────────────────────────────────────────────────
  // null = follow system, 'light'/'dark' = manual override
  const manualTheme = ref<'light' | 'dark' | null>(
    localStorage.getItem('laruche-theme') as 'light' | 'dark' | null
  )

  const theme = computed<'light' | 'dark'>(() =>
    manualTheme.value ?? (prefersDark.value ? 'dark' : 'light')
  )

  const isDark = computed(() => theme.value === 'dark')

  // Apply data-theme whenever the resolved theme changes (including system changes)
  watch(theme, (t) => {
    document.documentElement.setAttribute('data-theme', t)
  }, { immediate: true })

  function toggleTheme() {
    manualTheme.value = theme.value === 'light' ? 'dark' : 'light'
    localStorage.setItem('laruche-theme', manualTheme.value)
  }

  function setTheme(t: 'light' | 'dark') {
    manualTheme.value = t
    localStorage.setItem('laruche-theme', t)
  }

  // ── Sidebar mobile (bottom nav) ────────────────────────────────
  const activeNavItem = ref<string>('dashboard')

  function setActiveNavItem(item: string) {
    activeNavItem.value = item
  }

  // ── Toast notifications (post-MVP: voir DDR future) ────────────
  // Placeholder — implemented when notifications are added

  return {
    theme,
    isDark,
    toggleTheme,
    setTheme,
    activeNavItem,
    setActiveNavItem,
  }
})
