import fs from 'node:fs'
import path from 'node:path'

const fr: Record<string, unknown> = JSON.parse(
  fs.readFileSync(
    path.resolve(import.meta.dirname, '../src/i18n/locales/fr.json'),
    'utf-8',
  ),
)

export function frText(key: string): string {
  const parts = key.split('.')
  let node: unknown = fr
  for (const p of parts) {
    node = (node as Record<string, unknown>)[p]
    if (node === undefined) throw new Error(`clé i18n inconnue: ${key}`)
  }
  return node as string
}
