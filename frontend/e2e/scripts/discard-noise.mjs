// Discard captures whose change vs HEAD is sub-visual noise.
// Rendering jitter between identical CI runs: effectively ZERO since fonts
// are self-hosted (fontsource in the bundle) — the Google-Fonts CDN served
// variable-font instances that differed between runs, producing 8-616 px
// deterministic-looking diffs on text-heavy pages. Measured on CI x86
// (2026-11-22): 0 px everywhere after re-baselining. 200 px keeps margin
// for PNG-encoding-level byte noise (pixelmatch 0) and AA drift from CI
// browser updates. Re-measure via the `noise (N px)` lines of the discard
// step's log before touching it. Pages with no baseline at HEAD (new) are
// always kept. Run from frontend/ after the capture step. Requires:
// pixelmatch + pngjs (devDependencies).

import { execSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { PNG } from 'pngjs'

const pixelmatch = createRequire(import.meta.url)('pixelmatch').default
const NOISE_PX = 200

const changed = execSync('git status --porcelain -- frontend/e2e/visual', {
  cwd: '..',
  encoding: 'utf8',
})
  .split('\n')
  .filter((l) => l.startsWith(' M ') || l.startsWith('M '))

let kept = 0
for (const line of changed) {
  const path = line.slice(3).trim() // repo-root relative
  const local = path.replace(/^frontend\//, '') // relative to this script's cwd (frontend/)
  let mismatch
  try {
    const head = PNG.sync.read(execSync(`git show HEAD:${path}`, { cwd: '..', encoding: 'buffer' }))
    const now = PNG.sync.read(readFileSync(local))
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
