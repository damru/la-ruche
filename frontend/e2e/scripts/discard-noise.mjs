// Discard captures whose change vs HEAD is sub-visual noise.
// Rendering jitter between identical CI runs measures 81-562 px with
// pixelmatch's AA detection (visually invisible); 1000 px absorbs it with
// ~2x margin while real content changes sit far above. Pages with no
// baseline at HEAD (new) are always kept. Run from frontend/ after the
// capture step. Requires: pixelmatch + pngjs (devDependencies).

import { execSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { PNG } from 'pngjs'

const pixelmatch = createRequire(import.meta.url)('pixelmatch').default
const NOISE_PX = 1000

const changed = execSync('git status --porcelain -- frontend/e2e/visual', {
  cwd: '..',
  encoding: 'utf8',
})
  .split('\n')
  .filter((l) => l.startsWith(' M ') || l.startsWith('M '))

let kept = 0
for (const line of changed) {
  const path = line.slice(3).trim()
  let mismatch
  try {
    const head = PNG.sync.read(execSync(`git show HEAD:${path}`, { cwd: '..', encoding: 'buffer' }))
    const now = PNG.sync.read(readFileSync(path))
    mismatch =
      head.width === now.width && head.height === now.height
        ? pixelmatch(head.data, now.data, null, now.width, now.height, { threshold: 0.1 })
        : Infinity // size change is always a real change
  } catch {
    continue // no baseline at HEAD yet — new page, keep
  }
  if (mismatch < NOISE_PX) {
    execSync(`git checkout -- ${path}`, { cwd: '..' })
    console.log(`noise (${mismatch} px): restored ${path}`)
  } else {
    kept++
    console.log(`change (${mismatch} px): kept ${path}`)
  }
}
console.log(`${changed.length - kept} discarded, ${kept} kept`)
