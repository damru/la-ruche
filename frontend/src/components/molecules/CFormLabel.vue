<script setup lang="ts">
defineProps<{ for?: string; required?: boolean }>()
</script>

<template>
  <label :for="$props.for" class="text-sm font-medium text-on-surface-variant">
    <slot />
    <span v-if="$props.required" class="text-error ml-0.5">*</span>
  </label>
</template>
