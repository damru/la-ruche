<script setup lang="ts">
// Composant organique — AppNav
// DDR-003 : bottom nav mobile, sidebar desktop
// Items filtered by role

import { useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useI18n } from 'vue-i18n'
import type { Role } from '@/types/domain'
import CIconHome from '@/components/atoms/CIconHome.vue'
import CIconDot from '@/components/atoms/CIconDot.vue'

const { t } = useI18n()
const route = useRoute()
const auth = useAuthStore()

interface NavItem {
  name: string
  route: string
  label: string
  roles: Role[]    // vide = tous
  mobileOrder?: number
}

const allItems: NavItem[] = [
  { name: 'dashboard',     route: '/dashboard',     label: t('nav.dashboard'),     roles: [],                                         mobileOrder: 1 },
  { name: 'assemblies',    route: '/meetings',      label: t('nav.meetings'),      roles: ['manager','board_member','co_owner'],        mobileOrder: 2 },
  { name: 'budget',        route: '/budget',        label: t('nav.budget'),        roles: ['manager','board_member','co_owner'],        mobileOrder: 3 },
  { name: 'documents',     route: '/documents',     label: t('nav.documents'),     roles: ['manager','board_member','co_owner'],        mobileOrder: 5 },
  { name: 'tasks',         route: '/tasks',         label: t('nav.tasks'),         roles: ['manager','board_member'],                   mobileOrder: 4 },
  { name: 'contracts',     route: '/contracts',     label: t('nav.contracts'),     roles: ['manager'],                                  mobileOrder: 6 },
  { name: 'claims',        route: '/claims',        label: t('nav.claims'),        roles: ['manager','board_member'],                   mobileOrder: 7 },
  { name: 'maintenance',   route: '/logbook',       label: t('nav.maintenance'),   roles: ['manager','board_member'],                   mobileOrder: 8 },
  { name: 'consultations', route: '/consultations', label: t('nav.consultations'), roles: [],                                         mobileOrder: 4 },
  { name: 'reports',       route: '/reports',       label: t('nav.reports'),       roles: [],                                         mobileOrder: 4 },
  { name: 'residents',     route: '/residents',     label: t('nav.residents'),     roles: ['manager'],                                  mobileOrder: 5 },
  { name: 'settings',      route: '/settings',      label: t('nav.settings'),      roles: ['manager'] },
]

const visibleItems = computed(() =>
  allItems.filter(item =>
    item.roles.length === 0 || auth.hasAnyRole(item.roles)
  )
)

// Mobile: up to 4 items based on role
const mobileItems = computed(() =>
  visibleItems.value
    .filter(i => i.mobileOrder !== undefined)
    .sort((a, b) => (a.mobileOrder ?? 99) - (b.mobileOrder ?? 99))
    .slice(0, 4)
)

function isActive(item: NavItem): boolean {
  return route.path.startsWith(item.route)
}
</script>

<template>
  <!-- ── Sidebar desktop ──────────────────────────────────────── -->
  <aside class="c-sidebar" role="navigation" :aria-label="t('nav.dashboard')">
    <!-- Logo -->
    <div class="c-sidebar__logo">
      <div class="c-sidebar__logo-mark" aria-hidden="true">
        <CIconHome stroke="white" stroke-width="2.5"/>
      </div>
      <div>
        <div class="c-sidebar__logo-name">La Ruche</div>
        <div class="c-sidebar__logo-sub">Gestion participative</div>
      </div>
    </div>

    <!-- Items -->
    <nav class="c-sidebar__nav">
      <router-link
        v-for="item in visibleItems"
        :key="item.name"
        :to="item.route"
        class="c-sidebar__item"
        :class="{ 'c-sidebar__item--active': isActive(item) }"
        :aria-current="isActive(item) ? 'page' : undefined"
      >
        <span class="c-sidebar__item-icon" aria-hidden="true">
          <slot :name="`icon-${item.name}`">
            <CIconDot/>
          </slot>
        </span>
        <span class="c-sidebar__item-label">{{ item.label }}</span>
      </router-link>
    </nav>

    <div class="c-sidebar__spacer" />

    <!-- Utilisateur -->
    <div class="c-sidebar__user">
      <div class="c-sidebar__user-avatar" aria-hidden="true">
        {{ auth.user?.first_name?.[0] }}{{ auth.user?.last_name?.[0] }}
      </div>
      <div class="c-sidebar__user-info">
        <div class="c-sidebar__user-name">{{ auth.user?.first_name }} {{ auth.user?.last_name }}</div>
        <div class="c-sidebar__user-role">{{ t(`roles.${auth.user?.roles[0]}`) }}</div>
      </div>
    </div>
  </aside>

  <!-- ── Bottom nav mobile ─────────────────────────────────────── -->
  <nav class="c-bottom-nav" role="navigation" :aria-label="'Navigation principale'">
    <router-link
      v-for="item in mobileItems"
      :key="item.name"
      :to="item.route"
      class="c-bottom-nav__item"
      :class="{ 'c-bottom-nav__item--active': isActive(item) }"
      :aria-current="isActive(item) ? 'page' : undefined"
      :aria-label="item.label"
    >
      <div class="c-bottom-nav__icon-wrap">
        <slot :name="`icon-${item.name}`">
          <CIconDot/>
        </slot>
      </div>
      <span class="c-bottom-nav__label">{{ item.label }}</span>
    </router-link>
  </nav>
