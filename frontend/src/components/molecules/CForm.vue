<script setup lang="ts">
import type { FeedbackStatus } from '@/composables/useFormFeedback'
import CFormActions from '@/components/molecules/CFormActions.vue'

withDefaults(defineProps<{
  status?: FeedbackStatus
  message?: string
  cols?: number
}>(), {
  status: 'idle',
  cols: 1,
})

const emit = defineEmits<{ submit: [] }>()
</script>

<template>
  <form
    class="flex flex-col gap-4"
    :class="$props.cols === 2 ? 'sm:grid sm:grid-cols-2' : ''"
    @submit.prevent="emit('submit')"
  >
    <slot></slot>
    <CFormActions
      v-if="$slots.actions"
      :status="status"
      :message="message"
      class="col-span-full"
    >
      <slot name="actions"></slot>
    </CFormActions>
  </form>
</template>