import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import type { Role } from '@/types/domain'

// ── Lazy imports ──────────────────────────────────────────────────
const LoginView          = () => import('@/features/auth/LoginView.vue')
const AcceptInviteView   = () => import('@/features/auth/AcceptInviteView.vue')
const ResetPasswordView  = () => import('@/features/auth/ResetPasswordView.vue')
const OnboardingView     = () => import('@/features/onboarding/OnboardingView.vue')
const DashboardView      = () => import('@/features/dashboard/DashboardView.vue')
const AssemblyListView   = () => import('@/features/assemblies/AssemblyListView.vue')
const AssemblyDetailView = () => import('@/features/assemblies/AssemblyDetailView.vue')
const BudgetView         = () => import('@/features/budget/BudgetView.vue')
const DocumentsView      = () => import('@/features/documents/DocumentsView.vue')
const TasksView          = () => import('@/features/tasks/TasksView.vue')
const ContractsView      = () => import('@/features/contracts/ContractsView.vue')
const ClaimsView         = () => import('@/features/claims/ClaimsView.vue')
const MaintenanceView    = () => import('@/features/maintenance/MaintenanceView.vue')
const ConsultationsView  = () => import('@/features/consultations/ConsultationsView.vue')
const ReportsView        = () => import('@/features/reports/ReportsView.vue')
const ResidentsView      = () => import('@/features/residents/ResidentsView.vue')
const SettingsView       = () => import('@/features/settings/SettingsView.vue')
const ContractorPage     = () => import('@/features/contractor/ContractorPage.vue')
const AppLayout          = () => import('@/layouts/AppLayout.vue')
const ForbiddenView      = () => import('@/views/ForbiddenView.vue')
const NotFoundView       = () => import('@/views/NotFoundView.vue')

// ── Meta typage ───────────────────────────────────────────────────
declare module 'vue-router' {
  interface RouteMeta {
    requiresAuth?: boolean
    roles?: Role[]         // allowed roles (empty = all authenticated users)
    title?: string         // pour le <title> de la page
  }
}

const routes: RouteRecordRaw[] = [
  // ── Public ────────────────────────────────────────────────────
  { path: '/login',          name: 'login',         component: LoginView,         meta: { title: 'Connexion' } },
  { path: '/invite/:token',  name: 'invite',        component: AcceptInviteView,  meta: { title: 'Rejoindre' } },
  { path: '/reset/:token',   name: 'reset',         component: ResetPasswordView, meta: { title: 'Nouveau mot de passe' } },  { path: '/onboarding',     name: 'onboarding',    component: OnboardingView,    meta: { title: 'Bienvenue' } },
  // ── Contractor (token URL, no JWT) ────────────────────────────
  { path: '/vendor/:token', name: 'contractor', component: ContractorPage, meta: { title: 'Contractor' } },

  // ── App (layout with nav) ─────────────────────────────────────
  {
    path: '/',
    component: AppLayout,
    meta: { requiresAuth: true },
    children: [
      {
        path: '',
        redirect: '/dashboard',
      },
      {
        path: 'dashboard',
        name: 'dashboard',
        component: DashboardView,
        meta: { title: 'Dashboard', roles: [] },
      },
      {
        path: 'meetings',
        name: 'assembly-list',
        component: AssemblyListView,
        meta: { title: 'General Meetings', roles: ['manager', 'board_member', 'co_owner'] },
      },
      {
        path: 'meetings/:id',
        name: 'assembly-detail',
        component: AssemblyDetailView,
        meta: { title: 'General Meeting', roles: ['manager', 'board_member', 'co_owner'] },
      },
      {
        path: 'budget',
        name: 'budget',
        component: BudgetView,
        meta: { title: 'Budget', roles: ['manager', 'board_member', 'co_owner'] },
      },
      {
        path: 'documents',
        name: 'documents',
        component: DocumentsView,
        meta: { title: 'Documents', roles: ['manager', 'board_member', 'co_owner'] },
      },
      {
        path: 'tasks',
        name: 'tasks',
        component: TasksView,
        meta: { title: 'Tasks', roles: ['manager', 'board_member'] },
      },
      {
        path: 'contracts',
        name: 'contracts',
        component: ContractsView,
        meta: { title: 'Contracts', roles: ['manager'] },
      },
      {
        path: 'claims',
        name: 'claims',
        component: ClaimsView,
        meta: { title: 'Claims', roles: ['manager', 'board_member'] },
      },
      {
        path: 'logbook',
        name: 'maintenance',
        component: MaintenanceView,
        meta: { title: 'Maintenance Log', roles: ['manager', 'board_member'] },
      },
      {
        path: 'consultations',
        name: 'consultations',
        component: ConsultationsView,
        meta: { title: 'Consultations', roles: [] },
      },
      {
        path: 'reports',
        name: 'reports',
        component: ReportsView,
        meta: { title: 'Reports', roles: [] },
      },
      {
        path: 'residents',
        name: 'residents',
        component: ResidentsView,
        meta: { title: 'Residents', roles: ['manager'] },
      },
      {
        path: 'settings',
        name: 'settings',
        component: SettingsView,
        meta: { title: 'Settings', roles: ['manager'] },
      },
    ],
  },

  // ── Errors ────────────────────────────────────────────────────
  { path: '/403', name: 'forbidden', component: ForbiddenView, meta: { title: 'Forbidden' } },
  { path: '/:pathMatch(.*)*', name: 'not-found', component: NotFoundView, meta: { title: 'Not Found' } },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior: () => ({ top: 0 }),
})

// ── Guard global ────────────────────────────────────────────────
router.beforeEach(async (to) => {
  const auth = useAuthStore()

  // Update page title
  document.title = to.meta.title ? `${to.meta.title} — La Ruche` : 'La Ruche'

  // Restore session from cookie (once per page load, no-op if already done)
  await auth.restoreSession()

  // Check onboarding status (once per session)
  if (auth.needsOnboarding === null) {
    await auth.checkOnboardingStatus()
  }

  // Instance not initialised → redirect to onboarding
  if (auth.needsOnboarding && to.name !== 'onboarding') {
    return { name: 'onboarding' }
  }

  // Instance already initialised → block onboarding route
  if (!auth.needsOnboarding && to.name === 'onboarding') {
    return auth.isAuthenticated ? { name: 'dashboard' } : { name: 'login' }
  }

  // Route publique (hors onboarding)
  if (!to.meta.requiresAuth) return true

  // Not authenticated → redirect to login
  if (!auth.isAuthenticated) {
    return { name: 'login', query: { redirect: to.fullPath } }
  }

  // Role check
  const requiredRoles = to.meta.roles ?? []
  if (requiredRoles.length > 0 && !auth.hasAnyRole(requiredRoles)) {
    // 403 rather than 404 — 404 leaks information
    return { name: 'forbidden' }
  }

  return true
})

export default router
