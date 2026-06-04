<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    name?: string | null
    size?: 'sm' | 'md' | 'lg'
  }>(),
  { 
    name: undefined,
    size: 'md' 
  },
)

function nameToHue(name: string): number {
  const normalized = name.trim().replace(/\s+/g, ' ').toLowerCase()
  let hash = 0
  for (let i = 0; i < normalized.length; i++) {
    hash = normalized.charCodeAt(i) + ((hash << 5) - hash)
    hash |= 0
  }
  return Math.abs(hash) % 360
}

function initials(name?: string | null): string {
  if (!name?.trim()) return '??'
  const parts = name.trim().split(/\s+/).filter(Boolean)
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase()
  return `${parts[0][0] ?? ''}${parts[parts.length - 1][0] ?? ''}`.toUpperCase()
}

const hueStyle = computed(() =>
  props.name?.trim()
    ? { '--avatar-hue': String(nameToHue(props.name)) }
    : {},
)
</script>

<template>
  <div
    class="avatar"
    :class="`avatar--${size}`"
    :style="hueStyle"
    aria-hidden="true"
  >
    {{ initials(name) }}
  </div>
</template>

<style scoped>
.avatar {
  --_l-bg: 0.90;
  --_c-bg: 0.07;
  --_l-fg: 0.30;
  --_c-fg: 0.12;

  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border-radius: var(--radius-full);
  font-family: var(--font-display);
  font-weight: var(--fw-bold);
  background: oklch(var(--_l-bg) var(--_c-bg) var(--avatar-hue, 260));
  color: oklch(var(--_l-fg) var(--_c-fg) var(--avatar-hue, 260));
  user-select: none;
}

[data-theme="dark"] .avatar {
  --_l-bg: 0.28;
  --_c-bg: 0.07;
  --_l-fg: 0.88;
  --_c-fg: 0.07;
}

.avatar--sm {
  width: 28px;
  height: 28px;
  font-size: var(--text-2xs);
}

.avatar--md {
  width: 36px;
  height: 36px;
  font-size: var(--text-xs);
}

.avatar--lg {
  width: 42px;
  height: 42px;
  font-size: var(--text-sm);
}
</style>
