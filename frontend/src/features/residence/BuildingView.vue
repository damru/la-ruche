<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CLegalDescription from '@/components/atoms/CLegalDescription.vue'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CLegalNotice from '@/components/molecules/CLegalNotice.vue'
import CLegalCard from '@/components/molecules/CLegalCard.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CFormRow from '@/components/molecules/CFormRow.vue'
import CIconBook from '@/components/atoms/CIconBook.svg'
import CIconDownload from '@/components/atoms/CIconDownload.svg'
import { buildingApi } from '@/api'
import { usePageLoader } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import type { Building } from '@/types/domain'

const { t } = useI18n()

const form = ref<Partial<Building>>({})
const feedback = useFormFeedback()
const saving = ref(false)

watch(form, () => feedback.reset(), { deep: true })

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

const { loading, error, reload } = usePageLoader(loadBuilding)
onMounted(reload)
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ form.name ?? '...' }}
          <CIconChevronRight />
          <span>{{ t('nav.groups.residence') }}</span>
          <CIconChevronRight />
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
          <CHeading type="header">
            {{ t('nav.residence-info') }}
          </CHeading>
          <p
            class="body-md"
            style="color:var(--color-on-surface-variant)"
          >
            {{ t('settings.building.subtitle') }}
          </p>
        </header>

        <CCard>
          <CHeading
            type="card"
            class="mb-4"
          >
            {{ t('settings.building.title') }}
          </CHeading>
          <CFormRow :cols="2">
            <CFormInput
              v-model="form.name"
              :label="t('settings.building.name')"
            />
            <CFormInput
              v-model="form.manager_siret"
              :label="t('settings.building.siret')"
            />
            <CFormInput
              v-model="form.address_line1"
              :label="t('addressAutocomplete.addressLine1')"
              class="flex flex-col gap-1 col-span-full"
            />
            <CFormInput
              v-model="form.postal_code"
              :label="t('addressAutocomplete.postalCode')"
            />
            <CFormInput
              v-model="form.city"
              :label="t('addressAutocomplete.city')"
            />
            <CFormInput
              v-model="form.shares_total"
              :label="t('settings.building.sharesTotal')"
              type="number"
              :step="1"
            />
            <CFormInput
              v-model="form.bylaws_date"
              :label="t('settings.building.bylawsDate')"
              type="date"
            />
          </CFormRow>
          <div class="flex items-center gap-3">
            <CBtn
              variant="primary"
              :title="saving ? t('common.saving') : t('settings.saveChanges')"
              :loading="saving"
              :disabled="saving"
              @click="saveBuilding"
            />
            <CFormFeedback
              :status="feedback.status.value"
              :message="feedback.message.value"
            />
          </div>
        </CCard>

        <CLegalCard>
          <CHeading
            type="card"
            class="mb-4"
          >
            {{ t('settings.building.legalData') }}
          </CHeading>
          <CLegalNotice>
            <CLegalDescription
              title="Art. 18 loi n°65-557 du 10 juillet 1965"
              :summary="t('settings.building.legalText')"
            />
          </CLegalNotice>
          <div class="flex items-center gap-3">
            <CBtn
              variant="secondary"
              :icon="CIconDownload"
              :title="t('settings.exportData')"
              disabled
            />
            <CBtn
              variant="secondary"
              :icon="CIconBook"
              :title="t('settings.downloadBylaws')"
              disabled
            />
          </div>
        </CLegalCard>
      </template>
    </div>
  </div>
</template>

<style scoped>
</style>
