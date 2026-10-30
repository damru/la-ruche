<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader } from '@/composables/usePageLoader'
import CBadge from '@/components/atoms/CBadge.vue'
import CIconTile from '@/components/atoms/CIconTile.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CEyebrow from '@/components/atoms/CEyebrow.vue'
import CCard from '@/components/molecules/CCard.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CIconArrowLeft from '@/components/atoms/CIconArrowLeft.svg'
import CIconBolt from '@/components/atoms/CIconBolt.svg'
import CIconChevronRight from '@/components/atoms/CIconChevronRight.svg'
import CIconDroplet from '@/components/atoms/CIconDroplet.svg'
import CIconLayers from '@/components/atoms/CIconLayers.svg'
import CIconLeaf from '@/components/atoms/CIconLeaf.svg'
import CIconSettings from '@/components/atoms/CIconSettings.svg'
import CIconShield from '@/components/atoms/CIconShield.svg'
import CIconSparkles from '@/components/atoms/CIconSparkles.svg'
import CIconWrench from '@/components/atoms/CIconWrench.svg'
import { contractorsApi, tasksApi } from '@/api'
import { useBuildingStore } from '@/stores/building'
import type {Contractor, Task} from '@/types/domain'
import { TASK_PRIORITY_HIGH, TASK_PRIORITY_LOW, TASK_PRIORITY_NORMAL, TASK_PRIORITY_URGENT, TASK_STATUS_AWAITING_VALIDATION, TASK_STATUS_AWAITING_VENDOR, TASK_STATUS_CLOSED, TASK_STATUS_IN_PROGRESS, TASK_STATUS_OPEN } from '@/utils/constants'

const { t } = useI18n()
const { formatRelative } = useFormatters()
const route = useRoute()
const router = useRouter()

const contractorId = computed(() => route.params.id as string)
const buildingName = ref('')
const contractor = ref<Contractor | null>(null)
const assignedTasks = ref<Task[]>([])

