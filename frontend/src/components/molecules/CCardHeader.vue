<script setup lang="ts">
withDefaults(defineProps<{
  withIcon?: boolean
}>(), {
  withIcon: false,
})
</script>

<template>
  <div class="flex flex-col gap-2 min-w-0">
    <div
      v-if="$slots.chips"
      class="flex flex-wrap items-center gap-x-2 gap-y-1 min-w-0 text-xs"
    >
      <slot name="chips"></slot>
    </div>
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
    </div>
  </div>
</template>
