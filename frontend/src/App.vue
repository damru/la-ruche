<script setup lang="ts">
import { onMounted } from 'vue'
import CConfirmDialog from '@/components/organisms/CConfirmDialog.vue'
import { useUiStore } from '@/stores/ui'

const ui = useUiStore()

onMounted(() => {
  void ui.loadRuntimeStatus()
})
</script>

<template>
  <RouterView />
  <CConfirmDialog />
</template>
