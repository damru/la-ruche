import { ref } from 'vue'
import type { TimelineEvent } from '@/types/domain'

export function useTimelineFeed(load: () => Promise<TimelineEvent[]>) {
  const events = ref<TimelineEvent[]>([])
  const loading = ref(false)
  const error = ref(false)

  let latest = 0

  async function reload(): Promise<void> {
    const seq = ++latest
    loading.value = true
    error.value = false
    try {
      const data = await load()
      if (seq === latest) events.value = data
    } catch {
      if (seq === latest) error.value = true
    } finally {
      if (seq === latest) loading.value = false
    }
  }

  return { events, loading, error, reload }
}
