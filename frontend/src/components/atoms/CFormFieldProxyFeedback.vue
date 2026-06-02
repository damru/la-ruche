<script setup lang="ts">
const props = withDefaults(defineProps<{
  readonly?: boolean
  showProxy?: boolean
  name?: string
  required?: boolean
  proxyType?: 'text' | 'checkbox'
  proxyValue?: string | number
  proxyChecked?: boolean
  displayError?: string
  errorId?: string
  hint?: string
  hintId?: string
  hintClass?: string
}>(), {
  readonly: false,
  showProxy: true,
  name: undefined,
  required: false,
  proxyType: 'text',
  proxyValue: '',
  proxyChecked: false,
  displayError: undefined,
  errorId: undefined,
  hint: undefined,
  hintId: undefined,
  hintClass: 'form-hint',
})
</script>

<template>
  <input
    v-if="props.showProxy && !props.readonly && props.proxyType === 'text'"
    :name="props.name"
    :required="props.required"
    :value="props.proxyValue"
    type="text"
    tabindex="-1"
    aria-hidden="true"
    class="c-field__native-proxy"
  />
  <input
    v-else-if="props.showProxy && !props.readonly && props.proxyType === 'checkbox'"
    :name="props.name"
    :required="props.required"
    :checked="props.proxyChecked"
    type="checkbox"
    tabindex="-1"
    aria-hidden="true"
    class="c-field__native-proxy"
  />

  <Transition name="field-feedback" mode="out-in">
    <div
      v-if="!props.readonly && (props.displayError || props.hint)"
      :key="props.displayError ? 'error' : 'hint'"
      class="field-feedback-wrap"
    >
      <p
        v-if="props.displayError"
        :id="props.errorId"
        class="field-feedback text-xs text-error flex items-center gap-1"
        role="alert"
      >
        ✗ {{ props.displayError }}
      </p>
      <p
        v-else
        :id="props.hintId"
        :class="['field-feedback', props.hintClass]"
      >
        {{ props.hint }}
      </p>
    </div>
  </Transition>
</template>

<style scoped>
.c-field__native-proxy {
  position: absolute;
  width: 1px;
  height: 1px;
  opacity: 0;
  pointer-events: none;
}

.field-feedback-wrap {
  display: grid;
  grid-template-rows: 1fr;
  width: 100%;
}

.field-feedback-wrap > .field-feedback {
  overflow: hidden;
}

.field-feedback {
  padding-top: var(--space-1);
}

.field-feedback-enter-active,
.field-feedback-leave-active {
  transition: grid-template-rows 0.28s ease, opacity 0.25s ease;
}

.field-feedback-enter-from,
.field-feedback-leave-to {
  grid-template-rows: 0fr;
  opacity: 0;
}
</style>
