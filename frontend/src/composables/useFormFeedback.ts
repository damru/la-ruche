import { ref } from 'vue'
import { AUTO_CLOSE_DELAY_DEFAULT_SEC } from '@/stores/ui'

export type FeedbackStatus = 'idle' | 'success' | 'error'

// Default inline success-message clear delay (ms) — fixed, unrelated to the
// modal auto-close delay (EP-019-S09 keeps this timer at its default).
export const SUCCESS_AUTO_CLEAR_MS = AUTO_CLOSE_DELAY_DEFAULT_SEC * 1000

export function useFormFeedback() {
  const status  = ref<FeedbackStatus>('idle')
  const message = ref('')
  let timer: ReturnType<typeof setTimeout> | undefined

  function setSuccess(msg = '') {
    clearTimeout(timer)
    status.value  = 'success'
    message.value = msg
    timer = setTimeout(() => { status.value = 'idle'; message.value = '' }, SUCCESS_AUTO_CLEAR_MS)
  }

  function setError(msg: string) {
    clearTimeout(timer)
    status.value  = 'error'
    message.value = msg
  }

  function reset() {
    clearTimeout(timer)
    status.value  = 'idle'
    message.value = ''
  }

  return { status, message, setSuccess, setError, reset }
}
