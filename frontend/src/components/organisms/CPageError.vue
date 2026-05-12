<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'

const props = withDefaults(defineProps<{
  message: string
  title?: string
}>(), {
  title: undefined,
})

const emit = defineEmits<{
  retry: []
}>()

const { t } = useI18n()
</script>

<template>
  <CCard class="c-page-error">
    <div
      class="c-page-error__icon tone--err"
      aria-hidden="true"
    >
      <CIconXCircle
        width="18"
        height="18"
      />
    </div>
    <div class="c-page-error__body">
      <h2 class="c-page-error__title">
        {{ props.title ?? t('common.loadError') }}
      </h2>
      <p class="c-page-error__message">
        {{ props.message }}
      </p>
    </div>
    <CBtn
      variant="secondary"
      @click="emit('retry')"
    >
      {{ t('common.retry') }}
    </CBtn>
  </CCard>
</template>

<style scoped>
.c-page-error {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-5);
}

.c-page-error__icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--tone-bg, var(--color-error-container));
  color: var(--tone-fg, var(--color-error));
  flex-shrink: 0;
}

.c-page-error__body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.c-page-error__title {
  margin: 0;
  font-size: var(--text-lg);
  font-weight: var(--fw-semibold);
  color: var(--color-on-surface);
}

.c-page-error__message {
  margin: 0;
  color: var(--color-on-surface-variant);
}

@media (max-width: 640px) {
  .c-page-error {
    flex-direction: column;
    align-items: stretch;
  }

  .c-page-error__icon {
    width: 36px;
    height: 36px;
  }
}
</style>