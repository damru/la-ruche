import { ref } from 'vue'
import i18n from '@/i18n'

const { t } = i18n.global

export function toErrorMessage(error: unknown, fallbackMessage = 'common.error'): string {
  const apiMessage = (error as { response?: { data?: { message?: string } } })?.response?.data?.message
  if (typeof apiMessage === 'string' && apiMessage.trim().length > 0) {
    return t(apiMessage)
  }

  if (error instanceof Error && error.message.trim().length > 0) {
    return error.message
  }

  return t(fallbackMessage)
}

export function usePageLoader(loader: () => Promise<void>, fallbackMessage = 'common.loadError') {
  const loading = ref(true)
  const error = ref<string | null>(null)

  async function reload() {
    loading.value = true
    error.value = null

    try {
      await loader()
    } catch (err) {
      error.value = toErrorMessage(err, fallbackMessage)
    } finally {
      loading.value = false
    }
  }

  return {
    loading,
    error,
    reload,
  }
}