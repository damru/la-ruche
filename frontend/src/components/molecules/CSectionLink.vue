<script setup lang="ts">
import type { RouteLocationRaw } from 'vue-router'

defineProps<{
  to: RouteLocationRaw
}>()
</script>

<template>
  <RouterLink
    :to="to"
    class="flex items-center gap-1 text-xs text-primary font-medium no-underline whitespace-nowrap shrink-0 hover:underline"
  >
    <slot></slot> <CIconChevronRight />
  </RouterLink>
</template>
