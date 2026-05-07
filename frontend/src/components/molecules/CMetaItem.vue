<script setup lang="ts">
withDefaults(defineProps<{
  as?: 'div' | 'span' | 'a'
  variant?: 'footer' | 'sub'
}>(), {
  as: 'span',
  variant: 'footer',
})
</script>

<template>
  <component
    :is="as"
    :class="['c-meta-item', `c-meta-item--${variant}`]"
  >
    <span class="c-meta-item__icon">
      <slot name="icon"></slot>
    </span>
    <span class="c-meta-item__content">
      <slot></slot>
    </span>
  </component>
</template>

<style scoped>
.c-meta-item {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  width: fit-content;
  color: var(--color-on-surface-variant);
}

.c-meta-item--footer {
  font-size: var(--text-2xs);
  line-height: 1.4;
}

.c-meta-item--sub {
  font-size: var(--text-sm);
  line-height: 1.5;
}

.c-meta-item__icon,
.c-meta-item__content {
  display: inline-flex;
  align-items: center;
}

.c-meta-item :deep(svg) {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}
</style>