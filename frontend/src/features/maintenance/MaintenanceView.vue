<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CMockBanner from '@/components/atoms/CMockBanner.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import { mockMaintenanceEntries, mockBuildingStats } from '@/mock/data'

const { t } = useI18n()
const { formatDate, formatCurrency } = useFormatters()

const search = ref('')

const entries = computed(() =>
  search.value
    ? mockMaintenanceEntries.filter(e => e.title.toLowerCase().includes(search.value.toLowerCase()) || (e.vendor_name ?? '').toLowerCase().includes(search.value.toLowerCase()))
    : mockMaintenanceEntries
)

const totalCost = computed(() =>
  mockMaintenanceEntries.reduce((s, e) => s + (e.cost ?? 0), 0)
)
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ mockBuildingStats.name }}
          <CIconChevronRight
            width="14"
            height="14"
          />
          <span>{{ t('maintenance.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="primary"
          size="sm"
        >
          <CIconPlus style="width:16px;height:16px" />
          <span class="btn-label">{{ t('maintenance.addEntry') }}</span>
        </CBtn>
      </template>
    </CTopbar>

    <div class="view-container">
      <CMockBanner />

      <div class="view-header">
        <div>
          <h1 class="display-md">
            {{ t('maintenance.title') }}
          </h1>
          <p
            class="body-md"
            style="color:var(--color-on-surface-variant)"
          >
            {{ t('maintenance.summary', { count: mockMaintenanceEntries.length, cost: formatCurrency(totalCost) }) }}
          </p>
        </div>
      </div>

      <div class="search-bar">
        <CIconSearch class="search-icon" />
        <input
          v-model="search"
          type="search"
          :placeholder="t('maintenance.searchPlaceholder')"
          class="search-input"
        />
      </div>

      <div class="timeline">
        <div
          v-for="entry in entries"
          :key="entry.id"
          class="timeline-item"
        >
          <div
            class="timeline-dot"
            :class="entry.source === 'automatic' ? 'timeline-dot--auto' : ''"
          ></div>
          <CCard class="timeline-card">
            <div class="entry-header">
              <div>
                <span class="title-sm">{{ entry.title }}</span>
                <CBadge
                  :variant="entry.source === 'automatic' ? 'primary' : 'default'"
                  style="margin-left:8px"
                >
                  {{ entry.source === 'automatic' ? t('maintenance.automatic') : t('maintenance.manual') }}
                </CBadge>
              </div>
              <span class="entry-date">{{ formatDate(entry.intervention_date) }}</span>
            </div>
            <p
              v-if="entry.description"
              class="body-sm entry-desc"
            >
              {{ entry.description }}
            </p>
            <div class="entry-footer">
              <span
                v-if="entry.vendor_name"
                class="vendor"
              >🔧 {{ entry.vendor_name }}</span>
              <span
                v-if="entry.cost"
                class="cost"
              >{{ formatCurrency(entry.cost) }}</span>
            </div>
          </CCard>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>

.timeline { display: flex; flex-direction: column; gap: 0; }
.timeline-item {
  display: flex; gap: 16px; align-items: stretch;
  padding-bottom: 16px;
  position: relative;
}

.timeline-dot {
  width: 14px; height: 14px; border-radius: 50%; flex-shrink: 0;
  background: var(--color-primary);
  border: 3px solid var(--color-surface);
  margin-top: 16px;
  z-index: 1;
  position: relative;
}
.timeline-dot--auto { background: var(--color-secondary); }

.timeline-item::before {
  content: '';
  position: absolute;
  left: 7px; top: 28px; bottom: 0;
  width: 2px;
  background: var(--color-outline-variant);
}
.timeline-item:last-child::before { display: none; }

.timeline-card { flex: 1; }

.entry-header { display: flex; align-items: flex-start; justify-content: space-between; gap: var(--space-3); margin-bottom: 6px; }
.entry-date { font-size: var(--text-xs); color: var(--color-on-surface-variant); flex-shrink: 0; }
.entry-desc { color: var(--color-on-surface-variant); margin-bottom: var(--space-2); }
.entry-footer { display: flex; justify-content: space-between; font-size: var(--text-sm); }
.vendor { color: var(--color-on-surface-variant); }
.cost { font-weight: var(--fw-semibold); color: var(--color-on-surface); }
</style>

