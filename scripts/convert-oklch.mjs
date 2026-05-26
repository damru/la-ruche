// scripts/convert-oklch.mjs
// Usage: node scripts/convert-oklch.mjs
import { converter, formatCss } from 'culori'

const toOklch = converter('oklch')

function hex(h) {
  const c = toOklch(h)
  const l = c.l.toFixed(3)
  const ch = c.c.toFixed(4)
  const hue = (c.h ?? 0).toFixed(2)
  return `oklch(${l} ${ch} ${hue})`
}

function hexAlpha(h, a) {
  const c = toOklch(h)
  const l = c.l.toFixed(3)
  const ch = c.c.toFixed(4)
  const hue = (c.h ?? 0).toFixed(2)
  return `oklch(${l} ${ch} ${hue} / ${a})`
}

const colors = {
  'primary':                    '#643fd5',
  'primary-container':          '#7d5bf0',
  'on-primary':                 '#ffffff',
  'secondary':                  '#5a6073',
  'secondary-container':        '#dde1f9',
  'on-secondary-container':     '#1a1d2e',
  'error':                      '#ba1a1a',
  'error-container':            '#ffdad6',
  'on-error':                   '#ffffff',
  'success':                    '#2e7d32',
  'success-container':          '#e8f5e9',
  'warning':                    '#e65100',
  'warning-container':          '#fff8e1',
  'role-manager-bg':            '#e8def8',
  'role-manager-fg':            '#21005d',
  'role-board-bg':              '#d2e4fb',
  'role-board-fg':              '#001d36',
  'role-co-owner-bg':           '#cef5e5',
  'role-co-owner-fg':           '#00391d',
  'role-tenant-bg':             '#fcecd1',
  'role-tenant-fg':             '#3a2000',
  'surface':                    '#f8f9fa',
  'surface-lowest':             '#ffffff',
  'surface-low':                '#f3f4f5',
  'surface-container':          '#edeff0',
  'surface-high':               '#e7e9ea',
  'surface-highest':            '#e1e3e4',
  'on-surface':                 '#191c1d',
  'on-surface-variant':         '#42474e',
  'outline':                    '#74777f',
  'outline-variant':            '#c2c7cf',
}

console.log('/* Light mode colors → OKLCH */')
for (const [name, value] of Object.entries(colors)) {
  console.log(`  --color-${name}: ${hex(value)};`)
}

console.log('\n/* Shadows with alpha */')
console.log(`  --shadow-sm:  0 2px 8px ${hexAlpha('#191c1d', '0.04')};`)
console.log(`  --shadow-md:  0 8px 32px ${hexAlpha('#191c1d', '0.07')}, 0 2px 8px ${hexAlpha('#191c1d', '0.04')};`)
console.log(`  --shadow-lg:  0 16px 48px ${hexAlpha('#191c1d', '0.10')}, 0 4px 16px ${hexAlpha('#191c1d', '0.06')};`)
console.log(`  --shadow-glass: 0 20px 60px ${hexAlpha('#191c1d', '0.14')}, 0 4px 16px ${hexAlpha('#191c1d', '0.08')};`)
console.log(`  --shadow-primary-ring:       0 0 0 2px ${hexAlpha('#643fd5', '0.35')};`)
console.log(`  --shadow-error-ring:         0 0 0 2px ${hexAlpha('#ba1a1a', '0.35')};`)
console.log(`  --shadow-primary-lift:       0 4px 16px ${hexAlpha('#643fd5', '0.25')};`)
console.log(`  --shadow-primary-lift-hover: 0 8px 24px ${hexAlpha('#643fd5', '0.35')};`)

console.log('\n/* Gradient */')
console.log(`  --gradient-primary: linear-gradient(135deg, ${hex('#643fd5')}, ${hex('#7d5bf0')});`)
