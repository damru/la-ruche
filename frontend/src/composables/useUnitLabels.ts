import { useI18n } from 'vue-i18n'
import type { Unit } from '@/types/domain'

export function useUnitLabels() {
  const { t } = useI18n()

  function indexUnits(units: Unit[]): Map<string, Unit> {
    return new Map(units.map(unit => [unit.id, unit]))
  }

  function unitLabel(unitId: string | undefined, unitsById: Map<string, Unit>): string {
    if (!unitId) return t('claims.commonAreas')
    const unit = unitsById.get(unitId)
    return unit ? t('claims.unitLabel', { number: unit.unit_number }) : '—'
  }

  return { indexUnits, unitLabel }
}
