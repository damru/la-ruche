<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CForm from '@/components/molecules/CForm.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CTabs from '@/components/atoms/CTabs.vue'
import CToggle from '@/components/atoms/CToggle.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CIconClock from '@/components/atoms/CIconClock.svg'
import CIconLogIn from '@/components/atoms/CIconLogIn.svg'
import CIconMoon from '@/components/atoms/CIconMoon.svg'
import CIconRefresh from '@/components/atoms/CIconRefresh.svg'
import CIconSun from '@/components/atoms/CIconSun.svg'
import { useAuthStore } from '@/stores/auth'
import { usePermissions } from '@/composables/usePermissions'
import { useUiStore, NOTIF_EVENTS, AUTO_CLOSE_DELAY_MIN_SEC, AUTO_CLOSE_DELAY_MAX_SEC } from '@/stores/ui'
import { authApi } from '@/api'
import { useBuildingStore } from '@/stores/building'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useConfirm } from '@/composables/useConfirm'
import ResetDemoModal from './ResetDemoModal.vue'
import ResetDataModal from './ResetDataModal.vue'

const { t } = useI18n()
const router = useRouter()
const auth = useAuthStore()
const { canManageUsers } = usePermissions()
const ui = useUiStore()
const { confirm } = useConfirm()

// ── Tabs ──────────────────────────────────────────────────────────
// "Notifications" and "Appearance" live under the common "Préférences" tab;
// `prefsTab` holds the sub-tab selected inside it.
type ProfileTab = 'account' | 'preferences'
type PrefsTab = 'notifications' | 'appearance'
const activeTab = ref<ProfileTab>('account')
const prefsTab = ref<PrefsTab>('notifications')

// ── Account form (email = login, phone) ───────────────────────────
const saving = ref(false)
const profileFeedback = useFormFeedback()
const email = ref(auth.user?.email ?? '')
const phone = ref(auth.user?.phone ?? '')
const emailError = ref('')

// ── Access context (role + building scope) ────────────────────────
const buildingName = ref('')
onMounted(async () => {
  try {
    const building = await useBuildingStore().fetch()
    buildingName.value = building.name
  } catch {
    // Scope name is optional context — silently skip if unavailable.
  }
})

async function saveProfile() {
  saving.value = true
  profileFeedback.reset()
  emailError.value = ''
  try {
    const payload: { phone: string | null; email?: string } = {
      phone: phone.value.trim() || null,
    }
    const nextEmail = email.value.trim().toLowerCase()
    if (nextEmail && nextEmail !== (auth.user?.email ?? '').toLowerCase()) {
      payload.email = nextEmail
    }
    const res = await authApi.updateProfile(payload)
    if (auth.user) {
      auth.user.phone = res.data.phone
      auth.user.email = res.data.email ?? auth.user.email
    }
    profileFeedback.setSuccess(t('common.saved'))
  } catch (e: unknown) {
    const status = (e as { response?: { status?: number } })?.response?.status
    const msg = (e as { response?: { data?: { message?: string } } })?.response?.data?.message ?? ''
    if (status === 409 && msg.includes('email_already_exists')) {
      emailError.value = t('settings.account.emailTaken')
    } else if (status === 400 && msg.includes('invalid_email')) {
      emailError.value = t('settings.account.emailInvalid')
    } else {
      profileFeedback.setError(t('common.error'))
    }
  } finally {
    saving.value = false
  }
}

// ── Theme (appearance tab) ────────────────────────────────────────
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

// ── Notifications ─────────────────────────────────────────────────
// Hydrate from the server when the preferences tab is first opened;
// localStorage remains the fallback cache on network error.
watch(
  () => [activeTab.value, prefsTab.value],
  ([tab, subTab]) => {
    if (tab === 'preferences' && subTab === 'notifications') {
      void ui.loadNotifPrefs()
    }
  },
  { immediate: true },
)

// ── Auto-close delay selector ─────────────────────────────────────────────────
const autoCloseDelayOptions = computed(() =>
  Array.from(
    { length: AUTO_CLOSE_DELAY_MAX_SEC - AUTO_CLOSE_DELAY_MIN_SEC + 1 },
    (_, i) => {
      const seconds = AUTO_CLOSE_DELAY_MIN_SEC + i
      return {
        value: seconds,
        label: seconds === 0
          ? t('settings.preferences.autoCloseDelay.immediate')
          : t('settings.preferences.autoCloseDelay.seconds', { count: seconds }),
      }
    },
  ),
)

const selectedAutoCloseDelay = computed({
  get: () => String(ui.autoCloseDelaySec),
  set: (value: string) => ui.setAutoCloseDelay(Number(value)),
})

// ── Global preferences reset ──────────────────────────────────────────────────
async function resetAllPrefs() {
  const ok = await confirm({
    title: t('settings.preferences.resetConfirmTitle'),
    message: t('settings.preferences.resetConfirmMessage'),
    variant: 'primary',
  })
  if (ok) {
    await ui.resetAllPrefs()
  }
}

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

