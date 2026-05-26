import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { VueQueryPlugin, QueryClient } from '@tanstack/vue-query'

import App from './App.vue'
import router from './router'
import i18n from './i18n'
import './styles/base.css'
import './styles/typography.css'
import './styles/tabs.css'
import './styles/forms.css'
import './styles/tables.css'
import './styles/activity-cards.css'
import './styles/tailwind.css'

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

// ── App ───────────────────────────────────────────────────────────
const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(VueQueryPlugin, { queryClient })
app.use(i18n)

app.mount('#app')
