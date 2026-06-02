<script setup lang="ts">
import type { FeedbackStatus } from '@/composables/useFormFeedback'
import CFormActions from '@/components/molecules/CFormActions.vue'

withDefaults(defineProps<{
  status?: FeedbackStatus
  message?: string
}>(), {
  status: 'idle',
})

const emit = defineEmits<{ submit: [] }>()
</script>

<template>
  <form
    class="flex flex-col gap-4"
    @submit.prevent="emit('submit')"
  >
    <slot></slot>
    <CFormActions
      v-if="$slots.actions"
      :status="status"
      :message="message"
    >
      <slot name="actions"></slot>
    </CFormActions>
  </form>
</template>