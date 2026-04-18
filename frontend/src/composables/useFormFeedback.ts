import { ref } from 'vue'

export type FeedbackStatus = 'idle' | 'success' | 'error'

export const SUCCESS_AUTO_CLEAR_MS = 3000

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
