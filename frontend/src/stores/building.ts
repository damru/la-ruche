import { defineStore } from 'pinia'
import { buildingApi } from '@/api'
import type { Building } from '@/types/domain'

/**
 * Caches the current building for the session. The building rarely changes,
 * so views read it from here instead of issuing a fresh `GET /building` on
 * every mount. Concurrent callers share a single in-flight request.
 */
export const useBuildingStore = defineStore('building', () => {
  const building = ref<Building | null>(null)
  let inflight: Promise<Building> | null = null

  async function fetch(): Promise<Building> {
    if (building.value) return building.value
    if (inflight) return inflight
    inflight = buildingApi.get()
      .then((res) => {
        building.value = res.data
        return res.data
      })
      .finally(() => { inflight = null })
    return inflight
  }

  /** Replace the cached building after an edit. */
  function set(value: Building): void {
    building.value = value
  }

  /** Clear the cache (e.g. on logout). */
  function reset(): void {
    building.value = null
    inflight = null
  }

  return { building, fetch, set, reset }
})
