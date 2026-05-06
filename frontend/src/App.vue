<script setup lang="ts">
import { onMounted } from 'vue'
import CMockBanner from '@/components/atoms/CMockBanner.vue'
import CConfirmDialog from '@/components/molecules/CConfirmDialog.vue'
import { useUiStore } from '@/stores/ui'

const ui = useUiStore()

onMounted(() => {
  void ui.loadRuntimeStatus()
})
</script>

<template>
  <CMockBanner
    v-if="ui.demoMode"
    variant="demo"
    class="app-root__banner"
  />
  <RouterView />
  <CConfirmDialog />
</template>

<style scoped>
.app-root__banner {
  position: relative;
  z-index: var(--z-nav);
}
</style>
