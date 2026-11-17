/**
 * Matrice d'accès routes × rôles — source de vérité du smoke Phase 1.
 *
 * Dérivée de `frontend/src/router/index.ts` (meta.roles) et de
 * `src/utils/roles.ts` :
 *   ROLES          = [manager, board_member, co_owner, tenant]
 *   MANAGER_ROLES  = [manager, board_member]
 *   DOCUMENT_ROLES = ROLES
 *   ROLE_MANAGER   = [manager] / ROLE_CO_OWNER = [co_owner]
 *
 * Un test de cohérence (tests/consistance-matrix.spec.ts) relit le code du
 * routeur en texte (fs + regex — le routeur n'est pas importable en Node :
 * createWebHistory au niveau module) et vérifie que chaque route y figure.
 *
 * NB : les routes de détail (meetings/:id, tasks/:id, contracts/:id,
 * contractors/:id, residents/:id/charges) sont exclues du smoke Phase 1
 * (lecture seule sans navigation ciblée sur une entité) — elles seront
 * couvertes en Phase 2 via des entités créées par API.
 */

export type AuthRole = 'manager' | 'boardMember' | 'coOwner' | 'tenant'

/** Rôles backend correspondants (seed demo.sql). */
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
  /** Clé i18n du titre (meta.title) — assertée visible dans la page. */
  titleKey: string
  /** Rôles autorisés à voir cette route. */
  roles: AuthRole[]
  /**
   * Comportement des comptes non autorisés :
   * - 'route403' (défaut) : le guard routeur redirige vers /403 (ForbiddenView).
   * - 'pageError' : la route est ouverte au guard (meta.roles permissifs),
   *   le backend renvoie 403 et la vue rend CPageError (DDR-005).
   */
  deniedBehavior?: 'route403' | 'pageError'
  /**
   * Endpoints GET en 403 tolérés sur cette page pour certains rôles :
   * vue-query fetch au montage des widgets dont le backend restreint l'accès
   * par rôle (RBAC par endpoint, ex. levies du budget fermé sur /dashboard).
   * Trace l'exception explicitement — jamais de wildcard.
   */
  toleratedApi403?: Record<string, AuthRole[]>
}

/** Routes applicatives authentifiées (enfant du layout `/`). */
export const APP_ROUTES: MatrixEntry[] = [
  { path: '/dashboard',              titleKey: 'nav.dashboard',                 roles: ALL, toleratedApi403: { '/api/v1/budgets/00000000-0000-0000-0000-000000000421/levies': ['coOwner', 'tenant'] } },
  { path: '/meetings',               titleKey: 'nav.meetings',                  roles: ALL },
  { path: '/budget',                 titleKey: 'nav.budget',                    roles: ALL },
  { path: '/documents',              titleKey: 'common.documents',              roles: ALL },
  { path: '/tasks',                  titleKey: 'nav.tasks',                     roles: MANAGER },
  { path: '/contractors',            titleKey: 'contractors.title',             roles: ALL },
  { path: '/contracts',              titleKey: 'nav.contracts',                 roles: MANAGER },
  { path: '/claims',                 titleKey: 'nav.claims',                    roles: MANAGER },
  { path: '/logbook',                titleKey: 'nav.maintenance',               roles: [...MANAGER, 'coOwner'] },
  { path: '/calendar',               titleKey: 'nav.calendar',                  roles: ALL },
  { path: '/consultations',          titleKey: 'nav.consultations',             roles: ALL },
  { path: '/reports',                titleKey: 'nav.reports',                   roles: ALL },
  { path: '/residents',              titleKey: 'nav.residents',                 roles: MGR_ONLY },
  { path: '/residents/00000000-0000-0000-0000-000000000102/charges', titleKey: 'units.charges.title', roles: MANAGER, deniedBehavior: 'pageError' },
  { path: '/my-charges',             titleKey: 'nav.myCharges',                 roles: OWNER_ONLY },
  { path: '/registry',               titleKey: 'nav.registry',                  roles: ALL },
  { path: '/judicial-proceedings',   titleKey: 'nav.judicialProceedings',       roles: ALL },
  { path: '/governance',             titleKey: 'nav.governance',                roles: ALL },
  { path: '/etat-date',              titleKey: 'registers.etatDate.title',      roles: MGR_ONLY },
  { path: '/residence/building',     titleKey: 'common.informations',           roles: MGR_ONLY },
  { path: '/residence/keys',         titleKey: 'allocationKeys.title',          roles: MGR_ONLY },
  { path: '/profile',                titleKey: 'nav.account',                   roles: ALL },
]

/** Ids de lot seedés (demo.sql) — lot 2 (Emma Lefebvre, co_owner). */
export const SEEDED_UNIT_ID = '00000000-0000-0000-0000-000000000102'

/**
 * Rôles backend réels de chaque compte seedé (demo.sql) — Bernard cumule
 * board_member ET co_owner : le guard utilise hasAnyRole, donc la matrice
 * d'interdiction se raisonne en COMPTES, pas en rôles abstraits.
 */
export const ACCOUNT_ROLES: Record<AuthRole, string[]> = {
  manager: ['manager'],
  boardMember: ['board_member', 'co_owner'],
  coOwner: ['co_owner'],
  tenant: ['tenant'],
}

/**
 * Comptes seedés autorisés par une liste de rôles (AuthRole camelCase,
 * cf. MatrixEntry.roles) : conversion en rôles backend puis hasAnyRole
 * contre les rôles réels de chaque compte.
 */
export function allowedAccounts(requiredRoles: AuthRole[]): AuthRole[] {
  const requiredBackend = requiredRoles.map((r) => ROLE_BACKEND[r])
  return (Object.keys(ACCOUNT_ROLES) as AuthRole[]).filter((account) =>
    ACCOUNT_ROLES[account].some((r) => requiredBackend.includes(r)),
  )
}
