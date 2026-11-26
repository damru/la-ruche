
export type AuthRole = 'manager' | 'boardMember' | 'coOwner' | 'tenant'

export const ROLE_BACKEND: Record<AuthRole, string> = {
  manager: 'manager',
  boardMember: 'board_member',
  coOwner: 'co_owner',
  tenant: 'tenant',
}

const ALL: AuthRole[] = ['manager', 'boardMember', 'coOwner', 'tenant']
const MANAGER: AuthRole[] = ['manager', 'boardMember']
const OWNER_ONLY: AuthRole[] = ['coOwner']
const MGR_ONLY: AuthRole[] = ['manager']

export type MatrixEntry = {
  path: string
  titleKey: string
  roles: AuthRole[]
  deniedBehavior?: 'route403' | 'pageError'
  /** Also capture the alternate display mode (grid/cards) of this page. */
  captureGrid?: boolean
  /** Also capture these tabs (their visible label) once activated. */
  captureTabs?: string[]
  toleratedApi403?: Record<string, AuthRole[]>
}

export const APP_ROUTES: MatrixEntry[] = [
  { path: '/dashboard',              titleKey: 'nav.dashboard',                 roles: ALL, toleratedApi403: { '/api/v1/budgets/00000000-0000-0000-0000-000000000421/levies': ['coOwner', 'tenant'] } },
  { path: '/meetings',               titleKey: 'nav.meetings',                  roles: ALL },
  { path: '/budget',                 titleKey: 'nav.budget',                    roles: ALL },
  { path: '/documents',              titleKey: 'common.documents',              roles: ALL },
  { path: '/tasks',                  titleKey: 'nav.tasks',                     roles: MANAGER },
  { path: '/tasks/00000000-0000-0000-0000-000000000602', titleKey: 'nav.tasks', roles: MANAGER },
  { path: '/contractors',            titleKey: 'contractors.title',             roles: ALL, captureGrid: true },
  { path: '/contracts',              titleKey: 'nav.contracts',                 roles: MANAGER, captureGrid: true },
  { path: '/claims',                 titleKey: 'nav.claims',                    roles: MANAGER },
  { path: '/logbook',                titleKey: 'nav.maintenance',               roles: [...MANAGER, 'coOwner'] },
  { path: '/calendar',               titleKey: 'nav.calendar',                  roles: ALL },
  { path: '/consultations',          titleKey: 'nav.consultations',             roles: ALL },
  { path: '/reports',                titleKey: 'nav.reports',                   roles: ALL },
  { path: '/reports/00000000-0000-0000-0000-000000000703', titleKey: 'nav.reports', roles: ALL },
  { path: '/residents',              titleKey: 'nav.residents',                 roles: MGR_ONLY },
  { path: '/residents/00000000-0000-0000-0000-000000000102/charges', titleKey: 'units.charges.title', roles: MANAGER, deniedBehavior: 'pageError' },
  { path: '/my-charges',             titleKey: 'nav.myCharges',                 roles: OWNER_ONLY },
  { path: '/registry',               titleKey: 'nav.registry',                  roles: ALL },
  { path: '/judicial-proceedings',   titleKey: 'nav.judicialProceedings',       roles: ALL },
  { path: '/governance',             titleKey: 'nav.governance',                roles: ALL },
  { path: '/etat-date',              titleKey: 'registers.etatDate.title',      roles: MGR_ONLY },
  { path: '/residence/building',     titleKey: 'common.informations',           roles: MGR_ONLY },
  { path: '/residence/keys',         titleKey: 'allocationKeys.title',          roles: MGR_ONLY },
  { path: '/profile',                titleKey: 'nav.account',                   roles: ALL, captureTabs: ['Préférences'] },
]

export const SEEDED_UNIT_ID = '00000000-0000-0000-0000-000000000102'

export const ACCOUNT_ROLES: Record<AuthRole, string[]> = {
  manager: ['manager'],
  boardMember: ['board_member', 'co_owner'],
  coOwner: ['co_owner'],
  tenant: ['tenant'],
}

export function allowedAccounts(requiredRoles: AuthRole[]): AuthRole[] {
  const requiredBackend = requiredRoles.map((r) => ROLE_BACKEND[r])
  return (Object.keys(ACCOUNT_ROLES) as AuthRole[]).filter((account) =>
    ACCOUNT_ROLES[account].some((r) => requiredBackend.includes(r)),
  )
}
