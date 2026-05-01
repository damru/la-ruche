import type { Document } from '@/types/domain'

type Translate = (key: string, params?: Record<string, string | number>) => string

export function buildDocumentDeleteMessage(
  document: Document | undefined,
  t: Translate,
): string {
  if (!document) return t('documents.confirmDeleteMessage', { name: '' })

  const linkedContractNames = document.linked_contract_names ?? []
  const linkedReportTitles = document.linked_report_titles ?? []
  const linkedClaimTitles = document.linked_claim_titles ?? []
  const references = [
    ...linkedContractNames.map(name => t('documents.referenceContract', { name })),
    ...linkedReportTitles.map(title => t('documents.referenceReport', { title })),
    ...linkedClaimTitles.map(title => t('documents.referenceClaim', { title })),
  ]

  if (references.length === 0) {
    return t('documents.confirmDeleteMessage', { name: document.original_name })
  }

  return t('documents.confirmDeleteLinkedMessage', {
    name: document.original_name,
    links: references.join(', '),
  })
}
