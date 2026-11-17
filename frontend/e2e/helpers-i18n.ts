import fs from 'node:fs'
import path from 'node:path'

// Locale fr chargée en texte puis parsée (le module JSON n'est pas résolu
// proprement par tsc dans ce setup — fs est plus robuste côté Node).
const fr: Record<string, unknown> = JSON.parse(
  fs.readFileSync(
    path.resolve(import.meta.dirname, '../src/i18n/locales/fr.json'),
    'utf-8',
  ),
)

/** Récupère le texte i18n fr d'une clé (les locales sont du JSON pur). */
export function frText(key: string): string {
  const parts = key.split('.')
  let node: unknown = fr
  for (const p of parts) {
    node = (node as Record<string, unknown>)[p]
    if (node === undefined) throw new Error(`clé i18n inconnue: ${key}`)
  }
  return node as string
}
