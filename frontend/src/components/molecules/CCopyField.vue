<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import CBtn from '@/components/atoms/CBtn.vue'
import CIconCopy from '@/components/atoms/CIconCopy.svg'

const props = withDefaults(defineProps<{
  value: string
  label?: string
  hint?: string
  copyTitle?: string
  copiedTitle?: string
}>(), {
  label: '',
  hint: '',
  copyTitle: '',
  copiedTitle: '',
})

const { t } = useI18n()
const copied = ref(false)

async function copy() {
  if (!props.value) return
  await navigator.clipboard.writeText(props.value)
  copied.value = true
  setTimeout(() => { copied.value = false }, 2000)
}
</script>

<template>
  <div class="c-copy-field">
    <p
      v-if="label"
      class="c-copy-field__label label-sm"
    >
      {{ label }}
    </p>
    <div class="c-copy-field__row">
      <input
        :value="value"
        readonly
        class="c-copy-field__input"
        @click="($event.target as HTMLInputElement).select()"
      />
      <CBtn
        variant="secondary"
        size="sm"
        :icon="CIconCopy"
        :title="copied ? (copiedTitle || t('common.copy')) : (copyTitle || t('common.copy'))"
        @click.prevent="copy"
      />
    </div>
    <p
      v-if="hint"
      class="c-copy-field__hint label-sm tone--muted"
    >
      {{ hint }}
    </p>
  </div>
</template>

<style scoped>
.c-copy-field {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  padding: var(--space-3);
  background: var(--color-surface-container);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-outline-variant);
}
.c-copy-field__label {
  font-weight: var(--fw-medium);
  color: var(--color-on-surface-variant);
}
.c-copy-field__row {
  display: flex;
  gap: var(--space-2);
  align-items: center;
}
.c-copy-field__input {
  flex: 1;
  font-size: var(--text-sm);
  font-family: var(--font-mono, monospace);
  color: var(--color-on-surface);
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  border-radius: var(--radius-sm);
  padding: var(--space-1) var(--space-2);
  min-width: 0;
  outline: none;
}
.c-copy-field__hint {
  margin-top: var(--space-1);
}
</style>
