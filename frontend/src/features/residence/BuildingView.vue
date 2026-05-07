<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import CLegalDescription from '@/components/atoms/CLegalDescription.vue'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CLegalNotice from '@/components/molecules/CLegalNotice.vue'
import CPageError from '@/components/molecules/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import { buildingApi } from '@/api'
import { usePageLoader } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import type { Building } from '@/types/domain'

const { t } = useI18n()

const form = ref<Partial<Building>>({})
const feedback = useFormFeedback()
const saving = ref(false)

async function loadBuilding() {
  const res = await buildingApi.get()
  form.value = res.data
}

async function saveBuilding() {
  feedback.reset()
  saving.value = true
  try {
    const res = await buildingApi.update(form.value)
    form.value = res.data
    feedback.setSuccess(t('common.saved'))
  } catch (e: unknown) {
    feedback.setError((e as { response?: { data?: { message?: string } } })?.response?.data?.message ?? t('common.error'))
  } finally {
    saving.value = false
  }
}

const { loading, error, reload } = usePageLoader(loadBuilding, t('common.loadError'))
onMounted(reload)
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ form.name ?? '...' }}
          <CIconChevronRight
            width="14"
            height="14"
          />
          <span>{{ t('nav.groups.residence') }}</span>
          <CIconChevronRight
            width="14"
            height="14"
          />
          <span>{{ t('nav.residence-info') }}</span>
        </div>
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
        <header class="view-header">
          <CEyebrow>
            {{ t('page.eyebrows.building') }}
          </CEyebrow>
          <h1 class="display-md">
            {{ t('nav.residence-info') }}
          </h1>
          <p
            class="body-md"
            style="color:var(--color-on-surface-variant)"
          >
            {{ t('settings.building.subtitle') }}
          </p>
        </header>

        <CCard>
          <h2 class="section-title">
            {{ t('settings.building.title') }}
          </h2>
          <div class="form-grid-2col">
            <div class="field">
              <label class="field-label">{{ t('settings.building.name') }}</label>
              <input
                v-model="form.name"
                class="field-input"
                @input="feedback.reset()"
              />
            </div>
            <div class="field">
              <label class="field-label">{{ t('settings.building.siret') }}</label>
              <input
                v-model="form.manager_siret"
                class="field-input"
                @input="feedback.reset()"
              />
            </div>
            <div class="field field--full">
              <label class="field-label">{{ t('addressAutocomplete.addressLine1') }}</label>
              <input
                v-model="form.address_line1"
                class="field-input"
                @input="feedback.reset()"
              />
            </div>
            <div class="field">
              <label class="field-label">{{ t('addressAutocomplete.postalCode') }}</label>
              <input
                v-model="form.postal_code"
                class="field-input"
                @input="feedback.reset()"
              />
            </div>
            <div class="field">
              <label class="field-label">{{ t('addressAutocomplete.city') }}</label>
              <input
                v-model="form.city"
                class="field-input"
                @input="feedback.reset()"
              />
            </div>
            <div class="field">
              <label class="field-label">{{ t('settings.building.sharesTotal') }}</label>
              <input
                v-model="form.shares_total"
                type="number"
                step="1"
                class="field-input"
                @input="feedback.reset()"
              />
            </div>
            <div class="field">
              <label class="field-label">{{ t('settings.building.bylawsDate') }}</label>
              <input
                v-model="form.bylaws_date"
                type="date"
                class="field-input"
                @input="feedback.reset()"
              />
            </div>
          </div>
          <div class="form-actions">
            <CBtn
              variant="primary"
              :loading="saving"
              :disabled="saving"
              @click="saveBuilding"
            >
              {{ saving ? t('common.saving') : t('settings.saveChanges') }}
            </CBtn>
            <CFormFeedback
              :status="feedback.status.value"
              :message="feedback.message.value"
            />
          </div>
        </CCard>

        <CCard>
          <h2 class="section-title">
            {{ t('settings.building.legalData') }}
          </h2>
          <CLegalNotice>
            <CLegalDescription
              title="Art. 18 loi n°65-557 du 10 juillet 1965"
              :summary="t('settings.building.legalText')"
            />
          </CLegalNotice>
          <div class="form-actions">
            <CBtn variant="secondary">
              <CIconDownload
                width="16"
                height="16"
                aria-hidden="true"
              />
              {{ t('settings.exportData') }}
            </CBtn>
            <CBtn variant="secondary">
              <CIconBook
                width="16"
                height="16"
                aria-hidden="true"
              />
              {{ t('settings.downloadBylaws') }}
            </CBtn>
          </div>
        </CCard>
      </template>
    </div>
  </div>
</template>

<style scoped>
</style>
