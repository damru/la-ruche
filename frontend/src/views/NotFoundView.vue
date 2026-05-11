<script setup lang="ts">
import { useI18n } from 'vue-i18n'
const { t } = useI18n()
// TODO: implement 404 view
</script>

<template>
  <div class="view-stub">
    {{ t('errors.not_found') }}
  </div>
</template>