</template>

<style scoped>
/* ── Sidebar ─────────────────────────────────────────────────── */
.c-sidebar {
  width: var(--nav-sidebar-width);
  height: 100vh;
  position: sticky;
  top: 0;
  background: var(--color-surface-container-lowest);
  box-shadow: var(--shadow-md);
  display: flex;
  flex-direction: column;
  padding: var(--space-5) var(--space-3);
  flex-shrink: 0;
  overflow-y: auto;
  z-index: var(--z-nav);
}

@media (max-width: 1024px) { .c-sidebar { display: none; } }

.c-sidebar__logo {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: 0 var(--space-2) var(--space-5);
}

.c-sidebar__logo-mark {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  background: var(--gradient-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.c-sidebar__logo-mark svg { width: 18px; height: 18px; }

.c-sidebar__logo-name {
  font-family: var(--font-display);
  font-size: 14px;
  font-weight: 700;
  font-variation-settings: 'opsz' 20;
  color: var(--color-on-surface);
  letter-spacing: -0.01em;
}

.c-sidebar__logo-sub {
  font-size: 10px;
  color: var(--color-on-surface-variant);
  margin-top: 1px;
}

.c-sidebar__nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.c-sidebar__item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: 9px var(--space-3);
  border-radius: var(--radius-sm);
  text-decoration: none;
  transition: background var(--transition-fast);
  color: var(--color-on-surface-variant);
}

.c-sidebar__item:hover { background: var(--color-surface-container-low); color: var(--color-on-surface); }

.c-sidebar__item--active {
  background: var(--color-secondary-container);
  color: var(--color-on-secondary-container);
}
.c-sidebar__item--active .c-sidebar__item-icon { color: var(--color-primary); }

.c-sidebar__item-icon { width: 18px; height: 18px; flex-shrink: 0; }
.c-sidebar__item-icon :deep(svg) { width: 18px; height: 18px; }

.c-sidebar__item-label { font-size: 13px; font-weight: 500; }

.c-sidebar__spacer { flex: 1; }

.c-sidebar__user {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3);
  border-radius: var(--radius-sm);
  background: var(--color-surface-container-low);
}

.c-sidebar__user-avatar {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-full);
  background: var(--gradient-primary);
  color: #fff;
  font-family: var(--font-display);
  font-size: 12px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.c-sidebar__user-name { font-size: 12px; font-weight: 500; color: var(--color-on-surface); }
.c-sidebar__user-role { font-size: 10px; color: var(--color-primary); font-weight: 500; }

/* ── Bottom nav ───────────────────────────────────────────────── */
.c-bottom-nav {
  display: none;
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  background: var(--color-surface-container-lowest);
  border-top: 1px solid rgba(194, 199, 207, 0.2);
  padding-bottom: var(--nav-safe-area-bottom);
  z-index: var(--z-nav);
  box-shadow: 0 -4px 20px rgba(25, 28, 29, 0.06);
}

@media (max-width: 1024px) { .c-bottom-nav { display: flex; } }

.c-bottom-nav__item {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  padding: var(--space-2) var(--space-1) 10px;
  text-decoration: none;
  transition: color var(--transition-fast);
  color: var(--color-on-surface-variant);
  border: none;
  background: none;
  cursor: pointer;
}

.c-bottom-nav__icon-wrap {
  padding: 4px 16px;
  border-radius: var(--radius-full);
  transition: background var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
}
.c-bottom-nav__icon-wrap :deep(svg) { width: 22px; height: 22px; }

.c-bottom-nav__item--active { color: var(--color-primary); }
.c-bottom-nav__item--active .c-bottom-nav__icon-wrap {
  background: var(--color-secondary-container);
}
.c-bottom-nav__item:hover:not(.c-bottom-nav__item--active) .c-bottom-nav__icon-wrap {
  background: var(--color-surface-container-low);
}

.c-bottom-nav__label {
  font-size: 11px;
  font-weight: 500;
}
.c-bottom-nav__item--active .c-bottom-nav__label { font-weight: 600; }
</style>
