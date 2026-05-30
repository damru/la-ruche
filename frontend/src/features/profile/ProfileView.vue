<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CModal from '@/components/molecules/CModal.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CIconClock from '@/components/atoms/CIconClock.svg'
import CIconLogIn from '@/components/atoms/CIconLogIn.svg'
import CIconMoon from '@/components/atoms/CIconMoon.svg'
import CIconSun from '@/components/atoms/CIconSun.svg'
import { useAuthStore } from '@/stores/auth'
import { useUiStore } from '@/stores/ui'
import { authApi, adminApi } from '@/api'
import { useFormFeedback, SUCCESS_AUTO_CLEAR_MS } from '@/composables/useFormFeedback'
import CTabs from '@/components/atoms/CTabs.vue'
import CToggle from '@/components/atoms/CToggle.vue'

const { t } = useI18n()
const router = useRouter()
const auth = useAuthStore()
const ui = useUiStore()

// ── Tabs ──────────────────────────────────────────────────────────
type ProfileTab = 'account' | 'notifications'
const activeTab = ref<ProfileTab>('account')

// ── Account form ─────────────────────────────────────────────────
const saving = ref(false)
const profileFeedback = useFormFeedback()
const phone = ref(auth.user?.phone ?? '')

const themeOptions = [
  {
    value: 'light',
    labelKey: 'settings.preferences.theme.light.label',
    hintKey: 'settings.preferences.theme.light.hint',
    icon: CIconSun,
  },
  {
    value: 'dark',
    labelKey: 'settings.preferences.theme.dark.label',
    hintKey: 'settings.preferences.theme.dark.hint',
    icon: CIconMoon,
  },
  {
    value: 'auto',
    labelKey: 'settings.preferences.theme.auto.label',
    hintKey: 'settings.preferences.theme.auto.hint',
    icon: CIconClock,
  },
] as const

const selectedThemeOption = computed(
  () => themeOptions.find((option) => option.value === ui.themePreference) ?? themeOptions[2]
)

async function saveProfile() {
  saving.value = true
  profileFeedback.reset()
  try {
    const res = await authApi.updateProfile({ phone: phone.value || null })
    if (auth.user) {
      auth.user.phone = res.data.phone
    }
    profileFeedback.setSuccess(t('common.saved'))
    setTimeout(() => profileFeedback.reset(), SUCCESS_AUTO_CLEAR_MS)
  } catch {
    profileFeedback.setError(t('common.error'))
  } finally {
    saving.value = false
  }
}

// ── Notifications ─────────────────────────────────────────────────
const notifItems = ref([
  { key: 'contractReminders', enabled: true },
  { key: 'newReports', enabled: true },
  { key: 'lateCallsForFunds', enabled: true },
  { key: 'openConsultations', enabled: false },
  { key: 'weeklyDigest', enabled: false },
])

// ── Logout ────────────────────────────────────────────────────────
const loggingOut = ref(false)

async function logout() {
  loggingOut.value = true
  try {
    await auth.logout()
    router.push({ name: 'login' })
  } finally {
    loggingOut.value = false
  }
}

// ── Reset demo ────────────────────────────────────────────────────
const showResetModal = ref(false)
const resetting = ref(false)
const resetFeedback = useFormFeedback()

async function confirmResetDemo() {
  resetting.value = true
  resetFeedback.reset()
  try {
    await adminApi.resetDemo()
    resetFeedback.setSuccess(t('settings.account.resetAppSuccess'))
    setTimeout(() => {
      showResetModal.value = false
      resetFeedback.reset()
    }, SUCCESS_AUTO_CLEAR_MS)
  } catch {
    resetFeedback.setError(t('settings.account.resetAppError'))
  } finally {
    resetting.value = false
  }
}

// ── Reset data ────────────────────────────────────────────────────
const showResetDataModal = ref(false)
const resettingData = ref(false)
const resetDataFeedback = useFormFeedback()

