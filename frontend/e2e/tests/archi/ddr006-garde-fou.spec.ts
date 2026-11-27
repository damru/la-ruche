import { test, expect } from '@playwright/test'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

/**
 * DDR-006 guard: CFormFeedback is never used directly in a view (static
 * scan of src/ .vue files). Violation = product fix.
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

/**
 * base.css paints every anchor with the link colour outside any layer, so a
 * heading link only keeps the heading colour with an explicit text utility.
 * Contrast tests cannot catch it (the link colour passes AA).
 */
test('un lien dans un titre porte une couleur de texte explicite', () => {
  const offenders: string[] = []
  for (const file of listVueFiles(SRC)) {
    const rel = path.relative(SRC, file)
    const source = fs.readFileSync(file, 'utf8')
    if (!source.includes('RouterLink') && !source.includes('<a')) continue
    for (const heading of source.matchAll(/<h[1-6]\b[\s\S]*?<\/h[1-6]>/g)) {
      for (const tag of heading[0].matchAll(/<(RouterLink|a)\b[^>]*>/g)) {
        const classAttr = tag[0].match(/(?::?class)="([^"]*)"/)
        const hasColour = classAttr ? /\b!?text-[a-z-]+\b/.test(classAttr[1]) : false
        if (!hasColour) {
          const line = source.slice(0, heading.index).split('\n').length
          offenders.push(`${rel}:${line} — ${tag[0].replace(/\s+/g, ' ').slice(0, 80)}`)
        }
      }
    }
  }
  expect(offenders, `liens de titre sans couleur explicite :\n${offenders.join('\n')}`).toEqual([])
})
