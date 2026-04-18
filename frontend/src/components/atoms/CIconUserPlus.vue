<script setup lang="ts">
defineOptions({ inheritAttrs: false })
</script>

<template>
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" v-bind="$attrs">
    <circle cx="12" cy="7" r="4"/>
    <path d="M5 21v-2a7 7 0 0114 0v2"/>
    <line x1="12" y1="11" x2="12" y2="17"/>
    <line x1="9" y1="14" x2="15" y2="14"/>
  </svg>
</template>
