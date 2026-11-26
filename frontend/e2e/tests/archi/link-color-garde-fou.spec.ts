import { test, expect } from '@playwright/test'
import fs from 'node:fs'
import path from 'node:path'

/**
 * Unlayered base.css sets `a { color: var(--color-primary) }`, which beats
 * Tailwind's layered `text-*` utilities. A link inside a heading therefore
 * paints its title in the link colour unless it carries an explicit text
 * utility — a regression no contrast test catches (primary passes AA).
 * This guard keeps every heading link neutrally coloured by construction.
 */

const SRC = path.resolve(import.meta.dirname, '../../src')

function vueFiles(dir: string): string[] {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap(entry => {
    const full = path.join(dir, entry.name)
    if (entry.isDirectory()) return vueFiles(full)
    return entry.name.endsWith('.vue') ? [full] : []
  })
}

interface Offender {
  file: string
  line: number
  tag: string
}

function findHeadingLinksWithoutTextColour(): Offender[] {
  const offenders: Offender[] = []
  for (const file of vueFiles(SRC)) {
    const source = fs.readFileSync(file, 'utf-8')
    if (!source.includes('RouterLink') && !source.includes('<a')) continue
    for (const heading of source.matchAll(/<h[1-6]\b[\s\S]*?<\/h[1-6]>/g)) {
      const block = heading[0]
      for (const tag of block.matchAll(/<(RouterLink|a)\b[^>]*>/g)) {
        const classAttr = tag[0].match(/(?::?class)="([^"]*)"/)
        const hasTextColour = classAttr ? /\b!?text-[a-z-]+\b/.test(classAttr[1]) : false
        if (!hasTextColour) {
          offenders.push({
            file: path.relative(SRC, file),
            line: source.slice(0, heading.index).split('\n').length,
            tag: tag[0].replace(/\s+/g, ' ').slice(0, 90),
          })
        }
      }
    }
  }
  return offenders
}

test('every heading link carries an explicit text colour', () => {
  const offenders = findHeadingLinksWithoutTextColour()
  const report = offenders
    .map(o => `${o.file}:${o.line} — ${o.tag}`)
    .join('\n')
  expect(offenders, `heading links would inherit the base link colour:\n${report}`).toEqual([])
})
