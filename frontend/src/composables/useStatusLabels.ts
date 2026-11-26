import {
  REPORT_STATUS_CLOSED,
  REPORT_STATUS_IN_PROGRESS,
  REPORT_STATUS_OPEN,
  TASK_PRIORITY_HIGH,
  TASK_PRIORITY_LOW,
  TASK_PRIORITY_NORMAL,
  TASK_PRIORITY_URGENT,
  TASK_STATUS_AWAITING_QUOTE,
  TASK_STATUS_AWAITING_VALIDATION,
  TASK_STATUS_AWAITING_VENDOR,
  TASK_STATUS_CLOSED,
  TASK_STATUS_IN_PROGRESS,
  TASK_STATUS_OPEN,
  TASK_STATUS_QUOTE_RECEIVED,
} from '@/utils/constants'

type BadgeVariant = 'valid' | 'open' | 'pending' | 'closed' | 'default'

export function reportStatusVariant(status: string): BadgeVariant {
  if (status === REPORT_STATUS_OPEN) return 'pending'
  if (status === REPORT_STATUS_IN_PROGRESS) return 'open'
  if (status === REPORT_STATUS_CLOSED) return 'closed'
  return 'default'
}

export function taskStatusVariant(status: string): BadgeVariant {
  if (status === TASK_STATUS_OPEN) return 'valid'
  if (status === TASK_STATUS_IN_PROGRESS) return 'open'
  if (status === TASK_STATUS_AWAITING_QUOTE || status === TASK_STATUS_QUOTE_RECEIVED) return 'pending'
  if (status === TASK_STATUS_AWAITING_VENDOR) return 'default'
  if (status === TASK_STATUS_AWAITING_VALIDATION) return 'pending'
  if (status === TASK_STATUS_CLOSED) return 'closed'
  return 'default'
}

export function taskPriorityVariant(priority: string): 'err' | 'pending' | 'primary' | 'valid' | 'default' {
  if (priority === TASK_PRIORITY_URGENT) return 'err'
  if (priority === TASK_PRIORITY_HIGH) return 'pending'
  if (priority === TASK_PRIORITY_NORMAL) return 'primary'
  if (priority === TASK_PRIORITY_LOW) return 'valid'
  return 'default'
}
