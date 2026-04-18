import { computed, nextTick, ref, toValue, watch, type MaybeRefOrGetter, type WatchSource } from 'vue'

type ValidationError = string | undefined

interface UseFieldValidationOptions {
  externalError: MaybeRefOrGetter<ValidationError>
  readonly?: MaybeRefOrGetter<boolean | undefined>
  getInternalError: () => ValidationError
  watchSources?: WatchSource<unknown>[]
}

export function useFieldValidation(options: UseFieldValidationOptions) {
  const touched = ref(false)
  const internalError = ref<ValidationError>(undefined)

  const displayError = computed(() => toValue(options.externalError) ?? internalError.value)

  function recomputeInternalError() {
    if (!touched.value || toValue(options.readonly)) {
      internalError.value = undefined
      return
    }

    if (toValue(options.externalError)) {
      internalError.value = undefined
      return
    }

    internalError.value = options.getInternalError()
  }

  function onFieldBlur() {
    touched.value = true
    recomputeInternalError()
  }

  function onFieldInput() {
    if (!touched.value || toValue(options.externalError)) return
    void nextTick(recomputeInternalError)
  }

  if (options.watchSources?.length) {
    watch(options.watchSources, () => {
      if (!touched.value || toValue(options.externalError)) return
      void nextTick(recomputeInternalError)
    })
  }

  return {
    touched,
    displayError,
    recomputeInternalError,
    onFieldBlur,
    onFieldInput,
  }
}
