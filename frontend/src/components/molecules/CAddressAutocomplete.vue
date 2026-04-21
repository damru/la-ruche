<script setup lang="ts">
import { ref, watch, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'

interface AddressValue {
  address_line1: string
  postal_code: string
  city: string
}

interface BanFeature {
  properties: {
    id: string
    label: string
    housenumber?: string
    street?: string
    postcode: string
    city: string
  }
}

const props = defineProps<{
  modelValue: AddressValue
  disabled?: boolean
  errors?: { address_line1?: boolean; postal_code?: boolean; city?: boolean }
}>()

const emit = defineEmits<{
  'update:modelValue': [AddressValue]
  'blur:address_line1': []
  'blur:postal_code': []
  'blur:city': []
}>()

const { t } = useI18n()

// ── Suggestions state ──────────────────────────────────────────────
const suggestions = ref<BanFeature['properties'][]>([])
const activeIdx   = ref(-1)
const searching   = ref(false)
const open        = ref(false)

// ── DOM refs for dropdown positioning ─────────────────────────────
const addressInputRef = ref<HTMLInputElement | null>(null)
const dropdownRef     = ref<HTMLUListElement | null>(null)
const dropdownStyle   = ref<Record<string, string>>({})

// ── Debounce ───────────────────────────────────────────────────────
let debounceTimer: ReturnType<typeof setTimeout> | null = null

// ── BAN API search ─────────────────────────────────────────────────
async function search(q: string) {
  if (q.trim().length < 3) { closeDropdown(); return }
  searching.value = true
  try {
    const url = `https://api-adresse.data.gouv.fr/search/?q=${encodeURIComponent(q)}&limit=6`
    const res = await fetch(url)
    if (!res.ok) throw new Error()
    const data = await res.json()
    suggestions.value = (data.features ?? []).map((f: BanFeature) => f.properties)
    activeIdx.value = -1
    open.value = suggestions.value.length > 0
    if (open.value) recalcDropdown()
  } catch {
    suggestions.value = []
    open.value = false
  } finally {
    searching.value = false
  }
}

// ── Input handlers ─────────────────────────────────────────────────
function onAddress1Input(e: Event) {
  const value = (e.target as HTMLInputElement).value
  emit('update:modelValue', { ...props.modelValue, address_line1: value })
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => search(value), 300)
}

function onAddress1Blur() {
  setTimeout(closeDropdown, 150)
  emit('blur:address_line1')
}

function selectSuggestion(s: BanFeature['properties']) {
  const line1 = [s.housenumber, s.street].filter(Boolean).join(' ')
  emit('update:modelValue', {
    address_line1: line1,
    postal_code:   s.postcode,
    city:          s.city,
  })
  closeDropdown()
}

function navigate(dir: 1 | -1) {
  if (!open.value || !suggestions.value.length) return
  activeIdx.value = Math.max(-1, Math.min(suggestions.value.length - 1, activeIdx.value + dir))
}

function selectActive() {
  if (activeIdx.value >= 0 && suggestions.value[activeIdx.value]) {
    selectSuggestion(suggestions.value[activeIdx.value])
  }
}

function closeDropdown() {
  open.value = false
  activeIdx.value = -1
}

// ── Dropdown position (Teleport + fixed per DESIGN.md rule) ───────
const MARGIN = 8

function recalcDropdown() {
  if (!addressInputRef.value) return
  const rect = addressInputRef.value.getBoundingClientRect()
  const vw   = window.innerWidth
  const w    = rect.width
  const clampedLeft = Math.max(MARGIN, Math.min(rect.left, vw - w - MARGIN))
  dropdownStyle.value = {
    position: 'fixed',
    top:    `${rect.bottom + 4}px`,
    left:   `${clampedLeft}px`,
    width:  `${w}px`,
    zIndex: '200',
  }
}

function onClickOutside(e: MouseEvent) {
  const target = e.target as Node
  if (
    addressInputRef.value?.contains(target) === false &&
    dropdownRef.value?.contains(target) === false
  ) closeDropdown()
}

watch(open, (v) => {
  if (v) {
    window.addEventListener('mousedown', onClickOutside)
    window.addEventListener('resize', recalcDropdown)
    window.addEventListener('scroll', recalcDropdown, true)
  } else {
    window.removeEventListener('mousedown', onClickOutside)
    window.removeEventListener('resize', recalcDropdown)
    window.removeEventListener('scroll', recalcDropdown, true)
  }
})

onUnmounted(() => {
  window.removeEventListener('mousedown', onClickOutside)
  window.removeEventListener('resize', recalcDropdown)
  window.removeEventListener('scroll', recalcDropdown, true)
  if (debounceTimer) clearTimeout(debounceTimer)
})

function updateField(field: keyof AddressValue, value: string) {
  emit('update:modelValue', { ...props.modelValue, [field]: value })
}
</script>

