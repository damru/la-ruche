import { useI18n } from 'vue-i18n'
import type { Report } from '@/types/domain'

export function useReportLabels() {
  const { t } = useI18n()

  function authorLabel(report: Report): string {
    if (report.is_anonymous) {
      return report.is_author ? t('reports.authorAnonymousYou') : t('reports.authorAnonymous')
    }
    return `${report.author_first_name ?? ''} ${report.author_last_name ?? ''}`.trim()
  }

  function statusVariant(s: string): 'valid' | 'open' | 'pending' | 'closed' | 'default' {
    if (s === 'open') return 'pending'
    if (s === 'in_progress') return 'open'
    if (s === 'closed') return 'closed'
    return 'default'
  }

  return { authorLabel, statusVariant }
}
