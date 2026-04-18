<script setup lang="ts">
// Composant organique — AppLayout
// Root layout for all authenticated pages
// Sidebar desktop + bottom nav mobile + padding safe-area

import AppNav from '@/components/organisms/AppNav.vue'
import CIconHome from '@/components/atoms/CIconHome.vue'
import CIconUsers from '@/components/atoms/CIconUsers.vue'
import CIconCurrency from '@/components/atoms/CIconCurrency.vue'
import CIconFile from '@/components/atoms/CIconFile.vue'
import CIconWrench from '@/components/atoms/CIconWrench.vue'
import CIconWarn from '@/components/atoms/CIconWarn.vue'
import CIconBook from '@/components/atoms/CIconBook.vue'
import CIconMessageSquare from '@/components/atoms/CIconMessageSquare.vue'
import CIconInfo from '@/components/atoms/CIconInfo.vue'
import CIconUserPlus from '@/components/atoms/CIconUserPlus.vue'
import CIconSettings from '@/components/atoms/CIconSettings.vue'

// Listen for session-expired event
const router = useRouter()
onMounted(() => {
  window.addEventListener('laruche:session-expired', () => {
    router.push({ name: 'login', query: { reason: 'expired' } })
  })
})
</script>

<template>
  <div class="app-layout">
    <AppNav>
      <!-- Icônes SVG par nom de route — passées en slots nommés -->
      <template #icon-dashboard><CIconHome/></template>
      <template #icon-assemblies><CIconUsers/></template>
      <template #icon-budget><CIconCurrency/></template>
      <template #icon-documents><CIconFile/></template>
      <template #icon-tasks><CIconWrench/></template>
      <template #icon-contracts><CIconFile/></template>
      <template #icon-claims><CIconWarn/></template>
      <template #icon-maintenance><CIconBook/></template>
      <template #icon-consultations><CIconMessageSquare/></template>
      <template #icon-reports><CIconInfo/></template>
      <template #icon-residents><CIconUserPlus/></template>
      <template #icon-settings><CIconSettings/></template>
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
