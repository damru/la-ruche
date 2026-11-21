/**
 * Comptes de développement seedés — docs/DEV_ACCOUNTS.md.
 * Source de vérité : backend/seeds/demo.sql (mot de passe commun Demo1234!).
 * NB : le seed est ignoré si user_account n'est pas vide (DEMO_MODE) ;
 * re-seed via POST /api/v1/admin/reset-demo.
 */
export const DEV_PASSWORD = 'Demo1234!'

export const ACCOUNTS = {
  manager: 'alice.martin@laruche.test',
  boardMember: 'bernard.dupont@laruche.test',
  coOwner: 'david.laurent@laruche.test',
  tenant: 'isabelle.fontaine@laruche.test',
} as const

export type AuthRole = keyof typeof ACCOUNTS
