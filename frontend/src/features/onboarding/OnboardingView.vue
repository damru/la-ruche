<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { onboardingApi } from '@/api'
import { extractApiError } from '@/api/client'
import AuthBrandPanel from '@/components/organisms/AuthBrandPanel.vue'
import CAlert from '@/components/atoms/CAlert.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CFormAddressAutocomplete from '@/components/molecules/CFormAddressAutocomplete.vue'
import CIconArrowLeft from '@/components/atoms/CIconArrowLeft.svg'
import CIconArrowRight from '@/components/atoms/CIconArrowRight.svg'

const { t } = useI18n()
const router = useRouter()
const auth = useAuthStore()

// ── State ──────────────────────────────────────────────────────────
const loading = ref(false)
const error = ref('')
const alreadyInitialized = ref(false)
const step = ref<1 | 2>(1)
const showPassword = ref(false)

// ── Field touched state (blur → on, submit attempt → all on) ──────────
const touched = reactive({
  last_name: false,
  first_name: false,
  email: false,
  building_name: false,
  address_line1: false,
  postal_code: false,
  city: false,
})
// Password uses "dirty" (first keystroke) instead of blur
const passwordDirty = computed(() => form.password.length > 0)

// ── Brand panel steps ────────────────────────────────────────────
const brandSteps = computed(() => [
  { label: t('onboarding.titleStep1'), status: (step.value > 1 ? 'done' : step.value === 1 ? 'active' : 'future') as 'done' | 'active' | 'future' },
  { label: t('onboarding.titleStep2'), status: (step.value === 2 ? 'active' : 'future') as 'done' | 'active' | 'future' },
  { label: t('onboarding.step3Label'), status: 'future' as const },
])

const form = reactive({
  // Step 1 — Manager
  email: '',
  last_name: '',
  first_name: '',
  password: '',
  // Step 2 — Building
  building_name: '',
  address_line1: '',
  postal_code: '',
  city: '',
  shares_total: '',
})

// ── Per-step validation ───────────────────────────────────────────
const isStep1Valid = computed(() =>
  !!form.email &&
  !!form.last_name &&
  !!form.first_name &&
  form.password.length >= 8
)

const isStep2Valid = computed(() =>
  !!form.building_name &&
  !!form.address_line1 &&
  !!form.postal_code &&
  !!form.city
)

// ── Field-level error helpers ─────────────────────────────────────
const err = computed(() => ({
  last_name:     touched.last_name     && !form.last_name,
  first_name:    touched.first_name    && !form.first_name,
  email:         touched.email         && !form.email,
  building_name: touched.building_name && !form.building_name,
  address_line1: touched.address_line1 && !form.address_line1,
  postal_code:   touched.postal_code   && !form.postal_code,
  city:          touched.city          && !form.city,
}))

function touch(field: keyof typeof touched) {
  touched[field] = true
}

function touchAllStep1() {
  touched.last_name = true
  touched.first_name = true
  touched.email = true
}

function touchAllStep2() {
  touched.building_name = true
  touched.address_line1 = true
  touched.postal_code = true
  touched.city = true
}

// ── Address autocomplete binding ─────────────────────────────────
const addressValue = computed(() => ({
  address_line1: form.address_line1,
  postal_code: form.postal_code,
  city: form.city,
}))

function onAddressUpdate(val: { address_line1: string; postal_code: string; city: string }) {
  form.address_line1 = val.address_line1
  form.postal_code = val.postal_code
  form.city = val.city
}

// ── Navigation wizard ──────────────────────────────────────────────
function goNext() {
  if (isStep1Valid.value) {
    error.value = ''
    step.value = 2
  } else {
    touchAllStep1()
  }
}

function goBack() {
  error.value = ''
  step.value = 1
}

