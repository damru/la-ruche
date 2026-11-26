import { useI18n } from 'vue-i18n'
import type { Report, Unit } from '@/types/domain'
import { REPORT_STATUS_CLOSED, REPORT_STATUS_IN_PROGRESS, REPORT_STATUS_OPEN } from '@/utils/constants'

export function useReportLabels() {
  const { t } = useI18n()

  function authorLabel(report: Report): string {
    if (report.is_anonymous) {
      return report.is_author ? t('reports.authorAnonymousYou') : t('reports.authorAnonymous')
    }
    return `${report.author_first_name ?? ''} ${report.author_last_name ?? ''}`.trim()
  }

  function statusVariant(s: string): 'valid' | 'open' | 'pending' | 'closed' | 'default' {
    if (s === REPORT_STATUS_OPEN) return 'pending'
    if (s === REPORT_STATUS_IN_PROGRESS) return 'open'
    if (s === REPORT_STATUS_CLOSED) return 'closed'
    return 'default'
  }

  function indexUnits(units: Unit[]): Map<string, Unit> {
    return new Map(units.map(unit => [unit.id, unit]))
  }

  function unitLabel(unitId: string | undefined, unitsById: Map<string, Unit>): string {
    if (!unitId) return t('claims.commonAreas')
    const unit = unitsById.get(unitId)
    return unit ? t('claims.unitLabel', { number: unit.unit_number }) : '—'
  }

  return { authorLabel, statusVariant, indexUnits, unitLabel }
}