// ── Load data ─────────────────────────────────────────────────────
async function loadData() {
  const [building, contractorRes, tasksRes] = await Promise.all([
    useBuildingStore().fetch(),
    contractorsApi.get(contractorId.value),
    tasksApi.list({ contractor_id: contractorId.value }),
  ])
  buildingName.value = building.name
  contractor.value = contractorRes.data
  assignedTasks.value = tasksRes.data.items
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

// ── Domain icon helpers (mirrors ContractorsView) ─────────────────
function domainIcon(domains: string[]) {
  const icons: Record<string, unknown> = {
    maintenance: CIconWrench,
    insurance: CIconShield,
    service: CIconSettings,
    energy: CIconBolt,
    water: CIconDroplet,
    cleaning: CIconSparkles,
    green_spaces: CIconLeaf,
    other: CIconLayers,
  }
  return icons[domains[0] ?? ''] ?? CIconLayers
}

function domainTone(domains: string[]): string {
  const tones: Record<string, string> = {
    maintenance: 'tone--warn',
    insurance: 'tone--err',
    service: 'tone--default',
    energy: 'tone--accent',
    water: 'tone--default',
    cleaning: 'tone--valid',
    green_spaces: 'tone--valid',
    other: 'tone--default',
  }
  return tones[domains[0] ?? ''] ?? 'tone--default'
}

// ── Task status helpers ───────────────────────────────────────────
function statusVariant(s: string): 'valid' | 'open' | 'pending' | 'closed' | 'default' {
  if (s === TASK_STATUS_OPEN) return 'valid'
  if (s === TASK_STATUS_IN_PROGRESS) return 'open'
  if (s === TASK_STATUS_AWAITING_VENDOR) return 'default'
  if (s === TASK_STATUS_AWAITING_VALIDATION) return 'pending'
  if (s === TASK_STATUS_CLOSED) return 'closed'
  return 'default'
}

function priorityVariant(p: string): 'err' | 'pending' | 'primary' | 'valid' | 'default' {
  if (p === TASK_PRIORITY_URGENT) return 'err'
  if (p === TASK_PRIORITY_HIGH) return 'pending'
  if (p === TASK_PRIORITY_NORMAL) return 'primary'
  if (p === TASK_PRIORITY_LOW) return 'valid'
  return 'default'
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName }}
          <CIconChevronRight />
          <RouterLink
            :to="{ name: 'contractors' }"
            class="topbar-breadcrumb-link"
          >
            {{ t('contractors.title') }}
          </RouterLink>
          <CIconChevronRight />
          <span>{{ contractor?.name ?? '…' }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="ghost"
          size="sm"
          :icon="CIconArrowLeft"
          :title="t('common.back')"
          @click="router.push({ name: 'contractors' })"
        />
      </template>
    </CTopbar>

    <div class="view-container">
      <CPageError
        v-if="error"
        :message="error"
        @retry="reload"
      />
      <CPageLoader v-else-if="loading" />
      <template v-else-if="contractor">
        <!-- Contractor detail card -->
        <CCard class="contractor-detail-card">
          <div class="contractor-header">
            <CIconTile
              size="lg"
              :class="domainTone(contractor.domains)"
            >
              <component
                :is="domainIcon(contractor.domains)"
                aria-hidden="true"
              />
            </CIconTile>
            <div class="contractor-header-info">
              <CEyebrow>
                {{ t('page.eyebrows.contractorDetail') }}
              </CEyebrow>
              <CHeading
                type="header"
                class="mb-2"
              >
                {{ contractor.name }}
              </CHeading>
              <div class="contractor-domains">
                <CBadge
                  v-for="d in contractor.domains"
                  :key="d"
                  variant="default"
                >
                  {{ t(`contractors.domains.${d}`, d) }}
                </CBadge>
                <span
                  v-if="!contractor.domains.length"
                  class="body-xs text-on-surface-variant"
                >
                  {{ t('contractors.noDomain') }}
                </span>
              </div>
            </div>
          </div>

          <div
            v-if="contractor.email || contractor.phone"
            class="contractor-contacts"
          >
            <a
              v-if="contractor.email"
              :href="`mailto:${contractor.email}`"
              class="contact-link"
            >{{ contractor.email }}</a>
            <a
              v-if="contractor.phone"
              :href="`tel:${contractor.phone}`"
              class="contact-link"
            >{{ contractor.phone }}</a>
          </div>

          <p
            v-if="contractor.notes"
            class="contractor-notes body-xs"
          >
            {{ contractor.notes }}
          </p>
        </CCard>

        <!-- Assigned tasks section -->
        <CHeading>
          {{ t('contractors.assignedTasks', { count: assignedTasks.length }) }}
        </CHeading>

        <div
          v-if="assignedTasks.length === 0"
          class="py-8 text-center text-md text-on-surface-variant"
        >
          {{ t('contractors.noAssignedTasks') }}
        </div>

        <div
          v-else
          class="task-list"
        >
          <RouterLink
            v-for="task in assignedTasks"
            :key="task.id"
            :to="{ name: 'task-detail', params: { id: task.id } }"
            class="task-item-link"
          >
            <CCard
              padding="sm"
              class="task-item"
            >
              <div class="task-item-row">
                <div class="task-item-badges">
                  <CBadge :variant="priorityVariant(task.priority)">
                    {{ t(`tasks.priorities.${task.priority}`) }}
                  </CBadge>
                  <CBadge
                    :variant="statusVariant(task.status)"
                    dot
                  >
                    {{ t(`tasks.statuses.${task.status}`) }}
                  </CBadge>
                </div>
                <span class="body-xs task-item-meta">{{ formatRelative(task.created_at) }}</span>
              </div>
              <span class="title-sm task-item-title">{{ task.title }}</span>
              <p
                v-if="task.description"
                class="body-xs task-item-desc"
              >
                {{ task.description }}
              </p>
            </CCard>
          </RouterLink>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
/* ── Contractor header ───────────────────────────────────────────── */
.contractor-detail-card {
  margin-bottom: var(--space-6);
}

.contractor-header {
  display: flex;
  align-items: flex-start;
  gap: var(--space-4);
  margin-bottom: var(--space-4);
}


.contractor-header-info {
  flex: 1;
  min-width: 0;
}

.contractor-domains {
  display: flex;
  gap: var(--space-1);
  flex-wrap: wrap;
}

.contractor-contacts {
  display: flex;
  gap: var(--space-4);
  flex-wrap: wrap;
  margin-bottom: var(--space-3);
}

.contact-link {
  color: var(--color-primary);
  font-size: var(--text-sm);
  text-decoration: none;
}
.contact-link:hover { text-decoration: underline; }

.contractor-notes {
  color: var(--color-on-surface-variant);
  margin: 0;
  white-space: pre-line;
}

/* ── Task list ───────────────────────────────────────────────────── */
.task-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.task-item-link {
  text-decoration: none;
  color: inherit;
}

.task-item {
  transition: box-shadow 0.15s;
}
.task-item:hover {
  box-shadow: var(--shadow-sm);
}

.task-item-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-2);
  margin-bottom: var(--space-1);
}

.task-item-badges {
  display: flex;
  gap: var(--space-1);
  flex-wrap: wrap;
}

.task-item-meta {
  color: var(--color-on-surface-variant);
  white-space: nowrap;
}

.task-item-title { display: block; }

.task-item-desc {
  color: var(--color-on-surface-variant);
  margin: var(--space-1) 0 0;
}
</style>