<template>
  <div class="c-addr">
    <!-- address_line1 — doubles as BAN search trigger -->
    <div class="c-addr__field">
      <label
        for="address_line1"
        class="c-addr__label"
      >
        {{ t('addressAutocomplete.addressLine1') }}
        <span
          class="c-addr__required"
          aria-hidden="true"
        >*</span>
      </label>
      <div class="c-addr__input-wrap">
        <input
          id="address_line1"
          ref="addressInputRef"
          :value="modelValue.address_line1"
          type="text"
          :class="['c-addr__input', { 'c-addr__input--error': errors?.address_line1 }]"
          :placeholder="t('addressAutocomplete.searchPlaceholder')"
          autocomplete="street-address"
          aria-autocomplete="list"
          aria-haspopup="listbox"
          :aria-expanded="open"
          aria-controls="ban-suggestions"
          required
          :disabled="disabled"
          @input="onAddress1Input"
          @keydown.arrow-down.prevent="navigate(1)"
          @keydown.arrow-up.prevent="navigate(-1)"
          @keydown.enter.prevent="selectActive"
          @keydown.escape="closeDropdown"
          @blur="onAddress1Blur"
        />
        <CIconSpinner
          v-if="searching"
          class="c-addr__spinner"
          aria-hidden="true"
        />
      </div>
      <p
        v-if="errors?.address_line1"
        class="c-addr__error"
        role="alert"
      >
        {{ t('onboarding.fieldRequired') }}
      </p>
    </div>

    <!-- Teleported suggestion dropdown -->
    <Teleport to="body">
      <ul
        v-if="open"
        id="ban-suggestions"
        ref="dropdownRef"
        role="listbox"
        class="c-addr-suggestions"
        :style="dropdownStyle"
      >
        <li
          v-for="(s, i) in suggestions"
          :key="s.id"
          role="option"
          :aria-selected="i === activeIdx"
          :class="['c-addr-suggestions__item', { 'c-addr-suggestions__item--active': i === activeIdx }]"
          @mousedown.prevent="selectSuggestion(s)"
        >
          <CIconLocation aria-hidden="true" />
          {{ s.label }}
        </li>
      </ul>
    </Teleport>

    <!-- postal_code + city — always editable -->
    <div class="c-addr__row">
      <div class="c-addr__field c-addr__field--narrow">
        <label
          for="postal_code"
          class="c-addr__label"
        >
          {{ t('addressAutocomplete.postalCode') }}
          <span
            class="c-addr__required"
            aria-hidden="true"
          >*</span>
        </label>
        <input
          id="postal_code"
          :value="modelValue.postal_code"
          type="text"
          :class="['c-addr__input', { 'c-addr__input--error': errors?.postal_code }]"
          :placeholder="t('onboarding.postalCodePlaceholder')"
          autocomplete="postal-code"
          inputmode="numeric"
          maxlength="5"
          required
          :disabled="disabled"
          @input="updateField('postal_code', ($event.target as HTMLInputElement).value)"
          @blur="$emit('blur:postal_code')"
        />
        <p
          v-if="errors?.postal_code"
          class="c-addr__error"
          role="alert"
        >
          {{ t('onboarding.fieldRequired') }}
        </p>
      </div>
      <div class="c-addr__field">
        <label
          for="city"
          class="c-addr__label"
        >
          {{ t('addressAutocomplete.city') }}
          <span
            class="c-addr__required"
            aria-hidden="true"
          >*</span>
        </label>
        <input
          id="city"
          :value="modelValue.city"
          type="text"
          :class="['c-addr__input', { 'c-addr__input--error': errors?.city }]"
          :placeholder="t('onboarding.cityPlaceholder')"
          autocomplete="address-level2"
          required
          :disabled="disabled"
          @input="updateField('city', ($event.target as HTMLInputElement).value)"
          @blur="$emit('blur:city')"
        />
        <p
          v-if="errors?.city"
          class="c-addr__error"
          role="alert"
        >
          {{ t('onboarding.fieldRequired') }}
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.c-addr {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.c-addr__field {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.c-addr__label {
  font-family: var(--font-body);
  font-size: 13px;
  font-weight: 500;
  color: var(--color-on-surface);
}

.c-addr__required { color: var(--color-error); margin-left: 2px; }

.c-addr__input-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.c-addr__input {
  width: 100%;
  height: 44px;
  padding: 0 var(--space-4);
  background: var(--color-surface-container-high);
  border: none;
  border-radius: var(--radius-md);
  font-family: var(--font-body);
  font-size: 14px;
  color: var(--color-on-surface);
  outline: none;
  transition:
    background var(--transition-fast),
    box-shadow var(--transition-fast);
}

.c-addr__input::placeholder { color: var(--color-on-surface-variant); opacity: 0.7; }

.c-addr__input:focus {
  background: var(--color-surface-container);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-primary) 35%, transparent);
}

.c-addr__input--error {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-error) 35%, transparent);
}

.c-addr__input:disabled { opacity: 0.5; cursor: not-allowed; }

.c-addr__spinner {
  position: absolute;
  right: var(--space-3);
  width: 16px;
  height: 16px;
  color: var(--color-primary);
  animation: c-addr-spin 0.8s linear infinite;
  flex-shrink: 0;
  pointer-events: none;
}
@keyframes c-addr-spin { to { transform: rotate(360deg); } }

.c-addr__error {
  font-family: var(--font-body);
  font-size: 12px;
  color: var(--color-error);
}

.c-addr__row {
  display: flex;
  gap: var(--space-3);
  align-items: flex-start;
}

.c-addr__field--narrow { flex: 0 0 120px; }
.c-addr__row .c-addr__field:not(.c-addr__field--narrow) { flex: 1; }
</style>

<style>
.c-addr-suggestions {
  list-style: none;
  margin: 0;
  padding: var(--space-1) 0;
  background: var(--color-surface-container-lowest);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
}

.c-addr-suggestions__item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  font-family: var(--font-body);
  font-size: 13px;
  color: var(--color-on-surface);
  cursor: pointer;
  transition: background var(--transition-fast);
  user-select: none;
}

.c-addr-suggestions__item svg {
  width: 13px;
  height: 13px;
  flex-shrink: 0;
  color: var(--color-primary);
  opacity: 0.6;
}

.c-addr-suggestions__item:hover,
.c-addr-suggestions__item--active {
  background: var(--color-surface-container-high);
}

.c-addr-suggestions__item--active { color: var(--color-primary); }
.c-addr-suggestions__item--active svg { opacity: 1; }
</style>
