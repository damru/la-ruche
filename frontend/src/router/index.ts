import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import type { Role } from '@/types/domain'
import i18n from '@/i18n'

// ── Lazy imports ──────────────────────────────────────────────────
const LoginView               = () => import('@/features/auth/LoginView.vue')
const AcceptInviteView        = () => import('@/features/auth/AcceptInviteView.vue')
const ResetPasswordView       = () => import('@/features/auth/ResetPasswordView.vue')
const OnboardingView          = () => import('@/features/onboarding/OnboardingView.vue')
const DashboardView           = () => import('@/features/dashboard/DashboardView.vue')
const AssemblyListView        = () => import('@/features/assemblies/AssemblyListView.vue')
const AssemblyDetailView      = () => import('@/features/assemblies/AssemblyDetailView.vue')
const BudgetView              = () => import('@/features/budget/BudgetView.vue')
const DocumentsView           = () => import('@/features/documents/DocumentsView.vue')
const TasksView               = () => import('@/features/tasks/TasksView.vue')
const TaskDetailView          = () => import('@/features/tasks/TaskDetailView.vue')
const ContractsView           = () => import('@/features/contracts/ContractsView.vue')
const ContractDetailView      = () => import('@/features/contracts/ContractDetailView.vue')
const ContractorsView         = () => import('@/features/contractors/ContractorsView.vue')
const ContractorDetailView    = () => import('@/features/contractors/ContractorDetailView.vue')
const ClaimsView              = () => import('@/features/claims/ClaimsView.vue')
const MaintenanceView         = () => import('@/features/maintenance/MaintenanceView.vue')
const CalendarView            = () => import('@/features/calendar/CalendarView.vue')
const ConsultationsView       = () => import('@/features/consultations/ConsultationsView.vue')
const ReportsView             = () => import('@/features/reports/ReportsView.vue')
const ResidentsView           = () => import('@/features/residents/ResidentsView.vue')
const RegistryView            = () => import('@/features/residents/RegistryView.vue')
const JudicialProceedingsView = () => import('@/features/registers/JudicialProceedingsView.vue')
const GovernanceMandatesView  = () => import('@/features/registers/GovernanceMandatesView.vue')
const EtatDateView            = () => import('@/features/registers/EtatDateView.vue')
const UnitChargesView         = () => import('@/features/residents/UnitChargesView.vue')
const MyChargesView           = () => import('@/features/residents/MyChargesView.vue')
const BuildingView            = () => import('@/features/residence/BuildingView.vue')
const AllocationKeysView      = () => import('@/features/residence/AllocationKeysView.vue')
const ProfileView             = () => import('@/features/profile/ProfileView.vue')
const ContractorPage          = () => import('@/features/contractor/ContractorPage.vue')
const AppLayout               = () => import('@/layouts/AppLayout.vue')
const ForbiddenView           = () => import('@/views/ForbiddenView.vue')
const NotFoundView            = () => import('@/views/NotFoundView.vue')

// ── Meta typage ───────────────────────────────────────────────────
declare module 'vue-router' {
  interface RouteMeta {
    requiresAuth?: boolean
    roles?: Role[]         // allowed roles (empty = all authenticated users)
    title?: string         // pour le <title> de la page
    public?: boolean       // fully public route — skips all auth/onboarding checks
  }
}

