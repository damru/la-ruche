<script setup lang="ts">
// Composant organique — AppNav
// DDR-003 : bottom nav mobile, sidebar desktop
// DDR-004 : groupes visuels (pattern B) + dropdown profil
// Items filtered by role

import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useI18n } from 'vue-i18n'
import type { Role } from '@/types/domain'

const { t } = useI18n()
const route = useRoute()
const auth = useAuthStore()

interface NavItem {
  name: string
  route: string
  label: string
  mobileLabel?: string
  roles: Role[]    // empty = all authenticated users
  mobileOrder?: number
}

interface NavGroup {
  label?: string   // undefined = no section header
  items: NavItem[]
}

const allGroups: NavGroup[] = [
  {
    items: [
      { name: 'dashboard', route: '/dashboard', label: t('nav.dashboard'), mobileLabel: t('nav.mobile.dashboard'), roles: [], mobileOrder: 1 },
      { name: 'calendar', route: '/calendar', label: t('nav.calendar'), mobileLabel: t('calendar.modes.agenda'), roles: [], mobileOrder: 9 },
    ],
  },
  {
    label: t('nav.governance'),
    items: [
      { name: 'assemblies',    route: '/meetings',      label: t('nav.meetings'),      mobileLabel: t('meetings.typeLabel.annualShort'),    roles: ['manager', 'board_member', 'co_owner'], mobileOrder: 10 },
      { name: 'consultations', route: '/consultations', label: t('nav.consultations'), mobileLabel: t('nav.mobile.consultations'), roles: [],                                      mobileOrder: 4 },
      { name: 'reports',       route: '/reports',       label: t('nav.reports'),       mobileLabel: t('nav.reports'),       roles: [],                                      mobileOrder: 2 },
    ],
  },
  {
    label: t('nav.groups.finance'),
    items: [
      { name: 'budget',      route: '/budget',      label: t('nav.budget'),      mobileLabel: t('nav.mobile.budget'),      roles: ['manager', 'board_member', 'co_owner'], mobileOrder: 11 },
      { name: 'my-charges',  route: '/my-charges',  label: t('nav.myCharges'),   mobileLabel: t('nav.myCharges'),           roles: ['co_owner'],                            mobileOrder: 3 },
      { name: 'contracts',   route: '/contracts',   label: t('nav.contracts'),   mobileLabel: t('nav.contracts'),   roles: ['manager'],                             mobileOrder: 6 },
    ],
  },
  {
    label: t('nav.groups.operations'),
    items: [
      { name: 'tasks',        route: '/tasks',        label: t('nav.tasks'),        mobileLabel: t('nav.tasks'),        roles: ['manager', 'board_member'], mobileOrder: 4 },
      { name: 'contractors',  route: '/contractors',  label: t('contractors.title'),  mobileLabel: t('contractors.title'),  roles: [],                          mobileOrder: 3 },
      { name: 'claims',       route: '/claims',       label: t('nav.claims'),       mobileLabel: t('nav.claims'),       roles: ['manager', 'board_member'], mobileOrder: 7 },
      { name: 'maintenance', route: '/logbook',   label: t('nav.maintenance'), mobileLabel: t('nav.mobile.maintenance'), roles: ['manager', 'board_member'], mobileOrder: 8 },
      { name: 'documents',   route: '/documents', label: t('common.documents'),   mobileLabel: t('nav.mobile.documents'),   roles: ['manager', 'board_member', 'co_owner', 'tenant'], mobileOrder: 5 },
    ],
  },
  {
    label: t('nav.groups.residence'),
    items: [
      { name: 'residence-building', route: '/residence/building', label: t('common.informations'), roles: ['manager'] },
      { name: 'residence-keys',     route: '/residence/keys',     label: t('allocationKeys.title'), roles: ['manager'] },
      { name: 'registry',           route: '/registry',           label: t('nav.registry'),       roles: ['manager', 'board_member', 'co_owner'] },
      { name: 'judicial-proceedings', route: '/judicial-proceedings', label: t('nav.judicialProceedings'), roles: ['manager', 'board_member', 'co_owner'] },
      { name: 'governance',         route: '/governance',         label: t('nav.governance'),     roles: ['manager', 'board_member', 'co_owner'] },
      { name: 'etat-date',          route: '/etat-date',          label: t('registers.etatDate.title'),       roles: ['manager'] },
      { name: 'residents',          route: '/residents',          label: t('nav.residents'),      roles: ['manager'] },
    ],
  },
]

