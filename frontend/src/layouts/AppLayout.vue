<script setup lang="ts">
// Composant organique — AppLayout
// Root layout for all authenticated pages
// Sidebar desktop + bottom nav mobile + padding safe-area

import { onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import AppNav from '@/components/organisms/AppNav.vue'

// Listen for session-expired event
const router = useRouter()
const onSessionExpired = () => {
  router.push({ name: 'login', query: { reason: 'expired' } })
}
onMounted(() => {
  window.addEventListener('laruche:session-expired', onSessionExpired)
})
onUnmounted(() => {
  window.removeEventListener('laruche:session-expired', onSessionExpired)
})
</script>

<template>
  <div class="app-layout">
    <AppNav>
      <!-- Icônes SVG par nom de route — passées en slots nommés -->
      <template #icon-dashboard>
        <CIconHome />
      </template>
      <template #icon-assemblies>
        <CIconAgora />
      </template>
      <template #icon-budget>
        <CIconCurrency />
      </template>
      <template #icon-documents>
        <CIconFolder />
      </template>
      <template #icon-tasks>
        <CIconWrench />
      </template>
      <template #icon-contractors>
        <CIconWorker />
      </template>
      <template #icon-contracts>
        <CIconContract />
      </template>
      <template #icon-claims>
        <CIconWarn />
      </template>
      <template #icon-maintenance>
        <CIconBook />
      </template>
      <template #icon-consultations>
        <CIconMessageSquare />
      </template>
      <template #icon-reports>
        <CIconInfo />
      </template>
      <template #icon-residents>
        <CIconUserPlus />
      </template>
      <template #icon-registry>
        <CIconFile />
      </template>
      <template #icon-residence-building>
        <CIconSettings />
      </template>
      <template #icon-residence-keys>
        <CIconLayers />
      </template>
      <template #icon-my-charges>
        <CIconReceipt />
      </template>
    </AppNav>

    <!-- Zone de contenu -->
    <main class="app-layout__main">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  min-height: 100vh;
  background: var(--color-surface);
}

.app-layout__main {
  flex: 1;
  min-width: 0;
  /* Padding bas pour bottom nav sur mobile */
  padding-bottom: 0;
}

@media (max-width: 1024px) {
  .app-layout {
    flex-direction: column;
  }
  .app-layout__main {
    padding-bottom: calc(var(--nav-bottom-height) + var(--nav-safe-area-bottom));
  }
}
</style>
