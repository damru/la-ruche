<script setup lang="ts">
// MyChargesView — Co-owner entry point for charge statements.
// Loads the units owned by the current user and either redirects directly
// to the single unit's charges page (most common case) or renders a
// selection list for co-owners who own multiple units.
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import CCard from '@/components/molecules/CCard.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CEyebrow from '@/components/atoms/CEyebrow.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import { unitsApi } from '@/api'
import { usePageLoader } from '@/composables/usePageLoader'
import type { Unit } from '@/types/domain'

const { t } = useI18n()
const router = useRouter()
const auth = useAuthStore()

const ownedUnits = ref<Unit[]>([])

async function loadOwnedUnits() {
  const res = await unitsApi.list()
  ownedUnits.value = res.data.filter(u => u.current_owner?.id === auth.user?.user_id)

  if (ownedUnits.value.length === 1) {
    router.replace(`/residents/${ownedUnits.value[0].id}/charges`)
  }
}

const { loading, error, reload } = usePageLoader(loadOwnedUnits)
onMounted(reload)
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          <span>{{ t('nav.myCharges') }}</span>
        </div>
      </template>
    </CTopbar>

    <div class="flex flex-col p-10 max-w-[1200px] gap-6 max-sm:p-5">
      <CPageError
        v-if="error"
        :message="error"
        @retry="reload"
      />
      <CPageLoader v-else-if="loading" />
      <template v-else>
        <header class="flex flex-col items-start gap-2">
          <CEyebrow>
            {{ t('page.eyebrows.myCharges') }}
          </CEyebrow>
          <CHeading type="header">
            {{ t('nav.myCharges') }}
          </CHeading>
        </header>

        <!-- Multi-unit selection (co-owner with several units) -->
        <div
          v-if="ownedUnits.length > 1"
          class="grid grid-cols-[repeat(auto-fill,minmax(320px,1fr))] gap-4"
        >
          <CCard
            v-for="unit in ownedUnits"
            :key="unit.id"
            class="unit-card"
            @click="router.push(`/residents/${unit.id}/charges`)"
          >
            <div class="unit-card__inner">
              <span class="unit-card__number title-sm">{{ t('common.lot') }} {{ unit.unit_number }}</span>
              <span class="unit-card__type body-xs dim">{{ unit.description }}</span>
            </div>
            <CBtn
              variant="ghost"
              size="sm"
            >
              {{ t('units.charges.title') }}
            </CBtn>
          </CCard>
        </div>

        <!-- No owned unit -->
        <p
          v-else-if="ownedUnits.length === 0 && !error"
          class="py-8 text-center text-md text-on-surface-variant"
        >
          {{ t('units.charges.noUnit') }}
        </p>

        <!-- Single unit: spinner while redirect is in flight -->
        <p
          v-else
          class="dim"
          style="padding: var(--space-6) 0"
        >
          {{ t('common.loading') }}
        </p>
      </template>
    </div>
  </div>
</template>

<style scoped>
/* cards-grid is global — defined in styles/tables.css */

.unit-card {
  cursor: pointer;
}

.unit-card :deep(.c-card) {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  transition: background var(--transition-fast);
}

.unit-card :deep(.c-card):hover {
  background: var(--color-surface-container-low);
}

.unit-card__inner {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.dim { color: var(--color-on-surface-variant); }
.empty-state { text-align: center; color: var(--color-on-surface-variant); padding: var(--space-8) 0; }
</style>