const visibleGroups = computed(() =>
  allGroups
    .map(group => ({
      ...group,
      items: group.items.filter(item =>
        item.roles.length === 0 || auth.hasAnyRole(item.roles)
      ),
    }))
    .filter(group => group.items.length > 0)
)

// Mobile: up to 4 items based on role (flattened from all groups)
const mobileItems = computed(() =>
  allGroups
    .flatMap(g => g.items)
    .filter(item => item.mobileOrder !== undefined && (item.roles.length === 0 || auth.hasAnyRole(item.roles)))
    .sort((a, b) => (a.mobileOrder ?? 99) - (b.mobileOrder ?? 99))
    .slice(0, 4)
)

function isActive(item: NavItem): boolean {
  return route.path.startsWith(item.route)
}

// ── Mobile drawer ────────────────────────────────────────────────
const drawerOpen = ref(false)

function toggleDrawer() {
  drawerOpen.value = !drawerOpen.value
}

function closeDrawer() {
  drawerOpen.value = false
}

// Close drawer when navigating
watch(() => route.path, closeDrawer)
</script>

<template>
  <!-- ── Sidebar desktop ──────────────────────────────────────── -->
  <aside
    class="c-sidebar"
    role="navigation"
    :aria-label="t('nav.primary')"
  >
    <!-- Logo -->
    <div class="c-sidebar__logo">
      <div
        class="c-sidebar__logo-mark"
        aria-hidden="true"
      >
        <CLogoMark />
      </div>
      <div>
        <div class="c-sidebar__logo-name">
          {{ t('nav.appName') }}
        </div>
        <div class="c-sidebar__logo-sub">
          {{ t('auth.brandEyebrow') }}
        </div>
      </div>
    </div>

    <!-- Groups -->
    <nav class="c-sidebar__nav">
      <template
        v-for="(group, gi) in visibleGroups"
        :key="gi"
      >
        <!-- Section label -->
        <div
          v-if="group.label"
          class="c-sidebar__group-label"
        >
          {{ group.label }}
        </div>
        <!-- Nav items -->
        <router-link
          v-for="item in group.items"
          :key="item.name"
          :to="item.route"
          class="c-sidebar__item"
          :class="{ 'c-sidebar__item--active': isActive(item) }"
          :aria-current="isActive(item) ? 'page' : undefined"
        >
          <span
            class="c-sidebar__item-icon"
            aria-hidden="true"
          >
            <slot :name="`icon-${item.name}`">
              <CIconDot />
            </slot>
          </span>
          <span class="c-sidebar__item-label">{{ item.label }}</span>
        </router-link>
      </template>
    </nav>

    <div class="c-sidebar__spacer"></div>

    <div class="c-sidebar__footer">
      <!-- User section — links to profile page -->
      <router-link
        to="/profile"
        class="c-sidebar__user"
        :class="{ 'c-sidebar__user--open': route.path.startsWith('/profile') }"
      >
        <div
          class="c-sidebar__user-avatar"
          aria-hidden="true"
        >
          {{ auth.user?.first_name?.[0] }}{{ auth.user?.last_name?.[0] }}
        </div>
        <div class="c-sidebar__user-info">
          <div class="text-sm font-medium text-on-surface">
            {{ auth.user?.first_name }} {{ auth.user?.last_name }}
          </div>
          <div class="text-xs font-medium text-primary">
            {{ t(`roles.${auth.user?.roles[0]}`) }}
          </div>
        </div>
        <CIconUser
          class="icon-xs"
          aria-hidden="true"
        />
      </router-link>
    </div>
  </aside>

  <!-- ── Bottom nav mobile ─────────────────────────────────────── -->
  <nav
    class="c-bottom-nav"
    role="navigation"
    :aria-label="t('nav.primary')"
  >
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
          <CIconDot />
        </slot>
      </div>
      <span class="c-bottom-nav__label">{{ item.mobileLabel ?? item.label }}</span>
    </router-link>

    <!-- More button -->
    <button
      class="c-bottom-nav__item"
      :class="{ 'c-bottom-nav__item--active': drawerOpen }"
      :aria-label="t('nav.more')"
      aria-controls="mobile-nav-drawer"
      :aria-expanded="drawerOpen"
      @click="toggleDrawer"
    >
      <div class="c-bottom-nav__icon-wrap">
        <CIconDotsHorizontal />
      </div>
      <span class="c-bottom-nav__label">{{ t('nav.more') }}</span>
    </button>

    <router-link
      to="/profile"
      class="c-bottom-nav__item c-bottom-nav__item--profile"
      :class="{ 'c-bottom-nav__item--active': route.path.startsWith('/profile') }"
      :aria-label="auth.user?.first_name || t('nav.account')"
    >
      <div
        class="c-bottom-nav__avatar"
        aria-hidden="true"
      >
        {{ auth.user?.first_name?.[0] }}{{ auth.user?.last_name?.[0] }}
      </div>
      <span class="c-bottom-nav__label">{{ auth.user?.first_name || t('nav.account') }}</span>
    </router-link>
  </nav>

  <!-- ── Nav drawer (mobile) ────────────────────────────────────── -->
  <Teleport to="body">
    <Transition name="c-drawer-fade">
      <div
        v-if="drawerOpen"
        class="c-nav-drawer-overlay"
        @click="closeDrawer"
      ></div>
    </Transition>
    <div
      id="mobile-nav-drawer"
      class="c-nav-drawer"
      :class="{ 'c-nav-drawer--open': drawerOpen }"
      role="navigation"
      :aria-label="t('nav.more')"
      :aria-hidden="!drawerOpen"
    >
      <div
        class="c-nav-drawer__handle-area"
        @click="closeDrawer"
      >
        <div class="c-nav-drawer__handle"></div>
      </div>
      <nav class="c-nav-drawer__nav">
        <template
          v-for="(group, gi) in visibleGroups"
          :key="gi"
        >
          <div
            v-if="group.label"
            class="c-nav-drawer__group-label"
          >
            {{ group.label }}
          </div>
          <router-link
            v-for="item in group.items"
            :key="item.name"
            :to="item.route"
            class="c-nav-drawer__item"
            :class="{ 'c-nav-drawer__item--active': isActive(item) }"
            :aria-current="isActive(item) ? 'page' : undefined"
            @click="closeDrawer"
          >
            <span
              class="c-nav-drawer__item-icon"
              aria-hidden="true"
            >
              <slot :name="`icon-${item.name}`"><CIconDot /></slot>
            </span>
            {{ item.label }}
          </router-link>
        </template>
      </nav>
    </div>
  </Teleport>
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
  overflow: visible;
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
  flex-shrink: 0;
}
.c-sidebar__logo-mark svg { width: 32px; height: 32px; }

