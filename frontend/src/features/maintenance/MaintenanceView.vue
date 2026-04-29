<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback, SUCCESS_AUTO_CLEAR_MS } from '@/composables/useFormFeedback'
import { useConfirm } from '@/composables/useConfirm'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/molecules/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import { buildingApi, maintenanceApi } from '@/api'
import type { MaintenanceEntry } from '@/types/domain'

const { t } = useI18n()
const { formatDate, formatCurrency } = useFormatters()
const { confirm } = useConfirm()

// ── State ─────────────────────────────────────────────────────────
const buildingName = ref('')
const allEntries = ref<MaintenanceEntry[]>([])
const search = ref('')

// ── Entry form modal ───────────────────────────────────────────────
const showEntryModal = ref(false)
const entryFeedback = useFormFeedback()
const savingEntry = ref(false)

const defaultForm = () => ({
  title: '',
  description: '',
  intervention_date: new Date().toISOString().slice(0, 10),
  cost: '',
  vendor_name: '',
})
const entryForm = ref(defaultForm())

// ── Load data ─────────────────────────────────────────────────────
async function loadData() {
  const [buildingRes, entriesRes] = await Promise.all([
    buildingApi.get(),
    maintenanceApi.list(),
  ])
  buildingName.value = buildingRes.data.name
  allEntries.value = entriesRes.data
}

const { loading, error, reload } = usePageLoader(loadData, t('common.loadError'))
onMounted(reload)

// ── Derived ───────────────────────────────────────────────────────
const entries = computed(() => {
  const q = search.value.toLowerCase()
  if (!q) return allEntries.value
  return allEntries.value.filter(e =>
    e.title.toLowerCase().includes(q) ||
    (e.vendor_name ?? '').toLowerCase().includes(q),
  )
})

const totalCost = computed(() =>
  allEntries.value.reduce((s, e) => s + (e.cost ?? 0), 0),
)

// ── Entry modal actions ────────────────────────────────────────────
function openCreateModal() {
  entryForm.value = defaultForm()
  entryFeedback.reset()
  showEntryModal.value = true
}

async function saveEntry() {
  if (!entryForm.value.title.trim()) return
  savingEntry.value = true
  entryFeedback.reset()
  try {
    await maintenanceApi.create({
      title: entryForm.value.title,
      description: entryForm.value.description || undefined,
      intervention_date: entryForm.value.intervention_date,
      cost: entryForm.value.cost ? Number(entryForm.value.cost) : undefined,
      vendor_name: entryForm.value.vendor_name || undefined,
    })
    entryFeedback.setSuccess(t('common.saved'))
    setTimeout(() => {
      showEntryModal.value = false
      reload()
    }, SUCCESS_AUTO_CLEAR_MS)
  } catch (err) {
    entryFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    savingEntry.value = false
  }
}

async function deleteEntry(entry: MaintenanceEntry) {
  if (entry.source === 'automatic') {
    alert(t('maintenance.onlyManualDelete'))
    return
  }
  const ok = await confirm({
    title: t('maintenance.confirmDeleteTitle'),
    message: t('maintenance.confirmDeleteMessage', { title: entry.title }),
    confirmLabel: t('common.delete'),
    variant: 'danger',
  })
  if (!ok) return
  try {
    await maintenanceApi.delete(entry.id)
    await reload()
  } catch (err) {
    alert(toErrorMessage(err, t('common.error')))
  }
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName }}
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
          @click="openCreateModal"
        >
          <CIconPlus style="width:16px;height:16px" />
          <span class="btn-label">{{ t('maintenance.addEntry') }}</span>
        </CBtn>
      </template>
    </CTopbar>

    <div class="view-container">
      <CPageError
        v-if="error"
        :message="error"
        @retry="reload"
      />
      <CPageLoader v-else-if="loading" />
      <template v-else>
        <div class="view-header">
          <div>
            <h1 class="display-md">
              {{ t('maintenance.title') }}
            </h1>
            <p
              class="body-md"
              style="color:var(--color-on-surface-variant)"
            >
              {{ t('maintenance.summary', { count: allEntries.length, cost: formatCurrency(totalCost) }) }}
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
                <div class="entry-header-right">
                  <span class="entry-date">{{ formatDate(entry.intervention_date) }}</span>
                  <button
                    v-if="entry.source === 'manual'"
                    class="icon-btn icon-btn--danger"
                    :title="t('maintenance.deleteEntry')"
                    @click="deleteEntry(entry)"
                  >
                    <CIconTrash
                      width="14"
                      height="14"
                    />
                  </button>
                </div>
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
                >
                  <CIconWrench
                    width="13"
                    height="13"
                    style="margin-right:4px"
                  />{{ entry.vendor_name }}
                </span>
                <span
                  v-if="entry.cost"
                  class="cost"
                >{{ formatCurrency(entry.cost) }}</span>
              </div>
            </CCard>
          </div>

          <div
            v-if="!entries.length"
            class="empty-state"
          >
            {{ t('common.noData') }}
          </div>
        </div>
      </template>
    </div>

    <!-- Create entry modal -->
    <CModal
      :open="showEntryModal"
      :title="t('maintenance.addEntry')"
      size="md"
      @update:open="showEntryModal = $event"
    >
      <form
        class="form-grid"
        @submit.prevent="saveEntry"
      >
        <div class="form-field">
          <label class="form-label">{{ t('maintenance.fields.title') }} *</label>
          <input
            v-model="entryForm.title"
            type="text"
            class="form-input"
            required
          />
        </div>
        <div class="form-field">
          <label class="form-label">{{ t('maintenance.fields.description') }}</label>
          <textarea
            v-model="entryForm.description"
            class="form-input"
            rows="2"
          ></textarea>
        </div>
        <div class="form-field">
          <label class="form-label">{{ t('maintenance.fields.interventionDate') }} *</label>
          <input
            v-model="entryForm.intervention_date"
            type="date"
            class="form-input"
            required
          />
        </div>
        <div class="form-row">
          <div class="form-field">
            <label class="form-label">{{ t('maintenance.fields.cost') }}</label>
            <input
              v-model="entryForm.cost"
              type="number"
              step="0.01"
              min="0"
              class="form-input"
            />
          </div>
          <div class="form-field">
            <label class="form-label">{{ t('maintenance.fields.vendorName') }}</label>
            <input
              v-model="entryForm.vendor_name"
              type="text"
              class="form-input"
            />
          </div>
        </div>
        <CFormFeedback
          :status="entryFeedback.status.value"
          :message="entryFeedback.message.value"
        />
      </form>
      <template #footer>
        <CBtn
          variant="ghost"
          @click="showEntryModal = false"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="savingEntry"
          @click="saveEntry"
        >
          {{ t('common.save') }}
        </CBtn>
      </template>
    </CModal>
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
.entry-header-right { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }
.entry-date { font-size: var(--text-xs); color: var(--color-on-surface-variant); }
.entry-desc { color: var(--color-on-surface-variant); margin-bottom: var(--space-2); }
.entry-footer { display: flex; justify-content: space-between; font-size: var(--text-sm); align-items: center; }
.vendor { color: var(--color-on-surface-variant); display: flex; align-items: center; }
.cost { font-weight: var(--fw-semibold); color: var(--color-on-surface); }

.form-row { display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-3); }

.empty-state { padding: var(--space-8); text-align: center; font-size: var(--text-md); color: var(--color-on-surface-variant); }
</style>

