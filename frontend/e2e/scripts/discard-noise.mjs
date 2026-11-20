// Discard visual captures whose change vs HEAD is below the noise threshold.
//
// Even with a frozen clock, an anchored demo seed and Chromium font flags,
// two CI runs minutes apart render identical content with sub-visual glyph
// differences (measured: pixelmatch mismatch 81-562 px across 18 pages,
// 0.006-0.043% — antialiasing edge noise, text visually identical).
// pixelmatch's AA detection plus a 1000 px floor absorbs that noise while
// real content changes (layout moves, swapped text, new blocks) sit far
// above it. New files (pages without a baseline yet) are always kept.
//
// Run from frontend/, after the capture step, before the commit step.
// Requires: pixelmatch + pngjs (devDependencies).

import { execSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { PNG } from 'pngjs'

const require = createRequire(import.meta.url)
const pixelmatch = require('pixelmatch').default

const NOISE_THRESHOLD_PX = 1000
const VISUAL_DIR = 'frontend/e2e/visual'

function mismatchPx(baseline, captured) {
  const a = PNG.sync.read(baseline)
  const b = PNG.sync.read(captured)
  if (a.width !== b.width || a.height !== b.height) {
    return Number.MAX_SAFE_INTEGER // size change — never noise
  }
  return pixelmatch(a.data, b.data, null, a.width, a.height, { threshold: 0.1 })
}

const status = execSync(`git status --porcelain -- ${VISUAL_DIR}`, {
  encoding: 'utf8',
  cwd: '..',
})
const changed = status.split('\n')
const targets = changed
  .map((line) => line.trim())
  .filter(Boolean)
  .filter((line) => !line.startsWith('??')) // new files always kept
  .map((line) => line.slice(3).trim())

let discarded = 0
let kept = 0
for (const path of targets) {
  const head = execSync(`git show HEAD:${path}`, {
    encoding: 'buffer',
  })
  if (!head || head.length === 0) {
    kept++ // no baseline at HEAD — new page, keep
    continue
  }
  const captured = readFileSync(path)
  const n = mismatchPx(head, captured)
  if (n < NOISE_THRESHOLD_PX) {
    execSync(`git checkout -- ${path}`)
    discarded++
    console.log(`noise (${n} px <= ${NOISE_THRESHOLD_PX}): ${path} restored`)
  } else {
    kept++
    console.log(`real change (${n} px): ${path} kept`)
  }
}
console.log(`${discarded} discarded, ${kept} kept`)
