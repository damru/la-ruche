<script setup lang="ts">
// Bannière d'avertissement — mode démonstration
// Données 100% fictives, aucune connexion au backend.
</script>

<template>
  <div
    class="mock-banner"
    role="status"
    aria-live="polite"
  >
    <CIconLayers aria-hidden="true" />
    <span>
      <strong>Mode démonstration</strong> — Toutes les données sont fictives.
    </span>
  </div>
</template>

<style scoped>
.mock-banner {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 10px var(--space-4);
  background: repeating-linear-gradient(
    -45deg,
    #7c3aed18 0px,
    #7c3aed18 10px,
    transparent 10px,
    transparent 20px
  ),
  linear-gradient(135deg, #7c3aed14, #a855f714);
  border: 1.5px solid #7c3aed40;
  border-radius: var(--radius-md);
  color: #5b21b6;
  font-size: 13px;
  margin-bottom: var(--space-4);
}

[data-theme="dark"] .mock-banner {
  background: repeating-linear-gradient(
    -45deg,
    #a78bfa18 0px,
    #a78bfa18 10px,
    transparent 10px,
    transparent 20px
  ),
  linear-gradient(135deg, #7c3aed20, #a855f720);
  border-color: #7c3aed50;
  color: #c4b5fd;
}

.mock-banner svg {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
  opacity: 0.8;
}
</style>
