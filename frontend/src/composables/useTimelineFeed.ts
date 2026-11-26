import { ref } from 'vue'
import type { TimelineEvent } from '@/types/domain'

export function useTimelineFeed(load: () => Promise<TimelineEvent[]>) {
  const events = ref<TimelineEvent[]>([])
  const loading = ref(false)
  const error = ref(false)

  async function reload(): Promise<void> {
    loading.value = true
    error.value = false
    try {
      events.value = await load()
    } catch {
      error.value = true
    } finally {
      loading.value = false
    }
  }

  return { events, loading, error, reload }
}
