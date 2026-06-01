<script setup lang="ts">
import type { FeedbackStatus } from '@/composables/useFormFeedback'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'

defineProps<{
  status: FeedbackStatus
  message?: string
}>()
</script>

<template>
  <div class="flex flex-col w-full">
    <div class="flex justify-end items-center gap-2">
      <slot></slot>
    </div>
    <CFormFeedback
      :status="status"
      :message="message"
    />
  </div>
</template>