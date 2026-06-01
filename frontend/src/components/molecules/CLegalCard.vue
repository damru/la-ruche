<script setup lang="ts">
import CCard from '@/components/molecules/CCard.vue'
</script>

<template>
  <CCard
    class="rounded-none! border-l-2 border-primary opacity-75 text-xs bg-transparent!"
    :level="4"
    :shadow="false"
  >
    <slot></slot>
  </CCard>
</template>
