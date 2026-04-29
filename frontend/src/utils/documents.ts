import type { Document } from '@/types/domain'

type Translate = (key: string, params?: Record<string, string | number>) => string

export function buildDocumentDeleteMessage(
  document: Document | undefined,
  t: Translate,
): string {
  if (!document) return t('documents.confirmDeleteMessage', { name: '' })

  const linkedContractNames = document.linked_contract_names ?? []
  const linkedReportCount = document.linked_report_count ?? 0

  if (linkedContractNames.length === 0 && linkedReportCount === 0) {
    return t('documents.confirmDeleteMessage', { name: document.original_name })
  }

  if (linkedContractNames.length > 0 && linkedReportCount === 0) {
    return t('documents.confirmDeleteLinkedMessage', {
      name: document.original_name,
      links: linkedContractNames.join(', '),
    })
  }

  if (linkedContractNames.length === 0 && linkedReportCount > 0) {
    return t('documents.confirmDeleteLinkedReportMessage', {
      name: document.original_name,
      count: linkedReportCount,
    })
  }

  // linked to both contracts and reports
  return t('documents.confirmDeleteLinkedBothMessage', {
    name: document.original_name,
    links: linkedContractNames.join(', '),
    count: linkedReportCount,
  })
}