.c-sidebar__logo-name {
  font-family: var(--font-display);
  font-size: var(--text-md);
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
  overflow-y: auto;
  flex: 1;
  min-height: 0;
}

/* Section group label */
.c-sidebar__group-label {
  font-size: 10px;
  font-weight: var(--fw-medium, 500);
  letter-spacing: 0.07em;
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
  padding: 14px var(--space-3) var(--space-1);
  opacity: 0.6;
}

.c-sidebar__item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: 9px var(--space-3);
  border-radius: var(--radius-lg);
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

.c-sidebar__item-icon { width: var(--icon-md); height: var(--icon-md); flex-shrink: 0; }
.c-sidebar__item-icon :deep(svg) { width: var(--icon-md); height: var(--icon-md); }

.c-sidebar__item-label { font-size: 13px; font-weight: 500; }

.c-sidebar__spacer { flex: 0; }

/* Profile footer */
.c-sidebar__footer {
  flex-shrink: 0;
  margin-top: var(--space-3);
}

/* User link */
.c-sidebar__user {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3);
  border-radius: var(--radius-lg);
  background: var(--color-surface-container-low);
  border: none;
  cursor: pointer;
  width: 100%;
  text-align: left;
  transition: background var(--transition-fast);
}
.c-sidebar__user:hover,
.c-sidebar__user--open { background: var(--color-surface-container-high); }

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