const routes: RouteRecordRaw[] = [
  // ── Public ────────────────────────────────────────────────────
  { path: '/login',          name: 'login',         component: LoginView,         meta: { title: 'auth.login' } },
  { path: '/invite/:token',  name: 'invite',        component: AcceptInviteView,  meta: { title: 'auth.acceptInvite' } },
  { path: '/reset/:token',   name: 'reset',         component: ResetPasswordView, meta: { title: 'auth.resetPassword' } },
  { path: '/onboarding',     name: 'onboarding',    component: OnboardingView,    meta: { title: 'page.onboarding' } },
  // ── Contractor (token URL, no JWT) ────────────────────────────
  { path: '/vendor/:token', name: 'contractor', component: ContractorPage, meta: { title: 'page.contractor', public: true } },

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
        meta: { title: 'nav.dashboard', roles: [] },
      },
      {
        path: 'meetings',
        name: 'assembly-list',
        component: AssemblyListView,
        meta: { title: 'nav.meetings', roles: ['manager', 'board_member', 'co_owner'] },
      },
      {
        path: 'meetings/:id',
        name: 'assembly-detail',
        component: AssemblyDetailView,
        meta: { title: 'nav.meetings', roles: ['manager', 'board_member', 'co_owner'] },
      },
      {
        path: 'budget',
        name: 'budget',
        component: BudgetView,
        meta: { title: 'nav.budget', roles: ['manager', 'board_member', 'co_owner'] },
      },
      {
        path: 'documents',
        name: 'documents',
        component: DocumentsView,
        meta: { title: 'common.documents', roles: ['manager', 'board_member', 'co_owner', 'tenant'] },
      },
      {
        path: 'tasks',
        name: 'tasks',
        component: TasksView,
        meta: { title: 'nav.tasks', roles: ['manager', 'board_member'] },
      },
      {
        path: 'tasks/:id',
        name: 'task-detail',
        component: TaskDetailView,
        meta: { title: 'nav.tasks', roles: ['manager', 'board_member'] },
      },
      {
        path: 'contractors',
        name: 'contractors',
        component: ContractorsView,
        meta: { title: 'contractors.title', roles: [] },
      },
      {
        path: 'contractors/:id',
        name: 'contractor-detail',
        component: ContractorDetailView,
        meta: { title: 'page.contractorDetail', roles: [] },
      },
      {
        path: 'contracts',
        name: 'contracts',
        component: ContractsView,
        meta: { title: 'nav.contracts', roles: ['manager', 'board_member'] },
      },
      {
        path: 'contracts/:id',
        name: 'contract-detail',
        component: ContractDetailView,
        meta: { title: 'page.contractDetail', roles: ['manager', 'board_member'] },
      },
      {
        path: 'claims',
        name: 'claims',
        component: ClaimsView,
        meta: { title: 'nav.claims', roles: ['manager', 'board_member'] },
      },
      {
        path: 'logbook',
        name: 'maintenance',
        component: MaintenanceView,
        meta: { title: 'nav.maintenance', roles: ['manager', 'board_member'] },
      },
      {
        path: 'calendar',
        name: 'calendar',
        component: CalendarView,
        meta: { title: 'nav.calendar', roles: ['manager', 'board_member', 'co_owner', 'tenant'] },
      },
      {
        path: 'consultations',
        name: 'consultations',
        component: ConsultationsView,
        meta: { title: 'nav.consultations', roles: [] },
      },
      {
        path: 'reports',
        name: 'reports',
        component: ReportsView,
        meta: { title: 'nav.reports', roles: [] },
      },
      {
        path: 'residents',
        name: 'residents',
        component: ResidentsView,
        meta: { title: 'nav.residents', roles: ['manager'] },
      },
      {
        path: 'residents/:id/charges',
        name: 'unit-charges',
        component: UnitChargesView,
        meta: { title: 'page.unitCharges', roles: ['manager', 'board_member', 'co_owner'] },
      },
      {
        path: 'my-charges',
        name: 'my-charges',
        component: MyChargesView,
        meta: { title: 'nav.myCharges', roles: ['co_owner'] },
      },
      {
        path: 'registry',
        name: 'registry',
        component: RegistryView,
        meta: { title: 'nav.registry', roles: ['manager', 'board_member', 'co_owner'] },
      },
      {
        path: 'judicial-proceedings',
        name: 'judicial-proceedings',
        component: JudicialProceedingsView,
        meta: { title: 'nav.judicialProceedings', roles: ['manager', 'board_member', 'co_owner'] },
      },
      {
        path: 'governance',
        name: 'governance',
        component: GovernanceMandatesView,
        meta: { title: 'nav.governance', roles: ['manager', 'board_member', 'co_owner'] },
      },
      {
        path: 'etat-date',
        name: 'etat-date',
        component: EtatDateView,
        meta: { title: 'registers.etatDate.title', roles: ['manager'] },
      },
      // ── Residence ───────────────────────────────────────────
      {
        path: 'residence/building',
        name: 'residence-building',
        component: BuildingView,
        meta: { title: 'common.informations', roles: ['manager'] },
      },
      {
        path: 'residence/keys',
        name: 'residence-keys',
        component: AllocationKeysView,
        meta: { title: 'allocationKeys.title', roles: ['manager'] },
      },
      // ── Profile ──────────────────────────────────────────────
      {
        path: 'profile',
        name: 'profile',
        component: ProfileView,
        meta: { title: 'nav.account', roles: [] },
      },
      // Legacy redirects
      { path: 'profile/account',       redirect: '/profile' },
      { path: 'profile/notifications', redirect: '/profile' },
      // ── Legacy redirect ───────────────────────────────────────
      {
        path: 'settings',
        redirect: '/residence/building',
      },
    ],
  },

  // ── Errors ────────────────────────────────────────────────────
  { path: '/403', name: 'forbidden', component: ForbiddenView, meta: { title: 'page.forbidden' } },
  { path: '/:pathMatch(.*)*', name: 'not-found', component: NotFoundView, meta: { title: 'page.notFound' } },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior: () => ({ top: 0 }),
})

// ── Guard global ────────────────────────────────────────────────
router.beforeEach(async (to) => {
  const auth = useAuthStore()

  // Update page title using i18n
  document.title = to.meta.title ? `${i18n.global.t(to.meta.title)} — La Ruche` : 'La Ruche'

  // Fully public routes (e.g. contractor page) — skip all auth/onboarding checks
  if (to.meta.public) return true

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
