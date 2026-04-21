import { ref } from 'vue'

export function toErrorMessage(error: unknown, fallbackMessage: string): string {
  const apiMessage = (error as { response?: { data?: { message?: string } } })?.response?.data?.message
  if (typeof apiMessage === 'string' && apiMessage.trim().length > 0) {
    return apiMessage
  }

  if (error instanceof Error && error.message.trim().length > 0) {
    return error.message
  }

  return fallbackMessage
}

export function usePageLoader(loader: () => Promise<void>, fallbackMessage: string) {
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