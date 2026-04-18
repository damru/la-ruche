<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import CMockBanner from '@/components/atoms/CMockBanner.vue'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CIconChevronRight from '@/components/atoms/CIconChevronRight.vue'
import { mockBuilding } from '@/mock/data'

const { t } = useI18n()

const form = ref({ ...mockBuilding })
const activeTab = ref<'building' | 'notifications' | 'account'>('building')
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ form.name }}
          <CIconChevronRight width="14" height="14"/>
          <span>{{ t('nav.settings') }}</span>
        </div>
      </template>
    </CTopbar>

    <div class="view-container">
    <CMockBanner />

    <div class="view-header">
      <div>
        <h1 class="display-md">{{ t('nav.settings') }}</h1>
        <p class="body-md" style="color:var(--color-on-surface-variant)">Configuration de la résidence</p>
      </div>
    </div>

    <div class="tab-bar">
      <button v-for="tab in [{ key: 'building', label: 'Bâtiment' }, { key: 'notifications', label: 'Notifications' }, { key: 'account', label: 'Mon compte' }]"
        :key="tab.key" :class="['tab-btn', { 'tab-btn--active': activeTab === tab.key }]"
        @click="activeTab = tab.key as typeof activeTab"
      >{{ tab.label }}</button>
    </div>

    <!-- Bâtiment -->
    <template v-if="activeTab === 'building'">
      <CCard>
        <h2 class="section-title">Informations de la résidence</h2>
        <div class="form-grid">
          <div class="field">
            <label class="field-label">Nom de la résidence</label>
            <input v-model="form.name" class="field-input" />
          </div>
          <div class="field">
            <label class="field-label">SIRET syndic</label>
            <input v-model="form.manager_siret" class="field-input" />
          </div>
          <div class="field field--full">
            <label class="field-label">Adresse</label>
            <input v-model="form.address_line1" class="field-input" />
          </div>
          <div class="field">
            <label class="field-label">Code postal</label>
            <input v-model="form.postal_code" class="field-input" />
          </div>
          <div class="field">
            <label class="field-label">Ville</label>
            <input v-model="form.city" class="field-input" />
          </div>
          <div class="field">
            <label class="field-label">Total tantièmes</label>
            <input v-model="form.shares_total" type="number" class="field-input" />
          </div>
          <div class="field">
            <label class="field-label">Date du règlement de copropriété</label>
            <input v-model="form.bylaws_date" type="date" class="field-input" />
          </div>
        </div>
        <div class="form-actions">
          <CBtn variant="primary">Enregistrer les modifications</CBtn>
        </div>
      </CCard>

      <CCard>
        <h2 class="section-title">Données légales</h2>
        <p class="body-sm" style="color:var(--color-on-surface-variant); margin-bottom:12px">
          ⚖️ <strong>Art. 18 loi 10/07/1965</strong> — Le syndic est tenu de tenir à jour
          les informations de la copropriété et de les communiquer à tout copropriétaire qui en fait la demande.
        </p>
        <div class="form-actions">
          <CBtn variant="secondary">📥 Exporter les données</CBtn>
          <CBtn variant="secondary">📋 Télécharger le règlement</CBtn>
        </div>
      </CCard>
    </template>

    <!-- Notifications -->
    <template v-if="activeTab === 'notifications'">
      <CCard>
        <h2 class="section-title">Préférences de notification</h2>
        <div class="notif-list">
          <div v-for="item in [
            { label: 'Rappels de contrats', desc: 'Recevoir un email avant l\'échéance des contrats', enabled: true },
            { label: 'Nouveaux signalements', desc: 'Être notifié lorsqu\'un résident soumet un signalement', enabled: true },
            { label: 'Appels de fonds en retard', desc: 'Alerte quand un appel de fonds passe en statut retard', enabled: true },
            { label: 'Consultations ouvertes', desc: 'Email aux résidents lors d\'une nouvelle consultation', enabled: false },
            { label: 'Récapitulatif hebdomadaire', desc: 'Synthèse hebdomadaire des activités de la résidence', enabled: false },
          ]" :key="item.label" class="notif-item">
            <div class="notif-info">
              <span class="title-sm">{{ item.label }}</span>
              <p class="body-sm" style="color:var(--color-on-surface-variant)">{{ item.desc }}</p>
            </div>
            <div :class="['toggle', { 'toggle--on': item.enabled }]">
              <div class="toggle-thumb" />
            </div>
          </div>
        </div>
      </CCard>
    </template>

    <!-- Mon compte -->
    <template v-if="activeTab === 'account'">
      <CCard>
        <h2 class="section-title">Informations personnelles</h2>
        <div class="form-grid">
          <div class="field">
            <label class="field-label">Prénom</label>
            <input value="Sophie" class="field-input" />
          </div>
          <div class="field">
            <label class="field-label">Nom</label>
            <input value="Martin" class="field-input" />
          </div>
          <div class="field field--full">
            <label class="field-label">Email</label>
            <input value="sophie.martin@syndic-elite.fr" class="field-input" />
          </div>
        </div>
        <div class="form-actions">
          <CBtn variant="primary">Mettre à jour</CBtn>
          <CBtn variant="ghost">Changer le mot de passe</CBtn>
        </div>
      </CCard>

      <CCard>
        <h2 class="section-title" style="color:var(--color-error)">Zone dangereuse</h2>
        <p class="body-sm" style="color:var(--color-on-surface-variant);margin-bottom:12px">Ces actions sont irréversibles.</p>
        <CBtn variant="danger">Réinitialiser l'application</CBtn>
      </CCard>
    </template>
  </div>
  </div>
