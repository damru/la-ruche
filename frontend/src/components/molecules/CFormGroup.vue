<script setup lang="ts">
defineProps<{ full?: boolean; hint?: string; error?: string }>()
</script>

<template>
  <div class="flex flex-col gap-1" :class="$props.full ? 'col-span-full' : ''">
    <slot />
    <p v-if="$props.hint && !$props.error" class="text-xs text-on-surface-variant">{{ $props.hint }}</p>
    <p v-if="$props.error" class="text-xs text-error">{{ $props.error }}</p>
  </div>
</template>
