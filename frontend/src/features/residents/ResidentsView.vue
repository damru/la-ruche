<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import CMockBanner from '@/components/atoms/CMockBanner.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CIconChevronRight from '@/components/atoms/CIconChevronRight.vue'
import CIconPlus from '@/components/atoms/CIconPlus.vue'
import CIconSearch from '@/components/atoms/CIconSearch.vue'
import { mockUnits, mockUsers, mockBuildingStats } from '@/mock/data'

const { t } = useI18n()

const activeTab = ref<'units' | 'users'>('units')
const search = ref('')

const units = computed(() =>
  search.value
    ? mockUnits.filter(u => u.unit_number.toLowerCase().includes(search.value.toLowerCase()) || (u.current_owner?.last_name ?? '').toLowerCase().includes(search.value.toLowerCase()))
    : mockUnits
)

const users = computed(() =>
  search.value
    ? mockUsers.filter(u => `${u.first_name} ${u.last_name}`.toLowerCase().includes(search.value.toLowerCase()))
    : mockUsers
)

function unitTypeIcon(type: string): string {
  const icons: Record<string, string> = { apartment: '🏠', cellar: '🗄️', parking: '🚗', commercial_unit: '🏪', other: '📦' }
  return icons[type] ?? '📦'
}

function unitTypeLabel(type: string): string {
  const labels: Record<string, string> = { apartment: 'Appartement', cellar: 'Cave', parking: 'Parking', commercial_unit: 'Local commercial', other: 'Autre' }
  return labels[type] ?? type
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ mockBuildingStats.name }}
          <CIconChevronRight width="14" height="14"/>
          <span>Résidents &amp; Lots</span>
        </div>
      </template>
      <template #right>
        <CBtn variant="primary" size="sm">
          <CIconPlus style="width:16px;height:16px"/>
          Inviter un résident
        </CBtn>
      </template>
    </CTopbar>

    <div class="view-container">
    <CMockBanner />

    <div class="view-header">
      <div>
        <h1 class="display-md">Résidents &amp; Lots</h1>
        <p class="body-md" style="color:var(--color-on-surface-variant)">
          {{ mockUnits.length }} lots · {{ mockUsers.length }} utilisateurs
        </p>
      </div>
    </div>

    <div class="tab-bar">
      <button v-for="tab in [{ key: 'units', label: `Lots (${mockUnits.length})` }, { key: 'users', label: `Utilisateurs (${mockUsers.length})` }]"
        :key="tab.key" :class="['tab-btn', { 'tab-btn--active': activeTab === tab.key }]"
        @click="activeTab = tab.key as typeof activeTab"
      >{{ tab.label }}</button>
    </div>

    <div class="search-bar">
      <CIconSearch class="search-icon"/>
      <input v-model="search" type="search" :placeholder="activeTab === 'units' ? 'Rechercher un lot…' : 'Rechercher un utilisateur…'" class="search-input" />
    </div>

    <!-- Lots -->
    <template v-if="activeTab === 'units'">
      <CCard padding="none">
        <table class="data-table">
          <thead>
            <tr>
              <th>Lot</th>
              <th>Type</th>
              <th>Emplacement</th>
              <th>Propriétaire actuel</th>
              <th>Statut</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="unit in units" :key="unit.id">
              <td><strong>{{ unit.unit_number }}</strong></td>
              <td>{{ unitTypeIcon(unit.unit_type) }} {{ unitTypeLabel(unit.unit_type) }}</td>
              <td class="dim">
                <span v-if="unit.staircase">Cage {{ unit.staircase }}</span>
                <span v-if="unit.floor">, {{ unit.floor }}</span>
              </td>
              <td>
                <span v-if="unit.current_owner">{{ unit.current_owner.first_name }} {{ unit.current_owner.last_name }}</span>
                <span v-else class="dim">Vacant</span>
              </td>
              <td>
                <CBadge :variant="unit.active ? 'open' : 'closed'">{{ unit.active ? 'Actif' : 'Inactif' }}</CBadge>
              </td>
            </tr>
          </tbody>
        </table>
      </CCard>
    </template>

    <!-- Utilisateurs -->
    <template v-if="activeTab === 'users'">
      <CCard padding="none">
        <table class="data-table">
          <thead>
            <tr>
              <th>Nom</th>
              <th>Email</th>
              <th>Rôle(s)</th>
              <th>Statut</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="user in users" :key="user.id">
              <td>
                <div class="user-name">
                  <div class="avatar">{{ user.first_name[0] }}{{ user.last_name[0] }}</div>
                  <span>{{ user.first_name }} {{ user.last_name }}</span>
                </div>
              </td>
              <td class="dim">{{ user.email }}</td>
              <td>
                <div class="roles">
                  <CBadge v-for="role in user.roles" :key="role" :variant="role">{{ t(`roles.${role}`) }}</CBadge>
                </div>
              </td>
              <td><CBadge :variant="user.active ? 'open' : 'closed'">{{ user.active ? 'Actif' : 'Désactivé' }}</CBadge></td>
            </tr>
          </tbody>
        </table>
      </CCard>
    </template>
  </div>
  </div>
</template>

<style scoped>
.view-container { padding: 40px; max-width: 1100px; display: flex; flex-direction: column; gap: 24px; }
.view-header { display: flex; align-items: flex-start; justify-content: space-between; }

.tab-bar { display: flex; gap: 4px; border-bottom: 2px solid var(--color-outline-variant); }
.tab-btn {
  padding: 8px 20px; background: none; border: none;
  font-size: 14px; font-weight: 500; cursor: pointer;
  color: var(--color-on-surface-variant);
  border-bottom: 2px solid transparent; margin-bottom: -2px;
}
.tab-btn--active { color: var(--color-primary); border-bottom-color: var(--color-primary); }

.search-bar {
  display: flex; align-items: center; gap: 10px;
  padding: 10px 16px;
  background: var(--color-surface-container-lowest);
  border: 1.5px solid var(--color-outline-variant);
  border-radius: var(--radius-md);
}
.search-icon { width: 16px; height: 16px; color: var(--color-on-surface-variant); flex-shrink: 0; }
.search-input { flex: 1; background: none; border: none; outline: none; font-size: 14px; color: var(--color-on-surface); }

.data-table { width: 100%; border-collapse: collapse; }
.data-table th {
  padding: 10px 16px; text-align: left; font-size: 12px; font-weight: 600;
  text-transform: uppercase; letter-spacing: 0.05em;
  color: var(--color-on-surface-variant);
  border-bottom: 1px solid var(--color-outline-variant);
}
.data-table td { padding: 10px 16px; font-size: 13px; border-bottom: 1px solid var(--color-surface-container-low); vertical-align: middle; }
.data-table tr:last-child td { border-bottom: none; }
.dim { color: var(--color-on-surface-variant); }

.user-name { display: flex; align-items: center; gap: 10px; }
.avatar {
  width: 32px; height: 32px; border-radius: 50%;
  background: var(--gradient-primary); color: #fff;
  display: flex; align-items: center; justify-content: center;
  font-size: 12px; font-weight: 700; flex-shrink: 0;
}

.roles { display: flex; flex-wrap: wrap; gap: 4px; }
</style>
