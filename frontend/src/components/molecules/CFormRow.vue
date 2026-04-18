<script setup lang="ts">
defineProps<{ cols?: 1 | 2 }>()
</script>

<template>
  <div
    class="flex flex-col gap-4"
    :class="$props.cols === 2 ? 'sm:grid sm:grid-cols-2' : ''"
  >
    <slot></slot>
  </div>
</template>
