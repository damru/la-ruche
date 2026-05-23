<script setup lang="ts">
import CCard from '@/components/molecules/CCard.vue'

withDefaults(defineProps<{
  padding?: 'xs' | 'sm' | 'md' | 'lg' | 'none'
}>(), {
  padding: 'xs',
})
</script>

<template>
  <CCard
    class="border-l-2 border-primary opacity-75 text-xs bg-transparent!"
    :padding="padding"
    radius="none"
    :shadow="false"
  >
    <slot></slot>
  </CCard>
</template>

<style scoped>
</style>
