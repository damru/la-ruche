import { test, expect } from '@playwright/test'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

/**
 * DDR-006 garde-fou d'architecture : `CFormFeedback` ne s'utilise jamais
 * directement dans une vue — il passe par `CFormActions` (position
 * garantie) ou par `CForm` (qui délègue à `CFormActions`).
 *
 * Test statique : scan des `.vue` de src/. Échec = violation à corriger
 * dans le produit, jamais à exclure.
 */

const SRC = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../src')

function listVueFiles(dir: string): string[] {
  const out: string[] = []
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name)
    if (entry.isDirectory()) out.push(...listVueFiles(full))
    else if (entry.name.endsWith('.vue')) out.push(full)
  }
  return out
}

test('CFormFeedback jamais utilisé hors de CFormActions/CForm', () => {
  const violations: string[] = []
  for (const file of listVueFiles(SRC)) {
    const rel = path.relative(SRC, file)
    if (rel.startsWith('components')) continue
    const content = fs.readFileSync(file, 'utf8')
    const usesFeedback = /<CFormFeedback[\s>]/.test(content)
    if (!usesFeedback) continue
    const allowed = /<CFormActions[\s>]/.test(content) || rel.endsWith('CForm.vue')
    if (!allowed) violations.push(rel)
  }
  expect(
    violations,
    `CFormFeedback utilisé directement dans : ${violations.join(', ')}. Le passer par CFormActions.`,
  ).toEqual([])
})

test('CFormFeedback est bien encapsulé dans CFormActions', () => {
  const actions = path.join(SRC, 'components/molecules/CFormActions.vue')
  expect(fs.readFileSync(actions, 'utf8')).toContain('<CFormFeedback')
})