// ── Submission ─────────────────────────────────────────────────────
async function handleSubmit() {
  if (!isStep2Valid.value) {
    touchAllStep2()
    return
  }
  error.value = ''
  loading.value = true
  try {
    const { data } = await onboardingApi.bootstrap({
      email: form.email,
      last_name: form.last_name,
      first_name: form.first_name,
      password: form.password,
      building_name: form.building_name,
      address_line1: form.address_line1,
      postal_code: form.postal_code,
      city: form.city,
      shares_total: parseInt(String(form.shares_total).trim(), 10) || 1000,
    })
    auth.completeOnboarding({
      user_id: data.user_id,
      building_id: data.building_id,
      roles: data.roles as import('@/types/domain').Role[],
      last_name: data.last_name,
      first_name: data.first_name,
      email: form.email,
    })
    router.push('/dashboard')
  } catch (e: unknown) {
    const msg = t(extractApiError(e))
    if ((e as { response?: { status?: number } })?.response?.status === 409) {
      alreadyInitialized.value = true
    } else {
      error.value = msg
    }
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="onboarding-root">
    <!-- Left panel — brand -->
    <AuthBrandPanel
      class="onboarding-brand"
      :eyebrow="t('onboarding.brandEyebrow')"
      :title="t('onboarding.brandTitle')"
      :sub="t('onboarding.brandSub')"
      :steps="brandSteps"
    />

    <!-- Right panel — form -->
    <main class="onboarding-panel">
      <div class="onboarding-form-wrap">
        <!-- Already initialised -->
        <div
          v-if="alreadyInitialized"
          class="already-init"
        >
          <div
            class="already-init-icon"
            aria-hidden="true"
          >
            <CIconCheckCircle />
          </div>
          <h2 class="font-serif text-lg font-bold text-on-surface">
            {{ t('onboarding.alreadyInitTitle') }}
          </h2>
          <p class="already-init-text">
            {{ t('onboarding.alreadyInitText') }}
          </p>
          <router-link
            to="/login"
            class="link-btn"
          >
            {{ t('auth.login') }}
          </router-link>
        </div>

        <template v-else>
          <div class="form-header">
            <h2 class="form-title">
              {{ step === 1 ? t('onboarding.titleStep1') : t('onboarding.titleStep2') }}
            </h2>
            <p class="form-sub">
              {{ step === 1 ? t('onboarding.subtitleStep1') : t('onboarding.subtitleStep2') }}
            </p>
          </div>

          <!-- API error -->
          <CAlert
            v-if="error"
            variant="error"
            :title="t('errors.title')"
          >
            {{ error }}
          </CAlert>

          <form
            v-if="step === 1"
            class="flex flex-col gap-4"
            novalidate
            @submit.prevent="goNext"
          >
            <div class="flex flex-col gap-1-row">
              <CFormInput
                id="last_name"
                v-model="form.last_name"
                :label="t('onboarding.lastName')"
                :placeholder="t('onboarding.lastNamePlaceholder')"
                :error="err.last_name ? t('onboarding.fieldRequired') : undefined"
                autocomplete="family-name"
                required
                :disabled="loading"
                @blur="touch('last_name')"
              />
              <CFormInput
                id="first_name"
                v-model="form.first_name"
                :label="t('common.firstName')"
                :placeholder="t('onboarding.firstNamePlaceholder')"
                :error="err.first_name ? t('onboarding.fieldRequired') : undefined"
                autocomplete="given-name"
                required
                :disabled="loading"
                @blur="touch('first_name')"
              />
            </div>

            <CFormInput
              id="email"
              v-model="form.email"
              type="email"
              :label="t('common.email')"
              :placeholder="t('auth.emailPlaceholder')"
              :error="err.email ? t('onboarding.fieldRequired') : undefined"
              autocomplete="email"
              required
              :disabled="loading"
              @blur="touch('email')"
            />

            <div class="flex flex-col gap-1">
              <CFormInput
                id="password"
                v-model="form.password"
                :label="t('auth.password')"
                :type="showPassword ? 'text' : 'password'"
                :placeholder="t('auth.passwordPlaceholder')"
                :error="passwordDirty && form.password.length < 8 ? t('onboarding.passwordRuleLength') : undefined"
                autocomplete="new-password"
                :maxlength="undefined"
                required
                :disabled="loading"
              >
                <template #suffix>
                  <button
                    type="button"
                    class="pwd-toggle"
                    :aria-label="showPassword ? t('auth.hidePassword') : t('auth.showPassword')"
                    :aria-pressed="showPassword"
                    tabindex="0"
                    @click="showPassword = !showPassword"
                  >
                    <CIconEyeOff
                      v-if="showPassword"
                      aria-hidden="true"
                    />
                    <CIconEye
                      v-else
                      aria-hidden="true"
                    />
                  </button>
                </template>
              </CFormInput>
              <!-- Password rules — always visible, state changes on input -->
              <ul
                class="pwd-rules"
                aria-live="polite"
              >
                <li :class="!passwordDirty ? 'pwd-rule--idle' : form.password.length >= 8 ? 'pwd-rule--ok' : 'pwd-rule--err'">
                  <CIconCheck
                    v-if="form.password.length >= 8"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.5"
                    stroke-linecap="round"
                    aria-hidden="true"
                  />
                  <CIconX
                    v-else
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.5"
                    stroke-linecap="round"
                    aria-hidden="true"
                  />
                  {{ t('onboarding.passwordRuleLength') }}
                </li>
              </ul>
            </div>

            <CBtn
              type="submit"
              variant="primary"
              size="md"
              :icon="CIconArrowRight"
              :title="t('onboarding.next')"
              :disabled="loading"
              style="width:100%"
            />
          </form>

          <form
            v-else
            class="flex flex-col gap-4"
            novalidate
            @submit.prevent="handleSubmit"
          >
            <CFormInput
              id="building_name"
              v-model="form.building_name"
              :label="t('onboarding.buildingName')"
              :placeholder="t('onboarding.buildingNamePlaceholder')"
              :error="err.building_name ? t('onboarding.fieldRequired') : undefined"
              required
              :disabled="loading"
              @blur="touch('building_name')"
            />

            <CFormAddressAutocomplete
              :model-value="addressValue"
              :disabled="loading"
              :errors="{ address_line1: err.address_line1, postal_code: err.postal_code, city: err.city }"
              @update:model-value="onAddressUpdate"
              @blur:address_line1="touch('address_line1')"
              @blur:postal_code="touch('postal_code')"
              @blur:city="touch('city')"
            />

            <CFormInput
              id="shares_total"
              v-model="form.shares_total"
              :label="t('registry.totalShares')"
              :placeholder="t('onboarding.sharesTotalPlaceholder')"
              :hint="t('onboarding.sharesTotalHint')"
              inputmode="numeric"
              :disabled="loading"
            />

            <div class="flex items-center gap-3">
              <CBtn
                type="button"
                variant="ghost"
                :icon="CIconArrowLeft"
                :title="t('common.back')"
                :disabled="loading"
                @click="goBack"
              />
              <CBtn
                type="submit"
                variant="primary"
                :icon="!loading ? CIconArrowRight : undefined"
                :title="loading ? t('onboarding.submitting') : t('onboarding.submit')"
                :disabled="loading"
                :loading="loading"
                class="btn-submit"
              />
            </div>
          </form>
        </template>
      </div>
    </main>
  </div>
</template>

<style scoped>
/* ── Two-column layout ─────────────────────────────────── */
.onboarding-root {
  display: flex;
  min-height: 100dvh;
  background: var(--color-surface-container-lowest);
}

@media (max-width: 899px) {
  .onboarding-root {
    flex-direction: column;
  }
}

@media (min-width: 900px)  { .onboarding-brand { width: 400px; flex-shrink: 0; } }

/* ── Form panel (right) ──────────────────────────── */
.onboarding-panel {
  flex: 1;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: var(--space-10) var(--space-5);
  overflow-y: auto;
  min-height: 0;
}
.onboarding-form-wrap {
  width: 100%;
  max-width: 480px;
  display: flex;
  flex-direction: column;
  gap: var(--space-6);
  padding-top: var(--space-4);
}

/* ── Header ────────────────────────────────────────────── */
.form-header { display: flex; flex-direction: column; gap: var(--space-4); }

.form-title {
  font-family: var(--font-display);
  font-size: clamp(22px, 2.4vw, 28px);
  font-weight: 700;
  color: var(--color-on-surface);
}
.form-sub {
  font-size: var(--text-md);
  color: var(--color-on-surface-variant);
  line-height: var(--leading-normal);
}

/* ── Password checklist ────────────────────────────────────────── */
.pwd-rules {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.pwd-rule--idle,
.pwd-rule--ok,
.pwd-rule--err {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  font-size: var(--text-xs);
  line-height: 1.4;
  transition: color var(--transition-fast);
}
.pwd-rule--idle { color: var(--color-on-surface-variant); opacity: 0.6; }
.pwd-rule--ok   { color: var(--color-primary); }
.pwd-rule--err  { color: var(--color-error); }
.pwd-rule--idle svg,
.pwd-rule--ok svg,
.pwd-rule--err svg { width: 13px; height: 13px; flex-shrink: 0; }

.pwd-toggle {
  position: absolute;
  right: var(--space-2);
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--space-7);
  height: var(--space-7);
  padding: 0;
  border: none;
  background: transparent;
  color: var(--color-on-surface-variant);
  opacity: 0.55;
  cursor: pointer;
  border-radius: var(--radius-lg);
  transition: opacity var(--transition-fast), color var(--transition-fast);
  flex-shrink: 0;
}
.pwd-toggle:hover  { opacity: 1; color: var(--color-primary); }
.pwd-toggle:focus-visible { outline: 2px solid var(--color-primary); outline-offset: 2px; opacity: 1; }
.pwd-toggle svg { width: var(--icon-sm); height: var(--icon-sm); }

/* ── Actions (Back + Submit side by side) ───────────────────── */
.btn-submit { flex: 1; }

/* ── Link button (already-initialised state) ──────────────────── */
.link-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: var(--control-height);
  padding: 0 var(--space-5);
  background: var(--gradient-primary);
  color: var(--color-on-primary);
  border-radius: var(--radius-lg);
  font-family: var(--font-body);
  font-size: var(--text-sm);
  font-weight: 600;
  text-decoration: none;
  box-shadow: var(--shadow-primary-lift);
  transition: box-shadow var(--transition-fast), transform var(--transition-fast);
}
.link-btn:hover { transform: translateY(-1px); box-shadow: var(--shadow-primary-lift-hover); }

/* ── Already-initialised state ────────────────────────────── */
.already-init {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-4);
  text-align: center;
  padding: var(--space-10) var(--space-4);
}
.already-init-icon {
  width: var(--space-16);
  height: var(--space-16);
  border-radius: var(--radius-full);
  background: var(--color-primary-tint);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-primary);
}
.already-init-icon svg { width: 32px; height: 32px; }
.already-init-text {
  font-size: var(--text-md);
  color: var(--color-on-surface-variant);
  max-width: 340px;
  line-height: 1.6;
}

@media (max-width: 899px) {
  .onboarding-panel {
    padding: 0 var(--space-4) var(--space-5);
  }

  .onboarding-form-wrap {
    padding-top: var(--space-3);
  }
}
</style>
