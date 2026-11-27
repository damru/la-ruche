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

  return { authorLabel }
}