.c-sidebar__user-info { flex: 1; min-width: 0; }

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
  padding: var(--space-1) var(--space-4);
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
  text-align: center;
  width: 100%;
}
.c-bottom-nav__item--active .c-bottom-nav__label { font-weight: 600; }

/* Profile avatar item */
.c-bottom-nav__item--profile {
  background: var(--color-surface-container-lowest);
  border-left: 1px solid rgba(194, 199, 207, 0.15);
}

.c-bottom-nav__avatar {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-full);
  background: var(--gradient-primary);
  color: #fff;
  font-family: var(--font-display);
  font-size: 11px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.c-bottom-nav__item--active .c-bottom-nav__avatar {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

/* ── Nav drawer (mobile) ─────────────────────────────────────── */
.c-nav-drawer-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  z-index: calc(var(--z-nav) + 1);
}

.c-nav-drawer {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: calc(var(--z-nav) + 2);
  background: var(--color-surface-container-lowest);
  border-radius: var(--radius-2xl) var(--radius-2xl) 0 0;
  padding-bottom: var(--nav-safe-area-bottom, 0px);
  transform: translateY(100%);
  transition: transform var(--transition-base);
  max-height: 82vh;
  overflow-y: auto;
}

.c-nav-drawer--open { transform: translateY(0); }

.c-nav-drawer__handle-area {
  display: flex;
  justify-content: center;
  padding: var(--space-3) 0 var(--space-2);
  cursor: pointer;
}

.c-nav-drawer__handle {
  width: 40px;
  height: 4px;
  border-radius: 2px;
  background: var(--color-outline-variant);
}

.c-nav-drawer__nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 0 var(--space-3) var(--space-5);
}

.c-nav-drawer__group-label {
  font-size: 10px;
  font-weight: var(--fw-medium, 500);
  letter-spacing: 0.07em;
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
  padding: 14px var(--space-3) var(--space-1);
  opacity: 0.6;
}

.c-nav-drawer__item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-3);
  border-radius: var(--radius-lg);
  text-decoration: none;
  color: var(--color-on-surface-variant);
  font-size: var(--text-md);
  font-weight: 500;
  transition: background var(--transition-fast);
}

.c-nav-drawer__item:hover { background: var(--color-surface-container-low); color: var(--color-on-surface); }

.c-nav-drawer__item--active {
  background: var(--color-secondary-container);
  color: var(--color-on-secondary-container);
}
.c-nav-drawer__item--active .c-nav-drawer__item-icon { color: var(--color-primary); }

.c-nav-drawer__item-icon { width: 20px; height: 20px; flex-shrink: 0; }
.c-nav-drawer__item-icon :deep(svg) { width: 20px; height: 20px; }

/* Overlay fade transition */
.c-drawer-fade-enter-active,
.c-drawer-fade-leave-active { transition: opacity var(--transition-base); }
.c-drawer-fade-enter-from,
.c-drawer-fade-leave-to { opacity: 0; }
</style>
