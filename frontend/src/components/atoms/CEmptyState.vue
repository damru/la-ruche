<script setup lang="ts">
defineProps<{ message?: string }>()
</script>

<template>
  <div class="flex flex-col items-center justify-center py-8 gap-2 text-center text-on-surface-variant">
    <slot name="icon" />
    <p v-if="$props.message" class="text-sm">{{ $props.message }}</p>
    <slot />
  </div>
</template>
