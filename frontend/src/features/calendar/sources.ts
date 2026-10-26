// Shared calendar source constants — single source of truth for the feature
// (badge colors, filter order). Consumed by CalendarGrid, CalendarAgendaMode
// and CalendarFilterBar.

import type { CalendarSource } from '@/types/domain'

export const CALENDAR_SOURCES: readonly CalendarSource[] = [
  'meeting',
  'contract',
  'task',
  'claim',
  'maintenance',
  'manual',
]

export const SOURCE_BADGE_VARIANT: Record<CalendarSource, 'primary' | 'accent' | 'warn' | 'err' | 'default' | 'valid'> = {
  meeting: 'primary',
  contract: 'warn',
  task: 'valid',
  claim: 'err',
  maintenance: 'accent',
  manual: 'default',
}