async function confirmResetData() {
  resettingData.value = true
  resetDataFeedback.reset()
  try {
    await adminApi.resetData()
    resetDataFeedback.setSuccess(t('settings.account.resetDataSuccess'))
    setTimeout(() => {
      showResetDataModal.value = false
      resetDataFeedback.reset()
    }, SUCCESS_AUTO_CLEAR_MS)
  } catch {
    resetDataFeedback.setError(t('settings.account.resetDataError'))
  } finally {
    resettingData.value = false
  }
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          <span>{{ t('nav.account') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="ghost"
          size="sm"
          :icon="CIconLogIn"
          :title="t('nav.logout')"
          :loading="loggingOut"
          :disabled="loggingOut"
          @click="logout"
        />
      </template>
    </CTopbar>

    <div class="view-container">
      <header class="view-header">
        <CEyebrow>
          {{ t('page.eyebrows.profile') }}
        </CEyebrow>
        <CHeading type="header">
          {{ t('nav.account') }}
        </CHeading>
      </header>

      <!-- Tabs -->
      <CTabs
        v-model="activeTab"
        variant="tab"
        :tabs="[
          { value: 'account', label: t('nav.myAccount') },
          { value: 'notifications', label: t('nav.notifications') },
        ]"
      />

      <!-- ── Account tab ─────────────────────────────────────── -->
      <template v-if="activeTab === 'account'">
        <CCard>
          <CHeading type="card">
            {{ t('settings.account.personalInfo') }}
          </CHeading>
          <CForm
            :status="profileFeedback.status.value"
            :message="profileFeedback.message.value"
            @submit="saveProfile"
          >
            <CFormRow :cols="2">
              <CFormInput
                :model-value="auth.user?.first_name ?? ''"
                :label="t('common.firstName')"
                disabled
              />
              <CFormInput
                :model-value="auth.user?.last_name ?? ''"
                :label="t('common.lastName')"
                disabled
              />
              <CFormInput
                :model-value="auth.user?.email ?? ''"
                :label="t('common.email')"
                disabled
              />
              <CFormInput
                v-model="phone"
                :label="t('common.phone')"
                type="tel"
                :placeholder="t('common.phonePlaceholder')"
              />
            </CFormRow>
            <template #actions>
              <CBtn
                variant="primary"
                type="submit"
                :title="t('settings.account.update')"
                :loading="saving"
                :disabled="saving"
              />
              <CBtn
                variant="ghost"
                :title="t('settings.account.changePassword')"
              />
            </template>
          </CForm>
        </CCard>

        <CCard>
          <CHeading type="card">
            {{ t('settings.preferences.title') }}
          </CHeading>
          <p class="preferences-subtitle">
            {{ t('settings.preferences.subtitle') }}
          </p>

          <div
            class="theme-options"
            role="radiogroup"
            :aria-label="t('settings.preferences.theme.label')"
          >
            <button
              v-for="option in themeOptions"
              :key="option.value"
              type="button"
              class="theme-option"
              :class="{ 'theme-option--active': ui.themePreference === option.value }"
              role="radio"
              :aria-checked="ui.themePreference === option.value"
              @click="ui.setThemePreference(option.value)"
            >
              <component
                :is="option.icon"
                class="theme-option__icon"
                aria-hidden="true"
              />
              <span>{{ t(option.labelKey) }}</span>
            </button>
          </div>

          <p class="text-xs text-on-surface-variant theme-hint">
            {{ t(selectedThemeOption.hintKey) }}
          </p>
        </CCard>

        <CCard v-if="auth.isManager && ui.demoMode">
          <CHeading
            type="card"
            style="color: var(--color-error)"
          >
            {{ t('settings.account.dangerZone') }}
          </CHeading>
          <p
            class="body-xs"
            style="color: var(--color-on-surface-variant); margin-bottom: 12px"
          >
            {{ t('settings.account.dangerDesc') }}
          </p>
          <div class="flex flex-row gap-4">
            <CBtn
              variant="danger"
              :title="t('settings.account.resetApp')"
              @click="showResetModal = true"
            />
            <CBtn
              variant="danger"
              :title="t('settings.account.resetData')"
              @click="showResetDataModal = true"
            />
          </div>
        </CCard>
      </template>

      <!-- ── Notifications tab ───────────────────────────────── -->
      <template v-if="activeTab === 'notifications'">
        <CCard>
          <CHeading type="card">
            {{ t('settings.notifications.title') }}
          </CHeading>
          <div class="notif-list">
            <div
              v-for="item in notifItems"
              :key="item.key"
              class="notif-item"
              @click="item.enabled = !item.enabled"
            >
              <div class="notif-info">
                <span class="title-sm">{{ t(`settings.notifications.${item.key}`) }}</span>
                <p
                  class="body-xs"
                  style="color: var(--color-on-surface-variant)"
                >
                  {{ t(`settings.notifications.${item.key}Desc`) }}
                </p>
              </div>
              <CToggle
                v-model="item.enabled"
                :aria-label="t(`settings.notifications.${item.key}`)"
              />
            </div>
          </div>
        </CCard>
      </template>
    </div>
  </div>

  <!-- ── Reset demo modal ─────────────────────────────────────── -->
  <CModal
    :open="showResetModal"
    :title="t('settings.account.resetAppConfirmTitle')"
    @close="showResetModal = false"
  >
    <p class="body-xs">
      {{ t('settings.account.resetAppConfirmMessage') }}
    </p>
    <template #footer>
      <CFormActions
        :status="resetFeedback.status.value"
        :message="resetFeedback.message.value"
      >
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          :disabled="resetting"
          @click="showResetModal = false"
        />
        <CBtn
          variant="danger"
          :title="t('settings.account.resetAppConfirm')"
          :loading="resetting"
          :disabled="resetting"
          @click="confirmResetDemo"
        />
      </CFormActions>
    </template>
  </CModal>

  <!-- ── Reset data modal ──────────────────────────────────────── -->
  <CModal
    :open="showResetDataModal"
    :title="t('settings.account.resetDataConfirmTitle')"
    @close="showResetDataModal = false"
  >
    <p class="body-xs">
      {{ t('settings.account.resetDataConfirmMessage') }}
    </p>
    <template #footer>
      <CFormActions
        :status="resetDataFeedback.status.value"
        :message="resetDataFeedback.message.value"
      >
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          :disabled="resettingData"
          @click="showResetDataModal = false"
        />
        <CBtn
          variant="danger"
          :title="t('settings.account.resetDataConfirm')"
          :loading="resettingData"
          :disabled="resettingData"
          @click="confirmResetData"
        />
      </CFormActions>
    </template>
  </CModal>
</template>

<style scoped>

.preferences-subtitle {
  color: var(--color-on-surface-variant);
  margin-bottom: var(--space-4);
}

.theme-options {
  display: inline-flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.theme-option {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  border-radius: var(--radius-full);
  background: var(--color-surface-container-low);
  color: var(--color-on-surface);
  text-align: center;
  min-height: unset;
  min-width: unset;
  transition:
    background var(--transition-fast),
    color var(--transition-fast),
    box-shadow var(--transition-fast);
}

.theme-option:hover {
  background: var(--color-surface-container);
}

.theme-option--active {
  background: var(--color-secondary-container);
  box-shadow: inset 0 0 0 2px var(--color-primary);
}

.theme-option__icon {
  width: 16px;
  height: 16px;
  flex: 0 0 auto;
}

.theme-hint {
  margin-bottom: 0;
}

/* Form */

/* Notifications */
.notif-list { display: flex; flex-direction: column; gap: 0; }
.notif-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  padding: 14px 0;
  border-bottom: 1px solid var(--color-outline-variant);
  cursor: pointer;
}
.notif-item:last-child { border-bottom: none; }
.notif-info { display: flex; flex-direction: column; gap: 2px; }
</style>
