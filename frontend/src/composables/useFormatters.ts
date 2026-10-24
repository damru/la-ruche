import { BYTES_PER_KB, BYTES_PER_MB, MS_PER_DAY } from '@/utils/constants'
// ── useFormatters — locale-aware formatting (fr-FR)
// Uses native Intl — no external dependency (date-fns, dayjs…)

export function useFormatters() {
  const currency = new Intl.NumberFormat('fr-FR', {
    style: 'currency',
    currency: 'EUR',
    minimumFractionDigits: 2,
  })

  const number = new Intl.NumberFormat('fr-FR', {
    minimumFractionDigits: 0,
    maximumFractionDigits: 4,
  })

  const dateShort = new Intl.DateTimeFormat('fr-FR', {
    day: '2-digit', month: '2-digit', year: 'numeric',
  })

  const dateLong = new Intl.DateTimeFormat('fr-FR', {
    day: 'numeric', month: 'long', year: 'numeric',
  })

  const dateTime = new Intl.DateTimeFormat('fr-FR', {
    day: '2-digit', month: '2-digit', year: 'numeric',
    hour: '2-digit', minute: '2-digit',
  })

  function formatCurrency(value: number | string | null | undefined): string {
    if (value == null) return '—'
    const normalized = typeof value === 'string' ? Number(value) : value
    if (Number.isNaN(normalized)) return '—'
    return currency.format(normalized)
  }

  function formatDate(value: string | null | undefined, format: 'short' | 'long' | 'datetime' = 'short'): string {
    if (!value) return '—'
    const date = new Date(value)
    if (isNaN(date.getTime())) return '—'
    switch (format) {
      case 'long': return dateLong.format(date)
      case 'datetime': return dateTime.format(date)
      default: return dateShort.format(date)
    }
  }

  function formatShares(value: number | null | undefined): string {
    if (value == null) return '—'
    return number.format(value)
  }

  function formatFileSize(bytes: number): string {
    if (bytes < BYTES_PER_KB) return `${bytes} B`
    if (bytes < BYTES_PER_MB) return `${(bytes / BYTES_PER_KB).toFixed(1)} KB`
    return `${(bytes / BYTES_PER_MB).toFixed(1)} MB`
  }

  function daysUntil(dateStr: string): number {
    const target = new Date(dateStr)
    const now = new Date()
    const diff = target.getTime() - now.getTime()
    return Math.ceil(diff / MS_PER_DAY)
  }

  function formatRelative(dateStr: string): string {
    const rtf = new Intl.RelativeTimeFormat('fr-FR', { numeric: 'auto' })
    const days = daysUntil(dateStr)
    return rtf.format(days, 'day')
  }

  return {
    formatCurrency,
    formatDate,
    formatShares,
    formatFileSize,
    formatRelative,
    daysUntil,
  }
}
