<script setup lang="ts">
withDefaults(defineProps<{
  as?: 'div' | 'span' | 'a' | 'button'
  variant?: 'footer' | 'sub'
}>(), {
  as: 'span',
  variant: 'footer',
})
</script>

<template>
  <component
    :is="as"
    :class="[  
      'inline-flex', 
      'items-center', 
      'gap-1',
      'color-on-surface-variant',
      `${variant === 'footer' ? '[&_svg]:w-(--icon-xs) [&_svg]:h-(--icon-2xs) text-xs' : '[&_svg]:w-(--icon-sm) [&_svg]:h-(--icon-sm) text-sm'}`, 
    ]"
  >
    <slot v-if="$slots.icon" name="icon"></slot>
    <p>
      <slot></slot>
    </p>
  </component>
</template>

<style scoped>
* {
  min-height: unset;
  min-width: 0;
}
.c-meta-item {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  width: fit-content;
  color: var(--color-on-surface-variant);
}

</style>