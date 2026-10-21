<script setup lang="ts">
// Composant organique — AppLayout
// Root layout for all authenticated pages
// Sidebar desktop + bottom nav mobile + padding safe-area

import { onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import AppNav from '@/components/organisms/AppNav.vue'
import CMockBanner from '@/components/atoms/CMockBanner.vue'
import { useUiStore } from '@/stores/ui'

// Listen for session-expired event
const router = useRouter()

const ui = useUiStore()

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
    <!-- Bannière mode démo : plein écran au-dessus de la sidebar et du contenu -->
    <CMockBanner
      v-if="ui.demoMode"
      variant="demo"
      class="app-layout__banner"
    />

    <div class="app-layout__row">
      <AppNav>
      <!-- Icônes SVG par nom de route — passées en slots nommés -->
      <template #icon-dashboard>
        <CIconHome />
      </template>
      <template #icon-calendar>
        <CIconCalendar />
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
        <CIconHardHat />
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
        <CIconRegistry />
      </template>
      <template #icon-judicial-proceedings>
        <CIconScale />
      </template>
      <template #icon-governance>
        <CIconShield />
      </template>
      <template #icon-etat-date>
        <CIconEtatDate />
      </template>
      <template #icon-residence-building>
        <CIconSettings />
      </template>
      <template #icon-residence-keys>
        <CIconLayers />
      </template>
      <template #icon-residence-edd>
        <CIconScale />
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
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  background: var(--color-surface);
}

/* Rangée sidebar + contenu : la bannière démo, quand visible,
   s'affiche au-dessus des deux sans les décaler latéralement. */
.app-layout__row {
  display: flex;
  flex: 1;
  min-height: 0;
}

.app-layout__main {
  flex: 1;
  min-width: 0;
  /* Padding bas pour bottom nav sur mobile */
  padding-bottom: 0;
}

.app-layout__banner {
  position: relative;
  z-index: var(--z-nav);
}

@media (max-width: 1024px) {
  .app-layout__row {
    flex-direction: column;
  }
  .app-layout__main {
    padding-bottom: calc(var(--nav-bottom-height) + var(--space-5) + var(--nav-safe-area-bottom));
  }
}
</style>
