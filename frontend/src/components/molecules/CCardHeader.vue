<script setup lang="ts">
withDefaults(defineProps<{
  withIcon?: boolean
}>(), {
  withIcon: false,
})
</script>

<template>
  <div class="flex items-center gap-3 min-w-0">
    <div
      v-if="withIcon"
      class="shrink-0"
    >
      <slot name="icon"></slot>
    </div>
    <div class="flex-1 min-w-0">
      <slot></slot>
    </div>
    <div
      v-if="$slots.chips"
      class="flex flex-wrap justify-end gap-x-2 gap-y-1"
    >
      <slot name="chips"></slot>
    </div>
  </div>
</template>