// ── Reset demo / data (manager + demo only) ───────────────────────
const showResetModal = ref(false)
const showResetDataModal = ref(false)
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

      <!-- Tabs + global reset action -->
      <div class="flex items-center justify-between gap-3">
        <CTabs
          v-model="activeTab"
          variant="filter"
          :tabs="[
            { value: 'account', label: t('nav.myAccount') },
            { value: 'preferences', label: t('settings.preferences.title') },
          ]"
          class="mb-0!"
        />
        <CBtn
          variant="ghost"
          size="sm"
          :icon="CIconRefresh"
          :title="t('settings.preferences.reset')"
          @click="resetAllPrefs"
        />
      </div>

      <!-- Preferences sub-tabs -->
      <CTabs
        v-if="activeTab === 'preferences'"
        v-model="prefsTab"
        variant="filter"
        :tabs="[
          { value: 'notifications', label: t('settings.tabs.notifications') },
          { value: 'appearance', label: t('settings.preferences.theme.label') },
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
            :cols="2"
            @submit="saveProfile"
          >
            <CFormInput
              :model-value="auth.user?.first_name ?? ''"
              :label="t('common.firstName')"
              disabled
            />
            <CFormInput
              :model-value="auth.user?.last_name ?? ''"
              :label="t('lastName')"
              disabled
            />
            <CFormInput
              v-model="email"
              :label="t('common.email')"
              type="email"
              required
              :error="emailError"
              @update:model-value="emailError = ''"
            />
            <CFormInput
              v-model="phone"
              :label="t('common.phone')"
              type="tel"
              :placeholder="t('common.phonePlaceholder')"
            />
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
                disabled
              />
            </template>
          </CForm>
        </CCard>

        <CCard>
          <CHeading type="card">
            {{ t('settings.account.accessTitle') }}
          </CHeading>
          <div class="access-grid">
            <div class="access-row">
              <span class="body-xs access-label">{{ t('common.role') }}</span>
              <div class="access-badges">
                <CBadge
                  v-for="role in auth.user?.roles ?? []"
                  :key="role"
                  :variant="role"
                >
                  {{ t(`roles.${role}`) }}
                </CBadge>
              </div>
            </div>
            <div
              v-if="buildingName"
              class="access-row"
            >
              <span class="body-xs access-label">{{ t('settings.account.scopeLabel') }}</span>
              <span class="title-sm">{{ buildingName }}</span>
            </div>
          </div>
        </CCard>

        <CCard v-if="canManageUsers && ui.demoMode">
          <CHeading
            type="card"
            class="danger-title"
          >
            {{ t('settings.account.dangerZone') }}
          </CHeading>
          <p class="body-xs danger-desc">
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
      <template v-if="activeTab === 'preferences' && prefsTab === 'notifications'">
        <CCard>
          <div class="notif-header">
            <div class="notif-header__text">
              <CHeading type="card">
                {{ t('settings.notifications.title') }}
              </CHeading>
              <p class="body-xs notif-subtitle">
                {{ t('settings.notifications.subtitle') }}
              </p>
            </div>
          </div>
          <div class="notif-list">
            <div
              v-for="event in NOTIF_EVENTS"
              :key="event.id"
              class="notif-item"
            >
              <div class="notif-info">
                <div class="notif-head">
                  <span class="title-sm">{{ t(`settings.notifications.events.${event.id}`) }}</span>
                  <CBadge
                    v-if="event.mandatory"
                    variant="default"
                  >
                    {{ t('settings.notifications.mandatory') }}
                  </CBadge>
                </div>
                <p class="body-xs notif-desc">
                  {{ t(`settings.notifications.events.${event.id}Desc`) }}
                </p>
              </div>
              <CToggle
                :model-value="ui.notifPrefs[event.id]"
                :disabled="event.mandatory"
                :aria-label="t(`settings.notifications.events.${event.id}`)"
                @update:model-value="ui.setNotifPref(event.id, $event)"
              />
            </div>
          </div>
        </CCard>
      </template>

      <!-- ── Appearance tab ──────────────────────────────────── -->
      <template v-if="activeTab === 'preferences' && prefsTab === 'appearance'">
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

          <div class="auto-close-setting">
            <CFormSelect
              v-model="selectedAutoCloseDelay"
              :label="t('settings.preferences.autoCloseDelay.label')"
              :hint="t('settings.preferences.autoCloseDelay.hint')"
              :options="autoCloseDelayOptions"
            />
          </div>
        </CCard>
      </template>
    </div>
  </div>

  <!-- ── Reset demo modal ─────────────────────────────────────── -->
  <ResetDemoModal v-model:open="showResetModal" />

  <!-- ── Reset data modal ──────────────────────────────────────── -->
  <ResetDataModal v-model:open="showResetDataModal" />
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
  gap: var(--space-2);
  padding: var(--space-3) var(--space-4);
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
  width: var(--icon-sm);
  height: var(--icon-sm);
  flex: 0 0 auto;
}

.theme-hint {
  margin-bottom: 0;
}

.auto-close-setting {
  margin-top: var(--space-4);
  max-width: 320px;
}

/* Access context */
.access-grid {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}
.access-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  flex-wrap: wrap;
}
.access-label {
  min-width: var(--space-20);
  color: var(--color-on-surface-variant);
}
.access-badges {
  display: inline-flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

/* Danger zone */
.danger-title {
  color: var(--color-error);
}
.danger-desc {
  color: var(--color-on-surface-variant);
  margin-bottom: var(--space-3);
}

/* Notifications */
.notif-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
  margin-bottom: var(--space-2);
}
.notif-subtitle {
  color: var(--color-on-surface-variant);
}
.notif-list {
  display: flex;
  flex-direction: column;
  gap: 0;
}
.notif-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  padding: var(--space-4) 0;
  border-bottom: 1px solid var(--color-outline-variant);
}
.notif-item:last-child {
  border-bottom: none;
}
.notif-info {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}
.notif-head {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}
.notif-desc {
  color: var(--color-on-surface-variant);
}
</style>