</template>

<style scoped>
.view-container { padding: 40px; max-width: 800px; display: flex; flex-direction: column; gap: 24px; }
.view-header { display: flex; align-items: flex-start; justify-content: space-between; }

.tab-bar { display: flex; gap: 4px; border-bottom: 2px solid var(--color-outline-variant); }
.tab-btn {
  padding: 8px 20px; background: none; border: none;
  font-size: 14px; font-weight: 500; cursor: pointer;
  color: var(--color-on-surface-variant);
  border-bottom: 2px solid transparent; margin-bottom: -2px;
}
.tab-btn--active { color: var(--color-primary); border-bottom-color: var(--color-primary); }

.section-title {
  font-family: var(--font-display);
  font-size: 16px; font-weight: 600;
  margin-bottom: 16px;
}

.form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 20px; }
.field { display: flex; flex-direction: column; gap: 6px; }
.field--full { grid-column: 1 / -1; }
.field-label { font-size: 13px; font-weight: 500; color: var(--color-on-surface-variant); }
.field-input {
  padding: 8px 12px;
  border: 1.5px solid var(--color-outline-variant);
  border-radius: var(--radius-sm);
  background: var(--color-surface-container-lowest);
  color: var(--color-on-surface);
  font-size: 14px;
  outline: none;
  transition: border-color var(--transition-fast);
}
.field-input:focus { border-color: var(--color-primary); }

.form-actions { display: flex; gap: 10px; }

.notif-list { display: flex; flex-direction: column; gap: 16px; }
.notif-item { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 12px 0; border-bottom: 1px solid var(--color-surface-container-low); }
.notif-item:last-child { border-bottom: none; }
.notif-info { flex: 1; }

.toggle {
  width: 44px; height: 24px; border-radius: var(--radius-full);
  background: var(--color-outline-variant);
  position: relative; cursor: pointer; flex-shrink: 0;
  transition: background var(--transition-fast);
}
.toggle--on { background: var(--color-primary); }
.toggle-thumb {
  position: absolute; top: 3px; left: 3px;
  width: 18px; height: 18px; border-radius: 50%;
  background: white;
  transition: transform var(--transition-fast);
}
.toggle--on .toggle-thumb { transform: translateX(20px); }
</style>
