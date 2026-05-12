<script setup lang="ts">
// Molecule — CLegalCard
// Semantic container for legal / regulatory notices.
// Wraps CCard with no shadow and a 4px left accent border (--color-error token).
// DDR: see DESIGN.md
import CCard from '@/components/molecules/CCard.vue'

withDefaults(defineProps<{
  padding?: 'xs' | 'sm' | 'md' | 'lg' | 'none'
  radius?: 'xs' | 'sm' | 'md' | 'lg' | 'none'
}>(), {
  padding: 'md',
  radius: 'lg',
})
</script>

<template>
  <CCard
    class="border-l-4 border-error text-xs"
    :padding="padding"
    :radius="radius"
    :shadow="false"
  >
    <slot></slot>
  </CCard>
</template>

<style scoped>
</style>
