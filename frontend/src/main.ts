import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { VueQueryPlugin, QueryClient } from '@tanstack/vue-query'
import { createI18n } from 'vue-i18n'

import App from './App.vue'
import router from './router'
import fr from './i18n/locales/fr.json'
import './styles/tokens.css'
import './styles/base.css'
import './styles/typography.css'
import './styles/tones.css'
import './styles/tabs.css'
import './styles/kpi.css'

// ── TanStack Query ────────────────────────────────────────────────
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5,    // 5 min avant refetch
      gcTime: 1000 * 60 * 30,       // 30 min en cache
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
})

// ── i18n ──────────────────────────────────────────────────────────
const i18n = createI18n({
  legacy: false,
  locale: 'fr',
  fallbackLocale: 'fr',
  messages: { fr },
})

// ── App ───────────────────────────────────────────────────────────
const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(VueQueryPlugin, { queryClient })
app.use(i18n)

app.mount('#app')
